//! Apple Core Text and Core Graphics integration — `hb-coretext.h`.
//!
//! On macOS, iOS, and the rest of the Apple platforms, fonts are usually
//! already in hand as Core Graphics or Core Text objects rather than as file
//! paths or byte buffers. This sub-library bridges those objects to HarfBuzz's
//! own, in both directions:
//!
//! * A `CGFontRef` — Core Graphics' size-independent typeface — corresponds to
//!   an [`hb_face_t`]. Convert with [`hb_coretext_face_create`], and go back
//!   with [`hb_coretext_face_get_cg_font`].
//! * A `CTFontRef` — Core Text's typeface *at a particular size and
//!   configuration* — corresponds to an [`hb_font_t`]. Convert with
//!   [`hb_coretext_font_create`], and go back with
//!   [`hb_coretext_font_get_ct_font`].
//!
//! Two further constructors, [`hb_coretext_face_create_from_file_or_fail`] and
//! [`hb_coretext_face_create_from_blob_or_fail`], skip the Core Graphics step
//! and let Core Text parse the font data itself. They are the CoreText analogues
//! of `hb_face_create_from_file_or_fail` and `hb_face_create_or_fail`.
//!
//! Faces and fonts made this way are shaped by HarfBuzz's own OpenType engine
//! and use HarfBuzz's own font functions by default; Core Text is only the
//! source of the font data. [`hb_coretext_font_set_funcs`] opts a font into
//! Core Text's glyph metrics, glyph mapping, and outline extraction instead,
//! which is rarely what you want but is available for metric-compatibility work.
//!
//! The header also names three AAT table tags — [`HB_CORETEXT_TAG_MORT`],
//! [`HB_CORETEXT_TAG_MORX`], and [`HB_CORETEXT_TAG_KERX`]. Note that HarfBuzz
//! reads those tables natively on every platform, so simply wanting AAT shaping
//! is *not* a reason to use this module.
//!
//! This module is compiled only when the crate's `coretext` feature is enabled,
//! which is also what makes `build.rs` compile the CoreText sources and link
//! the Apple frameworks. Because of that gating the module is reached as
//! `harfbuzz_sys::coretext` rather than being re-exported at the crate root.
//! `build.rs` ignores the feature entirely on non-Apple targets, so on those
//! targets the declarations below have no symbols behind them.

use core::ffi::{c_char, c_uint};

use crate::{HB_TAG, hb_blob_t, hb_face_t, hb_font_t, hb_tag_t};

// ---------------------------------------------------------------------------
// Foreign types
//
// `CGFontRef` and `CTFontRef` belong to Apple's frameworks, not to HarfBuzz,
// and this crate has no dependency that supplies them. They are declared here
// so that the signatures below can be written honestly. Both are plain
// pointers, so they are layout- and ABI-compatible with the definitions in the
// `core-graphics` and `core-text` crates — a `cast()` converts between them.
// ---------------------------------------------------------------------------

crate::opaque_handle! {
    /// Core Graphics' opaque font object, spelled `struct CGFont` in
    /// `<CoreGraphics/CGFont.h>`. Only ever used through [`CGFontRef`].
    CGFont
}

/// A reference to a Core Graphics font — a typeface with no size attached.
///
/// This is the Core Graphics counterpart of [`hb_face_t`]. It is a
/// CoreFoundation-style object: retain it with `CGFontRetain` and release it
/// with `CGFontRelease`.
pub type CGFontRef = *mut CGFont;

crate::opaque_handle! {
    /// Core Text's opaque font object, spelled `struct __CTFont` in
    /// `<CoreText/CTFont.h>`. Only ever used through [`CTFontRef`].
    CTFont
}

/// A reference to a Core Text font — a typeface together with a point size,
/// a transform, and a variation configuration.
///
/// This is the Core Text counterpart of [`hb_font_t`]. It is a
/// CoreFoundation-style object: retain it with `CFRetain` and release it with
/// `CFRelease`.
pub type CTFontRef = *const CTFont;

// ---------------------------------------------------------------------------
// AAT table tags
// ---------------------------------------------------------------------------

/// The tag for the `mort` (glyph metamorphosis) table, which holds AAT
/// features.
///
/// See Apple's
/// [TrueType Reference Manual](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6mort.html)
/// for the table format. `mort` is the older, superseded form of
/// [`HB_CORETEXT_TAG_MORX`].
pub const HB_CORETEXT_TAG_MORT: hb_tag_t = HB_TAG(b'm', b'o', b'r', b't');

/// The tag for the `morx` (extended glyph metamorphosis) table, which holds
/// AAT features.
///
/// See Apple's
/// [TrueType Reference Manual](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6morx.html)
/// for the table format. This is the AAT counterpart of OpenType's `GSUB`.
pub const HB_CORETEXT_TAG_MORX: hb_tag_t = HB_TAG(b'm', b'o', b'r', b'x');

/// The tag for the `kerx` (extended kerning) table, which holds AAT kerning
/// information.
///
/// See Apple's
/// [TrueType Reference Manual](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6kerx.html)
/// for the table format. This is one of the AAT counterparts of OpenType's
/// `GPOS`.
pub const HB_CORETEXT_TAG_KERX: hb_tag_t = HB_TAG(b'k', b'e', b'r', b'x');

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

