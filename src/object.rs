//! Ownership and sharing for HarfBuzz's reference-counted objects.
//!
//! # How HarfBuzz manages lifetimes
//!
//! Every HarfBuzz object — a blob, a face, a font, a buffer — is reference
//! counted. `hb_x_create()` hands back an object with a count of one,
//! `hb_x_reference()` raises it, and `hb_x_destroy()` lowers it, freeing the
//! object when it hits zero. The counting itself is atomic, so it is safe to
//! reference and destroy the same object from several threads even when the
//! object's *contents* are not safe to share.
//!
//! Most object types also have a second phase to their life. HarfBuzz's own
//! documentation describes the pattern as: create the object, make a few
//! `set_*` calls, then use it without further modification. An object can be
//! marked immutable with `hb_x_make_immutable()`, after which it is safe to use
//! from several threads at once — and after which setters **fail silently**.
//! There is no way back; an immutable object can only be duplicated.
//!
//! # How this crate models that
//!
//! Silent failure is a poor fit for Rust, so this module turns HarfBuzz's
//! runtime rule into a compile-time one:
//!
//! * An owned wrapper — [`Face`](crate::Face), [`Font`](crate::Font), and so on
//!   — holds the only handle to its object. Its setters take `&mut self`, so
//!   the borrow checker guarantees nothing else can observe a half-configured
//!   object. These types are [`Send`] but not [`Sync`].
//!
//! * [`Shared<T>`] holds a counted reference to an object that has been made
//!   immutable. It is [`Clone`], and it dereferences to `&T`, which reaches
//!   exactly the accessors and none of the setters. Types whose contents
//!   HarfBuzz documents as thread-safe once frozen are additionally [`Send`]
//!   and [`Sync`] through this wrapper.
//!
//! The one-way trip from owned to shared is [`IntoShared::into_shared`], which
//! freezes the object and hands back the shared handle. That mirrors HarfBuzz's
//! own "no make-mutable" rule exactly: once you have a [`Shared<T>`], the only
//! route back to a mutable object is to copy it.

use core::marker::PhantomData;
use core::ops::Deref;
use core::ptr::NonNull;

/// A reference-counted HarfBuzz object.
///
/// # Safety
///
/// An implementor promises that:
///
/// * `Raw` is the C type this wrapper owns, and the wrapper's only meaningful
///   state is one non-null pointer to it.
/// * The wrapper owns exactly one reference for as long as it lives, and
///   releases it exactly once when dropped.
/// * `reference_raw` and `destroy_raw` call the matching HarfBuzz functions for
///   `Raw`, and nothing else.
/// * `from_raw` is only ever handed a pointer that carries a reference the
///   caller is giving up.
pub unsafe trait HarfBuzzObject: Sized {
    /// The opaque C type behind this wrapper, such as `hb_face_t`.
    type Raw;

    /// Borrows the underlying pointer without affecting the reference count.
    fn as_raw(&self) -> *mut Self::Raw;

    /// Takes ownership of a pointer that already carries a reference.
    ///
    /// # Safety
    ///
    /// `raw` must be non-null, must point at a live object of the right type,
    /// and must come with a reference that the caller is transferring. Calling
    /// this twice with the same pointer causes a double free.
    unsafe fn from_raw(raw: *mut Self::Raw) -> Self;

    /// Adds a reference and returns the same pointer.
    ///
    /// # Safety
    ///
    /// `raw` must point at a live object of the right type.
    unsafe fn reference_raw(raw: *mut Self::Raw) -> *mut Self::Raw;

    /// Releases a reference, destroying the object if it was the last one.
    ///
    /// # Safety
    ///
    /// `raw` must carry a reference that the caller is giving up.
    unsafe fn destroy_raw(raw: *mut Self::Raw);
}

