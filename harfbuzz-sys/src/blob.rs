//! Reference-counted containers for binary data — `hb-blob.h`.
//!
//! A blob wraps a chunk of bytes — usually a font file — and manages its
//! lifetime as it travels between the client program and HarfBuzz.

use core::ffi::{c_char, c_int, c_uint, c_void};

use crate::{hb_bool_t, hb_destroy_func_t, hb_user_data_key_t};

/// The memory modes available to client programs when wrapping data in a blob.
///
/// The mode tells HarfBuzz how it may treat the memory it has been handed:
///
/// * A HarfBuzz client must never modify memory it has passed to HarfBuzz in a
///   blob. If there is any chance of that happening, use
///   [`HB_MEMORY_MODE_DUPLICATE`] so that HarfBuzz takes its own copy
///   immediately.
/// * Otherwise use [`HB_MEMORY_MODE_READONLY`], unless you really, really know
///   what you are doing.
/// * [`HB_MEMORY_MODE_WRITABLE`] is appropriate only if you made a copy of the
///   data solely to hand it to HarfBuzz, and are doing so exactly once — no
///   reuse.
/// * If the font is `mmap`ed it is acceptable to use
///   [`HB_MEMORY_MODE_READONLY_MAY_MAKE_WRITABLE`], but using that mode
///   correctly is very tricky. Prefer [`HB_MEMORY_MODE_READONLY`].
///
/// The C enumeration has no explicit sentinel and its largest enumerator is 3,
/// so it fits in an `int`.
pub type hb_memory_mode_t = c_int;

/// HarfBuzz immediately makes a copy of the data.
pub const HB_MEMORY_MODE_DUPLICATE: hb_memory_mode_t = 0;

/// The HarfBuzz client will never modify the data, and HarfBuzz will never
/// modify the data.
pub const HB_MEMORY_MODE_READONLY: hb_memory_mode_t = 1;

/// The HarfBuzz client made a copy of the data solely for HarfBuzz, so
/// HarfBuzz may modify the data.
pub const HB_MEMORY_MODE_WRITABLE: hb_memory_mode_t = 2;

/// The data is read-only, but HarfBuzz may attempt to make it writable in
/// place — by re-protecting the underlying pages, for instance.
pub const HB_MEMORY_MODE_READONLY_MAY_MAKE_WRITABLE: hb_memory_mode_t = 3;

opaque_handle! {
    /// Data type for blobs.
    ///
    /// A blob wraps a chunk of binary data and facilitates its lifecycle
    /// management between a client program and HarfBuzz.
    hb_blob_t
}

