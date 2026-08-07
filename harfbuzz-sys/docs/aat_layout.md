# AAT layout

Header: `hb-aat-layout.h` (also reachable through the umbrella `hb-aat.h`) —
Rust module: `harfbuzz_sys::aat_layout`, glob re-exported at the crate root.
No Cargo feature is needed: AAT support is part of the core library on every
platform.

## Overview

**Apple Advanced Typography** is Apple's layout technology, and it is the older
of the two systems that ride on top of TrueType. Where OpenType Layout describes
shaping declaratively — a `GSUB` table full of typed lookups (single
substitution, ligature substitution, contextual chaining) that the shaping engine
knows how to interpret, selected by script, language and feature tag — AAT
describes shaping as **finite state machines**. An AAT font's `morx` table
contains state tables that the engine runs over the glyph stream: rearrangement,
contextual substitution, ligature, insertion. The font author, in effect, writes
a small program; the engine is an interpreter.

Two consequences follow, and they shape the whole API.

**AAT has no script/language model.** OpenType selects lookups through
`ScriptList` → `LangSysList` → `FeatureList`. AAT has nothing of the kind — its
state machines see the glyph stream and nothing else. (The `ltag` table can
attach BCP 47 language tags to `morx` subtables, but that is the extent of it.)
This is why HarfBuzz's script-specific shapers largely stand down when AAT
shaping is in play: with no way to tell the font "this is Devanagari", the
shaper's script-specific reordering would fight the font's own state machines.

**AAT features are numbered, not tagged.** OpenType names a feature with a
four-byte tag (`liga`, `smcp`, `ss01`). AAT names a *feature type* with a small
integer from Apple's Font Feature Registry, and each type offers a set of
numbered *selectors*: type 1 is Ligatures, and within it selector 2 is "common
ligatures on" while selector 3 is "common ligatures off". The `feat` table maps
those numbers onto human-readable strings in the `name` table, which is what
Apple's font panels display.

That numbering scheme is what this header is mostly about. It gives you the two
enumerations (`hb_aat_layout_feature_type_t`, `hb_aat_layout_feature_selector_t`),
a struct describing one setting (`hb_aat_layout_feature_selector_info_t`), and
three functions for reading a face's `feat` table — plus three cheap predicates
for "does this face carry AAT substitution / positioning / tracking data?"

### What HarfBuzz supports

Upstream's own summary: *"HarfBuzz supports all of the AAT tables used to
implement shaping. Other AAT tables and their associated features are not
supported."* Concretely, the tables HarfBuzz loads are:

| Table | Role | Used by HarfBuzz |
| --- | --- | --- |
| `morx` | Extended glyph metamorphosis — the substitution state machines | Yes, this is AAT shaping |
| `mort` | Legacy glyph metamorphosis (pre-`morx`) | Yes |
| `kerx` | Extended kerning, including state-machine and cross-stream kerning | Yes |
| `ankr` | Anchor points, referenced by `kerx` | Yes |
| `trak` | Tracking (size-dependent letterspacing) | Yes |
| `ltag` | Language tags for `morx` subtables | Yes |
| `feat` | Feature-type / selector names — the data behind this header's query API | Yes |
| `bsln` | Baselines | Compiled but unused |
| `just` | Justification | Compiled but unused |
| `opbd` | Optical bounds | Compiled but unused (commented out of the table list) |

### When the AAT shaper runs

There is no `hb_shape` flag for "use AAT". The decision is made per
face-and-direction while the shape plan is compiled, in `hb-ot-shape.cc`:

```c
static inline bool
_hb_apply_morx (hb_face_t *face, const hb_segment_properties_t &props)
{
  return hb_aat_layout_has_substitution (face) &&
         (HB_DIRECTION_IS_HORIZONTAL (props.direction) || !hb_ot_layout_has_substitution (face));
}
```

In words: **`morx`/`mort` wins over `GSUB`** whenever the face has AAT
substitution data *and* either the run is horizontal or the face has no `GSUB` at
all. The vertical carve-out exists because AAT fonts routinely lack vertical
handling that their `GSUB` does provide.

Several other decisions follow from that flag:

- If `morx` will be applied and the categorised shaper is anything other than
  the *default* shaper, HarfBuzz downgrades to the `dumber` shaper — the
  script-specific reordering is turned off, for the reason given above.
- Positioning is decided separately. `kerx` is preferred over `GPOS` unless the
  face has **both** `GSUB` and `GPOS`, in which case `GPOS` wins. If neither
  applies, HarfBuzz falls back to `kerx`, then to the legacy `kern` table, then
  to its own fallback kerning.
- Mark-position adjustment when zeroing mark advances is disabled under `morx`,
  because Apple Color Emoji assumes it is not done when forming emoji sequences.
- `trak` is applied when the face has tracking data **and** a `STAT` table —
  Apple's own heuristic for "this is a modern font".

So the three predicates on this page (`hb_aat_layout_has_substitution`,
`hb_aat_layout_has_positioning`, `hb_aat_layout_has_tracking`) are not
decorative: they are literally the inputs to that decision, exposed so a caller
can reason about it.

### How user features reach AAT

You still pass `hb_feature_t` values with OpenType tags to `hb_shape()`. Under
the AAT shaper, HarfBuzz translates them through a fixed table (in
`hb-aat-layout.cc`, "courtesy of Apple") that maps an OpenType tag to a feature
type plus an on-selector and an off-selector. A tag not in that table is silently
dropped for AAT shaping. Some of the mappings:

