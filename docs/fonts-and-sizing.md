# Fonts and sizing

A `Face` is a typeface as it exists in a file. A `Font` is that face
*instantiated*: pinned to a scale, optionally to a pixel size, and optionally to
a position on each of a variable font's axes. Shaping consumes a font, because
the positions it produces live in the font's coordinate space.

- [Face vs font](#face-vs-font)
- [Units per em](#units-per-em)
- [Scale and the 26.6 convention](#scale-and-the-266-convention)
- [Pixels per em](#pixels-per-em)
- [Variable fonts](#variable-fonts)
- [Metrics](#metrics)
- [Glyph lookup](#glyph-lookup)
- [Font methods](#font-methods)

---

## Face vs font

| | `Face` | `Font` |
| --- | --- | --- |
| Represents | A typeface in a file | That typeface at a size |
| Knows | Tables, glyph count, upem | Scale, ppem, variation-axis values |
| Cost | Expensive: parses the font | Cheap: upstream calls fonts "very light-weight" |
| Cardinality | One per font file | One per (size, axis setting) you render at |
| Needed for | Building fonts, reading tables | Shaping, all metrics |

```rust
use harfbuzz_rs::{Face, Font, IntoShared, points_to_scale};

fn main() -> harfbuzz_rs::Result<()> {
    let face = Face::from_file("font.ttf", 0)?.into_shared();

    let mut body = Font::new(face.clone());
    body.set_scale(points_to_scale(11), points_to_scale(11));

    let mut heading = Font::new(face.clone());
    heading.set_scale(points_to_scale(28), points_to_scale(28));

    println!("{:?} / {:?}", body.scale(), heading.scale());
    Ok(())
}
```

`Font::new` takes a `Shared<Face>` **by value**, so clone the handle for each
font. The font holds its own reference to the face and keeps it alive; you can
get it back with `font.face()`, which returns a fresh `Shared<Face>`.

## Units per em

A font does not draw at a size. It draws on an abstract square grid — the **em
square** — and the size is applied afterwards by scaling. `Face::upem()` reports
how many units that grid is divided into:

| Font format | Typical upem |
| --- | --- |
| TrueType (`glyf`) | 1024 or **2048** |
| CFF / OpenType-PS (`.otf`) | **1000** |

Everything in the font — advances, bearings, ascender, kerning — is expressed in
those units. An advance of 1336 in a 2048-upem font means 1336/2048 = 0.652 em
wide, which at 16 pixels per em is 10.4 pixels.

```rust
use harfbuzz_rs::Face;

fn main() -> harfbuzz_rs::Result<()> {
    let face = Face::from_file("font.ttf", 0)?;

    let upem = face.upem() as f32;
    let em_fraction = |units: i32| units as f32 / upem;

    println!("{:.3} em", em_fraction(1336));
    Ok(())
}
```

You will rarely do that arithmetic yourself, because `set_scale` does it for
you.

## Scale and the 26.6 convention

**A font's scale is how many output units there are per em.** HarfBuzz scales
every value it reports from the face's design units into that space.

```rust
use harfbuzz_rs::{Face, Font, IntoShared};

fn main() -> harfbuzz_rs::Result<()> {
    let face = Face::from_file("font.ttf", 0)?.into_shared();
    let upem = face.upem() as i32;

    let font = Font::new(face);

    // A new font starts scaled to the face's design grid: output is in font units.
    assert_eq!(font.scale(), (upem, upem));
    Ok(())
}
```

Three conventions cover essentially every use:

| Goal | Scale | Reading the output |
| --- | --- | --- |
| Resolution-independent font units | `face.upem()` (the default) | Divide by `upem` for ems; multiply by your size later |
| 26.6 fixed-point pixels | `px_per_em * 64` | `value as f32 / 64.0` gives pixels |
| Whole pixels | `px_per_em` | Direct, but rounding errors accumulate — avoid |

**26.6 fixed point** is FreeType's convention and the reason for the factor of
64: an `i32` where the low 6 bits are a fraction, giving 1/64-pixel precision.
Shaping in 26.6 and dividing by 64 at the end keeps subpixel positioning without
floating point in the hot path.

`points_to_scale` is the two-character helper for it:

```rust
use harfbuzz_rs::points_to_scale;

fn main() {
    assert_eq!(points_to_scale(16), 1024);   // 16 * 64
}
```

It is a `const fn`, so it works in constants.

> **`points_to_scale(n)` assumes one point equals one pixel** — a 72-dpi
> display. On any other device, compute the pixel size first:
> `px_per_em = points * dpi / 72.0`, then pass `(px_per_em * 64.0) as i32`. On a
> 2× Retina display at 16pt, that is `points_to_scale(32)`.

```rust
use harfbuzz_rs::{Face, Font};

/// A font sized for `points` on a display with `scale_factor` device pixels
/// per point, reporting positions in 26.6 device pixels.
fn font_at(face: &harfbuzz_rs::Shared<Face>, points: f32, scale_factor: f32) -> Font {
    let px_per_em = points * scale_factor;
    let scale = (px_per_em * 64.0).round() as i32;

    let mut font = Font::new(face.clone());
    font.set_scale(scale, scale);
    font
}
```

Scale is set independently on each axis, so `set_scale(x, y)` with different
values gives synthetic horizontal or vertical stretching. Pass the same number
twice unless that is what you want.

Advances scale proportionally, but each one is rounded to an integer
independently — doubling the scale doubles a run's total width only to within
one unit per glyph. Do not rely on exact proportionality when comparing runs
shaped at different scales.

## Pixels per em

`set_ppem` tells the font what pixel size it is being rendered at. This is
*separate* from the scale, and it matters only for backends that hint —
FreeType, primarily, which uses it to pick a bitmap strike or apply
size-dependent instructions.

```rust
use harfbuzz_rs::{Face, Font, IntoShared, points_to_scale};

fn main() -> harfbuzz_rs::Result<()> {
    let face = Face::from_file("font.ttf", 0)?.into_shared();

    let mut font = Font::new(face);
    font.set_scale(points_to_scale(16), points_to_scale(16));
    font.set_ppem(16, 16);

    assert_eq!(font.ppem(), (16, 16));
    Ok(())
}
```

The default is `(0, 0)`, meaning unhinted, and with HarfBuzz's own OpenType
backend — which is what a new font uses — it changes nothing. Set it when you
are pairing this crate with FreeType; otherwise leave it alone.

`Font::set_ot_funcs()` switches a font back to that default OpenType
implementation. It is only useful for undoing a different backend that was
installed through the raw `harfbuzz_sys` API.

## Variable fonts

A **variable font** carries one design that can be interpolated along named
axes. Each axis has a four-character tag and a range the font declares:

| Tag | Axis | Typical range |
| --- | --- | --- |
| `wght` | Weight | 100–900 |
| `wdth` | Width | 50–200 (percent) |
| `slnt` | Slant | −15–0 (degrees) |
| `ital` | Italic | 0–1 |
| `opsz` | Optical size | the font's usable point range |

A [`Variation`](values.md#variation) pins one axis to one value.
`Font::set_variations` applies a whole set at once; any axis you do not name
keeps its default.

```rust
use harfbuzz_rs::{Face, Font, IntoShared, Tag, Variation, buffer_from, shape};

fn main() -> harfbuzz_rs::Result<()> {
    let face = Face::from_file("variable.ttf", 0)?.into_shared();

    let mut light = Font::new(face.clone());
    light.set_variations(&[Variation::new(Tag::new(b"wght"), 300.0)]);

    let mut heavy = Font::new(face.clone());
    heavy.set_variations(&[
        Variation::new(Tag::new(b"wght"), 900.0),
        Variation::new(Tag::new(b"wdth"), 75.0),
    ]);

    let width = |font: &Font| -> harfbuzz_rs::Result<i32> {
        let output = shape(font, buffer_from("Hamburgefonstiv")?, &[]);
        Ok(output.positions().iter().map(|p| p.x_advance()).sum())
    };

    println!("light {} vs heavy {}", width(&light)?, width(&heavy)?);
    Ok(())
}
```

Variations parse from the same `axis=value` syntax the `hb-shape` tool uses:

```rust
use harfbuzz_rs::Variation;

fn main() -> harfbuzz_rs::Result<()> {
    let settings: Vec<Variation> = ["wght=700", "wdth=87.5"]
        .iter()
        .map(|s| s.parse())
        .collect::<harfbuzz_rs::Result<_>>()?;

    assert_eq!(settings.len(), 2);
    Ok(())
}
```

Three things to know:

- **Values outside an axis's range are clamped by the font**, not rejected.
  `wght=9999` silently becomes the heaviest weight the font has.
- **Setting an axis a font does not have is ignored**, silently. Check for an
  `fvar` table if you need to know: `!face.table(Tag::new(b"fvar")).is_empty()`.
- **`set_variations` takes `&mut self`**, so it is unreachable on a
  `Shared<Font>`. One font per axis setting; that is the intended pattern, and
  fonts are cheap.

Enumerating a font's axes and named instances is not wrapped by the safe API.
Use `hb_ot_var_get_axis_infos` through
[`HarfBuzzObject::as_raw()`](ownership-and-threads.md#the-unsafe-traits); the
functions are documented in
[`../harfbuzz-sys/docs/ot_var.md`](../harfbuzz-sys/docs/ot_var.md).

## Metrics

All metrics come from the **font**, not the face, and are therefore already in
the font's scaled units.

### `FontExtents`

The vertical metrics of the typeface as a whole — what you need to place
baselines.

```rust
use harfbuzz_rs::Font;

fn line_positions(font: &Font, lines: usize) -> Vec<i32> {
    let extents = font.extents();
    let step = extents.line_height();

    (0..lines)
        .map(|n| extents.ascender + step * n as i32)
        .collect()
}
```

| Field / method | Meaning |
| --- | --- |
| `ascender: i32` | Baseline to the top of the tallest glyph. **Positive.** |
| `descender: i32` | Baseline to the bottom of the lowest glyph. **Negative.** |
| `line_gap: i32` | Recommended extra space between lines; often zero. |
| `line_height() -> i32` | `ascender - descender + line_gap`. The baseline-to-baseline step. |

The sign convention is HarfBuzz's, and it is the opposite of most 2D graphics
APIs: **y grows upward**. If your renderer's y grows downward, negate as you
cross the boundary.

`FontExtents` is a plain `Copy` struct with public fields — `Debug`, `Clone`,
`Copy`, `PartialEq`, `Eq`, `Hash`.

### `GlyphExtents`

One glyph's bounding box. Returns `None` for a glyph that does not exist; a
glyph with no outline (a space, or `.notdef` in many fonts) returns `Some` with
zeroes.

```rust
use harfbuzz_rs::{Font, GlyphExtents};

/// The ink bounds of a glyph as (left, top, right, bottom), y growing upward.
fn ink_box(font: &Font, glyph: u32) -> Option<(i32, i32, i32, i32)> {
    let GlyphExtents { x_bearing, y_bearing, width, height } = font.glyph_extents(glyph)?;

    Some((x_bearing, y_bearing, x_bearing + width, y_bearing + height))
}
```

| Field | Meaning |
| --- | --- |
| `x_bearing: i32` | Origin to the left edge of the ink. |
| `y_bearing: i32` | Origin to the **top** edge of the ink. Usually positive. |
| `width: i32` | Width of the box. Positive. |
| `height: i32` | Height of the box. **Negative**, because the box extends *downward* from `y_bearing` in an upward-growing coordinate system. |

That negative height surprises everyone once. `y_bearing + height` is the bottom
edge.

### Advances

`Font::glyph_h_advance(glyph)` gives one glyph's horizontal advance in isolation.

```rust
use harfbuzz_rs::Font;

fn advance(font: &Font, glyph: u32) -> i32 {
    font.glyph_h_advance(glyph)
}
```

Use it for glyph-level work — a glyph atlas, a fallback measurement. **Do not
measure text with it.** Summing per-glyph advances ignores kerning, ligatures,
and every contextual adjustment; shape the run and sum
`GlyphPosition::x_advance` instead, as in
[getting-started.md](getting-started.md#measuring-without-drawing).

An unknown glyph id yields a fallback value rather than an error.

## Glyph lookup

```rust
use harfbuzz_rs::Font;

fn describe(font: &Font, ch: char) -> Option<(u32, Option<String>)> {
    let glyph = font.nominal_glyph(ch)?;
    Some((glyph, font.glyph_name(glyph)))
}
```

| Method | Returns | Notes |
| --- | --- | --- |
| `nominal_glyph(char)` | `Option<u32>` | The font's `cmap` lookup. `None` when the font has no glyph for the character. **This is not shaping** — it will not find glyphs that only appear through substitution, and it applies no contextual logic. |
| `glyph_name(u32)` | `Option<String>` | The name from the `post` table. Many fonts, including most subset and variable web fonts, carry no names at all and return `None`. Names longer than 127 bytes are truncated. |

`nominal_glyph` returning `None` is the cheapest way to test whether a font
covers a character before committing to it in a fallback chain.

## Font methods

| Method | Signature | Notes |
| --- | --- | --- |
| `Font::new` | `(Shared<Face>) -> Font` | Consumes the handle; clone to build several. Scale defaults to the face's upem. |
| `face` | `(&self) -> Shared<Face>` | A new counted reference to the face, already frozen. |
| `scale` / `set_scale` | `(&self) -> (i32, i32)` / `(&mut self, i32, i32)` | Output units per em. |
| `ppem` / `set_ppem` | `(&self) -> (u32, u32)` / `(&mut self, u32, u32)` | Hinting size. `(0, 0)` means unhinted. |
| `set_variations` | `(&mut self, &[Variation])` | Unnamed axes keep their defaults; out-of-range values are clamped. |
| `nominal_glyph` | `(&self, char) -> Option<u32>` | `cmap` lookup only. |
| `glyph_name` | `(&self, u32) -> Option<String>` | Often `None`. |
| `extents` | `(&self) -> FontExtents` | Horizontal-writing metrics. Zeroed if the font has none. |
| `glyph_extents` | `(&self, u32) -> Option<GlyphExtents>` | `None` for a nonexistent glyph. |
| `glyph_h_advance` | `(&self, u32) -> i32` | One glyph, no context. |
| `set_ot_funcs` | `(&mut self)` | Restores HarfBuzz's own OpenType backend. Already the default. |
| `into_shared` | `(self) -> Shared<Font>` | Freezes the font; required to share it across threads. |

`Font` is `Send` but not `Sync`. Freeze it into a `Shared<Font>` to use one font
from several threads — see
[ownership-and-threads.md](ownership-and-threads.md#send-and-sync).

---

More font surface than this crate wraps — synthetic bold and slant, vertical
origins, glyph outlines for rasterization, per-glyph `draw` callbacks — is
documented at the C level in
[`../harfbuzz-sys/docs/font.md`](../harfbuzz-sys/docs/font.md) and
[`../harfbuzz-sys/docs/draw.md`](../harfbuzz-sys/docs/draw.md). Whole-font style
and metrics queries are in
[`../harfbuzz-sys/docs/ot_metrics.md`](../harfbuzz-sys/docs/ot_metrics.md) and
[`../harfbuzz-sys/docs/style.md`](../harfbuzz-sys/docs/style.md).
