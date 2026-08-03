//! Colour-font information from OpenType faces — `hb-ot-color.h`.
//!
//! Covers the four colour-font mechanisms HarfBuzz understands: `CPAL` colour
//! palettes, `COLR` layered and paint glyphs, `SVG` glyph documents, and PNG
//! glyph images stored in `CBDT` or `sbix`.

use core::ffi::c_uint;

use crate::{
    hb_blob_t, hb_bool_t, hb_codepoint_t, hb_color_t, hb_face_t, hb_font_t, hb_ot_name_id_t,
};

/// Flags that describe the properties of a colour palette.
///
/// The values are a bit field: test them with a bitwise AND rather than
/// comparing for equality, since a palette may be marked usable with both a
/// light and a dark background.
///
/// The C enumeration has no sentinel and its largest enumerator is 2, so it
/// fits in an `int`.
///
/// Since HarfBuzz 2.1.0.
pub type hb_ot_color_palette_flags_t = core::ffi::c_int;

/// Default, indicating that there is nothing special to note about a colour
/// palette.
///
/// Since HarfBuzz 2.1.0.
pub const HB_OT_COLOR_PALETTE_FLAG_DEFAULT: hb_ot_color_palette_flags_t = 0x00000000;

/// The colour palette is appropriate to use when displaying the font on a light
/// background such as white.
///
/// Since HarfBuzz 2.1.0.
pub const HB_OT_COLOR_PALETTE_FLAG_USABLE_WITH_LIGHT_BACKGROUND: hb_ot_color_palette_flags_t =
    0x00000001;

/// The colour palette is appropriate to use when displaying the font on a dark
/// background such as black.
///
/// Since HarfBuzz 2.1.0.
pub const HB_OT_COLOR_PALETTE_FLAG_USABLE_WITH_DARK_BACKGROUND: hb_ot_color_palette_flags_t =
    0x00000002;

/// A pair of glyph and colour index, describing one layer of a `COLR` v0 colour
/// glyph.
///
/// A colour index of `0xFFFF` does not refer to a palette colour, but indicates
/// that the foreground colour should be used.
///
/// Since HarfBuzz 2.1.0.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_ot_color_layer_t {
    /// The glyph ID of the layer.
    pub glyph: hb_codepoint_t,
    /// The palette colour index of the layer, or `0xFFFF` for the foreground
    /// colour.
    pub color_index: c_uint,
}

