# OpenType metrics

Transcribed from `hb-ot-metrics.h`. Rust module: `harfbuzz_sys::ot_metrics`, glob
re-exported at the crate root.

## Overview

`hb-ot-metrics.h` answers questions about a font as a whole rather than about
any individual glyph: *how tall is the ascender? where should I draw the
underline? how thick is a strikeout? what is the x-height?* It is a tiny header —
one tag enumeration and five functions — but the surface it covers is one of the
messiest corners of OpenType, and the value it adds is entirely in the lookup it
performs on your behalf.

The mess is this. A font records its font-wide metrics across four different
tables, each with its own history and its own quirks: `hhea` (horizontal line
metrics and caret slope), `vhea` (the vertical equivalents), OS/2 (typographic
*and* Windows-clipping line metrics, sub/superscript geometry, strikeout,
x-height, cap height), and `post` (underline). OS/2 alone carries two
independent sets of ascender/descender values plus a `fsSelection` bit that says
which of them the designer intended you to use. On top of that, a variable font
may vary any of these values through the `MVAR` table, keyed by exactly the same
four-byte tags this header exposes. `hb_ot_metrics_get_position()` walks that
whole structure for a given tag, applies the `MVAR` delta for the font's current
variation coordinates, scales the result into the font's coordinate space, and
hands back a single `hb_position_t`.

There are no objects here. Nothing is created, nothing is reference counted,
nothing must be destroyed. Every function takes an `hb_font_t *` and a tag and
returns a scalar. Metrics are read from the **font**, not the face, because two
things that live on the font affect the answer: the scale set with
`hb_font_set_scale()` (which converts font design units into the font's output
units) and the variation coordinates set with `hb_font_set_variations()` and
friends (which select the `MVAR` deltas). Two fonts made from the same face can
therefore report different metrics.

