//! Reference-counted byte buffers.

use std::path::Path;
use std::sync::Arc;

use harfbuzz_sys as sys;

use crate::error::{Error, Result};
use crate::object::{HarfBuzzObject, IntoShared, Shared, ThreadSafeWhenImmutable, harfbuzz_object};

harfbuzz_object! {
    /// A chunk of bytes with a lifetime HarfBuzz can track.
    ///
    /// Blobs are how font data gets into HarfBuzz. Rather than take a pointer
    /// and hope it outlives the [`Face`](crate::Face) built from it, HarfBuzz
    /// wraps the bytes in a reference-counted object and releases them through
    /// a callback when the last reference goes away.
    ///
    /// The safe constructors here keep that callback under Rust's control:
    /// [`Blob::from_bytes`] hands ownership of a Rust allocation to HarfBuzz
    /// and frees it with the matching Rust deallocator, so there is no way to
    /// pair the wrong allocator with the wrong free.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use harfbuzz_rs::Blob;
    ///
    /// let blob = Blob::from_file("font.ttf")?;
    /// assert!(!blob.is_empty());
    /// # Ok::<(), harfbuzz_rs::Error>(())
    /// ```
    Blob,
    raw: sys::hb_blob_t,
    reference: sys::hb_blob_reference,
    destroy: sys::hb_blob_destroy,
}

// SAFETY: a blob's bytes never change once created — the safe constructors
// here only ever build read-only blobs — and HarfBuzz's reference counting is
// atomic. Concurrent readers of a frozen blob therefore cannot race.
unsafe impl ThreadSafeWhenImmutable for Blob {}

impl Blob {
    /// Wraps bytes that Rust owns, transferring them to HarfBuzz.
    ///
    /// The data is *not* copied. HarfBuzz holds the allocation until the last
    /// reference to the blob is dropped, then hands it back to Rust to free.
    ///
    /// Accepts anything that can become an `Arc<[u8]>`, which covers `Vec<u8>`,
    /// `Box<[u8]>`, and an `Arc<[u8]>` you already have.
    pub fn from_bytes(bytes: impl Into<Arc<[u8]>>) -> Result<Self> {
        let bytes: Arc<[u8]> = bytes.into();
        let length = bytes.len();

        // The bytes must stay put for as long as the blob lives, so hand
        // HarfBuzz an owning handle to reclaim in the destroy callback.
        //
        // `Arc<[u8]>` is a *fat* pointer — address plus length — and a
        // `*mut c_void` can only carry the address, so `Arc::into_raw` cannot
        // survive the round trip through C. Boxing the `Arc` gives a thin
        // pointer to a fat one, which can.
        let data = bytes.as_ptr();
        let owner = Box::into_raw(Box::new(bytes));

        /// Reclaims the `Arc` boxed above. HarfBuzz calls this once, when the
        /// blob's last reference is released.
        unsafe extern "C" fn release(user_data: *mut std::ffi::c_void) {
            // SAFETY: `user_data` is the pointer produced by `Box::into_raw`
            // for this blob and nothing else, HarfBuzz invokes this exactly
            // once, and the box has not been reclaimed before now. Rebuilding
            // and dropping it releases the `Arc`'s strong reference.
            drop(unsafe { Box::from_raw(user_data as *mut Arc<[u8]>) });
        }

        let user_data = owner as *mut std::ffi::c_void;

        // SAFETY: `data` points at `length` initialised bytes owned by the
        // leaked `Arc`, which stays alive until `release` runs. READONLY
        // promises HarfBuzz will not write through the pointer, which is what
        // makes sharing an `Arc` sound. `release` is a valid callback for
        // `user_data`.
        let raw = unsafe {
            sys::hb_blob_create_or_fail(
                data.cast(),
                length as std::ffi::c_uint,
                sys::HB_MEMORY_MODE_READONLY,
                user_data,
                Some(release),
            )
        };

        if raw.is_null() {
            // The blob was not created, so `release` will never run and the
            // leaked reference is ours to reclaim.
            //
            // SAFETY: `owner` came from `Box::into_raw` and has not been
            // reclaimed, because the only other consumer would have been the
            // blob that failed to exist.
            drop(unsafe { Box::from_raw(owner) });
            return Err(Error::AllocationFailed);
        }

        // SAFETY: `raw` is non-null and carries the reference the constructor
        // created, which this wrapper now owns.
        Ok(unsafe { Self::from_raw(raw) })
    }

