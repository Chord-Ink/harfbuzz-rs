//! Font face objects — `hb-face.h`.
//!
//! A face is one typeface picked out of a binary font file; fonts are made
//! from faces.

use core::ffi::{c_char, c_uint, c_void};

use crate::{
    hb_blob_t, hb_bool_t, hb_codepoint_t, hb_destroy_func_t, hb_map_t, hb_set_t, hb_tag_t,
    hb_user_data_key_t,
};

opaque_handle! {
    /// Data type for holding font faces.
    ///
    /// A face represents a single face within a binary font file — a single
    /// member of a font family. Faces are typically built from a binary blob
    /// plus a face index, and are in turn used to create fonts.
    ///
    /// The face index selects one face out of a blob that holds several, as
    /// TrueType Collection (`.ttc`) and Mac `dfont` files do. A blob holding a
    /// regular and a bold face yields two face objects, one per index.
    ///
    /// Faces are reference counted: see [`hb_face_reference`] and
    /// [`hb_face_destroy`].
    hb_face_t
}

/// Callback that hands HarfBuzz the contents of one font table.
///
/// Used with [`hb_face_create_for_tables`]. `tag` is the table to reference;
/// the special value `HB_TAG_NONE` asks for the blob of the whole face. If
/// referencing the face blob is not possible, set an
/// [`hb_get_table_tags_func_t`] on the face instead, so that
/// [`hb_face_reference_blob`] can assemble a face blob out of the individual
/// table blobs.
///
/// Returns a reference to the table within the face — ownership of which
/// transfers to HarfBuzz — or null if the table is absent or cannot be
/// referenced.
///
/// Since HarfBuzz 0.9.2.
pub type hb_reference_table_func_t = Option<
    unsafe extern "C" fn(
        face: *mut hb_face_t,
        tag: hb_tag_t,
        user_data: *mut c_void,
    ) -> *mut hb_blob_t,
>;

/// Callback that enumerates the table tags present in a face.
///
/// Used with [`hb_face_set_get_table_tags_func`] and called by
/// [`hb_face_get_table_tags`]. `start_offset` is the index of the first tag to
/// retrieve. `table_count` is an in/out parameter: on input the capacity of
/// `table_tags`, on output the number of tags actually written (possibly zero).
/// `table_tags` is the output array, and may be null.
///
/// Returns the total number of tables in the face, or zero if the tables
/// cannot be listed.
///
/// Since HarfBuzz 10.0.0.
pub type hb_get_table_tags_func_t = Option<
    unsafe extern "C" fn(
        face: *const hb_face_t,
        start_offset: c_uint,
        table_count: *mut c_uint,
        table_tags: *mut hb_tag_t,
        user_data: *mut c_void,
    ) -> c_uint,
>;

