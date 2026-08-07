//! API retained only for source and binary compatibility — `hb-deprecated.h`.
//!
//! Nothing here should be used in new code. Each item's documentation names the
//! supported replacement.

// Deprecated declarations necessarily refer to one another: a deprecated setter
// takes a deprecated callback type. Allowing the lint inside this module keeps
// the crate warning-free while leaving `#[deprecated]` fully in force at every
// call site outside it.
#![allow(deprecated)]

use core::ffi::{c_uint, c_void};

use crate::{
    HB_AAT_LAYOUT_FEATURE_TYPE_CURSIVE_CONNECTION, HB_BUFFER_FLAG_DEFAULT,
    HB_BUFFER_SERIALIZE_FLAG_DEFAULT, HB_SCRIPT_CANADIAN_SYLLABICS, hb_aat_layout_feature_type_t,
    hb_bool_t, hb_buffer_flags_t, hb_buffer_serialize_flags_t, hb_codepoint_t, hb_color_t,
    hb_destroy_func_t, hb_draw_funcs_t, hb_font_funcs_t, hb_font_get_glyph_kerning_func_t,
    hb_font_t, hb_paint_funcs_t, hb_position_t, hb_script_t, hb_unicode_combining_class_t,
    hb_unicode_funcs_t,
};

// --- Renamed constants ------------------------------------------------------
//
// Each of these is a plain alias for a constant that was given a better name in
// a later release. The value is identical, so switching costs nothing.

/// Old name for [`HB_SCRIPT_CANADIAN_SYLLABICS`].
///
/// The two are the same value; only the spelling changed.
#[deprecated(note = "renamed in HarfBuzz 0.9.20; use HB_SCRIPT_CANADIAN_SYLLABICS")]
pub const HB_SCRIPT_CANADIAN_ABORIGINAL: hb_script_t = HB_SCRIPT_CANADIAN_SYLLABICS;

/// Old name for [`HB_BUFFER_FLAG_DEFAULT`].
///
/// The two are the same value; only the spelling changed.
#[deprecated(note = "renamed in HarfBuzz 0.9.20; use HB_BUFFER_FLAG_DEFAULT")]
pub const HB_BUFFER_FLAGS_DEFAULT: hb_buffer_flags_t = HB_BUFFER_FLAG_DEFAULT;

/// Old name for [`HB_BUFFER_SERIALIZE_FLAG_DEFAULT`].
///
/// The two are the same value; only the spelling changed.
#[deprecated(note = "renamed in HarfBuzz 0.9.20; use HB_BUFFER_SERIALIZE_FLAG_DEFAULT")]
pub const HB_BUFFER_SERIALIZE_FLAGS_DEFAULT: hb_buffer_serialize_flags_t =
    HB_BUFFER_SERIALIZE_FLAG_DEFAULT;

/// Old, misspelled name for [`HB_AAT_LAYOUT_FEATURE_TYPE_CURSIVE_CONNECTION`].
///
/// Note the transposed letters in `CURISVE`. The two are the same value.
#[deprecated(
    note = "misspelling fixed in HarfBuzz 8.3.0; use HB_AAT_LAYOUT_FEATURE_TYPE_CURSIVE_CONNECTION"
)]
pub const HB_AAT_LAYOUT_FEATURE_TYPE_CURISVE_CONNECTION: hb_aat_layout_feature_type_t =
    HB_AAT_LAYOUT_FEATURE_TYPE_CURSIVE_CONNECTION;

/// Tibetan combining class 133.
///
/// Dropped from the canonical [`hb_unicode_combining_class_t`] list because no
/// Unicode character is assigned this class. It stays defined here so that code
/// naming the full range still compiles.
#[deprecated(note = "unassigned in Unicode; removed from the enumeration in HarfBuzz 7.2.0")]
pub const HB_UNICODE_COMBINING_CLASS_CCC133: hb_unicode_combining_class_t = 133;

// --- Combined nominal/variation glyph lookup --------------------------------