    /// Reads a whole file into a blob.
    ///
    /// HarfBuzz memory-maps the file where the platform allows it, so this is
    /// cheaper than reading the bytes yourself for large fonts.
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        let c_path = std::ffi::CString::new(path.as_os_str().as_encoded_bytes())
            .map_err(|_| Error::FontLoadFailed)?;

        // SAFETY: `c_path` is a live NUL-terminated string for the duration of
        // the call. HarfBuzz copies what it needs and does not retain it.
        let raw = unsafe { sys::hb_blob_create_from_file_or_fail(c_path.as_ptr()) };

        if raw.is_null() {
            return Err(Error::FontLoadFailed);
        }

        // SAFETY: `raw` is non-null and carries the reference the constructor
        // created.
        Ok(unsafe { Self::from_raw(raw) })
    }

    /// The bytes the blob wraps.
    pub fn as_bytes(&self) -> &[u8] {
        let mut length = 0;

        // SAFETY: `self` owns a live blob, and `length` is a live local for
        // HarfBuzz to write through.
        let data = unsafe { sys::hb_blob_get_data(self.as_raw(), &mut length) };

        if data.is_null() || length == 0 {
            return &[];
        }

        // SAFETY: HarfBuzz reported `length` readable bytes at `data`. They
        // belong to the blob, so tying the slice to `&self` keeps them alive
        // for exactly as long as the borrow. The blob is read-only, so no
        // aliasing `&mut` can exist.
        unsafe { std::slice::from_raw_parts(data.cast(), length as usize) }
    }

    /// How many bytes the blob holds.
    pub fn len(&self) -> usize {
        // SAFETY: `self` owns a live blob.
        unsafe { sys::hb_blob_get_length(self.as_raw()) as usize }
    }

    /// Whether the blob holds no bytes.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// How many faces the data contains.
    ///
    /// A plain `.ttf` has one; a `.ttc` collection has several.
    pub fn face_count(&self) -> u32 {
        // SAFETY: `self` owns a live blob.
        unsafe { sys::hb_face_count(self.as_raw()) }
    }
}

impl IntoShared for Blob {
    fn into_shared(self) -> Shared<Self> {
        // SAFETY: `self` owns a live blob.
        unsafe { sys::hb_blob_make_immutable(self.as_raw()) };

        // SAFETY: the blob was just frozen, which is what `from_immutable`
        // requires.
        unsafe { Shared::from_immutable(self) }
    }
}

impl std::fmt::Debug for Blob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Blob").field("len", &self.len()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wraps_rust_owned_bytes_without_copying() {
        let blob = Blob::from_bytes(vec![1u8, 2, 3, 4]).unwrap();

        assert_eq!(blob.len(), 4);
        assert_eq!(blob.as_bytes(), &[1, 2, 3, 4]);
        assert!(!blob.is_empty());
    }

    #[test]
    fn frees_the_rust_allocation_when_the_last_reference_goes() {
        let bytes: Arc<[u8]> = vec![0u8; 64].into();
        let watch = Arc::clone(&bytes);
        assert_eq!(Arc::strong_count(&watch), 2);

        // The `Arc` moves into the box handed to HarfBuzz, so the count stays
        // at two: ours, and the one the blob now owns.
        let blob = Blob::from_bytes(bytes).unwrap();
        assert_eq!(Arc::strong_count(&watch), 2);

        drop(blob);
        // Dropping the blob ran the destroy callback, which reclaimed it.
        assert_eq!(Arc::strong_count(&watch), 1);
    }

    #[test]
    fn sharing_keeps_the_data_alive_until_every_handle_is_gone() {
        let bytes: Arc<[u8]> = vec![7u8; 8].into();
        let watch = Arc::clone(&bytes);

        let shared = Blob::from_bytes(bytes).unwrap().into_shared();
        let clone = shared.clone();
        assert_eq!(clone.as_bytes(), &[7u8; 8]);

        drop(shared);
        assert_eq!(clone.as_bytes(), &[7u8; 8], "still alive via the clone");

        drop(clone);
        assert_eq!(Arc::strong_count(&watch), 1);
    }

    #[test]
    fn reports_a_missing_file_as_an_error() {
        assert!(matches!(
            Blob::from_file("/nonexistent/font.ttf"),
            Err(Error::FontLoadFailed)
        ));
    }

    #[test]
    fn an_empty_input_still_produces_a_usable_blob() {
        let blob = Blob::from_bytes(Vec::new()).unwrap();

        assert!(blob.is_empty());
        assert_eq!(blob.as_bytes(), &[]);
    }
}