unsafe extern "C" {
    /// Fetches the number of faces in a blob.
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_face_count(blob: *mut hb_blob_t) -> c_uint;

    /// Constructs a new face from a blob and a face index into that blob.
    ///
    /// The index selects a face inside collection formats such as TTC and
    /// dfont, and is zero-based. If the blob is not a collection the index is
    /// ignored; otherwise only its low 16 bits are used. The unmodified value
    /// is still readable through [`hb_face_get_index`].
    ///
    /// The high 16 bits of `index`, when non-zero, are consumed by
    /// `hb_font_create` to select a named instance of a variable font.
    ///
    /// Never returns null: on failure it returns the singleton empty face.
    /// Use [`hb_face_create_or_fail`] to detect that case.
    ///
    /// The caller owns the returned face and must release it with
    /// [`hb_face_destroy`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_create(blob: *mut hb_blob_t, index: c_uint) -> *mut hb_face_t;

    /// Like [`hb_face_create`], but returns null when the blob holds no usable
    /// font face at `index`.
    ///
    /// The caller owns a non-null result and must release it with
    /// [`hb_face_destroy`].
    ///
    /// Since HarfBuzz 10.1.0.
    pub fn hb_face_create_or_fail(blob: *mut hb_blob_t, index: c_uint) -> *mut hb_face_t;

    /// Creates a face from a blob using a named face loader.
    ///
    /// Pass null or the empty string for `loader_name` to use the first
    /// available loader. Loaders differ in what they accept — the FreeType
    /// (`"ft"`) loader can read WOFF and WOFF2 when FreeType is built with
    /// those features, while the OpenType (`"ot"`) loader cannot.
    ///
    /// Returns null if the loader fails to load the face. The caller owns a
    /// non-null result and must release it with [`hb_face_destroy`].
    ///
    /// Since HarfBuzz 11.0.0.
    pub fn hb_face_create_or_fail_using(
        blob: *mut hb_blob_t,
        index: c_uint,
        loader_name: *const c_char,
    ) -> *mut hb_face_t;

    /// Creates a face directly from a font file on disk.
    ///
    /// A thin wrapper around `hb_blob_create_from_file_or_fail` followed by
    /// [`hb_face_create_or_fail`].
    ///
    /// Returns null if the file cannot be read or holds no face at `index`.
    /// The caller owns a non-null result and must release it with
    /// [`hb_face_destroy`].
    ///
    /// Since HarfBuzz 10.1.0.
    pub fn hb_face_create_from_file_or_fail(
        file_name: *const c_char,
        index: c_uint,
    ) -> *mut hb_face_t;

    /// Creates a face from a font file on disk using a named face loader.
    ///
    /// Pass null or the empty string for `loader_name` to use the first
    /// available loader. See [`hb_face_create_or_fail_using`] for why the
    /// choice of loader matters.
    ///
    /// Returns null if the file cannot be read or the loader fails. The caller
    /// owns a non-null result and must release it with [`hb_face_destroy`].
    ///
    /// Since HarfBuzz 11.0.0.
    pub fn hb_face_create_from_file_or_fail_using(
        file_name: *const c_char,
        index: c_uint,
        loader_name: *const c_char,
    ) -> *mut hb_face_t;

    /// Retrieves the face loaders supported by this build of HarfBuzz.
    ///
    /// Returns a null-terminated array of constant strings, owned by HarfBuzz.
    /// Do not modify or free it.
    ///
    /// Since HarfBuzz 11.0.0.
    pub fn hb_face_list_loaders() -> *mut *const c_char;

    /// Creates a face that fetches its tables one at a time through a callback.
    ///
    /// A variant of [`hb_face_create`] for cases where supplying individual
    /// tables is more convenient than supplying whole font data. Note that
    /// [`hb_face_get_table_tags`] does not work on a face built this way
    /// unless you also install a callback with
    /// [`hb_face_set_get_table_tags_func`].
    ///
    /// `destroy` is called on `user_data` when the face no longer needs it.
    ///
    /// The caller owns the returned face and must release it with
    /// [`hb_face_destroy`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_create_for_tables(
        reference_table_func: hb_reference_table_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    ) -> *mut hb_face_t;

    /// Fetches the singleton empty face.
    ///
    /// The empty face is immutable and has no tables. The returned pointer
    /// carries a reference, so it may be passed to [`hb_face_destroy`] like any
    /// other face.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_get_empty() -> *mut hb_face_t;

    /// Increases the reference count on a face and returns the same face.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_reference(face: *mut hb_face_t) -> *mut hb_face_t;

    /// Decreases the reference count on a face, destroying it and freeing all
    /// of its memory once the count reaches zero.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_destroy(face: *mut hb_face_t);

    /// Attaches a user-data key/value pair to a face.
    ///
    /// `destroy` is called on `data` when the face is destroyed or the entry is
    /// replaced. `replace` decides whether an existing entry under the same key
    /// is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_set_user_data(
        face: *mut hb_face_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to a face under the given key.
    ///
    /// The face retains ownership of the returned pointer.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_get_user_data(
        face: *const hb_face_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Makes a face immutable, so that later setters on it are ignored.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_make_immutable(face: *mut hb_face_t);

    /// Tests whether a face is immutable.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_is_immutable(face: *mut hb_face_t) -> hb_bool_t;

    /// Fetches a reference to one table within a face.
    ///
    /// Returns the empty blob when the table is missing or referencing table
    /// data is not possible, so the result is never null. The caller owns the
    /// returned blob and must release it with `hb_blob_destroy`.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_reference_table(face: *const hb_face_t, tag: hb_tag_t) -> *mut hb_blob_t;

    /// Fetches the binary blob holding the whole face.
    ///
    /// When the face data cannot be referenced directly, HarfBuzz assembles a
    /// blob from the individual table blobs — provided
    /// [`hb_face_get_table_tags`] works on this face. Failing that, the empty
    /// blob is returned, so the result is never null.
    ///
    /// The caller owns the returned blob and must release it with
    /// `hb_blob_destroy`.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_reference_blob(face: *mut hb_face_t) -> *mut hb_blob_t;

    /// Assigns a face index to a face. Ignored if the face is immutable.
    ///
    /// This changes nothing about the face itself — only the value reported by
    /// [`hb_face_get_index`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_set_index(face: *mut hb_face_t, index: c_uint);

    /// Fetches the face index of a face. Indices within a collection are
    /// zero-based.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_get_index(face: *const hb_face_t) -> c_uint;

    /// Sets the units-per-em of a face. Needed only in rare circumstances.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_set_upem(face: *mut hb_face_t, upem: c_uint);

    /// Fetches the units-per-em (upem) of a face.
    ///
    /// Typical values are 1000 or 2048, though OpenType permits anything from
    /// 16 to 16,384.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_face_get_upem(face: *const hb_face_t) -> c_uint;

    /// Sets the glyph count of a face. Needed only in rare circumstances.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_face_set_glyph_count(face: *mut hb_face_t, glyph_count: c_uint);

    /// Fetches the glyph count of a face.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_face_get_glyph_count(face: *const hb_face_t) -> c_uint;

    /// Installs the table-tag-enumerating callback on a face.
    ///
    /// `destroy` is called on `user_data` when the callback is no longer
    /// needed.
    ///
    /// Since HarfBuzz 10.0.0.
    pub fn hb_face_set_get_table_tags_func(
        face: *mut hb_face_t,
        func: hb_get_table_tags_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Fetches a slice of the table tags in a face, starting at
    /// `start_offset`.
    ///
    /// `table_count` is an in/out parameter: on input the capacity of
    /// `table_tags`, on output the number actually written. `table_tags` may
    /// be null.
    ///
    /// Returns the total number of tables in the face, or zero if the tables
    /// cannot be listed.
    ///
    /// Since HarfBuzz 1.6.0.
    pub fn hb_face_get_table_tags(
        face: *const hb_face_t,
        start_offset: c_uint,
        table_count: *mut c_uint,
        table_tags: *mut hb_tag_t,
    ) -> c_uint;

    /// Collects every Unicode character covered by a face into `out`.
    ///
    /// Since HarfBuzz 1.9.0.
    pub fn hb_face_collect_unicodes(face: *mut hb_face_t, out: *mut hb_set_t);

    /// Collects a face's mapping from Unicode characters to nominal glyphs into
    /// `mapping`, and optionally the covered characters into `unicodes`.
    ///
    /// `unicodes` may be null.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_face_collect_nominal_glyph_mapping(
        face: *mut hb_face_t,
        mapping: *mut hb_map_t,
        unicodes: *mut hb_set_t,
    );

    /// Collects every Unicode variation selector covered by a face into `out`.
    ///
    /// Since HarfBuzz 1.9.0.
    pub fn hb_face_collect_variation_selectors(face: *mut hb_face_t, out: *mut hb_set_t);

    /// Collects every Unicode character a face covers for the given variation
    /// selector into `out`.
    ///
    /// Since HarfBuzz 1.9.0.
    pub fn hb_face_collect_variation_unicodes(
        face: *mut hb_face_t,
        variation_selector: hb_codepoint_t,
        out: *mut hb_set_t,
    );

    /// Creates an empty face for assembling a font table by table.
    ///
    /// Add tables with [`hb_face_builder_add_table`], then compile the result
    /// into a binary font file by calling [`hb_face_reference_blob`].
    ///
    /// The caller owns the returned face and must release it with
    /// [`hb_face_destroy`].
    ///
    /// Since HarfBuzz 1.9.0.
    pub fn hb_face_builder_create() -> *mut hb_face_t;

    /// Adds the table `tag`, with data taken from `blob`, to a builder face.
    ///
    /// `face` must have come from [`hb_face_builder_create`]; the call fails
    /// and returns false otherwise. The builder takes its own reference on
    /// `blob`.
    ///
    /// Since HarfBuzz 1.9.0.
    pub fn hb_face_builder_add_table(
        face: *mut hb_face_t,
        tag: hb_tag_t,
        blob: *mut hb_blob_t,
    ) -> hb_bool_t;

    /// Sets the table ordering used when a builder face is serialized.
    ///
    /// `tags` is an array terminated by `HB_TAG_NONE`. Tables not named in it
    /// are written after those that are, in the default sort order.
    ///
    /// Since HarfBuzz 5.3.0.
    pub fn hb_face_builder_sort_tables(face: *mut hb_face_t, tags: *const hb_tag_t);
}
