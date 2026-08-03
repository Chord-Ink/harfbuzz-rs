//! OpenType font variations — axes and named instances — `hb-ot-var.h`.
//!
//! These functions read the `fvar` and `avar` tables of a variable font: which
//! design-variation axes it has, which named instances it ships, and how to
//! turn user-facing design coordinates into the normalized coordinates a font
//! actually applies.

use core::ffi::{c_float, c_int, c_uint};

use crate::{HB_TAG, hb_bool_t, hb_face_t, hb_ot_name_id_t, hb_tag_t, hb_variation_t};

/// Registered tag for the roman/italic axis (`ital`).
pub const HB_OT_TAG_VAR_AXIS_ITALIC: hb_tag_t = HB_TAG(b'i', b't', b'a', b'l');

/// Registered tag for the optical-size axis (`opsz`).
///
/// Note: the optical-size axis supersedes the OpenType `size` feature.
pub const HB_OT_TAG_VAR_AXIS_OPTICAL_SIZE: hb_tag_t = HB_TAG(b'o', b'p', b's', b'z');

/// Registered tag for the slant axis (`slnt`).
pub const HB_OT_TAG_VAR_AXIS_SLANT: hb_tag_t = HB_TAG(b's', b'l', b'n', b't');

/// Registered tag for the width axis (`wdth`).
pub const HB_OT_TAG_VAR_AXIS_WIDTH: hb_tag_t = HB_TAG(b'w', b'd', b't', b'h');

/// Registered tag for the weight axis (`wght`).
pub const HB_OT_TAG_VAR_AXIS_WEIGHT: hb_tag_t = HB_TAG(b'w', b'g', b'h', b't');

/// Flags for [`hb_ot_var_axis_info_t`].
///
/// This is a bit field, so values are combined with bitwise or. At present
/// [`HB_OT_VAR_AXIS_FLAG_HIDDEN`] is the only flag defined.
///
/// The alias is a signed C `int` because the C enumeration ends with a private
/// sentinel equal to [`HB_TAG_MAX_SIGNED`](crate::HB_TAG_MAX_SIGNED)
/// (`0x7FFFFFFF`), which fits in an `int`.
///
/// Since HarfBuzz 2.2.0.
pub type hb_ot_var_axis_flags_t = c_int;

/// The axis should not be exposed directly in user interfaces.
///
/// Since HarfBuzz 2.2.0.
pub const HB_OT_VAR_AXIS_FLAG_HIDDEN: hb_ot_var_axis_flags_t = 0x00000001;

/// Data type for holding variation-axis values.
///
/// Filled in by [`hb_ot_var_get_axis_infos`] and [`hb_ot_var_find_axis_info`].
/// The caller allocates it; there is nothing to free.
///
/// The minimum, default, and maximum values are in un-normalized, user scales —
/// the numbers a person would type, such as `400` for regular weight.
///
/// Since HarfBuzz 2.2.0.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct hb_ot_var_axis_info_t {
    /// Index of the axis in the face's variation-axis array.
    pub axis_index: c_uint,
    /// The tag identifying the design variation of the axis, such as `wght`.
    pub tag: hb_tag_t,
    /// The `name` table Name ID that provides display names for the axis.
    pub name_id: hb_ot_name_id_t,
    /// The flags for the axis. At present the only flag defined is
    /// [`HB_OT_VAR_AXIS_FLAG_HIDDEN`].
    pub flags: hb_ot_var_axis_flags_t,
    /// The minimum value on the variation axis that the font covers.
    pub min_value: c_float,
    /// The position on the variation axis corresponding to the font's defaults.
    pub default_value: c_float,
    /// The maximum value on the variation axis that the font covers.
    pub max_value: c_float,
    /// Private padding. HarfBuzz zeroes it; clients must ignore it.
    pub reserved: c_uint,
}