/// Retrieves the glyph ID for a Unicode code point, with an optional variation
/// selector.
///
/// A virtual method of [`hb_font_funcs_t`]. Returns true if data was found,
/// false otherwise; the glyph ID is written through `glyph`. A
/// `variation_selector` of zero means "no variation selector".
///
/// Superseded by the pair
/// [`hb_font_get_nominal_glyph_func_t`](crate::hb_font_get_nominal_glyph_func_t)
/// and
/// [`hb_font_get_variation_glyph_func_t`](crate::hb_font_get_variation_glyph_func_t),
/// which let HarfBuzz take a faster path when no variation selector is
/// involved.
#[deprecated(
    note = "deprecated in HarfBuzz 1.2.3; use hb_font_get_nominal_glyph_func_t and \
            hb_font_get_variation_glyph_func_t"
)]
pub type hb_font_get_glyph_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        unicode: hb_codepoint_t,
        variation_selector: hb_codepoint_t,
        glyph: *mut hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

// --- East Asian width -------------------------------------------------------

/// Retrieves the East Asian display width of a code point.
///
/// A virtual method of [`hb_unicode_funcs_t`]. HarfBuzz itself never calls it.
#[deprecated(note = "deprecated in HarfBuzz 2.0.0; unused by HarfBuzz")]
pub type hb_unicode_eastasian_width_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
        user_data: *mut c_void,
    ) -> c_uint,
>;

// --- Compatibility decomposition --------------------------------------------

/// Fully decomposes a code point to its Unicode compatibility decomposition.
///
/// The resulting code points are written to `decomposed`, and the length of the
/// decomposition is returned; zero means `u` has no compatibility
/// decomposition.
///
/// The Unicode standard guarantees that a buffer of
/// [`HB_UNICODE_MAX_DECOMPOSITION_LEN`] code points is always enough for any
/// compatibility decomposition plus a terminating zero, so the caller must
/// allocate at least that much. Implementations of this callback must not write
/// past the provided array.
#[deprecated(note = "deprecated in HarfBuzz 2.0.0; unused by HarfBuzz")]
pub type hb_unicode_decompose_compatibility_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        u: hb_codepoint_t,
        decomposed: *mut hb_codepoint_t,
        user_data: *mut c_void,
    ) -> c_uint,
>;

/// The longest compatibility decomposition Unicode defines, plus one slot for a
/// terminating zero.
///
/// See Unicode 6.1 for the derivation of the value 18.
#[deprecated(
    note = "deprecated in HarfBuzz 2.0.0; only useful with the compatibility-decomposition API, \
            which is itself deprecated"
)]
pub const HB_UNICODE_MAX_DECOMPOSITION_LEN: c_uint = 18 + 1;

// --- Vertical kerning -------------------------------------------------------

/// Retrieves the kerning adjustment for a glyph pair, for vertical text
/// segments.
///
/// A virtual method of [`hb_font_funcs_t`]. This is an alias for
/// [`hb_font_get_glyph_kerning_func_t`], the horizontal callback type, so its
/// parameters are named `first_glyph` and `second_glyph` rather than
/// `top_glyph` and `bottom_glyph`.
#[deprecated(
    note = "deprecated alongside hb_font_funcs_set_glyph_v_kerning_func in HarfBuzz 2.0.0"
)]
pub type hb_font_get_glyph_v_kerning_func_t = hb_font_get_glyph_kerning_func_t;

// --- Glyph outlines and colour glyphs ---------------------------------------

/// Draws the outline of a glyph by calling into `draw_funcs`.
///
/// A virtual method of [`hb_font_funcs_t`]. It has no return value, so an
/// implementation cannot report that it declined to draw the glyph.
///
/// Since HarfBuzz 4.0.0.
#[deprecated(note = "deprecated in HarfBuzz 7.0.0; use hb_font_draw_glyph_or_fail_func_t")]
pub type hb_font_get_glyph_shape_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        glyph: hb_codepoint_t,
        draw_funcs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        user_data: *mut c_void,
    ),
>;

/// Draws the outline of a glyph by calling into `draw_funcs`.
///
/// A virtual method of [`hb_font_funcs_t`]. Structurally identical to
/// [`hb_font_get_glyph_shape_func_t`], which it replaced; it too has no return
/// value and so cannot report failure.
///
/// Since HarfBuzz 7.0.0.
#[deprecated(note = "deprecated in HarfBuzz 11.2.0; use hb_font_draw_glyph_or_fail_func_t")]
pub type hb_font_draw_glyph_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        glyph: hb_codepoint_t,
        draw_funcs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        user_data: *mut c_void,
    ),
