# OpenType colour fonts

Transcribed from `hb-ot-color.h`. Rust module: `harfbuzz_sys::ot_color`, glob
re-exported at the crate root.

## Overview

`hb-ot-color.h` is the *discovery* half of HarfBuzz's colour-font support. It
answers questions like "does this face have colour glyphs at all?", "which
mechanism does it use?", "what colours are in palette 2?", and "give me the PNG
for glyph 47". It does **not** draw anything. Rendering COLR glyphs is the job of
`hb-paint.h` (`hb_font_paint_glyph()` / `hb_font_paint_glyph_or_fail()`); this
header is what you call to decide *whether* that path is worth taking, and to
feed it a palette index.

OpenType grew four unrelated ways to put colour in a font, and HarfBuzz supports
all four. They are genuinely different data models, and this header exposes one
small function family per model:

- **`CPAL` — colour palettes.** A face-level array of palettes, each an array of
  BGRA colours. `CPAL` carries no glyph outlines; it is the colour source that
  `COLR` indexes into. A face may declare several palettes (a light-background
  one, a dark-background one, seasonal themes), each optionally named through the
  `name` table. Functions: `hb_ot_color_has_palettes()`,
  `hb_ot_color_palette_get_count()`, `hb_ot_color_palette_get_colors()`,
  `hb_ot_color_palette_get_flags()`, `hb_ot_color_palette_get_name_id()`,
  `hb_ot_color_palette_color_get_name_id()`.
- **`COLR` v0 — layered glyphs.** Each colour glyph is a flat list of
  (glyph ID, palette colour index) layers, drawn back to front. The layer glyphs
  are ordinary monochrome outlines already in the font. This is simple enough
  that you can implement it yourself with the layer list plus a palette.
  Functions: `hb_ot_color_has_layers()`, `hb_ot_color_glyph_get_layers()`.
- **`COLR` v1 — paint graphs.** A far richer model: gradients, transforms,
  compositing, nested paint sub-graphs. There is no "get me the layers" call
  because the data is not a list. You render it through the paint API. This
  header only tells you whether the data exists. Functions:
  `hb_ot_color_has_paint()`, `hb_ot_color_glyph_has_paint()`.
- **`SVG` — embedded SVG documents.** Each glyph (or a glyph range) maps to an
  SVG document, possibly gzip-compressed. HarfBuzz hands you the bytes; parsing
  and rasterising them is your problem. Functions: `hb_ot_color_has_svg()`,
  `hb_ot_color_get_svg_document_count()`,
  `hb_ot_color_glyph_get_svg_document_index()`,
  `hb_ot_color_get_svg_document_glyph_range()`,
  `hb_ot_color_glyph_reference_svg()`.
- **`CBDT` / `sbix` — PNG bitmaps.** Pre-rendered raster images at one or more
  sizes. This is what most emoji fonts on Apple and Google platforms use.
  Functions: `hb_ot_color_has_png()`, `hb_ot_color_glyph_reference_png()`.

The four are not mutually exclusive. Real fonts ship `COLR`+`CPAL` *and* `CBDT`,
or `SVG` *and* `sbix`, so that consumers with different capabilities each find
something they can draw. Nothing in this header ranks them; you pick the order
you prefer. HarfBuzz's own internal preference, expressed by
`hb_font_paint_glyph()`, is COLRv1 → COLRv0 → SVG → PNG → monochrome outline.

**Lifecycle.** There are no objects to create or destroy in this header. Almost
every function is a pure query on an `hb_face_t` (or, for PNG, an `hb_font_t`)
and returns a scalar or fills a caller-supplied array. The two exceptions are
`hb_ot_color_glyph_reference_svg()` and `hb_ot_color_glyph_reference_png()`,
whose names carry the usual HarfBuzz warning: `reference` in the name means the
caller receives a reference and must call `hb_blob_destroy()` on the result.

**Colour representation.** Colours are `hb_color_t`, a `uint32_t` defined in
`hb-common.h` rather than here. The gtk-doc section for `hb-ot-color` claims
`hb_color_t`, `HB_COLOR()`, and the four channel accessors even though they are
declared in `hb-common.h`; in this crate they live in `harfbuzz_sys::common`.
They are documented below for completeness, and cross-referenced in
[`common.rs`](../src/common.rs).

**Face table caching.** All of these functions read lazily-sanitised face
tables. The first call for a given table on a given face materialises it; that
is internally synchronised, so concurrent first calls from several threads are
safe. Subsequent calls are read-only.

## Types

### `hb_color_t`

*Declared in `hb-common.h`; listed in the `hb-ot-color` section. Rust module:
`harfbuzz_sys::common`.*

```c
typedef uint32_t hb_color_t;
```

```rust
pub type hb_color_t = u32;
```

A colour value: eight bits per channel, RGB plus alpha transparency.

The packing is unusual and worth committing to memory. `HB_COLOR(b,g,r,a)` is
defined as `HB_TAG(b,g,r,a)`, and `HB_TAG` puts its first argument in the most
significant byte. So the layout is:

| Bits | Channel | Accessor |
| --- | --- | --- |
| 31–24 | blue | `hb_color_get_blue()` |
| 23–16 | green | `hb_color_get_green()` |
| 15–8 | red | `hb_color_get_red()` |
| 7–0 | alpha | `hb_color_get_alpha()` |

In other words the *integer* is `0xBBGGRRAA`, and the big-endian *byte* order is
B, G, R, A. Never hand an `hb_color_t` straight to an API expecting `0xAARRGGBB`
or `0xRRGGBBAA`; use the accessors and reassemble.

