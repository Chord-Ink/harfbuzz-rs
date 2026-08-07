//! Font objects and the virtual methods that query them — `hb-font.h`.
//!
//! An [`hb_font_t`] is a face at a particular size and configuration; an
//! [`hb_font_funcs_t`] is the table of callbacks that answers questions about it.

use core::ffi::{c_char, c_float, c_int, c_uint, c_void};

use crate::{
    hb_bool_t, hb_codepoint_t, hb_color_t, hb_destroy_func_t, hb_direction_t, hb_draw_funcs_t,
    hb_face_t, hb_glyph_extents_t, hb_paint_funcs_t, hb_position_t, hb_tag_t, hb_user_data_key_t,
    hb_variation_t,
};

opaque_handle! {
    /// A set of virtual methods used for working on [`hb_font_t`] font objects.
    ///
    /// HarfBuzz provides a lightweight default implementation of every method
    /// in an `hb_font_funcs_t`. Client programs can implement their own
    /// replacements for the individual font functions, as needed, and displace
    /// the default by calling the setter for that method.
    hb_font_funcs_t
}

opaque_handle! {
    /// A font: a face combined with a size and other rendering parameters.
    ///
    /// A font object represents a font face at a specific scale, and with
    /// certain other parameters — pixels-per-em, points-per-em, variation
    /// settings — specified. Font objects are created from face objects with
    /// [`hb_font_create`], and are the input to `hb_shape`, among other things.
    ///
    /// Client programs can optionally supply their own implementations of the
    /// basic, lower-level queries of a font object. That set of callbacks is
    /// the [`hb_font_funcs_t`] attached with [`hb_font_set_funcs`].
    ///
    /// The default font functions are implemented in terms of the
    /// [`hb_font_funcs_t`] methods of the *parent* font object. That lets a
    /// client override only the methods it cares about and inherit the parent
    /// font's implementation for the rest.
    hb_font_t
}

/// Font-wide extent values, measured in scaled units.
///
/// Note that `ascender` is typically positive and `descender` negative, in
/// coordinate systems that grow up.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_font_extents_t {
    /// The height of typographic ascenders.
    pub ascender: hb_position_t,
    /// The depth of typographic descenders.
    pub descender: hb_position_t,
    /// The suggested line-spacing gap.
    pub line_gap: hb_position_t,

    /// Private padding, present so the Rust layout matches C. Do not read.
    pub reserved9: hb_position_t,
    /// Private padding, present so the Rust layout matches C. Do not read.
    pub reserved8: hb_position_t,
    /// Private padding, present so the Rust layout matches C. Do not read.
    pub reserved7: hb_position_t,
    /// Private padding, present so the Rust layout matches C. Do not read.
    pub reserved6: hb_position_t,
    /// Private padding, present so the Rust layout matches C. Do not read.
    pub reserved5: hb_position_t,
    /// Private padding, present so the Rust layout matches C. Do not read.
    pub reserved4: hb_position_t,
    /// Private padding, present so the Rust layout matches C. Do not read.
    pub reserved3: hb_position_t,
    /// Private padding, present so the Rust layout matches C. Do not read.
    pub reserved2: hb_position_t,
    /// Private padding, present so the Rust layout matches C. Do not read.
    pub reserved1: hb_position_t,
}

// --- Virtual method signatures ----------------------------------------------
//
// Every callback receives the font it is being asked about, the `font_data`
// pointer that was attached with `hb_font_set_funcs`, and the `user_data`
// pointer that was supplied when this particular method was installed.

/// Retrieves the extents for a font.
pub type hb_font_get_font_extents_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        extents: *mut hb_font_extents_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// Retrieves the extents for a font, for horizontal-direction text segments.
///
/// Extents are returned through the `extents` out-parameter.
pub type hb_font_get_font_h_extents_func_t = hb_font_get_font_extents_func_t;

/// Retrieves the extents for a font, for vertical-direction text segments.
///
/// Extents are returned through the `extents` out-parameter.
pub type hb_font_get_font_v_extents_func_t = hb_font_get_font_extents_func_t;

/// Retrieves the nominal glyph ID for a Unicode code point.
///
/// The glyph ID is returned through the `glyph` out-parameter. Returns true if
/// data was found, false otherwise.
pub type hb_font_get_nominal_glyph_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        unicode: hb_codepoint_t,
        glyph: *mut hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// Retrieves the glyph ID for a Unicode code point followed by a Variation
/// Selector code point.
///
/// The glyph ID is returned through the `glyph` out-parameter. Returns true if
/// data was found, false otherwise.
pub type hb_font_get_variation_glyph_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        unicode: hb_codepoint_t,
        variation_selector: hb_codepoint_t,
        glyph: *mut hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// Retrieves the nominal glyph IDs for a sequence of Unicode code points.
