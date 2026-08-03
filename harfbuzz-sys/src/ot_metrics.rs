//! Font-wide OpenType metrics — ascender, x-height, underline, and friends — `hb-ot-metrics.h`.
//!
//! One tag enumeration naming the metrics an OpenType font can record, and a
//! handful of functions that read them off an [`hb_font_t`] with the font's
//! variation settings already applied.

use core::ffi::{c_float, c_int};

use crate::{HB_TAG, hb_bool_t, hb_font_t, hb_position_t};

/// A font-wide metric to fetch.
///
/// Every value is an [`hb_tag_t`](crate::hb_tag_t) taken from the
/// [MVAR value tag registry](https://docs.microsoft.com/en-us/typography/opentype/spec/mvar#value-tags).
/// The tag identifies both where the base value lives in the font's legacy
/// tables (`hhea`, `vhea`, OS/2, `post`) and which `MVAR` record varies it.
///
/// The alias is a signed C `int` because the C enumeration ends with a private
/// sentinel equal to [`HB_TAG_MAX_SIGNED`](crate::HB_TAG_MAX_SIGNED), which
/// pins the underlying type. Tags outside the list below are accepted by the
/// API and simply report "not present", so this is an open value space — which
/// is why it is an integer alias and not a Rust `enum`.
///
/// Since HarfBuzz 2.6.0.
pub type hb_ot_metrics_tag_t = c_int;

/// Horizontal ascender (`hasc`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_HORIZONTAL_ASCENDER: hb_ot_metrics_tag_t =
    HB_TAG(b'h', b'a', b's', b'c') as hb_ot_metrics_tag_t;

/// Horizontal descender (`hdsc`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_HORIZONTAL_DESCENDER: hb_ot_metrics_tag_t =
    HB_TAG(b'h', b'd', b's', b'c') as hb_ot_metrics_tag_t;

/// Horizontal line gap (`hlgp`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_HORIZONTAL_LINE_GAP: hb_ot_metrics_tag_t =
    HB_TAG(b'h', b'l', b'g', b'p') as hb_ot_metrics_tag_t;

/// Horizontal clipping ascent (`hcla`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_HORIZONTAL_CLIPPING_ASCENT: hb_ot_metrics_tag_t =
    HB_TAG(b'h', b'c', b'l', b'a') as hb_ot_metrics_tag_t;

/// Horizontal clipping descent (`hcld`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_HORIZONTAL_CLIPPING_DESCENT: hb_ot_metrics_tag_t =
    HB_TAG(b'h', b'c', b'l', b'd') as hb_ot_metrics_tag_t;

/// Vertical ascender (`vasc`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_VERTICAL_ASCENDER: hb_ot_metrics_tag_t =
    HB_TAG(b'v', b'a', b's', b'c') as hb_ot_metrics_tag_t;

/// Vertical descender (`vdsc`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_VERTICAL_DESCENDER: hb_ot_metrics_tag_t =
    HB_TAG(b'v', b'd', b's', b'c') as hb_ot_metrics_tag_t;

/// Vertical line gap (`vlgp`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_VERTICAL_LINE_GAP: hb_ot_metrics_tag_t =
    HB_TAG(b'v', b'l', b'g', b'p') as hb_ot_metrics_tag_t;

/// Horizontal caret rise (`hcrs`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_HORIZONTAL_CARET_RISE: hb_ot_metrics_tag_t =
    HB_TAG(b'h', b'c', b'r', b's') as hb_ot_metrics_tag_t;

/// Horizontal caret run (`hcrn`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_HORIZONTAL_CARET_RUN: hb_ot_metrics_tag_t =
    HB_TAG(b'h', b'c', b'r', b'n') as hb_ot_metrics_tag_t;

/// Horizontal caret offset (`hcof`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_HORIZONTAL_CARET_OFFSET: hb_ot_metrics_tag_t =
    HB_TAG(b'h', b'c', b'o', b'f') as hb_ot_metrics_tag_t;

/// Vertical caret rise (`vcrs`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_VERTICAL_CARET_RISE: hb_ot_metrics_tag_t =
    HB_TAG(b'v', b'c', b'r', b's') as hb_ot_metrics_tag_t;

/// Vertical caret run (`vcrn`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_VERTICAL_CARET_RUN: hb_ot_metrics_tag_t =
    HB_TAG(b'v', b'c', b'r', b'n') as hb_ot_metrics_tag_t;

/// Vertical caret offset (`vcof`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_VERTICAL_CARET_OFFSET: hb_ot_metrics_tag_t =
    HB_TAG(b'v', b'c', b'o', b'f') as hb_ot_metrics_tag_t;

/// x-height (`xhgt`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_X_HEIGHT: hb_ot_metrics_tag_t =
    HB_TAG(b'x', b'h', b'g', b't') as hb_ot_metrics_tag_t;

/// Cap height (`cpht`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_CAP_HEIGHT: hb_ot_metrics_tag_t =
    HB_TAG(b'c', b'p', b'h', b't') as hb_ot_metrics_tag_t;

/// Subscript em x size (`sbxs`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_SUBSCRIPT_EM_X_SIZE: hb_ot_metrics_tag_t =
    HB_TAG(b's', b'b', b'x', b's') as hb_ot_metrics_tag_t;

