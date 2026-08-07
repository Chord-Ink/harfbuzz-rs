//! Callbacks for painting colour glyphs — `hb-paint.h`.
//!
//! A paint-functions object is a rendering back end: HarfBuzz walks a colour
//! glyph's paint tree and drives it with transform, clip, fill, gradient, and
//! compositing operations.

use core::ffi::{c_float, c_int, c_uint, c_void};

use crate::{
    HB_TAG, hb_blob_t, hb_bool_t, hb_codepoint_t, hb_color_t, hb_destroy_func_t, hb_draw_funcs_t,
    hb_font_t, hb_glyph_extents_t, hb_tag_t, hb_user_data_key_t,
};

opaque_handle! {
    /// Glyph paint callbacks — the rendering back end HarfBuzz drives.
    ///
    /// The callbacks assume that the caller maintains a stack of current
    /// transforms, clips, and intermediate surfaces, as evidenced by the pairs
    /// of push/pop callbacks. The push/pop calls are always properly nested, so
    /// it is fine to store the different kinds of object on a single stack.
    ///
    /// Not all callbacks are required for all kinds of glyphs. For rendering
    /// COLRv0 or non-colour outline glyphs, the gradient callbacks are not
    /// needed, and the composite callback only needs to handle simple alpha
    /// compositing ([`HB_PAINT_COMPOSITE_MODE_SRC_OVER`]).
    ///
    /// The paint-image callback is only needed for glyphs with image blobs in
    /// the `CBDT`, `sbix`, or `SVG` tables.
    ///
    /// The custom-palette-colour callback is only necessary if you want to
    /// override colours from the font palette with custom colours.
    ///
    /// Since HarfBuzz 7.0.0.
    hb_paint_funcs_t
}

/// A virtual method for [`hb_paint_funcs_t`] to apply a transform to subsequent
/// paint calls.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `xx`/`yx`/`xy`/`yy`/`dx`/`dy` are the components of
/// the transform matrix, and `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_push_transform_func`].
///
/// This transform is applied after the current transform, and remains in effect
/// until a matching [`hb_paint_pop_transform_func_t`] call.
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_push_transform_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        xx: c_float,
        yx: c_float,
        xy: c_float,
        yy: c_float,
        dx: c_float,
        dy: c_float,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_paint_funcs_t`] to undo the effect of a prior
/// [`hb_paint_push_transform_func_t`] call.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, and `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_pop_transform_func`].
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_pop_transform_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_paint_funcs_t`] to render a colour glyph by glyph
/// index.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `glyph` is the glyph ID, `font` is the font, and
/// `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_color_glyph_func`].
///
/// Returns true if the glyph was painted, false otherwise.
///
/// Since HarfBuzz 8.2.0.
pub type hb_paint_color_glyph_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        glyph: hb_codepoint_t,
        font: *mut hb_font_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// A virtual method for [`hb_paint_funcs_t`] to fill a glyph's shape with a
/// solid colour.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `glyph` is the glyph ID, `font` is the font,
/// `is_foreground` says whether the colour is the foreground, `color` is the
/// unpremultiplied colour to use, and `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_fill_glyph_func`].
///
/// If not implemented, a sequence of push-clip-glyph, colour, and pop-clip
/// paint operations is emitted instead, in that order.
///
/// Since HarfBuzz 14.3.0.
pub type hb_paint_fill_glyph_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        glyph: hb_codepoint_t,
        font: *mut hb_font_t,
        is_foreground: hb_bool_t,
        color: hb_color_t,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_paint_funcs_t`] to clip subsequent paint calls to
/// the outline of a glyph.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `glyph` is the glyph ID, `font` is the font, and
/// `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_push_clip_glyph_func`].
///
/// The coordinates of the glyph outline are expected in the current `font`
/// scale — that is, the results of calling `hb_font_draw_glyph()` with `font`.
/// The outline is transformed by the current transform.
///
/// This clip is applied in addition to the current clip, and remains in effect
/// until a matching [`hb_paint_pop_clip_func_t`] call.
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_push_clip_glyph_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        glyph: hb_codepoint_t,
        font: *mut hb_font_t,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_paint_funcs_t`] to clip subsequent paint calls to