unsafe extern "C" {
    /// Tests whether a face includes a `CPAL` colour-palette table.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_color_has_palettes(face: *mut hb_face_t) -> hb_bool_t;

    /// Fetches the number of colour palettes in a face.
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_color_palette_get_count(face: *mut hb_face_t) -> c_uint;

    /// Fetches the `name`-table Name ID that provides display names for a
    /// `CPAL` colour palette.
    ///
    /// Palette display names can be generic — "Default" — or provide specific,
    /// themed names such as "Spring", "Summer", "Fall", and "Winter".
    ///
    /// Returns the Name ID found for the palette, or
    /// [`HB_OT_NAME_ID_INVALID`](crate::HB_OT_NAME_ID_INVALID) if the requested
    /// palette has no name.
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_color_palette_get_name_id(
        face: *mut hb_face_t,
        palette_index: c_uint,
    ) -> hb_ot_name_id_t;

    /// Fetches the `name`-table Name ID that provides display names for the
    /// specified colour in a face's `CPAL` colour palette.
    ///
    /// Display names can be generic — "Background" — or specific, such as
    /// "Eye color".
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_color_palette_color_get_name_id(
        face: *mut hb_face_t,
        color_index: c_uint,
    ) -> hb_ot_name_id_t;

    /// Fetches the flags defined for a colour palette.
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_color_palette_get_flags(
        face: *mut hb_face_t,
        palette_index: c_uint,
    ) -> hb_ot_color_palette_flags_t;

    /// Fetches a list of the colours in a colour palette.
    ///
    /// On input `color_count` is the maximum number of colours to return; on
    /// output it holds the number actually written, which may be zero. Both
    /// `color_count` and `colors` may be null: passing a null `colors` returns
    /// the total number of colours without storing any, which is the way to
    /// size a buffer before calling a second time.
    ///
    /// The RGBA values in the palette are unpremultiplied.
    ///
    /// Returns the total number of colours in the palette.
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_color_palette_get_colors(
        face: *mut hb_face_t,
        palette_index: c_uint,
        start_offset: c_uint,
        color_count: *mut c_uint,
        colors: *mut hb_color_t,
    ) -> c_uint;

    /// Tests whether a face includes a `COLR` table with data according to
    /// COLRv0.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_color_has_layers(face: *mut hb_face_t) -> hb_bool_t;

    /// Fetches a list of all colour layers for the specified glyph index in the
    /// specified face. The list returned begins at the offset provided.
    ///
    /// On input `layer_count` is the maximum number of layers to return; on
    /// output it holds the number actually written, which may be zero. Both
    /// `layer_count` and `layers` may be null.
    ///
    /// Returns the total number of layers available for the glyph index
    /// queried.
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_color_glyph_get_layers(
        face: *mut hb_face_t,
        glyph: hb_codepoint_t,
        start_offset: c_uint,
        layer_count: *mut c_uint,
        layers: *mut hb_ot_color_layer_t,
    ) -> c_uint;

    /// Tests whether a face includes a `COLR` table with data according to
    /// COLRv1.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_ot_color_has_paint(face: *mut hb_face_t) -> hb_bool_t;

    /// Tests whether a face includes COLRv1 paint data for `glyph`.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_ot_color_glyph_has_paint(face: *mut hb_face_t, glyph: hb_codepoint_t) -> hb_bool_t;

    /// Tests whether a face includes any `SVG` glyph images.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_color_has_svg(face: *mut hb_face_t) -> hb_bool_t;

    /// Gets the number of SVG documents in the face's `SVG` table.
    ///
    /// Since HarfBuzz 12.1.0.
    pub fn hb_ot_color_get_svg_document_count(face: *mut hb_face_t) -> c_uint;

    /// Gets the `SVG`-table document index associated with a glyph.
    ///
    /// `svg_document_index` may be null, and is written only when the glyph
    /// does map to a document.
    ///
    /// Returns true if `glyph` maps to an SVG document, false otherwise.
    ///
    /// Since HarfBuzz 12.1.0.
    pub fn hb_ot_color_glyph_get_svg_document_index(
        face: *mut hb_face_t,
        glyph: hb_codepoint_t,
        svg_document_index: *mut c_uint,
    ) -> hb_bool_t;

    /// Gets the glyph range covered by an `SVG`-table document index.
    ///
    /// `start_glyph_id` and `end_glyph_id` may each be null.
    ///
    /// Returns true if `svg_document_index` is valid, false otherwise.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_ot_color_get_svg_document_glyph_range(
        face: *mut hb_face_t,
        svg_document_index: c_uint,
        start_glyph_id: *mut hb_codepoint_t,
        end_glyph_id: *mut hb_codepoint_t,
    ) -> hb_bool_t;

    /// Fetches the SVG document for a glyph. The blob may be either plain text
    /// or gzip-encoded.
    ///
    /// If the glyph has no SVG document, the singleton empty blob is returned.
    ///
    /// The caller owns a reference on the returned blob and must release it
    /// with [`hb_blob_destroy`](crate::hb_blob_destroy).
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_color_glyph_reference_svg(
        face: *mut hb_face_t,
        glyph: hb_codepoint_t,
    ) -> *mut hb_blob_t;

    /// Tests whether a face has PNG glyph images, in either the `CBDT` or the
    /// `sbix` table.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_color_has_png(face: *mut hb_face_t) -> hb_bool_t;

    /// Fetches the PNG image for a glyph.
    ///
    /// This function takes a font object, not a face object. To get an
    /// optimally sized PNG blob, the PPEM values must be set on `font`; if PPEM
    /// is unset, the blob returned is the largest PNG available.
    ///
    /// If the glyph has no PNG image, the singleton empty blob is returned.
    ///
    /// The caller owns a reference on the returned blob and must release it
    /// with [`hb_blob_destroy`](crate::hb_blob_destroy).
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_color_glyph_reference_png(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
    ) -> *mut hb_blob_t;
}
