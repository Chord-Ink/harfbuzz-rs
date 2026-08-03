# Style

Transcribed from `hb-style.h`. Rust module: `harfbuzz_sys::style`, glob
re-exported at the crate root.

## Overview

`hb-style.h` answers one question: *how italic / how bold / how wide / how large
is this font, as a number?* It exposes a single enumeration of style axes and a
single function that reads one of them off an `hb_font_t`. There are no objects
to create, no reference counts, and no memory to free.

The interesting part is not the API surface but the lookup it performs. A font's
style is recorded in several places that historically disagree with each other:
the `fvar` variation axes of a variable font, the `STAT` table's axis-value
records, the OS/2 table's `usWeightClass` / `usWidthClass` / `fsSelection`, the
`head` table's `macStyle` bits, the `post` table's `italicAngle`, and the GPOS
`size` feature. `hb_style_get_value()` consults them in a fixed priority order
and returns a single float, so the caller does not have to know which of those
tables a given font happens to have filled in. That is the value the header
adds — the axis tags themselves are just OpenType registered tags you could have
written by hand.

Style values are **per-font**, not per-face. This matters because two of the
inputs live on `hb_font_t` rather than on the face: the design coordinates set by
`hb_font_set_variations()` / `hb_font_set_var_coords_design()`, and the
point size set by `hb_font_set_ptem()` and the synthetic slant set by
`hb_font_set_synthetic_slant()`. Two fonts created from the same face can
therefore report different weights, widths, optical sizes, and slants. A font
sub-font (`hb_font_create_sub_font()`) inherits its parent's coordinates and so
usually reports the same values until you change them.

