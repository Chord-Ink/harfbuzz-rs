# GPU outlines

Header: `hb-gpu.h` — Rust module: `harfbuzz_sys::gpu`, behind the crate's `gpu`
feature. Unlike most modules this one is **not** glob re-exported at the crate
root: names are reached as `harfbuzz_sys::gpu::NAME`.

Upstream gtk-doc section: `hb-gpu`, titled *hb-gpu*, short description *GPU text
rendering*.

## Overview

`hb-gpu` is an optional HarfBuzz sub-library that renders text **on the GPU**,
with no intermediate bitmap atlas. The traditional pipeline rasterises each
glyph on the CPU at a particular size, uploads the pixels into a texture atlas,
and re-rasterises whenever the size, the transform, or the DPI changes.
`hb-gpu` inverts that: the CPU encodes a glyph's *geometry* once, into a compact
resolution-independent blob, and the fragment shader evaluates coverage
analytically at whatever size the fragment happens to land on. One upload
serves every size, every rotation, every zoom level.

The encoded blob is an array of `RGBA16I` texels — eight bytes each — that you
upload into a single shared buffer texture (the "atlas"). Every glyph occupies
a contiguous run of texels at some offset you choose; the shader is handed that
offset as a flat varying and decodes from there. Because the CPU side is pure
data production, `hb-gpu` never touches your graphics API: it hands you bytes
and shader source strings, and you do the binding.

There are **two renderers**, sharing one atlas, one vertex stage, and one
pipeline:

* **Draw** — `hb_gpu_draw_t`. An antialiased monochrome coverage mask for
  outline glyphs, implementing Eric Lengyel's
  [Slug](https://github.com/EricLengyel/Slug) algorithm. Its fragment helper
  returns a single `float` coverage in `[0, 1]`, ready to composite over any
  background or multiply with any colour.
* **Paint** — `hb_gpu_paint_t`. A premultiplied-RGBA accumulator that walks the
  font's paint tree — COLRv0 layers or a COLRv1 paint subtree — and records one
  layer operation per solid or gradient fill, each with its own clip outline
  encoded as a Slug. (The paint renderer therefore reuses the draw renderer
  internally, for clips.) Paint also handles plain monochrome outlines, by
  synthesising a single foreground-coloured layer, so it *can* be used for any
  font — at some cost over the draw path.

Applications typically pick one renderer per font, based on whether the face has
a colour paint table — `hb_ot_color_has_paint()` from `hb-ot-color.h`. The draw
path is meaningfully faster on monochrome fonts, so using paint unconditionally
leaves performance on the table.

Both encoders are **reference-counted opaque objects with identical rhythm**:

1. create once, up front (`hb_gpu_draw_create_or_fail` /
   `hb_gpu_paint_create_or_fail`);
2. optionally configure it (font scale; for paint, palette and custom palette
   colours);
3. feed exactly one glyph's geometry into it (`…_glyph` / `…_glyph_or_fail`, or
   drive the callbacks yourself);
4. call `…_encode` to get an `hb_blob_t`, which also hands back the glyph's ink
   extents;
5. upload the blob's bytes into your atlas at an offset you record;
6. give the blob back with `…_recycle_blob`;
7. go to step 3 for the next glyph.

`…_encode` **auto-clears** the accumulated geometry on return — including on the
failure paths — so the encoder is immediately ready for the next glyph. User
configuration (font scale; palette and custom-palette overrides) survives
clearing; only `…_reset` wipes it.

