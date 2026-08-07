# Vector output

Source header: `hb-vector.h`. Rust module: `harfbuzz_sys::vector` (behind the
`vector` Cargo feature; **not** glob re-exported at the crate root — reach it as
`harfbuzz_sys::vector::*`). gtk-doc section: `hb-vector`.

## Overview

`hb-vector` is an optional HarfBuzz sub-library that converts glyphs into
**SVG** or **PDF** documents. It is the mirror image of `hb-draw.h` and
`hb-paint.h`: where those define callback interfaces that *you* implement to
receive glyph geometry, `hb-vector` hands you two ready-made sinks that receive
that geometry for you, accumulate it, and serialise it. You never write a
callback.

There are two contexts, and which one you want depends on whether you care
about colour:

* **`hb_vector_draw_t`** converts monochrome glyph **outlines**. Internally it
  owns an immutable `hb_draw_funcs_t` — retrievable with
  `hb_vector_draw_get_funcs()` — whose five callbacks append SVG path data
  (`M`/`L`/`Q`/`C`/`Z`) or PDF content-stream operators (`m`/`l`/`c`/`h`/`f`).
  Everything is filled with a single foreground colour.

* **`hb_vector_paint_t`** converts **colour glyphs**: `COLR` v0 and v1 paint
  graphs, `SVG`-table glyphs, and embedded bitmap images. It owns an immutable
  `hb_paint_funcs_t` — retrievable with `hb_vector_paint_get_funcs()` — that
  reproduces gradients, layers, clips, and Porter-Duff compositing in the target
  format. It is strictly more capable than the draw context and correspondingly
  more machinery.

Both are reference-counted HarfBuzz objects with the usual lifecycle:
`hb_vector_*_create_or_fail()` → configure → feed glyphs → `hb_vector_*_render()`
→ `hb_vector_*_destroy()`. Both support `hb_vector_*_reference()` and the
standard user-data key/value pairs. Note the naming: the constructors are
`_create_or_fail`, and unlike most HarfBuzz `_create()` functions **they return
NULL on failure** rather than a nil singleton. There is no `_get_empty()` and no
`_create()` for either type.

**Coordinate pipeline.** A point travels through three stages: font units → the
context's affine **transform** → division by the context's **scale factors**.
The transform (`hb_vector_*_set_transform()`) is a full 2×3 affine matrix and is
where you place a glyph at its pen position and apply any rotation or skew. The
scale factors (`hb_vector_*_set_scale_factor()`) then *divide* the result, which
exists so that a font scaled in 26.6 fixed point can be emitted in user units by
passing `64.0`. Extents (`hb_vector_extents_t`) are always expressed in output
space — after both stages.

**Extents are mandatory.** A context with no extents renders nothing:
`hb_vector_*_render()` returns NULL. Extents populate the SVG `viewBox` /
`width` / `height` and the PDF `MediaBox`. You get them either by passing
`HB_VECTOR_EXTENTS_MODE_EXPAND` to the per-glyph convenience functions, by
calling `hb_vector_*_set_glyph_extents()` with a glyph's ink box, or by setting
an explicit box with `hb_vector_*_set_extents()`.

**Accumulate, then render.** A context holds an arbitrary number of glyphs;
move the transform between glyphs to lay out a whole run, then render once. A
successful render **clears** the context — including its extents — so it can
immediately be reused. If you need the extents of what you just rendered, read
them *before* calling render.

Both contexts are ordinary reference-counted objects with no internal locking.
Nothing in this header is documented as thread-safe; treat a context as owned by
one thread at a time.

## Types

### `hb_vector_format_t`

The output serialisation, chosen once at creation and immutable thereafter.
Every value is an `hb_tag_t` — a four-byte code.

```c
typedef enum {
  HB_VECTOR_FORMAT_INVALID = HB_TAG_NONE,
  HB_VECTOR_FORMAT_SVG     = HB_TAG ('s','v','g',' '),
  HB_VECTOR_FORMAT_PDF     = HB_TAG ('p','d','f',' '),
} hb_vector_format_t;
```

```rust
pub type hb_vector_format_t = c_int;
pub const HB_VECTOR_FORMAT_INVALID: hb_vector_format_t = HB_TAG_NONE as hb_vector_format_t;
pub const HB_VECTOR_FORMAT_SVG: hb_vector_format_t = HB_TAG(b's', b'v', b'g', b' ') as hb_vector_format_t;
pub const HB_VECTOR_FORMAT_PDF: hb_vector_format_t = HB_TAG(b'p', b'd', b'f', b' ') as hb_vector_format_t;
```

| Constant | Value | Meaning |
| --- | --- | --- |
| `HB_VECTOR_FORMAT_INVALID` | `0x00000000` (`HB_TAG_NONE`) | Invalid format. Both `_create_or_fail()` functions reject it and return NULL. Also what `hb_vector_*_get_funcs()` treats a NULL context as. |
| `HB_VECTOR_FORMAT_SVG` | `0x73766720` (`'svg '`) | SVG output. A complete `<svg>` document with a `viewBox`, optional `<defs>`, and a `scale(1,-1)` wrapper that flips font Y-up space into SVG Y-down space. |
| `HB_VECTOR_FORMAT_PDF` | `0x70646620` (`'pdf '`) | PDF output. A complete single-page PDF 1.4 file: catalog, pages, page with `MediaBox`, content stream, xref table, and trailer. |

Underlying type: the C enumeration has no `_MAX_VALUE` sentinel and its largest
enumerator is `0x73766720`, which fits in a signed `int` — hence `c_int`.

Since HarfBuzz 13.0.0.

### `hb_vector_extents_t`

The output bounding box, in output space (after transform and after division by
the scale factors). Maps to the SVG `viewBox` and the PDF `MediaBox`. A public
`#[repr(C)]` struct that the caller allocates.

```c
typedef struct hb_vector_extents_t {
  float x, y;
  float width, height;
} hb_vector_extents_t;
```

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `x` | `float` | `c_float` | Left edge of the output coordinate system. |
| `y` | `float` | `c_float` | Top edge of the output coordinate system. |
| `width` | `float` | `c_float` | Width of the output coordinate system. |
| `height` | `float` | `c_float` | Height of the output coordinate system. |

Derives `Debug, Clone, Copy, PartialEq` on the Rust side. No `Eq`/`Hash` — the
fields are floats.

Two directions matter here and they are not symmetric:

* **Reading** (`hb_vector_*_get_extents()`): the struct you receive is always
  normalised — `width` and `height` are non-negative, and `(x, y)` is the
  minimum corner.
* **Writing** (`hb_vector_*_set_extents()`): the box you pass is interpreted in
  *input* space and divided by the current scale factors, then normalised. You
  may pass a glyph-extents-style box with a negative `height`; HarfBuzz flips it
  for you.

Since HarfBuzz 13.0.0.

### `hb_vector_extents_mode_t`

Whether the per-glyph convenience functions (`hb_vector_draw_glyph()`,
`hb_vector_draw_glyph_or_fail()`, `hb_vector_paint_glyph()`,
`hb_vector_paint_glyph_or_fail()`) also grow the context's extents.

```c
typedef enum {
  HB_VECTOR_EXTENTS_MODE_NONE   = 0,
  HB_VECTOR_EXTENTS_MODE_EXPAND = 1,
} hb_vector_extents_mode_t;
```

```rust
pub type hb_vector_extents_mode_t = c_int;
pub const HB_VECTOR_EXTENTS_MODE_NONE: hb_vector_extents_mode_t = 0;
pub const HB_VECTOR_EXTENTS_MODE_EXPAND: hb_vector_extents_mode_t = 1;
```

| Constant | Value | Meaning |
| --- | --- | --- |
| `HB_VECTOR_EXTENTS_MODE_NONE` | 0 | Do not touch extents. Use when you are managing the output box yourself with `hb_vector_*_set_extents()` — for example a fixed page size. |
| `HB_VECTOR_EXTENTS_MODE_EXPAND` | 1 | Fetch the glyph's ink extents with `hb_font_get_glyph_extents()`, transform all four corners, and union the resulting box into the context's current extents. |

