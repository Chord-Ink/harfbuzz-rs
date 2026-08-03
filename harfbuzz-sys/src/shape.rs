//! Conversion of text strings into positioned glyphs — `hb-shape.h`.
//!
//! Shaping is the central operation of HarfBuzz: it takes a buffer of Unicode
//! characters and a font, and replaces the buffer's contents with the glyphs
//! and positions the font calls for.

use core::ffi::{c_char, c_uint};

use crate::{hb_bool_t, hb_buffer_t, hb_feature_t, hb_font_t};

#[cfg(feature = "experimental")]
use core::ffi::c_float;

#[cfg(feature = "experimental")]
use crate::hb_tag_t;

unsafe extern "C" {
    /// Shapes `buffer` using `font`, turning its Unicode character content into
    /// positioned glyphs.
    ///
    /// If `features` is non-null it controls which font features are applied
    /// during shaping; `num_features` gives its length. When two features carry
    /// the same tag over overlapping ranges, the one at the higher index wins.
    ///
    /// This is [`hb_shape_full`] with the default shaper list, and it discards
    /// the success flag.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_shape(
        font: *mut hb_font_t,
        buffer: *mut hb_buffer_t,
        features: *const hb_feature_t,
        num_features: c_uint,
    );

    /// Shapes `buffer` using `font`, choosing among a caller-supplied list of
    /// shapers.
    ///
    /// Behaves like [`hb_shape`], except that when `shaper_list` is non-null
    /// the named shapers are tried in the given order instead of the default
    /// list. `shaper_list` is a null-terminated array of NUL-terminated shaper
    /// names, drawn from what [`hb_shape_list_shapers`] reports.
    ///
    /// Returns false if every shaper failed, and true otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_shape_full(
        font: *mut hb_font_t,
        buffer: *mut hb_buffer_t,
        features: *const hb_feature_t,
        num_features: c_uint,
        shaper_list: *const *const c_char,
    ) -> hb_bool_t;

    /// Shapes `buffer` and then justifies the result to a target advance by
    /// varying a font axis.
    ///
    /// Works like [`hb_shape_full`], but additionally searches for a value of a
    /// justification variation axis — `jstf` if the face has one, otherwise
    /// `wdth` — that brings the buffer's total advance between
    /// `min_target_advance` and `max_target_advance`. Which of the buffer's
    /// width or height is measured follows the buffer's direction.
    ///
    /// `font` must be mutable, because the search sets variation coordinates on
    /// it. `advance` is in/out: set `*advance` to the advance of the buffer as
    /// already shaped by [`hb_shape_full`] if it is known, or to zero to have
    /// this function compute it; on return it holds the achieved advance.
    /// `var_tag` and `var_value` are outputs receiving the axis used and the
    /// value settled on; both are set to [`HB_TAG_NONE`](crate::HB_TAG_NONE)
    /// and zero when no justification was needed or no suitable axis exists.
    ///
    /// Returns false if every shaper failed, and true otherwise.
    ///
    /// This entry point is experimental — it is compiled only with
    /// `HB_EXPERIMENTAL_API`, and upstream expects it to change.
    #[cfg(feature = "experimental")]
    pub fn hb_shape_justify(
        font: *mut hb_font_t,
        buffer: *mut hb_buffer_t,
        features: *const hb_feature_t,
        num_features: c_uint,
        shaper_list: *const *const c_char,
        min_target_advance: c_float,
        max_target_advance: c_float,
        advance: *mut c_float,
        var_tag: *mut hb_tag_t,
        var_value: *mut c_float,
    ) -> hb_bool_t;

    /// Retrieves the list of shapers supported by this build of HarfBuzz.
    ///
    /// Returns a null-terminated array of NUL-terminated shaper names. The
    /// array and its strings are owned by HarfBuzz and must be neither modified
    /// nor freed.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_shape_list_shapers() -> *mut *const c_char;
}