The shader half of the library is delivered as source strings, in four
languages — GLSL, WGSL, MSL, HLSL — through three functions:
`hb_gpu_shader_source` (shared helpers), `hb_gpu_draw_shader_source`, and
`hb_gpu_paint_shader_source`. You concatenate a `#version` directive, the shared
source, the renderer source(s), and your own `main()`, then compile. Everything
these strings define is documented under [The shader side](#the-shader-side)
below, because you cannot use this header without it.

The whole module is compiled only when the crate's `gpu` feature is on; the same
feature is what makes `build.rs` compile the upstream `hb-gpu*` sources, so a
declaration here can never refer to a symbol that was left out of the archive.
Upstream ships this as a separate `harfbuzz-gpu` library with its own
`harfbuzz-gpu.pc`. It is new and marked experimental in spirit — the `Since`
versions below are 14.0.0 and 14.2.0.

There is a [live web demo](https://harfbuzz.github.io/hb-gpu-demo/).

### Coordinate space and quantisation

The encoder works in **font design units**, Y-up, exactly as `hb_draw_funcs_t`
delivers them. The blob format quantises coordinates to 16 bits at a fixed scale
of four steps per unit (`HB_GPU_UNITS_PER_EM 4` in both the C and the shader
source), so:

* the usable coordinate range is roughly **±8000 units**, and
* the effective precision is **~0.25 units**.

Choose a coordinate scale where (a) the overall bounding box stays inside
±8000, and (b) your smallest feature — stroke width, fine detail — is at least
1–2 units. Font-unit coordinates (a 1000- or 2048-unit em) satisfy both
comfortably. Tightly normalised coordinates (a 0..1 unit square) do not: a
one-pixel stroke in that space quantises to zero and vanishes. Reach larger
on-screen sizes by scaling the rendered quad or the vertex transform, not by
scaling the encoded geometry.

For the same reason, upstream recommends **not** setting a scale on the
`hb_font_t` you hand to `hb_gpu_draw_glyph()`: leave it at the default (upem) and
apply `font_size / upem` when you compute vertex positions. If you do set a font
scale — for hinting, say — the blob and the extents come back in that scaled
space, and the shader's `emPerPos` must be adjusted to match.

## Types

### `hb_gpu_shader_lang_t`

Shader language variant. Chooses which flavour of source the three
`*_shader_source` functions return. All four languages expose the same helper
functions, with the same names and the same semantics.

```c
typedef enum {
  HB_GPU_SHADER_LANG_INVALID,
  HB_GPU_SHADER_LANG_GLSL,
  HB_GPU_SHADER_LANG_WGSL,
  HB_GPU_SHADER_LANG_MSL,
  HB_GPU_SHADER_LANG_HLSL,
} hb_gpu_shader_lang_t;
```

```rust
pub type hb_gpu_shader_lang_t = c_int;
pub const HB_GPU_SHADER_LANG_INVALID: hb_gpu_shader_lang_t = 0;
pub const HB_GPU_SHADER_LANG_GLSL: hb_gpu_shader_lang_t = 1;
pub const HB_GPU_SHADER_LANG_WGSL: hb_gpu_shader_lang_t = 2;
pub const HB_GPU_SHADER_LANG_MSL: hb_gpu_shader_lang_t = 3;
pub const HB_GPU_SHADER_LANG_HLSL: hb_gpu_shader_lang_t = 4;
```

| Constant | Value | Meaning |
| --- | --- | --- |
| `HB_GPU_SHADER_LANG_INVALID` | 0 | Sentinel for an invalid or unspecified language. Every `*_shader_source` function returns `NULL` for it. |
| `HB_GPU_SHADER_LANG_GLSL` | 1 | GLSL — OpenGL 3.3, OpenGL ES 3.0, WebGL 2.0. The shared source states it requires GLSL 3.30 or GLSL ES 3.00. |
| `HB_GPU_SHADER_LANG_WGSL` | 2 | WGSL — WebGPU. |
| `HB_GPU_SHADER_LANG_MSL` | 3 | MSL — Metal. |
| `HB_GPU_SHADER_LANG_HLSL` | 4 | HLSL — Direct3D. |

Underlying type: the C enumeration has no `_MAX_VALUE` sentinel and its largest
enumerator is 4, so it fits in `int` — hence `c_int`.

Since HarfBuzz 14.0.0.

### `hb_gpu_shader_stage_t`

Shader pipeline stage.

```c
typedef enum {
  HB_GPU_SHADER_STAGE_VERTEX,
  HB_GPU_SHADER_STAGE_FRAGMENT,
} hb_gpu_shader_stage_t;
```

```rust
pub type hb_gpu_shader_stage_t = c_int;
pub const HB_GPU_SHADER_STAGE_VERTEX: hb_gpu_shader_stage_t = 0;
pub const HB_GPU_SHADER_STAGE_FRAGMENT: hb_gpu_shader_stage_t = 1;
```

| Constant | Value | Meaning |
| --- | --- | --- |
| `HB_GPU_SHADER_STAGE_VERTEX` | 0 | Vertex shader stage. The shared source supplies `hb_gpu_dilate()`; both renderer-specific sources are empty here. |
| `HB_GPU_SHADER_STAGE_FRAGMENT` | 1 | Fragment shader stage. This is where the atlas sampler and all the decoding lives. |

Underlying type: the C enumeration has no `_MAX_VALUE` sentinel and its largest
enumerator is 1, so it fits in `int` — hence `c_int`.

Since HarfBuzz 14.2.0.

### `hb_gpu_draw_t`

An opaque GPU shape encoder. Accumulates outlines via draw callbacks, then
encodes them into a compact blob for GPU rendering.

Reference-counted, created only by `hb_gpu_draw_create_or_fail()`, shared with
`hb_gpu_draw_reference()`, released with `hb_gpu_draw_destroy()`. There is no
"empty"/nil singleton accessor for this type, and no `create()` that substitutes
one — the only constructor is the `_or_fail` form, so you must check for `NULL`.

State it holds:

* the accumulated curve list and its running bounding box (cleared by
  `hb_gpu_draw_clear()` and by every `hb_gpu_draw_encode()`);
* an internal success flag, set false if accumulation ran out of memory;
* the font scale, `x_scale`/`y_scale`, defaulting to `0`/`0` and preserved
  across clears;
* internal encode scratch buffers, reused across encodes;
* at most one recycled blob handed back through
  `hb_gpu_draw_recycle_blob()`, destroyed with the encoder.

In Rust it is an `opaque_handle!` type — zero-sized, non-constructible, `!Send`
and `!Sync`, used only behind `*mut hb_gpu_draw_t`.

Since HarfBuzz 14.0.0.

### `hb_gpu_paint_t`

An opaque GPU color-glyph encoder. Accumulates color-glyph paint state via paint
callbacks, then encodes it into a compact blob for GPU rendering.

Same lifecycle shape as `hb_gpu_draw_t`: reference-counted, only
`hb_gpu_paint_create_or_fail()` builds one, no nil singleton.

State it holds:

* an operation stream (solid layer, gradient layer, push group, pop group) plus
  the sub-blobs holding each layer's clip outline, cleared by
  `hb_gpu_paint_clear()` and by every `hb_gpu_paint_encode()`;
* an `unsupported` flag, raised when the paint walk exceeds an encoder limit;
* the transform stack and current transform;
* the running ink bounding box;
* **user configuration**, preserved across clears: the palette index (default
  `0`), the custom-palette override map (default empty), and the font scale
  (default `0`/`0`);
* at most one recycled blob.

Encoder limits, read from `hb-gpu-paint.cc`; exceeding any of them raises
`unsupported` and makes `hb_gpu_paint_encode()` return `NULL` rather than emit a
blob that would render incorrectly:

| Limit | Value | What it bounds |
| --- | --- | --- |
| `HB_GPU_PAINT_MAX_OPS` | 0x7fff | Total ops in the stream (`num_ops` is an `int16` in the blob header). |
| `HB_GPU_PAINT_MAX_GROUP_DEPTH` | 4 | Nesting depth of push/pop group; matches the shader's group stack. |
| `HB_GPU_PAINT_MAX_CLIP_DEPTH` | 3 | Clip outlines the shader can intersect per layer. |
| `HB_GPU_PAINT_MAX_SUB_BLOBS` | 1024 | Clip-glyph rasterisations per paint walk. |

None of these four macros is public API — they are not in the header and not
transcribed into Rust. They are listed here because they are the *only*
explanation for a `NULL` from `hb_gpu_paint_encode()` that is not an allocation
failure.

In Rust it is an `opaque_handle!` type, used only behind `*mut hb_gpu_paint_t`.

Since HarfBuzz 14.2.0.

### Types from other headers used here

`hb-gpu.h` includes `hb.h` and declares no other types of its own. The following
appear in its signatures and are transcribed elsewhere in this crate. All of
them are glob re-exported at the crate root, so import them as
`harfbuzz_sys::NAME` — not from `harfbuzz_sys::gpu`, which re-exports nothing.

| Type | Declared in | Transcribed in | Role here |
| --- | --- | --- | --- |
| `hb_blob_t` | `hb-blob.h` | `blob.rs` | The encoded output. |
| `hb_bool_t` | `hb-common.h` | `common.rs` | `c_int`; zero false, non-zero true. |
| `hb_codepoint_t` | `hb-common.h` | `common.rs` | `u32`; here always a **glyph ID**, not a Unicode scalar. |
| `hb_color_t` | `hb-common.h` | `common.rs` | `u32` BGRA, built with `HB_COLOR(b, g, r, a)`. |
| `hb_destroy_func_t` | `hb-common.h` | `common.rs` | `Option<unsafe extern "C" fn(*mut c_void)>`. |
| `hb_draw_funcs_t` | `hb-draw.h` | `draw.rs` | The pen `hb_gpu_draw_get_funcs()` returns. |
| `hb_font_t` | `hb-font.h` | `font.rs` | Source of glyph outlines and paint trees. |
| `hb_glyph_extents_t` | `hb-common.h` | `common.rs` | Out-parameter of both `encode` functions. |
| `hb_paint_funcs_t` | `hb-paint.h` | `paint.rs` | The paint callbacks `hb_gpu_paint_get_funcs()` returns. |
| `hb_user_data_key_t` | `hb-common.h` | `common.rs` | Address-keyed user-data slot. |

## Functions

Thirty-nine symbols in the gtk-doc section: two enumerations, two opaque types,
and thirty-five functions. The functions divide into three shader-source getters,
fourteen draw-encoder functions, and eighteen paint-encoder functions — the two
encoder sets are deliberately parallel, with paint adding the four
palette-control calls.

### Shader sources

#### `hb_gpu_shader_source`

```c
const char *hb_gpu_shader_source (hb_gpu_shader_stage_t stage,
                                  hb_gpu_shader_lang_t  lang);
```

```rust
pub fn hb_gpu_shader_source(
    stage: hb_gpu_shader_stage_t,
    lang: hb_gpu_shader_lang_t,
) -> *const c_char;
```

Returns the **shared helper** shader source used by both renderers, for the
given stage and language.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `stage` | `HB_GPU_SHADER_STAGE_VERTEX` or `HB_GPU_SHADER_STAGE_FRAGMENT`. Any other value yields `NULL`. |
| `lang` | One of the four real languages. `HB_GPU_SHADER_LANG_INVALID`, or any unrecognised value, yields `NULL`. |

**Returns** — a NUL-terminated shader source string, or `NULL` if `stage` or
`lang` is unsupported. The fragment string defines `HB_GPU_UNITS_PER_EM`, the
`hb_gpu_atlas` sampler, the `hb_gpu_fetch()` accessor, `hb_gpu_ppem()`, and
`hb_gpu_stem_darken()`. The vertex string defines `hb_gpu_dilate()`.

**Ownership** — the string is static, lives for the life of the process, and
must **not** be freed. Nothing is copied and nothing is borrowed from you.

**Notes** — Since HarfBuzz 14.2.0. Pure function of its two arguments; safe to
call from any thread at any time. The renderer-specific sources assume these
helpers are already in scope, so this string must come *first* in the
concatenation, after the `#version` directive.

#### `hb_gpu_draw_shader_source`

```c
const char *hb_gpu_draw_shader_source (hb_gpu_shader_stage_t stage,
                                       hb_gpu_shader_lang_t  lang);
```

```rust
pub fn hb_gpu_draw_shader_source(
    stage: hb_gpu_shader_stage_t,
    lang: hb_gpu_shader_lang_t,
) -> *const c_char;
```

Returns the **draw-renderer-specific** shader source. The fragment string
defines `hb_gpu_draw()`.

**Parameters** — as `hb_gpu_shader_source`.

**Returns** — a shader source string, or `NULL` if `stage` or `lang` is
unsupported. For `HB_GPU_SHADER_STAGE_VERTEX` with a valid language it returns
the **empty string** `""` — not `NULL` — precisely so callers can concatenate
unconditionally without branching on stage. (`HB_GPU_SHADER_LANG_INVALID` still
gives `NULL`, even for the vertex stage.)

**Ownership** — static string; do not free.

**Notes** — Since HarfBuzz 14.2.0. Assumes `hb_gpu_shader_source()`'s output is
concatenated ahead of it. Full assembly: `#version` + shared + draw + your
`main()`.

#### `hb_gpu_paint_shader_source`

```c
const char *hb_gpu_paint_shader_source (hb_gpu_shader_stage_t stage,
                                        hb_gpu_shader_lang_t  lang);
```

```rust
pub fn hb_gpu_paint_shader_source(
    stage: hb_gpu_shader_stage_t,
    lang: hb_gpu_shader_lang_t,
) -> *const c_char;
```

Returns the **paint-renderer-specific** shader source. The fragment string
defines `hb_gpu_paint()` and the gradient evaluators it needs.

**Parameters** — as `hb_gpu_shader_source`.

**Returns** — a shader source string, or `NULL` if `stage` or `lang` is
unsupported; the empty string for the vertex stage with a valid language.

**Ownership** — static string; do not free.

**Notes** — Since HarfBuzz 14.2.0. This source assumes **both** the shared
helpers *and* the draw-renderer helpers are concatenated ahead of it, because
the paint interpreter calls `hb_gpu_draw()` to compute clip-glyph coverage. Full
assembly: `#version` + `hb_gpu_shader_source()` + `hb_gpu_draw_shader_source()`
+ `hb_gpu_paint_shader_source()` + your `main()`. Omitting the draw source is
the classic paint-path compile error.

### Draw encoder — creation and destruction

#### `hb_gpu_draw_create_or_fail`

```c
hb_gpu_draw_t *hb_gpu_draw_create_or_fail (void);
```

```rust
pub fn hb_gpu_draw_create_or_fail() -> *mut hb_gpu_draw_t;
```

Creates a new GPU shape encoder, with reference count 1, no accumulated
geometry, and a font scale of `0`/`0`.

**Parameters** — none.

**Returns** — a newly allocated `hb_gpu_draw_t`, or `NULL` on allocation
failure. Unlike most HarfBuzz `create` functions there is no nil-object
fallback and no `hb_gpu_draw_get_empty()`, so `NULL` must be checked.

**Ownership** — the caller owns the returned reference and must release it with
`hb_gpu_draw_destroy()`.

**Notes** — Since HarfBuzz 14.0.0. Create the encoder **once** and reuse it
across glyphs; it carries reusable scratch buffers, so per-glyph creation throws
away exactly the allocations it exists to amortise.

#### `hb_gpu_draw_reference`

```c
hb_gpu_draw_t *hb_gpu_draw_reference (hb_gpu_draw_t *draw);
```

```rust
pub fn hb_gpu_draw_reference(draw: *mut hb_gpu_draw_t) -> *mut hb_gpu_draw_t;
```

Increases the reference count on `draw` by one.

**Parameters** — `draw`: the encoder. Nullability is unspecified in the docs;
HarfBuzz's generic `hb_object_reference()` tolerates `NULL` and returns it.

**Returns** — the same pointer, now owning one more reference.

**Ownership** — transfers a new reference to the caller; balance it with
`hb_gpu_draw_destroy()`.

**Notes** — Since HarfBuzz 14.0.0. Reference counting is atomic, so
reference/destroy from several threads is safe even though *using* the encoder
from several threads is not. Marked `(skip)` for language bindings upstream.

#### `hb_gpu_draw_destroy`

```c
void hb_gpu_draw_destroy (hb_gpu_draw_t *draw);
```

```rust
pub fn hb_gpu_draw_destroy(draw: *mut hb_gpu_draw_t);
```

Decreases the reference count on `draw` by one. At zero the encoder is freed,
along with any stashed recycled blob and any user data (whose `destroy`
callbacks fire).

**Parameters** — `draw`: the encoder. `NULL` is tolerated by the underlying
`hb_object_should_destroy()` path.

**Returns** — nothing.

**Ownership** — consumes one reference.

**Notes** — Since HarfBuzz 14.0.0. Marked `(skip)` upstream.

#### `hb_gpu_draw_set_user_data`

```c
hb_bool_t hb_gpu_draw_set_user_data (hb_gpu_draw_t      *draw,
                                     hb_user_data_key_t *key,
                                     void               *data,
                                     hb_destroy_func_t   destroy,
                                     hb_bool_t           replace);
```

```rust
pub fn hb_gpu_draw_set_user_data(
    draw: *mut hb_gpu_draw_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

Attaches a user-data key/data pair to the encoder.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `draw` | The encoder. |
| `key` | The user-data key. HarfBuzz uses its *address*, so it must outlive the encoder — a `static` is the normal choice. |
| `data` | Your pointer. May be null. |
| `destroy` | Called with `data` when the encoder dies or the value is replaced. Nullable (`None` in Rust). |
| `replace` | Whether to overwrite data already stored under the same key. |

**Returns** — true on success, false otherwise (allocation failure, or a
non-`replace` call against an occupied key).

**Ownership** — the encoder does not copy `data`; it stores the pointer and, if
`destroy` is non-null, takes responsibility for calling it exactly once.

**Notes** — Since HarfBuzz 14.0.0. Marked `(skip)` upstream.

#### `hb_gpu_draw_get_user_data`

```c
void *hb_gpu_draw_get_user_data (const hb_gpu_draw_t *draw,
                                 hb_user_data_key_t  *key);
```

```rust
pub fn hb_gpu_draw_get_user_data(
    draw: *const hb_gpu_draw_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

Fetches the user data associated with `key`.

**Parameters** — `draw`: the encoder (const). `key`: the same key address used
when setting.

**Returns** — the stored pointer, or `NULL` if nothing is stored under that key.

**Ownership** — transfer none; the encoder keeps ownership and the caller must
not free the result.

**Notes** — Since HarfBuzz 14.0.0. Marked `(skip)` upstream.

### Draw encoder — configuration

#### `hb_gpu_draw_set_scale`

```c
void hb_gpu_draw_set_scale (hb_gpu_draw_t *draw,
                            int            x_scale,
                            int            y_scale);
```

```rust
pub fn hb_gpu_draw_set_scale(draw: *mut hb_gpu_draw_t, x_scale: c_int, y_scale: c_int);
```

Sets the font scale, so that the encoded blob can embed it for shader use — the
shader's `hb_gpu_ppem()` divides by it to compute pixels-per-em.

**Parameters** — `draw`: the encoder. `x_scale`, `y_scale`: the scale, typically
whatever `hb_font_get_scale()` reports for the font you are about to encode.
Units are the font's scale units; there is no range restriction in the API, and
the default on a fresh encoder is `0`/`0`.

**Returns** — nothing. Cannot fail.

**Ownership** — nothing is transferred.

**Notes** — Since HarfBuzz **14.1.0** (later than the rest of the draw API).
Called automatically by `hb_gpu_draw_glyph()` and
`hb_gpu_draw_glyph_or_fail()`, so you only call it by hand when you feed
outlines through the pen directly. The scale is **user configuration**: it
survives `hb_gpu_draw_clear()` and `hb_gpu_draw_encode()`, and is zeroed only by
`hb_gpu_draw_reset()`.

#### `hb_gpu_draw_get_scale`

```c
void hb_gpu_draw_get_scale (const hb_gpu_draw_t *draw,
                            int                 *x_scale,
                            int                 *y_scale);
```

```rust
pub fn hb_gpu_draw_get_scale(
    draw: *const hb_gpu_draw_t,
    x_scale: *mut c_int,
    y_scale: *mut c_int,
);
```

Gets the font scale previously set with `hb_gpu_draw_set_scale()` or implicitly
by `hb_gpu_draw_glyph()`.

**Parameters** — `draw`: the encoder (const). `x_scale`, `y_scale`: out
parameters. Both are **null-tolerant**: the implementation writes each only if
its pointer is non-null, so you may ask for one axis alone.

**Returns** — nothing.

**Ownership** — nothing is transferred.

**Notes** — Since HarfBuzz 14.2.0.

### Draw encoder — feeding geometry

#### `hb_gpu_draw_get_funcs`

```c
hb_draw_funcs_t *hb_gpu_draw_get_funcs (const hb_gpu_draw_t *draw);
```

```rust
pub fn hb_gpu_draw_get_funcs(draw: *const hb_gpu_draw_t) -> *mut hb_draw_funcs_t;
```

Fetches the `hb_draw_funcs_t` — the pen — that feeds outline data into a GPU
shape encoder. Pass the `hb_gpu_draw_t` itself as the `draw_data` argument when
you call through it.

**Parameters** — `draw`: a GPU draw context. In the current implementation the
argument is **unused**: the function returns a process-wide static pen, so any
non-null encoder (or, in practice, any value) yields the same pointer. Do not
rely on that; pass the encoder you mean.

**Returns** — the GPU draw functions, never `NULL`.

**Ownership** — transfer none. The pen is a shared immutable singleton owned by
HarfBuzz: do **not** destroy it, and do not try to install your own callbacks on
it.

**Notes** — Since HarfBuzz 14.2.0. This is how you encode geometry that is not a
single glyph — arbitrary shapes, decorations, several glyphs composed into one
blob. Feed it with `hb_draw_move_to()`, `hb_draw_line_to()`,
`hb_draw_quadratic_to()`, `hb_draw_cubic_to()`, `hb_draw_close_path()` from
`hb-draw.h`, or with the composite helpers `hb_draw_line()`,
`hb_draw_rectangle()`, `hb_draw_circle()`. All of those take an
`hb_draw_state_t *` you own, so initialise one from `HB_DRAW_STATE_DEFAULT` and
pass the same one throughout.

#### `hb_gpu_draw_glyph`

```c
void hb_gpu_draw_glyph (hb_gpu_draw_t  *draw,
                        hb_font_t      *font,
                        hb_codepoint_t  glyph);
```

```rust
pub fn hb_gpu_draw_glyph(draw: *mut hb_gpu_draw_t, font: *mut hb_font_t, glyph: hb_codepoint_t);
```

Draws a single glyph outline into the encoder. Exactly
`hb_gpu_draw_glyph_or_fail()` with the return value ignored — including the
implicit `hb_gpu_draw_set_scale()` from the font.

**Parameters** — `draw`: the encoder. `font`: the font to draw from; upstream
recommends leaving its scale at the default (upem). `glyph`: a **glyph ID**, not
a character; get one from a shaped buffer or `hb_font_get_nominal_glyph()`.

**Returns** — nothing. A glyph the font cannot draw is silently a no-op, which
then shows up as an empty blob from `hb_gpu_draw_encode()`.

**Ownership** — nothing is transferred; the font is only read.

**Notes** — Since HarfBuzz 14.0.0. Outlines **accumulate**: calling this twice
without an intervening clear or encode composes both glyphs into one blob, at
their own origins. That is a feature (ligature decomposition, badge composition)
and a trap (forgetting to clear).

#### `hb_gpu_draw_glyph_or_fail`

```c
hb_bool_t hb_gpu_draw_glyph_or_fail (hb_gpu_draw_t  *draw,
                                     hb_font_t      *font,
                                     hb_codepoint_t  glyph);
```

```rust
pub fn hb_gpu_draw_glyph_or_fail(
    draw: *mut hb_gpu_draw_t,
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
) -> hb_bool_t;
```

Convenience to draw one glyph outline, reporting whether the font had one. It
does exactly this:

```c
int x_scale, y_scale;
hb_font_get_scale (font, &x_scale, &y_scale);
hb_gpu_draw_set_scale (draw, x_scale, y_scale);
return hb_font_draw_glyph_or_fail (font, glyph,
                                   hb_gpu_draw_get_funcs (draw), draw);
```

**Parameters** — as `hb_gpu_draw_glyph`.

**Returns** — true if the glyph was drawn, false if the font has no outlines for
`glyph`. Note this reports *the font's* verdict, not the encoder's: an
out-of-memory inside accumulation does **not** show up here, only later as
`NULL` from `hb_gpu_draw_encode()`.

**Ownership** — nothing is transferred.

**Notes** — Since HarfBuzz 14.2.0. Prefer this over `hb_gpu_draw_glyph()` when
you want to distinguish "blank glyph" from "font has no such outline".

### Draw encoder — encoding and reuse

#### `hb_gpu_draw_encode`

```c
hb_blob_t *hb_gpu_draw_encode (hb_gpu_draw_t      *draw,
                               hb_glyph_extents_t *extents);
```

```rust
pub fn hb_gpu_draw_encode(
    draw: *mut hb_gpu_draw_t,
    extents: *mut hb_glyph_extents_t,
) -> *mut hb_blob_t;
```

Encodes the accumulated outlines into a compact blob suitable for GPU rendering.
The blob's data is an array of `RGBA16I` texels, eight bytes each, ready to be
uploaded into a texture buffer object.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `draw` | The encoder. |
| `extents` | Out parameter, **nullable** (`(out) (nullable)` upstream). Receives the computed glyph extents in **font units, Y-up**: `x_bearing` = min x, `y_bearing` = max y, `width` = max x − min x, `height` = min y − max y (so height is negative). Pass `NULL` if you do not need them. |

**Returns** — an `hb_blob_t` containing the encoded data, or:

* `NULL` if encoding failed — allocation failure, an accumulation error, or a
  bounding box that does not fit the 16-bit quantised range;
* the **empty-blob singleton** (`hb_blob_get_empty()`, length 0) if the encoder
  accumulated no outline at all, as for a space glyph.

That distinction is deliberate: length 0 means "nothing to render", `NULL` means
"the encoder failed". Do not conflate them.

**Ownership** — transfer full. The blob owns its own copy of the data; the
caller must release it, either with `hb_gpu_draw_recycle_blob()` (preferred) or
plain `hb_blob_destroy()`.

**Notes** — Since HarfBuzz 14.0.0. Two behaviours worth internalising:

* On return the encoder is **auto-cleared** — via a scope guard, so it happens on
  every path including the failure returns. The next `hb_gpu_draw_glyph()`
  starts fresh. User configuration (the font scale) is preserved.
* `extents` is captured **before** the clear and before the success check, so a
  call that ultimately returns `NULL` may still have written to `extents`. Do
  not trust the extents unless the return value was non-`NULL`.

The upstream section documentation shows a one-argument call
(`hb_gpu_draw_encode (draw)`); that snippet is stale. The real signature takes
two arguments.

#### `hb_gpu_draw_clear`

```c
void hb_gpu_draw_clear (hb_gpu_draw_t *draw);
```

```rust
pub fn hb_gpu_draw_clear(draw: *mut hb_gpu_draw_t);
```

Discards accumulated outlines so the encoder can be reused for another encode.
Resets the pen position, the contour bookkeeping, the internal success flag, and
the running bounding box.

**Parameters** — `draw`: the encoder.

**Returns** — nothing. Cannot fail.

**Ownership** — nothing is transferred.

**Notes** — Since HarfBuzz 14.2.0. User configuration — the font scale — is
**preserved**. Use `hb_gpu_draw_reset()` to also restore configuration. You
rarely need to call this explicitly, since `hb_gpu_draw_encode()` clears for
you; it earns its keep for abandoning a half-fed glyph.

#### `hb_gpu_draw_reset`

```c
void hb_gpu_draw_reset (hb_gpu_draw_t *draw);
```

```rust
pub fn hb_gpu_draw_reset(draw: *mut hb_gpu_draw_t);
```

Resets the encoder: sets `x_scale` and `y_scale` back to `0`, then does
everything `hb_gpu_draw_clear()` does.

**Parameters** — `draw`: the encoder.

**Returns** — nothing. Cannot fail.

**Ownership** — nothing is transferred. User data attached with
`hb_gpu_draw_set_user_data()` is *not* affected; neither is the stashed recycled
blob.

**Notes** — Since HarfBuzz 14.0.0. Note the version asymmetry: `reset` is older
than `clear`, so pre-14.2 code that means "clear" was written as "reset".

#### `hb_gpu_draw_recycle_blob`

```c
void hb_gpu_draw_recycle_blob (hb_gpu_draw_t *draw,
                               hb_blob_t     *blob);
```

```rust
pub fn hb_gpu_draw_recycle_blob(draw: *mut hb_gpu_draw_t, blob: *mut hb_blob_t);
```

Returns a blob to the encoder for potential reuse.

**Parameters** — `draw`: the encoder. `blob`: a blob previously returned by
`hb_gpu_draw_encode()`, `(transfer full)`.

**Returns** — nothing.

**Ownership** — the caller **transfers ownership** of `blob`. Do not touch the
blob afterwards, and do not also call `hb_blob_destroy()` on it — that is a
double free.

**Notes** — Since HarfBuzz 14.0.0. Upstream is explicit that the draw encoder
"currently … simply destroys the blob", and that a future version may reclaim
the underlying buffer to avoid an allocation per encode. So calling it is a
forward-compatible way to spell `hb_blob_destroy()`, and costs nothing today.
(The paint encoder already does reclaim — see below.) Whatever the encoder
stashes is freed when the encoder is destroyed.

### Paint encoder — creation and destruction

#### `hb_gpu_paint_create_or_fail`

```c
hb_gpu_paint_t *hb_gpu_paint_create_or_fail (void);
```

```rust
pub fn hb_gpu_paint_create_or_fail() -> *mut hb_gpu_paint_t;
```

Creates a new GPU color-glyph paint encoder, with reference count 1, palette 0,
no custom palette overrides, and font scale `0`/`0`.

**Parameters** — none.

**Returns** — a newly allocated `hb_gpu_paint_t`, or `NULL` on allocation
failure. No nil-object fallback, no `get_empty()`.

**Ownership** — caller owns the reference; release with
`hb_gpu_paint_destroy()`.

**Notes** — Since HarfBuzz 14.2.0. Create once, reuse across glyphs.

#### `hb_gpu_paint_reference`

```c
hb_gpu_paint_t *hb_gpu_paint_reference (hb_gpu_paint_t *paint);
```

```rust
pub fn hb_gpu_paint_reference(paint: *mut hb_gpu_paint_t) -> *mut hb_gpu_paint_t;
```

Increases the reference count on `paint` by one.

**Parameters** — `paint`: the encoder; nullability unspecified, but the generic
object path tolerates `NULL`.

**Returns** — the same pointer, `(transfer full)`.

**Ownership** — the caller gains a reference to balance with
`hb_gpu_paint_destroy()`.

**Notes** — Since HarfBuzz 14.2.0. Atomic refcount. Marked `(skip)` upstream.

#### `hb_gpu_paint_destroy`

```c
void hb_gpu_paint_destroy (hb_gpu_paint_t *paint);
```

```rust
pub fn hb_gpu_paint_destroy(paint: *mut hb_gpu_paint_t);
```

Decreases the reference count on `paint` by one; at zero, frees the encoder,
its sub-blobs, its custom-palette map, its recycled blob, and its user data.

**Parameters** — `paint`: the encoder; `NULL` tolerated.

**Returns** — nothing.

**Ownership** — consumes one reference.

**Notes** — Since HarfBuzz 14.2.0. Marked `(skip)` upstream.

#### `hb_gpu_paint_set_user_data`

```c
hb_bool_t hb_gpu_paint_set_user_data (hb_gpu_paint_t     *paint,
                                      hb_user_data_key_t *key,
                                      void               *data,
                                      hb_destroy_func_t   destroy,
                                      hb_bool_t           replace);
```

```rust
pub fn hb_gpu_paint_set_user_data(
    paint: *mut hb_gpu_paint_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

Attaches user data to the paint encoder. Parameters, return value and ownership
are identical to `hb_gpu_draw_set_user_data()`: `key` is used by address and
must outlive the encoder, `data` may be null, `destroy` is nullable and fires
once, `replace` decides whether an occupied key is overwritten.

**Returns** — true on success, false otherwise.

**Notes** — Since HarfBuzz 14.2.0. Marked `(skip)` upstream.

#### `hb_gpu_paint_get_user_data`

```c
void *hb_gpu_paint_get_user_data (const hb_gpu_paint_t *paint,
                                  hb_user_data_key_t   *key);
```

```rust
pub fn hb_gpu_paint_get_user_data(
    paint: *const hb_gpu_paint_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

Fetches the user data associated with `key`.

**Returns** — the stored pointer, or `NULL` if nothing is stored under that key.

**Ownership** — transfer none; do not free.

**Notes** — Since HarfBuzz 14.2.0. Marked `(skip)` upstream.

### Paint encoder — palette and colour configuration

Everything in this group is *baked into the blob at encode time*, so changing it
after encoding requires re-encoding the affected glyphs. The one colour that is
**not** baked is the **foreground colour**, which stays a shader uniform — that
is what lets a dark-mode toggle take effect without re-encoding anything.

#### `hb_gpu_paint_set_palette`

```c
void hb_gpu_paint_set_palette (hb_gpu_paint_t *paint,
                               unsigned        palette);
```

```rust
pub fn hb_gpu_paint_set_palette(paint: *mut hb_gpu_paint_t, palette: c_uint);
```

Selects which font palette is used when paint callbacks look up indexed colours.

**Parameters** — `paint`: the encoder. `palette`: a palette index; the default
is 0. The API does not validate it — an out-of-range index is resolved by the
font machinery downstream (`hb-ot-color.h` exposes
`hb_ot_color_palette_get_count()` if you want to bound it yourself).

**Returns** — nothing. Cannot fail.

**Notes** — Since HarfBuzz 14.2.0. This is user configuration: preserved by
`hb_gpu_paint_clear()` and by `hb_gpu_paint_encode()`, reset to 0 only by
`hb_gpu_paint_reset()`. Set it **before** `hb_gpu_paint_glyph()`, because the
palette index is passed to the font's paint walk at that moment.

#### `hb_gpu_paint_get_palette`

```c
unsigned hb_gpu_paint_get_palette (const hb_gpu_paint_t *paint);
```

```rust
pub fn hb_gpu_paint_get_palette(paint: *const hb_gpu_paint_t) -> c_uint;
```

Returns the palette index previously set on the encoder, or 0 if none was set.

**Parameters** — `paint`: the encoder (const).

**Returns** — the palette index.

**Ownership** — nothing is transferred.

**Notes** — Since HarfBuzz 14.2.0.

#### `hb_gpu_paint_clear_custom_palette_colors`

```c
void hb_gpu_paint_clear_custom_palette_colors (hb_gpu_paint_t *paint);
```

```rust
pub fn hb_gpu_paint_clear_custom_palette_colors(paint: *mut hb_gpu_paint_t);
```

Clears all custom palette colour overrides previously set on the encoder.

**Parameters** — `paint`: the encoder.

**Returns** — nothing. Cannot fail; a no-op if no overrides were ever set.

**Notes** — Since HarfBuzz 14.2.0. Note the naming asymmetry with
`hb_gpu_paint_clear()`: this one clears *colour overrides*, that one clears
*accumulated geometry*. They are unrelated.

#### `hb_gpu_paint_set_custom_palette_color`

```c
hb_bool_t hb_gpu_paint_set_custom_palette_color (hb_gpu_paint_t *paint,
                                                 unsigned int    color_index,
                                                 hb_color_t      color);
```

```rust
pub fn hb_gpu_paint_set_custom_palette_color(
    paint: *mut hb_gpu_paint_t,
    color_index: c_uint,
    color: hb_color_t,
) -> hb_bool_t;
```

Overrides one font palette colour entry.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `paint` | The encoder. |
| `color_index` | The palette entry index to override. Not validated against the font. |
| `color` | The replacement colour, an `hb_color_t` — build it with `HB_COLOR(b, g, r, a)`; note the blue-first argument order. |

**Returns** — true if the override was set, false on allocation failure (the
override table is an `hb_map_t` created lazily on first use).

**Ownership** — `color` is a value; nothing is transferred.

**Notes** — Since HarfBuzz 14.2.0. Overrides are keyed by `color_index` and
persist on the encoder until cleared, or replaced for the same index. They are
user configuration, preserved across clears and encodes, and dropped by
`hb_gpu_paint_reset()`. Set them **before** `hb_gpu_paint_glyph()`.

#### `hb_gpu_paint_set_scale`

```c
void hb_gpu_paint_set_scale (hb_gpu_paint_t *paint,
                             int             x_scale,
                             int             y_scale);
```

```rust
pub fn hb_gpu_paint_set_scale(paint: *mut hb_gpu_paint_t, x_scale: c_int, y_scale: c_int);
```

Sets the font scale used to dimension the clip-glyph Slug outlines inside the
encoded blob.

**Parameters** — `paint`: the encoder. `x_scale`, `y_scale`: typically from
`hb_font_get_scale()`. Default `0`/`0`.

**Returns** — nothing. Cannot fail.

**Notes** — Since HarfBuzz 14.2.0. Called automatically by
`hb_gpu_paint_glyph()` and `hb_gpu_paint_glyph_or_fail()`. Preserved across
clears and encodes; zeroed by `hb_gpu_paint_reset()`.

#### `hb_gpu_paint_get_scale`

```c
void hb_gpu_paint_get_scale (const hb_gpu_paint_t *paint,
                             int                  *x_scale,
                             int                  *y_scale);
```

```rust
pub fn hb_gpu_paint_get_scale(
    paint: *const hb_gpu_paint_t,
    x_scale: *mut c_int,
    y_scale: *mut c_int,
);
```

Fetches the font scale previously set on the encoder.

**Parameters** — `paint`: the encoder (const). `x_scale`, `y_scale`: out
parameters, each **null-tolerant** — the implementation writes only through
non-null pointers.

**Returns** — nothing.

**Notes** — Since HarfBuzz 14.2.0.

### Paint encoder — feeding geometry

#### `hb_gpu_paint_get_funcs`

```c
hb_paint_funcs_t *hb_gpu_paint_get_funcs (const hb_gpu_paint_t *paint);
```

```rust
pub fn hb_gpu_paint_get_funcs(paint: *const hb_gpu_paint_t) -> *mut hb_paint_funcs_t;
```

Fetches the `hb_paint_funcs_t` that feeds paint data into a GPU paint encoder.
Pass the `hb_gpu_paint_t` itself as the `paint_data` argument when calling
through it.

**Parameters** — `paint`: a GPU paint context. As with the draw pen, the
argument is unused in the current implementation — a process-wide static
callback table is returned.

**Returns** — the GPU paint functions, never `NULL`.

**Ownership** — transfer none; a shared immutable singleton owned by HarfBuzz.
Do not destroy it and do not install callbacks on it.

**Notes** — Since HarfBuzz 14.2.0. Use it with
`hb_font_paint_glyph()`/`hb_font_paint_glyph_or_fail()` from `hb-font.h` when
you need control over the palette and foreground arguments beyond what
`hb_gpu_paint_glyph()` gives you.

#### `hb_gpu_paint_glyph`

```c
void hb_gpu_paint_glyph (hb_gpu_paint_t *paint,
                         hb_font_t      *font,
                         hb_codepoint_t  glyph);
```

```rust
pub fn hb_gpu_paint_glyph(
    paint: *mut hb_gpu_paint_t,
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
);
```

Feeds a glyph into the encoder. Copies the font's scale onto the encoder, then
calls `hb_font_paint_glyph()` with the encoder's palette and an opaque black
foreground.

**Parameters** — `paint`: the encoder. `font`: the font to paint from. `glyph`:
a glyph ID.

**Returns** — nothing.

**Ownership** — nothing is transferred.

**Notes** — Since HarfBuzz 14.2.0. Unlike `hb_gpu_paint_glyph_or_fail()`,
**non-colour glyphs are handled transparently**: HarfBuzz synthesises a single
foreground-coloured layer from the glyph's outline, so any glyph with an outline
produces output. That is what lets an application use the paint renderer
uniformly for every font.

The opaque-black foreground passed internally is not a real colour. The encoder
records only an *is-foreground* flag per layer or gradient stop; the actual
colour is substituted in the shader from the `foreground` uniform. The alpha
must be opaque so that the baked colour carries only the paint tree's own alpha.

#### `hb_gpu_paint_glyph_or_fail`

```c
hb_bool_t hb_gpu_paint_glyph_or_fail (hb_gpu_paint_t *paint,
                                      hb_font_t      *font,
                                      hb_codepoint_t  glyph);
```

```rust
pub fn hb_gpu_paint_glyph_or_fail(
    paint: *mut hb_gpu_paint_t,
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
) -> hb_bool_t;
```

Convenience to feed a glyph's paint tree into the encoder. Equivalent to:

```c
int x_scale, y_scale;
hb_font_get_scale (font, &x_scale, &y_scale);
hb_gpu_paint_set_scale (paint, x_scale, y_scale);
return hb_font_paint_glyph_or_fail (font, glyph,
                                    hb_gpu_paint_get_funcs (paint), paint,
                                    palette, HB_COLOR (0, 0, 0, 0xff));
```

**Parameters** — as `hb_gpu_paint_glyph`.

**Returns** — true if the font had **paint data** for `glyph`, false otherwise.
Because this is the `_or_fail` form of the font call, a monochrome glyph in a
font with no paint table returns false and encodes nothing — the synthesised
foreground layer of `hb_gpu_paint_glyph()` does not happen here.

**Ownership** — nothing is transferred.

**Notes** — Since HarfBuzz 14.2.0. Encoder-level limitations — unsupported paint
operations, group-stack overflow, too many clip sub-blobs — do **not** fail
here. They raise the encoder's internal `unsupported` flag and surface later as
a `NULL` from `hb_gpu_paint_encode()`.

### Paint encoder — encoding and reuse

#### `hb_gpu_paint_encode`

```c
hb_blob_t *hb_gpu_paint_encode (hb_gpu_paint_t     *paint,
                                hb_glyph_extents_t *extents);
```

```rust
pub fn hb_gpu_paint_encode(
    paint: *mut hb_gpu_paint_t,
    extents: *mut hb_glyph_extents_t,
) -> *mut hb_blob_t;
```

Encodes the accumulated paint state into a GPU-renderable blob and writes the
glyph's ink extents.

The blob layout is a three-texel header, then the op stream (an array of
`int16` words, padded to a multiple of four), then the concatenated clip
sub-payloads — each of which is itself a draw-encoder Slug blob, so everything
stays eight-byte aligned. Texel format is the same `RGBA16I` as the draw
renderer's, so draw blobs and paint blobs can coexist in one atlas at different
offsets.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `paint` | The encoder. |
| `extents` | Out parameter. Upstream annotates it `(out)` with no `(nullable)`, but the implementation guards it — `if (extents)` — so `NULL` is accepted in practice. Receives ink extents in font units: `x_bearing` = min x, `y_bearing` = max y, `width` = max x − min x, `height` = min y − max y. |

**Returns** —

* a newly allocated blob on success;
* `NULL` if the paint walk hit an unsupported feature (see the encoder-limit
  table under `hb_gpu_paint_t`), or if encoding failed on allocation;
* the **empty-blob singleton** (length 0) if the paint accumulated no ink, as
  for a space glyph.

As with the draw path, length 0 means "nothing to render" and `NULL` means
"failed".

**Ownership** — transfer full. Release with `hb_gpu_paint_recycle_blob()`
(preferred, because the paint encoder really does reuse the buffer) or
`hb_blob_destroy()`.

**Notes** — Since HarfBuzz 14.2.0. On return the encoder is **auto-cleared** via
a scope guard, on every path; user configuration — palette, custom palette
overrides, font scale — is preserved. Unlike the draw path, `extents` is written
**only on the success path**, near the end of the function, so on a `NULL`
return the caller's struct is left untouched.

#### `hb_gpu_paint_clear`

```c
void hb_gpu_paint_clear (hb_gpu_paint_t *paint);
```

```rust
pub fn hb_gpu_paint_clear(paint: *mut hb_gpu_paint_t);
```

Discards accumulated paint state so the encoder can be reused for another
encode: the op stream, the clip sub-blobs (each destroyed), the clip depth, the
`unsupported` flag, the transform stack, and the running ink extents.

**Parameters** — `paint`: the encoder.

**Returns** — nothing. Cannot fail.

**Notes** — Since HarfBuzz 14.2.0. User configuration is preserved. Rarely
needed explicitly, since `hb_gpu_paint_encode()` clears for you.

#### `hb_gpu_paint_reset`

```c
void hb_gpu_paint_reset (hb_gpu_paint_t *paint);
```

```rust
pub fn hb_gpu_paint_reset(paint: *mut hb_gpu_paint_t);
```

Resets the encoder: zeroes the font scale, sets the palette back to 0, destroys
the custom-palette override map, then does everything `hb_gpu_paint_clear()`
does.

**Parameters** — `paint`: the encoder.

**Returns** — nothing. Cannot fail.

**Ownership** — user data attached with `hb_gpu_paint_set_user_data()` is not
affected.

**Notes** — Since HarfBuzz 14.2.0. This is the only call that drops custom
palette colours *and* the palette index in one step.

#### `hb_gpu_paint_recycle_blob`

```c
void hb_gpu_paint_recycle_blob (hb_gpu_paint_t *paint,
                                hb_blob_t      *blob);
```

```rust
pub fn hb_gpu_paint_recycle_blob(paint: *mut hb_gpu_paint_t, blob: *mut hb_blob_t);
```

Returns a blob to the encoder for potential reuse.

**Parameters** — `paint`: the encoder. `blob`: a blob previously returned by
`hb_gpu_paint_encode()`, `(transfer full)`.

**Returns** — nothing.

**Ownership** — the caller **transfers ownership**. Do not use or destroy the
blob afterwards.

**Notes** — Since HarfBuzz 14.2.0. Unlike the draw encoder's stub, this one
genuinely pays: if the blob came from `hb_gpu_paint_encode()`, its underlying
buffer is reused by the next `hb_gpu_paint_encode()`, avoiding one `malloc` and
one blob allocation per glyph. Only one blob is stashed at a time; the stash is
freed with the encoder. Call it **after** you have uploaded the bytes, never
before.

## The shader side

The GPU half of this library is not C API, so it has no entry in the section
list — but you cannot use `hb-gpu.h` without it. Everything below is defined by
the strings the three `*_shader_source` functions return. Names are the GLSL
spelling; the other three languages expose the same functions.

### Atlas setup

All encoded blobs go into a **single buffer texture** of `RGBA16I` texels. Each
glyph occupies a contiguous run of texels at an offset you record and later pass
to the fragment shader.

```c
GLuint buf, tex;
glGenBuffers (1, &buf);
glGenTextures (1, &tex);

glBindBuffer (GL_TEXTURE_BUFFER, buf);
glBufferData (GL_TEXTURE_BUFFER, capacity * 8, NULL, GL_STATIC_DRAW);

glBindTexture (GL_TEXTURE_BUFFER, tex);
glTexBuffer (GL_TEXTURE_BUFFER, GL_RGBA16I, buf);

/* Upload one glyph blob at texel 'offset': */
glBufferSubData (GL_TEXTURE_BUFFER,
                 offset * 8,
                 hb_blob_get_length (blob),
                 hb_blob_get_data (blob, NULL));
```

The shared fragment source declares `uniform isamplerBuffer hb_gpu_atlas;` and
reads through `ivec4 hb_gpu_fetch (int offset)`. On platforms without texture
buffers — WebGL 2, notably — define **`HB_GPU_ATLAS_2D`** before the shared
source; the atlas then becomes `uniform highp isampler2D hb_gpu_atlas` and you
must additionally set `uniform int hb_gpu_atlas_width` to the texture width in
texels.

The shared source also defines `HB_GPU_UNITS_PER_EM` as 4 unless you define it
first — the same quantisation constant as the C side.

### Vertex stage

The shared vertex source provides exactly one function:

```glsl
void hb_gpu_dilate (inout vec2 position, inout vec2 texcoord,
                    vec2 normal, vec4 jac,
                    mat4 m, vec2 viewport);
```

It expands each glyph quad by about half a pixel on screen so the antialiased
edges are not clipped at the quad boundary, adjusting both the position and the
texcoord (so the fragment shader still samples the right em-space location).
Call it before computing `gl_Position`.

Per-vertex attributes for a glyph quad — two triangles, six vertices, four
unique corners:

| Attribute | Type | Value |
| --- | --- | --- |
| `position` | `vec2` | Object-space vertex position. For corner `(cx, cy)` with `cx, cy ∈ {0, 1}`: `pos.x = pen_x + scale * mix(extent_min_x, extent_max_x, cx)`, `pos.y = pen_y - scale * mix(extent_min_y, extent_max_y, cy)`, where `scale = font_size / upem`. |
| `texcoord` | `vec2` | Em-space sample coordinates — the raw extent values `(ex, ey)` in font design units. |
| `normal` | `vec2` | Outward normal: `(-1, +1)` for corner (0,0), `(+1, +1)` for (1,0), `(-1, -1)` for (0,1), `(+1, -1)` for (1,1). Y is negated relative to `cx, cy` because the usual em-to-object transform flips Y. |
| `jac` | `vec4` | Inverse of the 2×2 linear part of the em-to-object transform, row-major `(j00, j01, j10, j11)`. For uniform scaling with Y-flip: `vec4(emPerPos, 0.0, 0.0, -emPerPos)` where `emPerPos = upem / font_size`. |
| `m` | `mat4` uniform | Model-view-projection matrix. |
| `viewport` | `vec2` uniform | Viewport size in pixels. |

A typical vertex `main`:

```glsl
uniform mat4 u_matViewProjection;
uniform vec2 u_viewport;

in vec2 a_position;
in vec2 a_texcoord;
in vec2 a_normal;
in float a_emPerPos;
in uint a_glyphLoc;

out vec2 v_texcoord;
flat out uint v_glyphLoc;

void main () {
  vec2 pos = a_position;
  vec2 tex = a_texcoord;
  vec4 jac = vec4 (a_emPerPos, 0.0, 0.0, -a_emPerPos);
  hb_gpu_dilate (pos, tex, a_normal, jac, u_matViewProjection, u_viewport);
  gl_Position = u_matViewProjection * vec4 (pos, 0.0, 1.0);
  v_texcoord = tex;
  v_glyphLoc = a_glyphLoc;
}
```

**Static dilation alternative** — if you do not need perspective correctness
(strictly 2D at known sizes), skip `hb_gpu_dilate` and expand each vertex by a
fixed amount along its normal, sized from the smallest font size you expect:

```glsl
float min_ppem = 10.0;              // smallest expected size in pixels
float pad = 0.5 * upem / min_ppem;  // half a pixel, in em units
pos += normal * pad * scale;
texcoord += normal * pad;
```

This over-dilates at larger sizes — wasted fill rate on transparent pixels — but
needs neither the MVP matrix nor the viewport in the vertex shader.

### Fragment stage

```glsl
/* from hb_gpu_shader_source (FRAGMENT, …) */
ivec4 hb_gpu_fetch       (int offset);
float hb_gpu_ppem        (vec2 renderCoord, uint glyphLoc);
float hb_gpu_stem_darken (float coverage, float brightness, float ppem);

/* from hb_gpu_draw_shader_source (FRAGMENT, …) */
float hb_gpu_draw (vec2 renderCoord, uint glyphLoc);

/* from hb_gpu_paint_shader_source (FRAGMENT, …) */
vec4 hb_gpu_paint (vec2 renderCoord, uint glyphLoc, vec4 foreground,
                   out float coverage);
```

`renderCoord` is the interpolated em-space coordinate from the vertex shader
(`v_texcoord`); `glyphLoc` is the texel offset of this glyph's encoded blob in
the atlas, passed as a **flat** varying.

`hb_gpu_draw` returns coverage in `[0, 1]`: 1 inside the glyph, 0 outside,
antialiased at the edges. Composite it with `(GL_SRC_ALPHA,
GL_ONE_MINUS_SRC_ALPHA)`, or multiply it into any colour.

`hb_gpu_paint` returns **premultiplied RGBA** — fully transparent outside the
glyph, composited layers inside — and writes the maximum antialiasing coverage
across all paint layers to its `out` parameter. Composite with `(GL_ONE,
GL_ONE_MINUS_SRC_ALPHA)`. Its `foreground` argument resolves palette entries
that COLRv1 marks as "foreground colour": the encoder recorded only the flag, so
a dark-mode toggle is a uniform change, not a re-encode.

`hb_gpu_stem_darken` optionally adjusts coverage at small sizes so thin stems
are not washed out by gamma correction. `brightness` is the on-screen brightness
of this fragment's colour, `dot(straight_color.rgb, vec3(1.0 / 3.0))`; `ppem` is
pixels per em, either `1.0 / max(fwidth(v_texcoord).x, fwidth(v_texcoord).y)` or
`hb_gpu_ppem(v_texcoord, v_glyphLoc)`.

Draw-path fragment `main`:

```glsl
in vec2 v_texcoord;
flat in uint v_glyphLoc;
out vec4 fragColor;

void main () {
  float coverage = hb_gpu_draw (v_texcoord, v_glyphLoc);
  fragColor = vec4 (0.0, 0.0, 0.0, coverage);
}
```

Paint-path fragment `main`, with stem darkening and gamma applied to the
*coverage* rather than the colour, so interior paint colours are untouched:

```glsl
uniform vec4 u_foreground;
uniform float u_gamma;
uniform float u_stem_darkening;
in vec2 v_texcoord;
flat in uint v_glyphLoc;
out vec4 fragColor;

void main () {
  float cov;
  vec4 c = hb_gpu_paint (v_texcoord, v_glyphLoc, u_foreground, cov);
  if (cov > 0.0 && cov < 1.0) {
    float adj = cov;
    if (u_stem_darkening > 0.0) {
      float brightness = c.a > 0.0 ? dot (c.rgb, vec3 (1.0 / 3.0)) / c.a : 0.0;
      adj = hb_gpu_stem_darken (adj, brightness,
              1.0 / max (fwidth (v_texcoord).x, fwidth (v_texcoord).y));
    }
    if (u_gamma != 1.0)
      adj = pow (adj, u_gamma);
    c *= adj / cov;
  }
  fragColor = c;
}
```

## Usage

### C — encode a run of monochrome glyphs

```c
#include <hb.h>
#include <hb-gpu.h>

hb_gpu_draw_t *draw = hb_gpu_draw_create_or_fail ();
if (!draw)
  return -1;  /* out of memory */

for (unsigned i = 0; i < glyph_count; i++)
{
  hb_codepoint_t gid = infos[i].codepoint;

  if (!hb_gpu_draw_glyph_or_fail (draw, font, gid))
    continue;  /* font has no outline for this glyph */

  hb_glyph_extents_t ext;
  hb_blob_t *blob = hb_gpu_draw_encode (draw, &ext);
  if (!blob)
    continue;  /* encoding failed; encoder was still auto-cleared */

  unsigned len = hb_blob_get_length (blob);
  if (len)
  {
    unsigned offset = upload_to_atlas (hb_blob_get_data (blob, NULL), len);
    record_glyph (gid, offset, &ext);
  }
  /* len == 0 is the empty-blob singleton: nothing to render. */

  hb_gpu_draw_recycle_blob (draw, blob);  /* transfers ownership */
}

hb_gpu_draw_destroy (draw);
```

### C — assemble the shaders

```c
const char *vert_sources[] = {
  "#version 330\n",
  hb_gpu_shader_source      (HB_GPU_SHADER_STAGE_VERTEX, HB_GPU_SHADER_LANG_GLSL),
  hb_gpu_draw_shader_source (HB_GPU_SHADER_STAGE_VERTEX, HB_GPU_SHADER_LANG_GLSL),
  your_vertex_main
};
glShaderSource (vert_shader, 4, vert_sources, NULL);

const char *frag_sources[] = {
  "#version 330\n",
  hb_gpu_shader_source      (HB_GPU_SHADER_STAGE_FRAGMENT, HB_GPU_SHADER_LANG_GLSL),
  hb_gpu_draw_shader_source (HB_GPU_SHADER_STAGE_FRAGMENT, HB_GPU_SHADER_LANG_GLSL),
  your_fragment_main
};
glShaderSource (frag_shader, 4, frag_sources, NULL);
```

For the paint path, insert `hb_gpu_paint_shader_source(...)` **after** the draw
source in the fragment array (five strings), because `hb_gpu_paint` calls
`hb_gpu_draw`.

### Rust — encode one glyph

```rust
use harfbuzz_sys::gpu::*;
use harfbuzz_sys::{hb_blob_get_data, hb_blob_get_length, hb_glyph_extents_t};

unsafe {
    let draw = hb_gpu_draw_create_or_fail();
    assert!(!draw.is_null(), "out of memory");

    // `font` is a *mut hb_font_t you obtained elsewhere; leave its scale at
    // the default (upem) so the blob stays in font design units.
    if hb_gpu_draw_glyph_or_fail(draw, font, glyph_id) != 0 {
        let mut ext = hb_glyph_extents_t {
            x_bearing: 0,
            y_bearing: 0,
            width: 0,
            height: 0,
        };
        let blob = hb_gpu_draw_encode(draw, &raw mut ext);

        if !blob.is_null() {
            let len = hb_blob_get_length(blob);
            if len != 0 {
                let ptr = hb_blob_get_data(blob, core::ptr::null_mut());
                let texels = core::slice::from_raw_parts(ptr.cast::<u8>(), len as usize);
                upload_to_atlas(texels, &ext);
            }
            // Hands ownership back; do NOT also call hb_blob_destroy.
            hb_gpu_draw_recycle_blob(draw, blob);
        }
    }

    hb_gpu_draw_destroy(draw);
}
```

### Rust — fetch a shader source string

```rust
use core::ffi::CStr;
use harfbuzz_sys::gpu::*;

unsafe fn glsl_fragment_prelude() -> Option<(&'static CStr, &'static CStr)> {
    let shared = hb_gpu_shader_source(
        HB_GPU_SHADER_STAGE_FRAGMENT,
        HB_GPU_SHADER_LANG_GLSL,
    );
    let draw = hb_gpu_draw_shader_source(
        HB_GPU_SHADER_STAGE_FRAGMENT,
        HB_GPU_SHADER_LANG_GLSL,
    );
    if shared.is_null() || draw.is_null() {
        return None; // unsupported stage/language combination
    }
    // Static strings owned by HarfBuzz: 'static is honest, and they must not
    // be freed.
    Some((CStr::from_ptr(shared), CStr::from_ptr(draw)))
}
```

### Rust — a colour glyph with a palette override

```rust
use harfbuzz_sys::gpu::*;
use harfbuzz_sys::{hb_glyph_extents_t, HB_COLOR};

unsafe {
    let paint = hb_gpu_paint_create_or_fail();
    assert!(!paint.is_null());

    // Configuration is baked into every subsequent encode, and survives
    // clears — set it once, before feeding any glyph.
    hb_gpu_paint_set_palette(paint, 1);
    if hb_gpu_paint_set_custom_palette_color(paint, 2, HB_COLOR(0, 0, 0xff, 0xff)) == 0 {
        // allocation failure building the override map
    }

    hb_gpu_paint_glyph(paint, font, glyph_id); // synthesises a layer for
                                               // monochrome glyphs too

    let mut ext = hb_glyph_extents_t { x_bearing: 0, y_bearing: 0, width: 0, height: 0 };
    let blob = hb_gpu_paint_encode(paint, &raw mut ext);
    if blob.is_null() {
        // unsupported paint feature, or out of memory. `ext` was NOT written.
    } else {
        // upload, then:
        hb_gpu_paint_recycle_blob(paint, blob);
    }

    hb_gpu_paint_destroy(paint);
}
```

### C — encode an arbitrary shape, not a glyph

The draw encoder is a general Slug encoder; glyphs are just the common case.
Grab its pen and drive it with the `hb-draw.h` primitives, passing the encoder
as `draw_data`:

```c
hb_gpu_draw_t   *draw   = hb_gpu_draw_create_or_fail ();
hb_draw_funcs_t *dfuncs = hb_gpu_draw_get_funcs (draw);
hb_draw_state_t  st     = HB_DRAW_STATE_DEFAULT;

/* Font-unit-scale coordinates: comfortably inside +/-8000, and details
 * stay well above the ~0.25 unit precision floor. */
hb_gpu_draw_set_scale (draw, 1000, 1000);

/* stroke_width = NAN means "filled"; a finite positive width outlines. */
hb_draw_rectangle (dfuncs, draw, &st, 0, 0, 800, 120, NAN);  /* underline bar */
hb_draw_circle    (dfuncs, draw, &st, 900, 60, 50, NAN);     /* a filled dot */

hb_glyph_extents_t ext;
hb_blob_t *blob = hb_gpu_draw_encode (draw, &ext);
/* ... upload ... */
hb_gpu_draw_recycle_blob (draw, blob);
hb_gpu_draw_destroy (draw);
```

`hb_draw_line()`, `hb_draw_rectangle()` and `hb_draw_circle()` cover tapered
lines, rectangles and circles; anything else is `hb_draw_move_to()` /
`hb_draw_line_to()` / `hb_draw_quadratic_to()` / `hb_draw_cubic_to()` /
`hb_draw_close_path()`. Remember to close the last contour.

## Pitfalls

**The `gpu` feature is not optional in practice.** Without it neither the Rust
module nor the C symbols exist. `harfbuzz_sys::gpu::*` is also *not* glob
re-exported at the crate root, unlike almost every other module, so
`harfbuzz_sys::hb_gpu_draw_encode` does not resolve —
`harfbuzz_sys::gpu::hb_gpu_draw_encode` does.

**`NULL` and the empty blob mean different things.** Both `encode` functions
return the empty-blob singleton (length 0) for a glyph with no ink, and `NULL`
for a real failure. Checking only `!= NULL` will make you upload zero bytes and
render nothing; checking only `length != 0` will make you dereference `NULL`.
Check both, in that order.

**`encode` clears the encoder even when it fails.** Both functions install a
scope guard, so every return path — including the `NULL` ones — leaves the
encoder empty. There is no way to retry an encode; you must feed the glyph
again.

**`hb_gpu_draw_encode` writes `extents` before it can fail.** The extents are
captured up front, then the success check runs. A `NULL` return can therefore
leave a half-meaningful value in your struct. `hb_gpu_paint_encode` is the other
way round — it writes extents only on success. Do not assume the two behave
alike; gate on the return value in both cases.

**Geometry accumulates.** Two `hb_gpu_draw_glyph()` calls without an intervening
encode or clear produce one blob containing both outlines. If you loop over
glyphs and forget to encode inside the loop, you get one enormous blob whose
bounding box probably exceeds the ±8000 quantisation range — which makes
`encode` return `NULL` with no other diagnostic.

**Quantisation is unforgiving of normalised coordinates.** Coordinates are
stored as `int16` at four steps per unit: ±~8000 range, ~0.25 unit precision.
Encoding a 0..1 unit square means everything lands in four quantisation steps
and thin features vanish entirely. Work in font units and scale in the vertex
shader.

**Do not scale the `hb_font_t`.** Upstream recommends leaving the font at its
default (upem) scale so the blob and extents stay in design units. If you scale
the font, the blob is in that scaled space and the shader's `emPerPos` must be
recomputed to match — a mismatch shows up as glyphs that are subtly the wrong
size, or as dilation that no longer lands on half a pixel.

**Recycle exactly once, and only after uploading.** `…_recycle_blob` takes
ownership. Calling `hb_blob_destroy()` on the same blob afterwards is a double
free; reading `hb_blob_get_data()` afterwards is a use-after-free. For the paint
encoder, which really does reuse the buffer, the next `hb_gpu_paint_encode()`
will overwrite those bytes — so upload first.

**The two `recycle` functions do not do the same thing.**
`hb_gpu_paint_recycle_blob` reclaims the buffer; `hb_gpu_draw_recycle_blob`
currently just destroys the blob, with reuse noted as a future improvement.
Write the calls anyway — they are the forward-compatible spelling.

**`clear` versus `reset` versus `clear_custom_palette_colors`.** `clear` drops
geometry, keeps configuration. `reset` drops geometry *and* configuration —
font scale, and for paint the palette index and every custom colour.
`hb_gpu_paint_clear_custom_palette_colors` drops only the colour overrides,
keeping the palette index and the geometry. Three similar names, three different
scopes.

**Palette and custom colours are baked at encode time; the foreground is not.**
Change the palette or an override and every already-encoded glyph keeps its old
colours until you re-encode it. The foreground colour, by contrast, is a shader
uniform, so dark-mode toggles are free.

**`hb_gpu_paint_glyph` and `hb_gpu_paint_glyph_or_fail` are not the same
function with a return value bolted on.** The `_or_fail` form uses
`hb_font_paint_glyph_or_fail()` and returns false for a glyph with no paint
data — encoding nothing. The plain form uses `hb_font_paint_glyph()`, which
synthesises a foreground-coloured layer from the outline, so a monochrome glyph
still renders. Choosing `_or_fail` for its error reporting silently loses
monochrome glyphs. (The draw pair *is* the simple case: `hb_gpu_draw_glyph()` is
`hb_gpu_draw_glyph_or_fail()` with the result discarded.)

**Paint failures are reported late.** Unsupported paint operations, group nesting
deeper than 4, more than 3 clips on a layer, more than 1024 clip sub-blobs, or
more than 0x7fff ops all raise an internal flag during the paint walk and only
surface as `NULL` from `hb_gpu_paint_encode()`. The glyph-feeding call returns
true.

**Shader source order is load-bearing.** Shared source first, then the renderer
source, then your `main()`. For the paint path the fragment order is shared →
draw → paint → `main`, because the paint interpreter calls `hb_gpu_draw()`. The
renderer sources return `""` (not `NULL`) for the vertex stage precisely so you
can concatenate without branching — but `HB_GPU_SHADER_LANG_INVALID` still gives
`NULL`, and passing a `NULL` into `glShaderSource` is undefined behaviour, so
validate the language once at startup.

**`get_funcs` returns a shared singleton.** Both `hb_gpu_draw_get_funcs()` and
`hb_gpu_paint_get_funcs()` ignore their argument and return a process-wide
static callback table. Never destroy it, never install callbacks on it, and do
not assume two encoders have distinct pens.

**`hb_codepoint_t` here is a glyph ID.** The same type carries Unicode scalars
elsewhere in HarfBuzz. Passing `'A' as u32` will encode whatever glyph happens
to sit at index 65.

**Encoders are not thread-safe.** The reference count is atomic, so
`reference`/`destroy` may cross threads, but the accumulation state is not
guarded. Use one encoder per thread. The shader-source functions are pure and
safe from anywhere.

**Rust reminders.** Every function here is `unsafe` and this crate adds no
checking. `hb_bool_t` is `c_int` — compare against `0`, do not transmute.
`hb_gpu_draw_t` and `hb_gpu_paint_t` are `opaque_handle!` types: zero-sized,
non-constructible, `!Send`/`!Sync`, only ever used behind pointers. The
`*_shader_source` returns are `*const c_char` pointing at static storage, so
`CStr::from_ptr` with a `'static` lifetime is honest — but check for null first.
Out-parameters that HarfBuzz documents as null-tolerant (`get_scale`,
`hb_gpu_draw_encode`'s `extents`) accept `core::ptr::null_mut()`.

## Related, but not declared in this header

`hb-gpu.h` includes `hb.h` and adds nothing else, but the following are part of
using it and live elsewhere in this crate:

* `hb_draw_move_to()`, `hb_draw_line_to()`, `hb_draw_quadratic_to()`,
  `hb_draw_cubic_to()`, `hb_draw_close_path()`, `hb_draw_line()`,
  `hb_draw_rectangle()`, `hb_draw_circle()`, `hb_draw_state_t`,
  `HB_DRAW_STATE_DEFAULT` — `hb-draw.h`, see `docs/draw.md`. Needed to feed the
  pen from `hb_gpu_draw_get_funcs()`.
* `hb_font_draw_glyph_or_fail()`, `hb_font_paint_glyph()`,
  `hb_font_paint_glyph_or_fail()`, `hb_font_get_scale()` — `hb-font.h`, see
  `docs/font.md`. The `*_glyph` convenience wrappers here are thin compositions
  of these.
* `hb_ot_color_has_paint()`, `hb_ot_color_palette_get_count()` — `hb-ot-color.h`,
  see `docs/ot_color.md`. Use the first to choose between the draw and paint
  renderers per font, and the second to bound a palette index.
* `hb_blob_get_data()`, `hb_blob_get_length()`, `hb_blob_destroy()`,
  `hb_blob_get_empty()` — `hb-blob.h`, see `docs/blob.md`. Everything you do
  with an encoded blob.
* `HB_COLOR()`, `hb_color_t`, `hb_glyph_extents_t` — `hb-common.h`.

All of these are glob re-exported at the crate root as `harfbuzz_sys::NAME`.

The C++-only `HB_DEFINE_VTABLE (gpu_draw, …)` / `HB_DEFINE_VTABLE (gpu_paint,
…)` block at the foot of the header is guarded by `defined(__cplusplus) &&
defined(HB_CPLUSPLUS_HH)` and defines nothing for C or Rust callers. It is not
transcribed.