unsafe extern "C" {
    /// Creates a face that reads its tables from a Core Graphics font.
    ///
    /// The face keeps its own retain on `cg_font`, so the caller may release
    /// its reference immediately afterwards. Table data is fetched lazily,
    /// one table at a time, through `CGFontCopyTableForTag`.
    ///
    /// Never returns null: on allocation failure the singleton empty face comes
    /// back instead. The caller owns the returned face and must release it with
    /// `hb_face_destroy`.
    ///
    /// Since HarfBuzz 0.9.10.
    pub fn hb_coretext_face_create(cg_font: CGFontRef) -> *mut hb_face_t;

    /// Creates a face by letting Core Text load a font file from disk.
    ///
    /// Similar in effect to `hb_face_create_from_file_or_fail`, but the file is
    /// parsed by `CTFontManager` rather than by HarfBuzz.
    ///
    /// The low 16 bits of `index` select a face within a font collection and
    /// must be zero — Core Text cannot read TrueType collections. The high 16
    /// bits, when non-zero, select a named instance of a variable font.
    ///
    /// Returns null if the file cannot be read or holds no face at `index`.
    /// The caller owns a non-null result and must release it with
    /// `hb_face_destroy`.
    ///
    /// Since HarfBuzz 10.1.0.
    pub fn hb_coretext_face_create_from_file_or_fail(
        file_name: *const c_char,
        index: c_uint,
    ) -> *mut hb_face_t;

    /// Creates a face by letting Core Text parse font data already in memory.
    ///
    /// Similar in effect to `hb_face_create_or_fail`, but the bytes are parsed
    /// by Core Graphics or `CTFontManager` rather than by HarfBuzz. The blob is
    /// made immutable and an extra reference is taken on it, so its bytes are
    /// borrowed for as long as the resulting face lives — they are not copied.
    ///
    /// The low 16 bits of `index` select a face within a font collection and
    /// must be zero — Core Text cannot read TrueType collections. The high 16
    /// bits, when non-zero, select a named instance of a variable font.
    ///
    /// Returns null if the data cannot be read or holds no face at `index`.
    /// The caller owns a non-null result and must release it with
    /// `hb_face_destroy`.
    ///
    /// Since HarfBuzz 11.0.0.
    pub fn hb_coretext_face_create_from_blob_or_fail(
        blob: *mut hb_blob_t,
        index: c_uint,
    ) -> *mut hb_face_t;

    /// Creates a font from a Core Text font.
    ///
    /// The point size of `ct_font` is copied to the new font's `ptem`, and its
    /// variation settings are copied through as if by `hb_font_set_variations`.
    /// The font retains `ct_font` and hands it back from
    /// [`hb_coretext_font_get_ct_font`]; the caller keeps its own reference.
    ///
    /// The created font uses HarfBuzz's own font functions, not Core Text's.
    /// Call [`hb_coretext_font_set_funcs`] to change that.
    ///
    /// The caller owns the returned font and must release it with
    /// `hb_font_destroy`.
    ///
    /// Since HarfBuzz 1.7.2.
    pub fn hb_coretext_font_create(ct_font: CTFontRef) -> *mut hb_font_t;

    /// Fetches the Core Graphics font backing a face.
    ///
    /// This works for any face, not only one made by
    /// [`hb_coretext_face_create`]: if the face has no Core Graphics font yet,
    /// one is created on demand from the face's font data and cached on the
    /// face.
    ///
    /// The returned reference is owned by the face and must not be released by
    /// the caller; it stays valid for as long as the face does. Returns null if
    /// a Core Graphics font could not be produced — for a TrueType collection,
    /// for instance.
    ///
    /// Since HarfBuzz 0.9.10.
    pub fn hb_coretext_face_get_cg_font(face: *mut hb_face_t) -> CGFontRef;

    /// Fetches the Core Text font backing a font object.
    ///
    /// This works for any font, not only one made by
    /// [`hb_coretext_font_create`]: if the font has no Core Text font yet, one
    /// is created on demand from the face's Core Graphics font, at the font's
    /// `ptem` — or at 12 points when `ptem` is unset — with the font's
    /// normalized variation coordinates applied, and cached on the font.
    ///
    /// The returned reference is owned by the font and must not be released by
    /// the caller; it stays valid for as long as the font does. Returns null if
    /// a Core Text font could not be produced.
    ///
    /// Since HarfBuzz 0.9.10.
    pub fn hb_coretext_font_get_ct_font(font: *mut hb_font_t) -> CTFontRef;

    /// Switches a font over to Core Text's font functions.
    ///
    /// Glyph mapping, advances, extents, outlines, and glyph names are then
    /// answered by Core Text instead of by HarfBuzz's own OpenType
    /// implementation. This works on any font, including one whose face was
    /// built with `hb_face_create` and never touched Core Text before.
    ///
    /// Internally this creates a Core Text font, exactly as
    /// [`hb_coretext_font_get_ct_font`] does. If that fails the font is given
    /// the *empty* font functions, which answer nothing — so a font that cannot
    /// be represented in Core Text ends up producing no glyphs at all rather
    /// than falling back.
    ///
    /// Since HarfBuzz 10.1.0.
    pub fn hb_coretext_font_set_funcs(font: *mut hb_font_t);
}
