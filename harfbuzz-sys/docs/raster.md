# Rasterization

Header: `hb-raster.h` — Rust module: `harfbuzz_sys::raster` (gated on the
`raster` Cargo feature; **not** glob re-exported at the crate root, so you must
name the module: `use harfbuzz_sys::raster::*;`).

## Overview

The raster sub-library is HarfBuzz's own CPU rasterizer. It takes the geometry
that a font produces — outlines, or a `COLR` v0/v1 paint graph — and turns it
into a pixel buffer, with no dependency on FreeType, Cairo, Skia, or any other
graphics stack. It is optional upstream (built only when HarfBuzz is configured
with the raster sub-library) and optional here (the `raster` Cargo feature both
compiles the C++ sources and exposes this module).

Three object types carry the whole API, and all three are ordinary
reference-counted HarfBuzz objects with the usual `_reference`, `_destroy`,
`_set_user_data`, and `_get_user_data` quartet:

- **`hb_raster_image_t`** — a pixel buffer plus the `hb_raster_extents_t` and
  `hb_raster_format_t` that describe its layout. Both rasterizers produce one;
  you can also create an empty one yourself, configure it, and use it as a
  destination for compositing, or fill it from a PNG blob.
- **`hb_raster_draw_t`** — the *outline* rasterizer. It publishes an
  `hb_draw_funcs_t` that accumulates flattened outline geometry as signed edges,
  and `hb_raster_draw_render` sweeps those edges with an analytic
  area/coverage algorithm into an 8-bit alpha mask (`HB_RASTER_FORMAT_A8`).
- **`hb_raster_paint_t`** — the *colour glyph* rasterizer. It publishes an
  `hb_paint_funcs_t` that executes a paint graph — solid fills, linear/radial/
  sweep gradients, clips, transforms, composite groups — into a 32-bit
  premultiplied BGRA surface (`HB_RASTER_FORMAT_BGRA32`).

The lifecycle of a render is the same in both cases:

1. Create the rasterizer with `hb_raster_draw_create_or_fail` or
   `hb_raster_paint_create_or_fail`.
2. Configure the mapping from glyph space to pixel space: an affine transform
   (`_set_transform`) and post-transform minification factors
   (`_set_scale_factor`).
3. Set the output extents, either explicitly with `_set_extents` or by handing
   over the glyph's own bounding box with `_set_glyph_extents`, which transforms
   it for you.
4. Feed glyphs in — the one-call convenience `_glyph` / `_glyph_or_fail`, or by
   passing `_get_funcs()` and the rasterizer itself to `hb_font_draw_glyph_or_fail`
   / `hb_font_paint_glyph`.
5. Call `_render` to get an `hb_raster_image_t`.
6. Read the pixels, then either `hb_raster_image_destroy` the image or hand it
   back with `_recycle_image` so the next render reuses the allocation.

Rendering clears the one-shot state (accumulated geometry, extents) but keeps
your configuration, so step 3 onwards repeats for the next glyph. `_clear`
does the same thing explicitly; `_reset` additionally returns the transform,
scale factors, and (for paint) the foreground colour and palette overrides to
their defaults.

Two conventions are worth internalising before writing any code. First, **rows
are stored bottom-to-top**: byte offset `row * stride` is the row at
`y_origin`, and the last row in the buffer is the top of the image. Second,
**setting extents explicitly is not optional in practice** — `hb_raster_paint_render`
returns null if extents were never set, and for the draw side auto-sizing from
the accumulated edge bounding box works but gives you bounds you did not choose
and an extra pass over the geometry.