/// a rectangle.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `xmin`/`ymin`/`xmax`/`ymax` bound the rectangle, and
/// `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_push_clip_rectangle_func`].
///
/// The coordinates of the rectangle are interpreted according to the current
/// transform.
///
/// This clip is applied in addition to the current clip, and remains in effect
/// until a matching [`hb_paint_pop_clip_func_t`] call.
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_push_clip_rectangle_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        xmin: c_float,
        ymin: c_float,
        xmax: c_float,
        ymax: c_float,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_paint_funcs_t`] to begin clipping to an arbitrary
/// path.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `draw_data` is an out-parameter for the draw data
/// the caller should pass alongside the returned draw funcs, and `user_data` is
/// the pointer passed to
/// [`hb_paint_funcs_set_push_clip_path_start_func`].
///
/// The back end returns an [`hb_draw_funcs_t`] it owns — the caller must not
/// free it — that the caller feeds the clip outline to via the `hb_draw_*()`
/// calls, plus a `draw_data` value to pass alongside those calls. Both are only
/// valid until the matching [`hb_paint_push_clip_path_end_func_t`] call; no
/// other paint calls should be made in between. The clip remains in effect
/// until a later [`hb_paint_pop_clip_func_t`] call.
///
/// Returns draw funcs that accumulate the clip path, ownership of which is not
/// transferred, or null if arbitrary-path clipping is not supported.
///
/// Since HarfBuzz 14.2.0.
pub type hb_paint_push_clip_path_start_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        draw_data: *mut *mut c_void,
        user_data: *mut c_void,
    ) -> *mut hb_draw_funcs_t,
>;

/// A virtual method for [`hb_paint_funcs_t`] to close the clip path started by
/// the [`hb_paint_push_clip_path_start_func_t`] method.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, and `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_push_clip_path_end_func`].
///
/// The emitted path is now active as a clip; subsequent paint operations are
/// masked by it until a matching [`hb_paint_pop_clip_func_t`] call.
///
/// Since HarfBuzz 14.2.0.
pub type hb_paint_push_clip_path_end_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_paint_funcs_t`] to undo the effect of a prior
/// [`hb_paint_push_clip_glyph_func_t`],
/// [`hb_paint_push_clip_rectangle_func_t`], or
/// [`hb_paint_push_clip_path_end_func_t`] call.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, and `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_pop_clip_func`].
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_pop_clip_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_paint_funcs_t`] to paint a colour everywhere
/// within the current clip.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `is_foreground` says whether the colour is the
/// foreground, `color` is the unpremultiplied colour to use, and `user_data` is
/// the pointer passed to [`hb_paint_funcs_set_color_func`].
///
/// When `is_foreground` is true, this colour originates from the
/// foreground-colour sentinel in the font's colour data. The `color` parameter
/// still carries a fully resolved RGBA value, with any paint-tree alpha already
/// applied, so back ends that do not need to distinguish the foreground can
/// simply use `color` directly.
///
/// Back ends that defer foreground resolution — to honour a CSS `currentColor`
/// or a runtime uniform, say — should substitute their own foreground RGB when
/// `is_foreground` is true, but must combine the alpha from `color` with their
/// foreground alpha, since it encodes additional modulation from the paint
/// tree. For this mode to work correctly, the caller should pass a fully opaque
/// foreground colour to `hb_font_paint_glyph()`, so that the alpha in `color`
/// reflects only the paint-tree contribution.
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_color_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        is_foreground: hb_bool_t,
        color: hb_color_t,
        user_data: *mut c_void,
    ),
>;

/// Tag identifying PNG images in [`hb_paint_image_func_t`] callbacks.
///
/// Since HarfBuzz 7.0.0.
pub const HB_PAINT_IMAGE_FORMAT_PNG: hb_tag_t = HB_TAG(b'p', b'n', b'g', b' ');

/// Tag identifying SVG images in [`hb_paint_image_func_t`] callbacks.
///
/// Since HarfBuzz 7.0.0.
pub const HB_PAINT_IMAGE_FORMAT_SVG: hb_tag_t = HB_TAG(b's', b'v', b'g', b' ');

/// Tag identifying raw pixel-data images in [`hb_paint_image_func_t`]
/// callbacks.
///
/// The data is in BGRA pre-multiplied sRGBA colour-space format.
///
/// Since HarfBuzz 7.0.0.
pub const HB_PAINT_IMAGE_FORMAT_BGRA: hb_tag_t = HB_TAG(b'B', b'G', b'R', b'A');

/// A virtual method for [`hb_paint_funcs_t`] to paint a glyph image.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `image` is the image data, `width` and `height` are
/// the raster image's dimensions in pixels (or 0), `format` is the image format
/// as a tag, `slant` is deprecated and always 0.0, `extents` holds the glyph
/// extents for the desired rendering and may be null, and `user_data` is the
/// pointer passed to [`hb_paint_funcs_set_image_func`].
///
/// This method is called for glyphs with image blobs in the `CBDT`, `sbix`, or
/// `SVG` tables. The `format` identifies the kind of data contained in `image`;
/// possible values include [`HB_PAINT_IMAGE_FORMAT_PNG`],
/// [`HB_PAINT_IMAGE_FORMAT_SVG`], and [`HB_PAINT_IMAGE_FORMAT_BGRA`].
///
/// The image dimensions and glyph extents are provided if available, and should
/// be used to size and position the image.
///
/// Returns whether the operation was successful.
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_image_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        image: *mut hb_blob_t,
        width: c_uint,
        height: c_uint,
        format: hb_tag_t,
        slant: c_float,
        extents: *mut hb_glyph_extents_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// Information about a colour stop on a colour line.
///
/// Colour lines typically have offsets ranging between 0 and 1, but that is not
/// required.
///
/// The `is_foreground` and `color` fields have the same semantics as in
/// [`hb_paint_color_func_t`].
///
/// Note that although `color` is unpremultiplied here, interpolation in
/// gradients happens in premultiplied space. See the
/// [COLR](https://learn.microsoft.com/en-us/typography/opentype/spec/colr)
/// section of the OpenType spec for details.
///
/// Since HarfBuzz 7.0.0.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct hb_color_stop_t {
    /// The offset of the colour stop.
    pub offset: c_float,
    /// Whether the colour is the foreground.
    pub is_foreground: hb_bool_t,
    /// The colour, unpremultiplied.
    pub color: hb_color_t,
}

/// How colour values outside a colour line's defined offset range are
/// determined.
///
/// See the
/// [COLR](https://learn.microsoft.com/en-us/typography/opentype/spec/colr)
/// section of the OpenType spec for details.
///
/// The C enumeration has no sentinel and its largest enumerator is 2, so it
/// fits in an `int`.
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_extend_t = c_int;

/// Outside the defined interval, the colour of the closest colour stop is used.
pub const HB_PAINT_EXTEND_PAD: hb_paint_extend_t = 0;
/// The colour line is repeated over repeated multiples of the defined interval.
pub const HB_PAINT_EXTEND_REPEAT: hb_paint_extend_t = 1;
/// The colour line is repeated over repeated intervals, as for the repeat mode.
/// However, in each repeated interval the ordering of colour stops is the
/// reverse of the adjacent interval.
pub const HB_PAINT_EXTEND_REFLECT: hb_paint_extend_t = 2;

/// A virtual method for [`hb_color_line_t`] to fetch colour stops.
///
/// `color_line_data` is the data accompanying `color_line`, `start` is the index
/// of the first colour stop to return, `count` is both the maximum number of
/// stops to return and, on output, the number actually returned (possibly
/// zero), `color_stops` is the array to populate, and `user_data` is the data
/// accompanying this method. Both `count` and `color_stops` may be null.
///
/// Returns the total number of colour stops in `color_line`.
///
/// Since HarfBuzz 7.0.0.
pub type hb_color_line_get_color_stops_func_t = Option<
    unsafe extern "C" fn(
        color_line: *mut hb_color_line_t,
        color_line_data: *mut c_void,
        start: c_uint,
        count: *mut c_uint,
        color_stops: *mut hb_color_stop_t,
        user_data: *mut c_void,
    ) -> c_uint,
>;

/// A virtual method for [`hb_color_line_t`] to fetch the extend mode.
///
/// `color_line_data` is the data accompanying `color_line`, and `user_data` is
/// the data accompanying this method.
///
/// Returns the extend mode of `color_line`.
///
/// Since HarfBuzz 7.0.0.
pub type hb_color_line_get_extend_func_t = Option<
    unsafe extern "C" fn(
        color_line: *mut hb_color_line_t,
        color_line_data: *mut c_void,
        user_data: *mut c_void,
    ) -> hb_paint_extend_t,
>;

/// Colour information for a gradient.
///
/// HarfBuzz constructs this structure on the stack and hands it to the gradient
/// callbacks; it is only valid for the duration of the callback. Rather than
/// reading the fields directly, call [`hb_color_line_get_color_stops`] and
/// [`hb_color_line_get_extend`], which dispatch through the function pointers
/// below.
///
/// The trailing `reserved*` fields are private padding that keeps the struct's
/// size stable across releases; do not read or write them. Note that upstream
/// numbers them 0–3 and then 5–8, skipping 4; that gap is reproduced here so
/// the field names match the C header exactly.
///
/// Since HarfBuzz 7.0.0.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct hb_color_line_t {
    /// The data accompanying this colour line, passed to both methods below.
    pub data: *mut c_void,

    /// The method that fetches colour stops.
    pub get_color_stops: hb_color_line_get_color_stops_func_t,
    /// The user data passed to `get_color_stops`.
    pub get_color_stops_user_data: *mut c_void,

    /// The method that fetches the extend mode.
    pub get_extend: hb_color_line_get_extend_func_t,
    /// The user data passed to `get_extend`.
    pub get_extend_user_data: *mut c_void,

    /// Private padding. Not part of the public contract.
    pub reserved0: *mut c_void,
    /// Private padding. Not part of the public contract.
    pub reserved1: *mut c_void,
    /// Private padding. Not part of the public contract.
    pub reserved2: *mut c_void,
    /// Private padding. Not part of the public contract.
    pub reserved3: *mut c_void,
    /// Private padding. Not part of the public contract.
    pub reserved5: *mut c_void,
    /// Private padding. Not part of the public contract.
    pub reserved6: *mut c_void,
    /// Private padding. Not part of the public contract.
    pub reserved7: *mut c_void,
    /// Private padding. Not part of the public contract.
    pub reserved8: *mut c_void,
}

/// A virtual method for [`hb_paint_funcs_t`] to paint a linear gradient
/// everywhere within the current clip.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `color_line` carries the gradient's colour
/// information, `x0`/`y0`, `x1`/`y1`, and `x2`/`y2` are the three defining
/// points, and `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_linear_gradient_func`].
///
/// The `color_line` object is only valid for the duration of the callback; you
/// cannot keep it around.
///
/// The coordinates of the points are interpreted according to the current
/// transform. See the
/// [COLR](https://learn.microsoft.com/en-us/typography/opentype/spec/colr)
/// section of the OpenType spec for details on how the points define the
/// direction of the gradient, and how to interpret the colour line.
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_linear_gradient_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        color_line: *mut hb_color_line_t,
        x0: c_float,
        y0: c_float,
        x1: c_float,
        y1: c_float,
        x2: c_float,
        y2: c_float,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_paint_funcs_t`] to paint a radial gradient
