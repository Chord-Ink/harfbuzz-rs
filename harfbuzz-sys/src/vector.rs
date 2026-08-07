//! Glyph vector conversion to SVG and PDF — `hb-vector.h`.
//!
//! This is an optional HarfBuzz sub-library, compiled and exposed only when the
//! crate's `vector` feature is enabled. Unlike the core modules it is *not*
//! glob re-exported at the crate root; reach it as `harfbuzz_sys::vector::*`.
//!
//! Where `hb-draw.h` and `hb-paint.h` hand you glyph geometry through
//! callbacks, this sub-library provides the other end: two ready-made sinks
//! that accumulate that geometry and serialise it as a standalone SVG or PDF
//! document. You never write a callback yourself.
//!
//! * [`hb_vector_draw_t`] consumes monochrome glyph *outlines*. It owns an
//!   immutable [`hb_draw_funcs_t`] (from [`hb_vector_draw_get_funcs`]) that
//!   turns move-to/line-to/curve-to into SVG path data or PDF content-stream
//!   operators, filled with a single foreground colour.
//!
//! * [`hb_vector_paint_t`] consumes *colour* glyphs — `COLR` v0 and v1 paint
//!   graphs, and embedded bitmap images — through an [`hb_paint_funcs_t`] (from
//!   [`hb_vector_paint_get_funcs`]), reproducing gradients, layers, clips, and
//!   compositing in the output format.
//!
//! Both are reference-counted objects created with a `*_create_or_fail()`
//! function, released with `*_destroy()`, and configured with a transform,
//! output scale factors, colours, and numeric precision. Both accumulate any
//! number of glyphs into one document; calling `*_render()` returns the
//! finished bytes as an [`hb_blob_t`] and clears the accumulated content so the
//! context can be reused.
//!
//! Coordinates flow through the pipeline as: font units, then the affine
//! [transform](hb_vector_draw_set_transform), then a division by the
//! [scale factors](hb_vector_draw_set_scale_factor). The transform is where you
//! place a glyph at its pen position and apply the font's scale; the scale
//! factors exist to divide that back down, so a font scaled in 26.6 fixed point
//! can be emitted in user units by passing `64.0`.

use core::ffi::{c_char, c_float, c_int, c_uint, c_void};

use crate::{
    HB_TAG, HB_TAG_NONE, hb_blob_t, hb_bool_t, hb_codepoint_t, hb_color_t, hb_destroy_func_t,
    hb_draw_funcs_t, hb_font_t, hb_glyph_extents_t, hb_paint_funcs_t, hb_user_data_key_t,
};

/// Output format for vector conversion.
///
/// Every value is an [`hb_tag_t`](crate::hb_tag_t): a four-byte code naming the
/// serialisation. The format is chosen once, when the context is created, and
/// cannot be changed afterwards.
///
/// The C enumeration has no sentinel and its largest enumerator is
/// `HB_TAG('s','v','g',' ')` = `0x73766720`, which fits in a signed `int` —
/// hence `c_int`.
///
/// Since HarfBuzz 13.0.0.
pub type hb_vector_format_t = c_int;

/// Invalid format. Equal to [`HB_TAG_NONE`]; the `*_create_or_fail` functions
/// reject it.
///
/// Since HarfBuzz 13.0.0.
pub const HB_VECTOR_FORMAT_INVALID: hb_vector_format_t = HB_TAG_NONE as hb_vector_format_t;

/// SVG output (`svg `).
///
/// Since HarfBuzz 13.0.0.
pub const HB_VECTOR_FORMAT_SVG: hb_vector_format_t =
    HB_TAG(b's', b'v', b'g', b' ') as hb_vector_format_t;

/// PDF output (`pdf `).
///
/// Since HarfBuzz 13.0.0.
pub const HB_VECTOR_FORMAT_PDF: hb_vector_format_t =
    HB_TAG(b'p', b'd', b'f', b' ') as hb_vector_format_t;