Colours coming out of `hb_ot_color_palette_get_colors()` are **unpremultiplied**
RGBA, per the OpenType
[`CPAL`](https://learn.microsoft.com/en-us/typography/opentype/spec/cpal)
specification. If your compositor wants premultiplied alpha you must multiply
each of R, G, B by A/255 yourself.

Since HarfBuzz 2.1.0.

### `hb_ot_color_palette_flags_t`

```c
typedef enum { /*< flags >*/
  HB_OT_COLOR_PALETTE_FLAG_DEFAULT                      = 0x00000000u,
  HB_OT_COLOR_PALETTE_FLAG_USABLE_WITH_LIGHT_BACKGROUND = 0x00000001u,
  HB_OT_COLOR_PALETTE_FLAG_USABLE_WITH_DARK_BACKGROUND  = 0x00000002u
} hb_ot_color_palette_flags_t;
```

```rust
pub type hb_ot_color_palette_flags_t = core::ffi::c_int;
```

Flags describing the properties of a colour palette, as returned by
`hb_ot_color_palette_get_flags()`. This is a **bit field** (the C declaration is
annotated `/*< flags >*/`), so test with a bitwise AND, never with `==`.

| Constant | Value | Meaning |
| --- | --- | --- |
| `HB_OT_COLOR_PALETTE_FLAG_DEFAULT` | `0x00000000` | Nothing special to note about this palette. Also what you get when the face's `CPAL` table is version 0, which has no flags array at all. |
| `HB_OT_COLOR_PALETTE_FLAG_USABLE_WITH_LIGHT_BACKGROUND` | `0x00000001` | The palette is appropriate when displaying the font on a light background such as white. |
| `HB_OT_COLOR_PALETTE_FLAG_USABLE_WITH_DARK_BACKGROUND` | `0x00000002` | The palette is appropriate when displaying the font on a dark background such as black. |

A palette can set both usability bits (`0x3`), meaning it works either way, or
neither (`0x0`, i.e. `HB_OT_COLOR_PALETTE_FLAG_DEFAULT`), meaning the font gives
no guidance. `HB_OT_COLOR_PALETTE_FLAG_DEFAULT` is therefore *not* a value you
can match against — it is the empty bit set.

The Rust transcription is a `c_int` alias plus constants rather than a Rust
`enum`. The C enumeration has no `_MAX_VALUE` sentinel and its largest
enumerator is 2, so the C type is `int`-sized; and the value is read straight out
of font data, so a value with unknown bits set is entirely possible and would be
undefined behaviour in a Rust `enum`. Mask off the bits you understand.

Since HarfBuzz 2.1.0.

### `hb_ot_color_layer_t`

```c
typedef struct hb_ot_color_layer_t {
  hb_codepoint_t glyph;
  unsigned int   color_index;
} hb_ot_color_layer_t;
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_ot_color_layer_t {
    pub glyph: hb_codepoint_t,
    pub color_index: c_uint,
}
```

One layer of a `COLR` v0 colour glyph: a pair of glyph and palette colour index.
You obtain these by the arrayful from `hb_ot_color_glyph_get_layers()`; the
struct is plain data with no ownership and nothing to free.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `glyph` | `hb_codepoint_t` | `hb_codepoint_t` (`u32`) | The glyph ID of the layer. This is an ordinary glyph in the same face — draw it with the normal outline path. |
| `color_index` | `unsigned int` | `c_uint` | The palette colour index of the layer: an index into the array returned by `hb_ot_color_palette_get_colors()` for whichever palette you have chosen. **The special value `0xFFFF` does not refer to a palette colour** and instead means "use the current foreground/text colour". |

Layers come back in paint order, back to front: index 0 is drawn first and every
later layer paints over it.

Since HarfBuzz 2.1.0.

### `hb_ot_name_id_t` (referenced, not declared here)

`hb-ot-color.h` includes `hb-ot-name.h` for `hb_ot_name_id_t`, which is
`typedef unsigned int hb_ot_name_id_t;`. Two functions here return one. The
sentinel `HB_OT_NAME_ID_INVALID` is `0xFFFF`. Resolve a Name ID to text with
`hb_ot_name_get_utf8()` / `hb_ot_name_get_utf16()` / `hb_ot_name_get_utf32()`
from `hb-ot-name.h`; see `docs/ot_name.md`. In this crate the type and the
sentinel live in `harfbuzz_sys::ot_name` and are re-exported at the crate root,
so `use harfbuzz_sys::hb_ot_name_id_t;` works.

## Functions

### Colour channels

These four are declared in `hb-common.h`, not in `hb-ot-color.h`, but upstream
lists them in the `hb-ot-color` section because they are useless anywhere else.
In this crate they are in `harfbuzz_sys::common`.

Each is declared as a real exported function *and* shadowed by a function-like
macro of the same name, so a C caller normally gets the inline arithmetic and a
non-C caller links against the symbol. The Rust transcription binds the symbol.

#### `HB_COLOR`

```c
#define HB_COLOR(b,g,r,a) ((hb_color_t) HB_TAG ((b),(g),(r),(a)))
```

```rust
pub const fn HB_COLOR(b: u8, g: u8, r: u8, a: u8) -> hb_color_t;
```

Constructs an `hb_color_t` from four integers.

**Parameters** — `b` blue, `g` green, `r` red, `a` alpha, each 0–255.
**Note the argument order**: blue first, alpha last. It is *not* RGBA and it is
*not* ARGB. This trips up nearly everyone the first time.

**Returns** — the packed colour, `0xBBGGRRAA`.

**Ownership** — none; a pure value.

**Notes** — Since HarfBuzz 2.1.0. Transcribed as a `const fn`, so it is usable
in Rust `const` context: `const RED: hb_color_t = HB_COLOR(0, 0, 255, 255);`.

#### `hb_color_get_alpha`

```c
HB_EXTERN uint8_t hb_color_get_alpha (hb_color_t color);
#define hb_color_get_alpha(color) ((color) & 0xFF)
```

```rust
pub fn hb_color_get_alpha(color: hb_color_t) -> u8;
```

**Parameters** — `color`: the colour to inspect. By value; no nullability
question.
**Returns** — the alpha channel, 0 (transparent) to 255 (opaque).
**Ownership** — none.
**Notes** — Since HarfBuzz 2.1.0. Pure function, trivially thread-safe.

#### `hb_color_get_red`

```c
HB_EXTERN uint8_t hb_color_get_red (hb_color_t color);
#define hb_color_get_red(color) (((color) >> 8) & 0xFF)
```

```rust
pub fn hb_color_get_red(color: hb_color_t) -> u8;
```

**Parameters** — `color`: the colour to inspect.
**Returns** — the red channel, 0–255.
**Ownership** — none.
**Notes** — Since HarfBuzz 2.1.0.

#### `hb_color_get_green`

```c
HB_EXTERN uint8_t hb_color_get_green (hb_color_t color);
#define hb_color_get_green(color) (((color) >> 16) & 0xFF)
```

```rust
pub fn hb_color_get_green(color: hb_color_t) -> u8;
```

**Parameters** — `color`: the colour to inspect.
**Returns** — the green channel, 0–255.
**Ownership** — none.
**Notes** — Since HarfBuzz 2.1.0.

#### `hb_color_get_blue`

```c
HB_EXTERN uint8_t hb_color_get_blue (hb_color_t color);
#define hb_color_get_blue(color) (((color) >> 24) & 0xFF)
```

```rust
pub fn hb_color_get_blue(color: hb_color_t) -> u8;
```

**Parameters** — `color`: the colour to inspect.
**Returns** — the blue channel, 0–255.
**Ownership** — none.
**Notes** — Since HarfBuzz 2.1.0.

### Colour palettes (`CPAL`)

#### `hb_ot_color_has_palettes`

```c
HB_EXTERN hb_bool_t
hb_ot_color_has_palettes (hb_face_t *face);
```

```rust
pub fn hb_ot_color_has_palettes(face: *mut hb_face_t) -> hb_bool_t;
```

Tests whether a face includes a `CPAL` colour-palette table.

**Parameters** — `face`: the face to work upon. The header does not document
null handling and the implementation dereferences it immediately; do not pass
null. `hb_face_get_empty()` is well defined and reports no palettes.

**Returns** — `true` if data was found, `false` otherwise. Concretely, true iff
the face has a sanitisable `CPAL` table with a non-zero palette count.

**Ownership** — borrows `face`; takes no reference, allocates nothing, and there
is nothing to destroy.

**Notes** — Since HarfBuzz 2.1.0. Loads and caches the face's `CPAL` table on
first call, which is internally synchronised. A `CPAL` table on its own draws
nothing; it is the colour source for `COLR`. A font can have `CPAL` without
`COLR` (rare, and useless) or `COLR` without `CPAL` (malformed).

#### `hb_ot_color_palette_get_count`

```c
HB_EXTERN unsigned int
hb_ot_color_palette_get_count (hb_face_t *face);
```

```rust
pub fn hb_ot_color_palette_get_count(face: *mut hb_face_t) -> c_uint;
```

Fetches the number of colour palettes in a face.

**Parameters** — `face`: the face to work upon. Non-null.

**Returns** — the number of palettes found. Zero when the face has no usable
`CPAL` table — which is exactly the condition
`hb_ot_color_has_palettes()` reports, so the two are interchangeable as a
presence test.

**Ownership** — borrows `face`; nothing to free.

**Notes** — Since HarfBuzz 2.1.0. Valid palette indices are `0` through
`count - 1`. Palette 0 is the font's default and is the one
`hb_font_paint_glyph()` uses when you pass palette index 0.

#### `hb_ot_color_palette_get_flags`

```c
HB_EXTERN hb_ot_color_palette_flags_t
hb_ot_color_palette_get_flags (hb_face_t *face,
                               unsigned int palette_index);
```

```rust
pub fn hb_ot_color_palette_get_flags(
    face: *mut hb_face_t,
    palette_index: c_uint,
) -> hb_ot_color_palette_flags_t;
```

Fetches the flags defined for a colour palette.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to work upon. Non-null. |
| `palette_index` | Index of the palette, `0 ..< hb_ot_color_palette_get_count(face)`. |

**Returns** — the `hb_ot_color_palette_flags_t` bit set for that palette.
Returns `HB_OT_COLOR_PALETTE_FLAG_DEFAULT` (0) when the face's `CPAL` table is
version 0, which has no palette-flags array. Behaviour for an out-of-range
`palette_index` is not specified by the header; check the count first.

**Ownership** — borrows `face`; nothing to free.

**Notes** — Since HarfBuzz 2.1.0. Use this to pick a palette that suits your
background:

```c
flags & HB_OT_COLOR_PALETTE_FLAG_USABLE_WITH_DARK_BACKGROUND
```

Do not write `flags == HB_OT_COLOR_PALETTE_FLAG_USABLE_WITH_DARK_BACKGROUND`; a
palette usable with both backgrounds sets both bits.

#### `hb_ot_color_palette_get_name_id`

```c
HB_EXTERN hb_ot_name_id_t
hb_ot_color_palette_get_name_id (hb_face_t *face,
                                 unsigned int palette_index);
```

```rust
pub fn hb_ot_color_palette_get_name_id(
    face: *mut hb_face_t,
    palette_index: c_uint,
) -> hb_ot_name_id_t;
```

Fetches the `name`-table Name ID that provides display names for a `CPAL` colour
palette.

Palette display names can be generic (for example "Default") or provide
specific, themed names (for example "Spring", "Summer", "Fall", "Winter").

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to work upon. Non-null. |
| `palette_index` | Index of the palette, `0 ..< hb_ot_color_palette_get_count(face)`. |

**Returns** — the Name ID found for the palette. If the requested palette has no
name — including the whole-table case of a `CPAL` v0 face, which carries no
labels — the result is `HB_OT_NAME_ID_INVALID` (`0xFFFF`). Behaviour for an
out-of-range `palette_index` is unspecified.

**Ownership** — borrows `face`; the Name ID is a plain integer. Turning it into
text with `hb_ot_name_get_utf8()` is a separate call that writes into your
buffer.

**Notes** — Since HarfBuzz 2.1.0. A Name ID is only meaningful together with the
face it came from — do not cache it across faces.

#### `hb_ot_color_palette_color_get_name_id`

```c
HB_EXTERN hb_ot_name_id_t
hb_ot_color_palette_color_get_name_id (hb_face_t *face,
                                       unsigned int color_index);
```

```rust
pub fn hb_ot_color_palette_color_get_name_id(
    face: *mut hb_face_t,
    color_index: c_uint,
) -> hb_ot_name_id_t;
```

Fetches the `name`-table Name ID that provides display names for the specified
colour in a face's `CPAL` colour palette.

Display names can be generic (for example "Background") or specific (for example
"Eye color").

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to work upon. Non-null. |
| `color_index` | Index of the colour **within a palette**, i.e. the same index space as `hb_ot_color_layer_t::color_index` and as positions in the array from `hb_ot_color_palette_get_colors()`. |

Note there is **no palette parameter**. Colour labels in `CPAL` are per *entry
index*, shared by every palette in the face: entry 3 means the same thing
("Eye color") in the Spring palette and in the Winter palette, only with a
different colour value. That is why the name is
`palette_color_get_name_id` and not `palette_get_color_name_id`.

**Returns** — the Name ID found for the colour, or `HB_OT_NAME_ID_INVALID`
(`0xFFFF`) when the face records no colour labels (a `CPAL` v0 face, or a v1
face with a null label array). Behaviour for an out-of-range `color_index` is
unspecified by the header.

**Ownership** — borrows `face`; returns a plain integer.

**Notes** — Since HarfBuzz 2.1.0.

#### `hb_ot_color_palette_get_colors`

```c
HB_EXTERN unsigned int
hb_ot_color_palette_get_colors (hb_face_t    *face,
                                unsigned int  palette_index,
                                unsigned int  start_offset,
                                unsigned int *color_count,  /* IN/OUT.  May be NULL. */
                                hb_color_t   *colors        /* OUT.     May be NULL. */);
```

```rust
pub fn hb_ot_color_palette_get_colors(
    face: *mut hb_face_t,
    palette_index: c_uint,
    start_offset: c_uint,
    color_count: *mut c_uint,
    colors: *mut hb_color_t,
) -> c_uint;
```

Fetches a list of the colours in a colour palette.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to work upon. Non-null. |
| `palette_index` | Index of the palette to query, `0 ..< hb_ot_color_palette_get_count(face)`. Out of range yields a return of 0 and `*color_count = 0`. |
| `start_offset` | Index of the first colour to retrieve, within the palette. Pass 0 for the whole palette. An offset past the end simply produces nothing. |
| `color_count` | In/out, **may be null**. On input, the capacity of `colors`; on output, the number of colours actually written, which may be zero. |
| `colors` | Out, **may be null**. Receives `*color_count` `hb_color_t` values starting at `start_offset`. Caller-allocated; HarfBuzz never allocates it. |

**Returns** — the *total* number of colours in the palette, regardless of
`start_offset`, `color_count`, or `colors`. This is the number of entries per
palette, which `CPAL` requires to be the same for every palette in the face.

**Ownership** — borrows `face`; writes into the caller's `colors` array and
takes no ownership of it. Nothing to destroy.

**Notes** — Since HarfBuzz 2.1.0.

- Colours are **unpremultiplied** RGBA.
- If `colors` is NULL the function just returns the total without storing
  anything; that is the documented way to size a buffer before calling a second
  time.
- Watch the AND, not OR, in the implementation: colours are written only when
  **both** `color_count` and `colors` are non-null. Passing a real `colors`
  pointer with a null `color_count` silently writes nothing and still returns the
  total. Always pass both.
- The two-call pattern (query size, allocate, fill) and the paged pattern
  (fixed-size buffer, advance `start_offset`) are both supported; see
  [Usage](#usage).

### Layered colour glyphs (`COLR` v0)

#### `hb_ot_color_has_layers`

```c
HB_EXTERN hb_bool_t
hb_ot_color_has_layers (hb_face_t *face);
```

```rust
pub fn hb_ot_color_has_layers(face: *mut hb_face_t) -> hb_bool_t;
```

Tests whether a face includes a `COLR` table with data according to COLRv0.

**Parameters** — `face`: the face to work upon. Non-null.

**Returns** — `true` if data was found, `false` otherwise. Note the precision:
this is specifically about *v0* data. A `COLR` v1 table that happens to carry no
v0 base-glyph records returns `false` here even though the face is very much a
colour font — use `hb_ot_color_has_paint()` for that case.

**Ownership** — borrows `face`; nothing to free.

**Notes** — Since HarfBuzz 2.1.0. A COLRv1 font usually *also* provides v0
records as a fallback for older consumers, so both predicates commonly return
true for the same face.

#### `hb_ot_color_glyph_get_layers`

```c
HB_EXTERN unsigned int
hb_ot_color_glyph_get_layers (hb_face_t           *face,
                              hb_codepoint_t       glyph,
                              unsigned int         start_offset,
                              unsigned int        *layer_count, /* IN/OUT.  May be NULL. */
                              hb_ot_color_layer_t *layers       /* OUT.     May be NULL. */);
```

```rust
pub fn hb_ot_color_glyph_get_layers(
    face: *mut hb_face_t,
    glyph: hb_codepoint_t,
    start_offset: c_uint,
    layer_count: *mut c_uint,
    layers: *mut hb_ot_color_layer_t,
) -> c_uint;
```

Fetches a list of all colour layers for the specified glyph index in the
specified face. The list returned begins at the offset provided.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to work upon. Non-null. |
| `glyph` | The **glyph index** to query — not a Unicode codepoint. Run shaping, or `hb_font_get_nominal_glyph()`, first. |
| `start_offset` | Index of the first layer to retrieve. Pass 0 for all of them. |
| `layer_count` | In/out, **may be null**. Input: capacity of `layers`. Output: number of layers actually written, possibly zero. |
| `layers` | Out, **may be null**. Caller-allocated array of `hb_ot_color_layer_t`. |

**Returns** — the total number of layers available for the glyph index queried.
Zero means the glyph has no COLRv0 record — draw it as an ordinary monochrome
outline.

**Ownership** — borrows `face`; fills the caller's array. Nothing to destroy.

**Notes** — Since HarfBuzz 2.1.0.

- Same AND-not-OR rule as `hb_ot_color_palette_get_colors()`: layers are written
  only when both `layer_count` and `layers` are non-null.
- Layers are in **paint order**, back to front.
- A `color_index` of `0xFFFF` means "foreground colour", not palette entry
  65535. Handle it before you index the palette array.
- `hb_ot_color_glyph_get_layers()` gives you the composition but not the colours;
  you still need `hb_ot_color_palette_get_colors()` for a palette of your
  choosing.

### Paint colour glyphs (`COLR` v1)

#### `hb_ot_color_has_paint`

```c
HB_EXTERN hb_bool_t
hb_ot_color_has_paint (hb_face_t *face);
```

```rust
pub fn hb_ot_color_has_paint(face: *mut hb_face_t) -> hb_bool_t;
```

Tests whether a face includes a `COLR` table with data according to COLRv1.

**Parameters** — `face`: the face to work upon. Non-null.

**Returns** — `true` if data was found, `false` otherwise.

**Ownership** — borrows `face`; nothing to free.

**Notes** — Since HarfBuzz 7.0.0 (later than the rest of this header — guard
your build if you support older HarfBuzz). There is deliberately no
"get the paint graph" call: COLRv1 data is a graph of gradients, transforms and
composites, and the only supported way to consume it is to drive it through an
`hb_paint_funcs_t` with `hb_font_paint_glyph()` or
`hb_font_paint_glyph_or_fail()` from `hb-paint.h`.

#### `hb_ot_color_glyph_has_paint`

```c
HB_EXTERN hb_bool_t
hb_ot_color_glyph_has_paint (hb_face_t      *face,
                             hb_codepoint_t  glyph);
```

```rust
pub fn hb_ot_color_glyph_has_paint(
    face: *mut hb_face_t,
    glyph: hb_codepoint_t,
) -> hb_bool_t;
```

Tests whether a face includes COLRv1 paint data for `glyph`.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to work upon. Non-null. |
| `glyph` | The glyph index to query — a glyph ID, not a codepoint. |

**Returns** — `true` if data was found, `false` otherwise. This is per glyph: a
face can pass `hb_ot_color_has_paint()` and still return `false` here for most of
its glyphs.

**Ownership** — borrows `face`; nothing to free.

**Notes** — Since HarfBuzz 7.0.0. Useful for deciding, per glyph, whether to run
the (relatively expensive) paint path or fall through to plain outlines.

### SVG glyph documents (`SVG`)

#### `hb_ot_color_has_svg`

```c
HB_EXTERN hb_bool_t
hb_ot_color_has_svg (hb_face_t *face);
```

```rust
pub fn hb_ot_color_has_svg(face: *mut hb_face_t) -> hb_bool_t;
```

Tests whether a face includes any `SVG` glyph images.

**Parameters** — `face`: the face to work upon. Non-null.

**Returns** — `true` if data was found, `false` otherwise. Specifically, true iff
the `SVG` table has a document-entry list.

**Ownership** — borrows `face`; nothing to free.

**Notes** — Since HarfBuzz 2.1.0. Upstream compiles the whole SVG family out
under the reduced-feature macro `HB_NO_SVG`, although the header still declares
them unconditionally — a program can compile and then fail to link against such
a build. This crate's `build.rs` does not define `HB_NO_SVG`, so the functions
are present.

#### `hb_ot_color_get_svg_document_count`

```c
HB_EXTERN unsigned int
hb_ot_color_get_svg_document_count (hb_face_t *face);
```

```rust
pub fn hb_ot_color_get_svg_document_count(face: *mut hb_face_t) -> c_uint;
```

Gets the number of SVG documents in the face's `SVG` table.

**Parameters** — `face`: the face to work upon. Non-null.

**Returns** — the number of SVG documents in the face; 0 when there is no `SVG`
table. Valid document indices are `0 ..< count`.

**Ownership** — borrows `face`; nothing to free.

**Notes** — Since HarfBuzz 12.1.0 — much newer than the rest of the SVG family.
Documents are shared: one SVG document typically covers a *range* of glyph IDs,
so the document count is normally far smaller than the glyph count. Pair this
with `hb_ot_color_get_svg_document_glyph_range()` to enumerate the table
document-by-document instead of glyph-by-glyph.

#### `hb_ot_color_glyph_get_svg_document_index`

```c
HB_EXTERN hb_bool_t
hb_ot_color_glyph_get_svg_document_index (hb_face_t      *face,
                                          hb_codepoint_t  glyph,
                                          unsigned int   *svg_document_index /* OUT */);
```

```rust
pub fn hb_ot_color_glyph_get_svg_document_index(
    face: *mut hb_face_t,
    glyph: hb_codepoint_t,
    svg_document_index: *mut c_uint,
) -> hb_bool_t;
```

Gets the `SVG`-table document index associated with a glyph.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to work upon. Non-null. |
| `glyph` | Glyph ID to query. |
| `svg_document_index` | Out, **may be null**. Receives the document index. Written **only when the function returns true**; left untouched on a false return, so initialise it yourself if you read it unconditionally. |

**Returns** — `true` if `glyph` maps to an SVG document, `false` otherwise
(including when the face has no `SVG` table at all).

**Ownership** — borrows `face`; writes through the caller's pointer. Nothing to
free.

**Notes** — Since HarfBuzz 12.1.0. Two glyphs in the same range share a document
index; use that to deduplicate parsing and caching of large multi-glyph SVG
documents.

#### `hb_ot_color_get_svg_document_glyph_range`

```c
HB_EXTERN hb_bool_t
hb_ot_color_get_svg_document_glyph_range (hb_face_t      *face,
                                          unsigned int    svg_document_index,
                                          hb_codepoint_t *start_glyph_id, /* OUT */
                                          hb_codepoint_t *end_glyph_id /* OUT */);
```

```rust
pub fn hb_ot_color_get_svg_document_glyph_range(
    face: *mut hb_face_t,
    svg_document_index: c_uint,
    start_glyph_id: *mut hb_codepoint_t,
    end_glyph_id: *mut hb_codepoint_t,
) -> hb_bool_t;
```

Gets the glyph range covered by an `SVG`-table document index.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to work upon. Non-null. |
| `svg_document_index` | Document index, `0 ..< hb_ot_color_get_svg_document_count(face)`. |
| `start_glyph_id` | Out, **may be null**. Receives the first glyph ID covered. |
| `end_glyph_id` | Out, **may be null**. Receives the last glyph ID covered — the OpenType `SVG` document record's `endGlyphID`, which is **inclusive**. |

**Returns** — `true` if `svg_document_index` is valid, `false` otherwise
(including when the face has no `SVG` table).

**Ownership** — borrows `face`; writes through the caller's pointers. Nothing to
free.

**Notes** — Since HarfBuzz 13.0.0 — the newest function in this header. The
range is inclusive at both ends: iterate `start ..= end`, not `start .. end`.
Both output pointers being nullable makes this usable as a pure validity check.

#### `hb_ot_color_glyph_reference_svg`

```c
HB_EXTERN hb_blob_t *
hb_ot_color_glyph_reference_svg (hb_face_t *face, hb_codepoint_t glyph);
```

```rust
pub fn hb_ot_color_glyph_reference_svg(
    face: *mut hb_face_t,
    glyph: hb_codepoint_t,
) -> *mut hb_blob_t;
```

Fetches the SVG document for a glyph.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to work upon. Non-null. Note this takes a **face**, unlike the PNG counterpart. |
| `glyph` | An SVG glyph index (a glyph ID). |

**Returns** — an `hb_blob_t` containing the SVG document, if available. **If the
glyph has no SVG document, the singleton empty blob is returned** — never NULL.
Test with `hb_blob_get_length(blob) == 0` rather than a null check.

The blob may be either **plain text or gzip-encoded**. HarfBuzz does not
decompress it for you. Sniff the first two bytes for the gzip magic `0x1F 0x8B`
and inflate if present.

**Ownership** — **transfer full.** The caller owns a reference on the returned
blob and must release it with `hb_blob_destroy()`, including in the empty-blob
case (destroying the singleton is harmless). The blob's bytes are a sub-range of
the face's `SVG` table and stay valid for as long as you hold the reference.

**Notes** — Since HarfBuzz 2.1.0. A single document commonly covers many glyphs,
so calling this per glyph returns the same underlying data repeatedly; use
`hb_ot_color_glyph_get_svg_document_index()` to cache your parse.

### PNG glyph images (`CBDT` / `sbix`)

#### `hb_ot_color_has_png`

```c
HB_EXTERN hb_bool_t
hb_ot_color_has_png (hb_face_t *face);
```

```rust
pub fn hb_ot_color_has_png(face: *mut hb_face_t) -> hb_bool_t;
```

Tests whether a face has PNG glyph images, in either the `CBDT` or the `sbix`
table.

**Parameters** — `face`: the face to work upon. Non-null.

**Returns** — `true` if data was found in *either* table, `false` otherwise. The
two are not distinguished; if you need to know which, you must inspect the tables
yourself with `hb_face_reference_table()`.

**Ownership** — borrows `face`; nothing to free.

**Notes** — Since HarfBuzz 2.1.0. Takes a **face**, while the matching fetch
function takes a **font**.

#### `hb_ot_color_glyph_reference_png`

```c
HB_EXTERN hb_blob_t *
hb_ot_color_glyph_reference_png (hb_font_t *font, hb_codepoint_t glyph);
```

```rust
pub fn hb_ot_color_glyph_reference_png(
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
) -> *mut hb_blob_t;
```

Fetches the PNG image for a glyph.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | The font to work upon. Non-null. **This function takes a font object, not a face object** — the only one in this header that does, because bitmap strike selection depends on size. |
| `glyph` | A glyph index. |

**Returns** — an `hb_blob_t` containing the PNG image, if available. **If the
glyph has no PNG image, the singleton empty blob is returned** — never NULL.
Test the length, not the pointer.

To get an optimally sized PNG blob, the **PPEM values must be set on `font`**
with `hb_font_set_ppem()`. If PPEM is unset, the blob returned is the *largest*
PNG available — which for a modern emoji font can be 160×160 or more, per glyph.

**Ownership** — **transfer full.** The caller must call `hb_blob_destroy()` on
the result. The bytes belong to the font's face tables and remain valid while
you hold the reference.

**Notes** — Since HarfBuzz 2.1.0. `sbix` is consulted first; if it yields a
zero-length blob and the face has `CBDT`, `CBDT` is consulted instead. The
returned data is always a complete PNG file (starting with the PNG signature),
suitable for handing straight to a decoder. HarfBuzz does not decode it, and does
not tell you the image's pixel dimensions or its placement offsets — read the
dimensions from the PNG header and get placement from
`hb_font_get_glyph_extents()`.

## Usage

### Deciding how to draw a glyph

The canonical dispatch, mirroring HarfBuzz's own preference order:

```c
static void
draw_glyph (hb_font_t *font, hb_face_t *face, hb_codepoint_t gid)
{
  if (hb_ot_color_glyph_has_paint (face, gid))
    {
      /* COLRv1: drive the paint API. */
      hb_font_paint_glyph (font, gid, my_paint_funcs, my_data,
                           /* palette_index */ 0,
                           /* foreground */ HB_COLOR (0, 0, 0, 255));
      return;
    }

  if (hb_ot_color_has_layers (face))
    {
      unsigned int n = hb_ot_color_glyph_get_layers (face, gid, 0, NULL, NULL);
      if (n)
        {
          draw_colrv0 (face, gid, n);
          return;
        }
    }

  if (hb_ot_color_has_svg (face))
    {
      hb_blob_t *svg = hb_ot_color_glyph_reference_svg (face, gid);
      unsigned int len = hb_blob_get_length (svg);
      if (len) { draw_svg (hb_blob_get_data (svg, NULL), len); }
      hb_blob_destroy (svg);
      if (len) return;
    }

  if (hb_ot_color_has_png (face))
    {
      hb_blob_t *png = hb_ot_color_glyph_reference_png (font, gid);
      unsigned int len = hb_blob_get_length (png);
      if (len) { draw_png (hb_blob_get_data (png, NULL), len); }
      hb_blob_destroy (png);
      if (len) return;
    }

  draw_outline (font, gid);   /* monochrome fallback */
}
```

Hoist the `has_*` calls out of the per-glyph loop: they are face-level properties
and do not change between glyphs.

### Reading a whole palette (two-call pattern)

```c
unsigned int count = hb_ot_color_palette_get_colors (face, palette_index,
                                                     0, NULL, NULL);
hb_color_t *colors = malloc (count * sizeof (hb_color_t));
unsigned int n = count;
hb_ot_color_palette_get_colors (face, palette_index, 0, &n, colors);
/* n == count on success */

for (unsigned int i = 0; i < n; i++)
  printf ("#%02X%02X%02X%02X\n",
          hb_color_get_red   (colors[i]),
          hb_color_get_green (colors[i]),
          hb_color_get_blue  (colors[i]),
          hb_color_get_alpha (colors[i]));
free (colors);
```

The Rust equivalent:

```rust
use core::ffi::c_uint;
use harfbuzz_sys::{
    hb_color_get_alpha, hb_color_get_blue, hb_color_get_green, hb_color_get_red, hb_color_t,
    hb_ot_color_palette_get_colors,
};

let total = unsafe {
    hb_ot_color_palette_get_colors(face, palette_index, 0, core::ptr::null_mut(), core::ptr::null_mut())
};

let mut colors: Vec<hb_color_t> = vec![0; total as usize];
let mut n: c_uint = total;
unsafe {
    hb_ot_color_palette_get_colors(face, palette_index, 0, &mut n, colors.as_mut_ptr());
}
colors.truncate(n as usize);

for &c in &colors {
    let (r, g, b, a) = unsafe {
        (
            hb_color_get_red(c),
            hb_color_get_green(c),
            hb_color_get_blue(c),
            hb_color_get_alpha(c),
        )
    };
    println!("#{r:02X}{g:02X}{b:02X}{a:02X}");
}
```

### Choosing a palette for the current theme

```c
unsigned int want = dark_mode
  ? HB_OT_COLOR_PALETTE_FLAG_USABLE_WITH_DARK_BACKGROUND
  : HB_OT_COLOR_PALETTE_FLAG_USABLE_WITH_LIGHT_BACKGROUND;

unsigned int chosen = 0;   /* palette 0 is the font's default */
unsigned int n = hb_ot_color_palette_get_count (face);
for (unsigned int i = 0; i < n; i++)
  if (hb_ot_color_palette_get_flags (face, i) & want) { chosen = i; break; }
```

Then pass `chosen` as the `palette_index` argument of `hb_font_paint_glyph()`,
or use it with `hb_ot_color_palette_get_colors()` for the COLRv0 path.

### Naming palettes and colours

```c
char name[128];
unsigned int size = sizeof (name);
hb_ot_name_id_t id = hb_ot_color_palette_get_name_id (face, palette_index);

if (id != HB_OT_NAME_ID_INVALID &&
    hb_ot_name_get_utf8 (face, id, HB_LANGUAGE_INVALID, &size, name))
  printf ("palette %u: %s\n", palette_index, name);
```

`HB_LANGUAGE_INVALID` asks for the font's default language entry. The same
pattern with `hb_ot_color_palette_color_get_name_id (face, color_index)` labels
individual entries — useful for building a "recolour this emoji" UI.

### Rendering a COLRv0 glyph by hand

```c
unsigned int total = hb_ot_color_glyph_get_layers (face, gid, 0, NULL, NULL);
if (!total) { draw_outline (font, gid); return; }

hb_ot_color_layer_t layers[32];
unsigned int offset = 0;
while (offset < total)
  {
    unsigned int n = sizeof (layers) / sizeof (layers[0]);
    hb_ot_color_glyph_get_layers (face, gid, offset, &n, layers);
    if (!n) break;                         /* defensive: avoid an infinite loop */
    for (unsigned int i = 0; i < n; i++)
      {
        hb_color_t c = layers[i].color_index == 0xFFFF
                     ? foreground
                     : palette[layers[i].color_index];
        set_paint_color (c);
        draw_outline (font, layers[i].glyph);   /* back to front */
      }
    offset += n;
  }
```

In Rust the same paging loop, with a `MaybeUninit` buffer avoided for clarity:

```rust
use core::ffi::c_uint;
use harfbuzz_sys::{hb_ot_color_glyph_get_layers, hb_ot_color_layer_t};

let total = unsafe {
    hb_ot_color_glyph_get_layers(face, gid, 0, core::ptr::null_mut(), core::ptr::null_mut())
};

let mut layers = vec![hb_ot_color_layer_t { glyph: 0, color_index: 0 }; total as usize];
let mut n: c_uint = total;
unsafe {
    hb_ot_color_glyph_get_layers(face, gid, 0, &mut n, layers.as_mut_ptr());
}
layers.truncate(n as usize);

for layer in &layers {
    let color = if layer.color_index == 0xFFFF {
        foreground
    } else {
        palette[layer.color_index as usize]
    };
    // draw layer.glyph filled with `color`, back to front
}
```

### Enumerating SVG documents once instead of per glyph

```c
unsigned int docs = hb_ot_color_get_svg_document_count (face);
for (unsigned int d = 0; d < docs; d++)
  {
    hb_codepoint_t first, last;
    if (!hb_ot_color_get_svg_document_glyph_range (face, d, &first, &last))
      continue;

    /* `last` is inclusive. */
    hb_blob_t *svg = hb_ot_color_glyph_reference_svg (face, first);
    parse_and_cache (d, svg, first, last);
    hb_blob_destroy (svg);
  }
```

Later, for a given glyph:

```c
unsigned int d;
if (hb_ot_color_glyph_get_svg_document_index (face, gid, &d))
  draw_from_cached_document (d, gid);
```

### Fetching a size-appropriate PNG

```c
hb_font_set_ppem (font, 32, 32);         /* ask for a ~32px strike */

hb_blob_t *png = hb_ot_color_glyph_reference_png (font, gid);
unsigned int len;
const char *data = hb_blob_get_data (png, &len);
if (len)
  decode_png (data, len);
hb_blob_destroy (png);
```

Without the `hb_font_set_ppem()` call you get the largest strike in the font.

## Pitfalls

**`hb_color_t` is `0xBBGGRRAA`.** Blue in the high byte, alpha in the low byte.
`HB_COLOR()`'s parameters are `(b, g, r, a)` in that order. Nearly every graphics
API you will hand these to expects something else. Always go through
`hb_color_get_red()` and friends rather than shifting by hand from memory.

**Palette colours are unpremultiplied.** If your compositor takes premultiplied
alpha — Core Graphics, Skia's default, most GPU pipelines — you must multiply
before use, or semi-transparent layers will render too bright.

**`color_index == 0xFFFF` is not a palette entry.** It means "use the foreground
colour". Indexing your palette array with it is an out-of-bounds read. There is
no named constant for it in the header; write it out or define your own.

**The out-parameters are written only when *both* pointers are non-null.**
`hb_ot_color_palette_get_colors()` and `hb_ot_color_glyph_get_layers()` both test
`count && array`. Passing a valid array with a null count writes nothing and
still returns a plausible-looking total. This is the single most common way to
get an empty buffer from these calls.

**The return value is the total, not the number written.** Both array-fetching
functions return the full count regardless of `start_offset` and buffer capacity.
Read the number actually written from `*count` after the call, and use the return
value only for sizing and loop termination.

**`hb_ot_color_has_layers()` is COLRv0-specific.** `false` from it does not mean
"not a colour font". Check `hb_ot_color_has_paint()` too. Conversely, a COLRv1
font usually keeps v0 records as a fallback, so `true` does not mean "no v1
data".

**There is no way to read COLRv1 data through this header.** If
`hb_ot_color_glyph_has_paint()` returns true and you want fidelity, you must use
`hb-paint.h`. Falling back to the v0 layers loses gradients, transforms, and
compositing; falling back to outlines loses colour entirely.

**Empty blob, not NULL.** `hb_ot_color_glyph_reference_svg()` and
`hb_ot_color_glyph_reference_png()` return the singleton empty blob when there is
no image. `if (!blob)` never fires. Check `hb_blob_get_length()`.

**But still destroy it.** Both are `transfer full`. Leaking one blob per glyph
per frame adds up quickly; the empty singleton is refcounted and immortal, so
destroying it is safe and correct.

**SVG blobs may be gzip-compressed.** Check for the `1F 8B` magic bytes. Handing
compressed bytes to an XML parser produces a confusing parse error rather than an
obvious "this is not XML".

**PNG needs a font, and needs PPEM.** `hb_ot_color_glyph_reference_png()` is the
only function here taking `hb_font_t *`; its sibling predicate
`hb_ot_color_has_png()` takes `hb_face_t *`. Forgetting `hb_font_set_ppem()`
silently gives you the largest strike, which is a large decode and a large upload
for a 12-pixel run of text.

**SVG document ranges are inclusive.** `hb_ot_color_get_svg_document_glyph_range()`
returns the last covered glyph ID, not one past it.
`for (g = first; g <= last; g++)`.

**`svg_document_index` is untouched on failure.**
`hb_ot_color_glyph_get_svg_document_index()` writes its out-parameter only when
it returns true. Initialise the variable if you intend to read it regardless.

**Glyph IDs, not codepoints.** Every `glyph` parameter in this header is a glyph
index in the face. Passing a Unicode scalar value compiles, runs, and quietly
queries the wrong glyph.

**Version floors differ within the header.** Most of it is 2.1.0, but
`hb_ot_color_has_paint()` and `hb_ot_color_glyph_has_paint()` are 7.0.0,
`hb_ot_color_get_svg_document_count()` and
`hb_ot_color_glyph_get_svg_document_index()` are 12.1.0, and
`hb_ot_color_get_svg_document_glyph_range()` is 13.0.0. Linking against a system
HarfBuzz older than the version this crate vendors will fail on those symbols.

**Reduced-feature builds.** Upstream can compile the SVG family out with
`HB_NO_SVG` while still declaring it in the header. This crate's `build.rs` does
not set that macro, so the symbols exist here — but code you port to a
system-HarfBuzz build may not link.

**Rust-side reminders.**

- Every `extern` function is `unsafe`; this crate adds no null checks or bounds
  checks. `face` and `font` must be non-null and valid.
- `hb_ot_color_palette_flags_t` is `c_int`, so mask with `&`, and do not assume
  the value only ever contains the two documented bits.
- `hb_bool_t` is `c_int`: compare against 0, not to `true`.
- `hb_ot_color_layer_t` derives `Debug, Clone, Copy, PartialEq, Eq, Hash`, so it
  is safe to put in a `Vec` and to use as a map key.
- `hb_ot_name_id_t` comes from `harfbuzz_sys::ot_name` (re-exported at the crate
  root), not from this module.