/// A HarfBuzz object whose contents are safe to read from several threads once
/// it has been made immutable.
///
/// This is what lets [`Shared<T>`] be [`Send`] and [`Sync`] for some types and
/// not others. Buffers, for instance, are mutated throughout shaping and never
/// qualify.
///
/// # Safety
///
/// The implementor promises that after `hb_x_make_immutable()`, concurrent
/// calls to this type's `&self` methods on the same object are free of data
/// races. In practice this means HarfBuzz either does not mutate the object at
/// all in those paths, or does so through atomics.
pub unsafe trait ThreadSafeWhenImmutable: HarfBuzzObject {}

/// An object that can be frozen and then shared.
pub trait IntoShared: HarfBuzzObject {
    /// Marks the object immutable and wraps it for sharing.
    ///
    /// This is a one-way door: HarfBuzz has no "make mutable" operation, and
    /// [`Shared<T>`] deliberately offers no way back. To keep editing, clone
    /// the object before freezing it.
    fn into_shared(self) -> Shared<Self>;
}

/// A counted reference to an immutable HarfBuzz object.
///
/// Cloning is cheap — it bumps HarfBuzz's own reference count rather than
/// copying anything — and dropping releases that reference. Because `Shared<T>`
/// only ever hands out `&T`, none of the wrapped type's setters are reachable
/// through it, which is what makes concurrent use sound.
///
/// # Examples
///
/// ```no_run
/// # use harfbuzz_rs::{Face, IntoShared};
/// let face = Face::from_file("font.ttf", 0)?.into_shared();
///
/// // Cheap: this shares the same underlying hb_face_t.
/// let also_face = face.clone();
/// assert_eq!(face.glyph_count(), also_face.glyph_count());
/// # Ok::<(), harfbuzz_rs::Error>(())
/// ```
pub struct Shared<T: HarfBuzzObject> {
    /// Invariant: owns exactly one reference, and is never moved out of.
    object: T,
}

impl<T: HarfBuzzObject> Shared<T> {
    /// Wraps an already-frozen object.
    ///
    /// Prefer [`IntoShared::into_shared`], which does the freezing for you.
    ///
    /// # Safety
    ///
    /// `object` must already have been made immutable, or concurrent readers
    /// could observe it changing.
    pub unsafe fn from_immutable(object: T) -> Self {
        Self { object }
    }

    /// Borrows the underlying pointer without affecting the reference count.
    pub fn as_raw(&self) -> *mut T::Raw {
        self.object.as_raw()
    }

    /// Gives up ownership of the pointer, leaving the caller responsible for
    /// releasing its reference.
    pub fn into_raw(self) -> *mut T::Raw {
        let raw = self.object.as_raw();

        // SAFETY: `self.object` owns one reference, which is exactly what we
        // are handing to the caller. Forgetting the wrapper skips the `Drop`
        // that would otherwise release it, so the count stays correct.
        core::mem::forget(self);

        raw
    }
}

impl<T: HarfBuzzObject> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.object
    }
}

impl<T: HarfBuzzObject> Clone for Shared<T> {
    fn clone(&self) -> Self {
        // SAFETY: `self.object` holds a live object, so referencing it is
        // valid; the returned pointer carries the new reference that the
        // reconstructed wrapper takes ownership of.
        let raw = unsafe { T::reference_raw(self.object.as_raw()) };

        // SAFETY: `raw` is the pointer we just added a reference for, and this
        // is the only wrapper that will release it.
        Self {
            object: unsafe { T::from_raw(raw) },
        }
    }
}

// SAFETY: `Shared` is only constructed around an object that has been made
// immutable, and `ThreadSafeWhenImmutable` is the promise that such an object's
// read paths are race-free. Reference counting is atomic in HarfBuzz, so
// cloning and dropping from several threads is sound as well.
unsafe impl<T: HarfBuzzObject + ThreadSafeWhenImmutable> Send for Shared<T> {}

// SAFETY: as above. `Shared` hands out only `&T`, which reaches no setter.
unsafe impl<T: HarfBuzzObject + ThreadSafeWhenImmutable> Sync for Shared<T> {}