/// Subscript em y size (`sbys`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_SUBSCRIPT_EM_Y_SIZE: hb_ot_metrics_tag_t =
    HB_TAG(b's', b'b', b'y', b's') as hb_ot_metrics_tag_t;

/// Subscript em x offset (`sbxo`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_SUBSCRIPT_EM_X_OFFSET: hb_ot_metrics_tag_t =
    HB_TAG(b's', b'b', b'x', b'o') as hb_ot_metrics_tag_t;

/// Subscript em y offset (`sbyo`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_SUBSCRIPT_EM_Y_OFFSET: hb_ot_metrics_tag_t =
    HB_TAG(b's', b'b', b'y', b'o') as hb_ot_metrics_tag_t;

/// Superscript em x size (`spxs`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_SUPERSCRIPT_EM_X_SIZE: hb_ot_metrics_tag_t =
    HB_TAG(b's', b'p', b'x', b's') as hb_ot_metrics_tag_t;

/// Superscript em y size (`spys`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_SUPERSCRIPT_EM_Y_SIZE: hb_ot_metrics_tag_t =
    HB_TAG(b's', b'p', b'y', b's') as hb_ot_metrics_tag_t;

/// Superscript em x offset (`spxo`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_SUPERSCRIPT_EM_X_OFFSET: hb_ot_metrics_tag_t =
    HB_TAG(b's', b'p', b'x', b'o') as hb_ot_metrics_tag_t;

/// Superscript em y offset (`spyo`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_SUPERSCRIPT_EM_Y_OFFSET: hb_ot_metrics_tag_t =
    HB_TAG(b's', b'p', b'y', b'o') as hb_ot_metrics_tag_t;

/// Strikeout size (`strs`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_STRIKEOUT_SIZE: hb_ot_metrics_tag_t =
    HB_TAG(b's', b't', b'r', b's') as hb_ot_metrics_tag_t;

/// Strikeout offset (`stro`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_STRIKEOUT_OFFSET: hb_ot_metrics_tag_t =
    HB_TAG(b's', b't', b'r', b'o') as hb_ot_metrics_tag_t;

/// Underline size (`unds`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_UNDERLINE_SIZE: hb_ot_metrics_tag_t =
    HB_TAG(b'u', b'n', b'd', b's') as hb_ot_metrics_tag_t;

/// Underline offset (`undo`).
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_METRICS_TAG_UNDERLINE_OFFSET: hb_ot_metrics_tag_t =
    HB_TAG(b'u', b'n', b'd', b'o') as hb_ot_metrics_tag_t;

unsafe extern "C" {
    /// Fetches the metric value corresponding to `metrics_tag` from `font`.
    ///
    /// `position` is an out parameter and may be null when only the presence of
    /// the metric matters. It receives the value scaled into the font's
    /// coordinate space, with the font's variations already applied.
    ///
    /// Returns whether the requested metric was found in the font.
    ///
    /// Since HarfBuzz 2.6.0.
    pub fn hb_ot_metrics_get_position(
        font: *mut hb_font_t,
        metrics_tag: hb_ot_metrics_tag_t,
        position: *mut hb_position_t,
    ) -> hb_bool_t;

    /// Fetches the metric value corresponding to `metrics_tag` from `font`,
    /// synthesizing a value when it is missing from the font.
    ///
    /// `position` always receives a value, so — unlike
    /// [`hb_ot_metrics_get_position`] — there is nothing to report and no
    /// return value.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_ot_metrics_get_position_with_fallback(
        font: *mut hb_font_t,
        metrics_tag: hb_ot_metrics_tag_t,
        position: *mut hb_position_t,
    );

    /// Fetches the metric value corresponding to `metrics_tag` from `font` with
    /// the current font variation settings applied.
    ///
    /// The result is the `MVAR` delta in unscaled font design units, not the
    /// metric itself. Use [`hb_ot_metrics_get_x_variation`] or
    /// [`hb_ot_metrics_get_y_variation`] for a value scaled into the font's
    /// coordinate space.
    ///
    /// Since HarfBuzz 2.6.0.
    pub fn hb_ot_metrics_get_variation(
        font: *mut hb_font_t,
        metrics_tag: hb_ot_metrics_tag_t,
    ) -> c_float;

    /// Fetches the horizontal metric variation corresponding to `metrics_tag`
    /// from `font` with the current font variation settings applied.
    ///
    /// This is [`hb_ot_metrics_get_variation`] scaled by the font's horizontal
    /// scale.
    ///
    /// Since HarfBuzz 2.6.0.
    pub fn hb_ot_metrics_get_x_variation(
        font: *mut hb_font_t,
        metrics_tag: hb_ot_metrics_tag_t,
    ) -> hb_position_t;

    /// Fetches the vertical metric variation corresponding to `metrics_tag`
    /// from `font` with the current font variation settings applied.
    ///
    /// This is [`hb_ot_metrics_get_variation`] scaled by the font's vertical
    /// scale.
    ///
    /// Since HarfBuzz 2.6.0.
    pub fn hb_ot_metrics_get_y_variation(
        font: *mut hb_font_t,
        metrics_tag: hb_ot_metrics_tag_t,
    ) -> hb_position_t;
}
