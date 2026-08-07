//! GPU text rendering — `hb-gpu.h`.
//!
//! This module binds HarfBuzz's optional `harfbuzz-gpu` sub-library, which
//! renders text on the GPU with no intermediate bitmap atlas. Instead of
//! rasterising glyphs on the CPU and uploading pixels, you encode each glyph
//! into a compact blob of `RGBA16I` texels, upload those texels into a single
//! shared buffer texture, and let a fragment shader — whose source HarfBuzz
//! also hands you — decode and rasterise the glyph directly.
//!
//! Two renderers share one atlas, one vertex stage, and one pipeline:
//!
//! * [`hb_gpu_draw_t`] — an antialiased monochrome coverage mask for outline
//!   glyphs, implementing Eric Lengyel's Slug algorithm. The fragment shader
//!   returns coverage in `[0, 1]`, ready to composite over any background or
//!   multiply with any colour.
//! * [`hb_gpu_paint_t`] — a premultiplied-RGBA accumulator for COLRv0 and
//!   COLRv1 colour glyphs. It also handles plain monochrome outlines, as a
//!   single foreground-coloured layer, at some cost over the draw path.
//!
//! Applications usually pick one renderer per font, based on whether the face
//! has a colour paint table — see
//! [`hb_ot_color_has_paint`](crate::hb_ot_color_has_paint). The draw path is
//! meaningfully faster on monochrome fonts, so using paint unconditionally
//! leaves performance on the table.
//!
//! Both encoders follow the same rhythm: create once, feed one glyph, call
//! `encode` to get a blob, upload the blob, hand the blob back with
//! `recycle_blob`, repeat. Encoding auto-clears the accumulated geometry, so
//! the encoder is immediately ready for the next glyph; user configuration —
//! font scale, palette, custom palette colours — survives across encodes.
//!
//! Shader sources come from [`hb_gpu_shader_source`] plus the renderer-specific
//! [`hb_gpu_draw_shader_source`] or [`hb_gpu_paint_shader_source`], in GLSL,
//! WGSL, MSL, or HLSL. Concatenate them in order with your own `main()`.
//!
//! This module is compiled only when the crate's `gpu` feature is enabled,
//! which is also what makes `build.rs` compile the upstream `hb-gpu*` sources.
//! It is exposed as `harfbuzz_sys::gpu` and is *not* glob re-exported at the
//! crate root.

use core::ffi::{c_char, c_int, c_uint, c_void};

use crate::{
    hb_blob_t, hb_bool_t, hb_codepoint_t, hb_color_t, hb_destroy_func_t, hb_draw_funcs_t,
    hb_font_t, hb_glyph_extents_t, hb_paint_funcs_t, hb_user_data_key_t,
};

/// Shader language variant.
///
/// Selects which flavour of shader source [`hb_gpu_shader_source`] and friends
/// return. All four languages expose the same helper functions with the same
/// names and semantics.
///
/// The C enumeration has no explicit sentinel and its largest enumerator is 4,
/// so it fits in an `int`.
///
/// Since HarfBuzz 14.0.0.
pub type hb_gpu_shader_lang_t = c_int;

/// Sentinel for an invalid or unspecified language.
pub const HB_GPU_SHADER_LANG_INVALID: hb_gpu_shader_lang_t = 0;

/// GLSL — OpenGL 3.3, OpenGL ES 3.0, WebGL 2.0.
pub const HB_GPU_SHADER_LANG_GLSL: hb_gpu_shader_lang_t = 1;

/// WGSL — WebGPU.
pub const HB_GPU_SHADER_LANG_WGSL: hb_gpu_shader_lang_t = 2;

/// MSL — Metal.
pub const HB_GPU_SHADER_LANG_MSL: hb_gpu_shader_lang_t = 3;

/// HLSL — Direct3D.
pub const HB_GPU_SHADER_LANG_HLSL: hb_gpu_shader_lang_t = 4;

/// Shader pipeline stage.
///
/// The C enumeration has no explicit sentinel and its largest enumerator is 1,
/// so it fits in an `int`.
///
/// Since HarfBuzz 14.2.0.
pub type hb_gpu_shader_stage_t = c_int;

/// Vertex shader stage.
pub const HB_GPU_SHADER_STAGE_VERTEX: hb_gpu_shader_stage_t = 0;

/// Fragment shader stage.
pub const HB_GPU_SHADER_STAGE_FRAGMENT: hb_gpu_shader_stage_t = 1;

crate::opaque_handle! {
    /// An opaque GPU shape encoder.
    ///
    /// Accumulates outlines via draw callbacks, then encodes them into a
    /// compact blob for GPU rendering.
    ///
    /// Since HarfBuzz 14.0.0.
    hb_gpu_draw_t
}