///
/// Inputs and outputs are strided arrays: `unicode_stride` and `glyph_stride`
/// are byte offsets between successive elements. Returns the number of code
/// points processed.
pub type hb_font_get_nominal_glyphs_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        count: c_uint,
        first_unicode: *const hb_codepoint_t,
        unicode_stride: c_uint,
        first_glyph: *mut hb_codepoint_t,
        glyph_stride: c_uint,
        user_data: *mut c_void,
    ) -> c_uint,
>;

/// Retrieves the advance for a glyph, returned as an [`hb_position_t`].
pub type hb_font_get_glyph_advance_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        glyph: hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_position_t,
>;

/// Retrieves the advance for a glyph in horizontal-direction text segments.
pub type hb_font_get_glyph_h_advance_func_t = hb_font_get_glyph_advance_func_t;

/// Retrieves the advance for a glyph in vertical-direction text segments.
pub type hb_font_get_glyph_v_advance_func_t = hb_font_get_glyph_advance_func_t;

/// Retrieves the advances for a sequence of glyphs.
///
/// Inputs and outputs are strided arrays: `glyph_stride` and `advance_stride`
/// are byte offsets between successive elements.
pub type hb_font_get_glyph_advances_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        count: c_uint,
        first_glyph: *const hb_codepoint_t,
        glyph_stride: c_uint,
        first_advance: *mut hb_position_t,
        advance_stride: c_uint,
        user_data: *mut c_void,
    ),
>;

/// Retrieves the advances for a sequence of glyphs, in horizontal-direction
/// text segments.
pub type hb_font_get_glyph_h_advances_func_t = hb_font_get_glyph_advances_func_t;

/// Retrieves the advances for a sequence of glyphs, in vertical-direction text
/// segments.
pub type hb_font_get_glyph_v_advances_func_t = hb_font_get_glyph_advances_func_t;

/// Retrieves the (X, Y) coordinates, in scaled units, of the origin for a
/// glyph.
///
/// Each coordinate is returned through its own out-parameter. Returns true if
/// data was found, false otherwise.
pub type hb_font_get_glyph_origin_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        glyph: hb_codepoint_t,
        x: *mut hb_position_t,
        y: *mut hb_position_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// Retrieves the origin of a glyph for horizontal-direction text segments.
pub type hb_font_get_glyph_h_origin_func_t = hb_font_get_glyph_origin_func_t;

/// Retrieves the origin of a glyph for vertical-direction text segments.
pub type hb_font_get_glyph_v_origin_func_t = hb_font_get_glyph_origin_func_t;

/// Retrieves the (X, Y) coordinates, in scaled units, of the origin for each of
/// a sequence of glyphs.
///
/// Inputs and outputs are strided arrays. Returns true if data was found, false
/// otherwise.
///
/// Since HarfBuzz 11.3.0.
pub type hb_font_get_glyph_origins_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        count: c_uint,
        first_glyph: *const hb_codepoint_t,
        glyph_stride: c_uint,
        first_x: *mut hb_position_t,
        x_stride: c_uint,
        first_y: *mut hb_position_t,
        y_stride: c_uint,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// Retrieves the origins of a sequence of glyphs, for horizontal-direction text
/// segments.
///
/// Since HarfBuzz 11.3.0.
pub type hb_font_get_glyph_h_origins_func_t = hb_font_get_glyph_origins_func_t;

/// Retrieves the origins of a sequence of glyphs, for vertical-direction text
/// segments.
///
/// Since HarfBuzz 11.3.0.
pub type hb_font_get_glyph_v_origins_func_t = hb_font_get_glyph_origins_func_t;

/// Retrieves the kerning adjustment for a glyph pair, for horizontal text
/// segments.
pub type hb_font_get_glyph_kerning_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        first_glyph: hb_codepoint_t,
        second_glyph: hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_position_t,
>;

/// Retrieves the kerning adjustment for a glyph pair, for horizontal text
/// segments.
pub type hb_font_get_glyph_h_kerning_func_t = hb_font_get_glyph_kerning_func_t;

/// Retrieves the extents for a glyph.
///
/// Extents are returned through the `extents` out-parameter. Returns true if
/// data was found, false otherwise.
pub type hb_font_get_glyph_extents_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        glyph: hb_codepoint_t,
        extents: *mut hb_glyph_extents_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// Retrieves the (X, Y) coordinates, in scaled units, of a specified contour
/// point of a glyph.
///
/// Returns true if data was found, false otherwise.
pub type hb_font_get_glyph_contour_point_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        glyph: hb_codepoint_t,
        point_index: c_uint,
        x: *mut hb_position_t,
        y: *mut hb_position_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// Retrieves the glyph name that corresponds to a glyph ID.
///
/// The name is written into the caller's `name` buffer, which holds `size`
/// bytes. Returns true if data was found, false otherwise.
pub type hb_font_get_glyph_name_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        glyph: hb_codepoint_t,
        name: *mut c_char,
        size: c_uint,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// Retrieves the glyph ID that corresponds to a glyph-name string.