The tags in `hb_ot_metrics_tag_t` are exactly the
[MVAR value tags](https://docs.microsoft.com/en-us/typography/opentype/spec/mvar#value-tags)
registered by OpenType, which is why the enumeration is a tag enumeration rather
than a list of small integers. That also explains the split between the two
families of function on this page. `hb_ot_metrics_get_position*()` gives you the
**metric** — the base value from `hhea`/`vhea`/OS/2/`post` plus its variation
delta, scaled. `hb_ot_metrics_get_variation*()` gives you only the **delta** that
`MVAR` contributes, which is what you want if you are maintaining your own copy
of the base values (from a font parser of your own, say) and only need HarfBuzz
to do the variation interpolation.

Finally, note that HarfBuzz distinguishes "the font does not have this metric"
from "the metric is zero". `hb_ot_metrics_get_position()` reports that
distinction through its return value; `hb_ot_metrics_get_position_with_fallback()`
throws it away and synthesizes something plausible instead. Which you want
depends on whether you have a better fallback of your own.

## Types

### `hb_ot_metrics_tag_t`

```c
typedef enum {
  HB_OT_METRICS_TAG_HORIZONTAL_ASCENDER         = HB_TAG ('h','a','s','c'),
  HB_OT_METRICS_TAG_HORIZONTAL_DESCENDER        = HB_TAG ('h','d','s','c'),
  HB_OT_METRICS_TAG_HORIZONTAL_LINE_GAP         = HB_TAG ('h','l','g','p'),
  HB_OT_METRICS_TAG_HORIZONTAL_CLIPPING_ASCENT  = HB_TAG ('h','c','l','a'),
  HB_OT_METRICS_TAG_HORIZONTAL_CLIPPING_DESCENT = HB_TAG ('h','c','l','d'),
  HB_OT_METRICS_TAG_VERTICAL_ASCENDER           = HB_TAG ('v','a','s','c'),
  HB_OT_METRICS_TAG_VERTICAL_DESCENDER          = HB_TAG ('v','d','s','c'),
  HB_OT_METRICS_TAG_VERTICAL_LINE_GAP           = HB_TAG ('v','l','g','p'),
  HB_OT_METRICS_TAG_HORIZONTAL_CARET_RISE       = HB_TAG ('h','c','r','s'),
  HB_OT_METRICS_TAG_HORIZONTAL_CARET_RUN        = HB_TAG ('h','c','r','n'),
  HB_OT_METRICS_TAG_HORIZONTAL_CARET_OFFSET     = HB_TAG ('h','c','o','f'),
  HB_OT_METRICS_TAG_VERTICAL_CARET_RISE         = HB_TAG ('v','c','r','s'),
  HB_OT_METRICS_TAG_VERTICAL_CARET_RUN          = HB_TAG ('v','c','r','n'),
  HB_OT_METRICS_TAG_VERTICAL_CARET_OFFSET       = HB_TAG ('v','c','o','f'),
  HB_OT_METRICS_TAG_X_HEIGHT                    = HB_TAG ('x','h','g','t'),
  HB_OT_METRICS_TAG_CAP_HEIGHT                  = HB_TAG ('c','p','h','t'),
  HB_OT_METRICS_TAG_SUBSCRIPT_EM_X_SIZE         = HB_TAG ('s','b','x','s'),
  HB_OT_METRICS_TAG_SUBSCRIPT_EM_Y_SIZE         = HB_TAG ('s','b','y','s'),
  HB_OT_METRICS_TAG_SUBSCRIPT_EM_X_OFFSET       = HB_TAG ('s','b','x','o'),
  HB_OT_METRICS_TAG_SUBSCRIPT_EM_Y_OFFSET       = HB_TAG ('s','b','y','o'),
  HB_OT_METRICS_TAG_SUPERSCRIPT_EM_X_SIZE       = HB_TAG ('s','p','x','s'),
  HB_OT_METRICS_TAG_SUPERSCRIPT_EM_Y_SIZE       = HB_TAG ('s','p','y','s'),
  HB_OT_METRICS_TAG_SUPERSCRIPT_EM_X_OFFSET     = HB_TAG ('s','p','x','o'),
  HB_OT_METRICS_TAG_SUPERSCRIPT_EM_Y_OFFSET     = HB_TAG ('s','p','y','o'),
  HB_OT_METRICS_TAG_STRIKEOUT_SIZE              = HB_TAG ('s','t','r','s'),
  HB_OT_METRICS_TAG_STRIKEOUT_OFFSET            = HB_TAG ('s','t','r','o'),
  HB_OT_METRICS_TAG_UNDERLINE_SIZE              = HB_TAG ('u','n','d','s'),
  HB_OT_METRICS_TAG_UNDERLINE_OFFSET            = HB_TAG ('u','n','d','o'),

  /*< private >*/
  _HB_OT_METRICS_TAG_MAX_VALUE = HB_TAG_MAX_SIGNED /*< skip >*/
} hb_ot_metrics_tag_t;
```

```rust
pub type hb_ot_metrics_tag_t = core::ffi::c_int;
```

The metric to fetch. Values are four-byte tags, so this is really an `hb_tag_t`
in disguise.

The Rust transcription is a `c_int` alias plus constants, not a Rust `enum`. The
C enumeration's private sentinel is `HB_TAG_MAX_SIGNED` (`0x7FFFFFFF`), which
pins the underlying C type at signed `int`; and the value space is open —
`hb_ot_metrics_get_position()` accepts any tag and returns false for one it does
not recognise, so a value outside the list below is legal input. A Rust `enum`
holding such a value would be undefined behaviour.

Every constant below is `Since: 2.6.0`.

| Constant | Tag | Value (hex) | Source in the font | Scaled by | Meaning |
| --- | --- | --- | --- | --- | --- |
| `HB_OT_METRICS_TAG_HORIZONTAL_ASCENDER` | `hasc` | `0x68617363` | OS/2 `sTypoAscender` if the OS/2 `USE_TYPO_METRICS` bit is set, else `hhea.ascender` | y-scale | Horizontal ascender. Forced positive. |
| `HB_OT_METRICS_TAG_HORIZONTAL_DESCENDER` | `hdsc` | `0x68647363` | OS/2 `sTypoDescender` if `USE_TYPO_METRICS`, else `hhea.descender` | y-scale | Horizontal descender. Forced **negative**. |
| `HB_OT_METRICS_TAG_HORIZONTAL_LINE_GAP` | `hlgp` | `0x686C6770` | OS/2 `sTypoLineGap` if `USE_TYPO_METRICS`, else `hhea.lineGap` | y-scale | Extra leading between horizontal lines. |
| `HB_OT_METRICS_TAG_HORIZONTAL_CLIPPING_ASCENT` | `hcla` | `0x68636C61` | OS/2 `usWinAscent` | y-scale | Windows clipping ascent — the top of the box outside which glyphs may be clipped. |
| `HB_OT_METRICS_TAG_HORIZONTAL_CLIPPING_DESCENT` | `hcld` | `0x68636C64` | OS/2 `usWinDescent` | y-scale | Windows clipping descent. Stored as a positive magnitude in OS/2 and returned as such. |
| `HB_OT_METRICS_TAG_VERTICAL_ASCENDER` | `vasc` | `0x76617363` | `vhea.ascender` | **x**-scale | Vertical ascender — the extent to one side of the vertical baseline. Forced positive. |
| `HB_OT_METRICS_TAG_VERTICAL_DESCENDER` | `vdsc` | `0x76647363` | `vhea.descender` | **x**-scale | Vertical descender. Forced **negative**. |
| `HB_OT_METRICS_TAG_VERTICAL_LINE_GAP` | `vlgp` | `0x766C6770` | `vhea.lineGap` | **x**-scale | Extra leading between vertical lines. |
| `HB_OT_METRICS_TAG_HORIZONTAL_CARET_RISE` | `hcrs` | `0x68637273` | `hhea.caretSlopeRise` | y-scale | Rise component of the caret slope for horizontal text. |
| `HB_OT_METRICS_TAG_HORIZONTAL_CARET_RUN` | `hcrn` | `0x6863726E` | `hhea.caretSlopeRun` | x-scale | Run component of the caret slope. `rise`/`run` together give the caret angle; upright fonts are rise 1, run 0. |
| `HB_OT_METRICS_TAG_HORIZONTAL_CARET_OFFSET` | `hcof` | `0x68636F66` | `hhea.caretOffset` | x-scale | Amount to shift the caret sideways to centre it on a slanted glyph. |
| `HB_OT_METRICS_TAG_VERTICAL_CARET_RISE` | `vcrs` | `0x76637273` | `vhea.caretSlopeRise` | **x**-scale | Rise component of the caret slope for vertical text. |
| `HB_OT_METRICS_TAG_VERTICAL_CARET_RUN` | `vcrn` | `0x7663726E` | `vhea.caretSlopeRun` | **y**-scale | Run component of the vertical caret slope. |
| `HB_OT_METRICS_TAG_VERTICAL_CARET_OFFSET` | `vcof` | `0x76636F66` | `vhea.caretOffset` | **y**-scale | Vertical caret offset. |
| `HB_OT_METRICS_TAG_X_HEIGHT` | `xhgt` | `0x78686774` | OS/2 **version 2 or later** `sxHeight` | y-scale | Height of a lowercase `x`. |
| `HB_OT_METRICS_TAG_CAP_HEIGHT` | `cpht` | `0x63706874` | OS/2 **version 2 or later** `sCapHeight` | y-scale | Height of a capital letter. |
| `HB_OT_METRICS_TAG_SUBSCRIPT_EM_X_SIZE` | `sbxs` | `0x73627873` | OS/2 `ySubscriptXSize` | x-scale | Horizontal size of the em square a subscript should be drawn at. |
| `HB_OT_METRICS_TAG_SUBSCRIPT_EM_Y_SIZE` | `sbys` | `0x73627973` | OS/2 `ySubscriptYSize` | y-scale | Vertical size of the subscript em square. |
| `HB_OT_METRICS_TAG_SUBSCRIPT_EM_X_OFFSET` | `sbxo` | `0x7362786F` | OS/2 `ySubscriptXOffset` | x-scale | Horizontal offset for a subscript. |
| `HB_OT_METRICS_TAG_SUBSCRIPT_EM_Y_OFFSET` | `sbyo` | `0x7362796F` | OS/2 `ySubscriptYOffset` | y-scale | Vertical offset for a subscript, **downward positive** per OS/2. |
| `HB_OT_METRICS_TAG_SUPERSCRIPT_EM_X_SIZE` | `spxs` | `0x73707873` | OS/2 `ySuperscriptXSize` | x-scale | Horizontal size of the superscript em square. |
| `HB_OT_METRICS_TAG_SUPERSCRIPT_EM_Y_SIZE` | `spys` | `0x73707973` | OS/2 `ySuperscriptYSize` | y-scale | Vertical size of the superscript em square. |
| `HB_OT_METRICS_TAG_SUPERSCRIPT_EM_X_OFFSET` | `spxo` | `0x7370786F` | OS/2 `ySuperscriptXOffset` | x-scale | Horizontal offset for a superscript. |
| `HB_OT_METRICS_TAG_SUPERSCRIPT_EM_Y_OFFSET` | `spyo` | `0x7370796F` | OS/2 `ySuperscriptYOffset` | y-scale | Vertical offset for a superscript, **upward positive** per OS/2. |
| `HB_OT_METRICS_TAG_STRIKEOUT_SIZE` | `strs` | `0x73747273` | OS/2 `yStrikeoutSize` | y-scale | Thickness of the strikeout rule. |
| `HB_OT_METRICS_TAG_STRIKEOUT_OFFSET` | `stro` | `0x7374726F` | OS/2 `yStrikeoutPosition` | y-scale | Distance from the baseline to the bottom of the strikeout rule. |
| `HB_OT_METRICS_TAG_UNDERLINE_SIZE` | `unds` | `0x756E6473` | `post.underlineThickness` | y-scale | Thickness of the underline rule. |
| `HB_OT_METRICS_TAG_UNDERLINE_OFFSET` | `undo` | `0x756E646F` | `post.underlinePosition` | y-scale | Distance from the baseline to the top of the underline rule. Normally **negative**. |

Two things in that table are easy to miss and are worth calling out.

**The vertical tags use the opposite scale axis from what their names suggest.**
`vasc`, `vdsc`, `vlgp`, and `vcrs` are scaled by the font's *x*-scale; `vcrn` and
`vcof` by the *y*-scale. This is correct: in vertical layout the "ascender" is a
horizontal distance from the vertical baseline. It is still a common source of
confusion when comparing against a font editor's numbers.

**Ascender and descender signs are normalised, but only for the four line
metrics.** `hasc` and `vasc` are returned as `+|value|`, `hdsc` and `vdsc` as
`-|value|`, regardless of how the font stored them. Fonts in the wild disagree
about the sign of `hhea.descender`, and this is HarfBuzz papering over that. No
such normalisation is applied to any other tag — `undo`, for instance, comes
straight out of `post` with whatever sign the designer used (conventionally
negative).

## Functions

### Reading a metric

#### `hb_ot_metrics_get_position`

```c
HB_EXTERN hb_bool_t
hb_ot_metrics_get_position (hb_font_t           *font,
                            hb_ot_metrics_tag_t  metrics_tag,
                            hb_position_t       *position     /* OUT.  May be NULL. */);
```

```rust
pub fn hb_ot_metrics_get_position(
    font: *mut hb_font_t,
    metrics_tag: hb_ot_metrics_tag_t,
    position: *mut hb_position_t,
) -> hb_bool_t;
```

Fetches the metric value corresponding to `metrics_tag` from `font`.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | The font to read from. Must be non-null — the implementation dereferences it immediately to reach the face. Passing `hb_font_get_empty()` is well defined and reports "not found" for every tag. |
| `metrics_tag` | Which metric to fetch. Any `hb_tag_t` value is accepted; unrecognised tags return false. |
| `position` | Out parameter, **may be NULL**. When non-null it receives the metric scaled into the font's coordinate space (design units × scale ÷ upem), with the `MVAR` delta for the font's current variation coordinates already added. When null the function still reports presence, so this is the cheap way to ask "does this font have a cap height?" |

**Returns** — true if the requested metric was found in the font, false
otherwise. False means one of: the backing table is absent (`vhea` for the
vertical metrics, OS/2 for most others, `post` for underline); the backing table
is present but too old (x-height and cap height need OS/2 version 2 or later);
or the tag is not one this function knows. When false is returned, `*position`
is left untouched — it is **not** zeroed, so initialise it yourself if you plan
to use it either way.

**Ownership** — nothing is allocated, nothing is retained, nothing needs
destroying. The font is only read from; it is not made immutable and is not
mutated.

**Notes**

- The value is scaled, so it is only meaningful together with the scale set by
  `hb_font_set_scale()`. A font created with `hb_font_create()` starts at a scale
  equal to the face's upem, which makes the returned values equal to the design
  units.
- Variation deltas are applied automatically. Change the font's variation
  coordinates and the same call returns a different number.
- For `hcrs` and `hcrn`, a synthetic slant set with
  `hb_font_set_synthetic_slant()` is folded into the result: the caret slope
  components are scaled by `min(upem / caretSlopeRise, 256)` when the rise is
  non-zero and smaller than the upem, and the run additionally gains the shear
  contribution from the slant. This makes the caret follow a synthetically
  slanted font.
- Since HarfBuzz 2.6.0.
- Thread-safety: read-only with respect to the font, so concurrent calls from
  several threads are safe provided no thread is mutating that font at the same
  time. The first call may lazily load and cache a table on the face; that
  caching is internally synchronised.
- Upstream compiles this function out entirely under the `HB_NO_METRICS`
  reduced-feature build option, which `HB_LEAN` and `HB_TINY` imply. This crate's
  default build does not set any of those.

#### `hb_ot_metrics_get_position_with_fallback`

```c
HB_EXTERN void
hb_ot_metrics_get_position_with_fallback (hb_font_t           *font,
                                          hb_ot_metrics_tag_t  metrics_tag,
                                          hb_position_t       *position     /* OUT */);
```

```rust
pub fn hb_ot_metrics_get_position_with_fallback(
    font: *mut hb_font_t,
    metrics_tag: hb_ot_metrics_tag_t,
    position: *mut hb_position_t,
);
```

Fetches the metric value corresponding to `metrics_tag` from `font`, and
synthesizes a value if the value is missing in the font.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | The font to read from. Must be non-null. |
| `metrics_tag` | Which metric to fetch. Tags this function does not recognise yield `0`. |
| `position` | Out parameter. **Must be non-null**, despite the gtk-doc annotation upstream marking it `(optional)` — the implementation writes through it unconditionally on the fallback path and dereferences it while deciding whether to take that path. The header comment on this function, unlike the one on `hb_ot_metrics_get_position()`, does not say "may be NULL". |

**Returns** — nothing. There is no way to learn whether the value came from the
font or was synthesized; call `hb_ot_metrics_get_position()` first if you need to
know.

**Ownership** — as `hb_ot_metrics_get_position()`: nothing allocated, nothing
retained.

**Notes**

- The function first tries `hb_ot_metrics_get_position()`. It returns that value
  immediately *unless* the tag is `HB_OT_METRICS_TAG_STRIKEOUT_SIZE` or
  `HB_OT_METRICS_TAG_UNDERLINE_SIZE` and the value came back as zero — a
  zero-thickness rule is treated as "missing" and replaced, because drawing a
  zero-width line is never what the caller wanted.
- Since HarfBuzz 4.0.0. Note that this is four major versions later than the rest
  of the header; if you compile against an older HarfBuzz this symbol will not
  exist.

The synthesized values, when the font has nothing to offer:

| Tag | Fallback |
| --- | --- |
| `hasc`, `hcla` | `hb_font_get_extents_for_direction(font, HB_DIRECTION_LTR).ascender` |
| `hdsc`, `hcld` | `hb_font_get_extents_for_direction(font, HB_DIRECTION_LTR).descender` |
| `hlgp` | `hb_font_get_extents_for_direction(font, HB_DIRECTION_LTR).line_gap` |
| `vasc` | `hb_font_get_extents_for_direction(font, HB_DIRECTION_TTB).ascender` |
| `vdsc` | `hb_font_get_extents_for_direction(font, HB_DIRECTION_TTB).ascender` — yes, `ascender`; see Pitfalls |
| `vlgp` | `hb_font_get_extents_for_direction(font, HB_DIRECTION_TTB).line_gap` |
| `hcrs`, `vcrs` | `1` |
| `hcrn`, `vcrn` | `0` |
| `hcof`, `vcof` | `0` |
| `xhgt` | The `y_bearing` of the glyph for U+0078 `x`, if that glyph and its extents can be found; otherwise `y_scale / 2` |
| `cpht` | `height + 2 * y_bearing` of the glyph for U+004F `O`, if available; otherwise `y_scale * 2 / 3` |
| `strs`, `unds` | `y_scale / 18` |
| `stro` | Half the fallback horizontal ascender, i.e. this function called recursively for `hasc` and divided by two |
| `undo` | `-y_scale / 18` |
| `sbxs`, `spxs` | `x_scale * 10 / 12` |
| `sbys`, `spys` | `y_scale * 10 / 12` |
| `sbxo`, `spxo` | `0` |
| `sbyo`, `spyo` | `y_scale / 5` |
| anything else | `0` |

The line-metric fallbacks go through `hb_font_get_extents_for_direction()`, which
means they end up in the font-funcs layer: a font with custom funcs, or an
`hb_ft_font`, can supply values there that the OpenType tables do not have. The
`xhgt` and `cpht` fallbacks actually measure a glyph, so they cost a glyph
lookup and an extents call.

### Reading a variation delta

These three functions return only the contribution that the `MVAR` table makes
for the font's current variation coordinates — not the metric itself. They are
useful when you already have the base values from your own font parser and want
HarfBuzz to do the variation interpolation. If you just want the metric, use
`hb_ot_metrics_get_position()`, which already adds the delta.

All three return `0` when the face has no `MVAR` table or when `MVAR` has no
record for the requested tag, which is the same value they return for a font
sitting at the default instance. There is no error channel.

#### `hb_ot_metrics_get_variation`

```c
HB_EXTERN float
hb_ot_metrics_get_variation (hb_font_t *font, hb_ot_metrics_tag_t metrics_tag);
```

```rust
pub fn hb_ot_metrics_get_variation(
    font: *mut hb_font_t,
    metrics_tag: hb_ot_metrics_tag_t,
) -> c_float;
```

Fetches the metric variation delta corresponding to `metrics_tag` from `font`,
with the current font variation settings applied.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | The font to read from. Must be non-null; the implementation dereferences it to reach the face and the normalized coordinates. |
| `metrics_tag` | Which metric's delta to fetch. Unrecognised tags simply miss in the `MVAR` binary search and yield `0.0`. |

**Returns** — the delta, in **unscaled font design units**, as a `float`. Unlike
the other functions on this page the result is *not* multiplied by the font's
scale, and it is not rounded to an integer.

**Ownership** — nothing allocated, nothing retained.

**Notes**

- Since HarfBuzz 2.6.0.
- Compiled out upstream under the `HB_NO_VAR` reduced-feature build option (and
  under `HB_NO_METRICS`). This crate's default build includes it.
- A return of `0.0` is ambiguous between "no `MVAR`", "no record for this tag",
  and "this instance's delta happens to be zero". Nothing distinguishes them.

#### `hb_ot_metrics_get_x_variation`

```c
HB_EXTERN hb_position_t
hb_ot_metrics_get_x_variation (hb_font_t *font, hb_ot_metrics_tag_t metrics_tag);
```

```rust
pub fn hb_ot_metrics_get_x_variation(
    font: *mut hb_font_t,
    metrics_tag: hb_ot_metrics_tag_t,
) -> hb_position_t;
```

Fetches the horizontal metric variation delta corresponding to `metrics_tag`
from `font`, with the current font variation settings applied.

**Parameters** — identical to `hb_ot_metrics_get_variation()`.

**Returns** — the same delta, scaled by the font's **x**-scale and rounded to an
`hb_position_t`. Exactly `hb_ot_metrics_get_variation()` put through the font's
horizontal em-scaling.

**Ownership** — nothing allocated, nothing retained.

**Notes** — Since HarfBuzz 2.6.0. Same `HB_NO_VAR` / `HB_NO_METRICS` caveat as
above. Because the result is rounded to an integer, a small delta on a small
scale can round to `0`.

#### `hb_ot_metrics_get_y_variation`

```c
HB_EXTERN hb_position_t
hb_ot_metrics_get_y_variation (hb_font_t *font, hb_ot_metrics_tag_t metrics_tag);
```

```rust
pub fn hb_ot_metrics_get_y_variation(
    font: *mut hb_font_t,
    metrics_tag: hb_ot_metrics_tag_t,
) -> hb_position_t;
```

Fetches the vertical metric variation delta corresponding to `metrics_tag` from
`font`, with the current font variation settings applied.

**Parameters** — identical to `hb_ot_metrics_get_variation()`.

**Returns** — the same delta, scaled by the font's **y**-scale and rounded to an
`hb_position_t`.

**Ownership** — nothing allocated, nothing retained.

**Notes** — Since HarfBuzz 2.6.0. Same caveats as
`hb_ot_metrics_get_x_variation()`. Choosing between the x and y variants is your
responsibility; nothing checks that you picked the axis that matches the tag.
Use the "Scaled by" column of the tag table above.

## Usage

### Laying out a line of horizontal text

The three line metrics plus the font's scale are enough to compute a line
height:

```c
hb_font_t *font = hb_font_create (face);
hb_font_set_scale (font, 16 * 64, 16 * 64);   /* 16pt in 26.6 fixed point */

hb_position_t ascender, descender, line_gap;
hb_ot_metrics_get_position_with_fallback (font, HB_OT_METRICS_TAG_HORIZONTAL_ASCENDER,  &ascender);
hb_ot_metrics_get_position_with_fallback (font, HB_OT_METRICS_TAG_HORIZONTAL_DESCENDER, &descender);
hb_ot_metrics_get_position_with_fallback (font, HB_OT_METRICS_TAG_HORIZONTAL_LINE_GAP,  &line_gap);

/* descender is negative, so this subtracts. */
hb_position_t line_height = ascender - descender + line_gap;
```

The Rust equivalent:

```rust
use harfbuzz_sys::{
    HB_OT_METRICS_TAG_HORIZONTAL_ASCENDER, HB_OT_METRICS_TAG_HORIZONTAL_DESCENDER,
    HB_OT_METRICS_TAG_HORIZONTAL_LINE_GAP, hb_font_set_scale,
    hb_ot_metrics_get_position_with_fallback, hb_position_t,
};

unsafe {
    hb_font_set_scale(font, 16 * 64, 16 * 64);

    let mut ascender: hb_position_t = 0;
    let mut descender: hb_position_t = 0;
    let mut line_gap: hb_position_t = 0;

    hb_ot_metrics_get_position_with_fallback(
        font,
        HB_OT_METRICS_TAG_HORIZONTAL_ASCENDER,
        &mut ascender,
    );
    hb_ot_metrics_get_position_with_fallback(
        font,
        HB_OT_METRICS_TAG_HORIZONTAL_DESCENDER,
        &mut descender,
    );
    hb_ot_metrics_get_position_with_fallback(
        font,
        HB_OT_METRICS_TAG_HORIZONTAL_LINE_GAP,
        &mut line_gap,
    );

    let line_height = ascender - descender + line_gap;
}
```

### Deciding whether the font actually has a metric

Use the return value rather than the with-fallback variant when you have a better
fallback of your own, or when the distinction matters to your UI:

```c
hb_position_t cap_height;
if (hb_ot_metrics_get_position (font, HB_OT_METRICS_TAG_CAP_HEIGHT, &cap_height))
  use_designer_cap_height (cap_height);
else
  measure_a_capital_O_yourself (font);
```

```rust
use harfbuzz_sys::{HB_OT_METRICS_TAG_CAP_HEIGHT, hb_ot_metrics_get_position, hb_position_t};

let mut cap_height: hb_position_t = 0;
let found = unsafe {
    hb_ot_metrics_get_position(font, HB_OT_METRICS_TAG_CAP_HEIGHT, &mut cap_height) != 0
};
```

Note the `!= 0`: `hb_bool_t` is a C `int`, not a Rust `bool`.

### Probing without reading

Passing a null `position` to `hb_ot_metrics_get_position()` asks only whether the
metric exists:

```c
hb_bool_t has_x_height =
  hb_ot_metrics_get_position (font, HB_OT_METRICS_TAG_X_HEIGHT, NULL);
```

```rust
let has_x_height = unsafe {
    hb_ot_metrics_get_position(font, HB_OT_METRICS_TAG_X_HEIGHT, core::ptr::null_mut()) != 0
};
```

This is the *only* function on the page where null is accepted.

### Drawing an underline

```c
hb_position_t thickness, offset;
hb_ot_metrics_get_position_with_fallback (font, HB_OT_METRICS_TAG_UNDERLINE_SIZE,   &thickness);
hb_ot_metrics_get_position_with_fallback (font, HB_OT_METRICS_TAG_UNDERLINE_OFFSET, &offset);

/* offset is measured from the baseline to the top of the rule and is
   normally negative, i.e. below the baseline in a y-up coordinate system. */
draw_rect (x, baseline_y + offset - thickness, width, thickness);
```

Using the fallback variant here is the right default: a zero `unds` is
specifically caught and replaced with `y_scale / 18`, so you never end up
drawing an invisible rule.

### Metrics follow the font's variation settings

```c
hb_variation_t v = { HB_TAG ('w','g','h','t'), 700.f };
hb_font_set_variations (font, &v, 1);

hb_position_t thickness;
hb_ot_metrics_get_position (font, HB_OT_METRICS_TAG_UNDERLINE_SIZE, &thickness);
/* Now reflects the MVAR delta for wght=700, if the font varies `unds`. */
```

If you cache metrics, key the cache on the font and invalidate whenever you call
`hb_font_set_variations()`, `hb_font_set_var_coords_design()`,
`hb_font_set_var_coords_normalized()`, `hb_font_set_scale()`, or
`hb_font_set_synthetic_slant()`. Nothing here notifies you of a change.

### Interpolating your own base values

If you parsed the font tables yourself and only want HarfBuzz's `MVAR`
interpolation:

```c
float delta = hb_ot_metrics_get_variation (font, HB_OT_METRICS_TAG_STRIKEOUT_SIZE);
float my_varied_strikeout = my_os2_yStrikeoutSize + delta;   /* design units */
```

```rust
use harfbuzz_sys::{HB_OT_METRICS_TAG_STRIKEOUT_SIZE, hb_ot_metrics_get_variation};

let delta = unsafe { hb_ot_metrics_get_variation(font, HB_OT_METRICS_TAG_STRIKEOUT_SIZE) };
let varied = my_os2_y_strikeout_size as f32 + delta;
```

Add the delta in design units, then scale — do not scale the delta separately
with `hb_ot_metrics_get_y_variation()` and add it to an already-scaled base, or
you will accumulate two roundings.

### Positioning a superscript

```c
hb_position_t x_size, y_size, x_off, y_off;
hb_ot_metrics_get_position_with_fallback (font, HB_OT_METRICS_TAG_SUPERSCRIPT_EM_X_SIZE,   &x_size);
hb_ot_metrics_get_position_with_fallback (font, HB_OT_METRICS_TAG_SUPERSCRIPT_EM_Y_SIZE,   &y_size);
hb_ot_metrics_get_position_with_fallback (font, HB_OT_METRICS_TAG_SUPERSCRIPT_EM_X_OFFSET, &x_off);
hb_ot_metrics_get_position_with_fallback (font, HB_OT_METRICS_TAG_SUPERSCRIPT_EM_Y_OFFSET, &y_off);
```

The two `SIZE` values describe the em square the superscript glyphs should be
drawn at, which you turn into a scale factor by dividing by the font's scale.
The two `OFFSET` values are where to place that em square relative to the
baseline. Per OS/2, the superscript y-offset is positive upward and the subscript
y-offset is positive downward — HarfBuzz passes both through unchanged, so the
sign convention differs between the two families.

## Pitfalls

### `position` is left untouched when the metric is missing

`hb_ot_metrics_get_position()` writes `*position` only on success. On failure the
variable keeps whatever it held before. Uninitialised stack variables are the
classic way this bites:

```c
hb_position_t cap;                                       /* garbage */
hb_ot_metrics_get_position (font, HB_OT_METRICS_TAG_CAP_HEIGHT, &cap);
use (cap);                                               /* still garbage */
```

Either check the return value or initialise first. Rust makes you initialise, but
does not make you check the return value.

### `hb_ot_metrics_get_position_with_fallback()` does not accept a null `position`

The upstream gtk-doc annotation says `(out) (optional)`, but the header comment
says only `/* OUT */`, and the implementation writes through the pointer
unconditionally on the fallback path — and dereferences it while deciding whether
to take that path for the strikeout and underline size tags. Passing `NULL` will
crash. Only `hb_ot_metrics_get_position()` documents and honours a null
`position`.

### `vdsc` falls back to the vertical *ascender*

In the vendored HarfBuzz 14.3.0 sources, the
`HB_OT_METRICS_TAG_VERTICAL_DESCENDER` case of
`hb_ot_metrics_get_position_with_fallback()` assigns `font_extents.ascender`
rather than `font_extents.descender`, unlike the horizontal descender case
directly above it. If a font lacks `vhea` you will get a positive ascender-shaped
number where a negative descender belongs. Fonts with a `vhea` table are
unaffected, because the fallback path is not taken. If you support vertical text
and cannot rely on `vhea` being present, call `hb_ot_metrics_get_position()` and
compute your own fallback from `hb_font_get_v_extents()`.

### The variation functions return a delta, not a metric

`hb_ot_metrics_get_variation()` and its x/y variants are named as though they are
"the metric, with variations applied". They are not; they are only the `MVAR`
contribution. `hb_ot_metrics_get_position()` already includes it. Adding them
together double-counts the variation.

### The variation functions return `0` for everything on a static font

There is no way to tell "this font has no `MVAR`" from "this delta is zero".
Check for the presence of variation axes with `hb_ot_var_has_data()` first if you
need to branch on it.

### `hb_ot_metrics_get_variation()` is unscaled; everything else is scaled

The one `float` on this page is in font design units. Every `hb_position_t` on
this page has been multiplied by the font's scale and divided by the face's upem.
Mixing them silently produces values that are wrong by a factor of
`scale / upem` — which is exactly `1` on a freshly created font, so the bug hides
until someone calls `hb_font_set_scale()`.

### Vertical metrics are scaled by the x-scale

`vasc`, `vdsc`, `vlgp`, and `vcrs` are horizontal distances measured from the
vertical baseline, so they are scaled by the font's x-scale; `vcrn` and `vcof`
use the y-scale. If you set a non-square scale — `hb_font_set_scale(font, sx, sy)`
with `sx != sy` — and reason from the tag names, you will get these wrong.

### Descenders are negative, `usWinDescent` is not

`hdsc` and `vdsc` are normalised to be negative. `hcld`
(`HB_OT_METRICS_TAG_HORIZONTAL_CLIPPING_DESCENT`) is *not* normalised, and OS/2
stores `usWinDescent` as an unsigned positive magnitude, so it comes back
positive. The two "descent" values therefore have opposite signs. Likewise
`undo` (underline offset) is conventionally negative and receives no
normalisation at all — a font that stores it positive will hand you a positive
value.

### x-height and cap height need OS/2 version 2

Those two fields do not exist in OS/2 versions 0 and 1. A font with an old OS/2
table reports "not found" for `xhgt` and `cpht` even though it obviously has
lowercase and capital letters. The with-fallback variant handles this by
measuring the `x` and `O` glyphs.

### Typographic versus Windows line metrics

`hasc`/`hdsc`/`hlgp` return the OS/2 *typographic* values only when the font sets
the `USE_TYPO_METRICS` bit in `fsSelection`; otherwise they return the `hhea`
values. `hcla`/`hcld` always return the OS/2 *Windows* values. These three
families frequently disagree by a wide margin in real fonts. Use the `hasc`
family for line layout and the `hcla` family only for deciding a clipping box.

### Reduced-feature builds

Everything on this page except the shared internals of the six line metrics is
inside `#ifndef HB_NO_METRICS` upstream, and the three variation functions are
additionally inside `#ifndef HB_NO_VAR`. `HB_LEAN` and `HB_TINY` define both. The
header declares all five functions unconditionally, so a program can compile
against the header and still fail to link against such a build. Separately,
`HB_NO_VERTICAL` removes the `vhea` cases, making every vertical tag report "not
found". This crate's default build enables all of them.

### Undocumented private tags

`hb_ot_metrics_get_position()` also recognises six tags that are not in the
public header and not in the enumeration: `Oasc`, `Hasc`, `Odsc`, `Hdsc`, `Olgp`,
and `Hlgp`. They read OS/2 `sTypoAscender`, `hhea.ascender`, OS/2
`sTypoDescender`, `hhea.descender`, OS/2 `sTypoLineGap`, and `hhea.lineGap`
respectively, bypassing both the `USE_TYPO_METRICS` check and the
ascender/descender sign normalisation. They exist for callers that need to see
the raw disagreement between the two tables. They are private, undocumented, and
not covered by HarfBuzz's API stability promise — do not build on them.

### Rust-side reminders

- Every function is `unsafe` and takes `*mut hb_font_t`; this crate adds no
  checking. `font` must be non-null and valid.
- `hb_ot_metrics_get_position()` returns `hb_bool_t`, which is `core::ffi::c_int`.
  Compare with `!= 0`; it will not coerce to `bool`.
- `hb_ot_metrics_tag_t` is `c_int`, but `HB_TAG` produces `hb_tag_t` (`u32`), so
  a cast is needed when building a tag on the fly:
  `HB_TAG(b'h', b'a', b's', b'c') as hb_ot_metrics_tag_t`. The provided constants
  are already cast.
- `hb_position_t` is `i32`; `hb_ot_metrics_get_variation()` returns
  `core::ffi::c_float`, i.e. `f32`.
- There is no `hb_ot_metrics_tag_from_string()`. Parse tags with
  `hb_tag_from_string()` and cast.