/// Vector output extents, mapped to the SVG `viewBox` or the PDF `MediaBox`.
///
/// The values are in output space — that is, after the context's transform and
/// after division by its scale factors. A context with no extents produces no
/// output at all, because there is no box to render into.
///
/// Since HarfBuzz 13.0.0.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct hb_vector_extents_t {
    /// Left edge of the output coordinate system.
    pub x: c_float,
    /// Top edge of the output coordinate system.
    pub y: c_float,
    /// Width of the output coordinate system.
    pub width: c_float,
    /// Height of the output coordinate system.
    pub height: c_float,
}

/// Controls whether the convenience glyph APIs update the context's extents.
///
/// The C enumeration has no sentinel and its largest enumerator is 1, so it
/// fits in an `int` — hence `c_int`.
///
/// Since HarfBuzz 13.0.0.
pub type hb_vector_extents_mode_t = c_int;

/// Do not update extents.
///
/// Since HarfBuzz 13.0.0.
pub const HB_VECTOR_EXTENTS_MODE_NONE: hb_vector_extents_mode_t = 0;

/// Union the glyph's ink extents into the context's current extents.
///
/// Since HarfBuzz 13.0.0.
pub const HB_VECTOR_EXTENTS_MODE_EXPAND: hb_vector_extents_mode_t = 1;

crate::opaque_handle! {
    /// Opaque draw context for vector outline conversion.
    ///
    /// Reference-counted. Create with [`hb_vector_draw_create_or_fail`],
    /// release with [`hb_vector_draw_destroy`].
    ///
    /// Since HarfBuzz 13.0.0.
    hb_vector_draw_t
}

