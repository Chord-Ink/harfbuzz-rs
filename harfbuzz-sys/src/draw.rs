//! Callbacks for extracting glyph outlines as paths — `hb-draw.h`.
//!
//! A draw-functions object is a "pen": HarfBuzz walks a glyph's contours and
//! calls into it with move-to, line-to, curve-to, and close-path operations.

use core::ffi::{c_float, c_int, c_void};

use crate::{hb_bool_t, hb_destroy_func_t, hb_user_data_key_t, hb_var_num_t};

/// The current drawing state, threaded through every draw callback.
///
/// HarfBuzz owns this structure and updates it as it walks a glyph: the caller
/// reads it to learn where the pen currently is, but never writes to it. The
/// trailing `reserved*` fields are private padding that keeps the struct's size
/// stable across releases; do not read or write them.
///
/// Since HarfBuzz 4.0.0.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_draw_state_t {
    /// Whether there is an open path.
    pub path_open: hb_bool_t,

    /// X component of the start of the current path.
    pub path_start_x: c_float,
    /// Y component of the start of the current path.
    pub path_start_y: c_float,

    /// X component of the current point.
    pub current_x: c_float,
    /// Y component of the current point.
    pub current_y: c_float,

    /// Private padding. Not part of the public contract.
    pub reserved1: hb_var_num_t,
    /// Private padding. Not part of the public contract.
    pub reserved2: hb_var_num_t,
    /// Private padding. Not part of the public contract.
    pub reserved3: hb_var_num_t,
    /// Private padding. Not part of the public contract.
    pub reserved4: hb_var_num_t,
    /// Private padding. Not part of the public contract.
    pub reserved5: hb_var_num_t,
    /// Private padding. Not part of the public contract.
    pub reserved6: hb_var_num_t,
    /// Private padding. Not part of the public contract.
    pub reserved7: hb_var_num_t,
}

// `hb_var_num_t` is a union and so has no `Debug`, which rules out a derive.
// The private padding carries no meaning anyway, so only the public fields are
// printed.
impl core::fmt::Debug for hb_draw_state_t {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("hb_draw_state_t")
            .field("path_open", &self.path_open)
            .field("path_start_x", &self.path_start_x)
            .field("path_start_y", &self.path_start_y)
            .field("current_x", &self.current_x)
            .field("current_y", &self.current_y)
            .finish_non_exhaustive()
    }
}

/// The default [`hb_draw_state_t`] at the start of glyph drawing.
///
/// The Rust equivalent of the C `HB_DRAW_STATE_DEFAULT` macro: no open path,
/// pen at the origin, padding zeroed.
pub const HB_DRAW_STATE_DEFAULT: hb_draw_state_t = hb_draw_state_t {
    path_open: 0,
    path_start_x: 0.0,
    path_start_y: 0.0,
    current_x: 0.0,
    current_y: 0.0,
    reserved1: hb_var_num_t { f: 0.0 },
    reserved2: hb_var_num_t { f: 0.0 },
    reserved3: hb_var_num_t { f: 0.0 },
    reserved4: hb_var_num_t { f: 0.0 },
    reserved5: hb_var_num_t { f: 0.0 },
    reserved6: hb_var_num_t { f: 0.0 },
    reserved7: hb_var_num_t { f: 0.0 },
};

opaque_handle! {
    /// Glyph draw callbacks — a "pen" that HarfBuzz drives.
    ///
    /// [`hb_draw_move_to_func_t`], [`hb_draw_line_to_func_t`], and
    /// [`hb_draw_cubic_to_func_t`] need to be defined; if
    /// [`hb_draw_quadratic_to_func_t`] is left unset, HarfBuzz translates
    /// quadratic curves into cubic ones and reports them through the cubic
    /// callback instead.
    ///
    /// Since HarfBuzz 4.0.0.
    hb_draw_funcs_t
}

/// A virtual method for [`hb_draw_funcs_t`] to perform a "move-to" draw
/// operation.
///
/// `draw_data` is the data accompanying the draw functions in
/// `hb_font_draw_glyph()`, `st` is the current draw state, `to_x`/`to_y` are
/// the target point, and `user_data` is the pointer passed to
/// [`hb_draw_funcs_set_move_to_func`].
///
/// Since HarfBuzz 4.0.0.
pub type hb_draw_move_to_func_t = Option<
    unsafe extern "C" fn(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
        to_x: c_float,
        to_y: c_float,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_draw_funcs_t`] to perform a "line-to" draw
/// operation.
///
/// `draw_data` is the data accompanying the draw functions in
/// `hb_font_draw_glyph()`, `st` is the current draw state, `to_x`/`to_y` are
/// the target point, and `user_data` is the pointer passed to
/// [`hb_draw_funcs_set_line_to_func`].
///
/// Since HarfBuzz 4.0.0.
pub type hb_draw_line_to_func_t = Option<
    unsafe extern "C" fn(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
        to_x: c_float,
        to_y: c_float,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_draw_funcs_t`] to perform a "quadratic-to" draw