| OpenType tag | AAT feature type | Selector when on | Selector when off |
| --- | --- | --- | --- |
| `liga` | 1 Ligatures | 2 `COMMON_LIGATURES_ON` | 3 `COMMON_LIGATURES_OFF` |
| `dlig` | 1 Ligatures | 4 `RARE_LIGATURES_ON` | 5 `RARE_LIGATURES_OFF` |
| `hlig` | 1 Ligatures | 20 `HISTORICAL_LIGATURES_ON` | 21 `HISTORICAL_LIGATURES_OFF` |
| `clig` | 1 Ligatures | 18 `CONTEXTUAL_LIGATURES_ON` | 19 `CONTEXTUAL_LIGATURES_OFF` |
| `rlig` | 1 Ligatures | 0 `REQUIRED_LIGATURES_ON` | 1 `REQUIRED_LIGATURES_OFF` |
| `smcp` | 37 Lower Case | 1 `LOWER_CASE_SMALL_CAPS` | 0 `DEFAULT_LOWER_CASE` |
| `pcap` | 37 Lower Case | 2 `LOWER_CASE_PETITE_CAPS` | 0 `DEFAULT_LOWER_CASE` |
| `c2sc` | 38 Upper Case | 1 `UPPER_CASE_SMALL_CAPS` | 0 `DEFAULT_UPPER_CASE` |
| `c2pc` | 38 Upper Case | 2 `UPPER_CASE_PETITE_CAPS` | 0 `DEFAULT_UPPER_CASE` |
| `calt` | 36 Contextual Alternatives | 0 `CONTEXTUAL_ALTERNATES_ON` | 1 `CONTEXTUAL_ALTERNATES_OFF` |
| `swsh` | 36 Contextual Alternatives | 2 `SWASH_ALTERNATES_ON` | 3 `SWASH_ALTERNATES_OFF` |
| `cswh` | 36 Contextual Alternatives | 4 `CONTEXTUAL_SWASH_ALTERNATES_ON` | 5 `CONTEXTUAL_SWASH_ALTERNATES_OFF` |
| `frac` | 11 Fractions | 2 `DIAGONAL_FRACTIONS` | 0 `NO_FRACTIONS` |
| `afrc` | 11 Fractions | 1 `VERTICAL_FRACTIONS` | 0 `NO_FRACTIONS` |
| `zero` | 14 Typographic Extras | 4 `SLASHED_ZERO_ON` | 5 `SLASHED_ZERO_OFF` |
| `sups` / `subs` / `sinf` / `ordn` | 10 Vertical Position | 1 / 2 / 4 / 3 | 0 `NORMAL_POSITION` |
| `onum` / `lnum` | 21 Number Case | 0 / 1 | 2 (no named constant) |
| `pnum` / `tnum` | 6 Number Spacing | 1 / 0 | 4 (no named constant) |
| `fwid` / `hwid` / `twid` / `qwid` / `pwid` / `palt` / `halt` | 22 Text Spacing | 1 / 2 / 3 / 4 / 0 / 5 / 6 | 7 (no named constant) |
| `ss01` … `ss20` | 35 Stylistic Alternatives | 2, 4, 6 … 40 | 3, 5, 7 … 41 |
| `case` / `cpsp` | 33 Case Sensitive Layout | 0 / 2 | 1 / 3 |
| `ital` | 32 Italic CJK Roman | 2 `CJK_ITALIC_ROMAN_ON` | 3 `CJK_ITALIC_ROMAN_OFF` |
| `ruby` | 28 Ruby Kana | 2 `RUBY_KANA_ON` | 3 `RUBY_KANA_OFF` |
| `hkna` / `vkna` | 34 Alternate Kana | 0 / 2 | 1 / 3 |
| `vert` / `vrt2` | 4 Vertical Substitution | 0 | 1 |
| `hngl` | 23 Transliteration | 1 `HANJA_TO_HANGUL` | 0 `NO_TRANSLITERATION` |
| `trad` / `smpl` / `jp78` / `jp83` / `jp90` / `jp04` / `expt` / `hojo` / `nlck` / `tnam` | 20 Character Shape | 0 / 1 / 2 / 3 / 4 / 11 / 10 / 12 / 13 / 14 | 16 (no named constant) |
| `titl` | 19 Style Options | 4 `TITLING_CAPS` | 0 `NO_STYLE_OPTIONS` |
| `mgrk` | 15 Mathematical Extras | 10 | 11 |
| `unic` | 3 Letter Case | 14 (no named constant) | 15 (no named constant) |
| `hist` | 40 (no named constant) | 0 | 1 |

Note the several selector values with no `HB_AAT_LAYOUT_FEATURE_SELECTOR_*`
constant: the enumeration in the header covers Apple's registry only partially,
and the mapping table falls back to raw casts. Values coming out of a font's
`feat` table are likewise not limited to the named constants.

## Types

### `hb_aat_layout_feature_type_t`

```c
typedef enum { HB_AAT_LAYOUT_FEATURE_TYPE_INVALID = 0xFFFF, ... } hb_aat_layout_feature_type_t;
```

```rust
pub type hb_aat_layout_feature_type_t = core::ffi::c_int;
```

The possible feature types defined for AAT shaping, from Apple's
[Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html).
A feature type names a family of typographic behaviours a font can offer;
the settings within a type are named by `hb_aat_layout_feature_selector_t`
values.

The C enumeration's private sentinel is `HB_TAG_MAX_SIGNED` (`0x7FFFFFFF`),
which fits an `int`, so the underlying type is `int` and this is transcribed as
`c_int` plus a set of constants. **Values come from font data and are not limited
to the constants below** — a `feat` table may legitimately report type 40 or 100.

Since HarfBuzz 2.2.0.

| Constant (`HB_AAT_LAYOUT_FEATURE_TYPE_` prefix omitted) | Value | Apple registry name |
| --- | ---: | --- |
| `INVALID` | 0xFFFF (65535) | Initial, unset feature type |
| `ALL_TYPOGRAPHIC` | 0 | All Typographic Features |
| `LIGATURES` | 1 | Ligatures |
| `CURSIVE_CONNECTION` | 2 | Cursive Connection |
| `LETTER_CASE` | 3 | Letter Case |
| `VERTICAL_SUBSTITUTION` | 4 | Vertical Substitution |
| `LINGUISTIC_REARRANGEMENT` | 5 | Linguistic Rearrangement |
| `NUMBER_SPACING` | 6 | Number Spacing |
| `SMART_SWASH_TYPE` | 8 | Smart Swash |
| `DIACRITICS_TYPE` | 9 | Diacritics |
| `VERTICAL_POSITION` | 10 | Vertical Position |
| `FRACTIONS` | 11 | Fractions |
| `OVERLAPPING_CHARACTERS_TYPE` | 13 | Overlapping Characters |
| `TYPOGRAPHIC_EXTRAS` | 14 | Typographic Extras |
| `MATHEMATICAL_EXTRAS` | 15 | Mathematical Extras |
| `ORNAMENT_SETS_TYPE` | 16 | Ornament Sets |
| `CHARACTER_ALTERNATIVES` | 17 | Character Alternatives |
| `DESIGN_COMPLEXITY_TYPE` | 18 | Design Complexity |
| `STYLE_OPTIONS` | 19 | Style Options |
| `CHARACTER_SHAPE` | 20 | Character Shape |
| `NUMBER_CASE` | 21 | Number Case |
| `TEXT_SPACING` | 22 | Text Spacing |
| `TRANSLITERATION` | 23 | Transliteration |
| `ANNOTATION_TYPE` | 24 | Annotation |
| `KANA_SPACING_TYPE` | 25 | Kana Spacing |
| `IDEOGRAPHIC_SPACING_TYPE` | 26 | Ideographic Spacing |
| `UNICODE_DECOMPOSITION_TYPE` | 27 | Unicode Decomposition |
| `RUBY_KANA` | 28 | Ruby Kana |
| `CJK_SYMBOL_ALTERNATIVES_TYPE` | 29 | CJK Symbol Alternatives |
| `IDEOGRAPHIC_ALTERNATIVES_TYPE` | 30 | Ideographic Alternatives |
| `CJK_VERTICAL_ROMAN_PLACEMENT_TYPE` | 31 | CJK Vertical Roman Placement |
| `ITALIC_CJK_ROMAN` | 32 | Italic CJK Roman |
| `CASE_SENSITIVE_LAYOUT` | 33 | Case Sensitive Layout |
| `ALTERNATE_KANA` | 34 | Alternate Kana |
| `STYLISTIC_ALTERNATIVES` | 35 | Stylistic Alternatives |
| `CONTEXTUAL_ALTERNATIVES` | 36 | Contextual Alternatives |
| `LOWER_CASE` | 37 | Lower Case |
| `UPPER_CASE` | 38 | Upper Case |
| `LANGUAGE_TAG_TYPE` | 39 | Language Tag |
| `CJK_ROMAN_SPACING_TYPE` | 103 | CJK Roman Spacing |

Note the gaps: 7 and 12 are absent from the registry, as are 40–102. Registry
type 39 (`LANGUAGE_TAG_TYPE`) has no selector constants in the header at all.

### `hb_aat_layout_feature_selector_t`

```c
typedef enum { HB_AAT_LAYOUT_FEATURE_SELECTOR_INVALID = 0xFFFF, ... } hb_aat_layout_feature_selector_t;
```

```rust
pub type hb_aat_layout_feature_selector_t = core::ffi::c_int;
```

