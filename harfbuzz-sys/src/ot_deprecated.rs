//! Superseded OpenType API, kept for compatibility — `hb-ot-deprecated.h`.
//!
//! Nothing here should be used in new code; every item names its replacement.
//! The symbols remain exported because removing them would break ABI.

use core::ffi::{c_float, c_uint};

use crate::{
    HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER, HB_OT_TAG_MATH_SCRIPT, hb_bool_t, hb_face_t,
    hb_language_t, hb_ot_math_glyph_part_flags_t, hb_ot_name_id_t, hb_script_t, hb_tag_t,
};

/// Use [`HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER`] instead.
///
/// An unprefixed spelling that predates the `HB_OT_` naming convention; the two
/// constants have the same value.
///
/// Deprecated since HarfBuzz 2.5.1.
#[deprecated(note = "use `HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER` instead")]
pub const HB_MATH_GLYPH_PART_FLAG_EXTENDER: hb_ot_math_glyph_part_flags_t =
    HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER;

/// Use [`HB_SCRIPT_MATH`](crate::HB_SCRIPT_MATH) or [`HB_OT_TAG_MATH_SCRIPT`]
/// instead.
///
/// This is the OpenType script tag `math`, not an [`hb_script_t`]. Previous
/// versions of HarfBuzz's documentation recommended passing it to
/// [`hb_buffer_set_script`](crate::hb_buffer_set_script) to enable math
/// shaping, but that usage is no longer supported — use
/// [`HB_SCRIPT_MATH`](crate::HB_SCRIPT_MATH) for that.
///
/// Since HarfBuzz 1.3.3. Deprecated since HarfBuzz 3.4.0.
#[deprecated(note = "use `HB_SCRIPT_MATH` or `HB_OT_TAG_MATH_SCRIPT` instead")]
pub const HB_OT_MATH_SCRIPT: hb_tag_t = HB_OT_TAG_MATH_SCRIPT;

/// Do not use.
///
/// The sentinel that [`hb_ot_var_find_axis`] writes to its `axis_index`
/// argument before searching, and leaves there when the axis is not found.
///
/// Since HarfBuzz 1.4.2. Deprecated since HarfBuzz 2.2.0.
#[deprecated(note = "do not use")]
pub const HB_OT_VAR_NO_AXIS_INDEX: c_uint = 0xFFFFFFFF;

/// Use [`hb_ot_var_axis_info_t`](crate::hb_ot_var_axis_info_t) instead.
///
/// The original description of a variation axis. It carries no axis index and
/// no flags, so a caller cannot tell a hidden axis from a visible one.
///
/// Since HarfBuzz 1.4.2. Deprecated since HarfBuzz 2.2.0.
#[deprecated(note = "use `hb_ot_var_axis_info_t` instead")]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct hb_ot_var_axis_t {
    /// The tag identifying the design variation of the axis, such as `wght`.
    pub tag: hb_tag_t,
    /// The `name` table Name ID that provides display names for the axis.
    pub name_id: hb_ot_name_id_t,
    /// The minimum value on the variation axis that the font covers.
    pub min_value: c_float,
    /// The position on the variation axis corresponding to the font's defaults.
    pub default_value: c_float,
    /// The maximum value on the variation axis that the font covers.
    pub max_value: c_float,
}

