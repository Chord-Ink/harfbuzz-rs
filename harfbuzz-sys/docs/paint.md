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