The selectors defined for specifying AAT feature settings. **Selector numbers
are only meaningful relative to their feature type**: `0` means "required
ligatures on" under `LIGATURES` and "monospaced numbers" under `NUMBER_SPACING`.
The constants are therefore grouped by the feature type they belong to, and the
same numeric value recurs in every group.

As with feature types, the C enumeration's private sentinel is
`HB_TAG_MAX_SIGNED` (`0x7FFFFFFF`), so the underlying type is `int`, and values
read from font data are not limited to the constants below.

Since HarfBuzz 2.2.0.

Constant names below omit the `HB_AAT_LAYOUT_FEATURE_SELECTOR_` prefix.

**Unset**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `INVALID` | 0xFFFF (65535) | Initial, unset feature selector |

**Type 0 — All Typographic**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `ALL_TYPE_FEATURES_ON` | 0 | Enable every typographic feature the font offers |
| `ALL_TYPE_FEATURES_OFF` | 1 | Disable them all |

**Type 1 — Ligatures**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `REQUIRED_LIGATURES_ON` | 0 | Ligatures the script requires (`rlig`) |
| `REQUIRED_LIGATURES_OFF` | 1 | — |
| `COMMON_LIGATURES_ON` | 2 | Standard ligatures such as fi, fl (`liga`) |
| `COMMON_LIGATURES_OFF` | 3 | — |
| `RARE_LIGATURES_ON` | 4 | Discretionary ligatures (`dlig`) |
| `RARE_LIGATURES_OFF` | 5 | — |
| `LOGOS_ON` | 6 | Logotype ligatures |
| `LOGOS_OFF` | 7 | — |
| `REBUS_PICTURES_ON` | 8 | Rebus picture substitutions |
| `REBUS_PICTURES_OFF` | 9 | — |
| `DIPHTHONG_LIGATURES_ON` | 10 | Æ, Œ and friends |
| `DIPHTHONG_LIGATURES_OFF` | 11 | — |
| `SQUARED_LIGATURES_ON` | 12 | CJK squared forms |
| `SQUARED_LIGATURES_OFF` | 13 | — |
| `ABBREV_SQUARED_LIGATURES_ON` | 14 | Abbreviated squared forms |
| `ABBREV_SQUARED_LIGATURES_OFF` | 15 | — |
| `SYMBOL_LIGATURES_ON` | 16 | Symbol ligatures |
| `SYMBOL_LIGATURES_OFF` | 17 | — |
| `CONTEXTUAL_LIGATURES_ON` | 18 | Context-dependent ligatures (`clig`) |
| `CONTEXTUAL_LIGATURES_OFF` | 19 | — |
| `HISTORICAL_LIGATURES_ON` | 20 | Historical ligatures (`hlig`) |
| `HISTORICAL_LIGATURES_OFF` | 21 | — |

**Type 2 — Cursive Connection**

The C header mislabels this group's comment as belonging to
`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`; the values are Apple's type 2.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `UNCONNECTED` | 0 | No cursive joining |
| `PARTIALLY_CONNECTED` | 1 | Partial joining |
| `CURSIVE` | 2 | Fully cursive |

**Type 3 — Letter Case** (all six selectors are deprecated in the C header)

| Constant | Value | Meaning |
| --- | ---: | --- |
| `UPPER_AND_LOWER_CASE` | 0 | Deprecated. Mixed case |
| `ALL_CAPS` | 1 | Deprecated |
| `ALL_LOWER_CASE` | 2 | Deprecated |
| `SMALL_CAPS` | 3 | Deprecated; use type 37/38 instead |
| `INITIAL_CAPS` | 4 | Deprecated |
| `INITIAL_CAPS_AND_SMALL_CAPS` | 5 | Deprecated |

Selectors 14 and 15 of this type are used by the `unic` OpenType mapping but
have no named constants.

**Type 4 — Vertical Substitution**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `SUBSTITUTE_VERTICAL_FORMS_ON` | 0 | Use vertical glyph variants (`vert`, `vrt2`) |
| `SUBSTITUTE_VERTICAL_FORMS_OFF` | 1 | — |

Selectors 2 and 3 are used by the `vrtr` mapping but have no named constants.

**Type 5 — Linguistic Rearrangement**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `LINGUISTIC_REARRANGEMENT_ON` | 0 | Apply the font's rearrangement rules |
| `LINGUISTIC_REARRANGEMENT_OFF` | 1 | — |

**Type 6 — Number Spacing**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `MONOSPACED_NUMBERS` | 0 | Tabular figures (`tnum`) |
| `PROPORTIONAL_NUMBERS` | 1 | Proportional figures (`pnum`) |
| `THIRD_WIDTH_NUMBERS` | 2 | One-third-em figures |
| `QUARTER_WIDTH_NUMBERS` | 3 | Quarter-em figures |

Selector 4 is the "off" value used by the `pnum`/`tnum` mappings; no constant.

**Type 8 — Smart Swash**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `WORD_INITIAL_SWASHES_ON` | 0 | Swashes at the start of a word |
| `WORD_INITIAL_SWASHES_OFF` | 1 | — |
| `WORD_FINAL_SWASHES_ON` | 2 | Swashes at the end of a word |
| `WORD_FINAL_SWASHES_OFF` | 3 | — |
| `LINE_INITIAL_SWASHES_ON` | 4 | Swashes at the start of a line |
| `LINE_INITIAL_SWASHES_OFF` | 5 | — |
| `LINE_FINAL_SWASHES_ON` | 6 | Swashes at the end of a line |
| `LINE_FINAL_SWASHES_OFF` | 7 | — |
| `NON_FINAL_SWASHES_ON` | 8 | Swashes anywhere but the end |
| `NON_FINAL_SWASHES_OFF` | 9 | — |

**Type 9 — Diacritics**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `SHOW_DIACRITICS` | 0 | Render diacritics normally |
| `HIDE_DIACRITICS` | 1 | Suppress them |
| `DECOMPOSE_DIACRITICS` | 2 | Split precomposed forms into base + mark |

**Type 10 — Vertical Position**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `NORMAL_POSITION` | 0 | Baseline |
| `SUPERIORS` | 1 | Superscript (`sups`) |
| `INFERIORS` | 2 | Subscript (`subs`) |
| `ORDINALS` | 3 | Ordinal forms (`ordn`) |
| `SCIENTIFIC_INFERIORS` | 4 | Scientific inferiors (`sinf`) |

**Type 11 — Fractions**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `NO_FRACTIONS` | 0 | Leave digits and solidus alone |
| `VERTICAL_FRACTIONS` | 1 | Stacked / nut fractions (`afrc`) |
| `DIAGONAL_FRACTIONS` | 2 | Diagonal fractions (`frac`) |

**Type 13 — Overlapping Characters**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `PREVENT_OVERLAP_ON` | 0 | Adjust glyphs so they do not collide |
| `PREVENT_OVERLAP_OFF` | 1 | — |