unsafe extern "C" {
    /// Creates a new draw context for vector output.
    ///
    /// Returns a newly allocated context, or null on failure — including when
    /// `format` is neither [`HB_VECTOR_FORMAT_SVG`] nor
    /// [`HB_VECTOR_FORMAT_PDF`]. Release it with [`hb_vector_draw_destroy`].
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_create_or_fail(format: hb_vector_format_t) -> *mut hb_vector_draw_t;

    /// Increases the reference count on `draw` and returns it.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_reference(draw: *mut hb_vector_draw_t) -> *mut hb_vector_draw_t;

    /// Decreases the reference count on `draw`, destroying it and freeing all
    /// its memory when the count reaches zero.
    ///
    /// Any blob handed over with [`hb_vector_draw_recycle_blob`] is released
    /// here too.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_destroy(draw: *mut hb_vector_draw_t);

    /// Attaches a user-data key/data pair to the specified draw context.
    ///
    /// `destroy` — which may be null — is called with `data` when the context
    /// is destroyed or the value is replaced. `replace` decides whether
    /// existing data stored under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_set_user_data(
        draw: *mut hb_vector_draw_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to the draw context under the specified
    /// key.
    ///
    /// Ownership stays with the context; the caller must not free the returned
    /// pointer. Null when nothing is stored under `key`.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_get_user_data(
        draw: *const hb_vector_draw_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Sets the affine transform applied to glyph geometry as it is drawn.
    ///
    /// The six components form the matrix `[xx yx; xy yy]` plus the translation
    /// `(dx, dy)`. This is where you place a glyph at its pen position.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_set_transform(
        draw: *mut hb_vector_draw_t,
        xx: c_float,
        yx: c_float,
        xy: c_float,
        yy: c_float,
        dx: c_float,
        dy: c_float,
    );

    /// Fetches the affine transform used when drawing glyphs.
    ///
    /// Every out parameter may be null, in which case it is skipped.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_get_transform(
        draw: *const hb_vector_draw_t,
        xx: *mut c_float,
        yx: *mut c_float,
        xy: *mut c_float,
        yy: *mut c_float,
        dx: *mut c_float,
        dy: *mut c_float,
    );

    /// Sets additional output scaling factors.
    ///
    /// Transformed coordinates are *divided* by these before being written out,
    /// so a font scaled in 26.6 fixed point renders in user units when both
    /// factors are `64.0`. Values that are not greater than zero are clamped to
    /// `1.0`.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_set_scale_factor(
        draw: *mut hb_vector_draw_t,
        x_scale_factor: c_float,
        y_scale_factor: c_float,
    );

    /// Fetches the additional output scaling factors.
    ///
    /// Either out parameter may be null, in which case it is skipped.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_get_scale_factor(
        draw: *const hb_vector_draw_t,
        x_scale_factor: *mut c_float,
        y_scale_factor: *mut c_float,
    );

    /// Sets or expands the output extents on `draw`.
    ///
    /// Passing null clears the extents. A box with zero width or zero height is
    /// ignored. Otherwise the box — given in input space, so it is divided by
    /// the current scale factors — is normalised to a positive-size rectangle
    /// and unioned into whatever extents the context already has.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_set_extents(
        draw: *mut hb_vector_draw_t,
        extents: *const hb_vector_extents_t,
    );

    /// Fetches the current output extents from `draw`.
    ///
    /// Returns true if extents are set, false otherwise. `extents` may be null
    /// when you only want the boolean; it is written only when the return value
    /// is true.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_get_extents(
        draw: *const hb_vector_draw_t,
        extents: *mut hb_vector_extents_t,
    ) -> hb_bool_t;

    /// Expands the context's extents by a glyph's ink box, in font units, under
    /// the current transform and scale factors.
    ///
    /// All four corners are transformed and their bounding box is unioned in,
    /// so a rotating or skewing transform is handled correctly.
    ///
    /// Returns true on success, false if the transformed box is degenerate —
    /// zero width or zero height — in which case the extents are unchanged.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_set_glyph_extents(
        draw: *mut hb_vector_draw_t,
        glyph_extents: *const hb_glyph_extents_t,
    ) -> hb_bool_t;

    /// Fetches the output format `draw` was created with.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_draw_get_format(draw: *const hb_vector_draw_t) -> hb_vector_format_t;

    /// Fetches the draw callbacks for feeding outline data into `draw`.
    ///
    /// Pass `draw` itself as the `draw_data` argument when calling them — for
    /// instance to [`hb_font_draw_glyph`](crate::hb_font_draw_glyph).
    ///
    /// The returned object is a shared, immutable singleton per format: it is
    /// borrowed, not owned, so do not destroy it. Null if `draw` is null or its
    /// format is invalid.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_draw_get_funcs(draw: *const hb_vector_draw_t) -> *mut hb_draw_funcs_t;

    /// Flushes any pending path and starts a new one.
    ///
    /// Call this between glyphs that you feed in by hand, so their outlines are
    /// emitted as separate elements and fill rules do not interact across
    /// glyphs.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_draw_new_path(draw: *mut hb_vector_draw_t);

    /// Draws one glyph into `draw`.
    ///
    /// Equivalent to [`hb_vector_draw_glyph_or_fail`] with the return value
    /// ignored.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_draw_glyph(
        draw: *mut hb_vector_draw_t,
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        extents_mode: hb_vector_extents_mode_t,
    );

    /// Draws one glyph into `draw`, reporting whether any outline was emitted.
    ///
    /// With [`HB_VECTOR_EXTENTS_MODE_EXPAND`] the glyph's ink extents are
    /// unioned into the context's extents first. Then the pending path is
    /// flushed and the glyph's outline is fed through the callbacks from
    /// [`hb_vector_draw_get_funcs`].
    ///
    /// Returns true if glyph data was emitted, false otherwise.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_draw_glyph_or_fail(
        draw: *mut hb_vector_draw_t,
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        extents_mode: hb_vector_extents_mode_t,
    ) -> hb_bool_t;

    /// Sets the number of decimal places used when writing numbers.
    ///
    /// Clamped to at most 12. The default is 2.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_draw_set_precision(draw: *mut hb_vector_draw_t, precision: c_uint);

    /// Fetches the numeric output precision, or the default if none was set.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_draw_get_precision(draw: *const hb_vector_draw_t) -> c_uint;

    /// Sets the fill colour for drawn glyph outlines. The default is opaque
    /// black.
    ///
    /// Any path accumulated so far is flushed first, so a colour change applies
    /// only to subsequent geometry.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_draw_set_foreground(draw: *mut hb_vector_draw_t, foreground: hb_color_t);

    /// Fetches the foreground fill colour.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_draw_get_foreground(draw: *const hb_vector_draw_t) -> hb_color_t;

    /// Sets the background colour. The default is transparent, meaning no
    /// background.
    ///
    /// If the colour is not fully transparent, a filled rectangle covering the
    /// extents is emitted behind all content.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_draw_set_background(draw: *mut hb_vector_draw_t, background: hb_color_t);

    /// Fetches the background colour.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_draw_get_background(draw: *const hb_vector_draw_t) -> hb_color_t;

    /// Renders the accumulated content to an output blob.
    ///
    /// Returns a new blob holding the complete SVG or PDF document, or null if
    /// rendering cannot proceed — most commonly because the context has no
    /// extents. Release it with [`hb_blob_destroy`](crate::hb_blob_destroy), or
    /// hand it back with [`hb_vector_draw_recycle_blob`].
    ///
    /// On success the context is cleared as if by [`hb_vector_draw_clear`],
    /// which discards the extents — read them with
    /// [`hb_vector_draw_get_extents`] *before* rendering if you need them.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_render(draw: *mut hb_vector_draw_t) -> *mut hb_blob_t;

    /// Discards the accumulated output and extents so `draw` can be reused for
    /// another render.
    ///
    /// User configuration — transform, scale factors, precision, colours — is
    /// preserved. Use [`hb_vector_draw_reset`] to also restore the defaults.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_draw_clear(draw: *mut hb_vector_draw_t);

    /// Resets `draw` to its default configuration and clears accumulated
    /// content.
    ///
    /// The transform becomes the identity, both scale factors become `1.0`, and
    /// precision returns to 2.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_reset(draw: *mut hb_vector_draw_t);

    /// Hands a previously rendered blob back to `draw` so its buffer can be
    /// reused by later render calls.
    ///
    /// Takes ownership of `blob`: the context stores it and destroys it later.
    /// Passing null, or the singleton empty blob, simply drops any blob held
    /// from an earlier call. Do not use `blob` afterwards.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_draw_recycle_blob(draw: *mut hb_vector_draw_t, blob: *mut hb_blob_t);
}