Everything in this header was introduced in HarfBuzz 13.0.0, except the PNG
serialization pair (13.1.0) and a set of accessors and convenience wrappers
added in 14.2.0 (`_get_funcs`, `_glyph`, `_glyph_or_fail`, `_clear`, and the
paint context's background/palette getters and setters). Individual entries
below record the version.

## Types

### `hb_raster_format_t`

```c
typedef enum {
  HB_RASTER_FORMAT_A8     = 0,
  HB_RASTER_FORMAT_BGRA32 = 1,
} hb_raster_format_t;
```

```rust
pub type hb_raster_format_t = core::ffi::c_int;
pub const HB_RASTER_FORMAT_A8: hb_raster_format_t = 0;
pub const HB_RASTER_FORMAT_BGRA32: hb_raster_format_t = 1;
```

Pixel format of a raster image. The C enumeration has no `/*< skip >*/`
sentinel and its largest enumerator is `1`, so it fits in an `int` and is
transcribed as `c_int` plus two constants.

| Constant | Value | Bytes/pixel | Meaning |
| --- | ---: | ---: | --- |
| `HB_RASTER_FORMAT_A8` | 0 | 1 | 8-bit alpha-only coverage. `0` = uncovered, `255` = fully covered. Output of `hb_raster_draw_render`. |
| `HB_RASTER_FORMAT_BGRA32` | 1 | 4 | 32-bit colour, **alpha-premultiplied**, stored blue-green-red-alpha in ascending byte order. Output of `hb_raster_paint_render`. |

`hb_raster_image_configure` silently substitutes `HB_RASTER_FORMAT_A8` for any
value that is neither of these, so an out-of-range format is not an error — it
is a quiet downgrade.

Note the byte order of `HB_RASTER_FORMAT_BGRA32`: the implementation reads a
pixel as a 32-bit word and takes blue from bits 0–7, green from 8–15, red from
16–23, alpha from 24–31. On a little-endian machine that is the byte sequence
`B G R A`. This matches the channel order of `hb_color_t` from `hb-common.h`
(built with `HB_COLOR(b, g, r, a)`), except that raster pixels are
premultiplied while `hb_color_t` is not.

### `hb_raster_extents_t`

```c
typedef struct hb_raster_extents_t {
  int      x_origin, y_origin;
  unsigned int width, height;
  unsigned int stride;
} hb_raster_extents_t;
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_raster_extents_t {
    pub x_origin: c_int,
    pub y_origin: c_int,
    pub width: c_uint,
    pub height: c_uint,
    pub stride: c_uint,
}
```

Describes both *where* an image sits in glyph space and *how* its bytes are
laid out. It is a plain value type — you allocate it yourself, on the stack,
and pass a pointer.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `x_origin` | `int` | `c_int` | X coordinate of the **left** edge of the image, in the post-transform pixel grid. |
| `y_origin` | `int` | `c_int` | Y coordinate of the **bottom** edge of the image, in the post-transform pixel grid. |
| `width` | `unsigned int` | `c_uint` | Width in pixels. |
| `height` | `unsigned int` | `c_uint` | Height in pixels. |
| `stride` | `unsigned int` | `c_uint` | Bytes per row. `0` on input means "compute it for me"; on output it is always the real value. |

Layout rules, from the implementation:

- The buffer is `stride * height` bytes. Row `r` (counting from `0`) starts at
  byte offset `r * stride`, and row `0` is the row at `y_origin` — **the bottom
  row**. Row `height - 1` is the top.
- Within a row, pixel `x` starts at byte `x * bytes_per_pixel`; pixel `0` is at
  `x_origin`.
- `hb_raster_image_configure` raises a `stride` of `0` — or any `stride` smaller
  than `width * bytes_per_pixel` — to exactly `width * bytes_per_pixel`. It
  never pads on your behalf beyond that. A larger `stride` you supply is
  honoured.
- `hb_raster_draw_render`, when it computes extents itself or when your extents
  carry `stride == 0`, uses `(width + 3) & ~3` — the width rounded up to a
  multiple of four — before handing them to `configure`. So an A8 image from
  the draw path is normally 4-byte-row-aligned, while one you configure by hand
  is packed.
- `hb_raster_paint_set_extents` and `hb_raster_paint_set_glyph_extents` replace
  a `stride` of `0` with `width * 4` immediately, so `hb_raster_paint_get_extents`
  never reports zero.
- The total buffer is capped at `HB_RASTER_MAX_BUFFER_SIZE`, which defaults to
  1 GiB (`(size_t) 1 << 30`) and is a HarfBuzz compile-time knob, not public
  API. `configure` also rejects `width > UINT_MAX / bytes_per_pixel` and any
  `stride * height` that would overflow `size_t`.

Since HarfBuzz 13.0.0.

### `hb_raster_image_t`

```c
typedef struct hb_raster_image_t hb_raster_image_t;
```

```rust
crate::opaque_handle! { hb_raster_image_t }
```

An opaque, reference-counted pixel buffer with an associated
`hb_raster_extents_t` and `hb_raster_format_t`. You get one from
`hb_raster_draw_render`, `hb_raster_paint_render`, or
`hb_raster_image_create_or_fail`. A freshly created image is empty: format
`HB_RASTER_FORMAT_A8`, all-zero extents, no allocation — call
`hb_raster_image_configure` before using it as a destination.

Release with `hb_raster_image_destroy`, or transfer ownership to a rasterizer
with `hb_raster_draw_recycle_image` / `hb_raster_paint_recycle_image`.

Since HarfBuzz 13.0.0.

### `hb_raster_draw_t`

```c
typedef struct hb_raster_draw_t hb_raster_draw_t;
```

```rust
crate::opaque_handle! { hb_raster_draw_t }
```

An opaque outline rasterizer. It holds an affine transform, x/y minification
factors, optional fixed extents, an accumulated edge list, and at most one
recycled image. Outlines arrive through the `hb_draw_funcs_t` from
`hb_raster_draw_get_funcs`, which flattens curves to line segments on the fly
and stores each non-horizontal segment as a signed edge in fixed point.
`hb_raster_draw_render` sweeps those edges into an `HB_RASTER_FORMAT_A8` image.

Defaults on creation: identity transform, scale factors `1.0` / `1.0`, no fixed
extents, no geometry.

Since HarfBuzz 13.0.0.

### `hb_raster_paint_t`

```c
typedef struct hb_raster_paint_t hb_raster_paint_t;
```

```rust
crate::opaque_handle! { hb_raster_paint_t }
```

An opaque colour-glyph paint context implementing the `hb_paint_funcs_t`
protocol. On top of the draw context's transform/scale/extents it carries a
foreground colour, a background colour, a palette index, a table of custom
palette colour overrides, a transform stack, a clip stack, a surface stack for
composite groups, a surface pool for reuse, and an internal `hb_raster_draw_t`
used to rasterize clip-to-glyph masks. `hb_raster_paint_render` pops the root
surface off the stack and returns it as an `HB_RASTER_FORMAT_BGRA32` image.

Defaults on creation: identity base transform, scale factors `1.0` / `1.0`, no
extents, foreground opaque black (`HB_COLOR(0, 0, 0, 255)`), background fully
transparent, palette `0`, no custom palette colours.

Since HarfBuzz 13.0.0.

## Functions

### Raster images — lifecycle

#### `hb_raster_image_create_or_fail`

```c
hb_raster_image_t *hb_raster_image_create_or_fail (void);
```

```rust
pub fn hb_raster_image_create_or_fail() -> *mut hb_raster_image_t;
```

**Parameters** — none.

**Returns** — a new image with a reference count of one, or **null** on
allocation failure. The image starts empty: format `HB_RASTER_FORMAT_A8`,
zeroed extents, no pixel storage.

**Ownership** — the caller owns the reference. Release it with
`hb_raster_image_destroy`, or transfer it with `hb_raster_draw_recycle_image` /
`hb_raster_paint_recycle_image`.

**Notes** — since HarfBuzz 13.0.0. There is no non-failing `_create` variant
and no empty-object singleton for raster images, so null really does mean
out-of-memory.

#### `hb_raster_image_reference`

```c
hb_raster_image_t *hb_raster_image_reference (hb_raster_image_t *image);
```

```rust
pub fn hb_raster_image_reference(image: *mut hb_raster_image_t) -> *mut hb_raster_image_t;
```

**Parameters** — `image`: the image whose count to raise. Nullability is
unspecified in the header; HarfBuzz's generic object reference helper tolerates
null and returns it.

**Returns** — `image`, with its reference count increased by one.

**Ownership** — the caller gains a reference that must be balanced by a call to
`hb_raster_image_destroy`.

**Notes** — since HarfBuzz 13.0.0. HarfBuzz object reference counts are atomic
in a normally configured build, so reference/destroy pairs may be issued from
different threads.

#### `hb_raster_image_destroy`

```c
void hb_raster_image_destroy (hb_raster_image_t *image);
```

```rust
pub fn hb_raster_image_destroy(image: *mut hb_raster_image_t);
```

**Parameters** — `image`: the image to release. Null is tolerated (the generic
object destroy helper checks for it), the way `free(NULL)` is.

**Returns** — nothing.

**Ownership** — consumes one reference. At zero, the pixel buffer is freed, any
user-data destroy callbacks run, and the object itself is freed. Pointers
previously obtained from `hb_raster_image_get_buffer` dangle from that moment.

**Notes** — since HarfBuzz 13.0.0.

#### `hb_raster_image_set_user_data`

```c
hb_bool_t hb_raster_image_set_user_data (hb_raster_image_t  *image,
                                         hb_user_data_key_t *key,
                                         void               *data,
                                         hb_destroy_func_t   destroy,
                                         hb_bool_t           replace);
```

```rust
pub fn hb_raster_image_set_user_data(
    image: *mut hb_raster_image_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `image` | The image to annotate. |
| `key` | The user-data key. HarfBuzz uses its **address** as the identity, never its contents, so it is normally a `static` with a stable address. |
| `data` | Arbitrary pointer to store. May be null. |
| `destroy` | Called with `data` when the image is destroyed or the entry is replaced. May be null (`None` in Rust). |
| `replace` | Non-zero to overwrite an existing entry under the same key; zero to leave an existing entry alone. |

**Returns** — true on success, false otherwise (allocation failure, or a
non-`replace` call against an occupied key).

**Ownership** — HarfBuzz stores the pointer and, if `destroy` is non-null, takes
responsibility for calling it exactly once. It does not copy `data`.

**Notes** — since HarfBuzz 13.0.0.

#### `hb_raster_image_get_user_data`

```c
void *hb_raster_image_get_user_data (const hb_raster_image_t *image,
                                     hb_user_data_key_t      *key);
```

```rust
pub fn hb_raster_image_get_user_data(
    image: *const hb_raster_image_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

**Parameters** — `image`: the image to query. `key`: the same key address that
was passed to `hb_raster_image_set_user_data`.

**Returns** — the stored pointer, or null if nothing is stored under that key.
Note that a stored value of null is indistinguishable from "not found".

**Ownership** — borrowed. The image still owns the data and will still invoke
the destroy callback; the caller must not free it.

**Notes** — since HarfBuzz 13.0.0.

### Raster images — pixels

#### `hb_raster_image_configure`

```c
hb_bool_t hb_raster_image_configure (hb_raster_image_t         *image,
                                     hb_raster_format_t         format,
                                     const hb_raster_extents_t *extents);
```

```rust
pub fn hb_raster_image_configure(
    image: *mut hb_raster_image_t,
    format: hb_raster_format_t,
    extents: *const hb_raster_extents_t,
) -> hb_bool_t;
```

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `image` | The image to configure. Dereferenced unconditionally — treat null as forbidden. |
| `format` | `HB_RASTER_FORMAT_A8` or `HB_RASTER_FORMAT_BGRA32`. Any other value is silently replaced by `HB_RASTER_FORMAT_A8`. |
| `extents` | Desired extents. **Explicitly nullable**: passing null clears the extents to all-zero and releases the backing allocation. |

**Returns** — true if the image now has the requested format and extents; false
on failure, which means one of: `width > UINT_MAX / bytes_per_pixel`,
`stride * height` overflowing `size_t`, a buffer larger than
`HB_RASTER_MAX_BUFFER_SIZE` (1 GiB by default), or the allocation itself
failing. On false the image's previous format and extents are unchanged.

**Ownership** — `extents` is read and copied; the caller keeps it. The pixel
buffer stays owned by the image.

**Notes** — since HarfBuzz 13.0.0. This sets format and extents *together* so
that storage is resized at most once. It **does not clear the pixels** — the
buffer is resized "dirty" and may contain arbitrary bytes. Follow with
`hb_raster_image_clear` if you need zeros. A `stride` of `0`, or one below
`width * bytes_per_pixel`, is raised to exactly `width * bytes_per_pixel`.

#### `hb_raster_image_clear`

```c
void hb_raster_image_clear (hb_raster_image_t *image);
```

```rust
pub fn hb_raster_image_clear(image: *mut hb_raster_image_t);
```

**Parameters** — `image`: the image to zero. Dereferenced unconditionally.

**Returns** — nothing.

**Ownership** — nothing changes hands.

**Notes** — since HarfBuzz 13.0.0. Writes zeros over `stride * height` bytes,
keeping the current extents and format. This is the natural partner to
`hb_raster_image_configure`, which deliberately leaves the buffer dirty.

#### `hb_raster_image_get_buffer`

```c
const uint8_t *hb_raster_image_get_buffer (const hb_raster_image_t *image);
```

```rust
pub fn hb_raster_image_get_buffer(image: *const hb_raster_image_t) -> *const u8;
```

**Parameters** — `image`: the image to read. Dereferenced unconditionally.

**Returns** — a pointer to the first byte of the pixel buffer, or **null** when
the image has no allocation (a freshly created or `configure(NULL)`-cleared
image). The valid length is `stride * height` bytes, using the values from
`hb_raster_image_get_extents`.

**Ownership** — borrowed, and only for as long as you hold a reference to the
image. The pointer is invalidated by `hb_raster_image_configure` (which may
reallocate), by `hb_raster_image_destroy`, and by handing the image to
`_recycle_image` — a recycled image will be reconfigured and cleared by the
next render.

**Notes** — since HarfBuzz 13.0.0. **Rows are stored bottom-to-top.** The
pointer is `const`; there is no public writable accessor. The `hb-view`-style
utilities in HarfBuzz's own tree cast the constness away to composite into an
image they configured themselves, which works but is outside what the header
promises.

#### `hb_raster_image_get_extents`

```c
void hb_raster_image_get_extents (const hb_raster_image_t *image,
                                  hb_raster_extents_t     *extents);
```

```rust
pub fn hb_raster_image_get_extents(
    image: *const hb_raster_image_t,
    extents: *mut hb_raster_extents_t,
);
```

**Parameters** — `image`: the image to query, dereferenced unconditionally.
`extents`: out-parameter, **explicitly nullable** — passing null makes the call
a no-op.

**Returns** — nothing; the result is written through `extents`.

**Ownership** — the struct is copied out by value; nothing is borrowed.

**Notes** — since HarfBuzz 13.0.0. The reported `stride` is always the real
one, never `0`. Call this before touching the buffer: the extents you asked
for and the extents you got can differ in `stride`.

#### `hb_raster_image_get_format`

```c
hb_raster_format_t hb_raster_image_get_format (const hb_raster_image_t *image);
```

```rust
pub fn hb_raster_image_get_format(image: *const hb_raster_image_t) -> hb_raster_format_t;
```

**Parameters** — `image`: the image to query. Dereferenced unconditionally.

**Returns** — `HB_RASTER_FORMAT_A8` or `HB_RASTER_FORMAT_BGRA32`. A fresh image
reports `HB_RASTER_FORMAT_A8`.

**Ownership** — none.

**Notes** — since HarfBuzz 13.0.0. Together with the extents this gives you
everything needed to interpret the buffer: bytes per pixel is 4 for
`HB_RASTER_FORMAT_BGRA32` and 1 otherwise.

### Raster images — PNG

Both functions require HarfBuzz to have been built with libpng (`HAVE_PNG`); in
this crate that is the `png` Cargo feature. Without it they compile and link
but always fail — `false` and null respectively — with no way to tell that
apart from a malformed input.

#### `hb_raster_image_deserialize_from_png_or_fail`

```c
hb_bool_t hb_raster_image_deserialize_from_png_or_fail (hb_raster_image_t *image,
                                                        hb_blob_t         *png);
```

```rust
pub fn hb_raster_image_deserialize_from_png_or_fail(
    image: *mut hb_raster_image_t,
    png: *mut hb_blob_t,
) -> hb_bool_t;
```

**Parameters** — `image`: the image to overwrite, dereferenced unconditionally.
`png`: a blob holding an encoded PNG; null is tolerated and reported as
failure, as is a zero-length blob.

**Returns** — true if the PNG decoded and the image was replaced; false
otherwise, including when libpng is absent. **On failure the image is left
completely unchanged** — the decode happens into a scratch image that is only
swapped in at the end.

**Ownership** — the blob is read, not retained; the caller keeps its reference.
The image's previous buffer is released.

**Notes** — since HarfBuzz 13.1.0. The result is always
`HB_RASTER_FORMAT_BGRA32` with origin `(0, 0)`, `width`/`height` from the PNG,
and rows stored bottom-to-top — so decoding a PNG and re-encoding it round-trips
the orientation, but reading the buffer directly gives you a vertically flipped
view compared with the file. Colour values are converted to HarfBuzz's
premultiplied representation.

#### `hb_raster_image_serialize_to_png_or_fail`

```c
hb_blob_t *hb_raster_image_serialize_to_png_or_fail (const hb_raster_image_t *image);
```

```rust
pub fn hb_raster_image_serialize_to_png_or_fail(
    image: *const hb_raster_image_t,
) -> *mut hb_blob_t;
```

**Parameters** — `image`: the image to encode. Dereferenced unconditionally.

**Returns** — a new blob holding PNG bytes, or **null** on failure. Failure
covers: libpng absent; a format other than `HB_RASTER_FORMAT_BGRA32`; a zero
`width` or `height`; and any allocation or libpng error.

**Ownership** — the caller owns the returned blob and must release it with
`hb_blob_destroy`.

**Notes** — since HarfBuzz 13.1.0. The encoder writes 8-bit RGBA, top row
first, and **un-premultiplies** each pixel (dividing by alpha, with pixels of
zero alpha written as transparent black). An A8 coverage mask cannot be encoded
this way — convert it to BGRA32 yourself first.

### Outline rasterizer — lifecycle

#### `hb_raster_draw_create_or_fail`

```c
hb_raster_draw_t *hb_raster_draw_create_or_fail (void);
```

```rust
pub fn hb_raster_draw_create_or_fail() -> *mut hb_raster_draw_t;
```

**Parameters** — none.

**Returns** — a new outline rasterizer with a reference count of one, or
**null** on allocation failure.

**Ownership** — the caller owns the reference; release it with
`hb_raster_draw_destroy`.

**Notes** — since HarfBuzz 13.0.0. The new object starts with the identity
transform, scale factors of `1.0`, no fixed extents, and no geometry.

#### `hb_raster_draw_reference`

```c
hb_raster_draw_t *hb_raster_draw_reference (hb_raster_draw_t *draw);
```

```rust
pub fn hb_raster_draw_reference(draw: *mut hb_raster_draw_t) -> *mut hb_raster_draw_t;
```

**Parameters** — `draw`: the rasterizer whose count to raise. Nullability
unspecified in the header; the generic helper tolerates null and returns it.

**Returns** — `draw`, with its reference count increased by one.

**Ownership** — the caller gains a reference to be balanced with
`hb_raster_draw_destroy`.

**Notes** — since HarfBuzz 13.0.0. Reference counting is atomic, but the
rasterizer's *contents* are not synchronised — see Pitfalls.

#### `hb_raster_draw_destroy`

```c
void hb_raster_draw_destroy (hb_raster_draw_t *draw);
```

```rust
pub fn hb_raster_draw_destroy(draw: *mut hb_raster_draw_t);
```

**Parameters** — `draw`: the rasterizer to release. Null is tolerated.

**Returns** — nothing.

**Ownership** — consumes one reference. At zero, any image previously handed
over with `hb_raster_draw_recycle_image` is destroyed, user-data callbacks run,
and the rasterizer is freed.

**Notes** — since HarfBuzz 13.0.0.

#### `hb_raster_draw_set_user_data`

```c
hb_bool_t hb_raster_draw_set_user_data (hb_raster_draw_t   *draw,
                                        hb_user_data_key_t *key,
                                        void               *data,
                                        hb_destroy_func_t   destroy,
                                        hb_bool_t           replace);
```

```rust
pub fn hb_raster_draw_set_user_data(
    draw: *mut hb_raster_draw_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

**Parameters** — identical in meaning to
`hb_raster_image_set_user_data`: `key` is identified by address, `data` may be
null, `destroy` may be null, `replace` decides whether an existing entry under
the same key is overwritten.

**Returns** — true on success, false on allocation failure or a non-`replace`
call against an occupied key.

**Ownership** — HarfBuzz stores the pointer and owns the eventual `destroy`
call.

**Notes** — since HarfBuzz 13.0.0.

#### `hb_raster_draw_get_user_data`

```c
void *hb_raster_draw_get_user_data (const hb_raster_draw_t *draw,
                                    hb_user_data_key_t     *key);
```

```rust
pub fn hb_raster_draw_get_user_data(
    draw: *const hb_raster_draw_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

**Parameters** — `draw`: the rasterizer to query. `key`: the key address used
when setting.

**Returns** — the stored pointer, or null if nothing is stored under that key.

**Ownership** — borrowed; the rasterizer still owns the data.

**Notes** — since HarfBuzz 13.0.0.

### Outline rasterizer — geometry mapping

#### `hb_raster_draw_set_transform`

```c
void hb_raster_draw_set_transform (hb_raster_draw_t *draw,
                                   float xx, float yx,
                                   float xy, float yy,
                                   float dx, float dy);
```

```rust
pub fn hb_raster_draw_set_transform(
    draw: *mut hb_raster_draw_t,
    xx: c_float, yx: c_float,
    xy: c_float, yy: c_float,
    dx: c_float, dy: c_float,
);
```

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `draw` | The rasterizer. Dereferenced unconditionally. |
| `xx`, `yx` | First column of the 2×2 linear part: `x' = xx·x + xy·y + dx`. |
| `xy`, `yy` | Second column: `y' = yx·x + yy·y + dy`. |
| `dx`, `dy` | Translation, in pixels. Use these to place the glyph at a pen position. |

**Returns** — nothing.

**Ownership** — values are copied.

**Notes** — since HarfBuzz 13.0.0. The transform is applied to every incoming
draw coordinate *before* rasterization, so it maps font units (or whatever
space the font's draw callbacks emit) to the pixel grid. The default is the
identity. There is no validation — a singular matrix simply produces empty or
degenerate output. Changing the transform does **not** retro-transform geometry
already accumulated; set it before drawing.

#### `hb_raster_draw_get_transform`

```c
void hb_raster_draw_get_transform (const hb_raster_draw_t *draw,
                                   float *xx, float *yx,
                                   float *xy, float *yy,
                                   float *dx, float *dy);
```

```rust
pub fn hb_raster_draw_get_transform(
    draw: *const hb_raster_draw_t,
    xx: *mut c_float, yx: *mut c_float,
    xy: *mut c_float, yy: *mut c_float,
    dx: *mut c_float, dy: *mut c_float,
);
```

**Parameters** — `draw`: the rasterizer. Every one of the six out-pointers is
**explicitly nullable**; nulls are skipped, so you can fetch just the
translation.

**Returns** — nothing; results are written through the pointers.

**Ownership** — none.

**Notes** — since HarfBuzz 13.0.0. Returns exactly what was set, not the
transform with the scale factors folded in.

#### `hb_raster_draw_set_scale_factor`

```c
void hb_raster_draw_set_scale_factor (hb_raster_draw_t *draw,
                                      float x_scale_factor,
                                      float y_scale_factor);
```

```rust
pub fn hb_raster_draw_set_scale_factor(
    draw: *mut hb_raster_draw_t,
    x_scale_factor: c_float,
    y_scale_factor: c_float,
);
```

**Parameters** — `draw`: the rasterizer. `x_scale_factor`, `y_scale_factor`:
post-transform *minification* factors. A factor of `n` makes the output `n`
times smaller in that axis.

**Returns** — nothing.

**Ownership** — none.

**Notes** — since HarfBuzz 13.0.0. Values that are not strictly positive
(zero, negative, NaN) are **silently replaced by `1.0`** — there is no error
report, and a later `hb_raster_draw_get_scale_factor` will show `1.0`. The
default is `1.0` on both axes. This is the mechanism for supersampling: draw at
a large transform scale, set a matching minification factor, and the rasterizer
shrinks the result during the sweep.

#### `hb_raster_draw_get_scale_factor`

```c
void hb_raster_draw_get_scale_factor (const hb_raster_draw_t *draw,
                                      float *x_scale_factor,
                                      float *y_scale_factor);
```

```rust
pub fn hb_raster_draw_get_scale_factor(
    draw: *const hb_raster_draw_t,
    x_scale_factor: *mut c_float,
    y_scale_factor: *mut c_float,
);
```

**Parameters** — `draw`: the rasterizer. Both out-pointers are **explicitly
nullable**.

**Returns** — nothing; results are written through the pointers.

**Ownership** — none.

**Notes** — since HarfBuzz 13.0.0. Reports the sanitised values actually in
effect, not what you passed in.

### Outline rasterizer — extents

#### `hb_raster_draw_set_extents`

```c
void hb_raster_draw_set_extents (hb_raster_draw_t          *draw,
                                 const hb_raster_extents_t *extents);
```

```rust
pub fn hb_raster_draw_set_extents(
    draw: *mut hb_raster_draw_t,
    extents: *const hb_raster_extents_t,
);
```

**Parameters** — `draw`: the rasterizer. `extents`: the desired output
rectangle in the post-transform pixel grid. **Dereferenced unconditionally —
null is not allowed**, unlike `hb_raster_image_configure`, which does accept it.

**Returns** — nothing.

**Ownership** — the struct is copied; the caller keeps it.

**Notes** — since HarfBuzz 13.0.0. Once set, `hb_raster_draw_render` uses these
extents verbatim instead of auto-computing a bounding box, and geometry outside
them is clipped. A `stride` of `0` is *not* normalised here — it is left alone
and resolved at render time to `(width + 3) & ~3`. The setting is one-shot:
`hb_raster_draw_render` and `hb_raster_draw_clear` both discard it.

#### `hb_raster_draw_get_extents`

```c
hb_bool_t hb_raster_draw_get_extents (const hb_raster_draw_t *draw,
                                      hb_raster_extents_t    *extents);
```

```rust
pub fn hb_raster_draw_get_extents(
    draw: *const hb_raster_draw_t,
    extents: *mut hb_raster_extents_t,
) -> hb_bool_t;
```

**Parameters** — `draw`: the rasterizer. `extents`: out-parameter, **explicitly
nullable** — pass null to test only whether extents are set.

**Returns** — true if fixed extents are currently set (and, if `extents` is
non-null, were written); false if none are set, in which case `extents` is left
untouched.

**Ownership** — none.

**Notes** — since HarfBuzz 13.0.0. Because the setting is cleared by every
render, this returns false again after `hb_raster_draw_render`.

#### `hb_raster_draw_set_glyph_extents`

```c
hb_bool_t hb_raster_draw_set_glyph_extents (hb_raster_draw_t         *draw,
                                            const hb_glyph_extents_t *glyph_extents);
```

```rust
pub fn hb_raster_draw_set_glyph_extents(
    draw: *mut hb_raster_draw_t,
    glyph_extents: *const hb_glyph_extents_t,
) -> hb_bool_t;
```

**Parameters** — `draw`: the rasterizer. `glyph_extents`: a glyph bounding box,
normally straight from `hb_font_get_glyph_extents`. Dereferenced
unconditionally — null is not allowed.

**Returns** — true if the transformed box is non-empty and was installed as the
fixed extents; **false** if it collapses to zero width or height, in which case
the rasterizer is left with *no* extents at all (any previously set extents are
discarded).

**Ownership** — the struct is read and discarded.

**Notes** — since HarfBuzz 13.0.0. The implementation normalises the box (so a
negative `height`, which is how `hb_glyph_extents_t` expresses a downward-growing
coordinate system, is handled), transforms all four corners with the
rasterizer's current transform, takes the axis-aligned bounding box of the
results, floors the minimum and ceils the maximum, and stores the result with
`stride = 0`. Set the transform *before* calling this — it uses the transform as
it stands.

### Outline rasterizer — drawing and rendering

#### `hb_raster_draw_get_funcs`

```c
hb_draw_funcs_t *hb_raster_draw_get_funcs (const hb_raster_draw_t *draw);
```

```rust
pub fn hb_raster_draw_get_funcs(draw: *const hb_raster_draw_t) -> *mut hb_draw_funcs_t;
```

**Parameters** — `draw`: the rasterizer. The argument is in fact unused by the
implementation — the funcs object is a process-wide singleton — but pass a valid
pointer anyway, since that is what the API promises.

**Returns** — the `hb_draw_funcs_t` that feeds outline data into a raster draw
context. Never null.

**Ownership** — **borrowed, and shared.** This is a static singleton owned by
HarfBuzz. Do not call `hb_draw_funcs_destroy` on it, and do not try to mutate it
with `hb_draw_funcs_set_*` — it is immutable.

**Notes** — since HarfBuzz 14.2.0. Pass the `hb_raster_draw_t *` itself as the
`draw_data` argument to whatever consumes the funcs, for example
`hb_font_draw_glyph_or_fail(font, glyph, hb_raster_draw_get_funcs(draw), draw)`.
Using it lets you drive the rasterizer from any source of outlines, not just
`hb_font_*` — for instance a `hb_draw_funcs_t`-emitting path of your own.

#### `hb_raster_draw_glyph`

```c
void hb_raster_draw_glyph (hb_raster_draw_t *draw,
                           hb_font_t        *font,
                           hb_codepoint_t    glyph);
```

```rust
pub fn hb_raster_draw_glyph(
    draw: *mut hb_raster_draw_t,
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
);
```

**Parameters** — `draw`: the rasterizer. `font`: the font to draw from; its
scale and variation settings decide the coordinate space the outlines arrive
in. `glyph`: a **glyph ID**, not a Unicode codepoint, despite the
`hb_codepoint_t` type.

**Returns** — nothing. A glyph with no outlines is silently a no-op.

**Ownership** — neither the rasterizer nor the font changes hands; no reference
is taken on `font`.

**Notes** — since HarfBuzz 14.2.0. Exactly `hb_raster_draw_glyph_or_fail` with
the result thrown away. Geometry accumulates, so calling this several times
before a single `hb_raster_draw_render` composes several glyphs into one image —
change `hb_raster_draw_set_transform`'s translation between calls to place them.
Note the parameter list: older HarfBuzz documentation shows a five-argument form
with `pen_x`/`pen_y`, which does not exist.

#### `hb_raster_draw_glyph_or_fail`

```c
hb_bool_t hb_raster_draw_glyph_or_fail (hb_raster_draw_t *draw,
                                        hb_font_t        *font,
                                        hb_codepoint_t    glyph);
```

```rust
pub fn hb_raster_draw_glyph_or_fail(
    draw: *mut hb_raster_draw_t,
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
) -> hb_bool_t;
```

**Parameters** — as `hb_raster_draw_glyph`.

**Returns** — true if the glyph was drawn; false if the font has no outline for
it (an empty glyph such as a space, a bitmap-only glyph, or a glyph ID out of
range).

**Ownership** — nothing changes hands.

**Notes** — since HarfBuzz 14.2.0. Literally
`hb_font_draw_glyph_or_fail(font, glyph, hb_raster_draw_get_funcs(draw), draw)`.
This is the variant to use when "nothing was drawn" needs to be distinguishable
from "drew an empty shape".

#### `hb_raster_draw_render`

```c
hb_raster_image_t *hb_raster_draw_render (hb_raster_draw_t *draw);
```

```rust
pub fn hb_raster_draw_render(draw: *mut hb_raster_draw_t) -> *mut hb_raster_image_t;
```

**Parameters** — `draw`: the rasterizer holding the accumulated geometry.
Dereferenced unconditionally.

**Returns** — a new `hb_raster_image_t` in `HB_RASTER_FORMAT_A8`, or **null** on
allocation or configuration failure (including extents too large for
`HB_RASTER_MAX_BUFFER_SIZE`). If no geometry was accumulated and no extents were
set, you get a valid but 0×0 image — not null.

**Ownership** — the caller owns the returned image. Release it with
`hb_raster_image_destroy`, or give it back with `hb_raster_draw_recycle_image`.
If a recycled image was previously handed over, *that* object is what comes
back, reconfigured and cleared; the rasterizer no longer holds it.

**Notes** — since HarfBuzz 13.0.0. The extents used are the fixed ones if set,
otherwise the bounding box of the accumulated edges. A `stride` of `0` becomes
`(width + 3) & ~3`. The image is cleared to zero before the sweep, so the result
contains only this render's coverage. **On every exit path — success or
failure — `hb_raster_draw_clear` runs**, discarding the accumulated edges and
the fixed extents, so the rasterizer is immediately ready for the next glyph and
you must re-set extents each time.

### Outline rasterizer — reuse

#### `hb_raster_draw_clear`

```c
void hb_raster_draw_clear (hb_raster_draw_t *draw);
```

```rust
pub fn hb_raster_draw_clear(draw: *mut hb_raster_draw_t);
```

**Parameters** — `draw`: the rasterizer. Dereferenced unconditionally.

**Returns** — nothing.

**Ownership** — nothing changes hands; internal vectors are emptied but keep
their capacity.

**Notes** — since HarfBuzz 14.2.0. Discards accumulated geometry and fixed
extents. **Preserves** the transform and the scale factors. `hb_raster_draw_render`
already does this for you, so an explicit call is only needed to abandon a
half-built glyph.

#### `hb_raster_draw_reset`

```c
void hb_raster_draw_reset (hb_raster_draw_t *draw);
```

```rust
pub fn hb_raster_draw_reset(draw: *mut hb_raster_draw_t);
```

**Parameters** — `draw`: the rasterizer. Dereferenced unconditionally.

**Returns** — nothing.

**Ownership** — nothing changes hands. A recycled image, if any, is **kept**.

**Notes** — since HarfBuzz 13.0.0. Everything `hb_raster_draw_clear` does, plus
restoring the identity transform and scale factors of `1.0`. Use it when reusing
one rasterizer for unrelated work; use `hb_raster_draw_clear` when the transform
should survive.

#### `hb_raster_draw_recycle_image`

```c
void hb_raster_draw_recycle_image (hb_raster_draw_t  *draw,
                                   hb_raster_image_t *image);
```

```rust
pub fn hb_raster_draw_recycle_image(
    draw: *mut hb_raster_draw_t,
    image: *mut hb_raster_image_t,
);
```

**Parameters** — `draw`: the rasterizer that will reuse the image. `image`: the
image to hand over. Passing null is harmless and simply drops any previously
held image.

**Returns** — nothing.

**Ownership** — **transfers ownership of `image` to `draw`.** The caller must
not use the pointer, or the buffer pointer obtained from it, afterwards. If
`draw` already holds a recycled image, that earlier image is destroyed
immediately.

**Notes** — since HarfBuzz 13.0.0. The rasterizer holds **at most one** recycled
image; the next `hb_raster_draw_render` takes it, reconfigures it to the new
extents, clears it, and returns it. This is the allocation-free render loop:
render, use the pixels, recycle, repeat. `hb_raster_draw_destroy` destroys a
still-held recycled image.

### Paint context — lifecycle

#### `hb_raster_paint_create_or_fail`

```c
hb_raster_paint_t *hb_raster_paint_create_or_fail (void);
```

```rust
pub fn hb_raster_paint_create_or_fail() -> *mut hb_raster_paint_t;
```

**Parameters** — none.

**Returns** — a new paint context with a reference count of one, or **null** on
allocation failure. Creation allocates an internal `hb_raster_draw_t` for
clip-to-glyph masks, so it can fail for that reason too.

**Ownership** — the caller owns the reference; release it with
`hb_raster_paint_destroy`.

**Notes** — since HarfBuzz 13.0.0. Defaults: identity base transform, scale
factors `1.0`, no extents, foreground `HB_COLOR(0, 0, 0, 255)` (opaque black),
background `HB_COLOR(0, 0, 0, 0)` (transparent), palette `0`, no custom palette
colours.

#### `hb_raster_paint_reference`

```c
hb_raster_paint_t *hb_raster_paint_reference (hb_raster_paint_t *paint);
```

```rust
pub fn hb_raster_paint_reference(paint: *mut hb_raster_paint_t) -> *mut hb_raster_paint_t;
```

**Parameters** — `paint`: the context whose count to raise. Nullability
unspecified; the generic helper tolerates null and returns it.

**Returns** — `paint`, with its reference count increased by one.

**Ownership** — the caller gains a reference to be balanced with
`hb_raster_paint_destroy`.

**Notes** — since HarfBuzz 13.0.0.

#### `hb_raster_paint_destroy`

```c
void hb_raster_paint_destroy (hb_raster_paint_t *paint);
```

```rust
pub fn hb_raster_paint_destroy(paint: *mut hb_raster_paint_t);
```

**Parameters** — `paint`: the context to release. Null is tolerated.

**Returns** — nothing.

**Ownership** — consumes one reference. At zero it destroys the custom-palette
map, the internal clip rasterizer, every surface still on the surface stack, and
every surface in the reuse pool — including anything handed over with
`hb_raster_paint_recycle_image` — then runs user-data callbacks and frees the
object.

**Notes** — since HarfBuzz 13.0.0.

#### `hb_raster_paint_set_user_data`

```c
hb_bool_t hb_raster_paint_set_user_data (hb_raster_paint_t  *paint,
                                         hb_user_data_key_t *key,
                                         void               *data,
                                         hb_destroy_func_t   destroy,
                                         hb_bool_t           replace);
```

```rust
pub fn hb_raster_paint_set_user_data(
    paint: *mut hb_raster_paint_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

**Parameters** — as `hb_raster_image_set_user_data`: `key` identified by
address, `data` may be null, `destroy` may be null, `replace` decides whether an
existing entry is overwritten.

**Returns** — true on success, false on allocation failure or a non-`replace`
call against an occupied key.

**Ownership** — HarfBuzz stores the pointer and owns the eventual `destroy`
call.

**Notes** — since HarfBuzz 13.0.0.

#### `hb_raster_paint_get_user_data`

```c
void *hb_raster_paint_get_user_data (const hb_raster_paint_t *paint,
                                     hb_user_data_key_t      *key);
```

```rust
pub fn hb_raster_paint_get_user_data(
    paint: *const hb_raster_paint_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

**Parameters** — `paint`: the context to query. `key`: the key address used
when setting.

**Returns** — the stored pointer, or null if nothing is stored under that key.

**Ownership** — borrowed; the context still owns the data.

**Notes** — since HarfBuzz 13.0.0.

### Paint context — geometry mapping

#### `hb_raster_paint_set_transform`

```c
void hb_raster_paint_set_transform (hb_raster_paint_t *paint,
                                    float xx, float yx,
                                    float xy, float yy,
                                    float dx, float dy);
```

```rust
pub fn hb_raster_paint_set_transform(
    paint: *mut hb_raster_paint_t,
    xx: c_float, yx: c_float,
    xy: c_float, yy: c_float,
    dx: c_float, dy: c_float,
);
```

**Parameters** — `paint`: the context. The six floats are the 2×3 affine
matrix, laid out exactly as in `hb_raster_draw_set_transform`: `x' = xx·x +
xy·y + dx`, `y' = yx·x + yy·y + dy`.

**Returns** — nothing.

**Ownership** — values are copied.

**Notes** — since HarfBuzz 13.0.0. This is the **base** transform, mapping
glyph space to pixel space. `hb_raster_paint_glyph` and
`hb_raster_paint_glyph_or_fail` push it onto the paint transform stack before
running the paint graph and pop it afterwards, so the font's own
`PaintTransform` nodes compose on top of it. The default is the identity.

#### `hb_raster_paint_get_transform`

```c
void hb_raster_paint_get_transform (const hb_raster_paint_t *paint,
                                    float *xx, float *yx,
                                    float *xy, float *yy,
                                    float *dx, float *dy);
```

```rust
pub fn hb_raster_paint_get_transform(
    paint: *const hb_raster_paint_t,
    xx: *mut c_float, yx: *mut c_float,
    xy: *mut c_float, yy: *mut c_float,
    dx: *mut c_float, dy: *mut c_float,
);
```

**Parameters** — `paint`: the context. All six out-pointers are **explicitly
nullable**.

**Returns** — nothing; results are written through the pointers.

**Ownership** — none.

**Notes** — since HarfBuzz 13.0.0. Reports the base transform only — not the
current top of the transform stack during painting, and not the scale factors.

#### `hb_raster_paint_set_scale_factor`

```c
void hb_raster_paint_set_scale_factor (hb_raster_paint_t *paint,
                                       float x_scale_factor,
                                       float y_scale_factor);
```

```rust
pub fn hb_raster_paint_set_scale_factor(
    paint: *mut hb_raster_paint_t,
    x_scale_factor: c_float,
    y_scale_factor: c_float,
);
```

**Parameters** — `paint`: the context. `x_scale_factor`, `y_scale_factor`:
post-transform minification factors; larger than one shrinks the output.

**Returns** — nothing.

**Ownership** — none.

**Notes** — since HarfBuzz 13.0.0. As on the draw side, values that are not
strictly positive are **silently replaced by `1.0`**. Unlike the draw side,
these factors *are* folded into the transform used by
`hb_raster_paint_set_glyph_extents`, so the extents it computes already account
for them.

#### `hb_raster_paint_get_scale_factor`

```c
void hb_raster_paint_get_scale_factor (const hb_raster_paint_t *paint,
                                       float *x_scale_factor,
                                       float *y_scale_factor);
```

```rust
pub fn hb_raster_paint_get_scale_factor(
    paint: *const hb_raster_paint_t,
    x_scale_factor: *mut c_float,
    y_scale_factor: *mut c_float,
);
```

**Parameters** — `paint`: the context. Both out-pointers are **explicitly
nullable**.

**Returns** — nothing; results are written through the pointers.

**Ownership** — none.

**Notes** — since HarfBuzz 13.0.0. Reports the sanitised values in effect.

### Paint context — extents

#### `hb_raster_paint_set_extents`

```c
void hb_raster_paint_set_extents (hb_raster_paint_t         *paint,
                                  const hb_raster_extents_t *extents);
```

```rust
pub fn hb_raster_paint_set_extents(
    paint: *mut hb_raster_paint_t,
    extents: *const hb_raster_extents_t,
);
```

**Parameters** — `paint`: the context. `extents`: the output pixel rectangle.
**Dereferenced unconditionally — null is not allowed.**

**Returns** — nothing.

**Ownership** — the struct is copied; the caller keeps it.

**Notes** — since HarfBuzz 13.0.0. Unlike the draw variant, this **normalises
`stride` immediately**: a `stride` of `0` becomes `width * 4`, the packed
minimum for `HB_RASTER_FORMAT_BGRA32`. Extents must be set before painting —
`hb_raster_paint_render` returns null otherwise — and every render clears them,
so set them again for each glyph.

#### `hb_raster_paint_get_extents`

```c
hb_bool_t hb_raster_paint_get_extents (const hb_raster_paint_t *paint,
                                       hb_raster_extents_t     *extents);
```

```rust
pub fn hb_raster_paint_get_extents(
    paint: *const hb_raster_paint_t,
    extents: *mut hb_raster_extents_t,
) -> hb_bool_t;
```

**Parameters** — `paint`: the context. `extents`: out-parameter, **explicitly
nullable** — pass null to test only whether extents are set.

**Returns** — true if extents are currently set (and, when `extents` is
non-null, were written); false if none are set, leaving `extents` untouched.

**Ownership** — none.

**Notes** — since HarfBuzz 13.0.0. Useful as a pre-flight check before
`hb_raster_paint_render`, whose null return is otherwise ambiguous between "no
extents" and "out of memory".

#### `hb_raster_paint_set_glyph_extents`

```c
hb_bool_t hb_raster_paint_set_glyph_extents (hb_raster_paint_t        *paint,
                                             const hb_glyph_extents_t *glyph_extents);
```

```rust
pub fn hb_raster_paint_set_glyph_extents(
    paint: *mut hb_raster_paint_t,
    glyph_extents: *const hb_glyph_extents_t,
) -> hb_bool_t;
```

**Parameters** — `paint`: the context. `glyph_extents`: a glyph bounding box,
normally from `hb_font_get_glyph_extents`. Dereferenced unconditionally — null
is not allowed.

**Returns** — true if the transformed box is non-empty and was installed;
**false** if it collapses, in which case the context is left with *no* extents.

**Ownership** — the struct is read and discarded.

**Notes** — since HarfBuzz 13.0.0. Normalises the box, transforms all four
corners with the base transform **combined with the scale factors**, takes the
axis-aligned bounding box, floors the minimum and ceils the maximum, then sets
`stride = width * 4`. Set the transform and scale factors before calling.
`hb_raster_paint_glyph` and `hb_raster_paint_glyph_or_fail` call this
automatically when no extents are set, using the glyph's own extents — which is
convenient, but silently produces a tight box you did not choose.

### Paint context — colour

All colours here are `hb_color_t` from `hb-common.h`: a 32-bit value built with
`HB_COLOR(b, g, r, a)` and taken apart with `hb_color_get_blue` and friends.
They are **not** premultiplied, unlike the pixels in a `HB_RASTER_FORMAT_BGRA32`
image; the paint context premultiplies on the way in.

#### `hb_raster_paint_set_foreground`

```c
void hb_raster_paint_set_foreground (hb_raster_paint_t *paint,
                                     hb_color_t         foreground);
```

```rust
pub fn hb_raster_paint_set_foreground(paint: *mut hb_raster_paint_t, foreground: hb_color_t);
```

**Parameters** — `paint`: the context. `foreground`: the colour to substitute
wherever the paint graph asks for the text colour.

**Returns** — nothing.

**Ownership** — the value is copied.

**Notes** — since HarfBuzz 13.0.0. Used whenever a `COLR` paint node refers to
the special "foreground" palette index — an `is_foreground` colour stop, or a
solid fill with palette index `0xFFFF` — and as the fill colour when
`hb_raster_paint_glyph` synthesizes an outline for a glyph that has no colour
data. The default is opaque black. It is passed on to
`hb_font_paint_glyph`/`hb_font_paint_glyph_or_fail` by the `_glyph` helpers;
if you drive `hb_font_paint_glyph` yourself you must pass the same colour, since
the funcs object does not read it back.

#### `hb_raster_paint_get_foreground`

```c
hb_color_t hb_raster_paint_get_foreground (const hb_raster_paint_t *paint);
```

```rust
pub fn hb_raster_paint_get_foreground(paint: *const hb_raster_paint_t) -> hb_color_t;
```

**Parameters** — `paint`: the context. Dereferenced unconditionally.

**Returns** — the foreground colour, or opaque black — `HB_COLOR(0, 0, 0, 255)`
— if none was set.

**Ownership** — none.

**Notes** — since HarfBuzz 14.2.0. `hb_raster_paint_reset` restores the
default; `hb_raster_paint_clear` does not.

#### `hb_raster_paint_set_background`

```c
void hb_raster_paint_set_background (hb_raster_paint_t *paint,
                                     hb_color_t         background);
```

```rust
pub fn hb_raster_paint_set_background(paint: *mut hb_raster_paint_t, background: hb_color_t);
```

**Parameters** — `paint`: the context. `background`: the colour to pre-fill the
surface with.

**Returns** — nothing.

**Ownership** — the value is copied.

**Notes** — since HarfBuzz 14.2.0. Only takes effect when its alpha is
non-zero: the root surface is filled with the premultiplied form of this colour
before any glyph content is composited on top. The default is fully
transparent, i.e. no pre-fill. Note that `hb_raster_paint_reset` restores the
foreground but leaves the background as you set it.

#### `hb_raster_paint_get_background`

```c
hb_color_t hb_raster_paint_get_background (const hb_raster_paint_t *paint);
```

```rust
pub fn hb_raster_paint_get_background(paint: *const hb_raster_paint_t) -> hb_color_t;
```

**Parameters** — `paint`: the context. Dereferenced unconditionally.

**Returns** — the background colour, or fully transparent —
`HB_COLOR(0, 0, 0, 0)` — if none was set.

**Ownership** — none.

**Notes** — since HarfBuzz 14.2.0.

#### `hb_raster_paint_set_palette`

```c
void hb_raster_paint_set_palette (hb_raster_paint_t *paint,
                                  unsigned           palette);
```

```rust
pub fn hb_raster_paint_set_palette(paint: *mut hb_raster_paint_t, palette: c_uint);
```

**Parameters** — `paint`: the context. `palette`: a zero-based index into the
font's `CPAL` palettes. Not validated here — an out-of-range index is resolved
(and typically ignored) further down, by the font's paint machinery.

**Returns** — nothing.

**Ownership** — none.

**Notes** — since HarfBuzz 14.2.0. The default is palette `0`. Use
`hb_ot_color_palette_get_count` from `hb-ot-color.h` to discover how many the
font has. As with the foreground colour, the `_glyph` helpers forward this to
`hb_font_paint_glyph`; a hand-rolled `hb_font_paint_glyph` call must pass it
itself.

#### `hb_raster_paint_get_palette`

```c
unsigned hb_raster_paint_get_palette (const hb_raster_paint_t *paint);
```

```rust
pub fn hb_raster_paint_get_palette(paint: *const hb_raster_paint_t) -> c_uint;
```

**Parameters** — `paint`: the context. Dereferenced unconditionally.

**Returns** — the palette index, or `0` if none was set.

**Ownership** — none.

**Notes** — since HarfBuzz 14.2.0.

#### `hb_raster_paint_set_custom_palette_color`

```c
hb_bool_t hb_raster_paint_set_custom_palette_color (hb_raster_paint_t *paint,
                                                    unsigned int       color_index,
                                                    hb_color_t         color);
```

```rust
pub fn hb_raster_paint_set_custom_palette_color(
    paint: *mut hb_raster_paint_t,
    color_index: c_uint,
    color: hb_color_t,
) -> hb_bool_t;
```

**Parameters** — `paint`: the context. `color_index`: the palette entry index to
override. `color`: the replacement colour.

**Returns** — true if the override was recorded; false on allocation failure
(the override table is an `hb_map_t` created lazily on first use).

**Ownership** — both values are copied into the context's override table.

**Notes** — since HarfBuzz 13.0.0. Overrides are keyed by index and persist
across renders until cleared or replaced for the same index. They are consulted
by every paint operation that resolves a `CPAL` entry, which is how you
recolour an emoji font without touching the font data. Survives
`hb_raster_paint_clear`; cleared by `hb_raster_paint_reset`.

#### `hb_raster_paint_clear_custom_palette_colors`

```c
void hb_raster_paint_clear_custom_palette_colors (hb_raster_paint_t *paint);
```

```rust
pub fn hb_raster_paint_clear_custom_palette_colors(paint: *mut hb_raster_paint_t);
```

**Parameters** — `paint`: the context. Dereferenced unconditionally.

**Returns** — nothing.

**Ownership** — nothing changes hands; the override table is emptied but not
freed.

**Notes** — since HarfBuzz 13.0.0. Afterwards, palette lookups use the selected
font palette with no overrides. Safe to call when no overrides were ever set.

### Paint context — painting and rendering

#### `hb_raster_paint_get_funcs`

```c
hb_paint_funcs_t *hb_raster_paint_get_funcs (const hb_raster_paint_t *paint);
```

```rust
pub fn hb_raster_paint_get_funcs(paint: *const hb_raster_paint_t) -> *mut hb_paint_funcs_t;
```

**Parameters** — `paint`: the context. As on the draw side the argument is
unused by the implementation — the funcs object is a process-wide singleton —
but pass a valid pointer.

**Returns** — the `hb_paint_funcs_t` that renders colour glyphs into a raster
paint context. Never null.

**Ownership** — **borrowed, and shared.** A static singleton owned by HarfBuzz;
do not destroy it and do not attempt to override its callbacks.

**Notes** — since HarfBuzz 14.2.0. Pass the `hb_raster_paint_t *` itself as
`paint_data`, for example
`hb_font_paint_glyph(font, glyph, hb_raster_paint_get_funcs(paint), paint, palette, foreground)`.
Driving it by hand is what you want for multi-glyph runs, because it lets you
push your own transform per glyph.

#### `hb_raster_paint_glyph`

```c
void hb_raster_paint_glyph (hb_raster_paint_t *paint,
                            hb_font_t         *font,
                            hb_codepoint_t     glyph);
```

```rust
pub fn hb_raster_paint_glyph(
    paint: *mut hb_raster_paint_t,
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
);
```

**Parameters** — `paint`: the context. `font`: the font to paint from.
`glyph`: a **glyph ID**.

**Returns** — nothing.

**Ownership** — nothing changes hands; no reference is taken on `font`.

**Notes** — since HarfBuzz 14.2.0. Two conveniences over a raw
`hb_font_paint_glyph` call. First, if no extents are set it calls
`hb_font_get_glyph_extents` and feeds the result to
`hb_raster_paint_set_glyph_extents`. Second, it pushes the base transform onto
the paint transform stack around the call and pops it after. It forwards the
context's palette index and foreground colour. Because it routes through
`hb_font_paint_glyph` rather than the `_or_fail` form, **a glyph with no colour
paint data falls back to a synthesized foreground-coloured outline**, so any
glyph with an outline or a bitmap produces output.

#### `hb_raster_paint_glyph_or_fail`

```c
hb_bool_t hb_raster_paint_glyph_or_fail (hb_raster_paint_t *paint,
                                         hb_font_t         *font,
                                         hb_codepoint_t     glyph);
```

```rust
pub fn hb_raster_paint_glyph_or_fail(
    paint: *mut hb_raster_paint_t,
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
) -> hb_bool_t;
```

**Parameters** — as `hb_raster_paint_glyph`.

**Returns** — true if painting succeeded; false otherwise — in practice, when
the glyph has no colour paint data at all, since this routes through
`hb_font_paint_glyph_or_fail` and does **not** synthesize a monochrome
fallback.

**Ownership** — nothing changes hands.

**Notes** — since HarfBuzz 14.2.0. Same auto-extents and transform push/pop
behaviour as `hb_raster_paint_glyph`. Use this one to detect "this glyph is not
a colour glyph" and fall back to `hb_raster_draw_t` yourself; use
`hb_raster_paint_glyph` when you want HarfBuzz to do that fallback for you.
Note that a false return still leaves whatever partial state the attempt
produced, so pair it with `hb_raster_paint_clear` if you are abandoning the
render.

#### `hb_raster_paint_render`

```c
hb_raster_image_t *hb_raster_paint_render (hb_raster_paint_t *paint);
```

```rust
pub fn hb_raster_paint_render(paint: *mut hb_raster_paint_t) -> *mut hb_raster_image_t;
```

**Parameters** — `paint`: the context holding the painted surface.
Dereferenced unconditionally.

**Returns** — a new `hb_raster_image_t` in `HB_RASTER_FORMAT_BGRA32`, or
**null**. Null means either that no extents were set before painting, or that
allocation or configuration failed — the two are not distinguishable from the
return value alone, so check `hb_raster_paint_get_extents` first if you care.
If extents were set but nothing was painted, you get a valid image filled with
the background colour (transparent by default), not null.

**Ownership** — the caller owns the returned image. Release it with
`hb_raster_image_destroy`, or return it to the pool with
`hb_raster_paint_recycle_image`.

**Notes** — since HarfBuzz 13.0.0. This is an *extraction*, not a rasterization
pass: the pixels were produced by the paint callbacks as they ran, and this
function pops the root surface off the surface stack (releasing any stray group
surfaces above it). **On every exit path — including the two failure paths —
`hb_raster_paint_clear` runs**, so the extents, transform stack, clip stack, and
surface stack are all reset and the context is ready for the next glyph.

### Paint context — reuse

#### `hb_raster_paint_clear`

```c
void hb_raster_paint_clear (hb_raster_paint_t *paint);
```

```rust
pub fn hb_raster_paint_clear(paint: *mut hb_raster_paint_t);
```

**Parameters** — `paint`: the context. Dereferenced unconditionally.

**Returns** — nothing.

**Ownership** — surfaces on the surface stack are returned to the internal
pool, not destroyed.

**Notes** — since HarfBuzz 14.2.0. Clears the extents, the transform stack, the
clip stack, and the surface stack, and resets the internal clip rasterizer.
**Preserves** the base transform, scale factors, foreground colour, background
colour, palette index, and custom palette colours. `hb_raster_paint_render`
already does this, so call it explicitly only to abandon a partial render.

#### `hb_raster_paint_reset`

```c
void hb_raster_paint_reset (hb_raster_paint_t *paint);
```

```rust
pub fn hb_raster_paint_reset(paint: *mut hb_raster_paint_t);
```

**Parameters** — `paint`: the context. Dereferenced unconditionally.

**Returns** — nothing.

**Ownership** — internal image caches are **kept**, so the next render is still
allocation-free.

**Notes** — since HarfBuzz 13.0.0. Restores the identity base transform, scale
factors of `1.0`, and opaque-black foreground; clears the custom palette
colours; then does everything `hb_raster_paint_clear` does. Read the list
carefully: as implemented it does **not** reset the background colour or the
palette index, even though the documentation calls it "clearing all
configuration".

#### `hb_raster_paint_recycle_image`

```c
void hb_raster_paint_recycle_image (hb_raster_paint_t *paint,
                                    hb_raster_image_t *image);
```

```rust
pub fn hb_raster_paint_recycle_image(
    paint: *mut hb_raster_paint_t,
    image: *mut hb_raster_image_t,
);
```

**Parameters** — `paint`: the context that will reuse the image. `image`: the
image to hand over.

**Returns** — nothing.

**Ownership** — **transfers ownership of `image` to `paint`.** The caller must
not touch the pointer, or any buffer pointer derived from it, afterwards.

**Notes** — since HarfBuzz 13.0.0. Unlike the draw side's single recycled slot,
this pushes onto an **unbounded surface pool** that the paint machinery also
draws on for composite-group surfaces; if the push itself fails to allocate, the
image is destroyed instead. Every pooled surface is destroyed by
`hb_raster_paint_destroy`. Recycled surfaces are reconfigured to the current
extents and cleared before reuse, so stale contents never leak into a render.

## Usage

### Rasterizing one glyph to an alpha mask (C)

```c
#include <hb.h>
#include <hb-raster.h>

hb_blob_t *blob = hb_blob_create_from_file_or_fail ("font.ttf");
hb_face_t *face = hb_face_create (blob, 0);
hb_font_t *font = hb_font_create (face);
hb_font_set_scale (font, 64, 64);          /* 64 px em */

hb_raster_draw_t *draw = hb_raster_draw_create_or_fail ();
if (!draw) return;

/* Glyph space -> pixel space. Identity here because hb_font_set_scale
   already put the outlines in pixels. */
hb_raster_draw_set_transform (draw, 1.f, 0.f, 0.f, 1.f, 0.f, 0.f);
hb_raster_draw_set_scale_factor (draw, 1.f, 1.f);

/* Choose deterministic bounds from the glyph's own box. */
hb_codepoint_t gid = 42;
hb_glyph_extents_t gext;
if (hb_font_get_glyph_extents (font, gid, &gext))
  hb_raster_draw_set_glyph_extents (draw, &gext);

hb_raster_draw_glyph (draw, font, gid);

hb_raster_image_t *mask = hb_raster_draw_render (draw);
if (mask)
{
  hb_raster_extents_t ext;
  hb_raster_image_get_extents (mask, &ext);
  const uint8_t *buf = hb_raster_image_get_buffer (mask);

  /* Rows are bottom-to-top: row 0 sits at ext.y_origin. */
  for (unsigned row = 0; row < ext.height; row++)
  {
    const uint8_t *p = buf + (size_t) row * ext.stride;
    for (unsigned x = 0; x < ext.width; x++)
      (void) p[x];                         /* coverage, 0..255 */
  }

  /* Hand the buffer back instead of freeing it: the next render reuses it. */
  hb_raster_draw_recycle_image (draw, mask);
}

hb_raster_draw_destroy (draw);
hb_font_destroy (font);
hb_face_destroy (face);
hb_blob_destroy (blob);
```

### Rasterizing one glyph to an alpha mask (Rust)

```rust
use core::ffi::c_char;
use harfbuzz_sys::raster::*;
use harfbuzz_sys::{
    hb_blob_create_from_file_or_fail, hb_blob_destroy, hb_face_create, hb_face_destroy,
    hb_font_create, hb_font_destroy, hb_font_get_glyph_extents, hb_font_set_scale,
    hb_glyph_extents_t,
};

/// Renders one glyph and returns the coverage mask as (bottom-up rows, extents).
unsafe fn render_mask(path: &core::ffi::CStr, gid: u32) -> Option<(alloc::vec::Vec<u8>, hb_raster_extents_t)> {
    let blob = unsafe { hb_blob_create_from_file_or_fail(path.as_ptr() as *const c_char) };
    if blob.is_null() {
        return None;
    }
    let face = unsafe { hb_face_create(blob, 0) };
    let font = unsafe { hb_font_create(face) };
    unsafe { hb_font_set_scale(font, 64, 64) };

    let draw = unsafe { hb_raster_draw_create_or_fail() };
    if draw.is_null() {
        unsafe {
            hb_font_destroy(font);
            hb_face_destroy(face);
            hb_blob_destroy(blob);
        }
        return None;
    }

    unsafe {
        hb_raster_draw_set_transform(draw, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0);

        let mut gext = core::mem::MaybeUninit::<hb_glyph_extents_t>::uninit();
        if hb_font_get_glyph_extents(font, gid, gext.as_mut_ptr()) != 0 {
            hb_raster_draw_set_glyph_extents(draw, gext.as_ptr());
        }

        hb_raster_draw_glyph(draw, font, gid);
    }

    let image = unsafe { hb_raster_draw_render(draw) };
    let out = if image.is_null() {
        None
    } else {
        unsafe {
            let mut ext = core::mem::MaybeUninit::<hb_raster_extents_t>::uninit();
            hb_raster_image_get_extents(image, ext.as_mut_ptr());
            let ext = ext.assume_init();

            let buf = hb_raster_image_get_buffer(image);
            let len = ext.stride as usize * ext.height as usize;
            let pixels = if buf.is_null() || len == 0 {
                alloc::vec::Vec::new()
            } else {
                core::slice::from_raw_parts(buf, len).to_vec()
            };

            // Copy taken; give the allocation back to the rasterizer.
            hb_raster_draw_recycle_image(draw, image);
            Some((pixels, ext))
        }
    };

    unsafe {
        hb_raster_draw_destroy(draw);
        hb_font_destroy(font);
        hb_face_destroy(face);
        hb_blob_destroy(blob);
    }
    out
}
```

`harfbuzz-sys` is `#![no_std]`, so the `alloc::vec::Vec` above assumes the
caller's crate has `alloc` (or `std`) available; the FFI layer itself never
allocates on the Rust side.

### Indexing pixels correctly

The one thing worth writing a helper for. Rows run bottom-to-top, and `stride`
is not necessarily `width * bytes_per_pixel`.

```rust
use harfbuzz_sys::raster::{hb_raster_extents_t, hb_raster_format_t,
                           HB_RASTER_FORMAT_BGRA32};

fn bytes_per_pixel(format: hb_raster_format_t) -> usize {
    if format == HB_RASTER_FORMAT_BGRA32 { 4 } else { 1 }
}

/// Byte offset of pixel (`x`, `y`) where `y` counts **down from the top**,
/// the convention most image code expects.
fn offset_top_down(ext: &hb_raster_extents_t, format: hb_raster_format_t, x: u32, y: u32) -> usize {
    debug_assert!(x < ext.width && y < ext.height);
    let row_from_bottom = (ext.height - 1 - y) as usize;
    row_from_bottom * ext.stride as usize + x as usize * bytes_per_pixel(format)
}

/// Un-premultiply one BGRA32 pixel into straight (r, g, b, a).
fn unpremultiply(px: [u8; 4]) -> (u8, u8, u8, u8) {
    let (b, g, r, a) = (px[0] as u32, px[1] as u32, px[2] as u32, px[3] as u32);
    if a == 0 {
        return (0, 0, 0, 0);
    }
    let f = |c: u32| ((c * 255 + a / 2) / a).min(255) as u8;
    (f(r), f(g), f(b), a as u8)
}
```

### Rendering a colour glyph and saving it as PNG (C)

```c
hb_raster_paint_t *paint = hb_raster_paint_create_or_fail ();

hb_raster_paint_set_transform (paint, 1.f, 0.f, 0.f, 1.f, 0.f, 0.f);
hb_raster_paint_set_scale_factor (paint, 1.f, 1.f);
hb_raster_paint_set_foreground (paint, HB_COLOR (0, 0, 0, 255));  /* opaque black */
hb_raster_paint_set_background (paint, HB_COLOR (255, 255, 255, 255)); /* opaque white */
hb_raster_paint_set_palette (paint, 0);

hb_glyph_extents_t gext;
if (hb_font_get_glyph_extents (font, gid, &gext))
  hb_raster_paint_set_glyph_extents (paint, &gext);

if (hb_raster_paint_glyph_or_fail (paint, font, gid))
{
  hb_raster_image_t *img = hb_raster_paint_render (paint);   /* BGRA32 */
  if (img)
  {
    hb_blob_t *png = hb_raster_image_serialize_to_png_or_fail (img);
    if (png)
    {
      unsigned len;
      const char *data = hb_blob_get_data (png, &len);
      fwrite (data, 1, len, out);
      hb_blob_destroy (png);
    }
    hb_raster_paint_recycle_image (paint, img);
  }
}
else
{
  /* Not a colour glyph — fall back to the outline rasterizer, or call
     hb_raster_paint_glyph, which synthesizes a foreground-coloured outline. */
  hb_raster_paint_clear (paint);
}

hb_raster_paint_destroy (paint);
```

### Recolouring a colour font

```c
/* Replace palette entry 3 with red, everywhere it is used. */
hb_raster_paint_set_palette (paint, 0);
hb_raster_paint_clear_custom_palette_colors (paint);
if (!hb_raster_paint_set_custom_palette_color (paint, 3, HB_COLOR (0, 0, 255, 255)))
  /* allocation failure */;
```

Overrides survive across renders and across `hb_raster_paint_clear`; only
`hb_raster_paint_clear_custom_palette_colors` and `hb_raster_paint_reset` drop
them.

### Composing a whole run into one image

Neither rasterizer positions glyphs for you — that is what the transform's
translation is for. Set extents once to cover the whole run, then move the pen
between glyphs. For the draw path, accumulate everything and render once:

```c
hb_raster_extents_t run_ext = { 0, -12, 320, 48, 0 };  /* x0, y0, w, h, stride */
hb_raster_draw_set_extents (draw, &run_ext);

float pen_x = 0.f, pen_y = 0.f;
for (unsigned i = 0; i < glyph_count; i++)
{
  hb_raster_draw_set_transform (draw, 1.f, 0.f, 0.f, 1.f, pen_x, pen_y);
  hb_raster_draw_glyph (draw, font, infos[i].codepoint);
  pen_x += positions[i].x_advance / 64.f;
  pen_y += positions[i].y_advance / 64.f;
}

hb_raster_image_t *run = hb_raster_draw_render (draw);   /* one A8 mask */
```

The paint path cannot accumulate the same way — `hb_raster_paint_render`
consumes the surface stack — so render each glyph separately and composite the
results into an image you configured yourself:

```c
hb_raster_extents_t out_ext = { 0, -12, 320, 48, 0 };
hb_raster_image_t *canvas = hb_raster_image_create_or_fail ();
if (!hb_raster_image_configure (canvas, HB_RASTER_FORMAT_BGRA32, &out_ext))
  /* too large, or out of memory */;
hb_raster_image_clear (canvas);   /* configure leaves the buffer dirty */
/* ... render each glyph with hb_raster_paint_render and blend it in ... */
hb_raster_image_destroy (canvas);
```

### Driving the rasterizers from your own geometry

`hb_raster_draw_get_funcs` and `hb_raster_paint_get_funcs` expose the callback
tables directly, so any producer of `hb_draw_funcs_t` / `hb_paint_funcs_t`
calls can target the rasterizer:

```c
hb_draw_funcs_t *dfuncs = hb_raster_draw_get_funcs (draw);
hb_font_draw_glyph_or_fail (font, gid, dfuncs, draw);      /* the rasterizer is the draw_data */

hb_paint_funcs_t *pfuncs = hb_raster_paint_get_funcs (paint);
hb_font_paint_glyph (font, gid, pfuncs, paint,
                     hb_raster_paint_get_palette (paint),
                     hb_raster_paint_get_foreground (paint));
```

Doing it this way is necessary whenever you need to push your own transform
around the glyph, or to emit paths that did not come from a font. Note that
`hb_font_paint_glyph` takes the palette and foreground as arguments — the funcs
object does not read them back off the context, which is why the snippet passes
them explicitly.

## Pitfalls

### Rows are stored bottom-to-top

This is the single most common source of upside-down output. Row `0` of the
buffer is the row at `y_origin`, the **bottom** of the image. Most image
formats and GPU upload paths want the top row first, so you either iterate in
reverse or flip during the copy. `hb_raster_image_serialize_to_png_or_fail`
flips for you; `hb_raster_image_deserialize_from_png_or_fail` flips on the way
in. Reading the buffer directly does not.

### Extents are one-shot and every render clears them

Both `hb_raster_draw_render` and `hb_raster_paint_render` run their `_clear`
function on **every** exit path, including failures. That means the fixed
extents you set are gone afterwards, and `hb_raster_draw_get_extents` /
`hb_raster_paint_get_extents` will report false. In a render loop you must set
extents again for each glyph. Forgetting to on the paint side turns into a null
return from `hb_raster_paint_render` on the second iteration.

### `hb_raster_paint_render` returns null for two different reasons

Null means "extents were never set" *or* "allocation/configuration failed", with
no way to tell from the return value. If you need to distinguish them, check
`hb_raster_paint_get_extents(paint, NULL)` before painting.

### `hb_raster_image_configure` does not clear the buffer

The storage is resized "dirty" for speed. A newly configured image contains
whatever was in memory — often the previous render's pixels, if the allocation
was reused. Call `hb_raster_image_clear` before compositing into it. The
rasterizers do this internally for the images they return, so only
hand-configured images are affected.

### Ownership transfer in `_recycle_image` is total

After `hb_raster_draw_recycle_image` or `hb_raster_paint_recycle_image`, the
image pointer is dead to you, and so is any pointer previously obtained from
`hb_raster_image_get_buffer`. The next render reconfigures and clears that
buffer. Copy out anything you still need first. In Rust this is exactly the
shape that a `Drop` impl gets wrong: a wrapper that both recycles *and* destroys
is a double free.

### `hb_raster_image_get_buffer` returns a pointer that moves

`hb_raster_image_configure` may reallocate, so a buffer pointer taken before a
`configure` call must not be used after it. The pointer is also `const`: there
is no public writable accessor, and HarfBuzz's own tools cast the constness away
to composite into images they configured themselves. That works in practice but
is outside the header's contract.

### Invalid scale factors are silently corrected

`hb_raster_draw_set_scale_factor` and `hb_raster_paint_set_scale_factor` replace
any value that is not strictly greater than zero — including `0.0`, negatives,
and NaN — with `1.0`, with no diagnostic. A getter afterwards reports `1.0`, so
a typo shows up as "the image is the wrong size" rather than as an error.

### An out-of-range format is silently downgraded

`hb_raster_image_configure` substitutes `HB_RASTER_FORMAT_A8` for any value that
is neither of the two constants. Passing a garbage format therefore produces a
valid, wrongly sized image rather than a failure. This is also why
`hb_raster_format_t` is transcribed as an integer alias rather than a Rust
`enum` — the C API tolerates values outside the enumeration, and a Rust `enum`
holding one would be undefined behaviour.

### `_glyph` takes a glyph ID, not a character

`hb_raster_draw_glyph`, `hb_raster_draw_glyph_or_fail`,
`hb_raster_paint_glyph`, and `hb_raster_paint_glyph_or_fail` all take an
`hb_codepoint_t`, but the value is a **glyph index** in the font — normally
`hb_glyph_info_t::codepoint` after shaping, or the result of
`hb_font_get_nominal_glyph`. Passing a Unicode scalar renders whatever glyph
happens to sit at that index.

Related: HarfBuzz's own section documentation for `hb-raster` shows
`hb_raster_draw_glyph (draw, font, gid, pen_x, pen_y)`. That five-argument form
does not exist; positioning is done with the transform's translation.

### `hb_raster_paint_glyph` never reports "not a colour glyph"

It falls back to a synthesized foreground-coloured outline, so it produces
output for any glyph that has an outline or a bitmap. If you need to know
whether the glyph actually had `COLR` data — to route it to the cheaper A8 path,
for instance — use `hb_raster_paint_glyph_or_fail` and check the result.

### `hb_raster_paint_reset` does not reset everything

Despite the name and the upstream wording ("clearing all configuration"), the
implementation restores the base transform, scale factors, and foreground
colour, and clears the custom palette colours — but leaves the **background
colour** and the **palette index** as they were. Set those explicitly if you are
recycling a context across unrelated jobs.

### The PNG functions fail silently without libpng

`hb_raster_image_serialize_to_png_or_fail` returns null and
`hb_raster_image_deserialize_from_png_or_fail` returns false when HarfBuzz was
built without `HAVE_PNG` — the same results as a malformed image. In this crate
that means the `png` Cargo feature. Serialization additionally requires
`HB_RASTER_FORMAT_BGRA32` and a non-zero width and height, so an A8 coverage
mask can never be encoded directly.

### Size limits are implementation behaviour, not API

`hb_raster_image_configure` rejects any buffer larger than
`HB_RASTER_MAX_BUFFER_SIZE`, 1 GiB by default, and rejects extents whose
`width * bytes_per_pixel` or `stride * height` would overflow. None of that is
in the header; it is a HarfBuzz build-time constant. A very large glyph at a
very large scale therefore surfaces as a null return from `_render`, not as a
crash.

### Threading

The header says nothing about thread safety. Reference counts are atomic in a
normally configured build, so `_reference` and `_destroy` may be called from
several threads. Everything else mutates the object: the transform, the scale
factors, the extents, the accumulated geometry, the palette overrides, and the
internal surface pool. **Confine each `hb_raster_draw_t` and `hb_raster_paint_t`
to one thread**, or give each thread its own. The funcs singletons from
`_get_funcs` are immutable and safe to share. The Rust handles are `!Send` and
`!Sync` by construction, which matches this.

### Null tolerance is mostly unspecified

The header carries no nullability annotations. From the implementation:
`_destroy` and `_reference` tolerate null the way `free` does;
`hb_raster_image_configure` explicitly accepts a null `extents`; the `_get_extents`,
`_get_transform`, and `_get_scale_factor` out-parameters are explicitly
nullable; `hb_raster_image_deserialize_from_png_or_fail` tolerates a null blob
and reports failure. Everything else dereferences its object argument
unconditionally — in particular `hb_raster_draw_set_extents`,
`hb_raster_paint_set_extents`, and both `_set_glyph_extents` functions require a
non-null struct pointer, so do not assume the `configure` precedent generalises.