**Type 14 — Typographic Extras**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HYPHENS_TO_EM_DASH_ON` | 0 | `--` → em dash |
| `HYPHENS_TO_EM_DASH_OFF` | 1 | — |
| `HYPHEN_TO_EN_DASH_ON` | 2 | `-` → en dash |
| `HYPHEN_TO_EN_DASH_OFF` | 3 | — |
| `SLASHED_ZERO_ON` | 4 | Zero with a slash (`zero`) |
| `SLASHED_ZERO_OFF` | 5 | — |
| `FORM_INTERROBANG_ON` | 6 | `?!` → ‽ |
| `FORM_INTERROBANG_OFF` | 7 | — |
| `SMART_QUOTES_ON` | 8 | Straight quotes → curly |
| `SMART_QUOTES_OFF` | 9 | — |
| `PERIODS_TO_ELLIPSIS_ON` | 10 | `...` → … |
| `PERIODS_TO_ELLIPSIS_OFF` | 11 | — |

**Type 15 — Mathematical Extras**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HYPHEN_TO_MINUS_ON` | 0 | `-` → U+2212 |
| `HYPHEN_TO_MINUS_OFF` | 1 | — |
| `ASTERISK_TO_MULTIPLY_ON` | 2 | `*` → × |
| `ASTERISK_TO_MULTIPLY_OFF` | 3 | — |
| `SLASH_TO_DIVIDE_ON` | 4 | `/` → ÷ |
| `SLASH_TO_DIVIDE_OFF` | 5 | — |
| `INEQUALITY_LIGATURES_ON` | 6 | `<=` → ≤, `>=` → ≥ |
| `INEQUALITY_LIGATURES_OFF` | 7 | — |
| `EXPONENTS_ON` | 8 | Raise digits after `^` |
| `EXPONENTS_OFF` | 9 | — |
| `MATHEMATICAL_GREEK_ON` | 10 | Mathematical Greek forms (`mgrk`) |
| `MATHEMATICAL_GREEK_OFF` | 11 | — |

**Type 16 — Ornament Sets**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `NO_ORNAMENTS` | 0 | No ornament substitution |
| `DINGBATS` | 1 | Dingbat set |
| `PI_CHARACTERS` | 2 | Pi character set |
| `FLEURONS` | 3 | Fleuron set |
| `DECORATIVE_BORDERS` | 4 | Border set |
| `INTERNATIONAL_SYMBOLS` | 5 | International symbol set |
| `MATH_SYMBOLS` | 6 | Math symbol set |

**Type 17 — Character Alternatives**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `NO_ALTERNATES` | 0 | The default; higher selectors are font-defined |

This type is open-ended by design — a font declares as many alternates as it
likes, and only the "none" selector has a fixed meaning.

**Type 18 — Design Complexity**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `DESIGN_LEVEL1` | 0 | Simplest design level |
| `DESIGN_LEVEL2` | 1 | — |
| `DESIGN_LEVEL3` | 2 | — |
| `DESIGN_LEVEL4` | 3 | — |
| `DESIGN_LEVEL5` | 4 | Most elaborate |

**Type 19 — Style Options**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `NO_STYLE_OPTIONS` | 0 | Plain |
| `DISPLAY_TEXT` | 1 | Display-size design |
| `ENGRAVED_TEXT` | 2 | Engraved style |
| `ILLUMINATED_CAPS` | 3 | Illuminated capitals |
| `TITLING_CAPS` | 4 | Titling capitals (`titl`) |
| `TALL_CAPS` | 5 | Tall capitals |

**Type 20 — Character Shape**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `TRADITIONAL_CHARACTERS` | 0 | Traditional Chinese forms (`trad`) |
| `SIMPLIFIED_CHARACTERS` | 1 | Simplified forms (`smpl`) |
| `JIS1978_CHARACTERS` | 2 | JIS78 forms (`jp78`) |
| `JIS1983_CHARACTERS` | 3 | JIS83 forms (`jp83`) |
| `JIS1990_CHARACTERS` | 4 | JIS90 forms (`jp90`) |
| `TRADITIONAL_ALT_ONE` | 5 | Traditional alternative 1 |
| `TRADITIONAL_ALT_TWO` | 6 | Traditional alternative 2 |
| `TRADITIONAL_ALT_THREE` | 7 | Traditional alternative 3 |
| `TRADITIONAL_ALT_FOUR` | 8 | Traditional alternative 4 |
| `TRADITIONAL_ALT_FIVE` | 9 | Traditional alternative 5 |
| `EXPERT_CHARACTERS` | 10 | Expert forms (`expt`) |
| `JIS2004_CHARACTERS` | 11 | JIS2004 forms (`jp04`) |
| `HOJO_CHARACTERS` | 12 | Hojo Kanji forms (`hojo`) |
| `NLCCHARACTERS` | 13 | NLC Kanji forms (`nlck`) |
| `TRADITIONAL_NAMES_CHARACTERS` | 14 | Traditional forms for names (`tnam`) |

Selector 16 is the "off" value used by every `CHARACTER_SHAPE` mapping; no
constant.

**Type 21 — Number Case**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `LOWER_CASE_NUMBERS` | 0 | Old-style figures (`onum`) |
| `UPPER_CASE_NUMBERS` | 1 | Lining figures (`lnum`) |

Selector 2 is the "off" value used by those mappings; no constant.

**Type 22 — Text Spacing**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `PROPORTIONAL_TEXT` | 0 | Proportional widths (`pwid`, `pkna`) |
| `MONOSPACED_TEXT` | 1 | Full-width / monospaced (`fwid`) |
| `HALF_WIDTH_TEXT` | 2 | Half-width (`hwid`) |
| `THIRD_WIDTH_TEXT` | 3 | Third-width (`twid`) |
| `QUARTER_WIDTH_TEXT` | 4 | Quarter-width (`qwid`) |
| `ALT_PROPORTIONAL_TEXT` | 5 | Alternate proportional (`palt`, `valt`, `vpal`) |
| `ALT_HALF_WIDTH_TEXT` | 6 | Alternate half-width (`halt`, `vhal`) |

Selector 7 is the "off" value used by every `TEXT_SPACING` mapping; no constant.

**Type 23 — Transliteration**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `NO_TRANSLITERATION` | 0 | Leave the text alone |
| `HANJA_TO_HANGUL` | 1 | Hanja → Hangul (`hngl`) |
| `HIRAGANA_TO_KATAKANA` | 2 | — |
| `KATAKANA_TO_HIRAGANA` | 3 | — |
| `KANA_TO_ROMANIZATION` | 4 | — |
| `ROMANIZATION_TO_HIRAGANA` | 5 | — |
| `ROMANIZATION_TO_KATAKANA` | 6 | — |
| `HANJA_TO_HANGUL_ALT_ONE` | 7 | Alternative reading 1 |
| `HANJA_TO_HANGUL_ALT_TWO` | 8 | Alternative reading 2 |
| `HANJA_TO_HANGUL_ALT_THREE` | 9 | Alternative reading 3 |

**Type 24 — Annotation**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `NO_ANNOTATION` | 0 | Unannotated |
| `BOX_ANNOTATION` | 1 | Boxed |
| `ROUNDED_BOX_ANNOTATION` | 2 | Rounded box |
| `CIRCLE_ANNOTATION` | 3 | Circled |
| `INVERTED_CIRCLE_ANNOTATION` | 4 | Inverted circle |
| `PARENTHESIS_ANNOTATION` | 5 | Parenthesised |
| `PERIOD_ANNOTATION` | 6 | Followed by a period |
| `ROMAN_NUMERAL_ANNOTATION` | 7 | Roman numeral |
| `DIAMOND_ANNOTATION` | 8 | Diamond |
| `INVERTED_BOX_ANNOTATION` | 9 | Inverted box |
| `INVERTED_ROUNDED_BOX_ANNOTATION` | 10 | Inverted rounded box |

**Type 25 — Kana Spacing**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `FULL_WIDTH_KANA` | 0 | Full-width kana |
| `PROPORTIONAL_KANA` | 1 | Proportional kana |

**Type 26 — Ideographic Spacing**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `FULL_WIDTH_IDEOGRAPHS` | 0 | Full-width ideographs |
| `PROPORTIONAL_IDEOGRAPHS` | 1 | Proportional ideographs |
| `HALF_WIDTH_IDEOGRAPHS` | 2 | Half-width ideographs |

