# OpenType math

Reference for `hb-ot-math.h` — read access to the OpenType `MATH` table — as
transcribed in `harfbuzz_sys::ot_math`. The module is glob re-exported at the
crate root, so every name below is reachable as `harfbuzz_sys::NAME`.

## Overview

**HarfBuzz does not lay out mathematics.** Upstream says so in the section
blurb: *"HarfBuzz itself does not implement a math layout solution. The
functions and types provided can be used by client programs to access the font
data necessary for typesetting OpenType Math layout."* This header is a reader
for one font table and nothing more. It creates no objects, allocates nothing,
takes no references, and has no destroy functions. Every call is a pure query
against an `hb_face_t` or an `hb_font_t` that you already own.

The table it reads is the OpenType
[`MATH` table](https://docs.microsoft.com/en-us/typography/opentype/spec/math),
which a math font (Latin Modern Math, STIX Two Math, Cambria Math, XITS Math,
Libertinus Math, …) carries in addition to the usual `cmap`/`glyf`/`GSUB`/`GPOS`
machinery. The `MATH` table has three parts, and this header exposes all three:

1. **`MathConstants`** — 56 named layout parameters (`axisHeight`,
   `fractionRuleThickness`, `superscriptShiftUp`, …) that a typesetter needs in
   order to position fractions, radicals, scripts, limits and bars. Read with
   `hb_ot_math_get_constant()` using an `hb_ot_math_constant_t` selector.
2. **`MathGlyphInfo`** — per-glyph data: italics correction, top-accent
   attachment point, the "extended shape" flag, and *math kerning* (also called
   cut-ins), which is the staircase of kern values used to tuck a script into
   the corner of a base glyph. Read with
   `hb_ot_math_get_glyph_italics_correction()`,
   `hb_ot_math_get_glyph_top_accent_attachment()`,
   `hb_ot_math_is_glyph_extended_shape()`, `hb_ot_math_get_glyph_kerning()` and
   `hb_ot_math_get_glyph_kernings()`.
3. **`MathVariants`** — how to grow a stretchy glyph (parentheses, braces,
   radicals, arrows, over/underbraces). A font offers a discrete ladder of
   pre-drawn *size variants*, and beyond the largest one a recipe for an
   *assembly* built out of repeatable parts. Read with
   `hb_ot_math_get_glyph_variants()`, `hb_ot_math_get_glyph_assembly()` and
   `hb_ot_math_get_min_connector_overlap()`.

Three of the calls take an `hb_face_t` and the rest take an `hb_font_t`. That
split is meaningful, not accidental: the face-level calls
(`hb_ot_math_has_data`, `hb_ot_math_is_glyph_extended_shape`) return facts about
the font *data*, while the font-level calls return values already scaled from
font design units into the font's user space by `hb_font_set_scale()` and
adjusted for the active variation coordinates. Change the scale or the
variations and every font-level value here changes with it. Nothing is cached on
your behalf.

Everything in this header is a value type or a plain integer. There is no
opaque object, no reference counting, and no `_destroy` function; the three
structs (`hb_ot_math_kern_entry_t`, `hb_ot_math_glyph_variant_t`,
`hb_ot_math_glyph_part_t`) are filled into arrays you allocate yourself. The
three array-returning functions share one calling convention, described under
[The array-fetch convention](#the-array-fetch-convention) below, and it has a
trap in it worth reading before you use them.

Upstream compiles this entire file out under the reduced-feature build option
`HB_NO_MATH` (`hb-ot-math.cc` is wrapped in `#ifndef HB_NO_MATH`). The header
declares the functions unconditionally, so a program can compile against the
header and fail to link against such a build. This crate's `build.rs` defines no
`HB_NO_*` macros, so math is present in the default configuration.

## Macros

Both are plain value macros and are transcribed as `pub const`. There are no
function-like macros in this header, so nothing was skipped.

### `HB_OT_TAG_MATH`

```c
#define HB_OT_TAG_MATH HB_TAG('M','A','T','H')
```
```rust
pub const HB_OT_TAG_MATH: hb_tag_t = HB_TAG(b'M', b'A', b'T', b'H');
```

The table tag of the OpenType
[Mathematical Typesetting Table](https://docs.microsoft.com/en-us/typography/opentype/spec/math),
numerically `0x4D415448`. Use it wherever a raw table tag is wanted — for
example `hb_face_reference_table(face, HB_OT_TAG_MATH)` to get the raw bytes, or
`hb_face_get_table_tags()` to enumerate. Note that `hb_face_reference_table()`
returns the *empty blob* rather than null when the table is absent, so
`hb_ot_math_has_data()` is the better presence test.

`hb_tag_t` is `u32`, and `HB_TAG` comes from `hb-common.h` (`crate::common`).
Since HarfBuzz 1.3.3.

### `HB_OT_TAG_MATH_SCRIPT`

```c
#define HB_OT_TAG_MATH_SCRIPT HB_TAG('m','a','t','h')
```
```rust
pub const HB_OT_TAG_MATH_SCRIPT: hb_tag_t = HB_TAG(b'm', b'a', b't', b'h');
```

The OpenType **script** tag `math` (lower case). The header describes it as the
tag "for features specific to math shaping" — it is how you select the
`GSUB`/`GPOS` feature lookups a math font provides, such as `ssty`
(script-style alternates), `dtls` (dotless forms) and `flac` (flattened
accents).

The header attaches an explicit warning to this one:

> `HB_OT_TAG_MATH_SCRIPT` is not a valid `hb_script_t` and should only be used
> with functions that accept raw OpenType script tags, such as
> `hb_ot_layout_collect_features`. In other cases, `HB_SCRIPT_MATH` should be
> used instead.

`HB_SCRIPT_MATH` lives in `hb-common.h`/`hb-script-list.h` (`crate::script`) and
is the tag `Zmth`; the two are different four-byte values and are not
interchangeable. Passing `HB_OT_TAG_MATH_SCRIPT` where an `hb_script_t` is
expected will not be diagnosed — both are 32-bit integers — it will simply
select the wrong script.

Since HarfBuzz 3.4.0.

## Types

### `hb_ot_math_constant_t`

```c
typedef enum {
  HB_OT_MATH_CONSTANT_SCRIPT_PERCENT_SCALE_DOWN = 0,
  /* … 56 enumerators, 0 through 55 … */
  HB_OT_MATH_CONSTANT_RADICAL_DEGREE_BOTTOM_RAISE_PERCENT = 55
} hb_ot_math_constant_t;
```
```rust
pub type hb_ot_math_constant_t = core::ffi::c_int;
```

A selector for one of the 56 entries in the `MATH` table's `MathConstants`
sub-table. It is the only argument `hb_ot_math_get_constant()` takes besides the
font.

The C enumeration carries **no private sentinel**, and its largest enumerator is
55, so the underlying C type is a plain signed `int`; the Rust transcription is
therefore a `c_int` alias plus constants. As everywhere in this crate, no Rust
`enum` is emitted — an out-of-range value is not undefined behaviour here, it is
simply a value `hb_ot_math_get_constant()` answers `0` for (its `switch` has a
`default: return 0`).

Values are **not** all in the same units. The table's *Kind* column below tells
you which:

* **Percent** — a raw integer between 0 and 100, straight from the font, not
  scaled at all.
* **y-scaled** — a length along the vertical axis, converted to
  `hb_position_t` using the font's y-scale.
* **x-scaled** — a length along the horizontal axis, converted using the
  font's x-scale.

| # | Constant | `MATH` field | Kind | Meaning |
| --- | --- | --- | --- | --- |
| 0 | `HB_OT_MATH_CONSTANT_SCRIPT_PERCENT_SCALE_DOWN` | `scriptPercentScaleDown` | Percent | Scale applied to script-level (first-level sub/superscript) text. |
| 1 | `HB_OT_MATH_CONSTANT_SCRIPT_SCRIPT_PERCENT_SCALE_DOWN` | `scriptScriptPercentScaleDown` | Percent | Scale applied to script-script-level (second-level and deeper) text. |
| 2 | `HB_OT_MATH_CONSTANT_DELIMITED_SUB_FORMULA_MIN_HEIGHT` | `delimitedSubFormulaMinHeight` | y-scaled | Minimum height a sub-formula in delimiters must reach before the delimiters stretch. |
| 3 | `HB_OT_MATH_CONSTANT_DISPLAY_OPERATOR_MIN_HEIGHT` | `displayOperatorMinHeight` | y-scaled | Minimum height of an n-ary operator (∑, ∏, ∫) in display style. |
| 4 | `HB_OT_MATH_CONSTANT_MATH_LEADING` | `mathLeading` | y-scaled | White space between math formulae to keep line spacing sane. |
| 5 | `HB_OT_MATH_CONSTANT_AXIS_HEIGHT` | `axisHeight` | y-scaled | Height of the math axis above the baseline — where fraction bars and binary operators are centred. |
| 6 | `HB_OT_MATH_CONSTANT_ACCENT_BASE_HEIGHT` | `accentBaseHeight` | y-scaled | Maximum base height above which an accent is raised no further. |
| 7 | `HB_OT_MATH_CONSTANT_FLATTENED_ACCENT_BASE_HEIGHT` | `flattenedAccentBaseHeight` | y-scaled | Base height above which the flattened accent form is used. |
| 8 | `HB_OT_MATH_CONSTANT_SUBSCRIPT_SHIFT_DOWN` | `subscriptShiftDown` | y-scaled | Standard shift down applied to subscripts. |
| 9 | `HB_OT_MATH_CONSTANT_SUBSCRIPT_TOP_MAX` | `subscriptTopMax` | y-scaled | Maximum allowed height of a subscript's top. |
| 10 | `HB_OT_MATH_CONSTANT_SUBSCRIPT_BASELINE_DROP_MIN` | `subscriptBaselineDropMin` | y-scaled | Minimum drop of the subscript baseline below the base's bottom. |
| 11 | `HB_OT_MATH_CONSTANT_SUPERSCRIPT_SHIFT_UP` | `superscriptShiftUp` | y-scaled | Standard shift up applied to superscripts. |
| 12 | `HB_OT_MATH_CONSTANT_SUPERSCRIPT_SHIFT_UP_CRAMPED` | `superscriptShiftUpCramped` | y-scaled | Shift up applied to superscripts in cramped style. |
| 13 | `HB_OT_MATH_CONSTANT_SUPERSCRIPT_BOTTOM_MIN` | `superscriptBottomMin` | y-scaled | Minimum allowed height of a superscript's bottom. |
| 14 | `HB_OT_MATH_CONSTANT_SUPERSCRIPT_BASELINE_DROP_MAX` | `superscriptBaselineDropMax` | y-scaled | Maximum drop of the superscript baseline below the base's top. |
| 15 | `HB_OT_MATH_CONSTANT_SUB_SUPERSCRIPT_GAP_MIN` | `subSuperscriptGapMin` | y-scaled | Minimum gap between a superscript's bottom and a subscript's top. |
| 16 | `HB_OT_MATH_CONSTANT_SUPERSCRIPT_BOTTOM_MAX_WITH_SUBSCRIPT` | `superscriptBottomMaxWithSubscript` | y-scaled | Maximum superscript bottom when a subscript is present too. |
| 17 | `HB_OT_MATH_CONSTANT_SPACE_AFTER_SCRIPT` | `spaceAfterScript` | **x-scaled** | Extra white space after each sub/superscript. |
| 18 | `HB_OT_MATH_CONSTANT_UPPER_LIMIT_GAP_MIN` | `upperLimitGapMin` | y-scaled | Minimum gap between an upper limit and the operator below it. |
| 19 | `HB_OT_MATH_CONSTANT_UPPER_LIMIT_BASELINE_RISE_MIN` | `upperLimitBaselineRiseMin` | y-scaled | Minimum rise of the upper limit's baseline above the operator's top. |
| 20 | `HB_OT_MATH_CONSTANT_LOWER_LIMIT_GAP_MIN` | `lowerLimitGapMin` | y-scaled | Minimum gap between a lower limit and the operator above it. |
| 21 | `HB_OT_MATH_CONSTANT_LOWER_LIMIT_BASELINE_DROP_MIN` | `lowerLimitBaselineDropMin` | y-scaled | Minimum drop of the lower limit's baseline below the operator's bottom. |
| 22 | `HB_OT_MATH_CONSTANT_STACK_TOP_SHIFT_UP` | `stackTopShiftUp` | y-scaled | Shift up of a stack's top element. |
| 23 | `HB_OT_MATH_CONSTANT_STACK_TOP_DISPLAY_STYLE_SHIFT_UP` | `stackTopDisplayStyleShiftUp` | y-scaled | As above, in display style. |
| 24 | `HB_OT_MATH_CONSTANT_STACK_BOTTOM_SHIFT_DOWN` | `stackBottomShiftDown` | y-scaled | Shift down of a stack's bottom element. |
| 25 | `HB_OT_MATH_CONSTANT_STACK_BOTTOM_DISPLAY_STYLE_SHIFT_DOWN` | `stackBottomDisplayStyleShiftDown` | y-scaled | As above, in display style. |
| 26 | `HB_OT_MATH_CONSTANT_STACK_GAP_MIN` | `stackGapMin` | y-scaled | Minimum gap between a stack's elements. |
| 27 | `HB_OT_MATH_CONSTANT_STACK_DISPLAY_STYLE_GAP_MIN` | `stackDisplayStyleGapMin` | y-scaled | As above, in display style. |
| 28 | `HB_OT_MATH_CONSTANT_STRETCH_STACK_TOP_SHIFT_UP` | `stretchStackTopShiftUp` | y-scaled | Shift up of the top element of a stretch stack (over/underbrace). |
| 29 | `HB_OT_MATH_CONSTANT_STRETCH_STACK_BOTTOM_SHIFT_DOWN` | `stretchStackBottomShiftDown` | y-scaled | Shift down of the bottom element of a stretch stack. |
| 30 | `HB_OT_MATH_CONSTANT_STRETCH_STACK_GAP_ABOVE_MIN` | `stretchStackGapAboveMin` | y-scaled | Minimum gap above the stretched element. |
| 31 | `HB_OT_MATH_CONSTANT_STRETCH_STACK_GAP_BELOW_MIN` | `stretchStackGapBelowMin` | y-scaled | Minimum gap below the stretched element. |
| 32 | `HB_OT_MATH_CONSTANT_FRACTION_NUMERATOR_SHIFT_UP` | `fractionNumeratorShiftUp` | y-scaled | Shift up of a fraction numerator. |
| 33 | `HB_OT_MATH_CONSTANT_FRACTION_NUMERATOR_DISPLAY_STYLE_SHIFT_UP` | `fractionNumeratorDisplayStyleShiftUp` | y-scaled | As above, in display style. |
| 34 | `HB_OT_MATH_CONSTANT_FRACTION_DENOMINATOR_SHIFT_DOWN` | `fractionDenominatorShiftDown` | y-scaled | Shift down of a fraction denominator. |
| 35 | `HB_OT_MATH_CONSTANT_FRACTION_DENOMINATOR_DISPLAY_STYLE_SHIFT_DOWN` | `fractionDenominatorDisplayStyleShiftDown` | y-scaled | As above, in display style. |
| 36 | `HB_OT_MATH_CONSTANT_FRACTION_NUMERATOR_GAP_MIN` | `fractionNumeratorGapMin` | y-scaled | Minimum gap between the fraction bar and the numerator. |
| 37 | `HB_OT_MATH_CONSTANT_FRACTION_NUM_DISPLAY_STYLE_GAP_MIN` | `fractionNumDisplayStyleGapMin` | y-scaled | As above, in display style. |
| 38 | `HB_OT_MATH_CONSTANT_FRACTION_RULE_THICKNESS` | `fractionRuleThickness` | y-scaled | Thickness of the fraction bar. |
| 39 | `HB_OT_MATH_CONSTANT_FRACTION_DENOMINATOR_GAP_MIN` | `fractionDenominatorGapMin` | y-scaled | Minimum gap between the fraction bar and the denominator. |
| 40 | `HB_OT_MATH_CONSTANT_FRACTION_DENOM_DISPLAY_STYLE_GAP_MIN` | `fractionDenomDisplayStyleGapMin` | y-scaled | As above, in display style. |
| 41 | `HB_OT_MATH_CONSTANT_SKEWED_FRACTION_HORIZONTAL_GAP` | `skewedFractionHorizontalGap` | **x-scaled** | Horizontal distance between numerator and denominator of a skewed fraction. |
| 42 | `HB_OT_MATH_CONSTANT_SKEWED_FRACTION_VERTICAL_GAP` | `skewedFractionVerticalGap` | y-scaled | Vertical ink distance between numerator and denominator of a skewed fraction. |
| 43 | `HB_OT_MATH_CONSTANT_OVERBAR_VERTICAL_GAP` | `overbarVerticalGap` | y-scaled | Distance between the overbar and the base's ink top. |
| 44 | `HB_OT_MATH_CONSTANT_OVERBAR_RULE_THICKNESS` | `overbarRuleThickness` | y-scaled | Thickness of the overbar. |
| 45 | `HB_OT_MATH_CONSTANT_OVERBAR_EXTRA_ASCENDER` | `overbarExtraAscender` | y-scaled | Extra white space above the overbar. |
| 46 | `HB_OT_MATH_CONSTANT_UNDERBAR_VERTICAL_GAP` | `underbarVerticalGap` | y-scaled | Distance between the underbar and the base's ink bottom. |
| 47 | `HB_OT_MATH_CONSTANT_UNDERBAR_RULE_THICKNESS` | `underbarRuleThickness` | y-scaled | Thickness of the underbar. |
| 48 | `HB_OT_MATH_CONSTANT_UNDERBAR_EXTRA_DESCENDER` | `underbarExtraDescender` | y-scaled | Extra white space below the underbar. |
| 49 | `HB_OT_MATH_CONSTANT_RADICAL_VERTICAL_GAP` | `radicalVerticalGap` | y-scaled | Space between the radicand's ink top and the radical rule. |
| 50 | `HB_OT_MATH_CONSTANT_RADICAL_DISPLAY_STYLE_VERTICAL_GAP` | `radicalDisplayStyleVerticalGap` | y-scaled | As above, in display style. |
| 51 | `HB_OT_MATH_CONSTANT_RADICAL_RULE_THICKNESS` | `radicalRuleThickness` | y-scaled | Thickness of the radical rule. |
| 52 | `HB_OT_MATH_CONSTANT_RADICAL_EXTRA_ASCENDER` | `radicalExtraAscender` | y-scaled | Extra white space above the radical rule. |
| 53 | `HB_OT_MATH_CONSTANT_RADICAL_KERN_BEFORE_DEGREE` | `radicalKernBeforeDegree` | **x-scaled** | Horizontal kern before a radical's degree. |
| 54 | `HB_OT_MATH_CONSTANT_RADICAL_KERN_AFTER_DEGREE` | `radicalKernAfterDegree` | **x-scaled** | Horizontal kern after a radical's degree (normally negative). |
| 55 | `HB_OT_MATH_CONSTANT_RADICAL_DEGREE_BOTTOM_RAISE_PERCENT` | `radicalDegreeBottomRaisePercent` | Percent | Height of the degree's bottom as a percentage of the radical sign's ascender. |

The type and all 56 constants are Since HarfBuzz 1.3.3.

### `hb_ot_math_kern_t`

```c
typedef enum {
  HB_OT_MATH_KERN_TOP_RIGHT    = 0,
  HB_OT_MATH_KERN_TOP_LEFT     = 1,
  HB_OT_MATH_KERN_BOTTOM_RIGHT = 2,
  HB_OT_MATH_KERN_BOTTOM_LEFT  = 3
} hb_ot_math_kern_t;
```
```rust
pub type hb_ot_math_kern_t = core::ffi::c_int;
```

Which of the four corners of a glyph the math-kerning (cut-in) table is being
asked about. A `MATH` font stores an independent kern staircase for each corner,
because a superscript sits at the top right of a base, a pre-superscript at the
top left, and so on.

No sentinel, largest enumerator 3, so signed `int`. Note the ordering — *right*
comes before *left* in each pair, which is easy to misread.

| Constant | Value | Meaning |
| --- | --- | --- |
| `HB_OT_MATH_KERN_TOP_RIGHT` | 0 | The top right corner of the glyph. |
| `HB_OT_MATH_KERN_TOP_LEFT` | 1 | The top left corner of the glyph. |
| `HB_OT_MATH_KERN_BOTTOM_RIGHT` | 2 | The bottom right corner of the glyph. |
| `HB_OT_MATH_KERN_BOTTOM_LEFT` | 3 | The bottom left corner of the glyph. |

Values outside 0–3 are checked for internally and yield `0` from
`hb_ot_math_get_glyph_kerning()` and an empty result from
`hb_ot_math_get_glyph_kernings()`. Since HarfBuzz 1.3.3.

### `hb_ot_math_kern_entry_t`

```c
typedef struct {
  hb_position_t max_correction_height;
  hb_position_t kern_value;
} hb_ot_math_kern_entry_t;
```
```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_ot_math_kern_entry_t {
    pub max_correction_height: hb_position_t,
    pub kern_value: hb_position_t,
}
```

One step of a corner's kern staircase, as returned in bulk by
`hb_ot_math_get_glyph_kernings()`. You allocate the array; HarfBuzz fills it.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `max_correction_height` | `hb_position_t` | `i32` | The maximum height at which this entry should be used. Scaled with the font's **y**-scale. |
| `kern_value` | `hb_position_t` | `i32` | The kern value of the entry. Scaled with the font's **x**-scale. |

The two fields are scaled along different axes, which matters as soon as your
font has a non-square scale. The header carries an important structural note:

> For a glyph with *n* defined kern values (where *n* > 0), there are only
> *n* − 1 defined correction heights, as each correction height defines a
> boundary past which the next kern value should be selected. Therefore, only
> the `kern_value` of the uppermost entry actually comes from the font; its
> corresponding `max_correction_height` is always set to `INT32_MAX`.

So an array of *n* entries always ends with `max_correction_height == i32::MAX`,
which is a sentinel HarfBuzz synthesises, not data. Since HarfBuzz 3.4.0.

### `hb_ot_math_glyph_variant_t`

```c
typedef struct hb_ot_math_glyph_variant_t {
  hb_codepoint_t glyph;
  hb_position_t  advance;
} hb_ot_math_glyph_variant_t;
```
```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_ot_math_glyph_variant_t {
    pub glyph: hb_codepoint_t,
    pub advance: hb_position_t,
}
```

One rung of the size ladder for a stretchy glyph, as returned by
`hb_ot_math_get_glyph_variants()`.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `glyph` | `hb_codepoint_t` | `u32` | The glyph index of the variant. A glyph ID in *this* face — not a Unicode codepoint, despite the type name. |
| `advance` | `hb_position_t` | `i32` | The advance of the variant *in the direction of stretching*: its width for a horizontal stretch, its height for a vertical one. Scaled with the x- or y-scale to match. |

Variants come back in the order the font stores them; HarfBuzz does not sort or
filter them, and the header says nothing about whether the base glyph itself
appears in the list. Treat the ordering as "whatever the font said" and scan the
whole array rather than assuming monotonicity. Since HarfBuzz 1.3.3.

### `hb_ot_math_glyph_part_flags_t`

```c
typedef enum { /*< flags >*/
  HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER = 0x00000001u
} hb_ot_math_glyph_part_flags_t;
```
```rust
pub type hb_ot_math_glyph_part_flags_t = core::ffi::c_int;
```

Bit flags for a glyph-assembly part. Marked `/*< flags >*/` for gtk-doc, so it
is a bitmask type rather than a set of alternatives, even though only one bit is
currently defined. No sentinel, and the only enumerator is 1, so `int`.

| Constant | Value | Meaning |
| --- | --- | --- |
| `HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER` | `0x00000001` | This is an extender glyph part that can be repeated to reach the desired length. |

HarfBuzz masks the raw font value down to the defined bits before handing it
over, so today the `flags` field is only ever `0` or `1`. Test it with a bitwise
`&`, not `==`, so that future flags do not break your code. Since HarfBuzz
1.3.3.

### `hb_ot_math_glyph_part_t`

```c
typedef struct hb_ot_math_glyph_part_t {
  hb_codepoint_t                glyph;
  hb_position_t                 start_connector_length;
  hb_position_t                 end_connector_length;
  hb_position_t                 full_advance;
  hb_ot_math_glyph_part_flags_t flags;
} hb_ot_math_glyph_part_t;
```
```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_ot_math_glyph_part_t {
    pub glyph: hb_codepoint_t,
    pub start_connector_length: hb_position_t,
    pub end_connector_length: hb_position_t,
    pub full_advance: hb_position_t,
    pub flags: hb_ot_math_glyph_part_flags_t,
}
```

One component of a glyph assembly. Per the header: *"Large variants for
stretchable math glyphs (such as parentheses) can be constructed on the fly from
parts."* Returned by `hb_ot_math_get_glyph_assembly()`.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `glyph` | `hb_codepoint_t` | `u32` | The glyph index of the variant part. |
| `start_connector_length` | `hb_position_t` | `i32` | Length of the straight connector material at the *beginning* of the part, in the direction of extension. This is how much of the part may overlap its predecessor. |
| `end_connector_length` | `hb_position_t` | `i32` | Length of the straight connector material at the *end* of the part. How much of the part may overlap its successor. |
| `full_advance` | `hb_position_t` | `i32` | The total advance of the part in the direction of extension. |
| `flags` | `hb_ot_math_glyph_part_flags_t` | `c_int` | See `hb_ot_math_glyph_part_flags_t`. `HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER` marks a part that may be repeated (or omitted). |

All three lengths are scaled along the axis of the stretch — x-scale for
`HB_DIRECTION_LTR`/`RTL`, y-scale for `TTB`/`BTT`. Parts arrive in the font's
order, which HarfBuzz's table code documents as "from left to right and bottom
to top" — so a **vertical** assembly is delivered bottom-first.

Since HarfBuzz 1.3.3.

## The array-fetch convention

Three functions — `hb_ot_math_get_glyph_kernings()`,
`hb_ot_math_get_glyph_variants()`, `hb_ot_math_get_glyph_assembly()` — share one
paging convention:

```c
unsigned int total = hb_ot_math_get_glyph_X (font, …,
                                             start_offset,
                                             &count,   /* IN/OUT */
                                             buffer);  /* OUT */
```

* `start_offset` — index of the first item to write into `buffer`.
* `count` — **in**: capacity of `buffer`; **out**: how many items were written.
* `buffer` — caller-allocated array of at least the in-value of `count` items.
* **return value** — the *total* number of items the font has for this query,
  independent of `start_offset` and `count`.

To size a buffer, call once with a null `buffer` and read the return value, then
call again. To page, advance `start_offset` by the out-value of `count` until
`start_offset + count == total`.

**The trap:** in all three implementations the write-back of `count` is guarded
by `if (count_ptr && buffer_ptr)`. Passing a non-null `count` together with a
null `buffer` therefore leaves your `count` variable **untouched** — not zeroed —
while the return value is still correct. There is one exception:
`hb_ot_math_get_glyph_kernings()` sets `*entries_count = 0` when the corner has
no `MathKern` table at all, even if `kern_entries` is null. Rely on the return
value for totals, and only trust `count` when you also passed a real buffer.

Passing both pointers as null is well defined and is exactly what HarfBuzz's own
test suite does when it only wants the total (or, for the assembly, only wants
the italics correction).

## Functions

Every function below is declared in one `unsafe extern "C"` block in
`harfbuzz_sys::ot_math`. The C signatures are quoted from `hb-ot-math.h`; the
prose and "Since" versions come from the gtk-doc comments upstream keeps in
`hb-ot-math.cc`.

None of these functions allocate, take a reference, mutate their `hb_face_t` /
`hb_font_t`, or require the object to be immutable. There is nothing to destroy
and no ownership transfer anywhere in this header, so the **Ownership** note for
every function is the same and is stated once here rather than repeated below:
inputs are borrowed for the duration of the call, output arrays belong to the
caller, and the return values are plain scalars.

The header does not annotate `face` or `font` as nullable, and every
implementation dereferences them on the first line. Treat them as
**non-null and required**. Passing the singleton empty objects
(`hb_face_get_empty()`, `hb_font_get_empty()`) is well defined and behaves like
a font with no `MATH` table.

### Table presence

#### `hb_ot_math_has_data`

```c
HB_EXTERN hb_bool_t
hb_ot_math_has_data (hb_face_t *face);
```
```rust
pub fn hb_ot_math_has_data(face: *mut hb_face_t) -> hb_bool_t;
```

Tests whether a face has a `MATH` table.

**Parameters** — `face`: the face to test. Non-null.

**Returns** — true (non-zero) if the table is found, false (0) otherwise.

**Notes** — this is the gate you should put in front of everything else in this
header. A "true" answer means the table exists and passed sanitization; it does
*not* promise that any particular sub-table is populated. HarfBuzz's own tests
cover a `MathTestFontEmpty.otf` for which `hb_ot_math_has_data()` returns true
while every constant reads back as 0. Face tables are lazily loaded and cached
internally, so the first call may do I/O-free parsing work; it is safe from
multiple threads. Since HarfBuzz 1.3.3.

### Layout constants

#### `hb_ot_math_get_constant`

```c
HB_EXTERN hb_position_t
hb_ot_math_get_constant (hb_font_t             *font,
                         hb_ot_math_constant_t  constant);
```
```rust
pub fn hb_ot_math_get_constant(
    font: *mut hb_font_t,
    constant: hb_ot_math_constant_t,
) -> hb_position_t;
```

Fetches the specified math constant.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | The font to work upon. Non-null. Values are scaled by this font's x/y scale and reflect its variation coordinates. |
| `constant` | Which constant. Any `int` is accepted; unrecognised values return 0. |

**Returns** — the requested constant, or zero. For most constants the value is
an `hb_position_t` length. For
`HB_OT_MATH_CONSTANT_SCRIPT_PERCENT_SCALE_DOWN`,
`HB_OT_MATH_CONSTANT_SCRIPT_SCRIPT_PERCENT_SCALE_DOWN` and
`HB_OT_MATH_CONSTANT_RADICAL_DEGREE_BOTTOM_RAISE_PERCENT` the return value is
instead "an integer between 0 and 100 representing that percentage".

**Notes**

Zero is ambiguous: it is what you get for a font with no `MATH` table, for a
`MATH` table with no `MathConstants`, for an out-of-range selector, and for a
font that genuinely stores 0. There is no error channel.

Four constants are scaled by the font's **x**-scale rather than its y-scale —
`SPACE_AFTER_SCRIPT`, `SKEWED_FRACTION_HORIZONTAL_GAP`,
`RADICAL_KERN_BEFORE_DEGREE`, `RADICAL_KERN_AFTER_DEGREE` — matching the fact
that they are horizontal measurements. The percentage constants are not scaled
at all.

`MathConstants` values can carry a `Device`/`VariationIndex` table, so the
returned value already includes any hinting delta or variation adjustment
appropriate to the font's size and coordinates.

There is one font-specific workaround baked into this function. Cambria Math
ships an incorrect `displayOperatorMinHeight`, and the Microsoft implementation
effectively swaps `displayOperatorMinHeight` and
`delimitedSubFormulaMinHeight`; HarfBuzz detects that specific bad Cambria Math
and swaps the two constants to match
([issue 4653](https://github.com/harfbuzz/harfbuzz/issues/4653)). Nothing you
need to do — but it explains why the two values may not match a raw table dump.

Since HarfBuzz 1.3.3.

### Per-glyph metrics

#### `hb_ot_math_get_glyph_italics_correction`

```c
HB_EXTERN hb_position_t
hb_ot_math_get_glyph_italics_correction (hb_font_t      *font,
                                         hb_codepoint_t  glyph);
```
```rust
pub fn hb_ot_math_get_glyph_italics_correction(
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
) -> hb_position_t;
```

Fetches an italics-correction value, if one exists, for the specified glyph
index.

**Parameters** — `font`: non-null. `glyph`: a **glyph index** in the font's
face, not a character. Out-of-range or uncovered glyphs are not an error.

**Returns** — the italics correction of the glyph, or zero. Scaled with the
font's x-scale.

**Notes** — the italics correction is the horizontal space to add after a
slanted glyph before setting an upright superscript, so that the superscript
does not collide with the overhanging italic. Zero is returned both for "the
font says zero" and for "the font says nothing"; there is no way to tell them
apart through this API. Since HarfBuzz 1.3.3.

#### `hb_ot_math_get_glyph_top_accent_attachment`

```c
HB_EXTERN hb_position_t
hb_ot_math_get_glyph_top_accent_attachment (hb_font_t      *font,
                                            hb_codepoint_t  glyph);
```
```rust
pub fn hb_ot_math_get_glyph_top_accent_attachment(
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
) -> hb_position_t;
```

Fetches a top-accent-attachment value, if one exists, for the specified glyph
index. This is the x-position, measured from the glyph's origin, at which an
accent placed above the glyph should be centred.

**Parameters** — `font`: non-null. `glyph`: a glyph index.

**Returns** — the top accent attachment of the glyph, or **half the advance
width of `glyph`**. Scaled with the font's x-scale.

**Notes** — this is the one function in the header that *synthesises* a value
rather than returning zero. The header is explicit:

> For any glyph that does not have a top-accent-attachment value — that is, a
> glyph not covered by the `MathTopAccentAttachment` table (or, when `font` has
> no `MathTopAccentAttachment` table or no `MATH` table, any glyph) — the
> function synthesizes a value, returning the position at one-half the glyph's
> advance width.

Consequently a return of `advance / 2` is ambiguous between "the font said so"
and "the font said nothing", and the fallback path calls the font's horizontal
advance callback, so it is affected by anything that changes advances
(variations, `hb_font_set_funcs()`, synthetic bold). Since HarfBuzz 1.3.3.

#### `hb_ot_math_is_glyph_extended_shape`

```c
HB_EXTERN hb_bool_t
hb_ot_math_is_glyph_extended_shape (hb_face_t      *face,
                                    hb_codepoint_t  glyph);
```
```rust
pub fn hb_ot_math_is_glyph_extended_shape(
    face: *mut hb_face_t,
    glyph: hb_codepoint_t,
) -> hb_bool_t;
```

Tests whether the given glyph index is an extended shape in the face.

**Parameters** — `face`: non-null. Note this takes a **face**, not a font — the
answer is a property of the data and does not depend on scale or variations.
`glyph`: a glyph index.

**Returns** — true if the glyph is an extended shape, false otherwise (which
includes "no `MATH` table" and "no `ExtendedShapeCoverage`").

**Notes** — the `MATH` table's extended-shape coverage marks glyphs that are
tall or deep enough that a typesetter should position adjacent material against
their *ink box* rather than against the default heights from `MathConstants`.
The OpenType comment in HarfBuzz's own table code puts it as: "When the left or
right glyph of a box is an extended shape variant, the (ink) box (and not the
default position defined by values in MathConstants table) should be used for
vertical positioning purposes."

Because this takes a face while its siblings take a font, it is easy to write
`hb_ot_math_is_glyph_extended_shape(font, glyph)` by mistake; both are pointers
and C will only warn. In Rust the types differ, so the compiler catches it.
Since HarfBuzz 1.3.3.

### Math kerning (cut-ins)

#### `hb_ot_math_get_glyph_kerning`

```c
HB_EXTERN hb_position_t
hb_ot_math_get_glyph_kerning (hb_font_t         *font,
                              hb_codepoint_t     glyph,
                              hb_ot_math_kern_t  kern,
                              hb_position_t      correction_height);
```
```rust
pub fn hb_ot_math_get_glyph_kerning(
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
    kern: hb_ot_math_kern_t,
    correction_height: hb_position_t,
) -> hb_position_t;
```

Fetches the math kerning (cut-in) value for the specified font, glyph index and
corner. This is the convenience form: you give it a height and it picks the
right rung of the staircase for you.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | Non-null. |
| `glyph` | Glyph index of the *base* glyph whose corner is being kerned. |
| `kern` | Which corner. Values outside 0–3 return 0. |
| `correction_height` | The correction height to use to determine the kerning, in the font's y-scaled units — i.e. the same units `hb_ot_math_kern_entry_t::max_correction_height` comes back in, **not** design units. |

**Returns** — the requested kerning value, or zero. Scaled with the font's
x-scale.

**Notes** — the selection rule, from the header:

> If the `MathKern` table is found, the function examines it to find a height
> value that is greater or equal to `correction_height`. If such a height value
> is found, corresponding kerning value from the table is returned. If no such
> height value is found, the last kerning value is returned.

Concretely the implementation binary-searches for the index *i* satisfying
`correctionHeight[i-1] <= correction_height < correctionHeight[i]`, per the
OpenType spec, and it accounts for a negative y-scale (a flipped coordinate
system) by reversing the comparison sign. So the staircase is a step function:
below the first boundary you get the first kern value; at or above the last
boundary you get the last one.

Zero is returned when the face has no `MATH` table, no `MathGlyphInfo`, no
`MathKernInfo`, an empty `MathKernInfoRecords` array, or when this glyph is not
covered. Since HarfBuzz 1.3.3.

#### `hb_ot_math_get_glyph_kernings`

```c
HB_EXTERN unsigned int
hb_ot_math_get_glyph_kernings (hb_font_t               *font,
                               hb_codepoint_t           glyph,
                               hb_ot_math_kern_t        kern,
                               unsigned int             start_offset,
                               unsigned int            *entries_count, /* IN/OUT */
                               hb_ot_math_kern_entry_t *kern_entries   /* OUT */);
```
```rust
pub fn hb_ot_math_get_glyph_kernings(
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
    kern: hb_ot_math_kern_t,
    start_offset: c_uint,
    entries_count: *mut c_uint,
    kern_entries: *mut hb_ot_math_kern_entry_t,
) -> c_uint;
```

Fetches the **raw** `MathKern` (cut-in) data for the specified font, glyph index
and corner — the whole staircase, rather than one step of it.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | Non-null. |
| `glyph` | Glyph index of the base glyph. |
| `kern` | Which corner. Out-of-range values yield 0 entries. |
| `start_offset` | Index of the first entry to retrieve. Clamped to the total, so an offset past the end simply produces nothing. |
| `entries_count` | In/out. **Nullable** (`(optional)` upstream). In: capacity of `kern_entries`. Out: number of entries written. |
| `kern_entries` | Caller-allocated output array of at least `*entries_count` items. **Nullable.** |

**Returns** — the total number of kern values available, or zero. Independent of
`start_offset` and `entries_count`.

**Notes** — see [The array-fetch convention](#the-array-fetch-convention);
`*entries_count` is only written when *both* pointers are non-null, except that
a glyph/corner with no `MathKern` at all sets `*entries_count = 0` and returns
0.

The header points you at `hb_ot_math_get_glyph_kerning()` for the common case of
"I have a height, give me a kern". Use this function when you want to inspect or
re-implement the staircase — for instance to apply the TeX-ish rule of sampling
the kern at several heights.

Remember the *n* values / *n* − 1 heights asymmetry: the last entry's
`max_correction_height` is always `INT32_MAX` (`i32::MAX`) and is synthesised,
not read from the font.

Since HarfBuzz 3.4.0.

### Stretchy glyph construction

#### `hb_ot_math_get_glyph_variants`

```c
HB_EXTERN unsigned int
hb_ot_math_get_glyph_variants (hb_font_t                  *font,
                               hb_codepoint_t              glyph,
                               hb_direction_t              direction,
                               unsigned int                start_offset,
                               unsigned int               *variants_count, /* IN/OUT */
                               hb_ot_math_glyph_variant_t *variants        /* OUT */);
```
```rust
pub fn hb_ot_math_get_glyph_variants(
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
    direction: hb_direction_t,
    start_offset: c_uint,
    variants_count: *mut c_uint,
    variants: *mut hb_ot_math_glyph_variant_t,
) -> c_uint;
```

Fetches the `MathGlyphConstruction` for the specified font, glyph index and
direction. "The corresponding list of size variants is returned as a list of
`hb_ot_math_glyph_variant_t` structs."

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | Non-null. |
| `glyph` | The index of the glyph to stretch. |
| `direction` | The direction of the *stretching*. See the note below. |
| `start_offset` | Index of the first variant to retrieve. |
| `variants_count` | In/out: capacity, then number written. Nullable in practice. |
| `variants` | Caller-allocated output array. **Nullable** per upstream's annotation. |

**Returns** — the total number of size variants available, or zero.

**Notes** — the header's warning about `direction`:

> The `direction` parameter is only used to select between horizontal or
> vertical directions for the construction. Even though all `hb_direction_t`
> values are accepted, only the result of `HB_DIRECTION_IS_HORIZONTAL` is
> considered.

In the implementation the test is actually `HB_DIRECTION_IS_VERTICAL`, so the
practical rule is: `HB_DIRECTION_TTB` and `HB_DIRECTION_BTT` select the vertical
construction; **everything else, including `HB_DIRECTION_INVALID`, selects the
horizontal one**. If you have a buffer direction to hand, pass it straight
through; if you are asking about a vertically stretchy glyph such as a
parenthesis, pass `HB_DIRECTION_TTB` regardless of the text direction.

`direction` also picks the scale axis for `hb_ot_math_glyph_variant_t::advance`
(x-scale for horizontal, y-scale for vertical).

Zero means the glyph has no construction for that axis — either the whole
`MathVariants` table is missing, or the glyph is not in the relevant coverage.
A glyph can have a vertical construction and no horizontal one, which is the
normal case for delimiters.

Since HarfBuzz 1.3.3.

#### `hb_ot_math_get_min_connector_overlap`

```c
HB_EXTERN hb_position_t
hb_ot_math_get_min_connector_overlap (hb_font_t      *font,
                                      hb_direction_t  direction);
```
```rust
pub fn hb_ot_math_get_min_connector_overlap(
    font: *mut hb_font_t,
    direction: hb_direction_t,
) -> hb_position_t;
```

Fetches the `MathVariants` table for the specified font and returns the minimum
overlap of connecting glyphs required to draw a glyph assembly in the specified
direction.

**Parameters** — `font`: non-null. `direction`: direction of the stretching;
same horizontal/vertical-only rule as above, and it selects the scale axis.

**Returns** — the requested minimum connector overlap, or zero.

**Notes** — this is a single per-font value (`minConnectorOverlap`), not a
per-glyph one, and it is the *floor* on how much consecutive parts of an
assembly overlap. The ceiling is not returned by any function here; it follows
from the connector lengths in `hb_ot_math_glyph_part_t`, since only the straight
connector material at each end of a part may be overlapped — that is,
`min(end_connector_length of the earlier part, start_connector_length of the
later part)`. If the floor exceeds that ceiling the font is inconsistent, and
this API will not tell you so. Since HarfBuzz 1.3.3.

#### `hb_ot_math_get_glyph_assembly`

```c
HB_EXTERN unsigned int
hb_ot_math_get_glyph_assembly (hb_font_t               *font,
                               hb_codepoint_t           glyph,
                               hb_direction_t           direction,
                               unsigned int             start_offset,
                               unsigned int            *parts_count,        /* IN/OUT */
                               hb_ot_math_glyph_part_t *parts,              /* OUT */
                               hb_position_t           *italics_correction  /* OUT */);
```
```rust
pub fn hb_ot_math_get_glyph_assembly(
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
    direction: hb_direction_t,
    start_offset: c_uint,
    parts_count: *mut c_uint,
    parts: *mut hb_ot_math_glyph_part_t,
    italics_correction: *mut hb_position_t,
) -> c_uint;
```

Fetches the `GlyphAssembly` for the specified font, glyph index and direction —
the recipe for building a stretchy glyph taller (or wider) than the largest
pre-drawn variant.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | Non-null. |
| `glyph` | The index of the glyph to stretch. |
| `direction` | Direction of the stretching; same horizontal/vertical-only rule as `hb_ot_math_get_glyph_variants()`, and it selects the scale axis for every length in `hb_ot_math_glyph_part_t`. |
| `start_offset` | Index of the first glyph part to retrieve. |
| `parts_count` | In/out: capacity, then number written. Nullable in practice. |
| `parts` | Caller-allocated output array. **Nullable** per upstream's annotation. |
| `italics_correction` | Out: italics correction of the glyph assembly. The implementation checks it for null before writing, so null is accepted; when non-null it is written **unconditionally**, even when `parts` is null and even when the assembly is empty (in which case it receives 0). |

**Returns** — the total number of parts in the glyph assembly. Zero means there
is no assembly for this glyph and axis.

**Notes** — the assembly's italics correction is a property of the whole
assembly and, per the OpenType spec comment in HarfBuzz's table code, "should
not depend on the assembly size". It is x-scaled regardless of `direction`.
Because `italics_correction` is filled in independently of the parts array, the
idiomatic way to ask only for it is
`hb_ot_math_get_glyph_assembly(font, glyph, dir, 0, NULL, NULL, &corr)` — which
is precisely what HarfBuzz's own test suite does.

Parts arrive in font order: left to right for a horizontal assembly, bottom to
top for a vertical one. Since HarfBuzz 1.3.3.

## Usage

### Detecting a math font

```c
#include <hb-ot.h>

hb_blob_t *blob = hb_blob_create_from_file ("STIXTwoMath-Regular.otf");
hb_face_t *face = hb_face_create (blob, 0);
hb_font_t *font = hb_font_create (face);
hb_font_set_scale (font, 1000, 1000);

if (!hb_ot_math_has_data (face)) {
  /* Not a math font: fall back to your own metrics. */
}
```

```rust
use harfbuzz_sys::{
    hb_blob_create_from_file, hb_face_create, hb_font_create, hb_font_set_scale,
    hb_ot_math_has_data,
};

let blob = unsafe { hb_blob_create_from_file(c"STIXTwoMath-Regular.otf".as_ptr()) };
let face = unsafe { hb_face_create(blob, 0) };
let font = unsafe { hb_font_create(face) };
unsafe { hb_font_set_scale(font, 1000, 1000) };

let is_math_font = unsafe { hb_ot_math_has_data(face) } != 0;
```

### Reading layout constants

```c
hb_position_t axis      = hb_ot_math_get_constant (font, HB_OT_MATH_CONSTANT_AXIS_HEIGHT);
hb_position_t rule      = hb_ot_math_get_constant (font, HB_OT_MATH_CONSTANT_FRACTION_RULE_THICKNESS);
hb_position_t sup_shift = hb_ot_math_get_constant (font, HB_OT_MATH_CONSTANT_SUPERSCRIPT_SHIFT_UP);

/* Percentages, not lengths: */
int script_scale = hb_ot_math_get_constant (font, HB_OT_MATH_CONSTANT_SCRIPT_PERCENT_SCALE_DOWN);
/* e.g. 70 → render first-level scripts at 70% of the current size. */
```

```rust
use harfbuzz_sys::{
    HB_OT_MATH_CONSTANT_AXIS_HEIGHT, HB_OT_MATH_CONSTANT_FRACTION_RULE_THICKNESS,
    HB_OT_MATH_CONSTANT_SCRIPT_PERCENT_SCALE_DOWN, hb_ot_math_get_constant,
};

let axis = unsafe { hb_ot_math_get_constant(font, HB_OT_MATH_CONSTANT_AXIS_HEIGHT) };
let rule = unsafe { hb_ot_math_get_constant(font, HB_OT_MATH_CONSTANT_FRACTION_RULE_THICKNESS) };
let script_scale =
    unsafe { hb_ot_math_get_constant(font, HB_OT_MATH_CONSTANT_SCRIPT_PERCENT_SCALE_DOWN) };
let script_factor = script_scale as f32 / 100.0;
```

### Positioning a superscript on a base glyph

The three per-glyph queries compose into the classic recipe: shift the script
up, then tuck it in horizontally by the italics correction plus the corner kern
evaluated at the height where the two boxes meet.

```c
hb_position_t shift = hb_ot_math_get_constant (font, HB_OT_MATH_CONSTANT_SUPERSCRIPT_SHIFT_UP);
hb_position_t ital  = hb_ot_math_get_glyph_italics_correction (font, base_glyph);

/* Height at which the superscript's bottom meets the base's top right. */
hb_position_t h = shift;

hb_position_t kern = hb_ot_math_get_glyph_kerning (font, base_glyph,
                                                   HB_OT_MATH_KERN_TOP_RIGHT, h);

hb_position_t dx = ital + kern;   /* horizontal offset for the superscript */
hb_position_t dy = shift;         /* vertical offset */
```

```rust
use harfbuzz_sys::{
    HB_OT_MATH_CONSTANT_SUPERSCRIPT_SHIFT_UP, HB_OT_MATH_KERN_TOP_RIGHT, hb_ot_math_get_constant,
    hb_ot_math_get_glyph_italics_correction, hb_ot_math_get_glyph_kerning,
};

let shift = unsafe { hb_ot_math_get_constant(font, HB_OT_MATH_CONSTANT_SUPERSCRIPT_SHIFT_UP) };
let ital = unsafe { hb_ot_math_get_glyph_italics_correction(font, base_glyph) };
let kern =
    unsafe { hb_ot_math_get_glyph_kerning(font, base_glyph, HB_OT_MATH_KERN_TOP_RIGHT, shift) };

let (dx, dy) = (ital + kern, shift);
```

### Centring an accent over a base

```c
hb_position_t attach = hb_ot_math_get_glyph_top_accent_attachment (font, base_glyph);
hb_position_t accent = hb_ot_math_get_glyph_top_accent_attachment (font, accent_glyph);

/* Draw the accent so that its attachment point lines up with the base's. */
hb_position_t dx = attach - accent;
```

### Dumping a corner's kern staircase

```c
unsigned total = hb_ot_math_get_glyph_kernings (font, glyph,
                                                HB_OT_MATH_KERN_TOP_RIGHT,
                                                0, NULL, NULL);

hb_ot_math_kern_entry_t *entries = malloc (total * sizeof (*entries));
unsigned count = total;
hb_ot_math_get_glyph_kernings (font, glyph, HB_OT_MATH_KERN_TOP_RIGHT,
                               0, &count, entries);

for (unsigned i = 0; i < count; i++)
  printf ("h < %d → kern %d\n",
          entries[i].max_correction_height,   /* INT32_MAX on the last one */
          entries[i].kern_value);
free (entries);
```

```rust
use harfbuzz_sys::{
    HB_OT_MATH_KERN_TOP_RIGHT, hb_ot_math_get_glyph_kernings, hb_ot_math_kern_entry_t,
};

let total = unsafe {
    hb_ot_math_get_glyph_kernings(
        font,
        glyph,
        HB_OT_MATH_KERN_TOP_RIGHT,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
};

let mut entries = vec![
    hb_ot_math_kern_entry_t { max_correction_height: 0, kern_value: 0 };
    total as usize
];
let mut count = total;
unsafe {
    hb_ot_math_get_glyph_kernings(
        font,
        glyph,
        HB_OT_MATH_KERN_TOP_RIGHT,
        0,
        &mut count,
        entries.as_mut_ptr(),
    );
}
entries.truncate(count as usize);
```

### Growing a delimiter: variants, then assembly

The standard algorithm for stretching a delimiter to a target size: walk the
size variants, and if none is big enough fall back to the assembly.

```c
hb_position_t target = 3000;   /* desired height, font units */

/* 1. Try the pre-drawn size variants. */
unsigned n = hb_ot_math_get_glyph_variants (font, glyph, HB_DIRECTION_TTB,
                                            0, NULL, NULL);
if (n) {
  hb_ot_math_glyph_variant_t *v = malloc (n * sizeof (*v));
  unsigned count = n;
  hb_ot_math_get_glyph_variants (font, glyph, HB_DIRECTION_TTB, 0, &count, v);

  for (unsigned i = 0; i < count; i++)
    if (v[i].advance >= target) { use_glyph (v[i].glyph); free (v); return; }

  free (v);
}

/* 2. Otherwise build an assembly. */
hb_position_t italics = 0;
unsigned p = hb_ot_math_get_glyph_assembly (font, glyph, HB_DIRECTION_TTB,
                                            0, NULL, NULL, &italics);
if (p) {
  hb_ot_math_glyph_part_t *parts = malloc (p * sizeof (*parts));
  unsigned count = p;
  hb_ot_math_get_glyph_assembly (font, glyph, HB_DIRECTION_TTB,
                                 0, &count, parts, &italics);

  hb_position_t min_overlap =
    hb_ot_math_get_min_connector_overlap (font, HB_DIRECTION_TTB);

  /* Parts are bottom-to-top for a vertical assembly. Repeat the parts whose
   * flags include HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER until the stack reaches
   * `target`, overlapping each junction by at least `min_overlap` and at most
   * min(prev.end_connector_length, next.start_connector_length). */
  free (parts);
}
```

```rust
use harfbuzz_sys::{
    HB_DIRECTION_TTB, HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER, hb_ot_math_get_glyph_assembly,
    hb_ot_math_get_glyph_variants, hb_ot_math_get_min_connector_overlap, hb_ot_math_glyph_part_t,
    hb_ot_math_glyph_variant_t, hb_position_t,
};

let target: hb_position_t = 3000;

// 1. Pre-drawn size variants.
let n = unsafe {
    hb_ot_math_get_glyph_variants(
        font,
        glyph,
        HB_DIRECTION_TTB,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    )
};
let mut chosen = None;
if n > 0 {
    let mut variants = vec![hb_ot_math_glyph_variant_t { glyph: 0, advance: 0 }; n as usize];
    let mut count = n;
    unsafe {
        hb_ot_math_get_glyph_variants(
            font,
            glyph,
            HB_DIRECTION_TTB,
            0,
            &mut count,
            variants.as_mut_ptr(),
        );
    }
    chosen = variants[..count as usize]
        .iter()
        .find(|v| v.advance >= target)
        .map(|v| v.glyph);
}

// 2. Assembly fallback.
if chosen.is_none() {
    let mut italics: hb_position_t = 0;
    let p = unsafe {
        hb_ot_math_get_glyph_assembly(
            font,
            glyph,
            HB_DIRECTION_TTB,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            &mut italics,
        )
    };
    if p > 0 {
        let mut parts = vec![
            hb_ot_math_glyph_part_t {
                glyph: 0,
                start_connector_length: 0,
                end_connector_length: 0,
                full_advance: 0,
                flags: 0,
            };
            p as usize
        ];
        let mut count = p;
        unsafe {
            hb_ot_math_get_glyph_assembly(
                font,
                glyph,
                HB_DIRECTION_TTB,
                0,
                &mut count,
                parts.as_mut_ptr(),
                &mut italics,
            );
        }
        let min_overlap =
            unsafe { hb_ot_math_get_min_connector_overlap(font, HB_DIRECTION_TTB) };

        let extenders = parts[..count as usize]
            .iter()
            .filter(|p| p.flags & HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER != 0)
            .count();
        let _ = (min_overlap, extenders, italics);
    }
}
```

### Asking only for an assembly's italics correction

```c
hb_position_t corr;
hb_ot_math_get_glyph_assembly (font, glyph, HB_DIRECTION_TTB, 0, NULL, NULL, &corr);
```

```rust
let mut corr: hb_position_t = 0;
unsafe {
    hb_ot_math_get_glyph_assembly(
        font,
        glyph,
        HB_DIRECTION_TTB,
        0,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        &mut corr,
    );
}
```

## Pitfalls

### Zero means several different things

Every value-returning function in this header answers `0` for "no `MATH`
table", "no sub-table", "glyph not covered" and "the font really does store
zero". There is no out-of-band error, no `hb_bool_t` success flag, and no
"unset" sentinel. Gate on `hb_ot_math_has_data()` once, and after that accept
that you cannot distinguish an absent value from a zero one.

### Three constants are percentages, four are x-scaled

`hb_ot_math_get_constant()` returns an `hb_position_t` for all 56 selectors, but
`SCRIPT_PERCENT_SCALE_DOWN`, `SCRIPT_SCRIPT_PERCENT_SCALE_DOWN` and
`RADICAL_DEGREE_BOTTOM_RAISE_PERCENT` are integers in 0–100, unaffected by
`hb_font_set_scale()`. Treating them as lengths produces absurdly small numbers;
treating a length as a percentage produces absurdly large ones.

Separately, `SPACE_AFTER_SCRIPT`, `SKEWED_FRACTION_HORIZONTAL_GAP`,
`RADICAL_KERN_BEFORE_DEGREE` and `RADICAL_KERN_AFTER_DEGREE` are scaled by the
font's **x**-scale while every other length constant uses the y-scale. This is
invisible with a square scale and obvious the moment you call
`hb_font_set_scale(font, 2000, 1000)`.

### `direction` is a two-way switch, and the header's wording is misleading

The gtk-doc note says only `HB_DIRECTION_IS_HORIZONTAL` is considered; the code
actually tests `HB_DIRECTION_IS_VERTICAL`. The observable rule is the same for
valid directions but differs for `HB_DIRECTION_INVALID` (0), which is neither:
it selects the **horizontal** construction. Delimiters almost always stretch
vertically, so passing a buffer's LTR direction to
`hb_ot_math_get_glyph_variants()` silently returns nothing. Pass
`HB_DIRECTION_TTB` when you mean "grow this taller".

### `count` is not written when the buffer is null

The paging convention writes back `*count` only when both the count pointer and
the array pointer are non-null. Code shaped like

```c
unsigned count = 0;
unsigned total = hb_ot_math_get_glyph_variants (font, g, dir, 0, &count, NULL);
/* count is STILL 0 here — and that is not the answer you wanted. */
```

works by accident because `count` was initialised to 0; the same code with an
uninitialised `count` reads garbage. Use the **return value** for totals.

### The last kern entry's height is a synthetic sentinel

`hb_ot_math_get_glyph_kernings()` always terminates the array with an entry
whose `max_correction_height` is `INT32_MAX`. That number is not in the font.
Do not plot it, do not scale it, and beware that in Rust `i32::MAX` scaled or
added to anything will overflow in debug builds.

### `max_correction_height` and `kern_value` use different axes

Within one `hb_ot_math_kern_entry_t`, the height is y-scaled and the kern is
x-scaled. The `correction_height` you pass to `hb_ot_math_get_glyph_kerning()`
must therefore be in y-scaled units, matching the heights — not in design units
and not in the kern's units.

### Top-accent attachment silently invents a value

Unlike everything else here, `hb_ot_math_get_glyph_top_accent_attachment()`
never fails visibly: it returns half the glyph's advance width when the font has
nothing to say. That is a sensible default for symmetric glyphs and wrong for
asymmetric ones. If you need to know whether the font actually specified a
value, this API cannot tell you — compare against `advance / 2` and accept the
false positives, or read the table yourself via
`hb_face_reference_table(face, HB_OT_TAG_MATH)`.

### Face versus font

`hb_ot_math_has_data()` and `hb_ot_math_is_glyph_extended_shape()` take an
`hb_face_t *`; the other eight take an `hb_font_t *`. In C both are pointers and
a mix-up compiles with at most a warning, then reads through the wrong struct.
Rust's type system rejects it, so this is mainly a hazard when porting C
examples.

### Values track the font, not the face

Everything font-level here is scaled by `hb_font_set_scale()` and adjusted for
the font's variation coordinates and any `Device`/`VariationIndex` deltas.
Nothing is cached for you and nothing notifies you of a change: if you memoise
constants, key the cache on the font and invalidate it whenever you call
`hb_font_set_scale()`, `hb_font_set_ppem()`, `hb_font_set_variations()` or the
`var_coords` setters.

### `HB_OT_TAG_MATH_SCRIPT` is not `HB_SCRIPT_MATH`

`HB_OT_TAG_MATH_SCRIPT` is the OpenType script tag `math`;
`HB_SCRIPT_MATH` is the HarfBuzz script value `Zmth`. Both are 32-bit integers
and neither the C nor the Rust type system will stop you swapping them, because
`hb_script_t` is itself a tag alias. Use the OpenType tag only with APIs that
say "OpenType script tag".

### Glyph indices, not characters

Every `glyph` parameter is an `hb_codepoint_t`, which is the same type HarfBuzz
uses for Unicode codepoints — but here it always means a **glyph index** in this
face. Map characters to glyphs with `hb_font_get_nominal_glyph()` (or take the
glyph IDs out of a shaped buffer) first. Passing U+221A instead of the radical's
glyph ID compiles fine and returns zeros.

### Reduced-feature builds

`hb-ot-math.cc` is compiled only when `HB_NO_MATH` is undefined, while the
header always declares the functions. Against a HarfBuzz built with `HB_NO_MATH`
you get link errors, not runtime zeros. This crate's `build.rs` sets no `HB_NO_*`
macros, so the functions are present in every configuration it produces.

### Thread safety

All ten functions only read from the face's lazily-loaded `MATH` table and from
the font's scale/variation state, so concurrent calls from several threads are
fine provided no thread is mutating the font at the same time. Table loading is
internally synchronised. The usual discipline of calling
`hb_font_make_immutable()` before sharing a font applies here as well.