The axis tags in `hb_style_tag_t` come from the
[OpenType Design-Variation Axis Tag Registry](https://docs.microsoft.com/en-us/typography/opentype/spec/dvaraxisreg),
with one exception. `HB_STYLE_TAG_SLANT_RATIO` is spelled `Slnt` with a capital
`S` — it is not a real OpenType axis and never appears in font data. It is
HarfBuzz's way of asking for the slant as a ratio (a tangent) rather than as an
angle in degrees, because that is the form a rendering back end typically wants
for a shear matrix.

Upstream compiles this entire file out under `HB_NO_STYLE`, a reduced-feature
build option. The header still declares `hb_style_get_value()` unconditionally,
so a program can compile against the header and still fail to link against such
a build. The default configuration used by this crate's `build.rs` includes it.

## Types

### `hb_style_tag_t`

```c
typedef enum {
  HB_STYLE_TAG_ITALIC       = HB_TAG ('i','t','a','l'),
  HB_STYLE_TAG_OPTICAL_SIZE = HB_TAG ('o','p','s','z'),
  HB_STYLE_TAG_SLANT_ANGLE  = HB_TAG ('s','l','n','t'),
  HB_STYLE_TAG_SLANT_RATIO  = HB_TAG ('S','l','n','t'),
  HB_STYLE_TAG_WIDTH        = HB_TAG ('w','d','t','h'),
  HB_STYLE_TAG_WEIGHT       = HB_TAG ('w','g','h','t'),

  /*< private >*/
  _HB_STYLE_TAG_MAX_VALUE   = HB_TAG_MAX_SIGNED /*< skip >*/
} hb_style_tag_t;
```

```rust
pub type hb_style_tag_t = core::ffi::c_int;
```

A style axis to query. Values are four-byte tags, so this is really an
`hb_tag_t` in disguise.

The Rust transcription is a `c_int` alias plus constants rather than a Rust
`enum`. Two reasons: the C enumeration's private sentinel is `HB_TAG_MAX_SIGNED`
(`0x7FFFFFFF`), which fixes the underlying C type at signed `int`; and the value
space is deliberately open. `hb_style_get_value()` looks the tag up among the
font's `fvar` axes *before* it looks at anything else, so passing an axis tag
that has no constant here — `GRAD`, `XTRA`, `CASL`, any custom axis — is
meaningful and supported. A Rust `enum` holding such a value would be undefined
behaviour.

| Constant | Tag | Meaning | Typical range |
| --- | --- | --- | --- |
| `HB_STYLE_TAG_ITALIC` | `ital` | Non-italic to italic. 0 reads as "Roman", 1 as fully italic. | 0 – 1 |
| `HB_STYLE_TAG_OPTICAL_SIZE` | `opsz` | Design tuned for a text size. Non-zero; read as points. | ~6 – 144 |
| `HB_STYLE_TAG_SLANT_ANGLE` | `slnt` | Oblique slant in counter-clockwise degrees from the designer's upright. Must be > -90 and < +90. Right-leaning italics are **negative**, typically about -12. | -90 – +90 |
| `HB_STYLE_TAG_SLANT_RATIO` | `Slnt` | The same slant expressed as a ratio. Right-leaning italics are **positive**, typically about 0.2. HarfBuzz-only pseudo-tag. | roughly -1 – +1 |
| `HB_STYLE_TAG_WIDTH` | `wdth` | Narrower to wider, as a percentage of the design's normal width. Non-zero. | 50 – 200 |
| `HB_STYLE_TAG_WEIGHT` | `wght` | Lighter to blacker. Directly comparable to OS/2 `usWeightClass` and CSS `font-weight`. | 1 – 1000 |

Note the sign flip between `slnt` and `Slnt`. They describe the same slant, and
HarfBuzz converts between them with `ratio = tan(angle * -pi / 180)`. An italic
that reports an angle of `-12` reports a ratio of about `+0.213`.

All six constants are `Since: 3.0.0`.

## Functions

### Accessors

#### `hb_style_get_value`

```c
HB_EXTERN float
hb_style_get_value (hb_font_t *font, hb_style_tag_t style_tag);
```

```rust
pub fn hb_style_get_value(
    font: *mut hb_font_t,
    style_tag: hb_style_tag_t,
) -> c_float;
```

Fetches the value of one style axis for `font`.

Per the header: it searches the font's variation axes for the requested axis
first; if that is not set, it tries the default style values in the `STAT`
table; and failing that it polyfills from other tables of the font. It returns
the corresponding axis value, or a default value for the style tag.

Concretely, the implementation resolves in this order:

1. **`Slnt` special case.** `HB_STYLE_TAG_SLANT_RATIO` is not looked up
   directly. It recurses on `HB_STYLE_TAG_SLANT_ANGLE` and converts the result
   with `tan(angle * -pi / 180)`.
2. **Variation axes.** If the face has an `fvar` axis with this tag, and the
   font has a design coordinate for it, that coordinate is returned. If the face
   has the axis but the font has not set a coordinate, the axis's `fvar` default
   is returned — deliberately in preference to `STAT`, because for a variable
   font `fvar` is the authority. (Skipped entirely in an `HB_NO_VAR` build.)
3. **Point size.** For `HB_STYLE_TAG_OPTICAL_SIZE` only: if
   `hb_font_set_ptem()` has been called with a non-zero size, that size is
   returned.
4. **`STAT` table.** The axis-value records are consulted for a value for this
   tag.
5. **Per-tag polyfill**, in the legacy tables:

   | Tag | Polyfill |
   | --- | --- |
   | `ital` | 1 if the OS/2 `fsSelection` italic bit **or** the `head` `macStyle` italic bit is set, else 0. |
   | `opsz` | Midpoint of OS/2 v5 `usLowerOpticalPointSize` and `usUpperOpticalPointSize`; else the GPOS `size` feature's design size divided by 10; else **12**. |
   | `slnt` | The `post` table's `italicAngle`, adjusted if a synthetic slant was set with `hb_font_set_synthetic_slant()`. |
   | `wdth` | OS/2 `usWidthClass` mapped to a percentage (50, 62.5, 75, 87.5, 100, 112.5, 125, 150, 200); if OS/2 is absent, 75 for the `head` condensed bit, 125 for expanded, else **100**. |
   | `wght` | OS/2 `usWeightClass`; if OS/2 is absent, 700 for the `head` bold bit, else **400**. |

6. **Anything else** — any tag not in the six constants that also matched no
   `fvar` axis and no `STAT` record — returns **0.0**.

Returns: always a `float`; there is no error channel and no sentinel for
"unknown". A return of `0.0` is ambiguous between "this font is not italic",
"this custom axis defaults to zero", and "I do not recognise that tag". If you
need to distinguish them, query the axis directly with
`hb_ot_var_find_axis_info()` first.

Ownership and lifetime: the function borrows `font` and returns a plain scalar.
It does not take a reference on the font, does not mutate it, and there is
nothing for the caller to destroy. `font` need not be immutable.

Nullability: the header does not document behaviour for a null `font`, and the
implementation dereferences it immediately (`font->face`). Do not pass null.
HarfBuzz's own idiom is that failed creation yields the empty object rather than
`NULL`; passing `hb_font_get_empty()` is well defined and yields the polyfill
defaults — `ital` 0, `opsz` 12, `slnt` 0, `wdth` 100, `wght` 400.

Since HarfBuzz 3.0.0.

## Usage notes

### Reading the style of a font

```c
hb_font_t *font = hb_font_create (face);

float weight = hb_style_get_value (font, HB_STYLE_TAG_WEIGHT);        /* e.g. 400 */
float width  = hb_style_get_value (font, HB_STYLE_TAG_WIDTH);         /* e.g. 100 */
int   italic = hb_style_get_value (font, HB_STYLE_TAG_ITALIC) >= 0.5f;
```

The Rust equivalent:

```rust
use harfbuzz_sys::{
    HB_STYLE_TAG_ITALIC, HB_STYLE_TAG_WEIGHT, HB_STYLE_TAG_WIDTH, hb_style_get_value,
};

let weight = unsafe { hb_style_get_value(font, HB_STYLE_TAG_WEIGHT) };
let width = unsafe { hb_style_get_value(font, HB_STYLE_TAG_WIDTH) };
let italic = unsafe { hb_style_get_value(font, HB_STYLE_TAG_ITALIC) } >= 0.5;
```

### Values track the font, not the face

Setting variations changes what you get back:

```c
hb_variation_t v = { HB_TAG ('w','g','h','t'), 700.f };
hb_font_set_variations (font, &v, 1);

hb_style_get_value (font, HB_STYLE_TAG_WEIGHT);   /* now 700 */
```

If you cache style values, cache them keyed on the font and invalidate whenever
you call `hb_font_set_variations()`, `hb_font_set_var_coords_design()`,
`hb_font_set_var_coords_normalized()`, `hb_font_set_ptem()`, or
`hb_font_set_synthetic_slant()`. Nothing in this API notifies you of a change.

### Optical size and `hb_font_set_ptem()`

`hb_font_set_ptem()` records the point size at which the font will be rendered.
For a font with an `opsz` axis, the axis coordinate wins and `ptem` is ignored by
this function; for a font without one, `ptem` is returned as the optical size.
This is a common source of confusion: setting `ptem` does *not* select an
`opsz` coordinate for you. If you want the design to follow the point size on a
variable font, set the `opsz` variation yourself.

### Slant: get the sign right

The two slant tags have opposite signs for the same physical slant, which is the
single most common mistake with this API.

```c
float angle = hb_style_get_value (font, HB_STYLE_TAG_SLANT_ANGLE);  /* -12.0  */
float ratio = hb_style_get_value (font, HB_STYLE_TAG_SLANT_RATIO);  /* +0.2126 */
```

The ratio is the one you want for a 2×2 shear matrix; using the angle in its
place shears the wrong way and by an absurd amount. `HB_STYLE_TAG_SLANT_RATIO`
also folds in any synthetic slant set with `hb_font_set_synthetic_slant()`,
which is itself expressed as a ratio, so the two compose correctly.

### `ital` is not a boolean

Nothing guarantees the value is exactly 0 or 1. A variable font with an `ital`
axis can be at 0.4, and a `STAT` record can carry any value the designer chose.
Threshold it (`>= 0.5`) rather than comparing for equality, and never compare
floats for equality here in general — `wdth` polyfills to 87.5 and 112.5, and
`opsz` to a midpoint, so exact comparisons will surprise you.

### Custom axes

Any tag is accepted, so this doubles as a convenient "read this axis's current
value" call for arbitrary axes:

```c
float grade = hb_style_get_value (font, (hb_style_tag_t) HB_TAG ('G','R','A','D'));
```

If the face has no such axis and `STAT` has nothing to say, you get 0.0 rather
than an error, so this is only safe when you already know the axis exists. Use
`hb_ot_var_find_axis_info()` when you need to distinguish "absent" from "zero",
or when you need the axis's min/max as well.

### Threading

`hb_style_get_value()` only reads from the font and its face, so concurrent
calls on the same font from multiple threads are fine as long as no thread is
mutating that font at the same time. Face table loading is internally
synchronised and lazily cached — the first call may populate `STAT`, OS/2,
`head`, or `post` for the face, and that is safe from multiple threads. The
usual pattern of making a font immutable with `hb_font_make_immutable()` before
sharing it applies here as well.

### Rust-side reminders

- The function is `unsafe` and takes `*mut hb_font_t`; this crate adds no
  checking. `font` must be non-null and valid.
- The return type is `core::ffi::c_float`, i.e. `f32`.
- `hb_style_tag_t` is `c_int`, but `HB_TAG` produces `hb_tag_t` (`u32`), so a
  cast is needed when building a tag on the fly:
  `HB_TAG(b'G', b'R', b'A', b'D') as hb_style_tag_t`. The provided constants are
  already cast.
- There is no `hb_style_tag_from_string()`; parse tags with
  `hb_tag_from_string()` and cast.