unsafe extern "C" {
    /// Creates a new blob wrapping `data`.
    ///
    /// The `mode` parameter negotiates ownership and lifecycle of `data`.
    /// `destroy` — which may be null — is called with `user_data` once the data
    /// is no longer needed.
    ///
    /// Returns a new blob, or the singleton empty blob if something failed or
    /// if `length` is zero. Release it with [`hb_blob_destroy`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_blob_create(
        data: *const c_char,
        length: c_uint,
        mode: hb_memory_mode_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    ) -> *mut hb_blob_t;

    /// Creates a new blob wrapping `data`, reporting failure instead of
    /// substituting the empty blob.
    ///
    /// The `mode` parameter negotiates ownership and lifecycle of `data`.
    /// `destroy` — which may be null — is called with `user_data` once the data
    /// is no longer needed.
    ///
    /// Note that this function returns a freshly allocated empty blob even when
    /// `length` is zero, in contrast to [`hb_blob_create`], which returns the
    /// singleton empty blob from [`hb_blob_get_empty`] in that case.
    ///
    /// Returns a new blob, or null on failure. Release it with
    /// [`hb_blob_destroy`].
    ///
    /// Since HarfBuzz 2.8.2.
    pub fn hb_blob_create_or_fail(
        data: *const c_char,
        length: c_uint,
        mode: hb_memory_mode_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    ) -> *mut hb_blob_t;

    /// Creates a new blob containing the data from the specified binary font
    /// file.
    ///
    /// The filename is passed directly to the system on all platforms except
    /// Windows, where it is interpreted as UTF-8; only if it is not valid UTF-8
    /// is it interpreted according to the system code page.
    ///
    /// Returns a blob holding the contents of the file, or the singleton empty
    /// blob on failure. Release it with [`hb_blob_destroy`].
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_blob_create_from_file(file_name: *const c_char) -> *mut hb_blob_t;

    /// Creates a new blob containing the data from the specified file,
    /// reporting failure instead of substituting the empty blob.
    ///
    /// The filename is passed directly to the system on all platforms except
    /// Windows, where it is interpreted as UTF-8; only if it is not valid UTF-8
    /// is it interpreted according to the system code page.
    ///
    /// Returns a blob holding the contents of the file, or null on failure.
    /// Release it with [`hb_blob_destroy`].
    ///
    /// Since HarfBuzz 2.8.2.
    pub fn hb_blob_create_from_file_or_fail(file_name: *const c_char) -> *mut hb_blob_t;

    /// Returns a blob representing a range of `length` bytes starting at
    /// `offset` within `parent`.
    ///
    /// The sub-blob is always created with [`HB_MEMORY_MODE_READONLY`]: even if
    /// the parent blob is writable, the user of a sub-blob must not be able to
    /// modify the parent's data, since that data may be shared among several
    /// sub-blobs. The parent data is not expected to be modified, and doing so
    /// is undefined behaviour.
    ///
    /// Makes `parent` immutable.
    ///
    /// Returns a new blob, or the singleton empty blob if something failed, if
    /// `length` is zero, or if `offset` is beyond the end of the parent's data.
    /// Release it with [`hb_blob_destroy`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_blob_create_sub_blob(
        parent: *mut hb_blob_t,
        offset: c_uint,
        length: c_uint,
    ) -> *mut hb_blob_t;

    /// Makes a writable copy of `blob`.
    ///
    /// Returns the new blob, or null if allocation failed. Release it with
    /// [`hb_blob_destroy`].
    ///
    /// Since HarfBuzz 1.8.0.
    pub fn hb_blob_copy_writable_or_fail(blob: *mut hb_blob_t) -> *mut hb_blob_t;

    /// Returns the singleton empty blob.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_blob_get_empty() -> *mut hb_blob_t;

    /// Increases the reference count on `blob` and returns it.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_blob_reference(blob: *mut hb_blob_t) -> *mut hb_blob_t;

    /// Decreases the reference count on `blob`, destroying it and freeing all
    /// its memory when the count reaches zero.
    ///
    /// Destroying a blob may call the destroy callback the blob was created
    /// with, if it has not been called already.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_blob_destroy(blob: *mut hb_blob_t);

    /// Attaches a user-data key/data pair to the specified blob.
    ///
    /// `destroy` — which may be null — is called with `data` when the blob is
    /// destroyed or the value is replaced. `replace` decides whether existing
    /// data stored under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_blob_set_user_data(
        blob: *mut hb_blob_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to the blob under the specified key.
    ///
    /// Ownership stays with the blob; the caller must not free the returned
    /// pointer.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_blob_get_user_data(
        blob: *const hb_blob_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Makes a blob immutable.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_blob_make_immutable(blob: *mut hb_blob_t);

    /// Tests whether a blob is immutable.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_blob_is_immutable(blob: *mut hb_blob_t) -> hb_bool_t;

    /// Fetches the length of a blob's data, in bytes.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_blob_get_length(blob: *mut hb_blob_t) -> c_uint;

    /// Fetches the data from a blob.
    ///
    /// When `length` is non-null it receives the length in bytes of the data
    /// returned. The bytes belong to the blob and must not be freed by the
    /// caller; the pointer may be null.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_blob_get_data(blob: *mut hb_blob_t, length: *mut c_uint) -> *const c_char;

    /// Tries to make a blob's data writable — copying it if necessary — and
    /// returns a pointer to it.
    ///
    /// Fails if the blob has been made immutable, or if memory allocation
    /// fails.
    ///
    /// Returns the writable data, or null on failure; `length`, when non-null,
    /// receives the length in bytes and is set to zero on failure. The bytes
    /// belong to the blob and must not be freed by the caller.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_blob_get_data_writable(blob: *mut hb_blob_t, length: *mut c_uint) -> *mut c_char;
}