unsafe extern "C" {
    /// Tests whether a face includes any OpenType variation data in the `fvar`
    /// table.
    ///
    /// Returns true if data was found, false otherwise.
    ///
    /// Since HarfBuzz 1.4.2.
    pub fn hb_ot_var_has_data(face: *mut hb_face_t) -> hb_bool_t;

    /// Fetches the number of OpenType variation axes included in the face.
    ///
    /// Since HarfBuzz 1.4.2.
    pub fn hb_ot_var_get_axis_count(face: *mut hb_face_t) -> c_uint;

    /// Fetches a list of all variation axes in the face, beginning at
    /// `start_offset`.
    ///
    /// On input `axes_count` holds the capacity of `axes_array`; on output it
    /// holds how many entries were actually written, which may be zero. Either
    /// pointer may be null, in which case nothing is written and only the count
    /// is returned.
    ///
    /// Returns the total number of variation axes in the face, regardless of
    /// how many were written.
    ///
    /// Since HarfBuzz 2.2.0.
    pub fn hb_ot_var_get_axis_infos(
        face: *mut hb_face_t,
        start_offset: c_uint,
        axes_count: *mut c_uint,
        axes_array: *mut hb_ot_var_axis_info_t,
    ) -> c_uint;

    /// Fetches the variation-axis information for `axis_tag` in the face.
    ///
    /// Returns true if the axis was found — in which case `axis_info` has been
    /// filled in — false otherwise.
    ///
    /// Since HarfBuzz 2.2.0.
    pub fn hb_ot_var_find_axis_info(
        face: *mut hb_face_t,
        axis_tag: hb_tag_t,
        axis_info: *mut hb_ot_var_axis_info_t,
    ) -> hb_bool_t;

    /// Fetches the number of named instances included in the face.
    ///
    /// Since HarfBuzz 2.2.0.
    pub fn hb_ot_var_get_named_instance_count(face: *mut hb_face_t) -> c_uint;

    /// Fetches the `name` table Name ID providing display names for the
    /// "Subfamily name" of the given named instance.
    ///
    /// Since HarfBuzz 2.2.0.
    pub fn hb_ot_var_named_instance_get_subfamily_name_id(
        face: *mut hb_face_t,
        instance_index: c_uint,
    ) -> hb_ot_name_id_t;

    /// Fetches the `name` table Name ID providing display names for the
    /// "PostScript name" of the given named instance.
    ///
    /// Since HarfBuzz 2.2.0.
    pub fn hb_ot_var_named_instance_get_postscript_name_id(
        face: *mut hb_face_t,
        instance_index: c_uint,
    ) -> hb_ot_name_id_t;

    /// Fetches the design-space coordinates of the given named instance.
    ///
    /// On input `coords_length` holds the capacity of `coords`; on output it
    /// holds how many coordinates were actually written, which may be zero.
    /// Either pointer may be null, in which case nothing is written and only
    /// the count is returned.
    ///
    /// Returns the number of variation axes in the face.
    ///
    /// Since HarfBuzz 2.2.0.
    pub fn hb_ot_var_named_instance_get_design_coords(
        face: *mut hb_face_t,
        instance_index: c_uint,
        coords_length: *mut c_uint,
        coords: *mut c_float,
    ) -> c_uint;

    /// Normalizes all of the coordinates in the given list of variation axes.
    ///
    /// `coords` receives `coords_length` normalized values, one per axis of the
    /// face, in axis order. It is zeroed first, so any axis not mentioned in
    /// `variations` — and any variation naming an axis the face does not
    /// have — comes out at its default.
    ///
    /// Since HarfBuzz 1.4.2.
    pub fn hb_ot_var_normalize_variations(
        face: *mut hb_face_t,
        variations: *const hb_variation_t,
        variations_length: c_uint,
        coords: *mut c_int,
        coords_length: c_uint,
    );

    /// Normalizes the given design-space coordinates.
    ///
    /// The minimum and maximum values for each axis are mapped to the interval
    /// [-1, 1], with the default axis value mapped to 0. The results have 14
    /// bits of fixed-point sub-integer precision, as the OpenType specification
    /// requires.
    ///
    /// Any additional scaling defined in the face's
    /// [`avar` table](https://docs.microsoft.com/en-us/typography/opentype/spec/avar)
    /// is also applied.
    ///
    /// `coords_length` must equal the number of axes in the face, as returned
    /// by [`hb_ot_var_get_axis_count`]; otherwise the behaviour is undefined.
    ///
    /// Since HarfBuzz 1.4.2.
    pub fn hb_ot_var_normalize_coords(
        face: *mut hb_face_t,
        coords_length: c_uint,
        design_coords: *const c_float,
        normalized_coords: *mut c_int,
    );
}