>;

/// Paints a colour glyph by calling into `paint_funcs`.
///
/// A virtual method of [`hb_font_funcs_t`]. Although the C signature returns
/// [`hb_bool_t`], HarfBuzz discards the value: the shim that adapts this
/// callback to the modern one reports success unconditionally.
///
/// Since HarfBuzz 7.0.0.
#[deprecated(note = "deprecated in HarfBuzz 11.2.0; use hb_font_paint_glyph_or_fail_func_t")]
pub type hb_font_paint_glyph_func_t = Option<
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

unsafe extern "C" {
    /// Sets the implementation of [`hb_font_get_glyph_func_t`].
    ///
    /// Internally this installs the same callback twice — once as the nominal
    /// glyph function, once as the variation glyph function — behind a shared
    /// trampoline. `destroy`, which may be null, is called with `user_data`
    /// exactly once, when both registrations have been released.
    ///
    /// Does nothing if `ffuncs` is immutable, or if allocating the trampoline
    /// fails; in either case `destroy` is called immediately.
    ///
    /// Since HarfBuzz 0.9.2.
    #[deprecated(
        note = "deprecated in HarfBuzz 1.2.3; use hb_font_funcs_set_nominal_glyph_func and \
                hb_font_funcs_set_variation_glyph_func"
    )]
    pub fn hb_font_funcs_set_glyph_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_unicode_eastasian_width_func_t`].
    ///
    /// `destroy`, which may be null, is called with `user_data` when the
    /// callback is replaced or `ufuncs` is destroyed. Passing a null `func`
    /// restores the parent's implementation.
    ///
    /// Does nothing if `ufuncs` is immutable, in which case `destroy` is called
    /// immediately.
    ///
    /// Since HarfBuzz 0.9.2.
    #[deprecated(note = "deprecated in HarfBuzz 2.0.0; unused by HarfBuzz")]
    pub fn hb_unicode_funcs_set_eastasian_width_func(
        ufuncs: *mut hb_unicode_funcs_t,
        func: hb_unicode_eastasian_width_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Fetches the East Asian display width of a code point.
    ///
    /// Not used by HarfBuzz for anything. The default implementation always
    /// returns 1.
    ///
    /// Since HarfBuzz 0.9.2.
    #[deprecated(note = "deprecated in HarfBuzz 2.0.0; unused by HarfBuzz")]
    pub fn hb_unicode_eastasian_width(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
    ) -> c_uint;

    /// Sets the implementation of
    /// [`hb_unicode_decompose_compatibility_func_t`].
    ///
    /// `destroy`, which may be null, is called with `user_data` when the
    /// callback is replaced or `ufuncs` is destroyed. Passing a null `func`
    /// restores the parent's implementation.
    ///
    /// Does nothing if `ufuncs` is immutable, in which case `destroy` is called
    /// immediately.
    ///
    /// Since HarfBuzz 0.9.2.
    #[deprecated(note = "deprecated in HarfBuzz 2.0.0; unused by HarfBuzz")]
    pub fn hb_unicode_funcs_set_decompose_compatibility_func(
        ufuncs: *mut hb_unicode_funcs_t,
        func: hb_unicode_decompose_compatibility_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Fetches the compatibility decomposition of a Unicode code point.
    ///
    /// `decomposed` must point to at least
    /// [`HB_UNICODE_MAX_DECOMPOSITION_LEN`] code points. On return it holds the
    /// decomposition followed by a terminating zero, and the length of the
    /// decomposition is returned. A single-element decomposition equal to `u`
    /// itself is reported as no decomposition: the return value is zero and
    /// `decomposed[0]` is set to zero.
    ///
    /// The default implementation always returns zero.
    ///
    /// Since HarfBuzz 0.9.2.
    #[deprecated(note = "deprecated in HarfBuzz 2.0.0; unused by HarfBuzz")]
    pub fn hb_unicode_decompose_compatibility(
        ufuncs: *mut hb_unicode_funcs_t,
        u: hb_codepoint_t,
        decomposed: *mut hb_codepoint_t,
    ) -> c_uint;

    /// Sets the implementation of [`hb_font_get_glyph_v_kerning_func_t`].
    ///
    /// `destroy`, which may be null, is called with `user_data` when the
    /// callback is replaced or `ffuncs` is destroyed. Passing a null `func`
    /// restores the built-in default, which delegates to the parent font.
    ///
    /// Does nothing if `ffuncs` is immutable, in which case `destroy` is called
    /// immediately.
    ///
    /// Since HarfBuzz 0.9.2.
    #[deprecated(note = "deprecated in HarfBuzz 2.0.0; vertical kerning is not supported")]
    pub fn hb_font_funcs_set_glyph_v_kerning_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_v_kerning_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Fetches the kerning adjustment for a glyph pair, for vertical text
    /// segments.
    ///
    /// Handles legacy kerning only — that is, whatever the corresponding
    /// [`hb_font_funcs_t`] callback returns. HarfBuzz ships no default
    /// implementation that reads a font table, so this returns zero unless a
    /// callback has been installed.
    ///
    /// Since HarfBuzz 0.9.2.
    #[deprecated(note = "deprecated in HarfBuzz 2.0.0; vertical kerning is not supported")]
    pub fn hb_font_get_glyph_v_kerning(
        font: *mut hb_font_t,
        top_glyph: hb_codepoint_t,
        bottom_glyph: hb_codepoint_t,
    ) -> hb_position_t;

    /// Sets the implementation of [`hb_font_get_glyph_shape_func_t`].
    ///
    /// The callback is wrapped in a shim that always reports success and
    /// installed as the modern draw-glyph function, so it and
    /// [`hb_font_funcs_set_draw_glyph_func`] overwrite each other.
    ///
    /// `destroy`, which may be null, is called with `user_data` when the
    /// callback is replaced or `ffuncs` is destroyed. Does nothing if `ffuncs`
    /// is immutable or allocation fails; in either case `destroy` is called
    /// immediately.
    ///
    /// Since HarfBuzz 4.0.0.
    #[deprecated(
        note = "deprecated in HarfBuzz 7.0.0; use hb_font_funcs_set_draw_glyph_or_fail_func"
    )]
    pub fn hb_font_funcs_set_glyph_shape_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_get_glyph_shape_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_draw_glyph_func_t`].
    ///
    /// The callback is wrapped in a shim that always reports success and
    /// installed as the modern draw-glyph function, so it and
    /// [`hb_font_funcs_set_glyph_shape_func`] overwrite each other.
    ///
    /// `destroy`, which may be null, is called with `user_data` when the
    /// callback is replaced or `ffuncs` is destroyed. Does nothing if `ffuncs`
    /// is immutable or allocation fails; in either case `destroy` is called
    /// immediately.
    ///
    /// Since HarfBuzz 7.0.0.
    #[deprecated(
        note = "deprecated in HarfBuzz 11.2.0; use hb_font_funcs_set_draw_glyph_or_fail_func"
    )]
    pub fn hb_font_funcs_set_draw_glyph_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_draw_glyph_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_font_paint_glyph_func_t`].
    ///
    /// The callback is wrapped in a shim that discards its return value and
    /// always reports success, then installed as the modern paint-glyph
    /// function.
    ///
    /// `destroy`, which may be null, is called with `user_data` when the
    /// callback is replaced or `ffuncs` is destroyed. Does nothing if `ffuncs`
    /// is immutable or allocation fails; in either case `destroy` is called
    /// immediately.
    ///
    /// Since HarfBuzz 7.0.0.
    #[deprecated(
        note = "deprecated in HarfBuzz 11.2.0; use hb_font_funcs_set_paint_glyph_or_fail_func"
    )]
    pub fn hb_font_funcs_set_paint_glyph_func(
        ffuncs: *mut hb_font_funcs_t,
        func: hb_font_paint_glyph_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Draws the outline of a glyph, reporting the result through calls to
    /// `dfuncs` with `draw_data` passed to each callback.
    ///
    /// Has no return value, so a font that cannot draw the glyph fails
    /// silently.
    ///
    /// Since HarfBuzz 4.0.0.
    #[deprecated(note = "deprecated in HarfBuzz 7.0.0; use hb_font_draw_glyph_or_fail")]
    pub fn hb_font_get_glyph_shape(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
    );
}
