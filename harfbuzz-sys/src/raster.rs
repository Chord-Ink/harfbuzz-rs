//! CPU glyph rasterization — `hb-raster.h`.
//!
//! HarfBuzz's raster sub-library turns glyph outlines and colour-glyph paint
//! graphs into pixel buffers, entirely on the CPU and without any external
//! graphics dependency. It is an optional part of HarfBuzz: the sources are
//! compiled only when this crate's `raster` feature is enabled, which is also
//! what gates this module. Because of that gating the module is exposed as
//! `harfbuzz_sys::raster` rather than being re-exported at the crate root.
//!
//! Three object types make up the API:
//!
//! * [`hb_raster_image_t`] — a reference-counted pixel buffer plus the extents
//!   and [`hb_raster_format_t`] that describe its layout. It is the output of
//!   both rasterizers, and can also be filled from a PNG blob.
//! * [`hb_raster_draw_t`] — the outline rasterizer. It exposes an
//!   [`hb_draw_funcs_t`] (via [`hb_raster_draw_get_funcs`]) that accumulates
//!   flattened outline geometry, and renders that to an 8-bit alpha coverage
//!   mask with [`hb_raster_draw_render`].
//! * [`hb_raster_paint_t`] — the colour-glyph rasterizer. It exposes an
//!   [`hb_paint_funcs_t`] (via [`hb_raster_paint_get_funcs`]) that executes a
//!   `COLR` v0/v1 paint graph, and produces a 32-bit BGRA image with
//!   [`hb_raster_paint_render`].
//!
//! The usual sequence for either rasterizer is: create it, set the transform
//! and scale factors that map glyph space to pixels, set the output extents
//! (directly, or from an [`hb_glyph_extents_t`]), feed one or more glyphs in,
//! then render. Both rasterizers are reusable: rendering clears the accumulated
//! geometry, and a finished image can be handed back with
//! [`hb_raster_draw_recycle_image`] or [`hb_raster_paint_recycle_image`] so the
//! next render reuses its allocation instead of making a new one.
//!
//! Setting extents explicitly before rendering avoids implicit allocations and
//! gives deterministic bounds. [`hb_raster_paint_render`] *requires* it, and
//! returns null otherwise.

use core::ffi::{c_float, c_int, c_uint, c_void};

use crate::{
    hb_blob_t, hb_bool_t, hb_codepoint_t, hb_color_t, hb_destroy_func_t, hb_draw_funcs_t,
    hb_font_t, hb_glyph_extents_t, hb_paint_funcs_t, hb_user_data_key_t,
};

/// Pixel format for raster images.
///
/// The C enumeration has no explicit sentinel and its largest enumerator is 1,
/// so it fits in an `int`.
///
/// Since HarfBuzz 13.0.0.
pub type hb_raster_format_t = c_int;

/// 8-bit alpha-only coverage: one byte per pixel. The output format of
/// [`hb_raster_draw_render`].
pub const HB_RASTER_FORMAT_A8: hb_raster_format_t = 0;

/// 32-bit BGRA colour: four bytes per pixel, blue first and alpha last. The
/// output format of [`hb_raster_paint_render`].
pub const HB_RASTER_FORMAT_BGRA32: hb_raster_format_t = 1;

/// Pixel-buffer extents for raster operations.
///
/// The origin is expressed in glyph space — the pixel grid that the
/// rasterizer's transform maps onto — while width, height, and stride describe
/// the buffer itself. Rows are stored bottom-to-top, so the first row of the
/// buffer is the one at `y_origin`.
///
/// Since HarfBuzz 13.0.0.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_raster_extents_t {
    /// X coordinate of the left edge of the image in glyph space.
    pub x_origin: c_int,
    /// Y coordinate of the bottom edge of the image in glyph space.
    pub y_origin: c_int,
    /// Width in pixels.
    pub width: c_uint,
    /// Height in pixels.
    pub height: c_uint,
    /// Bytes per row. Zero means auto-calculate on input, and is filled in on
    /// output.
    pub stride: c_uint,
}