Underlying type: the C enumeration has no sentinel and its largest enumerator is
1, so it fits in `int` — hence `c_int`.

Note that `EXPAND` is best-effort: if the font reports no extents for the glyph,
or the transformed box is degenerate, the extents are simply left alone and no
error is reported.

Since HarfBuzz 13.0.0.

### `hb_vector_draw_t`

Opaque, reference-counted draw context for monochrome outline conversion.
Created with `hb_vector_draw_create_or_fail()`, shared with
`hb_vector_draw_reference()`, released with `hb_vector_draw_destroy()`.

Its state is: output format (fixed at creation), affine transform, X/Y scale
factors, accumulated extents plus a "has extents" flag, foreground colour,
background colour, numeric precision, the accumulated output buffers, and an
optional recycled blob.

In Rust it is an `opaque_handle!` type — zero-sized, non-constructible, used
only behind `*mut hb_vector_draw_t` (or `*const` for the getters).

Since HarfBuzz 13.0.0.

### `hb_vector_paint_t`

Opaque, reference-counted paint context for colour-glyph conversion. Created
with `hb_vector_paint_create_or_fail()`, shared with
`hb_vector_paint_reference()`, released with `hb_vector_paint_destroy()`.

Everything `hb_vector_draw_t` holds, plus: a `CPAL` palette index, a map of
custom palette-colour overrides, an SVG id prefix, and the internal group /
clip / gradient bookkeeping needed to reproduce a `COLR` v1 paint graph.

In Rust it is an `opaque_handle!` type — zero-sized, non-constructible, used
only behind `*mut hb_vector_paint_t` (or `*const` for the getters).

Since HarfBuzz 13.0.0.

## Functions

The two contexts have deliberately parallel APIs. Where a draw function and a
paint function differ only in the type of their first argument, the entry below
documents both together; where the behaviour genuinely differs, it says so.

### Draw context — lifecycle

#### `hb_vector_draw_create_or_fail`

```c
hb_vector_draw_t * hb_vector_draw_create_or_fail (hb_vector_format_t format);
```

```rust
pub fn hb_vector_draw_create_or_fail(format: hb_vector_format_t) -> *mut hb_vector_draw_t;
```

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `format` | `HB_VECTOR_FORMAT_SVG` or `HB_VECTOR_FORMAT_PDF`. Any other value — including `HB_VECTOR_FORMAT_INVALID` — makes the call fail. |

**Returns** — a newly allocated context with a reference count of one, or
**NULL** on failure. Failure means either an unsupported `format` or an
allocation failure. There is no nil-object fallback and no `_get_empty()`
counterpart, so a NULL check here is mandatory.

**Ownership** — the caller owns the returned reference and must release it with
`hb_vector_draw_destroy()`.

**Notes** — the new context starts with the identity transform, scale factors of
`1.0`, no extents, opaque black foreground, transparent background, and
precision 2. Internal buffers are pre-allocated (2 KiB defs, 8 KiB body, 2 KiB
path) so the common case does no reallocation.

Since HarfBuzz 13.0.0.

#### `hb_vector_draw_reference`

```c
hb_vector_draw_t * hb_vector_draw_reference (hb_vector_draw_t *draw);
```

```rust
pub fn hb_vector_draw_reference(draw: *mut hb_vector_draw_t) -> *mut hb_vector_draw_t;
```

**Parameters** — `draw`: the context. Nullability is unspecified in the header;
HarfBuzz's generic object machinery tolerates NULL and returns it unchanged.

**Returns** — the same pointer, so the call chains.

**Ownership** — increments the reference count by one; balance it with a
matching `hb_vector_draw_destroy()`.

Since HarfBuzz 13.0.0.

#### `hb_vector_draw_destroy`

```c
void hb_vector_draw_destroy (hb_vector_draw_t *draw);
```

```rust
pub fn hb_vector_draw_destroy(draw: *mut hb_vector_draw_t);
```

**Parameters** — `draw`: the context. Nullability unspecified in the header; the
generic object machinery tolerates NULL.

**Returns** — nothing.

**Ownership** — decrements the reference count. At zero the context is freed,
along with any blob previously handed over via
`hb_vector_draw_recycle_blob()`, and every `destroy` callback registered
through `hb_vector_draw_set_user_data()` runs.

Since HarfBuzz 13.0.0.

#### `hb_vector_draw_set_user_data`

```c
hb_bool_t hb_vector_draw_set_user_data (hb_vector_draw_t   *draw,
                                        hb_user_data_key_t *key,
                                        void               *data,
                                        hb_destroy_func_t   destroy,
                                        hb_bool_t           replace);
```

```rust
pub fn hb_vector_draw_set_user_data(
    draw: *mut hb_vector_draw_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `key` | The user-data key. HarfBuzz uses its **address**, never its contents, so a `static`/`const` object of any lifetime longer than the context works. |
| `data` | The value to store. Opaque to HarfBuzz; may be NULL. |
| `destroy` | Called with `data` when the context is destroyed or the value is replaced. May be NULL. |
| `replace` | Non-zero to overwrite an existing entry for `key`; zero to leave an existing entry untouched. |

**Returns** — true on success, false otherwise (allocation failure, or `replace`
was false and an entry already existed).

**Ownership** — the context does not copy `data`; it stores the pointer and
calls `destroy` on it later.

Since HarfBuzz 13.0.0.

#### `hb_vector_draw_get_user_data`

```c
void * hb_vector_draw_get_user_data (const hb_vector_draw_t *draw,
                                     hb_user_data_key_t     *key);
