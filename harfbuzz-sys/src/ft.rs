//! FreeType integration — `hb-ft.h`.
//!
//! FreeType is the font engine that most of the free-software stack uses to
//! load, scale, hint and rasterize fonts. This sub-library is the bridge
//! between it and HarfBuzz: it lets an `FT_Face` you already have supply the
//! font data HarfBuzz shapes with, and lets FreeType — rather than HarfBuzz's
//! own OpenType implementation — answer the per-glyph questions that shaping
//! asks.
//!
//! The sources behind these declarations are compiled only when this crate's
//! `freetype` feature is enabled, which also needs the system `freetype2`
//! library to be discoverable through `pkg-config`. That same feature gates
//! this module, so a declaration here can never refer to a symbol that was
//! left out of the archive. Because of the gating the module is exposed as
//! `harfbuzz_sys::ft` rather than being glob re-exported at the crate root.
//!
//! There are two independent things this header can do for you, and they are
//! easy to confuse:
//!
//! * **Face data.** The `hb_ft_face_create*` family builds an [`hb_face_t`]
//!   whose table data is read out of an `FT_Face`. This is useful mostly
//!   because FreeType can open containers HarfBuzz cannot, such as WOFF and
//!   WOFF2. Fonts made from such a face still use HarfBuzz's own native font
//!   implementation for metrics and outlines.
//! * **Font functions.** The `hb_ft_font_create*` family and
//!   [`hb_ft_font_set_funcs`] install FreeType as the callback table behind an
//!   [`hb_font_t`], so that advances, extents, outlines and colour glyphs come
//!   from `FT_Load_Glyph` instead of from HarfBuzz's table readers. Use this
//!   when you need HarfBuzz's metrics to agree exactly with what FreeType will
//!   rasterize.
//!
//! Each family comes in variants that differ only in who keeps the `FT_Face`
//! alive:
//!
//! * `_create(ft_face, destroy)` — no lifecycle management at all. You must
//!   destroy the `FT_Face` yourself, and only after the HarfBuzz object is
//!   gone.
//! * `_create_referenced(ft_face)` — calls `FT_Reference_Face` on entry and
//!   `FT_Done_Face` when the HarfBuzz object dies. This is the variant to
//!   reach for.
//! * [`hb_ft_face_create_cached`] — stores the face in the `FT_Face`'s
//!   `generic` field so repeated calls hand back the same [`hb_face_t`]. You
//!   still own the `FT_Face`.
//!
//! Two objects can drift out of sync, because both an [`hb_font_t`] and an
//! `FT_Face` carry a size and a set of variation coordinates.
//! [`hb_ft_font_changed`] pushes the `FT_Face`'s state into the font;
//! [`hb_ft_hb_font_changed`] pushes the font's state back into the `FT_Face`.
//! Since HarfBuzz 11.0.0 the second direction is handled automatically on
//! every glyph query, so calling it by hand is no longer necessary.
//!
//! **None of this is thread-safe.** FreeType is not thread-safe, so neither
//! are these functions. The FreeType-backed callbacks do take an internal
//! mutex around each `FT_Face` access, and [`hb_ft_font_lock_face`] exposes
//! that mutex, but the `FT_Library` and any `FT_Face` you own remain yours to
//! synchronize.
//!
//! # The `FT_Face` type
//!
//! `hb-ft.h` includes FreeType's own headers to get `FT_Face`. Those
//! declarations belong to FreeType, not to HarfBuzz, and this crate has no
//! FreeType dependency to import them from — so [`FT_Face`] is declared here
//! as what it is in C: a plain pointer to an opaque record. It is
//! ABI-identical to FreeType's `FT_Face`, and a handle obtained from a
//! FreeType binding crate converts with a pointer cast:
//!
//! ```ignore
//! let hb_ft_face = their_ft_face as harfbuzz_sys::ft::FT_Face;
//! ```

use core::ffi::{c_char, c_int, c_uint};

use crate::{hb_blob_t, hb_bool_t, hb_destroy_func_t, hb_face_t, hb_font_t};

crate::opaque_handle! {
    /// FreeType's `FT_FaceRec_` — the record an [`FT_Face`] points at.
    ///
    /// This crate never looks inside it, and declares it only so that
    /// [`FT_Face`] has something to point to. The real definition lives in
    /// FreeType's `freetype.h`; if you need its fields, get them from a
    /// FreeType binding and cast the pointer.
    FT_FaceRec_
}