///
/// A `len` of `-1` means `name` is NUL-terminated. Returns true if data was
/// found, false otherwise.
pub type hb_font_get_glyph_from_name_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        name: *const c_char,
        len: c_int,
        glyph: *mut hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// Draws the outline of a glyph by calling into `draw_funcs`.
///
/// Returns true if the glyph was drawn, false otherwise.
///
/// Since HarfBuzz 11.2.0.
pub type hb_font_draw_glyph_or_fail_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        glyph: hb_codepoint_t,
        draw_funcs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// Paints a color glyph by calling into `paint_funcs`.
///
/// Returns true if the glyph was painted, false otherwise.
///
/// Since HarfBuzz 11.2.0.
pub type hb_font_paint_glyph_or_fail_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        glyph: hb_codepoint_t,
        paint_funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        palette_index: c_uint,
        foreground: hb_color_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// Signifies that a font has no named-instance index set. This is a font's
/// default.
///
/// Since HarfBuzz 7.0.0.
pub const HB_FONT_NO_VAR_NAMED_INSTANCE: c_uint = 0xFFFFFFFF;

unsafe extern "C" {
    // --- Font-functions lifecycle -------------------------------------------

    /// Creates a new font-functions structure, with every method set to
    /// HarfBuzz's default.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_create() -> *mut hb_font_funcs_t;

    /// Fetches the empty font-functions structure — a shared, immutable
    /// singleton whose methods all return failure.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_get_empty() -> *mut hb_font_funcs_t;

    /// Increases the reference count on a font-functions structure and returns
    /// it.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_reference(ffuncs: *mut hb_font_funcs_t) -> *mut hb_font_funcs_t;

    /// Decreases the reference count on a font-functions structure. When the
    /// count reaches zero the structure is destroyed, freeing all its memory.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_destroy(ffuncs: *mut hb_font_funcs_t);

    /// Attaches a user-data key/data pair to a font-functions structure.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_set_user_data(
        ffuncs: *mut hb_font_funcs_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to a font-functions structure under the
    /// given key. The pointer is not transferred to the caller.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_get_user_data(
        ffuncs: *const hb_font_funcs_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Makes a font-functions structure immutable. Later setter calls on it are
    /// ignored.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_make_immutable(ffuncs: *mut hb_font_funcs_t);

    /// Tests whether a font-functions structure is immutable.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_is_immutable(ffuncs: *mut hb_font_funcs_t) -> hb_bool_t;

    // --- Font-functions method setters --------------------------------------
    //
    // Each setter installs one callback along with a `user_data` pointer to
    // hand it, and a `destroy` callback that HarfBuzz invokes on that pointer
    // when the method is replaced or the structure is destroyed. `destroy` may
    // be null.

    /// Sets the implementation of [`hb_font_get_font_h_extents_func_t`].
    ///
    /// Since HarfBuzz 1.1.2.
    pub fn hb_font_funcs_set_font_h_extents_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_font_h_extents_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_font_v_extents_func_t`].
    ///
    /// Since HarfBuzz 1.1.2.
    pub fn hb_font_funcs_set_font_v_extents_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_font_v_extents_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_nominal_glyph_func_t`].
    ///
    /// Since HarfBuzz 1.2.3.
    pub fn hb_font_funcs_set_nominal_glyph_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_nominal_glyph_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_nominal_glyphs_func_t`].
    ///
    /// Since HarfBuzz 2.0.0.
    pub fn hb_font_funcs_set_nominal_glyphs_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_nominal_glyphs_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_variation_glyph_func_t`].
    ///
    /// Since HarfBuzz 1.2.3.
    pub fn hb_font_funcs_set_variation_glyph_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_variation_glyph_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_glyph_h_advance_func_t`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_set_glyph_h_advance_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_h_advance_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_glyph_v_advance_func_t`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_set_glyph_v_advance_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_v_advance_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_glyph_h_advances_func_t`].
    ///
    /// Since HarfBuzz 1.8.6.
    pub fn hb_font_funcs_set_glyph_h_advances_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_h_advances_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_glyph_v_advances_func_t`].
    ///
    /// Since HarfBuzz 1.8.6.
    pub fn hb_font_funcs_set_glyph_v_advances_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_v_advances_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_glyph_h_origin_func_t`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_set_glyph_h_origin_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_h_origin_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_glyph_v_origin_func_t`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_set_glyph_v_origin_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_v_origin_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_glyph_h_origins_func_t`].
    ///
    /// Since HarfBuzz 11.3.0.
    pub fn hb_font_funcs_set_glyph_h_origins_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_h_origins_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_glyph_v_origins_func_t`].
    ///
    /// Since HarfBuzz 11.3.0.
    pub fn hb_font_funcs_set_glyph_v_origins_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_v_origins_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_glyph_h_kerning_func_t`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_set_glyph_h_kerning_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_h_kerning_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_glyph_extents_func_t`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_set_glyph_extents_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_extents_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_glyph_contour_point_func_t`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_set_glyph_contour_point_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_contour_point_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_glyph_name_func_t`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_set_glyph_name_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_name_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_get_glyph_from_name_func_t`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_funcs_set_glyph_from_name_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_from_name_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_draw_glyph_or_fail_func_t`].
    ///
    /// Since HarfBuzz 11.2.0.
    pub fn hb_font_funcs_set_draw_glyph_or_fail_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_draw_glyph_or_fail_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_paint_glyph_or_fail_func_t`].
    ///
    /// Since HarfBuzz 11.2.0.
    pub fn hb_font_funcs_set_paint_glyph_or_fail_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_paint_glyph_or_fail_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    // --- Method dispatch ----------------------------------------------------
    //
    // These call straight through to the font's installed font-functions.

    /// Fetches the extents of a font, for horizontal text segments.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 1.1.3.
    pub fn hb_font_get_h_extents(
        font: *mut hb_font_t,
        extents: *mut hb_font_extents_t,
    ) -> hb_bool_t;

    /// Fetches the extents of a font, for vertical text segments.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 1.1.3.
    pub fn hb_font_get_v_extents(
        font: *mut hb_font_t,
        extents: *mut hb_font_extents_t,
    ) -> hb_bool_t;

    /// Fetches the nominal glyph ID for a Unicode code point.
    ///
    /// Do not use this to look up code points modified by a variation selector;
    /// use [`hb_font_get_variation_glyph`] or [`hb_font_get_glyph`] instead.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 1.2.3.
    pub fn hb_font_get_nominal_glyph(
        font: *mut hb_font_t,
        unicode: hb_codepoint_t,
        glyph: *mut hb_codepoint_t,
    ) -> hb_bool_t;

    /// Fetches the glyph ID for a Unicode code point when it is followed by the
    /// specified variation-selector code point.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 1.2.3.
    pub fn hb_font_get_variation_glyph(
        font: *mut hb_font_t,
        unicode: hb_codepoint_t,
        variation_selector: hb_codepoint_t,
        glyph: *mut hb_codepoint_t,
    ) -> hb_bool_t;

    /// Fetches the nominal glyph IDs for a sequence of Unicode code points,
    /// stopping at the first unsupported one.
    ///
    /// Returns the number of code points processed.
    ///
    /// Since HarfBuzz 2.6.3.
    pub fn hb_font_get_nominal_glyphs(
        font: *mut hb_font_t,
        count: c_uint,
        first_unicode: *const hb_codepoint_t,
        unicode_stride: c_uint,
        first_glyph: *mut hb_codepoint_t,
        glyph_stride: c_uint,
    ) -> c_uint;

    /// Fetches the advance of a glyph, for horizontal text segments.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_h_advance(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
    ) -> hb_position_t;

    /// Fetches the advance of a glyph, for vertical text segments.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_v_advance(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
    ) -> hb_position_t;

    /// Fetches the advances of a sequence of glyphs, for horizontal text
    /// segments.
    ///
    /// Since HarfBuzz 1.8.6.
    pub fn hb_font_get_glyph_h_advances(
        font: *mut hb_font_t,
        count: c_uint,
        first_glyph: *const hb_codepoint_t,
        glyph_stride: c_uint,
        first_advance: *mut hb_position_t,
        advance_stride: c_uint,
    );

    /// Fetches the advances of a sequence of glyphs, for vertical text
    /// segments.
    ///
    /// Since HarfBuzz 1.8.6.
    pub fn hb_font_get_glyph_v_advances(
        font: *mut hb_font_t,
        count: c_uint,
        first_glyph: *const hb_codepoint_t,
        glyph_stride: c_uint,
        first_advance: *mut hb_position_t,
        advance_stride: c_uint,
    );

    /// Fetches the (X, Y) coordinates of a glyph's origin, for horizontal text
    /// segments.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_h_origin(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        x: *mut hb_position_t,
        y: *mut hb_position_t,
    ) -> hb_bool_t;

    /// Fetches the (X, Y) coordinates of a glyph's origin, for vertical text
    /// segments.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_v_origin(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        x: *mut hb_position_t,
        y: *mut hb_position_t,
    ) -> hb_bool_t;

    /// Fetches the origins of a sequence of glyphs, for horizontal text
    /// segments.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 11.3.0.
    pub fn hb_font_get_glyph_h_origins(
        font: *mut hb_font_t,
        count: c_uint,
        first_glyph: *const hb_codepoint_t,
        glyph_stride: c_uint,
        first_x: *mut hb_position_t,
        x_stride: c_uint,
        first_y: *mut hb_position_t,
        y_stride: c_uint,
    ) -> hb_bool_t;

    /// Fetches the origins of a sequence of glyphs, for vertical text segments.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 11.3.0.
    pub fn hb_font_get_glyph_v_origins(
        font: *mut hb_font_t,
        count: c_uint,
        first_glyph: *const hb_codepoint_t,
        glyph_stride: c_uint,
        first_x: *mut hb_position_t,
        x_stride: c_uint,
        first_y: *mut hb_position_t,
        y_stride: c_uint,
    ) -> hb_bool_t;

    /// Fetches the kerning adjustment for a glyph pair, for horizontal text
    /// segments.
    ///
    /// This handles legacy `kern`-table kerning only — whatever the
    /// corresponding [`hb_font_funcs_t`] method returns. OpenType `GPOS`
    /// kerning is applied during shaping instead.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_h_kerning(
        font: *mut hb_font_t,
        left_glyph: hb_codepoint_t,
        right_glyph: hb_codepoint_t,
    ) -> hb_position_t;

    /// Fetches the [`hb_glyph_extents_t`] of a glyph.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_extents(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        extents: *mut hb_glyph_extents_t,
    ) -> hb_bool_t;

    /// Fetches the (X, Y) coordinates of a contour point of a glyph.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_contour_point(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        point_index: c_uint,
        x: *mut hb_position_t,
        y: *mut hb_position_t,
    ) -> hb_bool_t;

    /// Fetches the glyph-name string for a glyph ID, writing it into the
    /// caller's `name` buffer of `size` bytes.
    ///
    /// The OpenType specification limits glyph names to 63 characters drawn
    /// from a subset of ASCII.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_name(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        name: *mut c_char,
        size: c_uint,
    ) -> hb_bool_t;

    /// Fetches the glyph ID matching a glyph-name string. A `len` of `-1` means
    /// `name` is NUL-terminated.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_from_name(
        font: *mut hb_font_t,
        name: *const c_char,
        len: c_int,
        glyph: *mut hb_codepoint_t,
    ) -> hb_bool_t;

    /// Draws the outline of a glyph, reporting failure.
    ///
    /// The outline is delivered through calls to the callbacks of `dfuncs`,
    /// each of which receives `draw_data`. Returns false if the font has no
    /// outline for `glyph`.
    ///
    /// Since HarfBuzz 11.2.0.
    pub fn hb_font_draw_glyph_or_fail(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
    ) -> hb_bool_t;

    /// Paints a color glyph, reporting failure.
    ///
    /// Succeeds if `glyph` has `COLRv0` paint layers, a `COLRv1` paint graph,
    /// or a bitmap image that the font's callbacks render successfully. Returns
    /// false when the font has no color data for `glyph`; the caller can then
    /// fall back to [`hb_font_draw_glyph_or_fail`] for the monochrome outline.
    ///
    /// Painting instructions are delivered through calls to the callbacks of
    /// `pfuncs`, each of which receives `paint_data`. If the font has color
    /// palettes, `palette_index` selects which one to use; it is 0 for a font
    /// with a single palette. `foreground` is unpremultiplied.
    ///
    /// Since HarfBuzz 11.2.0.
    pub fn hb_font_paint_glyph_or_fail(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        pfuncs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        palette_index: c_uint,
        foreground: hb_color_t,
    ) -> hb_bool_t;

    // --- High-level queries, with fallback ----------------------------------

    /// Fetches the glyph ID for a Unicode code point, with an optional
    /// variation selector.
    ///
    /// Calls [`hb_font_get_nominal_glyph`] when `variation_selector` is 0, and
    /// [`hb_font_get_variation_glyph`] otherwise.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph(
        font: *mut hb_font_t,
        unicode: hb_codepoint_t,
        variation_selector: hb_codepoint_t,
        glyph: *mut hb_codepoint_t,
    ) -> hb_bool_t;

    /// Fetches the extents of a font for a text segment of the given direction,
    /// dispatching to the horizontal or vertical variant as appropriate.
    ///
    /// Since HarfBuzz 1.1.3.
    pub fn hb_font_get_extents_for_direction(
        font: *mut hb_font_t,
        direction: hb_direction_t,
        extents: *mut hb_font_extents_t,
    );

    /// Fetches the advance of a glyph for a text segment of the given
    /// direction, dispatching to the horizontal or vertical variant as
    /// appropriate.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_advance_for_direction(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        direction: hb_direction_t,
        x: *mut hb_position_t,
        y: *mut hb_position_t,
    );

    /// Fetches the advances of a sequence of glyphs for a text segment of the
    /// given direction, dispatching to the horizontal or vertical variant as
    /// appropriate.
    ///
    /// Since HarfBuzz 1.8.6.
    pub fn hb_font_get_glyph_advances_for_direction(
        font: *mut hb_font_t,
        direction: hb_direction_t,
        count: c_uint,
        first_glyph: *const hb_codepoint_t,
        glyph_stride: c_uint,
        first_advance: *mut hb_position_t,
        advance_stride: c_uint,
    );

    /// Fetches the (X, Y) coordinates of a glyph's origin for a text segment of
    /// the given direction, dispatching to the horizontal or vertical variant
    /// as appropriate.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_origin_for_direction(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        direction: hb_direction_t,
        x: *mut hb_position_t,
        y: *mut hb_position_t,
    );

    /// Adds a glyph's origin coordinates to the (X, Y) point in `x` and `y`,
    /// in place, for a text segment of the given direction.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_add_glyph_origin_for_direction(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        direction: hb_direction_t,
        x: *mut hb_position_t,
        y: *mut hb_position_t,
    );

    /// Subtracts a glyph's origin coordinates from the (X, Y) point in `x` and
    /// `y`, in place, for a text segment of the given direction.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_subtract_glyph_origin_for_direction(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        direction: hb_direction_t,
        x: *mut hb_position_t,
        y: *mut hb_position_t,
    );

    /// Fetches the kerning adjustment for a glyph pair for a text segment of
    /// the given direction, dispatching to the horizontal or vertical variant
    /// as appropriate.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_kerning_for_direction(
        font: *mut hb_font_t,
        first_glyph: hb_codepoint_t,
        second_glyph: hb_codepoint_t,
        direction: hb_direction_t,
        x: *mut hb_position_t,
        y: *mut hb_position_t,
    );

    /// Fetches the [`hb_glyph_extents_t`] of a glyph relative to the origin for
    /// a text segment of the given direction.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_extents_for_origin(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        direction: hb_direction_t,
        extents: *mut hb_glyph_extents_t,
    ) -> hb_bool_t;

    /// Fetches the (X, Y) coordinates of a contour point of a glyph relative to
    /// the origin for a text segment of the given direction.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_glyph_contour_point_for_origin(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        point_index: c_uint,
        direction: hb_direction_t,
        x: *mut hb_position_t,
        y: *mut hb_position_t,
    ) -> hb_bool_t;

    /// Writes the name of a glyph ID into the caller's buffer `s` of `size`
    /// bytes.
    ///
    /// If the glyph has no name in the font, a string of the form `gidDDD` is
    /// generated, with `DDD` being the glyph ID.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_glyph_to_string(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        s: *mut c_char,
        size: c_uint,
    );

    /// Fetches the glyph ID matching a string. Strings of the form `gidDDD` and
    /// `uniUUUU` are parsed automatically. A `len` of `-1` means `s` is
    /// NUL-terminated.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_glyph_from_string(
        font: *mut hb_font_t,
        s: *const c_char,
        len: c_int,
        glyph: *mut hb_codepoint_t,
    ) -> hb_bool_t;

    /// Draws the outline of a glyph.
    ///
    /// This is the older name for [`hb_font_draw_glyph_or_fail`], with no
    /// return value — failure is silent.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_font_draw_glyph(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
    );

    /// Paints a glyph, falling back to the monochrome outline.
    ///
    /// Like [`hb_font_paint_glyph_or_fail`], except that when painting a color
    /// glyph fails this paints the outline glyph instead, so there is no
    /// failure to report.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_font_paint_glyph(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        pfuncs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        palette_index: c_uint,
        foreground: hb_color_t,
    );

    // --- Font lifecycle -----------------------------------------------------

    /// Constructs a new font object from a face. Fonts are very lightweight
    /// objects.
    ///
    /// If the face's index (as passed to `hb_face_create`) has non-zero top 16
    /// bits, those bits minus one are passed to
    /// [`hb_font_set_var_named_instance`], loading a named instance of a
    /// variable font rather than the default instance.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_create(face: *mut hb_face_t) -> *mut hb_font_t;

    /// Constructs a sub-font from `parent`, replicating the parent's
    /// properties.
    ///
    /// The sub-font's default font functions delegate to `parent`, so
    /// overriding one method on the sub-font leaves the rest inherited.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_create_sub_font(parent: *mut hb_font_t) -> *mut hb_font_t;

    /// Fetches the empty font object — a shared, immutable singleton.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_empty() -> *mut hb_font_t;

    /// Increases the reference count on a font object and returns it.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_reference(font: *mut hb_font_t) -> *mut hb_font_t;

    /// Decreases the reference count on a font object. When the count reaches
    /// zero the font is destroyed, freeing all its memory.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_destroy(font: *mut hb_font_t);

    /// Attaches a user-data key/data pair to a font object.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_set_user_data(
        font: *mut hb_font_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to a font object under the given key. The
    /// pointer is not transferred to the caller.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_user_data(
        font: *const hb_font_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Makes a font immutable. Later setter calls on it are ignored.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_make_immutable(font: *mut hb_font_t);

    /// Tests whether a font object is immutable.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_is_immutable(font: *mut hb_font_t) -> hb_bool_t;

    /// Returns the font's internal serial number, which increases every time a
    /// setter changes a setting on the font.
    ///
    /// Since HarfBuzz 4.4.0.
    pub fn hb_font_get_serial(font: *mut hb_font_t) -> c_uint;

    /// Notifies the font that the underlying font data has changed.
    ///
    /// This increases the serial returned by [`hb_font_get_serial`], which
    /// invalidates HarfBuzz's internal caches for the font.
    ///
    /// Since HarfBuzz 4.4.0.
    pub fn hb_font_changed(font: *mut hb_font_t);

    // --- Font properties ----------------------------------------------------

    /// Sets the parent font of a font.
    ///
    /// Since HarfBuzz 1.0.5.
    pub fn hb_font_set_parent(font: *mut hb_font_t, parent: *mut hb_font_t);

    /// Fetches the parent font of a font. The reference is not transferred to
    /// the caller.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_parent(font: *mut hb_font_t) -> *mut hb_font_t;

    /// Sets the face of a font.
    ///
    /// Since HarfBuzz 1.4.3.
    pub fn hb_font_set_face(font: *mut hb_font_t, face: *mut hb_face_t);

    /// Fetches the face associated with a font. The reference is not
    /// transferred to the caller.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_face(font: *mut hb_font_t) -> *mut hb_face_t;

    /// Replaces the font-functions structure attached to a font, and updates
    /// the font's user data to `font_data` with the given `destroy` callback.
    ///
    /// `destroy` may be null. If the font is immutable the call is a no-op,
    /// except that `destroy` is invoked on `font_data` immediately.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_set_funcs(
        font: *mut hb_font_t,
        klass: *mut hb_font_funcs_t,
        font_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Replaces only the user data attached to a font, updating the font's
    /// `destroy` callback, without touching its font-functions structure.
    ///
    /// Be *very* careful with this: the installed font functions must be able
    /// to interpret the new `font_data`.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_set_funcs_data(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the font-functions structure to use for a font, by name.
    ///
    /// If `name` is null or empty, the default (first) working font functions
    /// are used. That default can be changed by setting the `HB_FONT_FUNCS`
    /// environment variable to the desired name. See [`hb_font_list_funcs`] for
    /// the available names.
    ///
    /// Returns true if the named font functions were found and set, false
    /// otherwise.
    ///
    /// Since HarfBuzz 11.0.0.
    pub fn hb_font_set_funcs_using(font: *mut hb_font_t, name: *const c_char) -> hb_bool_t;

    /// Retrieves the list of font-functions implementations supported by this
    /// build of HarfBuzz.
    ///
    /// Returns a null-terminated array of NUL-terminated strings. The array is
    /// owned by HarfBuzz and must not be modified or freed.
    ///
    /// Since HarfBuzz 11.0.0.
    pub fn hb_font_list_funcs() -> *mut *const c_char;

    /// Sets the horizontal and vertical scale of a font.
    ///
    /// The font scale is related to, but not the same as, font size. Because
    /// [`hb_position_t`] is an integer, the client typically establishes a
    /// fixed-point factor — 64 or 256, say — and multiplies the nominal size by
    /// it. To render at size 20 with 64 levels of fractional precision, call
    /// `hb_font_set_scale(font, 20 * 64, 20 * 64)`.
    ///
    /// What "size 20" means is up to the client: pixels, points, millimetres.
    /// HarfBuzz does not care. The scale must simply be consistent with what
    /// the client expects out of [`hb_position_t`] values and out of the draw
    /// and paint APIs.
    ///
    /// A font defaults to a scale equal to the units-per-em of its face; such a
    /// font is sometimes called "unscaled".
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_set_scale(font: *mut hb_font_t, x_scale: c_int, y_scale: c_int);

    /// Fetches the horizontal and vertical scale of a font.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_scale(font: *mut hb_font_t, x_scale: *mut c_int, y_scale: *mut c_int);

    /// Sets the horizontal and vertical pixels-per-em of a font.
    ///
    /// These drive pixel-size-specific adjustments to shaping and draw results.
    /// For the most part they are unused and can be left unset; a zero value
    /// means "no hinting in that direction".
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_set_ppem(font: *mut hb_font_t, x_ppem: c_uint, y_ppem: c_uint);

    /// Fetches the horizontal and vertical pixels-per-em of a font.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_font_get_ppem(font: *mut hb_font_t, x_ppem: *mut c_uint, y_ppem: *mut c_uint);

    /// Sets the point size per em of a font. Set to zero to unset.
    ///
    /// Used by CoreText to implement optical sizing. There are 72 points in an
    /// inch.
    ///
    /// Since HarfBuzz 1.6.0.
    pub fn hb_font_set_ptem(font: *mut hb_font_t, ptem: c_float);

    /// Fetches the point size per em of a font. A value of zero means "not
    /// set".
    ///
    /// Since HarfBuzz 1.6.0.
    pub fn hb_font_get_ptem(font: *mut hb_font_t) -> c_float;

    /// Tests whether a font is synthetic — that is, whether it has synthetic
    /// slant or synthetic bold set on it.
    ///
    /// Since HarfBuzz 11.2.0.
    pub fn hb_font_is_synthetic(font: *mut hb_font_t) -> hb_bool_t;

    /// Sets the synthetic boldness of a font.
    ///
    /// Positive values make a font bolder, negative values thinner; typical
    /// values are in the 0.01 to 0.05 range, and the default is zero. Synthetic
    /// boldness is applied by offsetting the contour points of the glyph shape,
    /// and takes effect when rendering a glyph via
    /// [`hb_font_draw_glyph_or_fail`].
    ///
    /// If `in_place` is false, glyph advance widths are adjusted too; if true
    /// they are left alone, which is useful for simulating font grading.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_font_set_synthetic_bold(
        font: *mut hb_font_t,
        x_embolden: c_float,
        y_embolden: c_float,
        in_place: hb_bool_t,
    );

    /// Fetches the synthetic-boldness parameters of a font.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_font_get_synthetic_bold(
        font: *mut hb_font_t,
        x_embolden: *mut c_float,
        y_embolden: *mut c_float,
        in_place: *mut hb_bool_t,
    );

    /// Sets the synthetic slant of a font — the graphical skew applied to it at
    /// rendering time. The default is zero.
    ///
    /// HarfBuzz needs this value in order to adjust shaping results, metrics,
    /// and style values to match the slanted rendering, and the glyph shape
    /// fetched via [`hb_font_draw_glyph_or_fail`] is slanted to reflect it too.
    ///
    /// The value is a ratio: a 20% slant is 0.2.
    ///
    /// Since HarfBuzz 3.3.0.
    pub fn hb_font_set_synthetic_slant(font: *mut hb_font_t, slant: c_float);

    /// Fetches the synthetic slant of a font. The default is zero.
    ///
    /// Since HarfBuzz 3.3.0.
    pub fn hb_font_get_synthetic_slant(font: *mut hb_font_t) -> c_float;

    // --- Variations ---------------------------------------------------------

    /// Applies a list of font-variation settings to a font.
    ///
    /// This overrides all existing variations on the font: axes not named in
    /// `variations` are set to their default values.
    ///
    /// Since HarfBuzz 1.4.2.
    pub fn hb_font_set_variations(
        font: *mut hb_font_t,
        variations: *const hb_variation_t,
        variations_length: c_uint,
    );

    /// Changes the value of a single variation axis on a font.
    ///
    /// This is expensive to call repeatedly; to set several axes at once, use
    /// [`hb_font_set_variations`] instead.
    ///
    /// Since HarfBuzz 7.1.0.
    pub fn hb_font_set_variation(font: *mut hb_font_t, tag: hb_tag_t, value: c_float);

    /// Applies a list of variation coordinates, in design-space units, to a
    /// font.
    ///
    /// This overrides all existing variations on the font: axes beyond
    /// `coords_length` are set to their default values.
    ///
    /// Since HarfBuzz 1.4.2.
    pub fn hb_font_set_var_coords_design(
        font: *mut hb_font_t,
        coords: *const c_float,
        coords_length: c_uint,
    );

    /// Fetches the design-space variation coordinates currently set on a font,
    /// writing the count into `length`.
    ///
    /// May return null when no variation coordinates are set. If variations
    /// were set with [`hb_font_set_var_coords_normalized`], the design
    /// coordinates are NaN. The returned pointer stays valid only as long as
    /// the font's variation coordinates are not modified.
    ///
    /// Since HarfBuzz 3.3.0.
    pub fn hb_font_get_var_coords_design(
        font: *mut hb_font_t,
        length: *mut c_uint,
    ) -> *const c_float;

    /// Applies a list of variation coordinates, in normalized 2.14 fixed-point
    /// units, to a font.
    ///
    /// This overrides all existing variations on the font: axes beyond
    /// `coords_length` are set to their default values.
    ///
    /// Since HarfBuzz 1.4.2.
    pub fn hb_font_set_var_coords_normalized(
        font: *mut hb_font_t,
        coords: *const c_int,
        coords_length: c_uint,
    );

    /// Fetches the normalized variation coordinates currently set on a font,
    /// writing the count into `length`.
    ///
    /// May return null when no variation coordinates are set. The returned
    /// pointer stays valid only as long as the font's variation coordinates are
    /// not modified.
    ///
    /// Since HarfBuzz 1.4.2.
    pub fn hb_font_get_var_coords_normalized(
        font: *mut hb_font_t,
        length: *mut c_uint,
    ) -> *const c_int;

    /// Sets the design coordinates of a font from a named-instance index.
    ///
    /// Since HarfBuzz 2.6.0.
    pub fn hb_font_set_var_named_instance(font: *mut hb_font_t, instance_index: c_uint);

    /// Returns the currently-set named-instance index of a font, or
    /// [`HB_FONT_NO_VAR_NAMED_INSTANCE`] if none is set.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_font_get_var_named_instance(font: *mut hb_font_t) -> c_uint;
}