```

```rust
pub fn hb_vector_draw_get_user_data(
    draw: *const hb_vector_draw_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

**Parameters** — `key`: the same key address used with the setter.

**Returns** — the stored value, or NULL if nothing is stored under `key`.

**Ownership** — borrowed. The context still owns the value and will still call
its `destroy`; do not free it yourself.

Since HarfBuzz 13.0.0.

### Draw context — geometry configuration

#### `hb_vector_draw_set_transform`

```c
void hb_vector_draw_set_transform (hb_vector_draw_t *draw,
                                   float xx, float yx,
                                   float xy, float yy,
                                   float dx, float dy);
```

```rust
pub fn hb_vector_draw_set_transform(
    draw: *mut hb_vector_draw_t,
    xx: c_float, yx: c_float,
    xy: c_float, yy: c_float,
    dx: c_float, dy: c_float,
);
```

**Parameters** — the six components of a 2×3 affine matrix: linear part
`[xx yx; xy yy]`, translation `(dx, dy)`. A point `(x, y)` maps to
`(xx·x + xy·y + dx, yx·x + yy·y + dy)`.

**Returns** — nothing. This replaces the transform outright; it does not
compose with the previous one.

**Notes** — this is the natural place to position each glyph of a shaped run:
set `dx`/`dy` to the pen position before each `hb_vector_draw_glyph()` call. It
applies to geometry emitted *after* the call; already-flushed paths keep the
transform that was in effect when they were emitted. Default: the identity
`(1, 0, 0, 1, 0, 0)`.

Since HarfBuzz 13.0.0.

#### `hb_vector_draw_get_transform`

```c
void hb_vector_draw_get_transform (const hb_vector_draw_t *draw,
                                   float *xx, float *yx,
                                   float *xy, float *yy,
                                   float *dx, float *dy);
```

```rust
pub fn hb_vector_draw_get_transform(
    draw: *const hb_vector_draw_t,
    xx: *mut c_float, yx: *mut c_float,
    xy: *mut c_float, yy: *mut c_float,
    dx: *mut c_float, dy: *mut c_float,
);
```

**Parameters** — six out pointers. **Every one may be NULL** and is then simply
skipped, so you can query a single component.

**Returns** — nothing.

Since HarfBuzz 13.0.0.

#### `hb_vector_draw_set_scale_factor`

```c
void hb_vector_draw_set_scale_factor (hb_vector_draw_t *draw,
                                      float x_scale_factor,
                                      float y_scale_factor);
```

```rust
pub fn hb_vector_draw_set_scale_factor(
    draw: *mut hb_vector_draw_t,
    x_scale_factor: c_float,
    y_scale_factor: c_float,
);
```

**Parameters** — `x_scale_factor`, `y_scale_factor`: divisors applied to
transformed coordinates. **Values that are not greater than zero are silently
clamped to `1.0`** — you cannot use this to flip an axis; use the transform for
that.

**Returns** — nothing.

**Notes** — the name is misleading: these *divide*. A font set up with
`hb_font_set_scale(font, upem*64, upem*64)` emits sane user units when you pass
`64.0, 64.0`. Default: `1.0, 1.0`.

Since HarfBuzz 13.0.0.

#### `hb_vector_draw_get_scale_factor`

```c
void hb_vector_draw_get_scale_factor (const hb_vector_draw_t *draw,
                                      float *x_scale_factor,
                                      float *y_scale_factor);
```

```rust
pub fn hb_vector_draw_get_scale_factor(
    draw: *const hb_vector_draw_t,
    x_scale_factor: *mut c_float,
    y_scale_factor: *mut c_float,
);
```

**Parameters** — two out pointers; **either may be NULL** and is then skipped.

**Returns** — nothing.

Since HarfBuzz 13.0.0.

### Draw context — extents

#### `hb_vector_draw_set_extents`

```c
void hb_vector_draw_set_extents (hb_vector_draw_t          *draw,
                                 const hb_vector_extents_t *extents);
```

```rust
pub fn hb_vector_draw_set_extents(
    draw: *mut hb_vector_draw_t,
    extents: *const hb_vector_extents_t,
);
```

**Parameters** — `extents`: the box to set or expand by. **NULL is allowed and
clears the extents entirely** (the context returns to "no extents", so a render
would fail).

**Returns** — nothing, and there is no way to detect the two silent no-ops
described below.

**Ownership** — the struct is read and copied; the caller keeps it.

**Notes** — despite the name this is an *expand*, not a replace: if the context
already has extents, the new box is unioned in. Two silent behaviours to know:

* A box whose `width` or `height` is exactly `0.0` is **ignored entirely** —
  neither set nor cleared.
* The box is treated as input-space and divided by the current scale factors
  before use, then normalised so `(x, y)` is the minimum corner and the size is
  positive. So `set` followed by `get` does **not** round-trip unless the scale
  factors are `1.0` and the box was already normalised.

To replace rather than expand, call with NULL first, then with the box.

Since HarfBuzz 13.0.0.

#### `hb_vector_draw_get_extents`

```c
hb_bool_t hb_vector_draw_get_extents (const hb_vector_draw_t *draw,
                                      hb_vector_extents_t    *extents);
```

```rust
pub fn hb_vector_draw_get_extents(
    draw: *const hb_vector_draw_t,
    extents: *mut hb_vector_extents_t,
) -> hb_bool_t;
```

**Parameters** — `extents`: where to store the result. **May be NULL** if you
only want the boolean.

**Returns** — true if the context has extents, false if not. `*extents` is
written **only** when the return value is true; on false it is left untouched,
so initialise it yourself if you plan to read it unconditionally.

**Notes** — the returned box is in output space and already normalised. Call
this *before* `hb_vector_draw_render()`: a successful render clears the extents.

Since HarfBuzz 13.0.0.

#### `hb_vector_draw_set_glyph_extents`

```c
hb_bool_t hb_vector_draw_set_glyph_extents (hb_vector_draw_t         *draw,
                                            const hb_glyph_extents_t *glyph_extents);
```

```rust
pub fn hb_vector_draw_set_glyph_extents(
    draw: *mut hb_vector_draw_t,
    glyph_extents: *const hb_glyph_extents_t,
) -> hb_bool_t;
```

**Parameters** — `glyph_extents`: an ink box in font units, as produced by
`hb_font_get_glyph_extents()`. Note the HarfBuzz convention that `height` is
negative in a Y-up coordinate system. Nullability is unspecified; the
implementation dereferences it unconditionally, so pass a valid pointer.

**Returns** — true if the extents were expanded, false if the transformed box
turned out degenerate (zero width or zero height), in which case the context's
extents are unchanged.

**Notes** — all four corners are transformed and their axis-aligned bounding box
is unioned in, so rotation and skew are handled correctly. Unlike
`hb_vector_draw_set_extents()`, this one takes the box in **font units** and
applies the full transform, not just the scale-factor division.

Since HarfBuzz 13.0.0.

### Draw context — output configuration

#### `hb_vector_draw_get_format`

```c
hb_vector_format_t hb_vector_draw_get_format (const hb_vector_draw_t *draw);
```

```rust
pub fn hb_vector_draw_get_format(draw: *const hb_vector_draw_t) -> hb_vector_format_t;
```

**Returns** — the format the context was created with. Since creation rejects
anything else, this is always `HB_VECTOR_FORMAT_SVG` or
`HB_VECTOR_FORMAT_PDF` for a live context.

Since HarfBuzz 14.2.0.

#### `hb_vector_draw_set_precision`

```c
void hb_vector_draw_set_precision (hb_vector_draw_t *draw, unsigned precision);
```

```rust
pub fn hb_vector_draw_set_precision(draw: *mut hb_vector_draw_t, precision: c_uint);
```

**Parameters** — `precision`: decimal places used when writing coordinates.
**Silently clamped to a maximum of 12.**

**Returns** — nothing.

**Notes** — the default is 2, which keeps SVG path data compact. Raise it if you
are emitting at font-unit scale where 0.01 is coarse. Internal scale values use
at least 7 places regardless of this setting.

Since HarfBuzz 14.2.0.

#### `hb_vector_draw_get_precision`

```c
unsigned hb_vector_draw_get_precision (const hb_vector_draw_t *draw);
```

```rust
pub fn hb_vector_draw_get_precision(draw: *const hb_vector_draw_t) -> c_uint;
```

**Returns** — the current precision, or the default (2) if none was set.

Since HarfBuzz 14.2.0.

#### `hb_vector_draw_set_foreground`

```c
void hb_vector_draw_set_foreground (hb_vector_draw_t *draw, hb_color_t foreground);
```

```rust
pub fn hb_vector_draw_set_foreground(draw: *mut hb_vector_draw_t, foreground: hb_color_t);
```

**Parameters** — `foreground`: an `hb_color_t`, built with `HB_COLOR(b, g, r, a)`
(note the channel order). Alpha below 255 is honoured: SVG emits
`fill-opacity`, PDF emits an `ExtGState` with `/ca`.

**Returns** — nothing.

**Ownership** — a plain value; nothing to release.

**Notes** — **the pending path is flushed before the colour changes**, so a
colour applies only to geometry emitted afterwards. That makes multi-colour
output possible from a single draw context: set a colour, draw some glyphs, set
another, draw more. Default: opaque black, `HB_COLOR(0, 0, 0, 255)`.

Since HarfBuzz 14.2.0.

#### `hb_vector_draw_get_foreground`

```c
hb_color_t hb_vector_draw_get_foreground (const hb_vector_draw_t *draw);
```

```rust
pub fn hb_vector_draw_get_foreground(draw: *const hb_vector_draw_t) -> hb_color_t;
```

**Returns** — the current foreground colour.

Since HarfBuzz 14.2.0.

#### `hb_vector_draw_set_background`

```c
void hb_vector_draw_set_background (hb_vector_draw_t *draw, hb_color_t background);
```

```rust
pub fn hb_vector_draw_set_background(draw: *mut hb_vector_draw_t, background: hb_color_t);
```

**Parameters** — `background`: an `hb_color_t`. Alpha zero means "no
background".

**Returns** — nothing.

**Notes** — when the alpha channel is non-zero, the renderer emits a filled
rectangle covering the whole extents box behind all content: an SVG `<rect>`
with `fill-opacity`, or a PDF `re f` with an `ExtGState` alpha. Unlike the
foreground, changing this does **not** flush the pending path — only one
background exists per render, and it is applied at render time. Default:
transparent, `HB_COLOR(0, 0, 0, 0)`.

Since HarfBuzz 14.2.0.

#### `hb_vector_draw_get_background`

```c
hb_color_t hb_vector_draw_get_background (const hb_vector_draw_t *draw);
```

```rust
pub fn hb_vector_draw_get_background(draw: *const hb_vector_draw_t) -> hb_color_t;
```

**Returns** — the current background colour, transparent if never set.

Since HarfBuzz 14.2.0.

### Draw context — feeding glyphs

#### `hb_vector_draw_get_funcs`

```c
hb_draw_funcs_t * hb_vector_draw_get_funcs (const hb_vector_draw_t *draw);
```

```rust
pub fn hb_vector_draw_get_funcs(draw: *const hb_vector_draw_t) -> *mut hb_draw_funcs_t;
```

**Parameters** — `draw`: the context. **NULL is explicitly tolerated** and makes
the function return NULL.

**Returns** — an immutable `hb_draw_funcs_t` suitable for any HarfBuzz API that
produces outlines, or NULL if `draw` is NULL or its format is invalid. Pass
`draw` itself as the `draw_data` argument when you use it.

**Ownership** — **transfer none.** This is a per-format lazily created singleton
shared by every context of that format; it is already immutable. Do not destroy
it, and do not try to install your own callbacks on it.

**Notes** — the SVG funcs implement move-to, line-to, quadratic-to, cubic-to and
close-path. The PDF funcs deliberately leave quadratic-to unset so HarfBuzz's
default converter promotes quadratics to cubics, which is what PDF content
streams need.

Since HarfBuzz 14.2.0.

#### `hb_vector_draw_new_path`

```c
void hb_vector_draw_new_path (hb_vector_draw_t *draw);
```

```rust
pub fn hb_vector_draw_new_path(draw: *mut hb_vector_draw_t);
```

**Parameters** — `draw`: the context.

**Returns** — nothing.

**Notes** — flushes whatever path has accumulated (emitting it as one `<path>`
element or one `f` fill operator with the current foreground colour) and starts
a fresh one. Call it **between glyphs** when you are feeding outlines by hand
through `hb_vector_draw_get_funcs()`; otherwise consecutive glyphs land in one
path and the non-zero fill rule makes overlapping contours interact.
`hb_vector_draw_glyph()` and `hb_vector_draw_glyph_or_fail()` call it for you.

Since HarfBuzz 14.2.0.

#### `hb_vector_draw_glyph_or_fail`

```c
hb_bool_t hb_vector_draw_glyph_or_fail (hb_vector_draw_t         *draw,
                                        hb_font_t                *font,
                                        hb_codepoint_t            glyph,
                                        hb_vector_extents_mode_t  extents_mode);
```

```rust
pub fn hb_vector_draw_glyph_or_fail(
    draw: *mut hb_vector_draw_t,
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
    extents_mode: hb_vector_extents_mode_t,
) -> hb_bool_t;
```

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | The font to take the outline from. Not referenced or retained. |
| `glyph` | A **glyph ID**, not a Unicode codepoint. |
| `extents_mode` | `HB_VECTOR_EXTENTS_MODE_EXPAND` to also grow the context's extents by this glyph's ink box; `HB_VECTOR_EXTENTS_MODE_NONE` to leave extents alone. |

**Returns** — true if outline data was emitted, false otherwise (for example a
blank glyph, or a font with no outline source).

**Ownership** — nothing is transferred. The glyph's geometry is copied into the
context's buffers.

**Notes** — equivalent to:

```c
// if extents_mode == EXPAND: expand extents by hb_font_get_glyph_extents()
hb_vector_draw_new_path (draw);
hb_font_draw_glyph_or_fail (font, glyph, hb_vector_draw_get_funcs (draw), draw);
```

The extents expansion happens **before** the draw and is independent of the
return value: a glyph can grow the extents and still return false.

Since HarfBuzz 14.2.0.

#### `hb_vector_draw_glyph`

```c
void hb_vector_draw_glyph (hb_vector_draw_t         *draw,
                           hb_font_t                *font,
                           hb_codepoint_t            glyph,
                           hb_vector_extents_mode_t  extents_mode);
```

```rust
pub fn hb_vector_draw_glyph(
    draw: *mut hb_vector_draw_t,
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
    extents_mode: hb_vector_extents_mode_t,
);
```

**Parameters** — identical to `hb_vector_draw_glyph_or_fail()`.

**Returns** — nothing. Exactly `hb_vector_draw_glyph_or_fail()` with the result
discarded; there is no fallback behaviour and no other difference.

Since HarfBuzz 14.2.0.

### Draw context — rendering and reuse

#### `hb_vector_draw_render`

```c
hb_blob_t * hb_vector_draw_render (hb_vector_draw_t *draw);
```

```rust
pub fn hb_vector_draw_render(draw: *mut hb_vector_draw_t) -> *mut hb_blob_t;
```

**Parameters** — `draw`: the context.

**Returns** — a blob holding the complete document, or **NULL** if rendering
cannot proceed. The dominant NULL cause is *no extents*; an invalid format also
returns NULL.

**Ownership** — **transfer full.** The caller owns the blob and must release it
with `hb_blob_destroy()` — or, better, hand it to
`hb_vector_draw_recycle_blob()` when finished so the buffer is reused.

**Notes** — the bytes are a whole file, not a fragment: for SVG a complete
`<svg>` document with the `viewBox` taken from the extents, an optional
`<defs>`, an optional background `<rect>`, and a `<g transform="scale(1,-1)">`
wrapper that converts the font's Y-up space into SVG's Y-down space; for PDF a
complete PDF 1.4 file with a `MediaBox` derived from the same extents.

On success the context is **cleared** exactly as by `hb_vector_draw_clear()` —
accumulated paths *and extents* are discarded. Read your extents before
rendering. Configuration (transform, scale factors, precision, colours) is
preserved, so the context is immediately ready for the next document.

Since HarfBuzz 13.0.0.

#### `hb_vector_draw_clear`

```c
void hb_vector_draw_clear (hb_vector_draw_t *draw);
```

```rust
pub fn hb_vector_draw_clear(draw: *mut hb_vector_draw_t);
```

**Parameters** — `draw`: the context.

**Returns** — nothing.

**Notes** — discards the accumulated defs, body, and pending path, and resets
the extents to "none". **Preserves** user configuration: transform, scale
factors, precision, foreground, background. Use it to abandon a partially built
document.

Since HarfBuzz 14.2.0.

#### `hb_vector_draw_reset`

```c
void hb_vector_draw_reset (hb_vector_draw_t *draw);
```

```rust
pub fn hb_vector_draw_reset(draw: *mut hb_vector_draw_t);
```

**Parameters** — `draw`: the context.

**Returns** — nothing.

**Notes** — everything `hb_vector_draw_clear()` does, plus restoring the
defaults: identity transform, scale factors `1.0`, precision 2. Note that the
implementation does **not** reset the foreground or background colours — only
the transform, scale factors and precision — so if you need black-on-transparent
again, set them explicitly. The format and any user data are untouched.

Since HarfBuzz 13.0.0.

#### `hb_vector_draw_recycle_blob`

```c
void hb_vector_draw_recycle_blob (hb_vector_draw_t *draw, hb_blob_t *blob);
```

```rust
pub fn hb_vector_draw_recycle_blob(draw: *mut hb_vector_draw_t, blob: *mut hb_blob_t);
```

**Parameters** — `blob`: a blob previously returned by
`hb_vector_draw_render()` that you are finished with. **NULL is allowed** and
just drops whatever the context was holding.

**Returns** — nothing.

**Ownership** — **transfer full: the context takes ownership of `blob`** and
destroys it when the context is destroyed or when another blob is recycled. Do
not touch `blob` afterwards, and do not call `hb_blob_destroy()` on it as well.
Any blob the context was already holding is destroyed first.

**Notes** — a throughput optimisation for rendering many documents from one
context: the next render reuses the recycled blob's allocation instead of
allocating fresh. The singleton empty blob is recognised and ignored.

Since HarfBuzz 13.0.0.

### Paint context — lifecycle

#### `hb_vector_paint_create_or_fail`

```c
hb_vector_paint_t * hb_vector_paint_create_or_fail (hb_vector_format_t format);
```

```rust
pub fn hb_vector_paint_create_or_fail(format: hb_vector_format_t) -> *mut hb_vector_paint_t;
```

**Parameters** — `format`: `HB_VECTOR_FORMAT_SVG` or `HB_VECTOR_FORMAT_PDF`;
anything else fails.

**Returns** — a newly allocated context with a reference count of one, or
**NULL** on failure (unsupported format, or allocation failure).

**Ownership** — the caller owns the reference; release it with
`hb_vector_paint_destroy()`.

**Notes** — defaults match the draw context, plus palette index 0, no custom
palette overrides, and no SVG id prefix.

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_reference`

```c
hb_vector_paint_t * hb_vector_paint_reference (hb_vector_paint_t *paint);
```

```rust
pub fn hb_vector_paint_reference(paint: *mut hb_vector_paint_t) -> *mut hb_vector_paint_t;
```

**Returns** — the same pointer with the reference count incremented by one.

**Ownership** — balance with `hb_vector_paint_destroy()`.

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_destroy`

```c
void hb_vector_paint_destroy (hb_vector_paint_t *paint);
```

```rust
pub fn hb_vector_paint_destroy(paint: *mut hb_vector_paint_t);
```

**Returns** — nothing.

**Ownership** — decrements the reference count; at zero the context is freed
along with its PDF resources, its recycled blob, its active-colour-glyph set,
its SVG id prefix, and all user-data `destroy` callbacks.

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_set_user_data`

```c
hb_bool_t hb_vector_paint_set_user_data (hb_vector_paint_t  *paint,
                                         hb_user_data_key_t *key,
                                         void               *data,
                                         hb_destroy_func_t   destroy,
                                         hb_bool_t           replace);
```

```rust
pub fn hb_vector_paint_set_user_data(
    paint: *mut hb_vector_paint_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

Identical in every respect to `hb_vector_draw_set_user_data()`, on a paint
context. `destroy` and `data` may be NULL; `key` is identified by address.

**Returns** — true on success, false otherwise.

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_get_user_data`

```c
void * hb_vector_paint_get_user_data (const hb_vector_paint_t *paint,
                                      hb_user_data_key_t      *key);
```

```rust
pub fn hb_vector_paint_get_user_data(
    paint: *const hb_vector_paint_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

**Returns** — the stored value, or NULL. Borrowed, not owned.

Since HarfBuzz 13.0.0.

### Paint context — geometry configuration

#### `hb_vector_paint_set_transform`

```c
void hb_vector_paint_set_transform (hb_vector_paint_t *paint,
                                    float xx, float yx,
                                    float xy, float yy,
                                    float dx, float dy);
```

```rust
pub fn hb_vector_paint_set_transform(
    paint: *mut hb_vector_paint_t,
    xx: c_float, yx: c_float,
    xy: c_float, yy: c_float,
    dx: c_float, dy: c_float,
);
```

**Parameters** — the six components of a 2×3 affine matrix, as for the draw
context.

**Returns** — nothing; replaces the transform outright.

**Notes** — the paint pipeline uses this differently from the draw pipeline:
`hb_vector_paint_glyph()` pushes it as a real transform onto the paint callbacks
(`hb_paint_push_transform()` … `hb_paint_pop_transform()`) around the glyph,
rather than baking it into every coordinate. The observable effect is the same,
but the emitted SVG carries an explicit `transform=` attribute. Default: the
identity.

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_get_transform`

```c
void hb_vector_paint_get_transform (const hb_vector_paint_t *paint,
                                    float *xx, float *yx,
                                    float *xy, float *yy,
                                    float *dx, float *dy);
```

```rust
pub fn hb_vector_paint_get_transform(
    paint: *const hb_vector_paint_t,
    xx: *mut c_float, yx: *mut c_float,
    xy: *mut c_float, yy: *mut c_float,
    dx: *mut c_float, dy: *mut c_float,
);
```

**Parameters** — six out pointers, **each independently nullable**.

**Returns** — nothing.

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_set_scale_factor`

```c
void hb_vector_paint_set_scale_factor (hb_vector_paint_t *paint,
                                       float x_scale_factor,
                                       float y_scale_factor);
```

```rust
pub fn hb_vector_paint_set_scale_factor(
    paint: *mut hb_vector_paint_t,
    x_scale_factor: c_float,
    y_scale_factor: c_float,
);
```

**Parameters** — divisors applied to transformed coordinates. Values not greater
than zero are clamped to `1.0`.

**Returns** — nothing. Default: `1.0, 1.0`.

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_get_scale_factor`

```c
void hb_vector_paint_get_scale_factor (const hb_vector_paint_t *paint,
                                       float *x_scale_factor,
                                       float *y_scale_factor);
```

```rust
pub fn hb_vector_paint_get_scale_factor(
    paint: *const hb_vector_paint_t,
    x_scale_factor: *mut c_float,
    y_scale_factor: *mut c_float,
);
```

**Parameters** — two out pointers, either may be NULL.

**Returns** — nothing.

Since HarfBuzz 13.0.0.

### Paint context — extents

#### `hb_vector_paint_set_extents`

```c
void hb_vector_paint_set_extents (hb_vector_paint_t         *paint,
                                  const hb_vector_extents_t *extents);
```

```rust
pub fn hb_vector_paint_set_extents(
    paint: *mut hb_vector_paint_t,
    extents: *const hb_vector_extents_t,
);
```

**Parameters** — `extents`: the box to set or expand by; **NULL clears the
extents**.

**Returns** — nothing.

**Ownership** — the struct is copied.

**Notes** — same semantics as `hb_vector_draw_set_extents()`: a zero-width or
zero-height box is ignored; the box is divided by the scale factors, normalised,
and unioned with any existing extents.

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_get_extents`

```c
hb_bool_t hb_vector_paint_get_extents (const hb_vector_paint_t *paint,
                                       hb_vector_extents_t     *extents);
```

```rust
pub fn hb_vector_paint_get_extents(
    paint: *const hb_vector_paint_t,
    extents: *mut hb_vector_extents_t,
) -> hb_bool_t;
```

**Parameters** — `extents`: out parameter, **may be NULL**.

**Returns** — true if extents are set (and only then is `*extents` written),
false otherwise.

**Notes** — read this before `hb_vector_paint_render()`, which clears extents.

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_set_glyph_extents`

```c
hb_bool_t hb_vector_paint_set_glyph_extents (hb_vector_paint_t        *paint,
                                             const hb_glyph_extents_t *glyph_extents);
```

```rust
pub fn hb_vector_paint_set_glyph_extents(
    paint: *mut hb_vector_paint_t,
    glyph_extents: *const hb_glyph_extents_t,
) -> hb_bool_t;
```

**Parameters** — `glyph_extents`: an ink box in font units. Dereferenced
unconditionally; pass a valid pointer.

**Returns** — true on success, false if the transformed box is degenerate, in
which case extents are unchanged.

Since HarfBuzz 13.0.0.

### Paint context — colour configuration

#### `hb_vector_paint_set_foreground`

```c
void hb_vector_paint_set_foreground (hb_vector_paint_t *paint, hb_color_t foreground);
```

```rust
pub fn hb_vector_paint_set_foreground(paint: *mut hb_vector_paint_t, foreground: hb_color_t);
```

**Parameters** — `foreground`: the colour that `COLR` paints referencing the
text foreground resolve to. Built with `HB_COLOR(b, g, r, a)`.

**Returns** — nothing.

**Notes** — this is *not* a fill colour the way the draw context's foreground
is; it is the value passed to `hb_font_paint_glyph()` as the caller's foreground.
Fonts that use `PaintSolid` with palette index 0xFFFF pick it up.
`hb_vector_paint_glyph()` also uses it for the synthesised outline it falls back
to for glyphs with no colour data. Unlike the draw context, setting it does not
flush anything. Default: opaque black.

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_get_foreground`

```c
hb_color_t hb_vector_paint_get_foreground (const hb_vector_paint_t *paint);
```

```rust
pub fn hb_vector_paint_get_foreground(paint: *const hb_vector_paint_t) -> hb_color_t;
```

**Returns** — the current foreground, or the default opaque black if never set.

Since HarfBuzz 14.2.0.

#### `hb_vector_paint_set_background`

```c
void hb_vector_paint_set_background (hb_vector_paint_t *paint, hb_color_t background);
```

```rust
pub fn hb_vector_paint_set_background(paint: *mut hb_vector_paint_t, background: hb_color_t);
```

**Parameters** — `background`: an `hb_color_t`; alpha zero means no background.

**Returns** — nothing.

**Notes** — when non-transparent, a filled rectangle covering the extents is
emitted behind all glyph content at render time. Default: transparent.

Since HarfBuzz 14.2.0.

#### `hb_vector_paint_get_background`

```c
hb_color_t hb_vector_paint_get_background (const hb_vector_paint_t *paint);
```

```rust
pub fn hb_vector_paint_get_background(paint: *const hb_vector_paint_t) -> hb_color_t;
```

**Returns** — the current background, transparent if never set.

Since HarfBuzz 14.2.0.

#### `hb_vector_paint_set_palette`

```c
void hb_vector_paint_set_palette (hb_vector_paint_t *paint, int palette);
```

```rust
pub fn hb_vector_paint_set_palette(paint: *mut hb_vector_paint_t, palette: c_int);
```

**Parameters** — `palette`: the `CPAL` palette index handed to
`hb_font_paint_glyph()` / `hb_font_paint_glyph_or_fail()`. Signed here, but cast
to `unsigned` internally before use — do not rely on negative values meaning
anything. Query the number of palettes a face has with
`hb_ot_color_palette_get_count()`. What happens for an out-of-range index is
unspecified by this header; it is 0 for a font with a single palette.

**Returns** — nothing. Default: 0.

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_get_palette`

```c
int hb_vector_paint_get_palette (const hb_vector_paint_t *paint);
```

```rust
pub fn hb_vector_paint_get_palette(paint: *const hb_vector_paint_t) -> c_int;
```

**Returns** — the current palette index, or 0 if none was set.

Since HarfBuzz 14.2.0.

#### `hb_vector_paint_set_custom_palette_color`

```c
void hb_vector_paint_set_custom_palette_color (hb_vector_paint_t *paint,
                                               unsigned           color_index,
                                               hb_color_t         color);
```

```rust
pub fn hb_vector_paint_set_custom_palette_color(
    paint: *mut hb_vector_paint_t,
    color_index: c_uint,
    color: hb_color_t,
);
```

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `color_index` | The `CPAL` entry index to override. Indices need not exist in the font's palette; they are simply consulted first when a lookup happens. |
| `color` | The replacement colour. |

**Returns** — nothing; there is no way to detect a failed insertion.

**Notes** — overrides are keyed by index, persist on the context until cleared
or replaced for the same index, and are consulted by every paint operation that
resolves a `CPAL` entry — including `SVG`-table glyph content that uses CSS
`var(--colorN)`. This is how you recolour an emoji font without editing it.

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_clear_custom_palette_colors`

```c
void hb_vector_paint_clear_custom_palette_colors (hb_vector_paint_t *paint);
```

```rust
pub fn hb_vector_paint_clear_custom_palette_colors(paint: *mut hb_vector_paint_t);
```

**Parameters** — `paint`: the context.

**Returns** — nothing.

**Notes** — drops every override, so palette lookups revert to the selected font
palette. There is no "clear one index" function; clear all and re-add.

Since HarfBuzz 13.0.0.

### Paint context — output configuration

#### `hb_vector_paint_get_format`

```c
hb_vector_format_t hb_vector_paint_get_format (const hb_vector_paint_t *paint);
```

```rust
pub fn hb_vector_paint_get_format(paint: *const hb_vector_paint_t) -> hb_vector_format_t;
```

**Returns** — the format the context was created with.

Since HarfBuzz 14.2.0.

#### `hb_vector_paint_set_precision`

```c
void hb_vector_paint_set_precision (hb_vector_paint_t *paint, unsigned precision);
```

```rust
pub fn hb_vector_paint_set_precision(paint: *mut hb_vector_paint_t, precision: c_uint);
```

**Parameters** — `precision`: decimal places for numeric output, clamped to at
most 12.

**Returns** — nothing. Default: 2.

Since HarfBuzz 14.2.0.

#### `hb_vector_paint_get_precision`

```c
unsigned hb_vector_paint_get_precision (const hb_vector_paint_t *paint);
```

```rust
pub fn hb_vector_paint_get_precision(paint: *const hb_vector_paint_t) -> c_uint;
```

**Returns** — the current precision, or the default (2).

Since HarfBuzz 14.2.0.

#### `hb_vector_paint_set_svg_prefix`

```c
void hb_vector_paint_set_svg_prefix (hb_vector_paint_t *paint, const char *prefix);
```

```rust
pub fn hb_vector_paint_set_svg_prefix(paint: *mut hb_vector_paint_t, prefix: *const c_char);
```

**Parameters** — `prefix`: a NUL-terminated ASCII string prepended to every
emitted SVG `id` attribute and `url(#…)` reference. **NULL is allowed** and
means "no prefix"; an empty string behaves the same as NULL.

**Returns** — nothing. The copy is best-effort: if the allocation fails the
prefix is silently left unset, and there is no way to detect that other than
reading it back with `hb_vector_paint_get_svg_prefix()`.

**Ownership** — the string is **copied**; the caller keeps ownership of the
buffer it passed. The previous prefix is freed.

**Notes** — hb-vector uses short ids (`c0`, `g1`, …) for clip paths, gradients,
and `use` references. If you inject several hb-vector SVGs into the same DOM
document, those ids collide and the browser resolves references to the wrong
element; give each context a distinct prefix. **No effect on PDF output.**

Since HarfBuzz 14.2.0.

#### `hb_vector_paint_get_svg_prefix`

```c
const char * hb_vector_paint_get_svg_prefix (const hb_vector_paint_t *paint);
```

```rust
pub fn hb_vector_paint_get_svg_prefix(paint: *const hb_vector_paint_t) -> *const c_char;
```

**Returns** — the current prefix, or the empty string `""` if none was set.
**Never NULL.**

**Ownership** — borrowed. The pointer belongs to the context and stays valid
until the next `hb_vector_paint_set_svg_prefix()` — the header's docs also name
`hb_vector_paint_reset()` — or until the context is destroyed. Do not free it.

Since HarfBuzz 14.2.0.

### Paint context — feeding glyphs

#### `hb_vector_paint_get_funcs`

```c
hb_paint_funcs_t * hb_vector_paint_get_funcs (const hb_vector_paint_t *paint);
```

```rust
pub fn hb_vector_paint_get_funcs(paint: *const hb_vector_paint_t) -> *mut hb_paint_funcs_t;
```

**Parameters** — `paint`: the context. **NULL is tolerated** and yields NULL.

**Returns** — an immutable `hb_paint_funcs_t` for the context's format, or NULL
if `paint` is NULL or the format is invalid. Pass `paint` itself as the
`paint_data` argument when using it.

**Ownership** — **transfer none.** A per-format singleton; do not destroy it and
do not install callbacks on it.

**Notes** — these callbacks implement the full `hb-paint.h` surface — push/pop
transform, push/pop clip glyph and clip rectangle, push/pop group, solid fill,
linear/radial/sweep gradients, and images — translating each into SVG elements
or PDF operators.

Since HarfBuzz 14.2.0.

#### `hb_vector_paint_glyph_or_fail`

```c
hb_bool_t hb_vector_paint_glyph_or_fail (hb_vector_paint_t        *paint,
                                         hb_font_t                *font,
                                         hb_codepoint_t            glyph,
                                         hb_vector_extents_mode_t  extents_mode);
```

```rust
pub fn hb_vector_paint_glyph_or_fail(
    paint: *mut hb_vector_paint_t,
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
    extents_mode: hb_vector_extents_mode_t,
) -> hb_bool_t;
```

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | The font to take colour data from. Not retained. |
| `glyph` | A **glyph ID**. |
| `extents_mode` | `EXPAND` to also union this glyph's ink box into the context's extents; `NONE` to leave them alone. |

**Returns** — true if glyph paint data was emitted, false otherwise — which for
this function means the glyph has **no colour data** (`hb_font_paint_glyph_or_fail()`
reports failure). Use this when you want to know whether to fall back to your
own monochrome path.

**Ownership** — nothing transferred.

**Notes** — equivalent to:

```c
// if extents_mode == EXPAND: expand extents by hb_font_get_glyph_extents()
hb_paint_funcs_t *funcs = hb_vector_paint_get_funcs (paint);
hb_paint_push_transform (funcs, paint, /* the context's transform */);
hb_font_paint_glyph_or_fail (font, glyph, funcs, paint, palette, foreground);
hb_paint_pop_transform (funcs, paint);
```

Since HarfBuzz 14.2.0.

#### `hb_vector_paint_glyph`

```c
void hb_vector_paint_glyph (hb_vector_paint_t        *paint,
                            hb_font_t                *font,
                            hb_codepoint_t            glyph,
                            hb_vector_extents_mode_t  extents_mode);
```

```rust
pub fn hb_vector_paint_glyph(
    paint: *mut hb_vector_paint_t,
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
    extents_mode: hb_vector_extents_mode_t,
);
```

**Parameters** — identical to `hb_vector_paint_glyph_or_fail()`.

**Returns** — nothing.

**Notes** — **this is not merely the `_or_fail` variant with the result
ignored.** It calls `hb_font_paint_glyph()` rather than
`hb_font_paint_glyph_or_fail()`, so a glyph with no colour data falls back to a
synthesised foreground-coloured outline. Any glyph with an outline or a bitmap
image therefore produces output. Prefer this one for "render whatever this glyph
is"; prefer `_or_fail` when you specifically need to know whether the glyph was
a colour glyph.

Since HarfBuzz 14.2.0.

### Paint context — rendering and reuse

#### `hb_vector_paint_render`

```c
hb_blob_t * hb_vector_paint_render (hb_vector_paint_t *paint);
```

```rust
pub fn hb_vector_paint_render(paint: *mut hb_vector_paint_t) -> *mut hb_blob_t;
```

**Parameters** — `paint`: the context.

**Returns** — a blob holding the complete SVG or PDF document, or **NULL** if
rendering cannot proceed — no extents, an internal initialisation failure, or an
invalid format.

**Ownership** — **transfer full.** Release with `hb_blob_destroy()`, or hand it
to `hb_vector_paint_recycle_blob()`.

**Notes** — as with the draw context, a successful render **clears** the
context, extents included. The SVG output declares both the SVG and XLink
namespaces, since colour glyph output may use `<use xlink:href>`.

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_clear`

```c
void hb_vector_paint_clear (hb_vector_paint_t *paint);
```

```rust
pub fn hb_vector_paint_clear(paint: *mut hb_vector_paint_t);
```

**Parameters** — `paint`: the context.

**Returns** — nothing.

**Notes** — discards accumulated output, extents, group/clip/gradient
bookkeeping, and (for PDF) the accumulated resource objects. **Preserves**
transform, scale factors, precision, foreground, background, palette index, and
custom palette colours.

Since HarfBuzz 14.2.0.

#### `hb_vector_paint_reset`

```c
void hb_vector_paint_reset (hb_vector_paint_t *paint);
```

```rust
pub fn hb_vector_paint_reset(paint: *mut hb_vector_paint_t);
```

**Parameters** — `paint`: the context.

**Returns** — nothing.

**Notes** — everything `hb_vector_paint_clear()` does, plus restoring: identity
transform, scale factors `1.0`, foreground opaque black, palette index 0,
precision 2. Note what it does **not** restore: the background colour and the
custom palette overrides survive a reset. (The header documents the SVG id
prefix as being invalidated by reset; the implementation of `reset` does not
free it, so treat the prefix pointer as invalid after a reset but do not rely on
the prefix itself being cleared.)

Since HarfBuzz 13.0.0.

#### `hb_vector_paint_recycle_blob`

```c
void hb_vector_paint_recycle_blob (hb_vector_paint_t *paint, hb_blob_t *blob);
```

```rust
pub fn hb_vector_paint_recycle_blob(paint: *mut hb_vector_paint_t, blob: *mut hb_blob_t);
```

**Parameters** — `blob`: a blob previously returned by
`hb_vector_paint_render()`. **NULL is allowed** and drops whatever the context
held.

**Returns** — nothing.

**Ownership** — **transfer full: the context takes ownership of `blob`.** Do not
use or destroy it afterwards. Any previously recycled blob is destroyed first.
The singleton empty blob is recognised and ignored.

Since HarfBuzz 13.0.0.

## Usage

### Rendering one glyph outline to SVG (C)

```c
#include <hb.h>
#include <hb-vector.h>

hb_blob_t *
glyph_to_svg (hb_font_t *font, hb_codepoint_t gid)
{
  hb_vector_draw_t *draw = hb_vector_draw_create_or_fail (HB_VECTOR_FORMAT_SVG);
  if (!draw)
    return NULL;                     /* NULL, not a nil object. Always check. */

  /* Font is scaled in 26.6; divide back down to user units. */
  hb_vector_draw_set_scale_factor (draw, 64.f, 64.f);
  hb_vector_draw_set_precision (draw, 3);
  hb_vector_draw_set_foreground (draw, HB_COLOR (0x20, 0x20, 0x20, 0xFF)); /* b,g,r,a */

  hb_vector_draw_glyph (draw, font, gid, HB_VECTOR_EXTENTS_MODE_EXPAND);

  /* Read extents BEFORE rendering — render clears them. */
  hb_vector_extents_t ext;
  if (hb_vector_draw_get_extents (draw, &ext))
    printf ("box %g %g %g %g\n", ext.x, ext.y, ext.width, ext.height);

  hb_blob_t *svg = hb_vector_draw_render (draw);   /* NULL if no extents */
  hb_vector_draw_destroy (draw);
  return svg;                                       /* caller: hb_blob_destroy */
}
```

### Laying out a shaped run into one document (C)

Move the transform between glyphs; the context accumulates.

```c
hb_vector_draw_t *draw = hb_vector_draw_create_or_fail (HB_VECTOR_FORMAT_SVG);
hb_vector_draw_set_scale_factor (draw, 64.f, 64.f);

unsigned len;
hb_glyph_info_t     *info = hb_buffer_get_glyph_infos (buffer, &len);
hb_glyph_position_t *pos  = hb_buffer_get_glyph_positions (buffer, &len);

float x = 0, y = 0;
for (unsigned i = 0; i < len; i++)
{
  hb_vector_draw_set_transform (draw, 1.f, 0.f, 0.f, 1.f,
                                x + pos[i].x_offset,
                                y + pos[i].y_offset);
  hb_vector_draw_glyph (draw, font, info[i].codepoint,
                        HB_VECTOR_EXTENTS_MODE_EXPAND);
  x += pos[i].x_advance;
  y += pos[i].y_advance;
}

hb_blob_t *svg = hb_vector_draw_render (draw);
```

### Colour glyphs, with a palette override (C)

```c
hb_vector_paint_t *paint = hb_vector_paint_create_or_fail (HB_VECTOR_FORMAT_SVG);
if (!paint) return;

hb_vector_paint_set_scale_factor (paint, 64.f, 64.f);
hb_vector_paint_set_palette (paint, 1);                       /* CPAL palette 1 */
hb_vector_paint_set_custom_palette_color (paint, 3,
                                          HB_COLOR (0, 0, 255, 255)); /* red */
hb_vector_paint_set_svg_prefix (paint, "g17-");   /* unique per context in a DOM */

if (!hb_vector_paint_glyph_or_fail (paint, font, gid,
                                    HB_VECTOR_EXTENTS_MODE_EXPAND))
{
  /* Not a colour glyph — fall back to the monochrome path, or just use
   * hb_vector_paint_glyph(), which synthesises a foreground outline. */
}

hb_blob_t *svg = hb_vector_paint_render (paint);
hb_vector_paint_destroy (paint);
```

### Reusing one context across many renders (C)

```c
hb_vector_draw_t *draw = hb_vector_draw_create_or_fail (HB_VECTOR_FORMAT_PDF);
hb_blob_t *prev = NULL;

for (unsigned i = 0; i < n; i++)
{
  if (prev)
  {
    hb_vector_draw_recycle_blob (draw, prev);  /* takes ownership of prev */
    prev = NULL;
  }

  hb_vector_draw_set_transform (draw, 1, 0, 0, 1, 0, 0);
  hb_vector_draw_glyph (draw, font, gids[i], HB_VECTOR_EXTENTS_MODE_EXPAND);

  hb_blob_t *pdf = hb_vector_draw_render (draw);  /* also clears the context */
  if (pdf)
  {
    consume (pdf);
    prev = pdf;             /* hand it back next iteration instead of freeing */
  }
}
hb_blob_destroy (prev);      /* whatever we still hold */
hb_vector_draw_destroy (draw);
```

### The same, in Rust

```rust
use std::ffi::CStr;
use std::slice;

use harfbuzz_sys::vector::*;
use harfbuzz_sys::{
    HB_COLOR, hb_blob_destroy, hb_blob_get_data, hb_codepoint_t, hb_font_t,
};

/// Renders one glyph outline as an SVG document.
///
/// # Safety
/// `font` must be a live `hb_font_t`.
unsafe fn glyph_to_svg(font: *mut hb_font_t, gid: hb_codepoint_t) -> Option<Vec<u8>> {
    let draw = unsafe { hb_vector_draw_create_or_fail(HB_VECTOR_FORMAT_SVG) };
    if draw.is_null() {
        return None;
    }

    unsafe {
        hb_vector_draw_set_scale_factor(draw, 64.0, 64.0);
        hb_vector_draw_set_precision(draw, 3);
        hb_vector_draw_set_foreground(draw, HB_COLOR(0x20, 0x20, 0x20, 0xFF));

        hb_vector_draw_glyph(draw, font, gid, HB_VECTOR_EXTENTS_MODE_EXPAND);

        // Extents must be read before rendering; render clears them.
        let mut ext = hb_vector_extents_t { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };
        let has_extents = hb_vector_draw_get_extents(draw, &mut ext) != 0;
        debug_assert!(has_extents, "nothing was drawn; render would return null");

        let blob = hb_vector_draw_render(draw);
        let out = if blob.is_null() {
            None
        } else {
            let mut len = 0u32;
            let ptr = hb_blob_get_data(blob, &mut len);
            let bytes = slice::from_raw_parts(ptr as *const u8, len as usize).to_vec();
            hb_blob_destroy(blob);
            Some(bytes)
        };

        hb_vector_draw_destroy(draw);
        out
    }
}
```

Reading the SVG id prefix back, which is never null:

```rust
let prefix = unsafe { CStr::from_ptr(hb_vector_paint_get_svg_prefix(paint)) };
assert_eq!(prefix.to_bytes(), b"" as &[u8]); // empty string when unset
```

### Feeding outlines by hand

You are not obliged to use the `_glyph` convenience functions. Any HarfBuzz API
that takes an `hb_draw_funcs_t` can write into a draw context:

```c
hb_draw_funcs_t *funcs = hb_vector_draw_get_funcs (draw);  /* borrowed */

hb_vector_draw_new_path (draw);                 /* separate this glyph */
hb_font_draw_glyph (font, gid, funcs, draw);    /* draw_data == draw */

/* Decorations go through the same pen. The shape helpers from hb-draw.h
 * take an hb_draw_state_t; hb-vector's callbacks ignore it, so a
 * zero-initialised one is fine. */
hb_vector_draw_new_path (draw);
hb_draw_state_t st = HB_DRAW_STATE_DEFAULT;
hb_draw_rectangle (funcs, draw, &st,
                   0.f, -100.f, 500.f, 20.f,
                   NAN);   /* NaN stroke width == filled; 0 draws nothing */

/* Extents are yours to maintain in this mode. */
hb_glyph_extents_t ge;
if (hb_font_get_glyph_extents (font, gid, &ge))
  hb_vector_draw_set_glyph_extents (draw, &ge);
```

## Pitfalls

**Rendering without extents silently produces nothing.** `hb_vector_*_render()`
returns NULL, not an empty document, when the context has no extents. If you
feed glyphs with `HB_VECTOR_EXTENTS_MODE_NONE` and never call
`hb_vector_*_set_extents()` or `hb_vector_*_set_glyph_extents()`, you will get
NULL every time and nothing will tell you why.

**Render clears the context — including the extents.** Read extents with
`hb_vector_*_get_extents()` *before* rendering. This surprises people who want
to report the box of what they just produced.

**`set_extents` expands, it does not set.** Calling it twice unions the two
boxes. To replace, pass NULL first (which clears) and then the box. Also, a box
with zero width or zero height is ignored outright — neither set nor cleared.

**`set_extents` does not round-trip with `get_extents`.** The box you pass is
divided by the current scale factors and then normalised; the box you read back
is in output space. They agree only when the scale factors are `1.0` and your
box was already normalised (minimum corner, positive size).

**Scale factors divide.** `hb_vector_*_set_scale_factor(ctx, 64, 64)` makes
output *smaller*, not larger. Values `<= 0` are silently clamped to `1.0`, so a
sign flip does nothing; use the transform for mirroring.

**`hb_vector_paint_glyph()` is not `_or_fail` with the result dropped.** The
`void` variant falls back to a synthesised foreground outline for glyphs without
colour data; the `_or_fail` variant reports false and emits nothing. Picking the
wrong one gives you either unexpected monochrome output or unexpectedly empty
output. (The draw-context pair *are* related in the simple way: `void` really is
`_or_fail` with the result ignored.)

**Constructors return NULL, not a nil object.** Unlike almost every other
HarfBuzz `hb_*_create()`, there is no empty singleton here. Every call to
`hb_vector_draw_create_or_fail()` / `hb_vector_paint_create_or_fail()` needs a
NULL check, and passing `HB_VECTOR_FORMAT_INVALID` is a guaranteed NULL.

**`recycle_blob` takes ownership.** After `hb_vector_*_recycle_blob(ctx, blob)`
the blob belongs to the context. Calling `hb_blob_destroy()` on it as well is a
double free. Conversely, if you never recycle, you must destroy every rendered
blob yourself.

**`get_funcs` is borrowed and shared.** The returned `hb_draw_funcs_t` /
`hb_paint_funcs_t` is a per-format singleton, already immutable. Destroying it
or trying to install your own callbacks is a bug; the immutability means the
setter calls would silently do nothing.

**Forgetting `hb_vector_draw_new_path()` merges glyphs.** When feeding outlines
by hand, consecutive glyphs accumulate into one path and the non-zero fill rule
makes overlapping contours cancel. The `_glyph` convenience functions call it for
you; raw `hb_font_draw_glyph()` does not.

**Colliding SVG ids.** hb-vector emits very short ids. Two SVGs from two
contexts dropped into one HTML document will cross-reference each other's
gradients and clip paths. Give each context a distinct
`hb_vector_paint_set_svg_prefix()`. It has no effect on PDF.

**Semi-transparent overlapping glyphs composite separately.** Each glyph is an
independent element, so overlapping regions of a semi-transparent run darken
where they overlap rather than reading as one uniform layer. Upstream documents
this explicitly; there is no flag to change it.

**`reset` does not restore everything.** `hb_vector_draw_reset()` leaves the
foreground and background colours as they were; `hb_vector_paint_reset()` leaves
the background and the custom palette overrides in place. If you need a truly
pristine context, set those explicitly (or destroy and recreate).

**`_set_glyph_extents` wants font units, `_set_extents` wants input space.**
The first applies the full transform to all four corners; the second only
divides by the scale factors. Mixing them up puts your box in the wrong place.

**Precision is clamped at 12** and there is no error when you ask for more.
The default of 2 is coarse if you are emitting at font-unit scale.

**Nothing here is documented as thread-safe.** The contexts are mutable,
reference-counted objects with no locking. One context, one thread at a time.