/// operation.
///
/// `draw_data` is the data accompanying the draw functions in
/// `hb_font_draw_glyph()`, `st` is the current draw state,
/// `control_x`/`control_y` are the control point, `to_x`/`to_y` are the target
/// point, and `user_data` is the pointer passed to
/// [`hb_draw_funcs_set_quadratic_to_func`].
///
/// Since HarfBuzz 4.0.0.
pub type hb_draw_quadratic_to_func_t = Option<
    unsafe extern "C" fn(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
        control_x: c_float,
        control_y: c_float,
        to_x: c_float,
        to_y: c_float,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_draw_funcs_t`] to perform a "cubic-to" draw
/// operation.
///
/// `draw_data` is the data accompanying the draw functions in
/// `hb_font_draw_glyph()`, `st` is the current draw state,
/// `control1_x`/`control1_y` and `control2_x`/`control2_y` are the two control
/// points, `to_x`/`to_y` are the target point, and `user_data` is the pointer
/// passed to [`hb_draw_funcs_set_cubic_to_func`].
///
/// Since HarfBuzz 4.0.0.
pub type hb_draw_cubic_to_func_t = Option<
    unsafe extern "C" fn(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
        control1_x: c_float,
        control1_y: c_float,
        control2_x: c_float,
        control2_y: c_float,
        to_x: c_float,
        to_y: c_float,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_draw_funcs_t`] to perform a "close-path" draw
/// operation.
///
/// `draw_data` is the data accompanying the draw functions in
/// `hb_font_draw_glyph()`, `st` is the current draw state, and `user_data` is
/// the pointer passed to [`hb_draw_funcs_set_close_path_func`].
///
/// Since HarfBuzz 4.0.0.
pub type hb_draw_close_path_func_t = Option<
    unsafe extern "C" fn(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
        user_data: *mut c_void,
    ),
>;

/// End-cap shape for [`hb_draw_line`].
///
/// The C enumeration has no sentinel and its largest enumerator is 1, so it
/// fits in an `int`.
///
/// Since HarfBuzz 14.2.0.
pub type hb_draw_line_cap_t = c_int;

/// No cap; the line ends exactly at its endpoint.
pub const HB_DRAW_LINE_CAP_BUTT: hb_draw_line_cap_t = 0;

/// Square cap; the line is extended past its endpoint by half the local stroke
/// width.
///
/// Useful for composing closed shapes from line segments — a rectangle made
/// from four lines, for instance.
pub const HB_DRAW_LINE_CAP_SQUARE: hb_draw_line_cap_t = 1;

