//! Shaping plans — the cached decision of how a text segment will be shaped —
//! `hb-shape-plan.h`.

use core::ffi::{c_char, c_int, c_uint, c_void};

use crate::{
    hb_bool_t, hb_buffer_t, hb_destroy_func_t, hb_face_t, hb_feature_t, hb_font_t,
    hb_segment_properties_t, hb_user_data_key_t, opaque_handle,
};

opaque_handle! {
    /// Data type for holding a shaping plan.
    ///
    /// Shape plans contain information about how HarfBuzz will shape a
    /// particular text segment, based on the segment's properties and the
    /// capabilities in the font face in use.
    ///
    /// Shape plans can be queried about how shaping will perform, given a set
    /// of specific input parameters (script, language, direction, features,
    /// and so on).
    hb_shape_plan_t
}

unsafe extern "C" {
    /// Constructs a shaping plan for a combination of `face`, `user_features`,
    /// `props`, and `shaper_list`.
    ///
    /// `shaper_list` is a NULL-terminated array of shaper names to try, in
    /// order; pass null to let HarfBuzz choose. The returned plan is owned by
    /// the caller and must be released with [`hb_shape_plan_destroy`].
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_shape_plan_create(
        face: *mut hb_face_t,
        props: *const hb_segment_properties_t,
        user_features: *const hb_feature_t,
        num_user_features: c_uint,
        shaper_list: *const *const c_char,
    ) -> *mut hb_shape_plan_t;

    /// Creates a cached shaping plan suitable for reuse, for a combination of
    /// `face`, `user_features`, `props`, and `shaper_list`.
    ///
    /// The plan is kept alive by the face, so an equivalent request later
    /// returns the same object with its reference count raised rather than
    /// building a new plan. The caller still owns the returned reference and
    /// must release it with [`hb_shape_plan_destroy`].
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_shape_plan_create_cached(
        face: *mut hb_face_t,
        props: *const hb_segment_properties_t,
        user_features: *const hb_feature_t,
        num_user_features: c_uint,
        shaper_list: *const *const c_char,
    ) -> *mut hb_shape_plan_t;

    /// The variable-font version of [`hb_shape_plan_create`].
    ///
    /// Constructs a shaping plan for a combination of `face`, `user_features`,
    /// `props`, and `shaper_list`, plus the variation-space coordinates
    /// `coords`. Coordinates are in HarfBuzz's normalized 2.14 fixed-point
    /// space, the same values [`hb_font_get_var_coords_normalized`] reports.
    ///
    /// Since HarfBuzz 1.4.0.
    ///
    /// [`hb_font_get_var_coords_normalized`]: crate::hb_font_get_var_coords_normalized
    pub fn hb_shape_plan_create2(
        face: *mut hb_face_t,
        props: *const hb_segment_properties_t,
        user_features: *const hb_feature_t,
        num_user_features: c_uint,
        coords: *const c_int,
        num_coords: c_uint,
        shaper_list: *const *const c_char,
    ) -> *mut hb_shape_plan_t;

    /// The variable-font version of [`hb_shape_plan_create_cached`].
    ///
    /// Creates a cached shaping plan suitable for reuse, for a combination of
    /// `face`, `user_features`, `props`, and `shaper_list`, plus the
    /// variation-space coordinates `coords`.
    ///
    /// Since HarfBuzz 1.4.0.
    pub fn hb_shape_plan_create_cached2(
        face: *mut hb_face_t,
        props: *const hb_segment_properties_t,
        user_features: *const hb_feature_t,
        num_user_features: c_uint,
        coords: *const c_int,
        num_coords: c_uint,
        shaper_list: *const *const c_char,
    ) -> *mut hb_shape_plan_t;

    /// Fetches the singleton empty shaping plan.
    ///
    /// This is the inert object the creation functions fall back to when a plan
    /// cannot be built, so it is never null. It may be passed to
    /// [`hb_shape_plan_destroy`] like any other plan.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_shape_plan_get_empty() -> *mut hb_shape_plan_t;

    /// Increases the reference count on the given shaping plan and returns it.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_shape_plan_reference(shape_plan: *mut hb_shape_plan_t) -> *mut hb_shape_plan_t;

    /// Decreases the reference count on the given shaping plan.
    ///
    /// When the reference count reaches zero, the shaping plan is destroyed,
    /// freeing all memory.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_shape_plan_destroy(shape_plan: *mut hb_shape_plan_t);

    /// Attaches a user-data key/data pair to the given shaping plan.
    ///
    /// `destroy` is called on `data` when the plan is destroyed or the entry is
    /// replaced, and may be null. `replace` decides whether an existing entry
    /// under the same key is overwritten. Returns true on success.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_shape_plan_set_user_data(
        shape_plan: *mut hb_shape_plan_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data associated with the specified key, attached to the
    /// specified shaping plan.
    ///
    /// The returned pointer is still owned by the plan.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_shape_plan_get_user_data(
        shape_plan: *const hb_shape_plan_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Executes the given shaping plan on the specified buffer, using the given
    /// `font` and `features`.
    ///
    /// The font's face and the buffer's segment properties must match the ones
    /// the plan was built for. Returns true on success, in which case a buffer
    /// holding Unicode content is switched to holding glyphs.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_shape_plan_execute(
        shape_plan: *mut hb_shape_plan_t,
        font: *mut hb_font_t,
        buffer: *mut hb_buffer_t,
        features: *const hb_feature_t,
        num_features: c_uint,
    ) -> hb_bool_t;

    /// Fetches the name of the shaper the given plan selected, as a
    /// NUL-terminated string owned by HarfBuzz — `"ot"`, `"coretext"`,
    /// `"graphite2"`, and so on.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_shape_plan_get_shaper(shape_plan: *mut hb_shape_plan_t) -> *const c_char;
}