// Every function below is deprecated, and two of them mention the equally
// deprecated `hb_ot_var_axis_t` in their signatures. Transcribing the C header
// faithfully means those uses are unavoidable.
#[allow(deprecated)]
unsafe extern "C" {
    /// Use [`hb_ot_layout_table_select_script`](crate::hb_ot_layout_table_select_script)
    /// instead.
    ///
    /// Like [`hb_ot_layout_table_find_script`](crate::hb_ot_layout_table_find_script),
    /// but takes a zero-terminated array of script tags to try in order of
    /// preference. `table_tag` must be
    /// [`HB_OT_TAG_GSUB`](crate::HB_OT_TAG_GSUB) or
    /// [`HB_OT_TAG_GPOS`](crate::HB_OT_TAG_GPOS).
    ///
    /// Returns true only when one of the requested scripts was found. On
    /// failure it still falls back to `DFLT`, then `dflt`, then `latn`, writing
    /// whichever it finds into the output parameters; if none of those is
    /// present either, `script_index` becomes
    /// [`HB_OT_LAYOUT_NO_SCRIPT_INDEX`](crate::HB_OT_LAYOUT_NO_SCRIPT_INDEX)
    /// and `chosen_script` becomes [`HB_TAG_NONE`](crate::HB_TAG_NONE).
    ///
    /// Both output pointers may be null.
    ///
    /// Deprecated since HarfBuzz 2.0.0.
    #[deprecated(note = "use `hb_ot_layout_table_select_script` instead")]
    pub fn hb_ot_layout_table_choose_script(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        script_tags: *const hb_tag_t,
        script_index: *mut c_uint,
        chosen_script: *mut hb_tag_t,
    ) -> hb_bool_t;

    /// Use [`hb_ot_layout_script_select_language`](crate::hb_ot_layout_script_select_language)
    /// instead.
    ///
    /// Fetches the index of `language_tag` in the face's `GSUB` or `GPOS`
    /// table, underneath the script at `script_index`.
    ///
    /// Returns true if the language tag was found, false otherwise. On failure
    /// it falls back to `dflt`, and if that is missing sets `language_index` to
    /// [`HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX`](crate::HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX).
    ///
    /// Since HarfBuzz 0.6.0. Deprecated since HarfBuzz 2.0.0.
    #[deprecated(note = "use `hb_ot_layout_script_select_language` instead")]
    pub fn hb_ot_layout_script_find_language(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        script_index: c_uint,
        language_tag: hb_tag_t,
        language_index: *mut c_uint,
    ) -> hb_bool_t;

    /// Use [`hb_ot_tags_from_script_and_language`](crate::hb_ot_tags_from_script_and_language)
    /// instead.
    ///
    /// Converts an [`hb_script_t`] to the one or two OpenType script tags a
    /// font might use for it. Both outputs are always written; either is set to
    /// `DFLT` when there is no corresponding tag.
    ///
    /// Since HarfBuzz 0.6.0. Deprecated since HarfBuzz 2.0.0.
    #[deprecated(note = "use `hb_ot_tags_from_script_and_language` instead")]
    pub fn hb_ot_tags_from_script(
        script: hb_script_t,
        script_tag_1: *mut hb_tag_t,
        script_tag_2: *mut hb_tag_t,
    );

    /// Use [`hb_ot_tags_from_script_and_language`](crate::hb_ot_tags_from_script_and_language)
    /// instead.
    ///
    /// Converts an [`hb_language_t`] to a single OpenType language tag,
    /// returning `dflt` when there is no match.
    ///
    /// Since HarfBuzz 0.6.0. Deprecated since HarfBuzz 2.0.0.
    #[deprecated(note = "use `hb_ot_tags_from_script_and_language` instead")]
    pub fn hb_ot_tag_from_language(language: hb_language_t) -> hb_tag_t;

    /// Use [`hb_ot_var_get_axis_infos`](crate::hb_ot_var_get_axis_infos)
    /// instead.
    ///
    /// Fetches a list of all variation axes in the face, beginning at
    /// `start_offset`.
    ///
    /// `axes_count` is in/out — capacity in, number written out — and nothing
    /// is written unless both it and `axes_array` are non-null. Returns the
    /// total number of axes in the face, regardless of `start_offset`.
    ///
    /// Since HarfBuzz 1.4.2. Deprecated since HarfBuzz 2.2.0.
    #[deprecated(note = "use `hb_ot_var_get_axis_infos` instead")]
    pub fn hb_ot_var_get_axes(
        face: *mut hb_face_t,
        start_offset: c_uint,
        axes_count: *mut c_uint,
        axes_array: *mut hb_ot_var_axis_t,
    ) -> c_uint;

    /// Use [`hb_ot_var_find_axis_info`](crate::hb_ot_var_find_axis_info)
    /// instead.
    ///
    /// Fetches the variation-axis information for `axis_tag` in the face.
    ///
    /// Returns true if the axis was found. `axis_index` — which may be null —
    /// receives the axis's position in the face's axis array, or
    /// [`HB_OT_VAR_NO_AXIS_INDEX`] if the tag was not found. `axis_info` is
    /// written only on success, and is not checked for null before that write.
    ///
    /// Since HarfBuzz 1.4.2. Deprecated since HarfBuzz 2.2.0.
    #[deprecated(note = "use `hb_ot_var_find_axis_info` instead")]
    pub fn hb_ot_var_find_axis(
        face: *mut hb_face_t,
        axis_tag: hb_tag_t,
        axis_index: *mut c_uint,
        axis_info: *mut hb_ot_var_axis_t,
    ) -> hb_bool_t;
}
