# Painting colour glyphs

Header: `hb-paint.h` — Rust module: `harfbuzz_sys::paint` (glob re-exported at
the crate root). Upstream gtk-doc section: `hb-paint`, short description
*"Glyph painting"*.

## Overview

An **`hb_paint_funcs_t`** is a rendering back end. You fill it with callbacks,
hand it to `hb_font_paint_glyph()` along with an opaque `paint_data` pointer of
your own, and HarfBuzz walks the glyph's paint tree and drives your callbacks
with transform, clip, fill, gradient, image, and compositing operations. Nothing
is returned and nothing is allocated on your behalf: the callbacks *are* the
output. The upstream section blurb states the purpose plainly — *"The main
purpose of these functions is to paint (extract) color glyph layers from the
COLRv1 table, but the API works for drawing ordinary outlines and images as
well."*

The model is a **push/pop stack machine**. The header is explicit: *"The
callbacks assume that the caller maintains a stack of current transforms, clips
and intermediate surfaces, as evidenced by the pairs of push/pop callbacks. The
push/pop calls will be properly nested, so it is fine to store the different
kinds of object on a single stack."* There are three stackable kinds and they
interleave freely:

- **Transforms** — `push_transform` / `pop_transform`. Each pushed matrix is
  applied *after* the current one, i.e. it composes onto the existing CTM, and
  stays in effect until its matching pop.
- **Clips** — `push_clip_glyph`, `push_clip_rectangle`, and the
  `push_clip_path_start`/`push_clip_path_end` pair, all unwound by `pop_clip`.
  A pushed clip intersects the current clip. One `pop_clip` matches exactly one
  push of any of the three flavours.
- **Groups** — `push_group` (or `push_group_for`) / `pop_group`. A group
  redirects all subsequent drawing to an intermediate surface; the matching pop
  stops the redirection and composites that surface onto the one below using an
  `hb_paint_composite_mode_t`.

Between the pushes come the leaf operations that actually put ink down:
`color` fills the entire current clip with a solid colour, the three gradient
callbacks fill it with a gradient, `image` draws an embedded PNG/SVG/BGRA
bitmap, and `fill_glyph` fills a glyph's outline with a solid colour in one
step. `color_glyph` is different in kind: it asks the back end whether it can
render a whole colour glyph by index itself, which lets a back end short-circuit
recursion into COLRv1 sub-glyphs (and, in practice, implement caching).

Not every callback is required. The header says so: *"For rendering COLRv0 or
non-color outline glyphs, the gradient callbacks are not needed, and the
composite callback only needs to handle simple alpha compositing
(`HB_PAINT_COMPOSITE_MODE_SRC_OVER`). The paint-image callback is only needed
for glyphs with image blobs in the `CBDT`, `sbix` or `SVG` tables. The
custom-palette-color callback is only necessary if you want to override colors
from the font palette with custom colors."* Any callback you leave unset gets a
built-in no-op stub, and two of the stubs are usefully non-trivial:
`fill_glyph`'s stub decomposes into `push_clip_glyph` → `color` → `pop_clip`, and
`push_group_for`'s stub calls `push_group`. That is what lets a back end written
against HarfBuzz 7 keep working when HarfBuzz 14 starts emitting the newer,
more specific operations.

The relationship to **COLRv1** is direct: the callback set is essentially a
one-to-one transcription of the OpenType `COLR` version 1 paint-graph node
types. `PaintSolid` becomes `color`, `PaintLinearGradient` /
`PaintRadialGradient` / `PaintSweepGradient` become the three gradient
callbacks, `PaintTransform` and friends become `push_transform`,
`PaintGlyph` becomes `push_clip_glyph`/`pop_clip` (or `fill_glyph`),
`PaintColrGlyph` becomes `color_glyph`, `PaintComposite` becomes
`push_group`/`pop_group`, and the `hb_paint_composite_mode_t` enumeration is the
COLRv1 `CompositeMode` list verbatim. HarfBuzz resolves variations, palette
indices, and alpha for you, so the values reaching your callbacks are already
concrete. The COLR specification is the normative reference for how the gradient
geometry and colour lines are to be interpreted, and the header points at it
repeatedly.

A `paint_data` pointer threads through every callback. It is whatever you passed
to `hb_font_paint_glyph()` and is where your back end keeps its cairo context,
its command list, its stack. It is entirely distinct from the per-callback
`user_data` you give each setter, and from the object's user-data table.

Finally, every callback has a **manual counterpart**: `hb_paint_color()`,
`hb_paint_push_transform()`, and so on invoke the corresponding callback
directly. Those exist so that intermediate layers — a back end that wraps
another back end, a font-funcs implementation that synthesises paint operations,
a test harness — can emit paint operations without going through a font.

## Types

### `hb_paint_funcs_t`

```c
typedef struct hb_paint_funcs_t hb_paint_funcs_t;
```

```rust
crate::opaque_handle! { hb_paint_funcs_t }
```

Glyph paint callbacks — an opaque, reference-counted object holding 18 callback
slots, each with its own `user_data` pointer and `destroy` notifier, plus the
usual object header (reference count, immutable flag, user-data table). Since
HarfBuzz 7.0.0.

You get one from `hb_paint_funcs_create()` (owned) or
`hb_paint_funcs_get_empty()` (borrowed singleton). In Rust it is a zero-sized
`#[repr(C)]` handle; you always hold `*mut hb_paint_funcs_t`.

The callback slots, in the order the implementation lists them, are:
`push_transform`, `pop_transform`, `color_glyph`, `push_clip_glyph`,
`push_clip_rectangle`, `push_clip_path_start`, `push_clip_path_end`, `pop_clip`,
`color`, `image`, `linear_gradient`, `radial_gradient`, `sweep_gradient`,
`push_group`, `push_group_for`, `pop_group`, `custom_palette_color`, and
`fill_glyph`.

### `hb_paint_extend_t`