unsafe extern "C" {
    /// Sets the move-to callback on a draw-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed. Passing a null `func`
    /// restores the default no-op callback, and calls `destroy` on the incoming
    /// `user_data` immediately.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_funcs_set_move_to_func(
        dfuncs: *mut hb_draw_funcs_t,
        func: hb_draw_move_to_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the line-to callback on a draw-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed. Passing a null `func`
    /// restores the default no-op callback, and calls `destroy` on the incoming
    /// `user_data` immediately.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_funcs_set_line_to_func(
        dfuncs: *mut hb_draw_funcs_t,
        func: hb_draw_line_to_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the quadratic-to callback on a draw-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed. Passing a null `func`
    /// restores the default behaviour, which converts each quadratic curve to a
    /// cubic one and reports it through the cubic-to callback.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_funcs_set_quadratic_to_func(
        dfuncs: *mut hb_draw_funcs_t,
        func: hb_draw_quadratic_to_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the cubic-to callback on a draw-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed. Passing a null `func`
    /// restores the default no-op callback, and calls `destroy` on the incoming
    /// `user_data` immediately.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_funcs_set_cubic_to_func(
        dfuncs: *mut hb_draw_funcs_t,
        func: hb_draw_cubic_to_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the close-path callback on a draw-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed. Passing a null `func`
    /// restores the default no-op callback, and calls `destroy` on the incoming
    /// `user_data` immediately.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_funcs_set_close_path_func(
        dfuncs: *mut hb_draw_funcs_t,
        func: hb_draw_close_path_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Creates a new draw-functions object with a reference count of one.
    ///
    /// Never returns null: if memory cannot be allocated, the singleton empty
    /// object from [`hb_draw_funcs_get_empty`] is returned instead. Release your
    /// reference with [`hb_draw_funcs_destroy`].
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_funcs_create() -> *mut hb_draw_funcs_t;

    /// Fetches the singleton empty draw-functions object.
    ///
    /// Every one of its callbacks is the default no-op, and it is permanently
    /// immutable.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_draw_funcs_get_empty() -> *mut hb_draw_funcs_t;

    /// Increases the reference count on a draw-functions object and returns it.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_funcs_reference(dfuncs: *mut hb_draw_funcs_t) -> *mut hb_draw_funcs_t;

    /// Decreases the reference count on a draw-functions object.
    ///
    /// When the count reaches zero the object and all associated resources are
    /// freed, which includes calling the `destroy` callback registered
    /// alongside each of its drawing callbacks.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_funcs_destroy(dfuncs: *mut hb_draw_funcs_t);

    /// Attaches a user-data key/data pair to a draw-functions object.
    ///
    /// `destroy` — which may be null — is called with `data` when the object is
    /// destroyed or the value is replaced. `replace` decides whether existing
    /// data stored under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_draw_funcs_set_user_data(
        dfuncs: *mut hb_draw_funcs_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to a draw-functions object under the
    /// specified key.
    ///
    /// Ownership stays with the object; the caller must not free the returned
    /// pointer.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_draw_funcs_get_user_data(
        dfuncs: *const hb_draw_funcs_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Makes a draw-functions object immutable.
    ///
    /// After this, the `hb_draw_funcs_set_*_func` setters silently do nothing —
    /// they still call the `destroy` callback on the `user_data` they were
    /// handed, so nothing leaks.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_funcs_make_immutable(dfuncs: *mut hb_draw_funcs_t);

    /// Tests whether a draw-functions object is immutable.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_funcs_is_immutable(dfuncs: *mut hb_draw_funcs_t) -> hb_bool_t;

    /// Performs a "move-to" draw operation, starting a new contour.
    ///
    /// Closes any path that is still open, then records `to_x`/`to_y` as the
    /// current point in `st`. The callback itself is not invoked until the first
    /// segment of the new contour is drawn.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_move_to(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
        to_x: c_float,
        to_y: c_float,
    );

    /// Performs a "line-to" draw operation.
    ///
    /// Opens a path if none is open, then draws a straight segment from the
    /// current point to `to_x`/`to_y` and makes that the new current point.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_line_to(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
        to_x: c_float,
        to_y: c_float,
    );

    /// Performs a "quadratic-to" draw operation.
    ///
    /// Opens a path if none is open, then draws a quadratic Bézier from the
    /// current point through `control_x`/`control_y` to `to_x`/`to_y`. If the
    /// object has no quadratic-to callback, the curve is elevated to a cubic and
    /// reported through the cubic-to callback instead.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_quadratic_to(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
        control_x: c_float,
        control_y: c_float,
        to_x: c_float,
        to_y: c_float,
    );

    /// Performs a "cubic-to" draw operation.
    ///
    /// Opens a path if none is open, then draws a cubic Bézier from the current
    /// point through the two control points to `to_x`/`to_y`.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_cubic_to(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
        control1_x: c_float,
        control1_y: c_float,
        control2_x: c_float,
        control2_y: c_float,
        to_x: c_float,
        to_y: c_float,
    );

    /// Performs a "close-path" draw operation.
    ///
    /// If a path is open, an implicit line back to the path's start point is
    /// emitted when the current point differs from it, then the close-path
    /// callback runs. The draw state is reset to
    /// [`HB_DRAW_STATE_DEFAULT`]'s pen position afterwards. Closing when no path
    /// is open does nothing.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_draw_close_path(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
    );

    /// Emits a tapered line segment as a filled trapezoid.
    ///
    /// `w0` and `w1` are the full stroke widths at the start and end points;
    /// they may differ for a tapered stroke or match for a uniform one. Pass NaN
    /// for `w1` to reuse `w0`. With [`HB_DRAW_LINE_CAP_SQUARE`] each endpoint is
    /// extended along the line direction by half its local stroke width, so four
    /// calls form a closed rectangle with no gaps at the corners.
    ///
    /// A zero-length segment draws nothing.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_draw_line(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
        x0: c_float,
        y0: c_float,
        w0: c_float,
        x1: c_float,
        y1: c_float,
        w1: c_float,
        cap: hb_draw_line_cap_t,
    );

    /// Emits an axis-aligned rectangle whose top-left corner is `x`/`y`.
    ///
    /// `w` and `h` may be negative. A finite positive `stroke_width` draws an
    /// outlined ring of that thickness centred on the edges; NaN draws a filled
    /// rectangle. Any other `stroke_width` — zero, negative, or infinite —
    /// draws nothing, as does a filled rectangle with zero width or height.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_draw_rectangle(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
        x: c_float,
        y: c_float,
        w: c_float,
        h: c_float,
        stroke_width: c_float,
    );

    /// Emits a circle centred on `cx`/`cy`, approximated by four cubic Béziers.
    ///
    /// A finite positive `stroke_width` draws an outlined ring of that thickness
    /// centred on the nominal radius; NaN draws a filled disc. Any other
    /// `stroke_width` — zero, negative, or infinite — draws nothing, as does a
    /// radius that is zero or negative.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_draw_circle(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
        cx: c_float,
        cy: c_float,
        r: c_float,
        stroke_width: c_float,
    );
}