crate::opaque_handle! {
    /// An opaque GPU color-glyph encoder.
    ///
    /// Accumulates color-glyph paint state via paint callbacks, then encodes
    /// it into a compact blob for GPU rendering.
    ///
    /// Since HarfBuzz 14.2.0.
    hb_gpu_paint_t
}

unsafe extern "C" {
    /// Returns the shared helper shader source used by both hb-gpu renderers
    /// for the given stage and language.
    ///
    /// The shared source defines the atlas sampler and the `hb_gpu_fetch()`
    /// accessor for the fragment stage, and the `hb_gpu_dilate()` helper for
    /// the vertex stage. Each renderer-specific source — see
    /// [`hb_gpu_draw_shader_source`] and [`hb_gpu_paint_shader_source`] —
    /// assumes these helpers are already in scope, so assemble a shader as a
    /// `#version` directive, then this source, then the renderer-specific
    /// source, then your own `main()`.
    ///
    /// Returns a static string that must not be freed, or null if `stage` or
    /// `lang` is unsupported.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_shader_source(
        stage: hb_gpu_shader_stage_t,
        lang: hb_gpu_shader_lang_t,
    ) -> *const c_char;

    /// Returns the draw-renderer-specific shader source for the given stage
    /// and language.
    ///
    /// This source assumes the shared helpers from [`hb_gpu_shader_source`]
    /// are concatenated ahead of it. The vertex stage currently has no
    /// draw-specific helpers, so the empty string is returned for it and
    /// callers can concatenate unconditionally.
    ///
    /// Returns a static string that must not be freed, or null if `stage` or
    /// `lang` is unsupported.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_draw_shader_source(
        stage: hb_gpu_shader_stage_t,
        lang: hb_gpu_shader_lang_t,
    ) -> *const c_char;

    /// Creates a new GPU shape encoder.
    ///
    /// Returns a newly allocated encoder, or null on allocation failure.
    /// Release it with [`hb_gpu_draw_destroy`].
    ///
    /// Since HarfBuzz 14.0.0.
    pub fn hb_gpu_draw_create_or_fail() -> *mut hb_gpu_draw_t;

    /// Increases the reference count on `draw` by one and returns it.
    ///
    /// Since HarfBuzz 14.0.0.
    pub fn hb_gpu_draw_reference(draw: *mut hb_gpu_draw_t) -> *mut hb_gpu_draw_t;

    /// Decreases the reference count on `draw` by one, freeing the encoder
    /// when the count reaches zero.
    ///
    /// Since HarfBuzz 14.0.0.
    pub fn hb_gpu_draw_destroy(draw: *mut hb_gpu_draw_t);

    /// Attaches a user-data key/data pair to the encoder.
    ///
    /// `destroy` — which may be null — is called with `data` when the encoder
    /// is destroyed or the value is replaced. `replace` decides whether
    /// existing data stored under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 14.0.0.
    pub fn hb_gpu_draw_set_user_data(
        draw: *mut hb_gpu_draw_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to the encoder under the specified key.
    ///
    /// Ownership stays with the encoder; the caller must not free the returned
    /// pointer.
    ///
    /// Since HarfBuzz 14.0.0.
    pub fn hb_gpu_draw_get_user_data(
        draw: *const hb_gpu_draw_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Sets the font scale, so the encoded blob can embed it for shader use —
    /// computing pixels-per-em, for instance.
    ///
    /// [`hb_gpu_draw_glyph`] and [`hb_gpu_draw_glyph_or_fail`] call this
    /// automatically with the scale of the font they were given.
    ///
    /// Since HarfBuzz 14.1.0.
    pub fn hb_gpu_draw_set_scale(draw: *mut hb_gpu_draw_t, x_scale: c_int, y_scale: c_int);

    /// Fetches the font scale previously set by [`hb_gpu_draw_set_scale`] or
    /// [`hb_gpu_draw_glyph`].
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_draw_get_scale(
        draw: *const hb_gpu_draw_t,
        x_scale: *mut c_int,
        y_scale: *mut c_int,
    );

    /// Fetches the draw callbacks that feed outline data into `draw`.
    ///
    /// Pass the encoder itself as the `draw_data` argument when calling the
    /// draw functions. Ownership stays with HarfBuzz; do not destroy the
    /// returned callbacks.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_draw_get_funcs(draw: *const hb_gpu_draw_t) -> *mut hb_draw_funcs_t;

    /// Draws a single glyph outline into the encoder.
    ///
    /// Equivalent to [`hb_gpu_draw_glyph_or_fail`] with the return value
    /// ignored.
    ///
    /// Since HarfBuzz 14.0.0.
    pub fn hb_gpu_draw_glyph(draw: *mut hb_gpu_draw_t, font: *mut hb_font_t, glyph: hb_codepoint_t);

    /// Draws a single glyph outline into the encoder, reporting whether the
    /// font had an outline for it.
    ///
    /// Copies the font's scale onto the encoder, then draws `glyph` through
    /// [`hb_gpu_draw_get_funcs`].
    ///
    /// Returns true if the glyph was drawn, false if the font has no outlines
    /// for `glyph`.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_draw_glyph_or_fail(
        draw: *mut hb_gpu_draw_t,
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
    ) -> hb_bool_t;

    /// Encodes the accumulated outlines into a compact blob suitable for GPU
    /// rendering.
    ///
    /// The blob data is an array of `RGBA16I` texels — eight bytes each — to be
    /// uploaded to a texture buffer object. The blob owns its own copy of the
    /// data.
    ///
    /// `extents`, which may be null, receives the computed glyph extents in
    /// font units, Y-up.
    ///
    /// On return the encoder is auto-cleared so it can be reused for the next
    /// glyph; user configuration — the font scale — is preserved.
    ///
    /// Returns a blob containing the encoded data, or null if encoding failed
    /// through allocation failure or an accumulation error. When the encoder
    /// accumulated no outline — a space glyph, say — the empty-blob singleton
    /// is returned instead of null, so callers can tell "nothing to render"
    /// (length zero) from a real failure (null). Release the blob with
    /// [`hb_gpu_draw_recycle_blob`] or
    /// [`hb_blob_destroy`](crate::hb_blob_destroy).
    ///
    /// Since HarfBuzz 14.0.0.
    pub fn hb_gpu_draw_encode(
        draw: *mut hb_gpu_draw_t,
        extents: *mut hb_glyph_extents_t,
    ) -> *mut hb_blob_t;

    /// Discards accumulated outlines so the encoder can be reused for another
    /// encode.
    ///
    /// User configuration — the font scale — is preserved. Use
    /// [`hb_gpu_draw_reset`] to also restore configuration to its defaults.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_draw_clear(draw: *mut hb_gpu_draw_t);

    /// Resets the encoder, discarding all accumulated outlines and restoring
    /// user configuration to its defaults.
    ///
    /// Since HarfBuzz 14.0.0.
    pub fn hb_gpu_draw_reset(draw: *mut hb_gpu_draw_t);

    /// Returns a blob previously produced by [`hb_gpu_draw_encode`] to the
    /// encoder for potential reuse, transferring ownership of it.
    ///
    /// A future version may reclaim the underlying buffer to avoid an
    /// allocation on the next encode.
    ///
    /// Since HarfBuzz 14.0.0.
    pub fn hb_gpu_draw_recycle_blob(draw: *mut hb_gpu_draw_t, blob: *mut hb_blob_t);

    /// Creates a new GPU color-glyph paint encoder.
    ///
    /// Returns a newly allocated encoder, or null on allocation failure.
    /// Release it with [`hb_gpu_paint_destroy`].
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_create_or_fail() -> *mut hb_gpu_paint_t;

    /// Increases the reference count on `paint` by one and returns it.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_reference(paint: *mut hb_gpu_paint_t) -> *mut hb_gpu_paint_t;

    /// Decreases the reference count on `paint` by one, freeing the encoder
    /// when the count reaches zero.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_destroy(paint: *mut hb_gpu_paint_t);

    /// Attaches a user-data key/data pair to the encoder.
    ///
    /// `destroy` — which may be null — is called with `data` when the encoder
    /// is destroyed or the value is replaced. `replace` decides whether
    /// existing data stored under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_set_user_data(
        paint: *mut hb_gpu_paint_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to the encoder under the specified key.
    ///
    /// Ownership stays with the encoder; the caller must not free the returned
    /// pointer.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_get_user_data(
        paint: *const hb_gpu_paint_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Fetches the paint callbacks that feed paint data into `paint`.
    ///
    /// Pass the encoder itself as the `paint_data` argument when calling the
    /// paint functions. Ownership stays with HarfBuzz; do not destroy the
    /// returned callbacks.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_get_funcs(paint: *const hb_gpu_paint_t) -> *mut hb_paint_funcs_t;

    /// Selects which font palette is used when paint callbacks look up indexed
    /// colours. The default is palette zero.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_set_palette(paint: *mut hb_gpu_paint_t, palette: c_uint);

    /// Returns the palette index previously set on the encoder, or zero if none
    /// was set.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_get_palette(paint: *const hb_gpu_paint_t) -> c_uint;

    /// Clears all custom palette colour overrides previously set on the
    /// encoder.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_clear_custom_palette_colors(paint: *mut hb_gpu_paint_t);

    /// Overrides one font palette colour entry.
    ///
    /// Overrides are keyed by `color_index` and persist on the encoder until
    /// cleared, or replaced for the same index. Their values are baked into
    /// the blob at encode time, so changing them afterwards requires
    /// re-encoding the affected glyphs.
    ///
    /// Returns true if the override was set, false on allocation failure.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_set_custom_palette_color(
        paint: *mut hb_gpu_paint_t,
        color_index: c_uint,
        color: hb_color_t,
    ) -> hb_bool_t;

    /// Sets the font scale used to dimension clip-glyph outlines inside the
    /// encoded blob.
    ///
    /// [`hb_gpu_paint_glyph`] and [`hb_gpu_paint_glyph_or_fail`] call this
    /// automatically with the scale of the font they were given.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_set_scale(paint: *mut hb_gpu_paint_t, x_scale: c_int, y_scale: c_int);

    /// Fetches the font scale previously set on the encoder.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_get_scale(
        paint: *const hb_gpu_paint_t,
        x_scale: *mut c_int,
        y_scale: *mut c_int,
    );

    /// Feeds a glyph's paint tree into the encoder.
    ///
    /// Unlike [`hb_gpu_paint_glyph_or_fail`], non-colour glyphs are handled
    /// transparently by synthesising a single foreground-coloured layer from
    /// the glyph's outline, so any glyph with an outline produces output.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_glyph(
        paint: *mut hb_gpu_paint_t,
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
    );

    /// Feeds a glyph's paint tree into the encoder, reporting whether the font
    /// had paint data for it.
    ///
    /// Copies the font's scale onto the encoder, then paints `glyph` through
    /// [`hb_gpu_paint_get_funcs`] with the encoder's palette.
    ///
    /// Encoder-level limitations — unsupported paint operations, group-stack
    /// overflow — do *not* fail here; they surface later as a null return from
    /// [`hb_gpu_paint_encode`].
    ///
    /// Returns true if the font had paint data for `glyph`, false otherwise.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_glyph_or_fail(
        paint: *mut hb_gpu_paint_t,
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
    ) -> hb_bool_t;

    /// Encodes the accumulated paint state into a GPU-renderable blob and
    /// writes the glyph's ink extents to `extents`.
    ///
    /// The blob's texel format is the same `RGBA16I` as the draw renderer's, so
    /// draw and paint blobs can coexist in one atlas at different offsets.
    ///
    /// On return the encoder is auto-cleared so it can be reused for the next
    /// glyph; user configuration — palette and custom palette overrides — is
    /// preserved.
    ///
    /// Returns a newly allocated blob, or null if the paint walk hit an
    /// unsupported feature or encoding failed through allocation failure. When
    /// the paint accumulated no ink — a space glyph, say — the empty-blob
    /// singleton is returned, so callers can tell "nothing to render" (length
    /// zero) from a real failure (null). Release the blob with
    /// [`hb_gpu_paint_recycle_blob`] or
    /// [`hb_blob_destroy`](crate::hb_blob_destroy).
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_encode(
        paint: *mut hb_gpu_paint_t,
        extents: *mut hb_glyph_extents_t,
    ) -> *mut hb_blob_t;

    /// Discards accumulated paint state so the encoder can be reused for
    /// another encode. User configuration is preserved.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_clear(paint: *mut hb_gpu_paint_t);

    /// Resets the encoder, discarding accumulated state and restoring user
    /// configuration to its defaults.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_reset(paint: *mut hb_gpu_paint_t);

    /// Returns a blob previously produced by [`hb_gpu_paint_encode`] to the
    /// encoder for potential reuse, transferring ownership of it.
    ///
    /// The underlying buffer is reused by the next call to
    /// [`hb_gpu_paint_encode`], avoiding a malloc and a blob allocation per
    /// glyph.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_recycle_blob(paint: *mut hb_gpu_paint_t, blob: *mut hb_blob_t);

    /// Returns the paint-renderer-specific shader source for the given stage
    /// and language.
    ///
    /// This source assumes that both the shared helpers from
    /// [`hb_gpu_shader_source`] *and* the draw-renderer helpers from
    /// [`hb_gpu_draw_shader_source`] are concatenated ahead of it — the paint
    /// interpreter calls `hb_gpu_draw()` to compute clip-glyph coverage. Full
    /// assembly is `#version` directive, shared source, draw source, paint
    /// source, your `main()`.
    ///
    /// Returns a static string that must not be freed, or null if `stage` or
    /// `lang` is unsupported.
    ///
    /// Since HarfBuzz 14.2.0.
    pub fn hb_gpu_paint_shader_source(
        stage: hb_gpu_shader_stage_t,
        lang: hb_gpu_shader_lang_t,
    ) -> *const c_char;
}