**Type 27 — Unicode Decomposition**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `CANONICAL_COMPOSITION_ON` | 0 | Apply canonical composition |
| `CANONICAL_COMPOSITION_OFF` | 1 | — |
| `COMPATIBILITY_COMPOSITION_ON` | 2 | Apply compatibility composition |
| `COMPATIBILITY_COMPOSITION_OFF` | 3 | — |
| `TRANSCODING_COMPOSITION_ON` | 4 | Apply transcoding composition |
| `TRANSCODING_COMPOSITION_OFF` | 5 | — |

**Type 28 — Ruby Kana**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `NO_RUBY_KANA` | 0 | Deprecated; use `RUBY_KANA_OFF` |
| `RUBY_KANA` | 1 | Deprecated; use `RUBY_KANA_ON` |
| `RUBY_KANA_ON` | 2 | Ruby-sized kana (`ruby`) |
| `RUBY_KANA_OFF` | 3 | — |

**Type 29 — CJK Symbol Alternatives**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `NO_CJK_SYMBOL_ALTERNATIVES` | 0 | Default symbols |
| `CJK_SYMBOL_ALT_ONE` | 1 | Alternative set 1 |
| `CJK_SYMBOL_ALT_TWO` | 2 | Alternative set 2 |
| `CJK_SYMBOL_ALT_THREE` | 3 | Alternative set 3 |
| `CJK_SYMBOL_ALT_FOUR` | 4 | Alternative set 4 |
| `CJK_SYMBOL_ALT_FIVE` | 5 | Alternative set 5 |

**Type 30 — Ideographic Alternatives**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `NO_IDEOGRAPHIC_ALTERNATIVES` | 0 | Default ideographs |
| `IDEOGRAPHIC_ALT_ONE` | 1 | Alternative set 1 |
| `IDEOGRAPHIC_ALT_TWO` | 2 | Alternative set 2 |
| `IDEOGRAPHIC_ALT_THREE` | 3 | Alternative set 3 |
| `IDEOGRAPHIC_ALT_FOUR` | 4 | Alternative set 4 |
| `IDEOGRAPHIC_ALT_FIVE` | 5 | Alternative set 5 |

**Type 31 — CJK Vertical Roman Placement**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `CJK_VERTICAL_ROMAN_CENTERED` | 0 | Centre Roman glyphs in the vertical em box |
| `CJK_VERTICAL_ROMAN_HBASELINE` | 1 | Align them to the horizontal baseline |

**Type 32 — Italic CJK Roman**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `NO_CJK_ITALIC_ROMAN` | 0 | Deprecated; use `CJK_ITALIC_ROMAN_OFF` |
| `CJK_ITALIC_ROMAN` | 1 | Deprecated; use `CJK_ITALIC_ROMAN_ON` |
| `CJK_ITALIC_ROMAN_ON` | 2 | Italic Roman inside CJK text (`ital`) |
| `CJK_ITALIC_ROMAN_OFF` | 3 | — |

**Type 33 — Case Sensitive Layout**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `CASE_SENSITIVE_LAYOUT_ON` | 0 | Case-sensitive forms (`case`) |
| `CASE_SENSITIVE_LAYOUT_OFF` | 1 | — |
| `CASE_SENSITIVE_SPACING_ON` | 2 | Capital spacing (`cpsp`) |
| `CASE_SENSITIVE_SPACING_OFF` | 3 | — |

**Type 34 — Alternate Kana**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `ALTERNATE_HORIZ_KANA_ON` | 0 | Horizontal kana alternates (`hkna`) |
| `ALTERNATE_HORIZ_KANA_OFF` | 1 | — |
| `ALTERNATE_VERT_KANA_ON` | 2 | Vertical kana alternates (`vkna`) |
| `ALTERNATE_VERT_KANA_OFF` | 3 | — |

**Type 35 — Stylistic Alternatives**

Twenty on/off pairs, matching OpenType `ss01`…`ss20`. Note that selector 1 is
unused — the pairs start at 2.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `NO_STYLISTIC_ALTERNATES` | 0 | No stylistic set applied |
| `STYLISTIC_ALT_ONE_ON` | 2 | Stylistic set 1 (`ss01`) |
| `STYLISTIC_ALT_ONE_OFF` | 3 | — |
| `STYLISTIC_ALT_TWO_ON` | 4 | Stylistic set 2 (`ss02`) |
| `STYLISTIC_ALT_TWO_OFF` | 5 | — |
| `STYLISTIC_ALT_THREE_ON` | 6 | Stylistic set 3 (`ss03`) |
| `STYLISTIC_ALT_THREE_OFF` | 7 | — |
| `STYLISTIC_ALT_FOUR_ON` | 8 | Stylistic set 4 (`ss04`) |
| `STYLISTIC_ALT_FOUR_OFF` | 9 | — |
| `STYLISTIC_ALT_FIVE_ON` | 10 | Stylistic set 5 (`ss05`) |
| `STYLISTIC_ALT_FIVE_OFF` | 11 | — |
| `STYLISTIC_ALT_SIX_ON` | 12 | Stylistic set 6 (`ss06`) |
| `STYLISTIC_ALT_SIX_OFF` | 13 | — |
| `STYLISTIC_ALT_SEVEN_ON` | 14 | Stylistic set 7 (`ss07`) |
| `STYLISTIC_ALT_SEVEN_OFF` | 15 | — |
| `STYLISTIC_ALT_EIGHT_ON` | 16 | Stylistic set 8 (`ss08`) |
| `STYLISTIC_ALT_EIGHT_OFF` | 17 | — |
| `STYLISTIC_ALT_NINE_ON` | 18 | Stylistic set 9 (`ss09`) |
| `STYLISTIC_ALT_NINE_OFF` | 19 | — |
| `STYLISTIC_ALT_TEN_ON` | 20 | Stylistic set 10 (`ss10`) |
| `STYLISTIC_ALT_TEN_OFF` | 21 | — |
| `STYLISTIC_ALT_ELEVEN_ON` | 22 | Stylistic set 11 (`ss11`) |
| `STYLISTIC_ALT_ELEVEN_OFF` | 23 | — |
| `STYLISTIC_ALT_TWELVE_ON` | 24 | Stylistic set 12 (`ss12`) |
| `STYLISTIC_ALT_TWELVE_OFF` | 25 | — |
| `STYLISTIC_ALT_THIRTEEN_ON` | 26 | Stylistic set 13 (`ss13`) |
| `STYLISTIC_ALT_THIRTEEN_OFF` | 27 | — |
| `STYLISTIC_ALT_FOURTEEN_ON` | 28 | Stylistic set 14 (`ss14`) |
| `STYLISTIC_ALT_FOURTEEN_OFF` | 29 | — |
| `STYLISTIC_ALT_FIFTEEN_ON` | 30 | Stylistic set 15 (`ss15`) |
| `STYLISTIC_ALT_FIFTEEN_OFF` | 31 | — |
| `STYLISTIC_ALT_SIXTEEN_ON` | 32 | Stylistic set 16 (`ss16`) |
| `STYLISTIC_ALT_SIXTEEN_OFF` | 33 | — |
| `STYLISTIC_ALT_SEVENTEEN_ON` | 34 | Stylistic set 17 (`ss17`) |
| `STYLISTIC_ALT_SEVENTEEN_OFF` | 35 | — |
| `STYLISTIC_ALT_EIGHTEEN_ON` | 36 | Stylistic set 18 (`ss18`) |
| `STYLISTIC_ALT_EIGHTEEN_OFF` | 37 | — |
| `STYLISTIC_ALT_NINETEEN_ON` | 38 | Stylistic set 19 (`ss19`) |
| `STYLISTIC_ALT_NINETEEN_OFF` | 39 | — |
| `STYLISTIC_ALT_TWENTY_ON` | 40 | Stylistic set 20 (`ss20`) |
| `STYLISTIC_ALT_TWENTY_OFF` | 41 | — |