/// everywhere within the current clip.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `color_line` carries the gradient's colour
/// information, `x0`/`y0`/`r0` and `x1`/`y1`/`r1` are the centres and radii of
/// the two defining circles, and `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_radial_gradient_func`].
///
/// The `color_line` object is only valid for the duration of the callback; you
/// cannot keep it around.
///
/// The coordinates of the points are interpreted according to the current
/// transform. See the
/// [COLR](https://learn.microsoft.com/en-us/typography/opentype/spec/colr)
/// section of the OpenType spec for details on how the points define the
/// direction of the gradient, and how to interpret the colour line.
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_radial_gradient_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        color_line: *mut hb_color_line_t,
        x0: c_float,
        y0: c_float,
        r0: c_float,
        x1: c_float,
        y1: c_float,
        r1: c_float,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_paint_funcs_t`] to paint a sweep gradient
/// everywhere within the current clip.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `color_line` carries the gradient's colour
/// information, `x0`/`y0` is the circle's centre, `start_angle` and `end_angle`
/// bound the sweep in radians, and `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_sweep_gradient_func`].
///
/// The `color_line` object is only valid for the duration of the callback; you
/// cannot keep it around.
///
/// The coordinates of the points are interpreted according to the current
/// transform. See the
/// [COLR](https://learn.microsoft.com/en-us/typography/opentype/spec/colr)
/// section of the OpenType spec for details on how the points define the
/// direction of the gradient, and how to interpret the colour line.
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_sweep_gradient_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        color_line: *mut hb_color_line_t,
        x0: c_float,
        y0: c_float,
        start_angle: c_float,
        end_angle: c_float,
        user_data: *mut c_void,
    ),
>;

/// The compositing modes that can be used when combining temporary redirected
/// drawing with the backdrop.
///
/// See the
/// [COLR](https://learn.microsoft.com/en-us/typography/opentype/spec/colr)
/// section of the OpenType spec for details.
///
/// The C enumeration has no sentinel and its largest enumerator is 27, so it
/// fits in an `int`.
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_composite_mode_t = c_int;

/// Clear the destination layer (bounded).
pub const HB_PAINT_COMPOSITE_MODE_CLEAR: hb_paint_composite_mode_t = 0;
/// Replace the destination layer (bounded).
pub const HB_PAINT_COMPOSITE_MODE_SRC: hb_paint_composite_mode_t = 1;
/// Ignore the source.
pub const HB_PAINT_COMPOSITE_MODE_DEST: hb_paint_composite_mode_t = 2;
/// Draw the source layer on top of the destination layer (bounded).
pub const HB_PAINT_COMPOSITE_MODE_SRC_OVER: hb_paint_composite_mode_t = 3;
/// Draw the destination on top of the source.
pub const HB_PAINT_COMPOSITE_MODE_DEST_OVER: hb_paint_composite_mode_t = 4;
/// Draw the source where there was destination content (unbounded).
pub const HB_PAINT_COMPOSITE_MODE_SRC_IN: hb_paint_composite_mode_t = 5;
/// Leave the destination only where there was source content (unbounded).
pub const HB_PAINT_COMPOSITE_MODE_DEST_IN: hb_paint_composite_mode_t = 6;
/// Draw the source where there was no destination content (unbounded).
pub const HB_PAINT_COMPOSITE_MODE_SRC_OUT: hb_paint_composite_mode_t = 7;
/// Leave the destination only where there was no source content.
pub const HB_PAINT_COMPOSITE_MODE_DEST_OUT: hb_paint_composite_mode_t = 8;
/// Draw the source on top of the destination content, and only there.
pub const HB_PAINT_COMPOSITE_MODE_SRC_ATOP: hb_paint_composite_mode_t = 9;
/// Leave the destination on top of the source content, and only there
/// (unbounded).
pub const HB_PAINT_COMPOSITE_MODE_DEST_ATOP: hb_paint_composite_mode_t = 10;
/// Show the source and destination where there is only one of them.
pub const HB_PAINT_COMPOSITE_MODE_XOR: hb_paint_composite_mode_t = 11;
/// Accumulate the source and destination layers.
pub const HB_PAINT_COMPOSITE_MODE_PLUS: hb_paint_composite_mode_t = 12;
/// Complement and multiply the source and destination. This causes the result
/// to be at least as light as the lighter inputs.
pub const HB_PAINT_COMPOSITE_MODE_SCREEN: hb_paint_composite_mode_t = 13;
/// Multiply or screen, depending on the lightness of the destination colour.
pub const HB_PAINT_COMPOSITE_MODE_OVERLAY: hb_paint_composite_mode_t = 14;
/// Replace the destination with the source if it is darker, otherwise keep the
/// source.
pub const HB_PAINT_COMPOSITE_MODE_DARKEN: hb_paint_composite_mode_t = 15;
/// Replace the destination with the source if it is lighter, otherwise keep the
/// source.
pub const HB_PAINT_COMPOSITE_MODE_LIGHTEN: hb_paint_composite_mode_t = 16;
/// Brighten the destination colour to reflect the source colour.
pub const HB_PAINT_COMPOSITE_MODE_COLOR_DODGE: hb_paint_composite_mode_t = 17;
/// Darken the destination colour to reflect the source colour.
pub const HB_PAINT_COMPOSITE_MODE_COLOR_BURN: hb_paint_composite_mode_t = 18;
/// Multiply or screen, dependent on the source colour.
pub const HB_PAINT_COMPOSITE_MODE_HARD_LIGHT: hb_paint_composite_mode_t = 19;
/// Darken or lighten, dependent on the source colour.
pub const HB_PAINT_COMPOSITE_MODE_SOFT_LIGHT: hb_paint_composite_mode_t = 20;
/// Take the difference of the source and destination colours.
pub const HB_PAINT_COMPOSITE_MODE_DIFFERENCE: hb_paint_composite_mode_t = 21;
/// Produce an effect similar to difference, but with lower contrast.
pub const HB_PAINT_COMPOSITE_MODE_EXCLUSION: hb_paint_composite_mode_t = 22;
/// Multiply the source and destination layers. This causes the result to be at
/// least as dark as the darker inputs.
pub const HB_PAINT_COMPOSITE_MODE_MULTIPLY: hb_paint_composite_mode_t = 23;
/// Create a colour with the hue of the source and the saturation and luminosity
/// of the target.
pub const HB_PAINT_COMPOSITE_MODE_HSL_HUE: hb_paint_composite_mode_t = 24;
/// Create a colour with the saturation of the source and the hue and luminosity
/// of the target. Painting with this mode onto a grey area produces no change.
pub const HB_PAINT_COMPOSITE_MODE_HSL_SATURATION: hb_paint_composite_mode_t = 25;
/// Create a colour with the hue and saturation of the source and the luminosity
/// of the target. This preserves the grey levels of the target and is useful
/// for colouring monochrome images or tinting colour images.
pub const HB_PAINT_COMPOSITE_MODE_HSL_COLOR: hb_paint_composite_mode_t = 26;
/// Create a colour with the luminosity of the source and the hue and saturation
/// of the target. This produces an inverse effect to
/// [`HB_PAINT_COMPOSITE_MODE_HSL_COLOR`].
pub const HB_PAINT_COMPOSITE_MODE_HSL_LUMINOSITY: hb_paint_composite_mode_t = 27;

/// A virtual method for [`hb_paint_funcs_t`] to use an intermediate surface for
/// subsequent paint calls.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, and `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_push_group_func`].
///
/// The drawing is redirected to an intermediate surface until a matching
/// [`hb_paint_pop_group_func_t`] call.
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_push_group_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_paint_funcs_t`] to use an intermediate surface for
/// subsequent paint calls, with the compositing mode known in advance.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `mode` is the compositing mode that will be used
/// when the group is popped, and `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_push_group_for_func`].
///
/// This is like [`hb_paint_push_group_func_t`], but the compositing mode is
/// provided at push time. By default it calls [`hb_paint_push_group_func_t`].
///
/// Since HarfBuzz 14.2.0.
pub type hb_paint_push_group_for_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        mode: hb_paint_composite_mode_t,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_paint_funcs_t`] to undo the effect of a prior
/// [`hb_paint_push_group_func_t`] call.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `mode` is the compositing mode to use, and
/// `user_data` is the pointer passed to
/// [`hb_paint_funcs_set_pop_group_func`].
///
/// This call stops the redirection to the intermediate surface, and then
/// composites it onto the previous surface using the compositing mode passed to
/// this call.
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_pop_group_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        mode: hb_paint_composite_mode_t,
        user_data: *mut c_void,
    ),
>;

/// A virtual method for [`hb_paint_funcs_t`] to fetch a custom palette override
/// colour for `color_index`.
///
/// `paint_data` is the data accompanying the paint functions in
/// `hb_font_paint_glyph()`, `color_index` is the colour index to fetch, `color`
/// is an out-parameter for the fetched colour, and `user_data` is the pointer
/// passed to [`hb_paint_funcs_set_custom_palette_color_func`].
///
/// Custom palette colours override colours from the font's selected colour
/// palette. It is not necessary to override all palette entries; return false
/// for entries that should be taken from the font palette.
///
/// This function might be called multiple times, but the custom palette is
/// expected to remain unchanged for the duration of one
/// `hb_font_paint_glyph()` call.
///
/// Returns true if a custom colour is provided, false otherwise.
///
/// Since HarfBuzz 7.0.0.
pub type hb_paint_custom_palette_color_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        color_index: c_uint,
        color: *mut hb_color_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// A callback invoked once per `(a0, a1)` sector of a sweep-gradient tiling.
///
/// `a0` and `a1` are the segment's start and end angles in radians, `c0` and
/// `c1` are the corresponding colours, and `user_data` is the pointer passed to
/// [`hb_paint_sweep_gradient_tiles`].
///
/// Since HarfBuzz 14.2.0.
pub type hb_paint_sweep_gradient_tile_func_t = Option<
    unsafe extern "C" fn(
        a0: c_float,
        c0: hb_color_t,
        a1: c_float,
        c1: hb_color_t,
        user_data: *mut c_void,
    ),
>;

unsafe extern "C" {
    /// Creates a new paint-functions object with a reference count of one.
    ///
    /// Every callback starts out unset, which means "do nothing"; install the
    /// ones your back end implements with the `hb_paint_funcs_set_*_func`
    /// setters. Release your reference with [`hb_paint_funcs_destroy`].
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_create() -> *mut hb_paint_funcs_t;

    /// Fetches the singleton empty paint-functions object.
    ///
    /// Every one of its callbacks is the default no-op, and it is permanently
    /// immutable.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_get_empty() -> *mut hb_paint_funcs_t;

    /// Increases the reference count on a paint-functions object and returns it.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_reference(funcs: *mut hb_paint_funcs_t) -> *mut hb_paint_funcs_t;

    /// Decreases the reference count on a paint-functions object.
    ///
    /// When the count reaches zero the object and all associated resources are
    /// freed, which includes calling the `destroy` callback registered alongside
    /// each of its painting callbacks.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_destroy(funcs: *mut hb_paint_funcs_t);

    /// Attaches a user-data key/data pair to a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `data` when the object is
    /// destroyed or the value is replaced. `replace` decides whether existing
    /// data stored under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_user_data(
        funcs: *mut hb_paint_funcs_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to a paint-functions object under the
    /// specified key.
    ///
    /// Ownership stays with the object; the caller must not free the returned
    /// pointer.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_get_user_data(
        funcs: *const hb_paint_funcs_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Makes a paint-functions object immutable.
    ///
    /// After this, the `hb_paint_funcs_set_*_func` setters silently do nothing —
    /// they still call the `destroy` callback on the `user_data` they were
    /// handed, so nothing leaks.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_make_immutable(funcs: *mut hb_paint_funcs_t);

    /// Tests whether a paint-functions object is immutable.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_is_immutable(funcs: *mut hb_paint_funcs_t) -> hb_bool_t;

    /// Fetches a span of colour stops from a colour line.
    ///
    /// On input `count` holds the capacity of `color_stops`; on output it holds
    /// how many entries were written, starting from index `start`. Pass a null
    /// `count` (or a null `color_stops`) to query only the total.
    ///
    /// Returns the total number of colour stops in `color_line`.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_color_line_get_color_stops(
        color_line: *mut hb_color_line_t,
        start: c_uint,
        count: *mut c_uint,
        color_stops: *mut hb_color_stop_t,
    ) -> c_uint;

    /// Fetches the extend mode of a colour line.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_color_line_get_extend(color_line: *mut hb_color_line_t) -> hb_paint_extend_t;

    /// Sets the push-transform callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_push_transform_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_push_transform_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the pop-transform callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_pop_transform_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_pop_transform_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the colour-glyph callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 8.2.0.
    pub fn hb_paint_funcs_set_color_glyph_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_color_glyph_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the fill-glyph callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 14.3.0.
    pub fn hb_paint_funcs_set_fill_glyph_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_fill_glyph_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the push-clip-glyph callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_push_clip_glyph_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_push_clip_glyph_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the push-clip-rectangle callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_push_clip_rectangle_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_push_clip_rectangle_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the push-clip-path-start callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_paint_funcs_set_push_clip_path_start_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_push_clip_path_start_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the push-clip-path-end callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_paint_funcs_set_push_clip_path_end_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_push_clip_path_end_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the pop-clip callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_pop_clip_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_pop_clip_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the paint-colour callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_color_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_color_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the paint-image callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_image_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_image_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the linear-gradient callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_linear_gradient_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_linear_gradient_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the radial-gradient callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_radial_gradient_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_radial_gradient_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the sweep-gradient callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_sweep_gradient_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_sweep_gradient_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the push-group callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_push_group_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_push_group_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the push-group-for callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_paint_funcs_set_push_group_for_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_push_group_for_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the pop-group callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_pop_group_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_pop_group_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the custom-palette-colour callback on a paint-functions object.
    ///
    /// `destroy` — which may be null — is called with `user_data` when the
    /// callback is replaced or the object is destroyed.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_funcs_set_custom_palette_color_func(
        funcs: *mut hb_paint_funcs_t,
        func: hb_paint_custom_palette_color_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Invokes the push-transform callback directly.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_push_transform(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        xx: c_float,
        yx: c_float,
        xy: c_float,
        yy: c_float,
        dx: c_float,
        dy: c_float,
    );

    /// Pushes the transform that maps font-unit coordinates into `font`'s scaled
    /// space.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_push_font_transform(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        font: *const hb_font_t,
    );

    /// Pushes the inverse of the transform pushed by
    /// [`hb_paint_push_font_transform`], mapping `font`'s scaled space back to
    /// font units.
    ///
    /// Since HarfBuzz 8.2.0.
    pub fn hb_paint_push_inverse_font_transform(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        font: *const hb_font_t,
    );

    /// Invokes the pop-transform callback directly.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_pop_transform(funcs: *mut hb_paint_funcs_t, paint_data: *mut c_void);

    /// Invokes the colour-glyph callback directly.
    ///
    /// Returns true if the glyph was painted, false otherwise.
    ///
    /// Since HarfBuzz 8.2.0.
    pub fn hb_paint_color_glyph(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        glyph: hb_codepoint_t,
        font: *mut hb_font_t,
    ) -> hb_bool_t;

    /// Invokes the fill-glyph callback directly.
    ///
    /// Since HarfBuzz 14.3.0.
    pub fn hb_paint_fill_glyph(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        glyph: hb_codepoint_t,
        font: *mut hb_font_t,
        is_foreground: hb_bool_t,
        color: hb_color_t,
    );

    /// Invokes the push-clip-glyph callback directly.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_push_clip_glyph(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        glyph: hb_codepoint_t,
        font: *mut hb_font_t,
    );

    /// Invokes the push-clip-rectangle callback directly.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_push_clip_rectangle(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        xmin: c_float,
        ymin: c_float,
        xmax: c_float,
        ymax: c_float,
    );

    /// Invokes the push-clip-path-start callback directly.
    ///
    /// Returns the draw funcs the back end wants the clip outline fed to, along
    /// with a `draw_data` value written through the out-parameter. The back end
    /// retains ownership of the draw funcs; do not destroy them. Returns null
    /// when arbitrary-path clipping is not supported.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_paint_push_clip_path_start(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        draw_data: *mut *mut c_void,
    ) -> *mut hb_draw_funcs_t;

    /// Invokes the push-clip-path-end callback directly.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_paint_push_clip_path_end(funcs: *mut hb_paint_funcs_t, paint_data: *mut c_void);

    /// Invokes the pop-clip callback directly.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_pop_clip(funcs: *mut hb_paint_funcs_t, paint_data: *mut c_void);

    /// Invokes the paint-colour callback directly.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_color(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        is_foreground: hb_bool_t,
        color: hb_color_t,
    );

    /// Invokes the paint-image callback directly.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_image(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        image: *mut hb_blob_t,
        width: c_uint,
        height: c_uint,
        format: hb_tag_t,
        slant: c_float,
        extents: *mut hb_glyph_extents_t,
    );

    /// Invokes the linear-gradient callback directly.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_linear_gradient(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        color_line: *mut hb_color_line_t,
        x0: c_float,
        y0: c_float,
        x1: c_float,
        y1: c_float,
        x2: c_float,
        y2: c_float,
    );

    /// Invokes the radial-gradient callback directly.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_radial_gradient(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        color_line: *mut hb_color_line_t,
        x0: c_float,
        y0: c_float,
        r0: c_float,
        x1: c_float,
        y1: c_float,
        r1: c_float,
    );

    /// Invokes the sweep-gradient callback directly.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_sweep_gradient(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        color_line: *mut hb_color_line_t,
        x0: c_float,
        y0: c_float,
        start_angle: c_float,
        end_angle: c_float,
    );

    /// Invokes the push-group callback directly.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_push_group(funcs: *mut hb_paint_funcs_t, paint_data: *mut c_void);

    /// Invokes the push-group-for callback directly, announcing the compositing
    /// mode the matching pop will use.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_paint_push_group_for(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        mode: hb_paint_composite_mode_t,
    );

    /// Invokes the pop-group callback directly.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_pop_group(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        mode: hb_paint_composite_mode_t,
    );

    /// Invokes the custom-palette-colour callback directly.
    ///
    /// Returns true if a custom colour was written to `color`, false if the
    /// entry should be taken from the font palette instead.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_paint_custom_palette_color(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        color_index: c_uint,
        color: *mut hb_color_t,
    ) -> hb_bool_t;

    /// Reduces the three anchor points of a COLRv1 linear gradient to the
    /// two-point form most 2D APIs expect.
    ///
    /// A COLRv1 linear gradient is defined by a start point, an end point, and a
    /// rotation point; this projects that triple onto the equivalent
    /// start/end pair, written to `xx0`/`yy0` and `xx1`/`yy1`.
    ///
    /// One of a small set of self-contained helpers that every COLRv1 renderer
    /// ends up reinventing, exposed so third-party paint back ends can share a
    /// single canonical implementation.
    pub fn hb_paint_reduce_linear_anchors(
        x0: c_float,
        y0: c_float,
        x1: c_float,
        y1: c_float,
        x2: c_float,
        y2: c_float,
        xx0: *mut c_float,
        yy0: *mut c_float,
        xx1: *mut c_float,
        yy1: *mut c_float,
    );

    /// Normalizes a colour line's stop offsets in place to the 0–1 range,
    /// reporting the original span through `min` and `max`.
    ///
    /// `stops` points to `len` colour stops, which are rewritten in place.
    pub fn hb_paint_normalize_color_line(
        stops: *mut hb_color_stop_t,
        len: c_uint,
        min: *mut c_float,
        max: *mut c_float,
    );

    /// Decomposes a sweep gradient into angular sectors, invoking `emit_patch`
    /// once per sector.
    ///
    /// `stops` points to `n_stops` colour stops, `extend` is the colour line's
    /// extend mode, and `start_angle`/`end_angle` bound the sweep in radians.
    /// Each callback receives one sector's bounding angles and the colours at
    /// them, which a back end can render as a flat patch or a small linear
    /// gradient.
    pub fn hb_paint_sweep_gradient_tiles(
        stops: *mut hb_color_stop_t,
        n_stops: c_uint,
        extend: hb_paint_extend_t,
        start_angle: c_float,
        end_angle: c_float,
        emit_patch: hb_paint_sweep_gradient_tile_func_t,
        user_data: *mut c_void,
    );
}