How colour values outside a colour line's minimum and maximum defined offset are
determined. See the OpenType
[COLR](https://learn.microsoft.com/en-us/typography/opentype/spec/colr) section
for details. Since HarfBuzz 7.0.0.

In C it is an unnamed-value `enum` with three enumerators, no sentinel; the
largest is 2, so it fits in an `int` and is transcribed as
`pub type hb_paint_extend_t = core::ffi::c_int;` plus three constants.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_PAINT_EXTEND_PAD` | 0 | Outside the defined interval, the colour of the closest colour stop is used. |
| `HB_PAINT_EXTEND_REPEAT` | 1 | The colour line is repeated over repeated multiples of the defined interval. |
| `HB_PAINT_EXTEND_REFLECT` | 2 | The colour line is repeated over repeated intervals, as for the repeat mode; however, in each repeated interval the ordering of colour stops is the reverse of the adjacent interval. |

### `hb_paint_composite_mode_t`

The compositing modes that can be used when combining temporary redirected
drawing with the backdrop — that is, the mode passed to `pop_group`. See the
OpenType COLR section for details. Since HarfBuzz 7.0.0.

In C it is an unnamed-value `enum` with 28 enumerators, no sentinel; the largest
is 27, so it fits in an `int` and is transcribed as
`pub type hb_paint_composite_mode_t = core::ffi::c_int;` plus 28 constants.

Note that the numeric order is *not* the order the documentation block lists
them in, and it is not the COLRv1 wire order either: `DEST` is 2 and `SRC_OVER`
is 3, and `MULTIPLY` is 23, well after `SCREEN` at 13. Always use the constants.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_PAINT_COMPOSITE_MODE_CLEAR` | 0 | Clear the destination layer (bounded). |
| `HB_PAINT_COMPOSITE_MODE_SRC` | 1 | Replace the destination layer (bounded). |
| `HB_PAINT_COMPOSITE_MODE_DEST` | 2 | Ignore the source. |
| `HB_PAINT_COMPOSITE_MODE_SRC_OVER` | 3 | Draw the source layer on top of the destination layer (bounded). |
| `HB_PAINT_COMPOSITE_MODE_DEST_OVER` | 4 | Draw the destination on top of the source. |
| `HB_PAINT_COMPOSITE_MODE_SRC_IN` | 5 | Draw the source where there was destination content (unbounded). |
| `HB_PAINT_COMPOSITE_MODE_DEST_IN` | 6 | Leave the destination only where there was source content (unbounded). |
| `HB_PAINT_COMPOSITE_MODE_SRC_OUT` | 7 | Draw the source where there was no destination content (unbounded). |
| `HB_PAINT_COMPOSITE_MODE_DEST_OUT` | 8 | Leave the destination only where there was no source content. |
| `HB_PAINT_COMPOSITE_MODE_SRC_ATOP` | 9 | Draw the source on top of the destination content, and only there. |
| `HB_PAINT_COMPOSITE_MODE_DEST_ATOP` | 10 | Leave the destination on top of the source content, and only there (unbounded). |
| `HB_PAINT_COMPOSITE_MODE_XOR` | 11 | Source and destination are shown where there is only one of them. |
| `HB_PAINT_COMPOSITE_MODE_PLUS` | 12 | Source and destination layers are accumulated. |
| `HB_PAINT_COMPOSITE_MODE_SCREEN` | 13 | Source and destination are complemented and multiplied; the result is at least as light as the lighter inputs. |
| `HB_PAINT_COMPOSITE_MODE_OVERLAY` | 14 | Multiplies or screens, depending on the lightness of the destination colour. |
| `HB_PAINT_COMPOSITE_MODE_DARKEN` | 15 | Replaces the destination with the source if it is darker, otherwise keeps the source. |
| `HB_PAINT_COMPOSITE_MODE_LIGHTEN` | 16 | Replaces the destination with the source if it is lighter, otherwise keeps the source. |
| `HB_PAINT_COMPOSITE_MODE_COLOR_DODGE` | 17 | Brightens the destination colour to reflect the source colour. |
| `HB_PAINT_COMPOSITE_MODE_COLOR_BURN` | 18 | Darkens the destination colour to reflect the source colour. |
| `HB_PAINT_COMPOSITE_MODE_HARD_LIGHT` | 19 | Multiplies or screens, dependent on source colour. |
| `HB_PAINT_COMPOSITE_MODE_SOFT_LIGHT` | 20 | Darkens or lightens, dependent on source colour. |
| `HB_PAINT_COMPOSITE_MODE_DIFFERENCE` | 21 | Takes the difference of the source and destination colour. |
| `HB_PAINT_COMPOSITE_MODE_EXCLUSION` | 22 | Produces an effect similar to difference, but with lower contrast. |
| `HB_PAINT_COMPOSITE_MODE_MULTIPLY` | 23 | Source and destination layers are multiplied; the result is at least as dark as the darker inputs. |
| `HB_PAINT_COMPOSITE_MODE_HSL_HUE` | 24 | Creates a colour with the hue of the source and the saturation and luminosity of the target. |
| `HB_PAINT_COMPOSITE_MODE_HSL_SATURATION` | 25 | Creates a colour with the saturation of the source and the hue and luminosity of the target. Painting with this mode onto a grey area produces no change. |
| `HB_PAINT_COMPOSITE_MODE_HSL_COLOR` | 26 | Creates a colour with the hue and saturation of the source and the luminosity of the target. Preserves the grey levels of the target; useful for colouring monochrome images or tinting colour images. |
| `HB_PAINT_COMPOSITE_MODE_HSL_LUMINOSITY` | 27 | Creates a colour with the luminosity of the source and the hue and saturation of the target. The inverse effect of `HB_PAINT_COMPOSITE_MODE_HSL_COLOR`. |

A back end that only handles COLRv0 and plain outlines needs
`HB_PAINT_COMPOSITE_MODE_SRC_OVER` and nothing else.

### `hb_color_stop_t`

Information about a colour stop on a colour line. Since HarfBuzz 7.0.0.

```c
typedef struct {
  float      offset;
  hb_bool_t  is_foreground;
  hb_color_t color;
} hb_color_stop_t;
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct hb_color_stop_t {
    pub offset: c_float,
    pub is_foreground: hb_bool_t,
    pub color: hb_color_t,
}
```

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `offset` | `float` | `c_float` | The offset of the colour stop along the colour line. Typically in 0..1, but **that is not required** — the header says so explicitly. |
| `is_foreground` | `hb_bool_t` | `hb_bool_t` (`c_int`) | Whether the colour is the foreground; same semantics as in `hb_paint_color_func_t`. |
| `color` | `hb_color_t` | `hb_color_t` (`u32`) | The colour, **unpremultiplied**. |

Two things to get right. First, the header's own note: *"despite `color` being
unpremultiplied here, interpolation in gradients shall happen in premultiplied
space"* — see the OpenType COLR section. Second, `hb_color_t` is a packed
`uint32_t` whose channel order is given by `HB_COLOR(b,g,r,a)`; use
`hb_color_get_red()`, `hb_color_get_green()`, `hb_color_get_blue()`, and
`hb_color_get_alpha()` rather than shifting by hand.

### `hb_color_line_t`

A struct containing colour information for a gradient. Since HarfBuzz 7.0.0.

```c
struct hb_color_line_t {
  void *data;

  hb_color_line_get_color_stops_func_t get_color_stops;
  void *get_color_stops_user_data;

  hb_color_line_get_extend_func_t get_extend;
  void *get_extend_user_data;

  void *reserved0; void *reserved1; void *reserved2; void *reserved3;
  void *reserved5; void *reserved6; void *reserved7; void *reserved8;
};
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct hb_color_line_t {
    pub data: *mut c_void,
    pub get_color_stops: hb_color_line_get_color_stops_func_t,
    pub get_color_stops_user_data: *mut c_void,
    pub get_extend: hb_color_line_get_extend_func_t,
    pub get_extend_user_data: *mut c_void,
    pub reserved0: *mut c_void,
    pub reserved1: *mut c_void,
    pub reserved2: *mut c_void,
    pub reserved3: *mut c_void,
    pub reserved5: *mut c_void,
    pub reserved6: *mut c_void,
    pub reserved7: *mut c_void,
    pub reserved8: *mut c_void,
}
```

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `data` | `void *` | `*mut c_void` | The data accompanying this colour line; passed as `color_line_data` to both methods. |
| `get_color_stops` | `hb_color_line_get_color_stops_func_t` | same | The method that fetches colour stops. |
| `get_color_stops_user_data` | `void *` | `*mut c_void` | The `user_data` passed to `get_color_stops`. |
| `get_extend` | `hb_color_line_get_extend_func_t` | same | The method that fetches the extend mode. |
| `get_extend_user_data` | `void *` | `*mut c_void` | The `user_data` passed to `get_extend`. |
| `reserved0`–`reserved3`, `reserved5`–`reserved8` | `void *` | `*mut c_void` | Private padding that keeps the struct's size stable across releases. Do not read or write them. |

This is the one non-opaque struct in the header, but you are not meant to poke
at it. HarfBuzz constructs it **on the stack** and hands you a pointer for the
duration of a gradient callback; the three gradient typedefs all say the same
thing: *"It is only valid for the duration of the callback, you cannot keep it
around."* Read it through `hb_color_line_get_color_stops()` and
`hb_color_line_get_extend()`, which dispatch through the function pointers for
you.

Note the reserved fields are numbered 0–3 and then 5–8, skipping 4. That gap is
upstream's, and the Rust binding reproduces it so the field names match the C
header exactly.

### Image format tags

The `format` argument to `hb_paint_image_func_t` is an `hb_tag_t`. Three values
are defined; the header says "possible values include" these, so a back end
should fall through gracefully on anything else. All since HarfBuzz 7.0.0.

| Constant | Tag | Rust | Meaning |
| --- | --- | --- | --- |
| `HB_PAINT_IMAGE_FORMAT_PNG` | `png ` (note the trailing space) | `HB_TAG(b'p', b'n', b'g', b' ')` | PNG image data. |
| `HB_PAINT_IMAGE_FORMAT_SVG` | `svg ` (note the trailing space) | `HB_TAG(b's', b'v', b'g', b' ')` | SVG document. |
| `HB_PAINT_IMAGE_FORMAT_BGRA` | `BGRA` | `HB_TAG(b'B', b'G', b'R', b'A')` | Raw pixel data, in **BGRA pre-multiplied sRGBA** colour-space format. |

```c
#define HB_PAINT_IMAGE_FORMAT_PNG  HB_TAG('p','n','g',' ')
#define HB_PAINT_IMAGE_FORMAT_SVG  HB_TAG('s','v','g',' ')
#define HB_PAINT_IMAGE_FORMAT_BGRA HB_TAG('B','G','R','A')
```

```rust
pub const HB_PAINT_IMAGE_FORMAT_PNG: hb_tag_t = HB_TAG(b'p', b'n', b'g', b' ');
pub const HB_PAINT_IMAGE_FORMAT_SVG: hb_tag_t = HB_TAG(b's', b'v', b'g', b' ');
pub const HB_PAINT_IMAGE_FORMAT_BGRA: hb_tag_t = HB_TAG(b'B', b'G', b'R', b'A');
```

Note the inconsistency, which is upstream's and cannot be changed: the first two
are lowercase with a trailing space, the third is uppercase with none. `BGRA` is
also the only one whose pixel data is *pre-multiplied*, unlike every `hb_color_t`
in this header.

### Callback typedefs

Every paint callback receives the `hb_paint_funcs_t` it was installed on, the
`paint_data` pointer the caller passed to `hb_font_paint_glyph()`, its own
operation-specific arguments, and last the `user_data` pointer given to its
setter. In Rust each is `Option<unsafe extern "C" fn(...)>`, so `None` is the
null function pointer — which is what you pass to a setter to restore the
built-in stub.

#### `hb_paint_push_transform_func_t`

```c
typedef void (*hb_paint_push_transform_func_t) (hb_paint_funcs_t *funcs,
                                                void *paint_data,
                                                float xx, float yx,
                                                float xy, float yy,
                                                float dx, float dy,
                                                void *user_data);
```

```rust
pub type hb_paint_push_transform_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        xx: c_float, yx: c_float,
        xy: c_float, yy: c_float,
        dx: c_float, dy: c_float,
        user_data: *mut c_void,
    ),
>;
```

Applies a transform to subsequent paint calls. The six floats are the components
of a 2×3 affine matrix in the usual `(xx, yx, xy, yy, dx, dy)` order — the same
order cairo uses. *"This transform is applied after the current transform, and
remains in effect until a matching call to the `hb_paint_pop_transform_func_t`
vfunc."* Since HarfBuzz 7.0.0.

One implementation detail worth knowing: HarfBuzz normalises `-0.f` to `0.f` in
`dx` and `dy` before invoking the callback, so you never see a negative zero
translation.

#### `hb_paint_pop_transform_func_t`

```c
typedef void (*hb_paint_pop_transform_func_t) (hb_paint_funcs_t *funcs,
                                               void *paint_data,
                                               void *user_data);
```

```rust
pub type hb_paint_pop_transform_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        user_data: *mut c_void,
    ),
>;
```

Undoes the effect of a prior `hb_paint_push_transform_func_t` call. Since
HarfBuzz 7.0.0.

#### `hb_paint_color_glyph_func_t`

```c
typedef hb_bool_t (*hb_paint_color_glyph_func_t) (hb_paint_funcs_t *funcs,
                                                  void *paint_data,
                                                  hb_codepoint_t glyph,
                                                  hb_font_t *font,
                                                  void *user_data);
```

```rust
pub type hb_paint_color_glyph_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        glyph: hb_codepoint_t,
        font: *mut hb_font_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;
```

Renders a colour glyph by glyph index. Returns `true` if the glyph was painted,
`false` otherwise — and returning `false` is the normal thing to do: HarfBuzz
then expands the glyph's paint tree into the other callbacks itself. Returning
`true` is how a back end says "I have my own cached rendering for this glyph, do
not recurse". The built-in stub returns `false`. Since HarfBuzz 8.2.0.

#### `hb_paint_fill_glyph_func_t`

```c
typedef void (*hb_paint_fill_glyph_func_t) (hb_paint_funcs_t *funcs,
                                            void *paint_data,
                                            hb_codepoint_t glyph,
                                            hb_font_t *font,
                                            hb_bool_t is_foreground,
                                            hb_color_t color,
                                            void *user_data);
```

```rust
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
```

Fills a glyph's shape with a solid colour — the overwhelmingly common case,
fused into one call. *"If not implemented, a sequence of 'push-clip-glyph',
'color', 'pop-clip' paint operations, in that order, will be emitted instead."*
`color` is unpremultiplied; `is_foreground` has the same meaning as in
`hb_paint_color_func_t`. Since HarfBuzz 14.3.0.

Implementing this is a pure optimisation: it lets a back end call its own
"fill path with colour" primitive instead of building a clip, filling it, and
tearing it down.

#### `hb_paint_push_clip_glyph_func_t`

```c
typedef void (*hb_paint_push_clip_glyph_func_t) (hb_paint_funcs_t *funcs,
                                                 void *paint_data,
                                                 hb_codepoint_t glyph,
                                                 hb_font_t *font,
                                                 void *user_data);
```

```rust
pub type hb_paint_push_clip_glyph_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        glyph: hb_codepoint_t,
        font: *mut hb_font_t,
        user_data: *mut c_void,
    ),
>;
```

Clips subsequent paint calls to the outline of a glyph. *"The coordinates of the
glyph outline are expected in the current `font` scale (ie. the results of
calling `hb_font_draw_glyph()` with `font`). The outline is transformed by the
current transform. This clip is applied in addition to the current clip, and
remains in effect until a matching call to the `hb_paint_pop_clip_func_t`
vfunc."* Since HarfBuzz 7.0.0.

The practical reading: to get the outline, call `hb_font_draw_glyph(font, glyph,
your_draw_funcs, your_draw_data)` — see [Drawing glyph outlines](draw.md) — and
use the resulting path as a clip.

#### `hb_paint_push_clip_rectangle_func_t`

```c
typedef void (*hb_paint_push_clip_rectangle_func_t) (hb_paint_funcs_t *funcs,
                                                     void *paint_data,
                                                     float xmin, float ymin,
                                                     float xmax, float ymax,
                                                     void *user_data);
```

```rust
pub type hb_paint_push_clip_rectangle_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        xmin: c_float, ymin: c_float,
        xmax: c_float, ymax: c_float,
        user_data: *mut c_void,
    ),
>;
```

Clips subsequent paint calls to a rectangle. *"The coordinates of the rectangle
are interpreted according to the current transform."* Applied in addition to the
current clip; unwound by `hb_paint_pop_clip_func_t`. Since HarfBuzz 7.0.0.

Because the rectangle is transformed by the CTM, a rotated transform makes this
a rotated quadrilateral, not an axis-aligned rectangle in device space.

#### `hb_paint_push_clip_path_start_func_t`

```c
typedef hb_draw_funcs_t * (*hb_paint_push_clip_path_start_func_t)
    (hb_paint_funcs_t *funcs, void *paint_data,
     void **draw_data, void *user_data);
```

```rust
pub type hb_paint_push_clip_path_start_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        draw_data: *mut *mut c_void,
        user_data: *mut c_void,
    ) -> *mut hb_draw_funcs_t,
>;
```

Begins clipping to an arbitrary path. *"The backend returns an
`hb_draw_funcs_t` it owns (the caller must not free it) that the caller feeds
the clip outline to via `hb_draw_*()` calls, plus a `draw_data` value to pass
alongside those calls. Both are only valid until the matching
`hb_paint_push_clip_path_end_func_t` call; no other paint calls should be made
in between. The clip remains in effect until a later
`hb_paint_pop_clip_func_t` call."*

`draw_data` is an out-parameter: write the value the caller should pass to the
draw functions. Return value annotated `(transfer none)`: draw funcs that
accumulate the clip path, **or `NULL` if arbitrary-path clipping is not
supported**. Since HarfBuzz 14.2.0.

The built-in stub sets `*draw_data` to null (when `draw_data` is non-null) and
returns null, so a back end that does not implement this simply advertises no
support and HarfBuzz falls back.

#### `hb_paint_push_clip_path_end_func_t`

```c
typedef void (*hb_paint_push_clip_path_end_func_t) (hb_paint_funcs_t *funcs,
                                                    void *paint_data,
                                                    void *user_data);
```

```rust
pub type hb_paint_push_clip_path_end_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        user_data: *mut c_void,
    ),
>;
```

Closes the clip path started by `hb_paint_push_clip_path_start_func_t`. *"The
emitted path is now active as a clip; subsequent paint ops are masked by it
until a matching `hb_paint_pop_clip_func_t` call."* Since HarfBuzz 14.2.0.

#### `hb_paint_pop_clip_func_t`

```c
typedef void (*hb_paint_pop_clip_func_t) (hb_paint_funcs_t *funcs,
                                          void *paint_data,
                                          void *user_data);
```

```rust
pub type hb_paint_pop_clip_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        user_data: *mut c_void,
    ),
>;
```

Undoes the effect of a prior `push_clip_glyph`, `push_clip_rectangle`, or
`push_clip_path_end` call. One `pop_clip` per push, whichever flavour. Since
HarfBuzz 7.0.0.

#### `hb_paint_color_func_t`

```c
typedef void (*hb_paint_color_func_t) (hb_paint_funcs_t *funcs,
                                       void *paint_data,
                                       hb_bool_t is_foreground,
                                       hb_color_t color,
                                       void *user_data);
```

```rust
pub type hb_paint_color_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        is_foreground: hb_bool_t,
        color: hb_color_t,
        user_data: *mut c_void,
    ),
>;
```

Paints a colour **everywhere within the current clip**. Not a shape — a flood
fill of the clip region. Since HarfBuzz 7.0.0.

`is_foreground` deserves care, and the header spells the contract out:

> When `is_foreground` is true, this color originates from the foreground-color
> sentinel in the font's color data. The `color` parameter still carries a fully
> resolved RGBA value (with any paint-tree alpha already applied), so backends
> that do not need to distinguish the foreground can simply use `color`
> directly.
>
> Backends that defer foreground resolution (e.g. to honor a CSS `currentColor`
> or a runtime uniform) should substitute their own foreground RGB when
> `is_foreground` is true, but **must combine the alpha from `color` with their
> foreground alpha**, since it encodes additional modulation from the paint
> tree. For this mode to work correctly, the caller should pass a fully-opaque
> foreground color to `hb_font_paint_glyph()`, so that the alpha in `color`
> reflects only the paint-tree contribution.

So: ignore `is_foreground` and you get correct output for the colour you passed
in. Honour it and you must multiply alphas, and you must have passed an opaque
foreground.

#### `hb_paint_image_func_t`

```c
typedef hb_bool_t (*hb_paint_image_func_t) (hb_paint_funcs_t *funcs,
                                            void *paint_data,
                                            hb_blob_t *image,
                                            unsigned int width,
                                            unsigned int height,
                                            hb_tag_t format,
                                            float slant,
                                            hb_glyph_extents_t *extents,
                                            void *user_data);
```

```rust
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
```

Paints a glyph image. *"This method is called for glyphs with image blobs in the
`CBDT`, `sbix` or `SVG` tables."* Since HarfBuzz 7.0.0.

| Parameter | Meaning |
| --- | --- |
| `image` | The image data, as an `hb_blob_t`. Borrowed for the call; reference it if you need it longer. |
| `width`, `height` | Width and height of the raster image in pixels, **or 0** when not known (which is normal for SVG). |
| `format` | One of the three `HB_PAINT_IMAGE_FORMAT_*` tags, or something else — the header says "possible values include". |
| `slant` | **Deprecated. Always set to 0.0.** Ignore it. |
| `extents` | Annotated `(nullable)`: glyph extents for the desired rendering, or null when unavailable. |
| `user_data` | The pointer passed to `hb_paint_funcs_set_image_func()`. |

*"The image dimensions and glyph extents are provided if available, and should
be used to size and position the image."* Returns whether the operation was
successful; the built-in stub returns `false`.

#### `hb_color_line_get_color_stops_func_t`

```c
typedef unsigned int (*hb_color_line_get_color_stops_func_t)
    (hb_color_line_t *color_line, void *color_line_data,
     unsigned int start, unsigned int *count,
     hb_color_stop_t *color_stops, void *user_data);
```

```rust
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
```

A virtual method for `hb_color_line_t` to fetch colour stops. `start` is the
index of the first stop to return; `count` is `(inout) (optional)` — input is
the maximum number to return, output is the actual number returned (may be
zero); `color_stops` is `(out) (array length=count) (optional)`, the array to
populate. Returns the **total** number of colour stops in `color_line`,
regardless of how many were written. Since HarfBuzz 7.0.0.

You implement this only if you are *producing* colour lines. If you are a paint
back end consuming them, call `hb_color_line_get_color_stops()` instead.

#### `hb_color_line_get_extend_func_t`

```c
typedef hb_paint_extend_t (*hb_color_line_get_extend_func_t)
    (hb_color_line_t *color_line, void *color_line_data, void *user_data);
```

```rust
pub type hb_color_line_get_extend_func_t = Option<
    unsafe extern "C" fn(
        color_line: *mut hb_color_line_t,
        color_line_data: *mut c_void,
        user_data: *mut c_void,
    ) -> hb_paint_extend_t,
>;
```

A virtual method for `hb_color_line_t` to fetch the extend mode. Returns the
extend mode of `color_line`. Since HarfBuzz 7.0.0.

#### `hb_paint_linear_gradient_func_t`

```c
typedef void (*hb_paint_linear_gradient_func_t) (hb_paint_funcs_t *funcs,
                                                 void *paint_data,
                                                 hb_color_line_t *color_line,
                                                 float x0, float y0,
                                                 float x1, float y1,
                                                 float x2, float y2,
                                                 void *user_data);
```

```rust
pub type hb_paint_linear_gradient_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        color_line: *mut hb_color_line_t,
        x0: c_float, y0: c_float,
        x1: c_float, y1: c_float,
        x2: c_float, y2: c_float,
        user_data: *mut c_void,
    ),
>;
```

Paints a linear gradient everywhere within the current clip. Since HarfBuzz
7.0.0.

**Three** points, not two — this is COLRv1's three-anchor form: P0 = `(x0, y0)`
is colour stop 0, P1 = `(x1, y1)` is colour stop 1, and P2 = `(x2, y2)` is a
rotation reference. Most 2D APIs want a two-point axis; `hb_paint_reduce_linear_anchors()`
performs that reduction for you.

*"The `color_line` object contains information about the colors of the
gradients. It is only valid for the duration of the callback, you cannot keep it
around."* Coordinates are interpreted according to the current transform. See
the OpenType COLR section for how the points define the direction of the
gradient and how to interpret the colour line.

#### `hb_paint_radial_gradient_func_t`

```c
typedef void (*hb_paint_radial_gradient_func_t) (hb_paint_funcs_t *funcs,
                                                 void *paint_data,
                                                 hb_color_line_t *color_line,
                                                 float x0, float y0, float r0,
                                                 float x1, float y1, float r1,
                                                 void *user_data);
```

```rust
pub type hb_paint_radial_gradient_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        color_line: *mut hb_color_line_t,
        x0: c_float, y0: c_float, r0: c_float,
        x1: c_float, y1: c_float, r1: c_float,
        user_data: *mut c_void,
    ),
>;
```

Paints a radial gradient everywhere within the current clip: two circles, given
as centre and radius each — `(x0, y0, r0)` is the first, `(x1, y1, r1)` the
second. This is the same two-circle form cairo and SVG use. Same colour-line
lifetime rule and same transform interpretation as the linear case. Since
HarfBuzz 7.0.0.

#### `hb_paint_sweep_gradient_func_t`

```c
typedef void (*hb_paint_sweep_gradient_func_t) (hb_paint_funcs_t *funcs,
                                                void *paint_data,
                                                hb_color_line_t *color_line,
                                                float x0, float y0,
                                                float start_angle,
                                                float end_angle,
                                                void *user_data);
```

```rust
pub type hb_paint_sweep_gradient_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        color_line: *mut hb_color_line_t,
        x0: c_float, y0: c_float,
        start_angle: c_float,
        end_angle: c_float,
        user_data: *mut c_void,
    ),
>;
```

Paints a sweep (conic) gradient everywhere within the current clip: centre
`(x0, y0)`, sweeping from `start_angle` to `end_angle`, both **in radians**.
Same colour-line lifetime rule and transform interpretation as the other two.
Since HarfBuzz 7.0.0.

Sweep gradients are the one gradient kind most 2D APIs lack;
`hb_paint_sweep_gradient_tiles()` decomposes one into angular sectors a back end
can render with primitives it does have.

#### `hb_paint_push_group_func_t`

```c
typedef void (*hb_paint_push_group_func_t) (hb_paint_funcs_t *funcs,
                                            void *paint_data,
                                            void *user_data);
```

```rust
pub type hb_paint_push_group_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        user_data: *mut c_void,
    ),
>;
```

Uses an intermediate surface for subsequent paint calls. *"The drawing will be
redirected to an intermediate surface until a matching call to the
`hb_paint_funcs_pop_group_func_t` vfunc."* Since HarfBuzz 7.0.0.

#### `hb_paint_push_group_for_func_t`

```c
typedef void (*hb_paint_push_group_for_func_t) (hb_paint_funcs_t *funcs,
                                                void *paint_data,
                                                hb_paint_composite_mode_t mode,
                                                void *user_data);
```

```rust
pub type hb_paint_push_group_for_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        mode: hb_paint_composite_mode_t,
        user_data: *mut c_void,
    ),
>;
```

Like `hb_paint_push_group_func_t`, but the compositing mode that the matching
pop will use is announced **at push time**. *"By default this calls
`hb_paint_push_group_func_t`."* Since HarfBuzz 14.2.0.

Knowing the mode up front lets a back end pick a cheaper strategy — for
`SRC_OVER` it may not need a separate surface at all, and for a bounded mode it
can size the surface to the clip.

#### `hb_paint_pop_group_func_t`

```c
typedef void (*hb_paint_pop_group_func_t) (hb_paint_funcs_t *funcs,
                                           void *paint_data,
                                           hb_paint_composite_mode_t mode,
                                           void *user_data);
```

```rust
pub type hb_paint_pop_group_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        mode: hb_paint_composite_mode_t,
        user_data: *mut c_void,
    ),
>;
```

Undoes a prior push-group. *"This call stops the redirection to the intermediate
surface, and then composites it on the previous surface, using the compositing
mode passed to this call."* Since HarfBuzz 7.0.0.

When `push_group_for` was used, the `mode` here is the same one that was
announced there.

#### `hb_paint_custom_palette_color_func_t`

```c
typedef hb_bool_t (*hb_paint_custom_palette_color_func_t)
    (hb_paint_funcs_t *funcs, void *paint_data,
     unsigned int color_index, hb_color_t *color, void *user_data);
```

```rust
pub type hb_paint_custom_palette_color_func_t = Option<
    unsafe extern "C" fn(
        funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        color_index: c_uint,
        color: *mut hb_color_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;
```

Fetches a custom palette override colour for `color_index`; `color` is an
`(out)` parameter. *"Custom palette colors override colors from the font's
selected color palette. It is not necessary to override all palette entries;
return `false` for entries that should be taken from the font palette. This
function might be called multiple times, but the custom palette is expected to
remain unchanged for the duration of one `hb_font_paint_glyph()` call."*
Returns `true` if a custom colour is provided, `false` otherwise. Since HarfBuzz
7.0.0.

Note the direction of control: this is HarfBuzz *pulling* from you, so you never
see a palette index in any other callback — colours arrive already resolved.

#### `hb_paint_sweep_gradient_tile_func_t`

```c
typedef void (*hb_paint_sweep_gradient_tile_func_t) (float       a0,
                                                     hb_color_t  c0,
                                                     float       a1,
                                                     hb_color_t  c1,
                                                     void       *user_data);
```

```rust
pub type hb_paint_sweep_gradient_tile_func_t = Option<
    unsafe extern "C" fn(
        a0: c_float,
        c0: hb_color_t,
        a1: c_float,
        c1: hb_color_t,
        user_data: *mut c_void,
    ),
>;
```

Callback invoked once per `(a0, a1)` sector of a sweep gradient tiling — see
`hb_paint_sweep_gradient_tiles()`. `a0`/`a1` are the segment's start and end
angles in radians and `c0`/`c1` the colours at them; `user_data` is the pointer
passed to `hb_paint_sweep_gradient_tiles()`. Since HarfBuzz 14.2.0.

This is the odd one out: it takes no `funcs` and no `paint_data`, because it is
a helper for back ends rather than a member of the paint vtable.

## Functions

### Object lifecycle

#### `hb_paint_funcs_create`

```c
hb_paint_funcs_t *hb_paint_funcs_create (void);
```

```rust
pub fn hb_paint_funcs_create() -> *mut hb_paint_funcs_t;
```

Creates a new paint-functions object with a reference count of one. Every
callback starts out unset, which means the built-in stub is installed; fill in
the ones your back end implements with the `hb_paint_funcs_set_*_func()`
setters.

**Returns** — the new object. Following HarfBuzz's universal object convention
it does not return null on allocation failure; it returns the singleton empty
object, so the only way to detect failure is to compare against
`hb_paint_funcs_get_empty()`.

**Ownership** — the caller owns the returned reference and must release it with
`hb_paint_funcs_destroy()`.

**Notes** — Since HarfBuzz 7.0.0. Note there is no `parent` argument here; unlike
`hb_unicode_funcs_t`, paint funcs do not chain.

#### `hb_paint_funcs_get_empty`

```c
hb_paint_funcs_t *hb_paint_funcs_get_empty (void);
```

```rust
pub fn hb_paint_funcs_get_empty() -> *mut hb_paint_funcs_t;
```

Fetches the singleton empty paint-functions object. Every one of its callbacks
is the default stub and it is permanently immutable. Never null.

**Ownership** — treat it like any other object and pass it to
`hb_paint_funcs_destroy()` when done; HarfBuzz's shared null objects are inert,
so referencing and destroying them are cheap no-ops.

**Notes** — Since HarfBuzz 7.0.0. This is also what `hb_paint_funcs_create()`
returns on allocation failure. Painting with it is legal and draws nothing —
useful as a "measure only" sink, though `fill_glyph`'s stub will still fan out
into `push_clip_glyph`/`color`/`pop_clip`, all of which are also no-ops.

#### `hb_paint_funcs_reference`

```c
hb_paint_funcs_t *hb_paint_funcs_reference (hb_paint_funcs_t *funcs);
```

```rust
pub fn hb_paint_funcs_reference(funcs: *mut hb_paint_funcs_t) -> *mut hb_paint_funcs_t;
```

Increases the reference count on a paint-functions object and returns the same
pointer. Every call must be matched by a `hb_paint_funcs_destroy()`. Since
HarfBuzz 7.0.0.

#### `hb_paint_funcs_destroy`

```c
void hb_paint_funcs_destroy (hb_paint_funcs_t *funcs);
```

```rust
pub fn hb_paint_funcs_destroy(funcs: *mut hb_paint_funcs_t);
```

Decreases the reference count on a paint-functions object. When the count
reaches zero the object and all associated resources are freed, which includes
calling the `destroy` callback registered alongside each of its painting
callbacks on that callback's `user_data`, and clearing the user-data table.

**Returns** — nothing; there is no way to observe whether the object actually
went away. Since HarfBuzz 7.0.0.

### User data

#### `hb_paint_funcs_set_user_data`

```c
hb_bool_t hb_paint_funcs_set_user_data (hb_paint_funcs_t *funcs,
                                        hb_user_data_key_t *key,
                                        void *              data,
                                        hb_destroy_func_t   destroy,
                                        hb_bool_t           replace);
```

```rust
pub fn hb_paint_funcs_set_user_data(
    funcs: *mut hb_paint_funcs_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

Attaches a key/data pair to the object. HarfBuzz uses the *address* of `key`, not
its contents, so the key object must outlive the paint funcs — a `static` is the
usual choice. `destroy` may be null; when non-null it is called with `data` when
the object is destroyed or the entry is replaced. `replace` selects whether an
existing entry stored under the same key is overwritten.

**Returns** — true on success, false otherwise (allocation failure, or a
non-replace call against an existing key). Since HarfBuzz 7.0.0.

This is a third, independent channel from `paint_data` and from the
per-callback `user_data`.

#### `hb_paint_funcs_get_user_data`

```c
void *hb_paint_funcs_get_user_data (const hb_paint_funcs_t *funcs,
                                    hb_user_data_key_t     *key);
```

```rust
pub fn hb_paint_funcs_get_user_data(
    funcs: *const hb_paint_funcs_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

Fetches the data previously attached under `key`. Note the `const` object
parameter. Ownership stays with the object; the caller must not free the
returned pointer. Returns null when no entry is present. Since HarfBuzz 7.0.0.

### Immutability

#### `hb_paint_funcs_make_immutable`

```c
void hb_paint_funcs_make_immutable (hb_paint_funcs_t *funcs);
```

```rust
pub fn hb_paint_funcs_make_immutable(funcs: *mut hb_paint_funcs_t);
```

Makes a paint-functions object immutable. **One-way** — there is no
`make_mutable`. After this, the `hb_paint_funcs_set_*_func()` setters silently
do nothing; they still call the `destroy` callback on the `user_data` they were
handed, so nothing leaks, but they report nothing. Since HarfBuzz 7.0.0.

Freeze the object once it is fully configured and before sharing it across
threads.

#### `hb_paint_funcs_is_immutable`

```c
hb_bool_t hb_paint_funcs_is_immutable (hb_paint_funcs_t *funcs);
```

```rust
pub fn hb_paint_funcs_is_immutable(funcs: *mut hb_paint_funcs_t) -> hb_bool_t;
```

Tests whether a paint-functions object is immutable; true if it is. Since
HarfBuzz 7.0.0. Because the setters return `void`, this is the only way to know
in advance whether an install will take effect.

### Installing callbacks

All eighteen setters have the identical shape, ownership contract, and failure
mode:

```c
void hb_paint_funcs_set_<name>_func (hb_paint_funcs_t      *funcs,
                                     hb_paint_<name>_func_t func,
                                     void                  *user_data,
                                     hb_destroy_func_t      destroy);
```

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `funcs` | The object to modify. Nullability unspecified in the header; the implementation dereferences it, so treat null as forbidden. |
| `func` | The callback. Upstream annotates it `(closure user_data) (destroy destroy) (scope notified)`. Passing null (`None` in Rust) **restores the built-in stub** for that slot. |
| `user_data` | Opaque pointer handed back to `func` on every invocation. |
| `destroy` | Annotated `(nullable)`. Called with `user_data` when the callback is replaced or the object is destroyed. |

**Returns** — nothing. No success or failure signal.

**Ownership** — the callee takes ownership of `user_data` unconditionally, on
every path. Concretely, the shared preamble does this:

- If `funcs` is immutable: run `destroy(user_data)` immediately and return
  without changing anything.
- If `func` is null: run `destroy(user_data)` immediately, then clear both
  `user_data` and `destroy` for that slot and install the built-in stub.
- Otherwise: release the previous `destroy`/`user_data` pair for that slot,
  install the new callback, and take ownership of the new pair.

There is also an allocation step (the per-slot `user_data` and `destroy` tables
are allocated lazily on first use); if it fails, the setter runs
`destroy(user_data)` and returns having changed nothing. So a Rust wrapper that
leaks a `Box` into `user_data` before the call is correct in every case, and one
that frees the `Box` afterwards is a double free.

The eighteen setters, with their since-versions:

| Setter | Installs | Since |
| --- | --- | ---: |
| `hb_paint_funcs_set_push_transform_func` | `hb_paint_push_transform_func_t` | 7.0.0 |
| `hb_paint_funcs_set_pop_transform_func` | `hb_paint_pop_transform_func_t` | 7.0.0 |
| `hb_paint_funcs_set_color_glyph_func` | `hb_paint_color_glyph_func_t` | 8.2.0 |
| `hb_paint_funcs_set_fill_glyph_func` | `hb_paint_fill_glyph_func_t` | 14.3.0 |
| `hb_paint_funcs_set_push_clip_glyph_func` | `hb_paint_push_clip_glyph_func_t` | 7.0.0 |
| `hb_paint_funcs_set_push_clip_rectangle_func` | `hb_paint_push_clip_rectangle_func_t` | 7.0.0 |
| `hb_paint_funcs_set_push_clip_path_start_func` | `hb_paint_push_clip_path_start_func_t` | 14.2.0 |
| `hb_paint_funcs_set_push_clip_path_end_func` | `hb_paint_push_clip_path_end_func_t` | 14.2.0 |
| `hb_paint_funcs_set_pop_clip_func` | `hb_paint_pop_clip_func_t` | 7.0.0 |
| `hb_paint_funcs_set_color_func` | `hb_paint_color_func_t` | 7.0.0 |
| `hb_paint_funcs_set_image_func` | `hb_paint_image_func_t` | 7.0.0 |
| `hb_paint_funcs_set_linear_gradient_func` | `hb_paint_linear_gradient_func_t` | 7.0.0 |
| `hb_paint_funcs_set_radial_gradient_func` | `hb_paint_radial_gradient_func_t` | 7.0.0 |
| `hb_paint_funcs_set_sweep_gradient_func` | `hb_paint_sweep_gradient_func_t` | 7.0.0 |
| `hb_paint_funcs_set_push_group_func` | `hb_paint_push_group_func_t` | 7.0.0 |
| `hb_paint_funcs_set_push_group_for_func` | `hb_paint_push_group_for_func_t` | 14.2.0 |
| `hb_paint_funcs_set_pop_group_func` | `hb_paint_pop_group_func_t` | 7.0.0 |
| `hb_paint_funcs_set_custom_palette_color_func` | `hb_paint_custom_palette_color_func_t` | 7.0.0 |

Their Rust signatures follow the same pattern throughout, for example:

```rust
pub fn hb_paint_funcs_set_color_func(
    funcs: *mut hb_paint_funcs_t,
    func: hb_paint_color_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);

pub fn hb_paint_funcs_set_fill_glyph_func(
    funcs: *mut hb_paint_funcs_t,
    func: hb_paint_fill_glyph_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);

pub fn hb_paint_funcs_set_pop_group_func(
    funcs: *mut hb_paint_funcs_t,
    func: hb_paint_pop_group_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

### Reading a colour line

These two are what a *consumer* of gradients calls, from inside a gradient
callback, on the `hb_color_line_t` it was handed.

#### `hb_color_line_get_color_stops`

```c
unsigned int hb_color_line_get_color_stops (hb_color_line_t *color_line,
                                            unsigned int     start,
                                            unsigned int    *count,
                                            hb_color_stop_t *color_stops);
```

```rust
pub fn hb_color_line_get_color_stops(
    color_line: *mut hb_color_line_t,
    start: c_uint,
    count: *mut c_uint,
    color_stops: *mut hb_color_stop_t,
) -> c_uint;
```

Fetches a list of colour stops from the given colour line object.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `color_line` | The colour line handed to your gradient callback. Nullability unspecified; the implementation dereferences it. |
| `start` | Index of the first colour stop to return. |
| `count` | `(inout) (optional)`. On input, the capacity of `color_stops`; on output, how many entries were actually written (may be zero). **May be null**, in which case nothing is written. |
| `color_stops` | `(out) (array length=count) (optional)`. Array to populate. **May be null** — pass null (or a null `count`) to query only the total. |

**Returns** — the **total** number of colour stops in `color_line`, independent
of `start` and `count`. That is the number to size a buffer against.

**Ownership** — nothing is transferred; the stops are copied into your array.

**Notes** — Since HarfBuzz 7.0.0.

> **The stops may be out of order.** The upstream documentation says: *"Note
> that due to variations being applied, the returned color stops may be out of
> order. It is the callers responsibility to ensure that color stops are sorted
> by their offset before they are used."* `hb_paint_normalize_color_line()` will
> sort them for you.

#### `hb_color_line_get_extend`

```c
hb_paint_extend_t hb_color_line_get_extend (hb_color_line_t *color_line);
```

```rust
pub fn hb_color_line_get_extend(color_line: *mut hb_color_line_t) -> hb_paint_extend_t;
```

Fetches the extend mode of a colour line. Returns one of the three
`HB_PAINT_EXTEND_*` values; there is no error return. Since HarfBuzz 7.0.0.

### The manual API

Each of these invokes the corresponding callback on `funcs` with `paint_data`,
exactly as HarfBuzz would. They exist so that code sitting between a font and a
back end — a wrapping back end, a custom `hb_font_funcs_t` paint implementation,
a test harness — can emit paint operations directly. If the slot is unset, the
built-in stub runs, so none of these can fail for lack of a callback.

None of them documents nullability for `funcs` or `paint_data`; the
implementation dereferences `funcs` immediately, so treat null as forbidden
there, while `paint_data` is opaque and passed straight through.

#### `hb_paint_push_transform`

```c
void hb_paint_push_transform (hb_paint_funcs_t *funcs, void *paint_data,
                              float xx, float yx,
                              float xy, float yy,
                              float dx, float dy);
```

```rust
pub fn hb_paint_push_transform(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    xx: c_float, yx: c_float,
    xy: c_float, yy: c_float,
    dx: c_float, dy: c_float,
);
```

Performs a "push-transform" paint operation. Since HarfBuzz 7.0.0. `-0.f` in
`dx`/`dy` is normalised to `0.f` before the callback sees it.

#### `hb_paint_push_font_transform`

```c
void hb_paint_push_font_transform (hb_paint_funcs_t *funcs, void *paint_data,
                                   const hb_font_t *font);
```

```rust
pub fn hb_paint_push_font_transform(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    font: *const hb_font_t,
);
```

Pushes the transform reflecting the font's **scale and slant** settings onto the
paint functions — i.e. the matrix that maps font-unit coordinates into `font`'s
scaled space. Note the `const hb_font_t *`, unusual in this header. Since
HarfBuzz 11.0.0 per upstream's `hb-paint.cc`.

> The Rust module currently annotates this as "Since HarfBuzz 7.0.0". Upstream's
> gtk-doc comment says 11.0.0, and `NEWS` lists `+hb_paint_push_font_transform()`
> among the 11.0.0 additions; 11.0.0 is the value to trust.

#### `hb_paint_push_inverse_font_transform`

```c
void hb_paint_push_inverse_font_transform (hb_paint_funcs_t *funcs, void *paint_data,
                                           const hb_font_t *font);
```

```rust
pub fn hb_paint_push_inverse_font_transform(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    font: *const hb_font_t,
);
```

Pushes the **inverse** of the transform reflecting the font's scale and slant
settings, mapping `font`'s scaled space back to font units. Since HarfBuzz
11.0.0 per upstream's `hb-paint.cc`.

> The Rust module currently annotates this as "Since HarfBuzz 8.2.0"; upstream's
> gtk-doc comment says 11.0.0.

The pair is what lets a back end move between the two coordinate systems the
paint tree mixes: `push_clip_glyph` outlines arrive in the font's scaled space,
while some COLRv1 geometry is specified in font units.

#### `hb_paint_pop_transform`

```c
void hb_paint_pop_transform (hb_paint_funcs_t *funcs, void *paint_data);
```

```rust
pub fn hb_paint_pop_transform(funcs: *mut hb_paint_funcs_t, paint_data: *mut c_void);
```

Performs a "pop-transform" paint operation. Since HarfBuzz 7.0.0. One pop per
push, including pushes made by the two font-transform helpers.

#### `hb_paint_color_glyph`

```c
hb_bool_t hb_paint_color_glyph (hb_paint_funcs_t *funcs, void *paint_data,
                                hb_codepoint_t glyph, hb_font_t *font);
```

```rust
pub fn hb_paint_color_glyph(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    glyph: hb_codepoint_t,
    font: *mut hb_font_t,
) -> hb_bool_t;
```

Invokes the colour-glyph callback directly. Returns true if the glyph was
painted, false otherwise — false being what the built-in stub returns. Since
HarfBuzz 8.2.0.

#### `hb_paint_fill_glyph`

```c
void hb_paint_fill_glyph (hb_paint_funcs_t *funcs, void *paint_data,
                          hb_codepoint_t glyph, hb_font_t *font,
                          hb_bool_t is_foreground, hb_color_t color);
```

```rust
pub fn hb_paint_fill_glyph(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    glyph: hb_codepoint_t,
    font: *mut hb_font_t,
    is_foreground: hb_bool_t,
    color: hb_color_t,
);
```

Invokes the fill-glyph callback directly. If the slot is unset, the stub emits
`push_clip_glyph` → `color` → `pop_clip` against the same `funcs`, so those three
callbacks fire instead. Since HarfBuzz 14.3.0.

#### `hb_paint_push_clip_glyph`

```c
void hb_paint_push_clip_glyph (hb_paint_funcs_t *funcs, void *paint_data,
                               hb_codepoint_t glyph, hb_font_t *font);
```

```rust
pub fn hb_paint_push_clip_glyph(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    glyph: hb_codepoint_t,
    font: *mut hb_font_t,
);
```

Invokes the push-clip-glyph callback directly. Since HarfBuzz 7.0.0.

#### `hb_paint_push_clip_rectangle`

```c
void hb_paint_push_clip_rectangle (hb_paint_funcs_t *funcs, void *paint_data,
                                   float xmin, float ymin,
                                   float xmax, float ymax);
```

```rust
pub fn hb_paint_push_clip_rectangle(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    xmin: c_float, ymin: c_float,
    xmax: c_float, ymax: c_float,
);
```

Invokes the push-clip-rectangle callback directly. Since HarfBuzz 7.0.0.

#### `hb_paint_push_clip_path_start`

```c
hb_draw_funcs_t *hb_paint_push_clip_path_start (hb_paint_funcs_t  *funcs,
                                                void              *paint_data,
                                                void             **draw_data);
```

```rust
pub fn hb_paint_push_clip_path_start(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    draw_data: *mut *mut c_void,
) -> *mut hb_draw_funcs_t;
```

Invokes the push-clip-path-start callback directly.

**Returns** — the draw funcs the back end wants the clip outline fed to, along
with a `draw_data` value written through the out-parameter. **Returns null when
arbitrary-path clipping is not supported**, which is what the built-in stub
does (it also writes null to `*draw_data` when `draw_data` is non-null). Always
check for null before using the result.

**Ownership** — `(transfer none)`: the back end retains ownership of the draw
funcs. Do not destroy them. Both the draw funcs and the `draw_data` are valid
only until the matching `hb_paint_push_clip_path_end()`.

**Notes** — Since HarfBuzz 14.2.0. Between start and end you must issue only
`hb_draw_*()` calls; no other paint calls.

#### `hb_paint_push_clip_path_end`

```c
void hb_paint_push_clip_path_end (hb_paint_funcs_t *funcs, void *paint_data);
```

```rust
pub fn hb_paint_push_clip_path_end(funcs: *mut hb_paint_funcs_t, paint_data: *mut c_void);
```

Invokes the push-clip-path-end callback directly. After this the emitted path is
an active clip, and it is unwound by `hb_paint_pop_clip()`, not by anything
named "path". Since HarfBuzz 14.2.0.

#### `hb_paint_pop_clip`

```c
void hb_paint_pop_clip (hb_paint_funcs_t *funcs, void *paint_data);
```

```rust
pub fn hb_paint_pop_clip(funcs: *mut hb_paint_funcs_t, paint_data: *mut c_void);
```

Invokes the pop-clip callback directly. Since HarfBuzz 7.0.0.

#### `hb_paint_color`

```c
void hb_paint_color (hb_paint_funcs_t *funcs, void *paint_data,
                     hb_bool_t is_foreground, hb_color_t color);
```

```rust
pub fn hb_paint_color(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    is_foreground: hb_bool_t,
    color: hb_color_t,
);
```

Invokes the paint-colour callback directly. Since HarfBuzz 7.0.0.

#### `hb_paint_image`

```c
void hb_paint_image (hb_paint_funcs_t *funcs, void *paint_data,
                     hb_blob_t *image,
                     unsigned int width, unsigned int height,
                     hb_tag_t format, float slant,
                     hb_glyph_extents_t *extents);
```

```rust
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
```

Invokes the paint-image callback directly. Note this wrapper returns `void`
although the callback returns `hb_bool_t` — the success flag is discarded. Pass
`0.0` for the deprecated `slant`, and null for `extents` when you have none.
Since HarfBuzz 7.0.0.

#### `hb_paint_linear_gradient`

```c
void hb_paint_linear_gradient (hb_paint_funcs_t *funcs, void *paint_data,
                               hb_color_line_t *color_line,
                               float x0, float y0,
                               float x1, float y1,
                               float x2, float y2);
```

```rust
pub fn hb_paint_linear_gradient(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    color_line: *mut hb_color_line_t,
    x0: c_float, y0: c_float,
    x1: c_float, y1: c_float,
    x2: c_float, y2: c_float,
);
```

Invokes the linear-gradient callback directly. You supply the `hb_color_line_t`,
which means filling in `data`, `get_color_stops`, and `get_extend` yourself; it
need only stay alive for the duration of the call. Since HarfBuzz 7.0.0.

#### `hb_paint_radial_gradient`

```c
void hb_paint_radial_gradient (hb_paint_funcs_t *funcs, void *paint_data,
                               hb_color_line_t *color_line,
                               float x0, float y0, float r0,
                               float x1, float y1, float r1);
```

```rust
pub fn hb_paint_radial_gradient(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    color_line: *mut hb_color_line_t,
    x0: c_float, y0: c_float, r0: c_float,
    x1: c_float, y1: c_float, r1: c_float,
);
```

Invokes the radial-gradient callback directly. Since HarfBuzz 7.0.0.

#### `hb_paint_sweep_gradient`

```c
void hb_paint_sweep_gradient (hb_paint_funcs_t *funcs, void *paint_data,
                              hb_color_line_t *color_line,
                              float x0, float y0,
                              float start_angle, float end_angle);
```

```rust
pub fn hb_paint_sweep_gradient(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    color_line: *mut hb_color_line_t,
    x0: c_float, y0: c_float,
    start_angle: c_float, end_angle: c_float,
);
```

Invokes the sweep-gradient callback directly. Angles in radians. Since HarfBuzz
7.0.0.

#### `hb_paint_push_group`

```c
void hb_paint_push_group (hb_paint_funcs_t *funcs, void *paint_data);
```

```rust
pub fn hb_paint_push_group(funcs: *mut hb_paint_funcs_t, paint_data: *mut c_void);
```

Invokes the push-group callback directly. Since HarfBuzz 7.0.0.

#### `hb_paint_push_group_for`

```c
void hb_paint_push_group_for (hb_paint_funcs_t *funcs, void *paint_data,
                              hb_paint_composite_mode_t mode);
```

```rust
pub fn hb_paint_push_group_for(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    mode: hb_paint_composite_mode_t,
);
```

Invokes the push-group-for callback directly, announcing the compositing mode
the matching pop will use. If that slot is unset, the stub calls
`hb_paint_push_group()` on the same `funcs`, so the plain push-group callback
fires instead and the mode is simply not announced early. Since HarfBuzz 14.2.0.

#### `hb_paint_pop_group`

```c
void hb_paint_pop_group (hb_paint_funcs_t *funcs, void *paint_data,
                         hb_paint_composite_mode_t mode);
```

```rust
pub fn hb_paint_pop_group(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    mode: hb_paint_composite_mode_t,
);
```

Invokes the pop-group callback directly. Since HarfBuzz 7.0.0.

#### `hb_paint_custom_palette_color`

```c
hb_bool_t hb_paint_custom_palette_color (hb_paint_funcs_t *funcs, void *paint_data,
                                         unsigned int color_index,
                                         hb_color_t *color);
```

```rust
pub fn hb_paint_custom_palette_color(
    funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    color_index: c_uint,
    color: *mut hb_color_t,
) -> hb_bool_t;
```

Invokes the custom-palette-colour callback directly. Returns true if a custom
colour was written to `color`, false if the entry should be taken from the font
palette instead — which is what the built-in stub always returns. `*color` is
only meaningful on true. Since HarfBuzz 7.0.0.

### Gradient helpers for paint back ends

Upstream describes these as *"small, self-contained utilities that every COLRv1
renderer ends up reinventing. Exposed here so third-party paint backends can
consume a single canonical implementation instead of forking one per project."*
They are pure functions — no objects, no allocation, no callbacks into the paint
vtable.

#### `hb_paint_reduce_linear_anchors`

```c
void hb_paint_reduce_linear_anchors (float x0, float y0,
                                     float x1, float y1,
                                     float x2, float y2,
                                     float *xx0, float *yy0,
                                     float *xx1, float *yy1);
```

```rust
pub fn hb_paint_reduce_linear_anchors(
    x0: c_float, y0: c_float,
    x1: c_float, y1: c_float,
    x2: c_float, y2: c_float,
    xx0: *mut c_float, yy0: *mut c_float,
    xx1: *mut c_float, yy1: *mut c_float,
);
```

Reduces a COLRv1 linear gradient's 3-anchor spec (P0 = colour stop 0, P1 =
colour stop 1, P2 = rotation reference) to the 2-point axis (P0, P1′) used by
SVG, cairo, and most software renderers.

**Parameters** — `(x0, y0)`, `(x1, y1)`, `(x2, y2)` are the three anchors as
handed to `hb_paint_linear_gradient_func_t`. `xx0`/`yy0`/`xx1`/`yy1` are all
`(out)` and receive the resulting axis start and end.

**Returns** — nothing. *"P1′ is the foot of P1 on the line through P0
perpendicular to (P2 − P0); the resulting axis is the gradient's actual
direction (perpendicular to the rotation line). Degenerate (P0 == P2) passes
through unchanged"* — in the degenerate case `(xx0, yy0)` is `(x0, y0)` and
`(xx1, yy1)` is `(x1, y1)`.

**Ownership** — none; scalars in, scalars out through pointers you own.

**Notes** — Since HarfBuzz 14.2.0. The out-parameters are not marked nullable
and are written unconditionally, so pass four real pointers.

#### `hb_paint_normalize_color_line`

```c
void hb_paint_normalize_color_line (hb_color_stop_t *stops,
                                    unsigned int     len,
                                    float           *min,
                                    float           *max);
```

```rust
pub fn hb_paint_normalize_color_line(
    stops: *mut hb_color_stop_t,
    len: c_uint,
    min: *mut c_float,
    max: *mut c_float,
);
```

Sorts `stops` by offset and rescales the offsets into 0..1 **in place**.

**Parameters** — `stops` is `(array length=len) (inout)`: your own array of
colour stops, rewritten by the call. `min` and `max` are `(out)` and receive the
*original* minimum and maximum offsets.

**Returns** — nothing. *"Writes the original (min, max) to `min`/`max` so the
caller can shift the gradient geometry (axis endpoints for linear,
centers+radii for radial, start+end angles for sweep) to keep the rendered
gradient visually unchanged after the rescale. Empty input is safe: both
out-parameters set to 0."* Note that when every stop shares one offset
(`min == max`) the offsets are left as they are rather than divided by zero.

**Ownership** — the array stays yours; nothing is allocated.

**Notes** — Since HarfBuzz 14.2.0. This is the canonical fix for the
out-of-order stops that `hb_color_line_get_color_stops()` warns about, and for
colour lines whose offsets fall outside 0..1 (which `hb_color_stop_t` explicitly
permits). The out-parameters are written unconditionally — including in the
`len == 0` case — so they must not be null.

#### `hb_paint_sweep_gradient_tiles`

```c
void hb_paint_sweep_gradient_tiles (hb_color_stop_t                     *stops,
                                    unsigned int                         n_stops,
                                    hb_paint_extend_t                    extend,
                                    float                                start_angle,
                                    float                                end_angle,
                                    hb_paint_sweep_gradient_tile_func_t  emit_patch,
                                    void                                *user_data);
```

```rust
pub fn hb_paint_sweep_gradient_tiles(
    stops: *mut hb_color_stop_t,
    n_stops: c_uint,
    extend: hb_paint_extend_t,
    start_angle: c_float,
    end_angle: c_float,
    emit_patch: hb_paint_sweep_gradient_tile_func_t,
    user_data: *mut c_void,
);
```

Iterates the full 0..2π sweep produced by a colour-stop list, invoking
`emit_patch` once per (start, end) angular segment. Handles
`HB_PAINT_EXTEND_PAD`, `HB_PAINT_EXTEND_REPEAT`, and `HB_PAINT_EXTEND_REFLECT`.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `stops` | `(array length=n_stops) (inout)`. **Stops must be pre-sorted by offset, with offsets in 0..1** — *"use `hb_paint_normalize_color_line()` first if they aren't."* Marked `inout` because the function may reverse the array in place when `end_angle < start_angle`. |
| `n_stops` | Number of stops. Zero returns immediately, emitting nothing. |
| `extend` | The colour line's extend mode. |
| `start_angle`, `end_angle` | The sweep bounds, in radians. If `end_angle < start_angle` they are swapped and the stops reversed. If they are equal, only the pad-mode filler sectors (if any) are emitted. |
| `emit_patch` | `(scope call)`: invoked once per sector. Not called at all when `n_stops` is zero. |
| `user_data` | Passed through to `emit_patch`. |

**Returns** — nothing; the output is the sequence of `emit_patch` calls. Each
gives you a sector's bounding angles and the colours at them, which a back end
can render as a flat patch or a small linear gradient.

**Ownership** — none is transferred. The `stops` array stays yours, but **it may
be mutated**, so pass a copy if you need the original.

**Notes** — Since HarfBuzz 14.2.0. `emit_patch`'s scope is `call`, so it is not
retained after the function returns and needs no destroy notifier.

## Usage

### The entry points

`hb-paint.h` defines no function that paints a glyph; that lives in
`hb-font.h`:

```c
/* Paints a color glyph; if that fails, draws the outline glyph instead. */
void hb_font_paint_glyph (hb_font_t *font, hb_codepoint_t glyph,
                          hb_paint_funcs_t *pfuncs, void *paint_data,
                          unsigned int palette_index, hb_color_t foreground);

/* Paints a color glyph, or reports failure. */
hb_bool_t hb_font_paint_glyph_or_fail (hb_font_t *font, hb_codepoint_t glyph,
                                       hb_paint_funcs_t *pfuncs, void *paint_data,
                                       unsigned int palette_index,
                                       hb_color_t foreground);
```

`palette_index` selects a `CPAL` palette (0 is the default palette), and
`foreground` is the colour used for the foreground-colour sentinel — pass a
fully opaque colour if your `color` callback honours `is_foreground`. Use the
`_or_fail` variant when you want to know whether the glyph actually had colour
data; the plain one silently falls back to the monochrome outline.

### A minimal back end in C

```c
#include <hb.h>
#include <stdio.h>

typedef struct { int depth; } my_data_t;

static void
my_push_transform (hb_paint_funcs_t *funcs, void *paint_data,
                   float xx, float yx, float xy, float yy, float dx, float dy,
                   void *user_data)
{
  my_data_t *d = paint_data;
  printf ("%*spush-transform %g %g %g %g %g %g\n",
          d->depth * 2, "", xx, yx, xy, yy, dx, dy);
  d->depth++;
}

static void
my_pop_transform (hb_paint_funcs_t *funcs, void *paint_data, void *user_data)
{
  my_data_t *d = paint_data;
  d->depth--;
  printf ("%*spop-transform\n", d->depth * 2, "");
}

static void
my_color (hb_paint_funcs_t *funcs, void *paint_data,
          hb_bool_t is_foreground, hb_color_t color, void *user_data)
{
  my_data_t *d = paint_data;
  printf ("%*scolor #%02x%02x%02x%02x%s\n", d->depth * 2, "",
          hb_color_get_red (color), hb_color_get_green (color),
          hb_color_get_blue (color), hb_color_get_alpha (color),
          is_foreground ? " (foreground)" : "");
}

int main (void)
{
  hb_paint_funcs_t *pfuncs = hb_paint_funcs_create ();

  hb_paint_funcs_set_push_transform_func (pfuncs, my_push_transform, NULL, NULL);
  hb_paint_funcs_set_pop_transform_func  (pfuncs, my_pop_transform,  NULL, NULL);
  hb_paint_funcs_set_color_func          (pfuncs, my_color,          NULL, NULL);

  hb_paint_funcs_make_immutable (pfuncs);

  hb_face_t *face = hb_face_create_from_file_or_fail ("font.ttf", 0);
  hb_font_t *font = hb_font_create (face);

  my_data_t data = { 0 };
  hb_font_paint_glyph (font, 42, pfuncs, &data,
                       0 /* palette */, HB_COLOR (0, 0, 0, 255) /* opaque black */);

  hb_font_destroy (font);
  hb_face_destroy (face);
  hb_paint_funcs_destroy (pfuncs);
  return 0;
}
```

Note what is *not* implemented: no clips, no gradients, no groups, no images. All
of those run their built-in no-op stubs. This program prints the transform and
colour skeleton of a glyph and nothing else, which is exactly what a partially
implemented back end should do — degrade, not crash.

### The same shape in Rust

```rust
use core::ffi::{c_float, c_void};
use harfbuzz_sys::{
    HB_COLOR, hb_bool_t, hb_color_get_alpha, hb_color_get_blue, hb_color_get_green,
    hb_color_get_red, hb_color_t, hb_font_paint_glyph, hb_font_t, hb_paint_funcs_create,
    hb_paint_funcs_destroy, hb_paint_funcs_make_immutable, hb_paint_funcs_set_color_func,
    hb_paint_funcs_set_pop_transform_func, hb_paint_funcs_set_push_transform_func,
    hb_paint_funcs_t,
};

/// The state threaded through every callback as `paint_data`.
#[derive(Default)]
struct Recorder {
    ops: Vec<String>,
    depth: usize,
}

unsafe extern "C" fn push_transform(
    _funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    xx: c_float, yx: c_float,
    xy: c_float, yy: c_float,
    dx: c_float, dy: c_float,
    _user_data: *mut c_void,
) {
    // SAFETY: `paint_data` is the `&mut Recorder` passed to hb_font_paint_glyph.
    // HarfBuzz never calls the callbacks re-entrantly on another thread, and the
    // borrow lasts only for this call, so the exclusive reference is sound.
    let rec = unsafe { &mut *(paint_data as *mut Recorder) };

    rec.ops.push(format!("{:indent$}push-transform {xx} {yx} {xy} {yy} {dx} {dy}",
                         "", indent = rec.depth * 2));
    rec.depth += 1;
}

unsafe extern "C" fn pop_transform(
    _funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    _user_data: *mut c_void,
) {
    // SAFETY: as above.
    let rec = unsafe { &mut *(paint_data as *mut Recorder) };

    rec.depth = rec.depth.saturating_sub(1);
    rec.ops.push(format!("{:indent$}pop-transform", "", indent = rec.depth * 2));
}

unsafe extern "C" fn color(
    _funcs: *mut hb_paint_funcs_t,
    paint_data: *mut c_void,
    is_foreground: hb_bool_t,
    color: hb_color_t,
    _user_data: *mut c_void,
) {
    // SAFETY: as above.
    let rec = unsafe { &mut *(paint_data as *mut Recorder) };

    let (r, g, b, a) = (
        hb_color_get_red(color),
        hb_color_get_green(color),
        hb_color_get_blue(color),
        hb_color_get_alpha(color),
    );
    let tail = if is_foreground != 0 { " (foreground)" } else { "" };
    rec.ops.push(format!("{:indent$}color #{r:02x}{g:02x}{b:02x}{a:02x}{tail}",
                         "", indent = rec.depth * 2));
}

fn record_glyph(font: *mut hb_font_t, glyph: u32) -> Vec<String> {
    let mut rec = Recorder::default();

    // SAFETY: the funcs object is created here and destroyed at the end; the
    // callbacks are plain fn pointers with no user_data, so passing null
    // user_data and a null destroy notifier leaks nothing. `&mut rec` outlives
    // the paint call, which is the only thing that dereferences it.
    unsafe {
        let pfuncs = hb_paint_funcs_create();

        hb_paint_funcs_set_push_transform_func(pfuncs, Some(push_transform), core::ptr::null_mut(), None);
        hb_paint_funcs_set_pop_transform_func(pfuncs, Some(pop_transform), core::ptr::null_mut(), None);
        hb_paint_funcs_set_color_func(pfuncs, Some(color), core::ptr::null_mut(), None);
        hb_paint_funcs_make_immutable(pfuncs);

        hb_font_paint_glyph(
            font,
            glyph,
            pfuncs,
            &mut rec as *mut Recorder as *mut c_void,
            0,
            HB_COLOR(0, 0, 0, 255),
        );

        hb_paint_funcs_destroy(pfuncs);
    }

    rec.ops
}
```

### Reading a gradient's colour line

The two-pass idiom: ask for the total, size a buffer, fill it, sort it.

```c
static void
my_linear_gradient (hb_paint_funcs_t *funcs, void *paint_data,
                    hb_color_line_t *color_line,
                    float x0, float y0, float x1, float y1, float x2, float y2,
                    void *user_data)
{
  /* Query the total; both out-args may be NULL for a count-only call. */
  unsigned int total = hb_color_line_get_color_stops (color_line, 0, NULL, NULL);

  hb_color_stop_t *stops = malloc (total * sizeof (hb_color_stop_t));
  unsigned int count = total;
  hb_color_line_get_color_stops (color_line, 0, &count, stops);
  /* `count` now holds how many were actually written. */

  /* Stops may be out of order and offsets may fall outside [0,1]. */
  float min, max;
  hb_paint_normalize_color_line (stops, count, &min, &max);

  hb_paint_extend_t extend = hb_color_line_get_extend (color_line);

  /* COLRv1 gives three anchors; most 2D APIs want two. */
  float ax0, ay0, ax1, ay1;
  hb_paint_reduce_linear_anchors (x0, y0, x1, y1, x2, y2, &ax0, &ay0, &ax1, &ay1);

  /* ... hand (ax0,ay0)-(ax1,ay1), the stops, and `extend` to your renderer,
     remembering to rescale the axis by (min, max) ... */

  free (stops);
}
```

```rust
use core::ffi::{c_float, c_uint, c_void};
use harfbuzz_sys::{
    hb_color_line_get_color_stops, hb_color_line_get_extend, hb_color_line_t, hb_color_stop_t,
    hb_paint_funcs_t, hb_paint_normalize_color_line, hb_paint_reduce_linear_anchors,
};

unsafe extern "C" fn linear_gradient(
    _funcs: *mut hb_paint_funcs_t,
    _paint_data: *mut c_void,
    color_line: *mut hb_color_line_t,
    x0: c_float, y0: c_float,
    x1: c_float, y1: c_float,
    x2: c_float, y2: c_float,
    _user_data: *mut c_void,
) {
    // SAFETY: `color_line` is valid for the duration of this callback only. The
    // first call passes null for both out-parameters, which the API documents
    // as a count-only query.
    let total = unsafe { hb_color_line_get_color_stops(
        color_line, 0, core::ptr::null_mut(), core::ptr::null_mut(),
    ) };

    let mut stops = vec![hb_color_stop_t { offset: 0.0, is_foreground: 0, color: 0 }; total as usize];
    let mut count: c_uint = total;

    // SAFETY: `stops` has capacity `count`, which is what `count` says on input.
    unsafe { hb_color_line_get_color_stops(color_line, 0, &mut count, stops.as_mut_ptr()) };
    stops.truncate(count as usize);

    let (mut min, mut max): (c_float, c_float) = (0.0, 0.0);
    // SAFETY: `stops` is a live slice of `stops.len()` elements, rewritten in
    // place; `min` and `max` are live locals.
    unsafe {
        hb_paint_normalize_color_line(stops.as_mut_ptr(), stops.len() as c_uint, &mut min, &mut max)
    };

    // SAFETY: `color_line` is still valid inside this callback.
    let extend = unsafe { hb_color_line_get_extend(color_line) };

    let (mut ax0, mut ay0, mut ax1, mut ay1) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
    // SAFETY: all four out-parameters point at live locals.
    unsafe {
        hb_paint_reduce_linear_anchors(
            x0, y0, x1, y1, x2, y2, &mut ax0, &mut ay0, &mut ax1, &mut ay1,
        )
    };

    let _ = (extend, min, max, ax0, ay0, ax1, ay1, stops);
}
```

### Rendering a sweep gradient with only linear gradients

```rust
use core::ffi::{c_float, c_void};
use harfbuzz_sys::{hb_color_stop_t, hb_color_t, hb_paint_extend_t, hb_paint_sweep_gradient_tiles};

unsafe extern "C" fn emit_patch(
    a0: c_float,
    c0: hb_color_t,
    a1: c_float,
    c1: hb_color_t,
    user_data: *mut c_void,
) {
    // SAFETY: `user_data` is the `&mut Vec<..>` passed below; the callback's
    // scope is `call`, so it is never invoked after the call returns.
    let sectors = unsafe { &mut *(user_data as *mut Vec<(f32, u32, f32, u32)>) };
    sectors.push((a0, c0, a1, c1));
}

/// `stops` must already be sorted with offsets in 0..1 — run
/// `hb_paint_normalize_color_line` first. The array may be reordered in place.
fn tile_sweep(
    stops: &mut [hb_color_stop_t],
    extend: hb_paint_extend_t,
    start_angle: f32,
    end_angle: f32,
) -> Vec<(f32, u32, f32, u32)> {
    let mut sectors: Vec<(f32, u32, f32, u32)> = Vec::new();

    // SAFETY: `stops` is a live, exclusively borrowed slice (the function may
    // mutate it), and `sectors` outlives the call.
    unsafe {
        hb_paint_sweep_gradient_tiles(
            stops.as_mut_ptr(),
            stops.len() as core::ffi::c_uint,
            extend,
            start_angle,
            end_angle,
            Some(emit_patch),
            &mut sectors as *mut _ as *mut c_void,
        )
    };

    sectors
}
```

### Overriding palette colours

```c
static hb_bool_t
my_palette_color (hb_paint_funcs_t *funcs, void *paint_data,
                  unsigned int color_index, hb_color_t *color, void *user_data)
{
  /* Recolour only entry 0; everything else comes from the font's CPAL. */
  if (color_index == 0)
    {
      *color = HB_COLOR (0x20, 0x40, 0xE0, 0xFF);   /* b, g, r, a */
      return true;
    }
  return false;
}

hb_paint_funcs_set_custom_palette_color_func (pfuncs, my_palette_color, NULL, NULL);
```

Remember the argument order of `HB_COLOR` is blue, green, red, alpha.

## Pitfalls

### The setters fail silently

`hb_paint_funcs_set_*_func()` returns `void`. On an immutable object the call
does nothing but run your `destroy` notifier. Since `hb_paint_funcs_create()`
also returns the (permanently immutable) empty singleton on allocation failure,
a low-memory situation produces a paint-funcs object that accepts every install
and honours none. Check `hb_paint_funcs_is_immutable()`, or compare the created
object against `hb_paint_funcs_get_empty()`.

### `hb_paint_funcs_create` never returns null

It returns the empty singleton instead. Painting with that object draws nothing
at all, which looks exactly like a font with no colour data.

### Every unset callback is a silent no-op

There is no "unimplemented" error. A back end that forgets `pop_clip` will
happily paint with an ever-growing clip stack and produce wrong output with no
diagnostic. Implement pushes and pops in pairs, always.

### Two stubs are not no-ops, and that is load-bearing

`fill_glyph`'s stub emits `push_clip_glyph` → `color` → `pop_clip`, and
`push_group_for`'s stub calls `push_group`. If you implement `fill_glyph` *and*
`push_clip_glyph`/`color`/`pop_clip`, only `fill_glyph` runs for solid glyph
fills — do not double-count. Conversely, if you implement neither `fill_glyph`
nor the clip trio, you get three no-ops and nothing is drawn.

### `paint_data`, `user_data`, and the user-data table are three different things

`paint_data` is per-paint-call and comes from `hb_font_paint_glyph()`.
`user_data` is per-callback-slot and comes from that slot's setter. The
`hb_user_data_key_t` table is per-object and comes from
`hb_paint_funcs_set_user_data()`. Every callback receives the first and the
second; none receives the third.

### The colour line is stack-allocated and callback-scoped

All three gradient typedefs say it: *"It is only valid for the duration of the
callback, you cannot keep it around."* HarfBuzz builds the `hb_color_line_t` on
its own stack. Copy the stops out before returning; storing the pointer is a
dangling-pointer bug that will usually appear to work.

### Colour stops can be out of order and outside 0..1

`hb_color_line_get_color_stops()` documents that variations can leave stops
unsorted, and `hb_color_stop_t` documents that offsets are *typically* 0..1 but
need not be. Run `hb_paint_normalize_color_line()` before doing anything with
them, and use the `min`/`max` it reports to rescale your gradient geometry so
the rendering does not change.

### Interpolate in premultiplied space

`hb_color_stop_t.color` is unpremultiplied, but the header is explicit that
gradient interpolation *"shall happen in premultiplied space"* per the COLR
specification. Interpolating the unpremultiplied values directly produces
visible haloes wherever alpha varies along the line.

### `hb_paint_sweep_gradient_tiles` mutates your array

It is annotated `(inout)` and will reverse the stops in place if
`end_angle < start_angle`. Pass a copy if you still need the original ordering,
and pre-sort with `hb_paint_normalize_color_line()` because the function assumes
sorted input in 0..1 and does not check.

### The linear gradient has three points, not two

`(x2, y2)` is a rotation reference, not the gradient's end. Feeding `(x0,y0)` and
`(x1,y1)` straight to a two-point API silently ignores the rotation and skews
every rotated gradient. Use `hb_paint_reduce_linear_anchors()`.

### `hb_paint_push_clip_path_start` can return null

The built-in stub does exactly that, meaning "arbitrary-path clipping is not
supported". Check the return before issuing draw calls. Also: the draw funcs are
`(transfer none)` — the back end owns them, so destroying them is an
over-release — and both they and `draw_data` die at the matching
`hb_paint_push_clip_path_end()`.

### Only draw calls between clip-path start and end

The header is explicit: *"no other paint calls should be made in between."*

### `pop_clip` unwinds all three clip flavours

One `pop_clip` matches one `push_clip_glyph`, *or* one `push_clip_rectangle`,
*or* one `push_clip_path_start`/`push_clip_path_end` pair. There is no
`pop_clip_rectangle`. Push and pop counts must balance across the whole
operation, and the header guarantees the calls are properly nested — so a single
stack works, as it says.

### Composite-mode values are not in the documented order

`HB_PAINT_COMPOSITE_MODE_DEST` is 2 and `HB_PAINT_COMPOSITE_MODE_SRC_OVER` is 3,
even though the doc block lists `SRC_OVER` before `DEST`. `MULTIPLY` is 23, not
adjacent to `SCREEN` at 13. Never derive the numbers from reading order, and
never persist them across HarfBuzz versions.

### `hb_paint_image`'s `slant` is dead

*"Deprecated. Always set to 0.0."* Pass `0.0` from the manual API and ignore it
in your callback.

### `hb_paint_image` discards the callback's return value

The callback returns `hb_bool_t`, but the manual `hb_paint_image()` wrapper
returns `void`. If you need to know whether the image was handled, call your own
callback directly rather than routing through the wrapper.

### `is_foreground` handling is all-or-nothing

Ignore it and use `color` verbatim: correct. Substitute your own foreground RGB:
also correct, *but* you must then multiply your foreground's alpha by the alpha
in `color`, and you must have passed a fully opaque `foreground` to
`hb_font_paint_glyph()` for the arithmetic to work out. Doing half of this
produces subtly wrong opacity in glyphs that use paint-tree alpha.

### `BGRA` image data is premultiplied; nothing else here is

`HB_PAINT_IMAGE_FORMAT_BGRA` pixels are "BGRA pre-multiplied sRGBA". Every
`hb_color_t` in this header — in `color`, `fill_glyph`, `hb_color_stop_t`, and
the custom-palette callback — is unpremultiplied.

### Image dimensions can be zero

`width` and `height` are documented as "width/height of the raster image in
pixels, **or 0**". SVG payloads normally report 0. Fall back to `extents`, which
is itself `(nullable)`.

### `color_glyph` returning true suppresses recursion

Return `false` unless you really did paint the glyph yourself. Returning `true`
from a stub-like implementation makes the glyph vanish.

### Since-version drift in the Rust bindings

`harfbuzz-sys/src/paint.rs` annotates `hb_paint_push_font_transform` as 7.0.0 and
`hb_paint_push_inverse_font_transform` as 8.2.0. Upstream's `hb-paint.cc` gtk-doc
comments say **11.0.0** for both, and `NEWS` lists
`+hb_paint_push_font_transform()` among the 11.0.0 additions. Trust upstream.

### Threading

The header is silent on thread safety. Reference counts are atomic in a normally
configured build. The setters and the immutable flag are not documented as
synchronised, so configure the object on one thread, make it immutable, then
share it. Your callbacks must be re-entrant if the object is shared, and the
`paint_data` you pass to each `hb_font_paint_glyph()` call should be per-thread —
that is what it is for.

### Availability

The whole header is compiled out when HarfBuzz is built with `HB_NO_PAINT`. In a
reduced-feature build the symbols may not exist at all.

## Section coverage

All 80 entries from upstream's `<FILE>hb-paint</FILE>` section are covered.

| Section entry | Covered under |
| --- | --- |
| `hb_paint_funcs_t` | Types |
| `hb_paint_funcs_create` | Object lifecycle |
| `hb_paint_funcs_get_empty` | Object lifecycle |
| `hb_paint_funcs_reference` | Object lifecycle |
| `hb_paint_funcs_destroy` | Object lifecycle |
| `hb_paint_funcs_set_user_data` | User data |
| `hb_paint_funcs_get_user_data` | User data |
| `hb_paint_funcs_make_immutable` | Immutability |
| `hb_paint_funcs_is_immutable` | Immutability |
| `hb_paint_push_transform_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_push_transform_func` | Installing callbacks |
| `hb_paint_pop_transform_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_pop_transform_func` | Installing callbacks |
| `hb_paint_color_glyph_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_color_glyph_func` | Installing callbacks |
| `hb_paint_fill_glyph_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_fill_glyph_func` | Installing callbacks |
| `hb_paint_push_clip_glyph_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_push_clip_glyph_func` | Installing callbacks |
| `hb_paint_push_clip_rectangle_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_push_clip_rectangle_func` | Installing callbacks |
| `hb_paint_push_clip_path_start_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_push_clip_path_start_func` | Installing callbacks |
| `hb_paint_push_clip_path_end_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_push_clip_path_end_func` | Installing callbacks |
| `hb_paint_pop_clip_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_pop_clip_func` | Installing callbacks |
| `hb_paint_color_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_color_func` | Installing callbacks |
| `HB_PAINT_IMAGE_FORMAT_PNG` | Types → Image format tags |
| `HB_PAINT_IMAGE_FORMAT_SVG` | Types → Image format tags |
| `HB_PAINT_IMAGE_FORMAT_BGRA` | Types → Image format tags |
| `hb_paint_image_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_image_func` | Installing callbacks |
| `hb_color_line_t` | Types |
| `hb_color_stop_t` | Types |
| `hb_color_line_get_color_stops_func_t` | Types → Callback typedefs |
| `hb_color_line_get_color_stops` | Reading a colour line |
| `hb_paint_extend_t` | Types |
| `hb_color_line_get_extend_func_t` | Types → Callback typedefs |
| `hb_color_line_get_extend` | Reading a colour line |
| `hb_paint_linear_gradient_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_linear_gradient_func` | Installing callbacks |
| `hb_paint_radial_gradient_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_radial_gradient_func` | Installing callbacks |
| `hb_paint_sweep_gradient_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_sweep_gradient_func` | Installing callbacks |
| `hb_paint_composite_mode_t` | Types |
| `hb_paint_push_group_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_push_group_func` | Installing callbacks |
| `hb_paint_push_group_for_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_push_group_for_func` | Installing callbacks |
| `hb_paint_pop_group_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_pop_group_func` | Installing callbacks |
| `hb_paint_custom_palette_color_func_t` | Types → Callback typedefs |
| `hb_paint_funcs_set_custom_palette_color_func` | Installing callbacks |
| `hb_paint_push_transform` | The manual API |
| `hb_paint_push_font_transform` | The manual API |
| `hb_paint_push_inverse_font_transform` | The manual API |
| `hb_paint_pop_transform` | The manual API |
| `hb_paint_color_glyph` | The manual API |
| `hb_paint_fill_glyph` | The manual API |
| `hb_paint_push_clip_glyph` | The manual API |
| `hb_paint_push_clip_rectangle` | The manual API |
| `hb_paint_push_clip_path_start` | The manual API |
| `hb_paint_push_clip_path_end` | The manual API |
| `hb_paint_pop_clip` | The manual API |
| `hb_paint_color` | The manual API |
| `hb_paint_image` | The manual API |
| `hb_paint_linear_gradient` | The manual API |
| `hb_paint_radial_gradient` | The manual API |
| `hb_paint_sweep_gradient` | The manual API |
| `hb_paint_push_group` | The manual API |
| `hb_paint_push_group_for` | The manual API |
| `hb_paint_pop_group` | The manual API |
| `hb_paint_custom_palette_color` | The manual API |
| `hb_paint_reduce_linear_anchors` | Gradient helpers |
| `hb_paint_normalize_color_line` | Gradient helpers |
| `hb_paint_sweep_gradient_tile_func_t` | Types → Callback typedefs |
| `hb_paint_sweep_gradient_tiles` | Gradient helpers |