**Type 36 — Contextual Alternatives**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `CONTEXTUAL_ALTERNATES_ON` | 0 | Contextual alternates (`calt`) |
| `CONTEXTUAL_ALTERNATES_OFF` | 1 | — |
| `SWASH_ALTERNATES_ON` | 2 | Swash alternates (`swsh`) |
| `SWASH_ALTERNATES_OFF` | 3 | — |
| `CONTEXTUAL_SWASH_ALTERNATES_ON` | 4 | Contextual swashes (`cswh`) |
| `CONTEXTUAL_SWASH_ALTERNATES_OFF` | 5 | — |

**Type 37 — Lower Case**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `DEFAULT_LOWER_CASE` | 0 | Ordinary lowercase |
| `LOWER_CASE_SMALL_CAPS` | 1 | Small caps (`smcp`) |
| `LOWER_CASE_PETITE_CAPS` | 2 | Petite caps (`pcap`) |

**Type 38 — Upper Case**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `DEFAULT_UPPER_CASE` | 0 | Ordinary uppercase |
| `UPPER_CASE_SMALL_CAPS` | 1 | Caps to small caps (`c2sc`) |
| `UPPER_CASE_PETITE_CAPS` | 2 | Caps to petite caps (`c2pc`) |

**Type 103 — CJK Roman Spacing**

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HALF_WIDTH_CJK_ROMAN` | 0 | Half-width Roman in CJK text |
| `PROPORTIONAL_CJK_ROMAN` | 1 | Proportional Roman |
| `DEFAULT_CJK_ROMAN` | 2 | The font's default |
| `FULL_WIDTH_CJK_ROMAN` | 3 | Full-width Roman |

### `hb_aat_layout_feature_selector_info_t`

```c
typedef struct hb_aat_layout_feature_selector_info_t {
  hb_ot_name_id_t                  name_id;
  hb_aat_layout_feature_selector_t enable;
  hb_aat_layout_feature_selector_t disable;
  /*< private >*/
  unsigned int                     reserved;
} hb_aat_layout_feature_selector_info_t;
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_aat_layout_feature_selector_info_t {
    pub name_id: hb_ot_name_id_t,
    pub enable: hb_aat_layout_feature_selector_t,
    pub disable: hb_aat_layout_feature_selector_t,
    pub reserved: c_uint,
}
```

A structure representing one setting of an `hb_aat_layout_feature_type_t`. You
never construct these; `hb_aat_layout_feature_type_get_selector_infos()` fills an
array of them from the face's `feat` table.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `name_id` | `hb_ot_name_id_t` | `c_uint` | The selector's name identifier, for lookup in the face's `name` table with `hb_ot_name_get_utf8()` etc. |
| `enable` | `hb_aat_layout_feature_selector_t` | `c_int` | The selector value that turns this setting **on**. |
| `disable` | `hb_aat_layout_feature_selector_t` | `c_int` | The selector value that turns this setting **off**. For a *non-exclusive* feature type this is `enable + 1`, following AAT's on/off-pair convention. For an *exclusive* feature type there is no "off" — turning one setting off means selecting another — so HarfBuzz reports the feature's default selector here instead. |
| `reserved` | `unsigned int` | `c_uint` | Private padding, marked `/*< private >*/` in the header. HarfBuzz sets it to zero. Do not read or write it. |

The header carries no `Since:` annotation on this struct; it arrived with the
rest of the `feat` API in HarfBuzz 2.2.0.

### `hb_face_t`, `hb_ot_name_id_t`, `hb_bool_t`

Ordinary HarfBuzz core types from `hb-face.h`, `hb-ot-name.h`, and
`hb-common.h`. `hb_ot_name_id_t` is `c_uint` in this crate; `hb_bool_t` is
`c_int` where non-zero means true.

## Constants

### `HB_AAT_LAYOUT_NO_SELECTOR_INDEX`

```c
#define HB_AAT_LAYOUT_NO_SELECTOR_INDEX 0xFFFFu
```

```rust
pub const HB_AAT_LAYOUT_NO_SELECTOR_INDEX: c_uint = 0xFFFF;
```

Used when getting or setting AAT feature selectors. Indicates that there is no
selector index corresponding to the selector of interest — value 65535.

In practice you meet it as the `default_index` out-parameter of
`hb_aat_layout_feature_type_get_selector_infos()`: when the function sets
`default_index` to this value, the feature type is **non-exclusive** (its
settings are independent on/off switches) and therefore has no single default
selector. Any other value is an index into the returned selector array.

Note that this is an *index* sentinel, distinct from
`HB_AAT_LAYOUT_FEATURE_SELECTOR_INVALID`, which is a *selector value* sentinel
that happens to share the same number.

The header's documentation block carries no `Since:` annotation.

## Functions

### Querying the `feat` table

#### `hb_aat_layout_get_feature_types`

```c
unsigned int hb_aat_layout_get_feature_types (hb_face_t                    *face,
                                              unsigned int                  start_offset,
                                              unsigned int                 *feature_count, /* IN/OUT. May be NULL. */
                                              hb_aat_layout_feature_type_t *features       /* OUT.    May be NULL. */);
```

```rust
pub fn hb_aat_layout_get_feature_types(
    face: *mut hb_face_t,
    start_offset: c_uint,
    feature_count: *mut c_uint,
    features: *mut hb_aat_layout_feature_type_t,
) -> c_uint;
```

Fetches a list of the AAT feature types included in the specified face — that is,
the entries of its `feat` table.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `face` | The face to work upon. | The implementation dereferences `face->table.feat` immediately; treat null as forbidden. A face with no `feat` table is fine — you get a total of 0. |
| `start_offset` | Index of the first feature type to retrieve. | Any value; an offset at or past the total yields zero entries. |
| `feature_count` | In: the maximum number of feature types to return. Out: how many were actually written, which may be zero. | Optional; may be null, in which case nothing is written. |
| `features` | Caller-allocated array of at least `*feature_count` entries. | Optional; may be null. Only meaningful when `feature_count` is non-null. |

**Returns** — the number of **all** available feature types, independent of
`start_offset` and `feature_count`. This is the standard HarfBuzz array-getter
convention: call once with null out-parameters to size your buffer, then again to
fill it.

**Ownership** — nothing is allocated; the caller owns `features`.

**Notes** — Since HarfBuzz 2.2.0. Reads only, so it is safe to call concurrently
on a face that is not being mutated. The returned values are raw `feat` table
data and may include types not covered by the named constants.

#### `hb_aat_layout_feature_type_get_name_id`

```c
hb_ot_name_id_t hb_aat_layout_feature_type_get_name_id (hb_face_t                    *face,
                                                        hb_aat_layout_feature_type_t  feature_type);