crate::opaque_handle! {
    /// An opaque raster image object holding a pixel buffer produced by
    /// [`hb_raster_draw_render`] or [`hb_raster_paint_render`].
    ///
    /// Use [`hb_raster_image_get_buffer`] and [`hb_raster_image_get_extents`]
    /// to access the pixels.
    ///
    /// Since HarfBuzz 13.0.0.
    hb_raster_image_t
}

crate::opaque_handle! {
    /// An opaque outline rasterizer object.
    ///
    /// Accumulates glyph outlines through the [`hb_draw_funcs_t`] callbacks
    /// obtained from [`hb_raster_draw_get_funcs`], then produces an
    /// [`hb_raster_image_t`] with [`hb_raster_draw_render`].
    ///
    /// Since HarfBuzz 13.0.0.
    hb_raster_draw_t
}

crate::opaque_handle! {
    /// An opaque colour-glyph paint context.
    ///
    /// Implements the [`hb_paint_funcs_t`] callbacks that render `COLR` v0/v1
    /// colour glyphs into a [`HB_RASTER_FORMAT_BGRA32`] [`hb_raster_image_t`].
    ///
    /// Since HarfBuzz 13.0.0.
    hb_raster_paint_t
}

unsafe extern "C" {
    // --- hb_raster_image_t --------------------------------------------------

    /// Creates a new raster image object with a reference count of one.
    ///
    /// Returns the new image, or null on allocation failure. Release it with
    /// [`hb_raster_image_destroy`], or transfer it for reuse with
    /// [`hb_raster_draw_recycle_image`] or [`hb_raster_paint_recycle_image`].
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_image_create_or_fail() -> *mut hb_raster_image_t;

    /// Increases the reference count on `image` by one and returns it.
    ///
    /// This prevents `image` from being destroyed until a matching call to
    /// [`hb_raster_image_destroy`] is made.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_image_reference(image: *mut hb_raster_image_t) -> *mut hb_raster_image_t;

    /// Decreases the reference count on `image` by one, freeing the image and
    /// its pixel buffer when the count reaches zero.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_image_destroy(image: *mut hb_raster_image_t);

    /// Attaches a user-data key/data pair to the specified raster image.
    ///
    /// `destroy` — which may be null — is called with `data` when the image is
    /// destroyed or the value is replaced. `replace` decides whether existing
    /// data stored under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_image_set_user_data(
        image: *mut hb_raster_image_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to the image under the specified key.
    ///
    /// Ownership stays with the image; the caller must not free the returned
    /// pointer.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_image_get_user_data(
        image: *const hb_raster_image_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Configures the image's format and extents together, resizing the backing
    /// storage at most once. Pixel contents are *not* cleared.
    ///
    /// Passing null for `extents` clears the extents and releases the backing
    /// allocation.
    ///
    /// Returns true if configuration succeeds, false on allocation failure.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_image_configure(
        image: *mut hb_raster_image_t,
        format: hb_raster_format_t,
        extents: *const hb_raster_extents_t,
    ) -> hb_bool_t;

    /// Clears the image's pixels to zero, keeping the current extents and
    /// format.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_image_clear(image: *mut hb_raster_image_t);

    /// Fetches the raw pixel buffer of the image.
    ///
    /// The layout is described by the extents from
    /// [`hb_raster_image_get_extents`] and the format from
    /// [`hb_raster_image_get_format`]. Rows are stored bottom-to-top.
    ///
    /// Returns the pixel buffer, or null. The bytes belong to the image and
    /// must not be freed by the caller.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_image_get_buffer(image: *const hb_raster_image_t) -> *const u8;

    /// Fetches the pixel-buffer extents of the image into `extents`, which may
    /// be null.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_image_get_extents(
        image: *const hb_raster_image_t,
        extents: *mut hb_raster_extents_t,
    );

    /// Fetches the pixel format of the image.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_image_get_format(image: *const hb_raster_image_t) -> hb_raster_format_t;

    /// Replaces the image's contents by deserializing a PNG blob into a
    /// [`HB_RASTER_FORMAT_BGRA32`] raster image.
    ///
    /// On success the extents are reset to pixel extents with origin `(0, 0)`,
    /// and rows in the resulting buffer are stored bottom-to-top. On failure
    /// the image is left unchanged.
    ///
    /// Returns true if deserialization succeeded, false otherwise — including
    /// when HarfBuzz was built without libpng.
    ///
    /// Since HarfBuzz 13.1.0.
    pub fn hb_raster_image_deserialize_from_png_or_fail(
        image: *mut hb_raster_image_t,
        png: *mut hb_blob_t,
    ) -> hb_bool_t;

    /// Serializes the image to a PNG blob.
    ///
    /// Only [`HB_RASTER_FORMAT_BGRA32`] images are currently supported.
    ///
    /// Returns a newly allocated PNG blob, or null on failure — including when
    /// HarfBuzz was built without libpng. Release it with
    /// [`hb_blob_destroy`](crate::hb_blob_destroy).
    ///
    /// Since HarfBuzz 13.1.0.
    pub fn hb_raster_image_serialize_to_png_or_fail(
        image: *const hb_raster_image_t,
    ) -> *mut hb_blob_t;

    // --- hb_raster_draw_t ---------------------------------------------------

    /// Creates a new outline rasterizer with a reference count of one.
    ///
    /// Returns the new rasterizer, or null on allocation failure. Release it
    /// with [`hb_raster_draw_destroy`].
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_create_or_fail() -> *mut hb_raster_draw_t;

    /// Increases the reference count on `draw` by one and returns it.
    ///
    /// This prevents `draw` from being destroyed until a matching call to
    /// [`hb_raster_draw_destroy`] is made.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_reference(draw: *mut hb_raster_draw_t) -> *mut hb_raster_draw_t;

    /// Decreases the reference count on `draw` by one, freeing the rasterizer
    /// when the count reaches zero.
    ///
    /// Any image previously handed over with [`hb_raster_draw_recycle_image`]
    /// is destroyed along with it.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_destroy(draw: *mut hb_raster_draw_t);

    /// Attaches a user-data key/data pair to the specified rasterizer.
    ///
    /// `destroy` — which may be null — is called with `data` when the
    /// rasterizer is destroyed or the value is replaced. `replace` decides
    /// whether existing data stored under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_set_user_data(
        draw: *mut hb_raster_draw_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to the rasterizer under the specified
    /// key.
    ///
    /// Ownership stays with the rasterizer; the caller must not free the
    /// returned pointer.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_get_user_data(
        draw: *const hb_raster_draw_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Sets the 2×3 affine transform applied to all incoming draw coordinates
    /// before rasterization. The default is the identity.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_set_transform(
        draw: *mut hb_raster_draw_t,
        xx: c_float,
        yx: c_float,
        xy: c_float,
        yy: c_float,
        dx: c_float,
        dy: c_float,
    );

    /// Fetches the current affine transform of the rasterizer. Every output
    /// pointer may be null.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_get_transform(
        draw: *const hb_raster_draw_t,
        xx: *mut c_float,
        yx: *mut c_float,
        xy: *mut c_float,
        yy: *mut c_float,
        dx: *mut c_float,
        dy: *mut c_float,
    );

    /// Sets the post-transform minification factors applied during
    /// rasterization.
    ///
    /// Factors larger than one shrink the output in pixels. The default is one,
    /// and values that are not strictly positive are silently replaced by one.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_set_scale_factor(
        draw: *mut hb_raster_draw_t,
        x_scale_factor: c_float,
        y_scale_factor: c_float,
    );

    /// Fetches the current post-transform minification factors. Both output
    /// pointers may be null.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_get_scale_factor(
        draw: *const hb_raster_draw_t,
        x_scale_factor: *mut c_float,
        y_scale_factor: *mut c_float,
    );

    /// Overrides the output image extents for the next render.
    ///
    /// When set, [`hb_raster_draw_render`] uses the given extents instead of
    /// computing them from the accumulated geometry.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_set_extents(
        draw: *mut hb_raster_draw_t,
        extents: *const hb_raster_extents_t,
    );

    /// Fetches the currently configured output extents into `extents`, which
    /// may be null.
    ///
    /// Returns true if extents are set, false otherwise.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_get_extents(
        draw: *const hb_raster_draw_t,
        extents: *mut hb_raster_extents_t,
    ) -> hb_bool_t;

    /// Transforms `glyph_extents` with the rasterizer's current transform and
    /// sets the resulting pixel extents for the next render.
    ///
    /// Equivalent to computing a transformed bounding box in pixel space and
    /// calling [`hb_raster_draw_set_extents`].
    ///
    /// Returns true if the transformed extents are non-empty and were set;
    /// false otherwise, in which case the rasterizer is left with no extents.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_set_glyph_extents(
        draw: *mut hb_raster_draw_t,
        glyph_extents: *const hb_glyph_extents_t,
    ) -> hb_bool_t;

    /// Fetches the [`hb_draw_funcs_t`] that feeds outline data into the
    /// rasterizer.
    ///
    /// Pass `draw` itself as the `draw_data` argument when calling the draw
    /// functions. The returned funcs object is a shared singleton owned by
    /// HarfBuzz; do not destroy it.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_raster_draw_get_funcs(draw: *const hb_raster_draw_t) -> *mut hb_draw_funcs_t;

    /// Draws one glyph into the rasterizer using its current transform.
    ///
    /// Equivalent to [`hb_raster_draw_glyph_or_fail`] with the return value
    /// ignored.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_raster_draw_glyph(
        draw: *mut hb_raster_draw_t,
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
    );

    /// Draws one glyph into the rasterizer, reporting whether it had outlines.
    ///
    /// Equivalent to calling
    /// [`hb_font_draw_glyph_or_fail`](crate::hb_font_draw_glyph_or_fail) with
    /// [`hb_raster_draw_get_funcs`] and `draw` as the draw data.
    ///
    /// Returns true if the glyph was drawn, false if the font has no outlines
    /// for it.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_raster_draw_glyph_or_fail(
        draw: *mut hb_raster_draw_t,
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
    ) -> hb_bool_t;

    /// Rasterizes the accumulated outline geometry into a new
    /// [`hb_raster_image_t`].
    ///
    /// The output format is always [`HB_RASTER_FORMAT_A8`]. After rendering,
    /// the accumulated edges are cleared so the rasterizer can be reused.
    ///
    /// Returns the rendered image, or null on allocation or configuration
    /// failure; if no geometry was accumulated it returns an empty image.
    /// Release it with [`hb_raster_image_destroy`], or hand it back with
    /// [`hb_raster_draw_recycle_image`].
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_render(draw: *mut hb_raster_draw_t) -> *mut hb_raster_image_t;

    /// Discards accumulated geometry and extents so the rasterizer can be
    /// reused for another render.
    ///
    /// User configuration — the transform and the scale factors — is preserved.
    /// Use [`hb_raster_draw_reset`] to also reset that to defaults.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_raster_draw_clear(draw: *mut hb_raster_draw_t);

    /// Resets the rasterizer to its initial state, clearing all accumulated
    /// geometry, the transform, the scale factors, and any fixed extents.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_reset(draw: *mut hb_raster_draw_t);

    /// Recycles `image` for reuse by a subsequent [`hb_raster_draw_render`]
    /// call, avoiding a per-render allocation.
    ///
    /// The caller transfers ownership of `image` to `draw` and must not use it
    /// afterwards. If `draw` already holds a recycled image, that previous
    /// image is destroyed.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_draw_recycle_image(
        draw: *mut hb_raster_draw_t,
        image: *mut hb_raster_image_t,
    );

    // --- hb_raster_paint_t --------------------------------------------------

    /// Creates a new colour-glyph paint context with a reference count of one.
    ///
    /// Returns the new context, or null on allocation failure. Release it with
    /// [`hb_raster_paint_destroy`].
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_create_or_fail() -> *mut hb_raster_paint_t;

    /// Increases the reference count on `paint` by one and returns it.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_reference(paint: *mut hb_raster_paint_t) -> *mut hb_raster_paint_t;

    /// Decreases the reference count on `paint` by one, freeing the paint
    /// context — and every image it has cached — when the count reaches zero.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_destroy(paint: *mut hb_raster_paint_t);

    /// Attaches a user-data key/data pair to the specified paint context.
    ///
    /// `destroy` — which may be null — is called with `data` when the context
    /// is destroyed or the value is replaced. `replace` decides whether
    /// existing data stored under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_set_user_data(
        paint: *mut hb_raster_paint_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to the paint context under the specified
    /// key.
    ///
    /// Ownership stays with the context; the caller must not free the returned
    /// pointer.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_get_user_data(
        paint: *const hb_raster_paint_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Sets the base 2×3 affine transform that maps glyph-space coordinates to
    /// pixel-space coordinates. The default is the identity.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_set_transform(
        paint: *mut hb_raster_paint_t,
        xx: c_float,
        yx: c_float,
        xy: c_float,
        yy: c_float,
        dx: c_float,
        dy: c_float,
    );

    /// Fetches the current base affine transform. Every output pointer may be
    /// null.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_get_transform(
        paint: *const hb_raster_paint_t,
        xx: *mut c_float,
        yx: *mut c_float,
        xy: *mut c_float,
        yy: *mut c_float,
        dx: *mut c_float,
        dy: *mut c_float,
    );

    /// Sets the post-transform minification factors applied during painting.
    ///
    /// Factors larger than one shrink the output in pixels. The default is one,
    /// and values that are not strictly positive are silently replaced by one.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_set_scale_factor(
        paint: *mut hb_raster_paint_t,
        x_scale_factor: c_float,
        y_scale_factor: c_float,
    );

    /// Fetches the current post-transform minification factors. Both output
    /// pointers may be null.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_get_scale_factor(
        paint: *const hb_raster_paint_t,
        x_scale_factor: *mut c_float,
        y_scale_factor: *mut c_float,
    );

    /// Sets the output image extents — the pixel rectangle to paint into.
    ///
    /// A zero `stride` is replaced by `width * 4`, the minimum for
    /// [`HB_RASTER_FORMAT_BGRA32`]. Call this, or
    /// [`hb_raster_paint_set_glyph_extents`], before painting;
    /// [`hb_raster_paint_render`] fails without extents.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_set_extents(
        paint: *mut hb_raster_paint_t,
        extents: *const hb_raster_extents_t,
    );

    /// Fetches the currently configured output extents into `extents`, which
    /// may be null.
    ///
    /// Returns true if extents are set, false otherwise.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_get_extents(
        paint: *const hb_raster_paint_t,
        extents: *mut hb_raster_extents_t,
    ) -> hb_bool_t;

    /// Transforms `glyph_extents` with the paint context's base transform and
    /// scale factors, and sets the resulting output image extents.
    ///
    /// Equivalent to computing a transformed bounding box in pixel space and
    /// calling [`hb_raster_paint_set_extents`].
    ///
    /// Returns true if the transformed extents are non-empty and were set;
    /// false otherwise.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_set_glyph_extents(
        paint: *mut hb_raster_paint_t,
        glyph_extents: *const hb_glyph_extents_t,
    ) -> hb_bool_t;

    /// Sets the foreground colour used when paint callbacks request it — an
    /// `is_foreground` colour stop or solid fill, for instance.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_set_foreground(paint: *mut hb_raster_paint_t, foreground: hb_color_t);

    /// Fetches the foreground colour previously set on the paint context, or
    /// the default opaque black if none was set.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_raster_paint_get_foreground(paint: *const hb_raster_paint_t) -> hb_color_t;

    /// Sets the background colour of the paint context.
    ///
    /// If it is not fully transparent, the rendered image is pre-filled with
    /// this colour before glyph content is composited on top. The default is
    /// transparent.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_raster_paint_set_background(paint: *mut hb_raster_paint_t, background: hb_color_t);

    /// Fetches the background colour previously set on the paint context, or
    /// transparent if none was set.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_raster_paint_get_background(paint: *const hb_raster_paint_t) -> hb_color_t;

    /// Selects which font palette is used when paint callbacks look up indexed
    /// colours. The default is palette zero.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_raster_paint_set_palette(paint: *mut hb_raster_paint_t, palette: c_uint);

    /// Fetches the palette index previously set on the paint context, or zero
    /// if none was set.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_raster_paint_get_palette(paint: *const hb_raster_paint_t) -> c_uint;

    /// Clears all custom palette colour overrides previously set on the paint
    /// context.
    ///
    /// Afterwards, palette lookups use the selected font palette with no
    /// override entries.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_clear_custom_palette_colors(paint: *mut hb_raster_paint_t);

    /// Overrides one font palette colour entry for subsequent paint operations.
    ///
    /// Overrides are keyed by `color_index` and persist on the paint context
    /// until cleared, or replaced for the same index. They are consulted by
    /// paint operations that resolve `CPAL` entries.
    ///
    /// Returns true if the override was set, false on allocation failure.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_set_custom_palette_color(
        paint: *mut hb_raster_paint_t,
        color_index: c_uint,
        color: hb_color_t,
    ) -> hb_bool_t;

    /// Fetches the [`hb_paint_funcs_t`] that renders colour glyphs into the
    /// paint context.
    ///
    /// Pass `paint` itself as the `paint_data` argument when calling
    /// [`hb_font_paint_glyph`](crate::hb_font_paint_glyph). The returned funcs
    /// object is a shared singleton owned by HarfBuzz; do not destroy it.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_raster_paint_get_funcs(paint: *const hb_raster_paint_t) -> *mut hb_paint_funcs_t;

    /// Paints one glyph into the paint context.
    ///
    /// Unlike [`hb_raster_paint_glyph_or_fail`], glyphs with no colour paint
    /// data fall back to a synthesized foreground-coloured outline, so any
    /// glyph with an outline or bitmap image produces output.
    ///
    /// If no extents have been set, this first tries to derive them from the
    /// glyph's own extents.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_raster_paint_glyph(
        paint: *mut hb_raster_paint_t,
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
    );

    /// Paints one glyph into the paint context, reporting failure.
    ///
    /// Equivalent to calling
    /// [`hb_font_paint_glyph_or_fail`](crate::hb_font_paint_glyph_or_fail) with
    /// [`hb_raster_paint_get_funcs`], `paint` as the paint data, and the
    /// context's current palette index and foreground colour.
    ///
    /// If no extents have been set, this first tries to derive them from the
    /// glyph's own extents.
    ///
    /// Returns true if painting succeeded, false otherwise.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_raster_paint_glyph_or_fail(
        paint: *mut hb_raster_paint_t,
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
    ) -> hb_bool_t;

    /// Extracts the rendered image after painting has completed.
    ///
    /// The paint context's surface stack is consumed and returned as a new
    /// [`hb_raster_image_t`]. The output format is always
    /// [`HB_RASTER_FORMAT_BGRA32`]. Internal drawing state is cleared here, so
    /// the same context can be reused without any client-side clearing.
    ///
    /// Extents must have been set — with [`hb_raster_paint_set_extents`] or
    /// [`hb_raster_paint_set_glyph_extents`] — before painting.
    ///
    /// Returns the rendered image, or null if extents were not set or if
    /// allocation or configuration failed; if extents were set but nothing was
    /// painted, it returns an empty image. Release it with
    /// [`hb_raster_image_destroy`], or hand it back with
    /// [`hb_raster_paint_recycle_image`].
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_render(paint: *mut hb_raster_paint_t) -> *mut hb_raster_image_t;

    /// Discards accumulated paint output so the context can be reused for
    /// another render.
    ///
    /// User configuration — the base transform, scale factors, foreground
    /// colour, and custom palette colours — is preserved. Use
    /// [`hb_raster_paint_reset`] to also reset that to defaults.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_raster_paint_clear(paint: *mut hb_raster_paint_t);

    /// Resets the paint context to its initial state, clearing all
    /// configuration while preserving its internal image caches.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_reset(paint: *mut hb_raster_paint_t);

    /// Recycles `image` for reuse by subsequent render calls.
    ///
    /// The caller transfers ownership of `image` to `paint` and must not use it
    /// afterwards.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_raster_paint_recycle_image(
        paint: *mut hb_raster_paint_t,
        image: *mut hb_raster_image_t,
    );
}