impl<T: HarfBuzzObject + core::fmt::Debug> core::fmt::Debug for Shared<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("Shared").field(&self.object).finish()
    }
}

/// The pointer every owned wrapper is built around.
///
/// This exists so the wrappers do not each repeat the same null-checking and
/// `PhantomData` boilerplate. It carries no `Drop` of its own: releasing the
/// reference is the wrapper's job, because only the wrapper knows which
/// `hb_x_destroy` to call.
#[repr(transparent)]
pub(crate) struct RawHandle<T> {
    ptr: NonNull<T>,
    /// Ties the handle to `T` without claiming to own a `T` that Rust could
    /// drop, and keeps the type `!Send`/`!Sync` until a wrapper opts in.
    _marker: PhantomData<*mut T>,
}

impl<T> RawHandle<T> {
    /// Wraps a pointer returned by a HarfBuzz constructor.
    ///
    /// HarfBuzz constructors never return null — on allocation failure they
    /// return an inert "empty" singleton instead — so this only rejects
    /// genuinely broken input.
    ///
    /// # Safety
    ///
    /// `ptr` must carry a reference that the caller is transferring.
    pub(crate) unsafe fn new(ptr: *mut T) -> Option<Self> {
        NonNull::new(ptr).map(|ptr| Self {
            ptr,
            _marker: PhantomData,
        })
    }

    pub(crate) fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }
}

/// Declare an owned wrapper around a reference-counted HarfBuzz object.
///
/// Generates the struct, its [`HarfBuzzObject`] implementation, `Drop`, and
/// `Send`. Everything else — constructors, accessors, setters — is written by
/// hand in the module that owns the type, because that is where the interesting
/// decisions live.
macro_rules! harfbuzz_object {
    (
        $(#[$meta:meta])*
        $name:ident,
        raw: $raw:ty,
        reference: $reference:path,
        destroy: $destroy:path
        $(,)?
    ) => {
        $(#[$meta])*
        #[repr(transparent)]
        pub struct $name {
            raw: $crate::object::RawHandle<$raw>,
        }

        // SAFETY: the wrapper holds one reference for its whole life and
        // releases it once in `Drop`. `reference_raw` and `destroy_raw`
        // forward to the matching HarfBuzz entry points and nothing else.
        unsafe impl $crate::object::HarfBuzzObject for $name {
            type Raw = $raw;

            fn as_raw(&self) -> *mut $raw {
                self.raw.as_ptr()
            }

            unsafe fn from_raw(raw: *mut $raw) -> Self {
                // SAFETY: the caller promises `raw` is live and transfers a
                // reference along with it.
                let handle = unsafe { $crate::object::RawHandle::new(raw) };
                Self {
                    raw: handle.expect(concat!(
                        stringify!($name),
                        "::from_raw was given a null pointer"
                    )),
                }
            }

            unsafe fn reference_raw(raw: *mut $raw) -> *mut $raw {
                // SAFETY: the caller promises `raw` points at a live object.
                unsafe { $reference(raw) }
            }

            unsafe fn destroy_raw(raw: *mut $raw) {
                // SAFETY: the caller promises `raw` carries a reference they
                // are giving up.
                unsafe { $destroy(raw) }
            }
        }

        impl Drop for $name {
            fn drop(&mut self) {
                // SAFETY: this wrapper owns exactly one reference, `Drop` runs
                // at most once, and the pointer is still live because we hold
                // that reference.
                unsafe { $destroy(self.raw.as_ptr()) }
            }
        }

        // SAFETY: an owned wrapper is the only handle to its object, so moving
        // it to another thread cannot race with anything. `Sync` is
        // deliberately NOT implemented: `&self` accessors may populate internal
        // caches, and only a frozen object shared through `Shared` is
        // documented to tolerate that concurrently.
        unsafe impl Send for $name {}
    };
}

pub(crate) use harfbuzz_object;