```

```rust
pub fn hb_aat_layout_feature_type_get_name_id(
    face: *mut hb_face_t,
    feature_type: hb_aat_layout_feature_type_t,
) -> hb_ot_name_id_t;
```

Fetches the name identifier of the specified feature type in the face's `name`
table — the string a font panel would show as the group heading ("Ligatures",
"Number Spacing", …).

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `face` | The face to work upon. | Treat null as forbidden. |
| `feature_type` | The feature type to look up. | Any value; a type the face does not declare yields the invalid name id. |

**Returns** — the name identifier, or `HB_OT_NAME_ID_INVALID` (0xFFFF) when the
face has no `feat` table or does not declare that feature type. Feed the id to
the `hb-ot-name.h` API (`hb_ot_name_get_utf8`, `hb_ot_name_get_utf16`) together
with a language to get the actual string.

**Ownership** — nothing is allocated.

**Notes** — Since HarfBuzz 2.2.0. Read-only.

#### `hb_aat_layout_feature_type_get_selector_infos`

```c
unsigned int hb_aat_layout_feature_type_get_selector_infos (hb_face_t                             *face,
                                                            hb_aat_layout_feature_type_t           feature_type,
                                                            unsigned int                           start_offset,
                                                            unsigned int                          *selector_count, /* IN/OUT. May be NULL. */
                                                            hb_aat_layout_feature_selector_info_t *selectors,      /* OUT.    May be NULL. */
                                                            unsigned int                          *default_index   /* OUT.    May be NULL. */);
```

```rust
pub fn hb_aat_layout_feature_type_get_selector_infos(
    face: *mut hb_face_t,
    feature_type: hb_aat_layout_feature_type_t,
    start_offset: c_uint,
    selector_count: *mut c_uint,
    selectors: *mut hb_aat_layout_feature_selector_info_t,
    default_index: *mut c_uint,
) -> c_uint;
```

Fetches a list of the selectors available for the specified feature type in the
given face.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `face` | The face to work upon. | Treat null as forbidden. |
| `feature_type` | The feature type whose settings you want. | Any value; an undeclared type yields a total of 0. |
| `start_offset` | Index of the first selector to retrieve. | Any value. |
| `selector_count` | In: the maximum number of selectors to return. Out: how many were actually written, which may be zero. | Optional; may be null. |
| `selectors` | Caller-allocated array of at least `*selector_count` `hb_aat_layout_feature_selector_info_t`. | Optional; may be null. Only meaningful when `selector_count` is non-null. |
| `default_index` | Receives the index of the feature's default selector. | Optional; may be null. Set to `HB_AAT_LAYOUT_NO_SELECTOR_INDEX` when the feature type is non-exclusive. |

**Returns** — the number of **all** available feature selectors for that type,
independent of `start_offset` and `selector_count`.

**Ownership** — nothing is allocated; the caller owns `selectors`.

**Notes** — Since HarfBuzz 2.2.0. Read-only.

The exclusive/non-exclusive distinction is the important one here, and it is the
same distinction that drives the `disable` field of each returned record:

- **Exclusive** feature type (a radio group — Number Spacing, Character Shape):
  exactly one selector is active at a time. `default_index` names it, and each
  record's `disable` is that same default.
- **Non-exclusive** feature type (a set of checkboxes — Ligatures, Typographic
  Extras): each setting toggles independently. `default_index` is
  `HB_AAT_LAYOUT_NO_SELECTOR_INDEX`, and each record's `disable` is
  `enable + 1`.

### Testing what a face carries

#### `hb_aat_layout_has_substitution`

```c
hb_bool_t hb_aat_layout_has_substitution (hb_face_t *face);
```

```rust
pub fn hb_aat_layout_has_substitution(face: *mut hb_face_t) -> hb_bool_t;
```

Tests whether the specified face includes any substitutions in the `morx` or
`mort` tables. The implementation is exactly
`face->table.morx->table->has_data() || face->table.mort->table->has_data()`.

**Parameters** — `face`: the face to work upon. Treat null as forbidden.

**Returns** — `true` if data was found, `false` otherwise.

**Ownership** — nothing is allocated. Note that the first call forces the lazy
table loader to sanitize and load `morx`/`mort`, so it is not free; subsequent
calls read a cached accelerator.

**Notes** — Since HarfBuzz 2.3.0. **Does not examine the `GSUB` table** — this
is purely about AAT. Pair it with `hb_ot_layout_has_substitution()` if you want
to know which engine will actually run; see the `_hb_apply_morx` rule in the
overview.

#### `hb_aat_layout_has_positioning`

```c
hb_bool_t hb_aat_layout_has_positioning (hb_face_t *face);
```

```rust
pub fn hb_aat_layout_has_positioning(face: *mut hb_face_t) -> hb_bool_t;
```

Tests whether the specified face includes any positioning information in the
`kerx` table. The implementation is `face->table.kerx->table->has_data()`.

**Parameters** — `face`: the face to work upon. Treat null as forbidden.

**Returns** — `true` if data was found, `false` otherwise.

**Ownership** — nothing is allocated; same lazy-load remark as above.

**Notes** — Since HarfBuzz 2.3.0. **Does not examine the `GPOS` table.** Note
also that it does not look at the legacy `kern` table — for that, use
`hb_ot_layout_has_kerning()`.

#### `hb_aat_layout_has_tracking`

```c
hb_bool_t hb_aat_layout_has_tracking (hb_face_t *face);
```

```rust
pub fn hb_aat_layout_has_tracking(face: *mut hb_face_t) -> hb_bool_t;
```

Tests whether the specified face includes any tracking information in the `trak`
table. The implementation is `face->table.trak->has_data()`.

**Parameters** — `face`: the face to work upon. Treat null as forbidden.

**Returns** — `true` if data was found, `false` otherwise.

**Ownership** — nothing is allocated.

**Notes** — Since HarfBuzz 2.3.0. A `true` here does **not** mean tracking will
be applied: HarfBuzz additionally requires the face to have a `STAT` table
("modern font", per Apple's heuristic) before it enables `trak` during shaping,
and disables `trak` entirely when built with `HB_NO_STYLE`.

## Usage

### C: dump a font's AAT feature panel

The two-pass array-getter pattern, applied twice — once for feature types, once
for each type's selectors.

```c
#include <hb.h>
#include <hb-aat.h>
#include <stdio.h>

static void
print_name (hb_face_t *face, hb_ot_name_id_t id)
{
  char buf[128];
  unsigned int len = sizeof buf;
  if (id == HB_OT_NAME_ID_INVALID) { printf ("(unnamed)"); return; }
  hb_ot_name_get_utf8 (face, id, HB_LANGUAGE_INVALID, &len, buf);
  printf ("%s", buf);
}

void dump_aat_features (hb_face_t *face)
{
  unsigned int n_types = hb_aat_layout_get_feature_types (face, 0, NULL, NULL);
  if (!n_types) { printf ("no `feat` table\n"); return; }

  hb_aat_layout_feature_type_t *types =
    malloc (n_types * sizeof *types);
  unsigned int count = n_types;
  hb_aat_layout_get_feature_types (face, 0, &count, types);

  for (unsigned int i = 0; i < count; i++)
  {
    hb_aat_layout_feature_type_t t = types[i];

    printf ("feature type %d: ", (int) t);
    print_name (face, hb_aat_layout_feature_type_get_name_id (face, t));

    unsigned int n_sel =
      hb_aat_layout_feature_type_get_selector_infos (face, t, 0, NULL, NULL, NULL);

    hb_aat_layout_feature_selector_info_t *sels =
      malloc (n_sel * sizeof *sels);
    unsigned int scount = n_sel, def_index = 0;
    hb_aat_layout_feature_type_get_selector_infos (face, t, 0, &scount, sels, &def_index);

    printf (" (%u settings, %s)\n", scount,
            def_index == HB_AAT_LAYOUT_NO_SELECTOR_INDEX ? "non-exclusive"
                                                         : "exclusive");

    for (unsigned int j = 0; j < scount; j++)
    {
      printf ("    on=%d off=%d  ", (int) sels[j].enable, (int) sels[j].disable);
      print_name (face, sels[j].name_id);
      if (def_index != HB_AAT_LAYOUT_NO_SELECTOR_INDEX && j == def_index)
        printf ("  [default]");
      printf ("\n");
    }
    free (sels);
  }
  free (types);
}
```

### C: decide which engine will shape a face

```c
hb_bool_t aat_sub = hb_aat_layout_has_substitution (face);
hb_bool_t ot_sub  = hb_ot_layout_has_substitution (face);