/// FreeType's handle to a face object — a pointer to an [`FT_FaceRec_`].
///
/// In FreeType this is `typedef struct FT_FaceRec_ *FT_Face`. A face there is
/// one typeface out of a font file together with the currently selected size,
/// charmap, transform and variation coordinates — so unlike an [`hb_face_t`],
/// it is a mutable, stateful object.
///
/// Set the size on the `FT_Face` (with `FT_Set_Char_Size` or
/// `FT_Set_Pixel_Sizes`) *before* handing it to [`hb_ft_font_create`] or
/// [`hb_ft_font_create_referenced`]: HarfBuzz assumes a size is always set and
/// dereferences the record's `size` member unconditionally.
pub type FT_Face = *mut FT_FaceRec_;

unsafe extern "C" {
    /// Creates a face object from an `FT_Face`, with no lifecycle management.
    ///
    /// The `FT_Face` is used only to reach the underlying font data. Fonts
    /// created from the returned face use HarfBuzz's native font
    /// implementation unless you also call [`hb_ft_font_set_funcs`] on them.
    ///
    /// `destroy` — which may be null — is called with the `FT_Face` when the
    /// returned face is destroyed. It does *not* release the `FT_Face` on your
    /// behalf: the caller remains responsible for destroying `ft_face`, and
    /// must do so only after the returned face has been destroyed.
    ///
    /// Most programs should call [`hb_ft_face_create_referenced`], or perhaps
    /// [`hb_ft_face_create_cached`], instead. In particular, passing a null
    /// `destroy` is a sign you want [`hb_ft_face_create_referenced`].
    ///
    /// Returns the new face. Release it with
    /// [`hb_face_destroy`](crate::hb_face_destroy).
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_ft_face_create(ft_face: FT_Face, destroy: hb_destroy_func_t) -> *mut hb_face_t;

    /// Creates a face object from an `FT_Face`, caching it on the `FT_Face`.
    ///
    /// Like [`hb_ft_face_create`], but the new face is stashed in the
    /// `FT_Face`'s `generic` field, so a later call with the same `ft_face`
    /// returns the same — correctly reference-counted — face rather than
    /// building another one.
    ///
    /// The caller is still responsible for destroying `ft_face`, and must do
    /// so only after the last returned face has been destroyed. Note that this
    /// takes over the `FT_Face`'s `generic` field, running whatever finalizer
    /// was there first.
    ///
    /// Returns the face, with one reference added for the caller. Release it
    /// with [`hb_face_destroy`](crate::hb_face_destroy).
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_ft_face_create_cached(ft_face: FT_Face) -> *mut hb_face_t;

    /// Creates a face object from an `FT_Face`, keeping the `FT_Face` alive.
    ///
    /// This is the preferred member of the `hb_ft_face_create*` family: it
    /// calls `FT_Reference_Face` on entry and `FT_Done_Face` when the returned
    /// face is destroyed, so the `FT_Face` cannot be released too early by
    /// accident. Use it unless you have a good reason not to.
    ///
    /// As with [`hb_ft_face_create`], the `FT_Face` supplies only table data;
    /// fonts built on the result still use HarfBuzz's native font
    /// implementation unless you call [`hb_ft_font_set_funcs`] on them.
    ///
    /// Returns the new face. Release it with
    /// [`hb_face_destroy`](crate::hb_face_destroy).
    ///
    /// Since HarfBuzz 0.9.38.
    pub fn hb_ft_face_create_referenced(ft_face: FT_Face) -> *mut hb_face_t;

    /// Creates a face object by opening a font file through FreeType.
    ///
    /// Similar in effect to
    /// [`hb_face_create_from_file_or_fail`](crate::hb_face_create_from_file_or_fail),
    /// but FreeType does the loading — which is how you read WOFF and WOFF2
    /// data, and anything else FreeType can decode but HarfBuzz cannot.
    /// `index` selects one face out of a collection file.
    ///
    /// The `FT_Face` and the `FT_Library` behind it are created internally and
    /// released with the returned face; there is nothing for the caller to
    /// clean up beyond the face itself.
    ///
    /// Returns the new face, or null if the file cannot be read or holds no
    /// face at `index`. Release it with
    /// [`hb_face_destroy`](crate::hb_face_destroy).
    ///
    /// Since HarfBuzz 10.1.0.
    pub fn hb_ft_face_create_from_file_or_fail(
        file_name: *const c_char,
        index: c_uint,
    ) -> *mut hb_face_t;

    /// Creates a face object from font data in a blob, through FreeType.
    ///
    /// Similar in effect to
    /// [`hb_face_create_or_fail`](crate::hb_face_create_or_fail), but FreeType
    /// does the parsing, so WOFF and WOFF2 payloads work. `index` selects one
    /// face out of a collection.
    ///
    /// Makes `blob` immutable and takes a reference on it, because the
    /// internally created `FT_Face` keeps reading from its bytes; the caller
    /// still owns its own reference and must still destroy it.
    ///
    /// Returns the new face, or null if the blob does not hold valid font
    /// data. Release it with [`hb_face_destroy`](crate::hb_face_destroy).
    ///
    /// Since HarfBuzz 11.0.0.
    pub fn hb_ft_face_create_from_blob_or_fail(
        blob: *mut hb_blob_t,
        index: c_uint,
    ) -> *mut hb_face_t;

    /// Creates a font object from an `FT_Face`, with no lifecycle management.
    ///
    /// The returned font is already configured to use FreeType font functions
    /// — there is no need to call [`hb_ft_font_set_funcs`] on it — and its
    /// scale is taken from the `FT_Face`'s current size. **Set the face size
    /// before calling this**, since HarfBuzz reads the `FT_Face`'s `size`
    /// member unconditionally.
    ///
    /// `destroy` — which may be null — is called with the `FT_Face` when the
    /// font's underlying face is destroyed. Even when it is supplied, the
    /// caller remains responsible for destroying `ft_face`, and must do so
    /// only after the returned font has been destroyed.
    ///
    /// Most programs should call [`hb_ft_font_create_referenced`] instead.
    ///
    /// Returns the new font. Release it with
    /// [`hb_font_destroy`](crate::hb_font_destroy).
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_ft_font_create(ft_face: FT_Face, destroy: hb_destroy_func_t) -> *mut hb_font_t;

    /// Creates a font object from an `FT_Face`, keeping the `FT_Face` alive.
    ///
    /// The preferred member of the `hb_ft_font_create*` family: it calls
    /// `FT_Reference_Face` on entry and `FT_Done_Face` when the font is
    /// destroyed. Use it unless you have a good reason not to.
    ///
    /// As with [`hb_ft_font_create`], set the face size on `ft_face` before
    /// calling this, and the returned font is already wired to FreeType font
    /// functions.
    ///
    /// Returns the new font. Release it with
    /// [`hb_font_destroy`](crate::hb_font_destroy).
    ///
    /// Since HarfBuzz 0.9.38.
    pub fn hb_ft_font_create_referenced(ft_face: FT_Face) -> *mut hb_font_t;

    /// Fetches the `FT_Face` behind a FreeType-backed font.
    ///
    /// Works only on fonts created by [`hb_ft_font_create`] or
    /// [`hb_ft_font_create_referenced`], or configured with
    /// [`hb_ft_font_set_funcs`]; any other font yields null.
    ///
    /// The font keeps ownership: do not call `FT_Done_Face` on the result. No
    /// lock is taken either, so use [`hb_ft_font_lock_face`] instead if
    /// HarfBuzz might touch the same `FT_Face` concurrently.
    ///
    /// Since HarfBuzz 10.4.0.
    pub fn hb_ft_font_get_ft_face(font: *mut hb_font_t) -> FT_Face;

    /// Locks and fetches the `FT_Face` behind a FreeType-backed font.
    ///
    /// Blocks HarfBuzz's own access to that `FT_Face` from every other API
    /// until [`hb_ft_font_unlock_face`] is called, so you can safely use
    /// FreeType directly on it in the meantime.
    ///
    /// Works only on fonts created by [`hb_ft_font_create`] or
    /// [`hb_ft_font_create_referenced`], or configured with
    /// [`hb_ft_font_set_funcs`]; any other font yields null — and in that case
    /// no lock was taken, so do not call [`hb_ft_font_unlock_face`].
    ///
    /// Ownership stays with the font; no reference is transferred.
    ///
    /// Since HarfBuzz 2.6.5.
    pub fn hb_ft_font_lock_face(font: *mut hb_font_t) -> FT_Face;

    /// Releases an `FT_Face` previously obtained with
    /// [`hb_ft_font_lock_face`].
    ///
    /// Does nothing if `font` is not FreeType-backed.
    ///
    /// Since HarfBuzz 2.6.5.
    pub fn hb_ft_font_unlock_face(font: *mut hb_font_t);

    /// Sets the `FT_Load_Glyph` load flags used by a FreeType-backed font.
    ///
    /// These are FreeType's own `FT_LOAD_*` flags, ORed together; see
    /// FreeType's [glyph-retrieval documentation](https://freetype.org/freetype2/docs/reference/ft2-glyph_retrieval.html#ft_load_xxx).
    /// The default is `FT_LOAD_DEFAULT | FT_LOAD_NO_HINTING`, which is what
    /// makes hb-ft's metrics match HarfBuzz's unhinted, unrounded model.
    ///
    /// Silently does nothing if the font is immutable or is not
    /// FreeType-backed.
    ///
    /// Since HarfBuzz 1.0.5.
    pub fn hb_ft_font_set_load_flags(font: *mut hb_font_t, load_flags: c_int);

    /// Fetches the `FT_Load_Glyph` load flags of a FreeType-backed font.
    ///
    /// Returns the flags, or zero if the font is not FreeType-backed — a value
    /// indistinguishable from a genuine `FT_LOAD_DEFAULT`.
    ///
    /// Since HarfBuzz 1.0.5.
    pub fn hb_ft_font_get_load_flags(font: *mut hb_font_t) -> c_int;

    /// Refreshes a font after its underlying `FT_Face` changed.
    ///
    /// Call this once you have changed the size or the variation-axis
    /// coordinates on the `FT_Face` directly, to copy that state into the
    /// font: it recomputes the font's scale from the face's size metrics and
    /// units-per-em, republishes the face's normalized variation coordinates,
    /// and clears the advance cache.
    ///
    /// Silently does nothing if the font is not FreeType-backed.
    ///
    /// Since HarfBuzz 1.0.5.
    pub fn hb_ft_font_changed(font: *mut hb_font_t);

    /// Refreshes the underlying `FT_Face` after the font changed.
    ///
    /// The opposite direction to [`hb_ft_font_changed`]: call it once you have
    /// changed the scale or the variation coordinates on the [`hb_font_t`]
    /// itself. It is cheap when nothing changed, because it compares the
    /// font's serial number against the cached one first.
    ///
    /// As of HarfBuzz 11.0.0 this is no longer necessary: every glyph query
    /// performs the same check and updates the `FT_Face` on demand.
    ///
    /// Returns true if the `FT_Face` was updated, false if nothing had changed
    /// or the font is not FreeType-backed.
    ///
    /// Since HarfBuzz 4.4.0.
    pub fn hb_ft_hb_font_changed(font: *mut hb_font_t) -> hb_bool_t;

    /// Configures a font to use FreeType font functions.
    ///
    /// This is how you put an existing font on FreeType even when its face
    /// came from [`hb_face_create`](crate::hb_face_create) rather than from
    /// this header. Internally it creates a fresh `FT_Face` — and an
    /// `FT_Library` to hold it — over the face's blob, selects the symbol or
    /// Unicode charmap, and installs FreeType's callback table on the font.
    /// Both are released with the font.
    ///
    /// Fonts made by [`hb_ft_font_create`] or [`hb_ft_font_create_referenced`]
    /// are already configured this way and do not need this call.
    ///
    /// Any font data previously attached to the font — by
    /// [`hb_font_set_funcs`](crate::hb_font_set_funcs) or another back end —
    /// is destroyed. The existing load flags are carried over if the font was
    /// already FreeType-backed, and otherwise reset to the default
    /// `FT_LOAD_DEFAULT | FT_LOAD_NO_HINTING`.
    ///
    /// There is no success indication: on allocation failure, on a missing
    /// `FT_Library`, or on an `FT_New_Memory_Face` error the font is left with
    /// the empty font functions, and every glyph query then fails.
    ///
    /// Since HarfBuzz 1.0.5.
    pub fn hb_ft_font_set_funcs(font: *mut hb_font_t);

    /// Fetches the `FT_Face` behind a FreeType-backed font.
    ///
    /// An older spelling of [`hb_ft_font_get_ft_face`], which it simply calls.
    ///
    /// Since HarfBuzz 0.9.2. Deprecated since HarfBuzz 10.4.0.
    #[deprecated(note = "use `hb_ft_font_get_ft_face` instead")]
    pub fn hb_ft_font_get_face(font: *mut hb_font_t) -> FT_Face;
}
