//! Introspection of the OpenType shaper — `hb-ot-shape.h`.
//!
//! These entry points do not shape anything themselves. They report what the
//! `ot` shaper *would* do: which lookups and features a plan brings into play,
//! which glyphs a text run can reach, and which version of the internal buffer
//! format this library was built with.

use core::ffi::c_uint;

use crate::{hb_buffer_t, hb_feature_t, hb_font_t, hb_set_t, hb_shape_plan_t, hb_tag_t};

/// The serial number of the current internal buffer format.
///
/// The serial number increases whenever the private members of
/// [`hb_glyph_info_t`](crate::hb_glyph_info_t) and
/// [`hb_glyph_position_t`](crate::hb_glyph_position_t) change their format.
/// This is the value the headers were compiled against; compare it with
/// [`hb_ot_shape_get_buffer_format_serial`], which reports the value the linked
/// library was built with.
///
/// Since HarfBuzz 13.2.0.
pub const HB_OT_SHAPE_BUFFER_FORMAT_SERIAL: c_uint = 1;

unsafe extern "C" {
    /// Computes the transitive closure of glyphs needed to shape `buffer` with
    /// `font` under the given feature list.
    ///
    /// Every glyph the buffer's characters map to is added to `glyphs`, along
    /// with every glyph reachable from those through the GSUB lookups the `ot`
    /// shaper would run. Mirrored forms are included when the buffer's script
    /// is right-to-left. The closure is computed as a set, not as a list: it
    /// carries no order and no multiplicity.
    ///
    /// `glyphs` is an output parameter that is added to, not cleared first.
    /// `features` may be null, in which case `num_features` should be zero.
    /// Neither `font` nor `buffer` is modified.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_ot_shape_glyphs_closure(
        font: *mut hb_font_t,
        buffer: *mut hb_buffer_t,
        features: *const hb_feature_t,
        num_features: c_uint,
        glyphs: *mut hb_set_t,
    );

    /// Computes the complete set of GSUB or GPOS lookups that are applicable
    /// under `shape_plan`.
    ///
    /// `table_tag` selects the table and must be `HB_OT_TAG_GSUB` or
    /// `HB_OT_TAG_GPOS`; any other tag leaves `lookup_indexes` untouched. The
    /// lookup indexes are added to `lookup_indexes`, which is an output
    /// parameter that is not cleared first.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_ot_shape_plan_collect_lookups(
        shape_plan: *mut hb_shape_plan_t,
        table_tag: hb_tag_t,
        lookup_indexes: *mut hb_set_t,
    );

    /// Fetches the list of OpenType feature tags enabled for a shaping plan.
    ///
    /// `start_offset` is the index of the first tag to retrieve. `tag_count` is
    /// in/out: on entry it holds the capacity of `tags`, and on return the
    /// number actually written, which may be zero. Pass a null `tag_count` to
    /// query only the total; `tags` may also be null, in which case `tag_count`
    /// is still clamped to the number of tags available from `start_offset`.
    ///
    /// Returns the total number of feature tags in the plan, independently of
    /// `start_offset` and `tag_count`.
    ///
    /// Since HarfBuzz 10.3.0.
    pub fn hb_ot_shape_plan_get_feature_tags(
        shape_plan: *mut hb_shape_plan_t,
        start_offset: c_uint,
        tag_count: *mut c_uint,
        tags: *mut hb_tag_t,
    ) -> c_uint;

    /// Returns the serial number of the internal buffer format this library was
    /// built with.
    ///
    /// Compare it with [`HB_OT_SHAPE_BUFFER_FORMAT_SERIAL`], the value the
    /// headers declare, before relying on the private members of
    /// [`hb_glyph_info_t`](crate::hb_glyph_info_t) or
    /// [`hb_glyph_position_t`](crate::hb_glyph_position_t).
    ///
    /// Since HarfBuzz 13.2.0.
    pub fn hb_ot_shape_get_buffer_format_serial() -> c_uint;
}