/* This mirrors HarfBuzz's own _hb_apply_morx() for a horizontal run. */
hb_bool_t will_use_morx = aat_sub && (is_horizontal || !ot_sub);

hb_bool_t aat_pos = hb_aat_layout_has_positioning (face);
hb_bool_t ot_pos  = hb_ot_layout_has_positioning (face);

/* kerx wins unless the face has both GSUB and GPOS. */
hb_bool_t will_use_kerx = aat_pos && !(ot_sub && ot_pos);
```

### Rust: enumerate feature types

```rust
use core::ffi::c_uint;
use harfbuzz_sys::{
    hb_aat_layout_feature_type_t, hb_aat_layout_get_feature_types, hb_face_t,
};

/// Collect every AAT feature type declared by `face`.
///
/// # Safety
/// `face` must be a live, non-null `hb_face_t`.
unsafe fn aat_feature_types(face: *mut hb_face_t) -> Vec<hb_aat_layout_feature_type_t> {
    // SAFETY: passing null for both out-parameters is explicitly allowed and
    // only queries the total.
    let total = unsafe {
        hb_aat_layout_get_feature_types(face, 0, core::ptr::null_mut(), core::ptr::null_mut())
    } as usize;

    let mut out = vec![0 as hb_aat_layout_feature_type_t; total];
    let mut count = total as c_uint;

    // SAFETY: `out` has room for `count` entries, which is what we promise via
    // the in/out `count` parameter.
    unsafe { hb_aat_layout_get_feature_types(face, 0, &mut count, out.as_mut_ptr()) };

    out.truncate(count as usize);
    out
}
```

### Rust: read one feature type's settings

```rust
use core::ffi::c_uint;
use harfbuzz_sys::{
    hb_aat_layout_feature_selector_info_t, hb_aat_layout_feature_type_get_selector_infos,
    hb_aat_layout_feature_type_t, hb_face_t, HB_AAT_LAYOUT_NO_SELECTOR_INDEX,
};

pub struct Settings {
    pub selectors: Vec<hb_aat_layout_feature_selector_info_t>,
    /// `None` when the feature type is non-exclusive (independent on/off
    /// switches); `Some(index)` into `selectors` when it is exclusive.
    pub default_index: Option<usize>,
}

/// # Safety
/// `face` must be a live, non-null `hb_face_t`.
unsafe fn selector_infos(
    face: *mut hb_face_t,
    feature_type: hb_aat_layout_feature_type_t,
) -> Settings {
    // SAFETY: null out-parameters query the total only.
    let total = unsafe {
        hb_aat_layout_feature_type_get_selector_infos(
            face,
            feature_type,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )
    } as usize;

    let mut selectors = vec![
        hb_aat_layout_feature_selector_info_t {
            name_id: 0,
            enable: 0,
            disable: 0,
            reserved: 0,
        };
        total
    ];
    let mut count = total as c_uint;
    let mut default_index: c_uint = 0;

    // SAFETY: `selectors` has room for `count` entries.
    unsafe {
        hb_aat_layout_feature_type_get_selector_infos(
            face,
            feature_type,
            0,
            &mut count,
            selectors.as_mut_ptr(),
            &mut default_index,
        )
    };

    selectors.truncate(count as usize);

    Settings {
        selectors,
        default_index: (default_index != HB_AAT_LAYOUT_NO_SELECTOR_INDEX)
            .then_some(default_index as usize),
    }
}
```

### Rust: the three predicates

```rust
use harfbuzz_sys::{
    hb_aat_layout_has_positioning, hb_aat_layout_has_substitution, hb_aat_layout_has_tracking,
    hb_face_t,
};

/// # Safety
/// `face` must be a live, non-null `hb_face_t`.
unsafe fn aat_capabilities(face: *mut hb_face_t) -> (bool, bool, bool) {
    // SAFETY: `face` is live by the caller's contract; these calls only read
    // (and lazily load) the face's AAT tables.
    unsafe {
        (
            hb_aat_layout_has_substitution(face) != 0, // morx / mort
            hb_aat_layout_has_positioning(face) != 0,  // kerx
            hb_aat_layout_has_tracking(face) != 0,     // trak
        )
    }
}
```

## Pitfalls

- **Selector values are only meaningful with their feature type.** `0` is
  "required ligatures on" under type 1 and "monospaced numbers" under type 6.
  Never store or compare a bare selector without its type. The Rust constants
  are all the same `c_int` alias, so the compiler will not catch a mix-up.

- **The named constants are not exhaustive.** Several selectors used by
  HarfBuzz's own OpenType→AAT mapping table (`TEXT_SPACING` 7,
  `CHARACTER_SHAPE` 16, `NUMBER_CASE` 2, `NUMBER_SPACING` 4, `LETTER_CASE` 14 and
  15, feature type 40 for `hist`) have no constant at all. Font data can contain
  anything. Do not write an exhaustive `match`; treat both enumerations as open.

- **`disable` means two different things.** On a non-exclusive feature type it
  really is the "off" selector (`enable + 1`). On an exclusive one there is no
  "off", so HarfBuzz puts the feature's *default* selector there. Check
  `default_index` before interpreting it.

- **`HB_AAT_LAYOUT_NO_SELECTOR_INDEX` is an index, not a selector.** It shares
  the number 0xFFFF with `HB_AAT_LAYOUT_FEATURE_SELECTOR_INVALID`, but they
  appear in different places and mean different things.

- **The C header mislabels the Cursive Connection selector group** as belonging
  to `HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`. The values (`UNCONNECTED`,
  `PARTIALLY_CONNECTED`, `CURSIVE`) are Apple's type 2, not type 1.

- **`has_substitution` says nothing about `GSUB`, and vice versa.** All three
  predicates are AAT-only by design. A font can have both; which one runs is the
  `_hb_apply_morx` rule, not "whichever exists".

- **AAT wins over OpenType for horizontal text.** If you ship a font with both
  `morx` and `GSUB`, horizontal runs shape through `morx`. That is often a
  surprise when a font was authored assuming `GSUB` would be used. There is no
  API to prefer `GSUB`; the choice is made inside the shape plan.

- **Script-specific shaping is disabled under `morx`.** When AAT substitution
  applies, HarfBuzz downgrades to its `dumber` shaper for non-default scripts.
  If your Indic or Arabic font relies on HarfBuzz's reordering, adding a `morx`
  table will silently turn that reordering off.

- **`hb_aat_layout_has_tracking` is necessary but not sufficient.** Tracking is
  only applied when the face also has a `STAT` table.

- **Only shaping tables are supported.** `bsln`, `just`, and `opbd` are parsed
  into HarfBuzz's source tree but not used; there is no API for baselines,
  justification, or optical bounds from AAT.

- **These functions lazily load tables.** The first `hb_aat_layout_has_*` call on
  a face pays for sanitizing the corresponding table. If you are probing many
  faces just to classify them, that cost is real.