crate::opaque_handle! {
    /// Opaque paint context for vector color-glyph conversion.
    ///
    /// Reference-counted. Create with [`hb_vector_paint_create_or_fail`],
    /// release with [`hb_vector_paint_destroy`].
    ///
    /// Since HarfBuzz 13.0.0.
    hb_vector_paint_t
}

unsafe extern "C" {
    /// Creates a new paint context for vector output.
    ///
    /// Returns a newly allocated context, or null on failure — including when
    /// `format` is neither [`HB_VECTOR_FORMAT_SVG`] nor
    /// [`HB_VECTOR_FORMAT_PDF`]. Release it with [`hb_vector_paint_destroy`].
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_create_or_fail(format: hb_vector_format_t) -> *mut hb_vector_paint_t;

    /// Increases the reference count on `paint` and returns it.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_reference(paint: *mut hb_vector_paint_t) -> *mut hb_vector_paint_t;

    /// Decreases the reference count on `paint`, destroying it and freeing all
    /// its memory when the count reaches zero.
    ///
    /// The SVG id prefix and any blob handed over with
    /// [`hb_vector_paint_recycle_blob`] are released here too.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_destroy(paint: *mut hb_vector_paint_t);

    /// Attaches a user-data key/data pair to the specified paint context.
    ///
    /// `destroy` — which may be null — is called with `data` when the context
    /// is destroyed or the value is replaced. `replace` decides whether
    /// existing data stored under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_set_user_data(
        paint: *mut hb_vector_paint_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to the paint context under the specified
    /// key.
    ///
    /// Ownership stays with the context; the caller must not free the returned
    /// pointer. Null when nothing is stored under `key`.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_get_user_data(
        paint: *const hb_vector_paint_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Sets the affine transform applied to glyph geometry as it is painted.
    ///
    /// The six components form the matrix `[xx yx; xy yy]` plus the translation
    /// `(dx, dy)`.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_set_transform(
        paint: *mut hb_vector_paint_t,
        xx: c_float,
        yx: c_float,
        xy: c_float,
        yy: c_float,
        dx: c_float,
        dy: c_float,
    );

    /// Fetches the affine transform used when painting glyphs.
    ///
    /// Every out parameter may be null, in which case it is skipped.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_get_transform(
        paint: *const hb_vector_paint_t,
        xx: *mut c_float,
        yx: *mut c_float,
        xy: *mut c_float,
        yy: *mut c_float,
        dx: *mut c_float,
        dy: *mut c_float,
    );

    /// Sets additional output scaling factors.
    ///
    /// Transformed coordinates are *divided* by these before being written out.
    /// Values that are not greater than zero are clamped to `1.0`.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_set_scale_factor(
        paint: *mut hb_vector_paint_t,
        x_scale_factor: c_float,
        y_scale_factor: c_float,
    );

    /// Fetches the additional output scaling factors.
    ///
    /// Either out parameter may be null, in which case it is skipped.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_get_scale_factor(
        paint: *const hb_vector_paint_t,
        x_scale_factor: *mut c_float,
        y_scale_factor: *mut c_float,
    );

    /// Sets or expands the output extents on `paint`.
    ///
    /// Passing null clears the extents. A box with zero width or zero height is
    /// ignored. Otherwise the box — given in input space, so it is divided by
    /// the current scale factors — is normalised to a positive-size rectangle
    /// and unioned into whatever extents the context already has.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_set_extents(
        paint: *mut hb_vector_paint_t,
        extents: *const hb_vector_extents_t,
    );

    /// Fetches the current output extents from `paint`.
    ///
    /// Returns true if extents are set, false otherwise. `extents` may be null
    /// when you only want the boolean; it is written only when the return value
    /// is true.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_get_extents(
        paint: *const hb_vector_paint_t,
        extents: *mut hb_vector_extents_t,
    ) -> hb_bool_t;

    /// Expands the context's extents by a glyph's ink box, in font units, under
    /// the current transform and scale factors.
    ///
    /// All four corners are transformed and their bounding box is unioned in.
    ///
    /// Returns true on success, false if the transformed box is degenerate —
    /// zero width or zero height — in which case the extents are unchanged.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_set_glyph_extents(
        paint: *mut hb_vector_paint_t,
        glyph_extents: *const hb_glyph_extents_t,
    ) -> hb_bool_t;

    /// Sets the fallback foreground colour used by paint operations.
    ///
    /// `COLR` paints that reference the text foreground colour resolve to this.
    /// The default is opaque black.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_set_foreground(paint: *mut hb_vector_paint_t, foreground: hb_color_t);

    /// Fetches the foreground colour, or the default opaque black if none was
    /// set.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_paint_get_foreground(paint: *const hb_vector_paint_t) -> hb_color_t;

    /// Sets the background colour. The default is transparent, meaning no
    /// background.
    ///
    /// If the colour is not fully transparent, a filled rectangle covering the
    /// extents is emitted behind all glyph content.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_paint_set_background(paint: *mut hb_vector_paint_t, background: hb_color_t);

    /// Fetches the background colour, or transparent if none was set.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_paint_get_background(paint: *const hb_vector_paint_t) -> hb_color_t;

    /// Sets the `CPAL` colour-palette index used by paint operations.
    ///
    /// The default is 0.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_set_palette(paint: *mut hb_vector_paint_t, palette: c_int);

    /// Fetches the palette index, or 0 if none was set.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_paint_get_palette(paint: *const hb_vector_paint_t) -> c_int;

    /// Overrides one font palette colour entry for subsequent paint operations.
    ///
    /// Overrides are keyed by `color_index` and persist on the context until
    /// cleared, or replaced for the same index. They are consulted by every
    /// paint operation that resolves a `CPAL` entry, including SVG glyph
    /// content that uses `var(--colorN)`.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_set_custom_palette_color(
        paint: *mut hb_vector_paint_t,
        color_index: c_uint,
        color: hb_color_t,
    );

    /// Clears all custom palette colour overrides previously set on `paint`.
    ///
    /// Afterwards, palette lookups use the selected font palette unmodified.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_clear_custom_palette_colors(paint: *mut hb_vector_paint_t);

    /// Fetches the output format `paint` was created with.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_paint_get_format(paint: *const hb_vector_paint_t) -> hb_vector_format_t;

    /// Fetches the paint callbacks for emitting paint operations into `paint`.
    ///
    /// Pass `paint` itself as the `paint_data` argument when calling them — for
    /// instance to [`hb_font_paint_glyph`](crate::hb_font_paint_glyph).
    ///
    /// The returned object is a shared, immutable singleton per format: it is
    /// borrowed, not owned, so do not destroy it. Null if `paint` is null or
    /// its format is invalid.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_paint_get_funcs(paint: *const hb_vector_paint_t) -> *mut hb_paint_funcs_t;

    /// Paints one glyph into `paint`.
    ///
    /// Unlike [`hb_vector_paint_glyph_or_fail`], a glyph with no colour paint
    /// data falls back to a synthesised foreground-coloured outline, so any
    /// glyph with an outline or a bitmap image produces output.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_paint_glyph(
        paint: *mut hb_vector_paint_t,
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        extents_mode: hb_vector_extents_mode_t,
    );

    /// Paints one colour glyph into `paint`, reporting whether anything was
    /// emitted.
    ///
    /// With [`HB_VECTOR_EXTENTS_MODE_EXPAND`] the glyph's ink extents are
    /// unioned into the context's extents first. The context's transform is
    /// then pushed onto the paint callbacks, the glyph is painted with the
    /// current palette and foreground colour, and the transform is popped.
    ///
    /// Returns true if glyph paint data was emitted, false otherwise.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_paint_glyph_or_fail(
        paint: *mut hb_vector_paint_t,
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        extents_mode: hb_vector_extents_mode_t,
    ) -> hb_bool_t;

    /// Sets the number of decimal places used when writing numbers.
    ///
    /// Clamped to at most 12. The default is 2.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_paint_set_precision(paint: *mut hb_vector_paint_t, precision: c_uint);

    /// Fetches the numeric output precision, or the default if none was set.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_paint_get_precision(paint: *const hb_vector_paint_t) -> c_uint;

    /// Namespaces the paint context's SVG output.
    ///
    /// `prefix` is a NUL-terminated ASCII string prepended to every emitted SVG
    /// `id` and `url(#…)` reference, or null for none. The string is copied.
    ///
    /// Callers that inject several hb-vector SVGs into one document must give
    /// each context a distinct prefix, or the short ids used for clip paths,
    /// gradients, and `use` references will collide in the DOM.
    ///
    /// Has no effect on PDF output.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_paint_set_svg_prefix(paint: *mut hb_vector_paint_t, prefix: *const c_char);

    /// Fetches the SVG id prefix, or the empty string if none was set.
    ///
    /// The pointer belongs to the context and stays valid until the next
    /// [`hb_vector_paint_set_svg_prefix`] or [`hb_vector_paint_destroy`] call
    /// on it.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_paint_get_svg_prefix(paint: *const hb_vector_paint_t) -> *const c_char;

    /// Renders the accumulated content to an output blob.
    ///
    /// Returns a new blob holding the complete SVG or PDF document, or null if
    /// rendering cannot proceed — most commonly because the context has no
    /// extents. Release it with [`hb_blob_destroy`](crate::hb_blob_destroy), or
    /// hand it back with [`hb_vector_paint_recycle_blob`].
    ///
    /// On success the context is cleared as if by [`hb_vector_paint_clear`],
    /// which discards the extents — read them with
    /// [`hb_vector_paint_get_extents`] *before* rendering if you need them.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_render(paint: *mut hb_vector_paint_t) -> *mut hb_blob_t;

    /// Discards the accumulated output and extents so `paint` can be reused for
    /// another render.
    ///
    /// User configuration — transform, scale factors, precision, foreground,
    /// palette, custom palette colours — is preserved. Use
    /// [`hb_vector_paint_reset`] to also restore the defaults.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_vector_paint_clear(paint: *mut hb_vector_paint_t);

    /// Resets `paint` to its default configuration and clears accumulated
    /// content.
    ///
    /// The transform becomes the identity, both scale factors become `1.0`, the
    /// foreground returns to opaque black, the palette index to 0, and
    /// precision to 2.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_reset(paint: *mut hb_vector_paint_t);

    /// Hands a previously rendered blob back to `paint` so its buffer can be
    /// reused by later render calls.
    ///
    /// Takes ownership of `blob`: the context stores it and destroys it later.
    /// Passing null, or the singleton empty blob, simply drops any blob held
    /// from an earlier call. Do not use `blob` afterwards.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_vector_paint_recycle_blob(paint: *mut hb_vector_paint_t, blob: *mut hb_blob_t);
}
