# OpenType layout

Reference for `hb-ot-layout.h`, transcribed in `harfbuzz-sys` as the
`ot_layout` module (re-exported at the crate root).

## Overview

This header is HarfBuzz's **introspection** API for the four OpenType Layout
tables: `GDEF` (Glyph Definition), `GSUB` (Glyph Substitution), `GPOS` (Glyph
Positioning), and `BASE` (Baseline). It answers questions about what a face's
layout tables *contain* — which scripts they cover, which features are declared
under those scripts, which lookups those features drive, which glyphs those
lookups touch. It does **not** shape text. Shaping is `hb_shape()` in
`hb-shape.h`, which walks the same tables internally and needs none of these
functions. Reach for this header when you are writing a font inspector, a
subsetter, a feature-picker UI, a test harness, or a shaping-engine
reimplementation — not when you simply want glyphs out of a string.

**The GSUB/GPOS query model.** Both tables share an identical four-level
structure, so almost every function here takes a `table_tag` that must be
either `HB_OT_TAG_GSUB` or `HB_OT_TAG_GPOS` and then addresses that structure by
index:

```
table  →  script (e.g. `latn`)  →  language system (e.g. `TRK `)  →  feature (e.g. `liga`)  →  lookups
```

You descend it in that order. `hb_ot_layout_table_select_script()` turns a
script tag into a *script index*; `hb_ot_layout_script_select_language()` turns
that index plus a language tag into a *language index*; the
`hb_ot_layout_language_get_feature_*()` family turns that pair into *feature
indices* or tags; and `hb_ot_layout_feature_get_lookups()` turns a feature index
into *lookup indices*. Every index is an `unsigned int` that is meaningful only
for the one table it came from — a `GSUB` script index is not a `GPOS` script
index, and neither survives a change of face. Three sentinel values mark
"nothing here": `HB_OT_LAYOUT_NO_SCRIPT_INDEX`, `HB_OT_LAYOUT_NO_FEATURE_INDEX`,
and `HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX` (all `0xFFFF`), plus
`HB_OT_LAYOUT_NO_VARIATIONS_INDEX` (`0xFFFFFFFF`) for the feature-variations
axis.

**The array-fetching convention.** The enumerating functions all share one
signature shape: `(…, unsigned int start_offset, unsigned int *count /* IN/OUT
*/, T *array /* OUT */) -> unsigned int`. On entry `*count` is the capacity of
`array`; on return it is how many elements were actually written, starting from
the element at `start_offset`. The **return value is the grand total**
available, which is generally larger than `*count`. Both `count` and `array` may
be null, so passing `(0, NULL, NULL)` is the idiomatic way to ask "how many are
there?" without allocating. This is the same convention used by
`hb_face_get_table_tags()` and `hb_ot_var_get_axis_infos()`.

**Everything here is a read.** No function in this header mutates a face or a
font; the mutating outputs are the caller's own `hb_set_t`, `hb_map_t`, or plain
arrays. All of them are safe to call concurrently on a shared face, subject to
HarfBuzz's usual rule that the *destination* objects you pass in must not be
shared between threads. There are no objects to create or destroy: this header
declares no opaque types and no reference-counted handles at all, only two
enumerations, twelve constants, and forty-eight functions that take faces and
fonts you obtained elsewhere.

**Tag conversion.** Four functions bridge HarfBuzz's `hb_script_t` /
`hb_language_t` vocabulary and OpenType's four-byte tag vocabulary. They are the
translation layer you need before you can call anything in the GSUB/GPOS query
family, because that family speaks only tags. The mapping is not one-to-one in
either direction — one script can map to up to `HB_OT_MAX_TAGS_PER_SCRIPT`
(three) OpenType tags, in preference order — which is why the modern function,
`hb_ot_tags_from_script_and_language()`, returns arrays.

## Types

### `hb_ot_layout_glyph_class_t`

```c
typedef enum {
  HB_OT_LAYOUT_GLYPH_CLASS_UNCLASSIFIED = 0,
  HB_OT_LAYOUT_GLYPH_CLASS_BASE_GLYPH   = 1,
  HB_OT_LAYOUT_GLYPH_CLASS_LIGATURE     = 2,
  HB_OT_LAYOUT_GLYPH_CLASS_MARK         = 3,
  HB_OT_LAYOUT_GLYPH_CLASS_COMPONENT    = 4
} hb_ot_layout_glyph_class_t;
```

```rust
pub type hb_ot_layout_glyph_class_t = c_int;
```

The GDEF classes defined for glyphs — the `GlyphClassDef` values of the `GDEF`
table. A glyph's class determines how the shaper treats it: marks attach to
bases, ligatures carry caret positions, components are skipped for mark
attachment, and so on.

| Constant | Value | Meaning |
| --- | --- | --- |
| `HB_OT_LAYOUT_GLYPH_CLASS_UNCLASSIFIED` | 0 | Glyphs not matching the other classifications. Also what you get back for every glyph when the face has no `GDEF` glyph-class table at all. |
| `HB_OT_LAYOUT_GLYPH_CLASS_BASE_GLYPH` | 1 | Spacing, single characters, capable of accepting marks. |
| `HB_OT_LAYOUT_GLYPH_CLASS_LIGATURE` | 2 | Glyphs that represent the ligation of multiple characters. |
| `HB_OT_LAYOUT_GLYPH_CLASS_MARK` | 3 | Non-spacing, combining glyphs that represent marks. |
| `HB_OT_LAYOUT_GLYPH_CLASS_COMPONENT` | 4 | Spacing glyphs that represent part of a single character. |

The C enumeration has no explicit sentinel and its largest enumerator is 4, so
it fits in an `int`; the Rust transcription is `c_int` plus constants rather
than a Rust `enum`, because the value comes from font data and a Rust `enum`
holding an out-of-range discriminant is undefined behaviour.

### `hb_ot_layout_baseline_tag_t`

```c
typedef enum {
  HB_OT_LAYOUT_BASELINE_TAG_ROMAN                       = HB_TAG ('r','o','m','n'),
  HB_OT_LAYOUT_BASELINE_TAG_HANGING                     = HB_TAG ('h','a','n','g'),
  HB_OT_LAYOUT_BASELINE_TAG_IDEO_FACE_BOTTOM_OR_LEFT    = HB_TAG ('i','c','f','b'),
  HB_OT_LAYOUT_BASELINE_TAG_IDEO_FACE_TOP_OR_RIGHT      = HB_TAG ('i','c','f','t'),
  HB_OT_LAYOUT_BASELINE_TAG_IDEO_FACE_CENTRAL           = HB_TAG ('I','c','f','c'),
  HB_OT_LAYOUT_BASELINE_TAG_IDEO_EMBOX_BOTTOM_OR_LEFT   = HB_TAG ('i','d','e','o'),
  HB_OT_LAYOUT_BASELINE_TAG_IDEO_EMBOX_TOP_OR_RIGHT     = HB_TAG ('i','d','t','p'),
  HB_OT_LAYOUT_BASELINE_TAG_IDEO_EMBOX_CENTRAL          = HB_TAG ('I','d','c','e'),
  HB_OT_LAYOUT_BASELINE_TAG_MATH                        = HB_TAG ('m','a','t','h'),

  /*< private >*/
  _HB_OT_LAYOUT_BASELINE_TAG_MAX_VALUE = HB_TAG_MAX_SIGNED /*< skip >*/
} hb_ot_layout_baseline_tag_t;
```

```rust
pub type hb_ot_layout_baseline_tag_t = c_int;
```

Baseline tags from the OpenType
[Baseline Tags](https://docs.microsoft.com/en-us/typography/opentype/spec/baselinetags)
registry. Each value *is* a four-byte tag, so it can be printed with
`hb_tag_to_string()` and round-trips through `HB_TAG`/`HB_UNTAG`. Since HarfBuzz
2.6.0.

| Constant | Tag | Hex | Decimal | Meaning |
| --- | --- | --- | --- | --- |
| `HB_OT_LAYOUT_BASELINE_TAG_ROMAN` | `romn` | `0x726F6D6E` | 1919905134 | The baseline used by alphabetic scripts such as Latin, Cyrillic and Greek. In vertical writing mode, the alphabetic baseline for characters rotated 90° clockwise — it does not apply to alphabetic characters that stay upright, since those are not rotated. |
| `HB_OT_LAYOUT_BASELINE_TAG_HANGING` | `hang` | `0x68616E67` | 1751215719 | The hanging baseline. Horizontally, the line from which syllables seem to hang in Tibetan and similar scripts; vertically, the same for such characters rotated 90° clockwise. |
| `HB_OT_LAYOUT_BASELINE_TAG_IDEO_FACE_BOTTOM_OR_LEFT` | `icfb` | `0x69636662` | 1768121954 | Ideographic character face bottom edge (horizontal) or left edge (vertical). |
| `HB_OT_LAYOUT_BASELINE_TAG_IDEO_FACE_TOP_OR_RIGHT` | `icft` | `0x69636674` | 1768121972 | Ideographic character face top edge (horizontal) or right edge (vertical). |
| `HB_OT_LAYOUT_BASELINE_TAG_IDEO_FACE_CENTRAL` | `Icfc` | `0x49636663` | 1231251043 | The centre of the ideographic character face. Since 4.0.0. |
| `HB_OT_LAYOUT_BASELINE_TAG_IDEO_EMBOX_BOTTOM_OR_LEFT` | `ideo` | `0x6964656F` | 1768187247 | Ideographic em-box bottom edge (horizontal) or left edge (vertical). |
| `HB_OT_LAYOUT_BASELINE_TAG_IDEO_EMBOX_TOP_OR_RIGHT` | `idtp` | `0x69647470` | 1768191088 | Ideographic em-box top edge (horizontal) or right edge (vertical). |
| `HB_OT_LAYOUT_BASELINE_TAG_IDEO_EMBOX_CENTRAL` | `Idce` | `0x49646365` | 1231315813 | The centre of the ideographic em-box. Since 4.0.0. |
| `HB_OT_LAYOUT_BASELINE_TAG_MATH` | `math` | `0x6D617468` | 1835103336 | The baseline about which mathematical characters are centred; in vertical writing mode, about which they are centred after a 90° clockwise rotation. |

Note the two capitalised tags, `Icfc` and `Idce`. They are capitalised in the
registry and in the header, and a lowercase `icfc` or `idce` is a *different*
(and unregistered) tag.

The C enumeration ends with `_HB_OT_LAYOUT_BASELINE_TAG_MAX_VALUE =
HB_TAG_MAX_SIGNED` (`0x7FFFFFFF`), which fits in an `int`, so the Rust alias is
`c_int`. The sentinel itself is marked `/*< skip >*/` and is not transcribed as
a constant.

### Types this header uses but does not declare

`hb-ot-layout.h` includes `hb.h` and `hb-ot-name.h`, so the following come from
elsewhere and are imported by the Rust module rather than redeclared:

| Type | Declared in | Rust module |
| --- | --- | --- |
| `hb_bool_t` | `hb-common.h` | `common` |
| `hb_codepoint_t` | `hb-common.h` | `common` |
| `hb_position_t` | `hb-common.h` | `common` |
| `hb_tag_t` | `hb-common.h` | `common` |
| `hb_direction_t` | `hb-common.h` | `common` |
| `hb_language_t` | `hb-common.h` | `common` |
| `hb_script_t` | `hb-script.h` | `script` |
| `hb_face_t` | `hb-face.h` | `face` |
| `hb_font_t` | `hb-font.h` | `font` |
| `hb_font_extents_t` | `hb-font.h` | `font` |
| `hb_set_t` | `hb-set.h` | `set` |
| `hb_map_t` | `hb-map.h` | `map` |
| `hb_ot_name_id_t` | `hb-ot-name.h` | `ot_name` |

`hb_ot_name_id_t` is a `typedef unsigned int`; the sentinel
`HB_OT_NAME_ID_INVALID` is `0xFFFF`.

## Constants

### Table tags

```c
#define HB_OT_TAG_BASE HB_TAG('B','A','S','E')
#define HB_OT_TAG_GDEF HB_TAG('G','D','E','F')
#define HB_OT_TAG_GSUB HB_TAG('G','S','U','B')
#define HB_OT_TAG_GPOS HB_TAG('G','P','O','S')
#define HB_OT_TAG_JSTF HB_TAG('J','S','T','F')
```

```rust
pub const HB_OT_TAG_BASE: hb_tag_t = HB_TAG(b'B', b'A', b'S', b'E');
pub const HB_OT_TAG_GDEF: hb_tag_t = HB_TAG(b'G', b'D', b'E', b'F');
pub const HB_OT_TAG_GSUB: hb_tag_t = HB_TAG(b'G', b'S', b'U', b'B');
pub const HB_OT_TAG_GPOS: hb_tag_t = HB_TAG(b'G', b'P', b'O', b'S');
pub const HB_OT_TAG_JSTF: hb_tag_t = HB_TAG(b'J', b'S', b'T', b'F');
```

| Constant | Tag | Value | OpenType table |
| --- | --- | --- | --- |
| `HB_OT_TAG_BASE` | `BASE` | `0x42415345` | [Baseline Table](https://docs.microsoft.com/en-us/typography/opentype/spec/base) |
| `HB_OT_TAG_GDEF` | `GDEF` | `0x47444546` | [Glyph Definition Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gdef) |
| `HB_OT_TAG_GSUB` | `GSUB` | `0x47535542` | [Glyph Substitution Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gsub) |
| `HB_OT_TAG_GPOS` | `GPOS` | `0x47504F53` | [Glyph Positioning Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos) |
| `HB_OT_TAG_JSTF` | `JSTF` | `0x4A535446` | [Justification Table](https://docs.microsoft.com/en-us/typography/opentype/spec/jstf) |

Only `HB_OT_TAG_GSUB` and `HB_OT_TAG_GPOS` are accepted as the `table_tag`
argument of the query functions. `HB_OT_TAG_BASE`, `HB_OT_TAG_GDEF` and
`HB_OT_TAG_JSTF` are provided for use with `hb_face_reference_table()` and
similar; `JSTF` in particular has no query API in HarfBuzz at all.

### Script and language tags

```c
#define HB_OT_TAG_DEFAULT_SCRIPT    HB_TAG ('D', 'F', 'L', 'T')
#define HB_OT_TAG_DEFAULT_LANGUAGE  HB_TAG ('d', 'f', 'l', 't')
#define HB_OT_MAX_TAGS_PER_SCRIPT   3u
#define HB_OT_MAX_TAGS_PER_LANGUAGE 3u
```

```rust
pub const HB_OT_TAG_DEFAULT_SCRIPT: hb_tag_t = HB_TAG(b'D', b'F', b'L', b'T');
pub const HB_OT_TAG_DEFAULT_LANGUAGE: hb_tag_t = HB_TAG(b'd', b'f', b'l', b't');
pub const HB_OT_MAX_TAGS_PER_SCRIPT: c_uint = 3;
pub const HB_OT_MAX_TAGS_PER_LANGUAGE: c_uint = 3;
```

| Constant | Value | Meaning |
| --- | --- | --- |
| `HB_OT_TAG_DEFAULT_SCRIPT` | `DFLT` (`0x44464C54`) | The OpenType script tag for features that are not script-specific. |
| `HB_OT_TAG_DEFAULT_LANGUAGE` | `dflt` (`0x64666C74`) | Not a valid language tag, but some fonts mistakenly use it — including, historically, several versions of DejaVu Sans Mono. HarfBuzz's script fallback chain tries it after `DFLT`. |
| `HB_OT_MAX_TAGS_PER_SCRIPT` | 3 | Maximum number of OpenType tags one `hb_script_t` can map to. Since 2.0.0. |
| `HB_OT_MAX_TAGS_PER_LANGUAGE` | 3 | Maximum number of OpenType tags one `hb_language_t` can map to. Since 2.0.0. |

The two `MAX_TAGS` constants are the safe stack-buffer size for the arrays you
hand to `hb_ot_tags_from_script_and_language()`.

### Index sentinels

```c
#define HB_OT_LAYOUT_NO_SCRIPT_INDEX         0xFFFFu
#define HB_OT_LAYOUT_NO_FEATURE_INDEX        0xFFFFu
#define HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX  0xFFFFu
#define HB_OT_LAYOUT_NO_VARIATIONS_INDEX     0xFFFFFFFFu
```

```rust
pub const HB_OT_LAYOUT_NO_SCRIPT_INDEX: c_uint = 0xFFFF;
pub const HB_OT_LAYOUT_NO_FEATURE_INDEX: c_uint = 0xFFFF;
pub const HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX: c_uint = 0xFFFF;
pub const HB_OT_LAYOUT_NO_VARIATIONS_INDEX: c_uint = 0xFFFF_FFFF;
```

| Constant | Value | Meaning |
| --- | --- | --- |
| `HB_OT_LAYOUT_NO_SCRIPT_INDEX` | `0xFFFF` | Script index indicating an unsupported script. |
| `HB_OT_LAYOUT_NO_FEATURE_INDEX` | `0xFFFF` | Feature index indicating an unsupported feature. |
| `HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX` | `0xFFFF` | Language index indicating the *default* language system of a script — the `DefaultLangSys` record — or an unsupported language. The two cases are indistinguishable from the value alone; the return value of the function that produced it tells them apart. |
| `HB_OT_LAYOUT_NO_VARIATIONS_INDEX` | `0xFFFFFFFF` | Variations index indicating that no feature-variations record applies. |

The first three share a value, so a `0xFFFF` you got from a script query is not
interchangeable with one from a feature query even though they compare equal.

## Functions

### Script and language tag conversion

These four live in `hb-ot-tag.cc` upstream but are declared by this header.

#### `hb_ot_tags_from_script_and_language`

```c
void
hb_ot_tags_from_script_and_language (hb_script_t   script,
                                     hb_language_t language,
                                     unsigned int *script_count   /* IN/OUT */,
                                     hb_tag_t     *script_tags    /* OUT */,
                                     unsigned int *language_count /* IN/OUT */,
                                     hb_tag_t     *language_tags  /* OUT */);
```

```rust
pub fn hb_ot_tags_from_script_and_language(
    script: hb_script_t,
    language: hb_language_t,
    script_count: *mut c_uint,
    script_tags: *mut hb_tag_t,
    language_count: *mut c_uint,
    language_tags: *mut hb_tag_t,
);
```

Converts an `hb_script_t` and an `hb_language_t` into the OpenType script and
language tags a font would use for them.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `script` | The script to convert. Required (passed by value). |
| `language` | The language to convert. **Nullable** — `HB_LANGUAGE_INVALID` is accepted and means "no language", in which case no language tags are produced. |
| `script_count` | In/out. On entry, capacity of `script_tags`; on return, number written. **May be null**, in which case `script_tags` is not filled. |
| `script_tags` | Out. Caller-allocated array of at least `*script_count` tags. **May be null.** |
| `language_count` | In/out, same convention. **May be null.** |
| `language_tags` | Out. Caller-allocated array of at least `*language_count` tags. **May be null.** |

**Returns** — nothing. The counts are the output channel.

**Ownership** — nothing is allocated; everything is written into caller memory.

**Notes** — at most `HB_OT_MAX_TAGS_PER_SCRIPT` (3) script tags and
`HB_OT_MAX_TAGS_PER_LANGUAGE` (3) language tags are ever produced, so
`hb_tag_t buf[3]` is always large enough. Tags come back in **preference
order**: try them against the font in the order given. Since HarfBuzz 2.0.0.

#### `hb_ot_tag_to_script`

```c
hb_script_t hb_ot_tag_to_script (hb_tag_t tag);
```

```rust
pub fn hb_ot_tag_to_script(tag: hb_tag_t) -> hb_script_t;
```

Converts an OpenType script tag to an `hb_script_t`.

**Parameters** — `tag`: any four-byte tag. Unregistered tags are handled by the
generic rule (uppercase the first letter of the ISO 15924-style tag), so this
function does not fail.

**Returns** — the corresponding `hb_script_t`.

**Ownership** — none; `hb_script_t` is a plain integer.

**Notes** — the header states no since-version for this function. The mapping is
not injective in the other direction: several OpenType tags (`deva`/`dev2`,
`beng`/`bng2`, …) map to one script.

#### `hb_ot_tag_to_language`

```c
hb_language_t hb_ot_tag_to_language (hb_tag_t tag);
```

```rust
pub fn hb_ot_tag_to_language(tag: hb_tag_t) -> hb_language_t;
```

Converts an OpenType language tag to an `hb_language_t`.

**Parameters** — `tag`: any four-byte tag.

**Returns** — the corresponding language, or `HB_LANGUAGE_INVALID` (null) when
the tag has no BCP 47 equivalent. **Check for null.**

**Ownership** — transfer none. Languages are interned for the lifetime of the
process; never free the result, and compare by pointer.

**Notes** — Since HarfBuzz 0.9.2.

#### `hb_ot_tags_to_script_and_language`

```c
void
hb_ot_tags_to_script_and_language (hb_tag_t       script_tag,
                                   hb_tag_t       language_tag,
                                   hb_script_t   *script   /* OUT */,
                                   hb_language_t *language /* OUT */);
```

```rust
pub fn hb_ot_tags_to_script_and_language(
    script_tag: hb_tag_t,
    language_tag: hb_tag_t,
    script: *mut hb_script_t,
    language: *mut hb_language_t,
);
```

The inverse of `hb_ot_tags_from_script_and_language()`: converts a script tag
and a language tag back into an `hb_script_t` and an `hb_language_t`.

**Parameters** — `script`, `language`: out pointers, **either may be null** if
you only want one half. The resolved `language` depends on *both* input tags,
not just `language_tag`, because a script tag can imply a language variant.

**Returns** — nothing.

**Ownership** — the written `hb_language_t` is interned; do not free it.

**Notes** — Since HarfBuzz 2.0.0.

### GDEF — glyph classes, attachment points, ligature carets

#### `hb_ot_layout_has_glyph_classes`

```c
hb_bool_t hb_ot_layout_has_glyph_classes (hb_face_t *face);
```

```rust
pub fn hb_ot_layout_has_glyph_classes(face: *mut hb_face_t) -> hb_bool_t;
```

Tests whether a face has any glyph classes defined in its `GDEF` table.

**Parameters** — `face`: the face to query. Nullability unspecified by the
header; pass a real face.

**Returns** — true if the data was found, false otherwise.

**Ownership** — borrows `face` for the duration of the call; takes no reference.

**Notes** — the header states no since-version. Call this before trusting
`hb_ot_layout_get_glyph_class()`, which cannot distinguish "explicitly
unclassified" from "no table".

#### `hb_ot_layout_get_glyph_class`

```c
hb_ot_layout_glyph_class_t
hb_ot_layout_get_glyph_class (hb_face_t      *face,
                              hb_codepoint_t  glyph);
```

```rust
pub fn hb_ot_layout_get_glyph_class(
    face: *mut hb_face_t,
    glyph: hb_codepoint_t,
) -> hb_ot_layout_glyph_class_t;
```

Fetches the `GDEF` class of `glyph` in `face`.

**Parameters** — `glyph` is a **glyph ID**, not a Unicode codepoint, despite the
`hb_codepoint_t` type. Out-of-range glyph IDs are not an error.

**Returns** — one of the five `HB_OT_LAYOUT_GLYPH_CLASS_*` values, or
`HB_OT_LAYOUT_GLYPH_CLASS_UNCLASSIFIED` when the glyph is unclassified, out of
range, or the face has no `GDEF` glyph-class table.

**Ownership** — none.

**Notes** — Since HarfBuzz 0.9.7.

#### `hb_ot_layout_get_glyphs_in_class`

```c
void
hb_ot_layout_get_glyphs_in_class (hb_face_t                  *face,
                                  hb_ot_layout_glyph_class_t  klass,
                                  hb_set_t                   *glyphs /* OUT */);
```

```rust
pub fn hb_ot_layout_get_glyphs_in_class(
    face: *mut hb_face_t,
    klass: hb_ot_layout_glyph_class_t,
    glyphs: *mut hb_set_t,
);
```

Retrieves every glyph of the face that belongs to `klass` in the face's `GDEF`
table.

**Parameters** — `glyphs`: a caller-owned set that receives the result. The
header does not mark it nullable; treat it as required.

**Returns** — nothing.

**Ownership** — the set is **added to**, not cleared first. Call
`hb_set_clear()` yourself if you want only this class's glyphs. The caller keeps
ownership of the set and must destroy it.

**Notes** — Since HarfBuzz 0.9.7. Watch for allocation failure on the set with
`hb_set_allocation_successful()`.

#### `hb_ot_layout_get_attach_points`

```c
unsigned int
hb_ot_layout_get_attach_points (hb_face_t      *face,
                                hb_codepoint_t  glyph,
                                unsigned int    start_offset,
                                unsigned int   *point_count /* IN/OUT */,
                                unsigned int   *point_array /* OUT */);
```

```rust
pub fn hb_ot_layout_get_attach_points(
    face: *mut hb_face_t,
    glyph: hb_codepoint_t,
    start_offset: c_uint,
    point_count: *mut c_uint,
    point_array: *mut c_uint,
) -> c_uint;
```

Fetches the list of attachment points defined for `glyph` in the face's `GDEF`
table, beginning at `start_offset`.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `glyph` | Glyph ID to query. |
| `start_offset` | Index of the first attachment point to return. |
| `point_count` | In/out: capacity in, number written out (may be zero). **Nullable.** |
| `point_array` | Out: caller-allocated array of contour-point indices. **Nullable.** |

**Returns** — the **total** number of attachment points for `glyph`, regardless
of `start_offset` or capacity.

**Ownership** — nothing allocated.

**Notes** — the header comments that this is "not that useful", and is provided
because a client may want to cache the list. The values are contour point
indices in the glyph outline, not coordinates. No since-version is stated.

#### `hb_ot_layout_get_ligature_carets`

```c
unsigned int
hb_ot_layout_get_ligature_carets (hb_font_t      *font,
                                  hb_direction_t  direction,
                                  hb_codepoint_t  glyph,
                                  unsigned int    start_offset,
                                  unsigned int   *caret_count /* IN/OUT */,
                                  hb_position_t  *caret_array /* OUT */);
```

```rust
pub fn hb_ot_layout_get_ligature_carets(
    font: *mut hb_font_t,
    direction: hb_direction_t,
    glyph: hb_codepoint_t,
    start_offset: c_uint,
    caret_count: *mut c_uint,
    caret_array: *mut hb_position_t,
) -> c_uint;
```

Fetches the caret positions defined for a ligature glyph in the font's `GDEF`
table, beginning at `start_offset`.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | A **font**, not a face — carets are returned in scaled font units, so the font's scale and variation coordinates matter. |
| `direction` | Text direction; selects the horizontal or vertical caret list. |
| `glyph` | Glyph ID of the ligature. |
| `start_offset` | Index of the first caret position to return. |
| `caret_count` | In/out: capacity in, number written out. **Nullable.** |
| `caret_array` | Out: caller-allocated array of positions. **Nullable.** |

**Returns** — the total number of ligature caret positions for `glyph`.

**Ownership** — nothing allocated.

**Notes** — a ligature formed from *n* characters has *n* − 1 caret positions:
the first character is not represented, because its caret position is the glyph
position. The positions are **"unshaped"** and must be fixed up for any kerning
applied to the ligature glyph. No since-version is stated.

### GSUB/GPOS — scripts

#### `hb_ot_layout_table_get_script_tags`

```c
unsigned int
hb_ot_layout_table_get_script_tags (hb_face_t    *face,
                                    hb_tag_t      table_tag,
                                    unsigned int  start_offset,
                                    unsigned int *script_count /* IN/OUT */,
                                    hb_tag_t     *script_tags  /* OUT */);
```

```rust
pub fn hb_ot_layout_table_get_script_tags(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    start_offset: c_uint,
    script_count: *mut c_uint,
    script_tags: *mut hb_tag_t,
) -> c_uint;
```

Fetches the list of all scripts enumerated in the face's `GSUB` or `GPOS` table,
beginning at `start_offset`.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `table_tag` | Must be `HB_OT_TAG_GSUB` or `HB_OT_TAG_GPOS`. Any other tag selects HarfBuzz's null table object, so the call returns 0 and writes nothing — a silent no-op, not an error. |
| `start_offset` | Index of the first script tag to return. |
| `script_count` | In/out: capacity in, number written out. **Nullable.** |
| `script_tags` | Out: caller-allocated array. **Nullable.** |

**Returns** — the total number of script tags in the table.

**Ownership** — nothing allocated.

**Notes** — the index of a tag in this enumeration *is* its script index for
every other function in this family. No since-version is stated.

#### `hb_ot_layout_table_find_script`

```c
hb_bool_t
hb_ot_layout_table_find_script (hb_face_t    *face,
                                hb_tag_t      table_tag,
                                hb_tag_t      script_tag,
                                unsigned int *script_index /* OUT */);
```

```rust
pub fn hb_ot_layout_table_find_script(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    script_tag: hb_tag_t,
    script_index: *mut c_uint,
) -> hb_bool_t;
```

Fetches the index of `script_tag` in the face's `GSUB` or `GPOS` table.

**Parameters** — `script_index`: out pointer. The header does not mark it
nullable, but the implementation guards every write, so null is tolerated.

**Returns** — true only when `script_tag` **itself** was found. When it was not,
the function returns **false** but has still tried `DFLT`, then `dflt`, then
`latn`, and written whichever of those it found into `script_index`. If none of
them is present either, `script_index` is set to
`HB_OT_LAYOUT_NO_SCRIPT_INDEX`. A false return therefore does *not* mean
`script_index` is meaningless — see *Pitfalls*.

**Ownership** — nothing allocated.

**Notes** — upstream's reference manual indexes this function under its
**deprecated** section and points at `hb_ot_layout_table_select_script()`
instead, although the header does not apply `HB_DEPRECATED_FOR` to it and the
Rust transcription therefore carries no `#[deprecated]` attribute. Prefer
`hb_ot_layout_table_select_script()` in new code: it makes the fallback outcome
visible through `chosen_script`. No since-version is stated.

#### `hb_ot_layout_table_select_script`

```c
hb_bool_t
hb_ot_layout_table_select_script (hb_face_t      *face,
                                  hb_tag_t        table_tag,
                                  unsigned int    script_count,
                                  const hb_tag_t *script_tags,
                                  unsigned int   *script_index  /* OUT */,
                                  hb_tag_t       *chosen_script /* OUT */);
```

```rust
pub fn hb_ot_layout_table_select_script(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    script_count: c_uint,
    script_tags: *const hb_tag_t,
    script_index: *mut c_uint,
    chosen_script: *mut hb_tag_t,
) -> hb_bool_t;
```

Selects an OpenType script for `table_tag` from the `script_tags` array, in
order of preference.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `script_count` | Number of tags in `script_tags`. |
| `script_tags` | Input array of candidate tags, most preferred first. Borrowed for the call only. |
| `script_index` | Out. **Nullable.** |
| `chosen_script` | Out. **Nullable.** |

**Returns** — true if one of the **requested** scripts was selected; false if a
fallback was used *or* nothing was found.

If the table has none of the requested scripts, `DFLT`, `dflt`, and `latn` are
tried in that order. If it has none of those either, `script_index` is set to
`HB_OT_LAYOUT_NO_SCRIPT_INDEX` and `chosen_script` to `HB_TAG_NONE`.

**Ownership** — `script_tags` is not retained.

**Notes** — this is the function to pair with
`hb_ot_tags_from_script_and_language()`, which produces exactly such a
preference-ordered array. Since HarfBuzz 2.0.0.

### GSUB/GPOS — languages

#### `hb_ot_layout_script_get_language_tags`

```c
unsigned int
hb_ot_layout_script_get_language_tags (hb_face_t    *face,
                                       hb_tag_t      table_tag,
                                       unsigned int  script_index,
                                       unsigned int  start_offset,
                                       unsigned int *language_count /* IN/OUT */,
                                       hb_tag_t     *language_tags  /* OUT */);
```

```rust
pub fn hb_ot_layout_script_get_language_tags(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    script_index: c_uint,
    start_offset: c_uint,
    language_count: *mut c_uint,
    language_tags: *mut hb_tag_t,
) -> c_uint;
```

Fetches the language tags declared under `script_index` in the face's `GSUB` or
`GPOS` table, beginning at `start_offset`.

**Parameters** — `script_index` must come from this same table. An out-of-range
index selects the null script object and yields zero tags.

**Returns** — the total number of language tags under that script.

**Ownership** — nothing allocated.

**Notes** — the enumeration covers only the explicit `LangSysRecord` entries. The
script's `DefaultLangSys` is **not** in this list; it is addressed by
`HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX`. Since HarfBuzz 0.6.0.

#### `hb_ot_layout_script_select_language`

```c
hb_bool_t
hb_ot_layout_script_select_language (hb_face_t      *face,
                                     hb_tag_t        table_tag,
                                     unsigned int    script_index,
                                     unsigned int    language_count,
                                     const hb_tag_t *language_tags,
                                     unsigned int   *language_index /* OUT */);
```

```rust
pub fn hb_ot_layout_script_select_language(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    script_index: c_uint,
    language_count: c_uint,
    language_tags: *const hb_tag_t,
    language_index: *mut c_uint,
) -> hb_bool_t;
```

Fetches the index of the first tag from `language_tags` that is present under
`script_index`.

**Parameters** — `language_tags`: input array of candidates, most preferred
first; borrowed for the call only. `language_index`: out.

**Returns** — true if one of the given tags was found. If none was, returns
false and sets `language_index` to the **default language index**
(`HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX`), which is a usable index for the
script's `DefaultLangSys` — so you can carry on regardless.

**Ownership** — `language_tags` is not retained.

**Notes** — Since HarfBuzz 2.0.0.

#### `hb_ot_layout_script_select_language2`

```c
hb_bool_t
hb_ot_layout_script_select_language2 (hb_face_t      *face,
                                      hb_tag_t        table_tag,
                                      unsigned int    script_index,
                                      unsigned int    language_count,
                                      const hb_tag_t *language_tags,
                                      unsigned int   *language_index  /* OUT */,
                                      hb_tag_t       *chosen_language /* OUT */);
```

```rust
pub fn hb_ot_layout_script_select_language2(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    script_index: c_uint,
    language_count: c_uint,
    language_tags: *const hb_tag_t,
    language_index: *mut c_uint,
    chosen_language: *mut hb_tag_t,
) -> hb_bool_t;
```

As `hb_ot_layout_script_select_language()`, but also reports which tag was
chosen.

**Parameters** — as above, plus `chosen_language`: out.

**Returns** — true if one of the given tags was found. If none was, returns
false, sets `language_index` to `HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX` and
`chosen_language` to `HB_TAG_NONE`.

**Ownership** — `language_tags` is not retained.

**Notes** — prefer this over the `1`-suffixless version when you need to report
or log which language system was actually used. Since HarfBuzz 7.0.0.

#### `hb_ot_layout_script_find_language`

Listed in this section of upstream's reference manual, but **declared in
`hb-ot-deprecated.h`**, not in `hb-ot-layout.h`. It is therefore transcribed in
the `ot_deprecated` Rust module, not in `ot_layout`.

```c
HB_DEPRECATED_FOR (hb_ot_layout_script_select_language)
hb_bool_t
hb_ot_layout_script_find_language (hb_face_t    *face,
                                   hb_tag_t      table_tag,
                                   unsigned int  script_index,
                                   hb_tag_t      language_tag,
                                   unsigned int *language_index);
```

Fetches the index of a single language tag under `script_index`. Exactly
equivalent to calling `hb_ot_layout_script_select_language()` with a one-element
array. Returns true if the language tag was found. Since HarfBuzz 0.6.0;
**deprecated since 2.0.0** in favour of
`hb_ot_layout_script_select_language()`. Compiled out when HarfBuzz is built
with `HB_DISABLE_DEPRECATED`.

### GSUB/GPOS — features

#### `hb_ot_layout_table_get_feature_tags`

```c
unsigned int
hb_ot_layout_table_get_feature_tags (hb_face_t    *face,
                                     hb_tag_t      table_tag,
                                     unsigned int  start_offset,
                                     unsigned int *feature_count /* IN/OUT */,
                                     hb_tag_t     *feature_tags  /* OUT */);
```

```rust
pub fn hb_ot_layout_table_get_feature_tags(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    start_offset: c_uint,
    feature_count: *mut c_uint,
    feature_tags: *mut hb_tag_t,
) -> c_uint;
```

Fetches all feature tags in the face's `GSUB` or `GPOS` table, beginning at
`start_offset`.

**Returns** — the total number of feature tags.

**Ownership** — nothing allocated.

**Notes** — this is the table's flat `FeatureList`, so there **may be duplicate
tags**, belonging to different script/language-system pairs. The position of an
entry here is its feature index. To de-duplicate, collect into an `hb_set_t`
instead, or use `hb_ot_layout_collect_features()`. Since HarfBuzz 0.6.0.

#### `hb_ot_layout_language_get_required_feature_index`

```c
hb_bool_t
hb_ot_layout_language_get_required_feature_index (hb_face_t    *face,
                                                  hb_tag_t      table_tag,
                                                  unsigned int  script_index,
                                                  unsigned int  language_index,
                                                  unsigned int *feature_index /* OUT */);
```

```rust
pub fn hb_ot_layout_language_get_required_feature_index(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    script_index: c_uint,
    language_index: c_uint,
    feature_index: *mut c_uint,
) -> hb_bool_t;
```

Fetches the index of the **required feature** — the `RequiredFeatureIndex` field
of a `LangSys` record — for the given script and language system.

**Parameters** — `language_index` may be `HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX`
to address the script's default language system.

**Returns** — true if a required feature is declared; false otherwise, in which
case `feature_index` is set to `HB_OT_LAYOUT_NO_FEATURE_INDEX`.

**Ownership** — nothing allocated.

**Notes** — a required feature is applied unconditionally by the shaper and is
not listed among the language system's ordinary feature indices. Since HarfBuzz
0.6.0.

#### `hb_ot_layout_language_get_required_feature`

```c
hb_bool_t
hb_ot_layout_language_get_required_feature (hb_face_t    *face,
                                            hb_tag_t      table_tag,
                                            unsigned int  script_index,
                                            unsigned int  language_index,
                                            unsigned int *feature_index /* OUT */,
                                            hb_tag_t     *feature_tag   /* OUT */);
```

```rust
pub fn hb_ot_layout_language_get_required_feature(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    script_index: c_uint,
    language_index: c_uint,
    feature_index: *mut c_uint,
    feature_tag: *mut hb_tag_t,
) -> hb_bool_t;
```

As `hb_ot_layout_language_get_required_feature_index()`, but also reports the
required feature's tag.

**Returns** — true if the feature is found, false otherwise.

**Ownership** — nothing allocated.

**Notes** — despite the name difference, the `_index` variant is the older of
the two. Since HarfBuzz 0.9.30.

#### `hb_ot_layout_language_get_feature_indexes`

```c
unsigned int
hb_ot_layout_language_get_feature_indexes (hb_face_t    *face,
                                           hb_tag_t      table_tag,
                                           unsigned int  script_index,
                                           unsigned int  language_index,
                                           unsigned int  start_offset,
                                           unsigned int *feature_count   /* IN/OUT */,
                                           unsigned int *feature_indexes /* OUT */);
```

```rust
pub fn hb_ot_layout_language_get_feature_indexes(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    script_index: c_uint,
    language_index: c_uint,
    start_offset: c_uint,
    feature_count: *mut c_uint,
    feature_indexes: *mut c_uint,
) -> c_uint;
```

Fetches the feature **indices** listed under the given script and language
system, beginning at `start_offset`.

**Returns** — the total number of features under that language system.

**Ownership** — nothing allocated.

**Notes** — these are indices into the table's flat feature list, suitable for
`hb_ot_layout_feature_get_lookups()`. Since HarfBuzz 0.6.0.

#### `hb_ot_layout_language_get_feature_tags`

```c
unsigned int
hb_ot_layout_language_get_feature_tags (hb_face_t    *face,
                                        hb_tag_t      table_tag,
                                        unsigned int  script_index,
                                        unsigned int  language_index,
                                        unsigned int  start_offset,
                                        unsigned int *feature_count /* IN/OUT */,
                                        hb_tag_t     *feature_tags  /* OUT */);
```

```rust
pub fn hb_ot_layout_language_get_feature_tags(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    script_index: c_uint,
    language_index: c_uint,
    start_offset: c_uint,
    feature_count: *mut c_uint,
    feature_tags: *mut hb_tag_t,
) -> c_uint;
```

The same query as `hb_ot_layout_language_get_feature_indexes()`, returning the
features' **tags** rather than their indices.

**Returns** — the total number of feature tags.

**Ownership** — nothing allocated.

**Notes** — this is the function behind a "which features does this font offer
for this script/language?" UI. Since HarfBuzz 0.6.0.

#### `hb_ot_layout_language_find_feature`

```c
hb_bool_t
hb_ot_layout_language_find_feature (hb_face_t    *face,
                                    hb_tag_t      table_tag,
                                    unsigned int  script_index,
                                    unsigned int  language_index,
                                    hb_tag_t      feature_tag,
                                    unsigned int *feature_index /* OUT */);
```

```rust
pub fn hb_ot_layout_language_find_feature(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    script_index: c_uint,
    language_index: c_uint,
    feature_tag: hb_tag_t,
    feature_index: *mut c_uint,
) -> hb_bool_t;
```

Fetches the index of `feature_tag` under the given script and language system.

**Returns** — true if the feature is found; false otherwise, with
`feature_index` set to `HB_OT_LAYOUT_NO_FEATURE_INDEX`.

**Ownership** — nothing allocated.

**Notes** — the direct way to answer "does this font support `liga` for Turkish
Latin?". Since HarfBuzz 0.6.0.

#### `hb_ot_layout_collect_features`

```c
void
hb_ot_layout_collect_features (hb_face_t      *face,
                               hb_tag_t        table_tag,
                               const hb_tag_t *scripts,
                               const hb_tag_t *languages,
                               const hb_tag_t *features,
                               hb_set_t       *feature_indexes /* OUT */);
```

```rust
pub fn hb_ot_layout_collect_features(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    scripts: *const hb_tag_t,
    languages: *const hb_tag_t,
    features: *const hb_tag_t,
    feature_indexes: *mut hb_set_t,
);
```

Collects into `feature_indexes` every feature index of the table that lies under
the given scripts, languages, and features.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `scripts` | **Nullable**, `HB_TAG_NONE`-terminated. Null means *all scripts*. |
| `languages` | **Nullable**, `HB_TAG_NONE`-terminated. Null means *all languages*. |
| `features` | **Nullable**, `HB_TAG_NONE`-terminated. Null means *all features*. |
| `feature_indexes` | Out set, caller-owned. |

**Returns** — nothing.

**Ownership** — the three arrays are borrowed for the call only. The set is
**added to**, not cleared.

**Notes** — the natural first call for a subsetter or a feature auditor: one
call replaces a triple loop over scripts, languages, and features. Since
HarfBuzz 1.8.5.

#### `hb_ot_layout_collect_features_map`

```c
void
hb_ot_layout_collect_features_map (hb_face_t *face,
                                   hb_tag_t   table_tag,
                                   unsigned   script_index,
                                   unsigned   language_index,
                                   hb_map_t  *feature_map /* OUT */);
```

```rust
pub fn hb_ot_layout_collect_features_map(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    script_index: c_uint,
    language_index: c_uint,
    feature_map: *mut hb_map_t,
);
```

Fetches the mapping from feature **tag** to feature **index** for one script and
language system.

**Parameters** — `feature_map`: a caller-owned `hb_map_t` that receives
`tag → index` entries.

**Returns** — nothing.

**Ownership** — the map is added to, not cleared; the caller destroys it.

**Notes** — this collapses the duplicate-tag problem of
`hb_ot_layout_table_get_feature_tags()`: because a map has unique keys, a tag
appearing more than once under the same language system is stored once. Since
HarfBuzz 8.1.0.

### GSUB/GPOS — lookups

#### `hb_ot_layout_table_get_lookup_count`

```c
unsigned int
hb_ot_layout_table_get_lookup_count (hb_face_t *face,
                                     hb_tag_t   table_tag);
```

```rust
pub fn hb_ot_layout_table_get_lookup_count(face: *mut hb_face_t, table_tag: hb_tag_t) -> c_uint;
```

Fetches the total number of lookups enumerated in the face's `GSUB` or `GPOS`
table.

**Returns** — the count. Valid lookup indices for that table run from 0 to
count − 1.

**Ownership** — nothing allocated.

**Notes** — use this to bound any loop over lookup indices, and to validate an
index before passing it to `hb_ot_layout_lookup_collect_glyphs()` or the
closure functions. Since HarfBuzz 0.9.22.

#### `hb_ot_layout_feature_get_lookups`

```c
unsigned int
hb_ot_layout_feature_get_lookups (hb_face_t    *face,
                                  hb_tag_t      table_tag,
                                  unsigned int  feature_index,
                                  unsigned int  start_offset,
                                  unsigned int *lookup_count   /* IN/OUT */,
                                  unsigned int *lookup_indexes /* OUT */);
```

```rust
pub fn hb_ot_layout_feature_get_lookups(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    feature_index: c_uint,
    start_offset: c_uint,
    lookup_count: *mut c_uint,
    lookup_indexes: *mut c_uint,
) -> c_uint;
```

Fetches the lookup indices enumerated for `feature_index`, beginning at
`start_offset`.

**Returns** — the total number of lookups for that feature.

**Ownership** — nothing allocated.

**Notes** — equivalent to calling
`hb_ot_layout_feature_with_variations_get_lookups()` with
`HB_OT_LAYOUT_NO_VARIATIONS_INDEX`. Since HarfBuzz 0.9.7.

#### `hb_ot_layout_collect_lookups`

```c
void
hb_ot_layout_collect_lookups (hb_face_t      *face,
                              hb_tag_t        table_tag,
                              const hb_tag_t *scripts,
                              const hb_tag_t *languages,
                              const hb_tag_t *features,
                              hb_set_t       *lookup_indexes /* OUT */);
```

```rust
pub fn hb_ot_layout_collect_lookups(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    scripts: *const hb_tag_t,
    languages: *const hb_tag_t,
    features: *const hb_tag_t,
    lookup_indexes: *mut hb_set_t,
);
```

Collects into `lookup_indexes` every lookup index reachable from the table under
the given scripts, languages, and features.

**Parameters** — `scripts`, `languages`, `features` are each **nullable** and
`HB_TAG_NONE`-terminated; null means "all of them", exactly as for
`hb_ot_layout_collect_features()`.

**Returns** — nothing.

**Ownership** — arrays borrowed for the call; the set is added to, not cleared.

**Notes** — Since HarfBuzz 0.9.8.

#### `hb_ot_layout_lookup_collect_glyphs`

```c
void
hb_ot_layout_lookup_collect_glyphs (hb_face_t    *face,
                                    hb_tag_t      table_tag,
                                    unsigned int  lookup_index,
                                    hb_set_t     *glyphs_before /* OUT.  May be NULL */,
                                    hb_set_t     *glyphs_input  /* OUT.  May be NULL */,
                                    hb_set_t     *glyphs_after  /* OUT.  May be NULL */,
                                    hb_set_t     *glyphs_output /* OUT.  May be NULL */);
```

```rust
pub fn hb_ot_layout_lookup_collect_glyphs(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    lookup_index: c_uint,
    glyphs_before: *mut hb_set_t,
    glyphs_input: *mut hb_set_t,
    glyphs_after: *mut hb_set_t,
    glyphs_output: *mut hb_set_t,
);
```

Collects every glyph affected by `lookup_index` in the face's `GSUB` or `GPOS`
table, split into four roles.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `glyphs_before` | Glyphs matched as **backtrack** context, preceding the substitution range. **Nullable.** |
| `glyphs_input` | Glyphs the lookup would act on. **Nullable.** |
| `glyphs_after` | Glyphs matched as **lookahead** context, following the range. **Nullable.** |
| `glyphs_output` | Glyphs the lookup would produce. **Nullable.** |

**Returns** — nothing.

**Ownership** — each non-null set is caller-owned and is **added to**, not
cleared.

**Notes** — pass null for the roles you do not need; that genuinely skips the
work. Since HarfBuzz 0.9.7.

### Feature variations

#### `hb_ot_layout_table_find_feature_variations`

```c
hb_bool_t
hb_ot_layout_table_find_feature_variations (hb_face_t    *face,
                                            hb_tag_t      table_tag,
                                            const int    *coords,
                                            unsigned int  num_coords,
                                            unsigned int *variations_index /* out */);
```

```rust
pub fn hb_ot_layout_table_find_feature_variations(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    coords: *const c_int,
    num_coords: c_uint,
    variations_index: *mut c_uint,
) -> hb_bool_t;
```

Finds the feature-variations record of the table that applies at the given
design-space location.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `coords` | **Normalized** variation coordinates — 2.14 fixed point, so −16384 … 16384 — one per axis, in axis order. This is exactly what `hb_font_get_var_coords_normalized()` returns. |
| `num_coords` | Number of coordinates in `coords`. Zero (with `coords` unused) asks for the default instance. |
| `variations_index` | Out. |

**Returns** — true if a record was found. `variations_index` receives the
record's index, or `HB_OT_LAYOUT_NO_VARIATIONS_INDEX` when none applies.

**Ownership** — `coords` is borrowed for the call only.

**Notes** — feature variations are how a variable font swaps in different
lookups at different design-space locations (the classic case being a different
`rvrn` behaviour at heavy weights). Since HarfBuzz 1.4.0.

#### `hb_ot_layout_feature_with_variations_get_lookups`

```c
unsigned int
hb_ot_layout_feature_with_variations_get_lookups (hb_face_t    *face,
                                                  hb_tag_t      table_tag,
                                                  unsigned int  feature_index,
                                                  unsigned int  variations_index,
                                                  unsigned int  start_offset,
                                                  unsigned int *lookup_count   /* IN/OUT */,
                                                  unsigned int *lookup_indexes /* OUT */);
```

```rust
pub fn hb_ot_layout_feature_with_variations_get_lookups(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    feature_index: c_uint,
    variations_index: c_uint,
    start_offset: c_uint,
    lookup_count: *mut c_uint,
    lookup_indexes: *mut c_uint,
) -> c_uint;
```

As `hb_ot_layout_feature_get_lookups()`, but reports the lookups the feature
enables at the given feature-variations index.

**Parameters** — `variations_index`: an index from
`hb_ot_layout_table_find_feature_variations()`, or
`HB_OT_LAYOUT_NO_VARIATIONS_INDEX` to get the unsubstituted lookups.

**Returns** — the total number of lookups.

**Ownership** — nothing allocated.

**Notes** — Since HarfBuzz 1.4.0.

### GSUB — substitution

#### `hb_ot_layout_has_substitution`

```c
hb_bool_t hb_ot_layout_has_substitution (hb_face_t *face);
```

```rust
pub fn hb_ot_layout_has_substitution(face: *mut hb_face_t) -> hb_bool_t;
```

Tests whether the face includes any `GSUB` substitutions.

**Returns** — true if the data was found, false otherwise.

**Ownership** — nothing allocated.

**Notes** — a face can have a `GSUB` table with no usable content; this reports
on the table's presence and validity as HarfBuzz sees it. Since HarfBuzz 0.6.0.

#### `hb_ot_layout_lookup_would_substitute`

```c
hb_bool_t
hb_ot_layout_lookup_would_substitute (hb_face_t            *face,
                                      unsigned int          lookup_index,
                                      const hb_codepoint_t *glyphs,
                                      unsigned int          glyphs_length,
                                      hb_bool_t             zero_context);
```

```rust
pub fn hb_ot_layout_lookup_would_substitute(
    face: *mut hb_face_t,
    lookup_index: c_uint,
    glyphs: *const hb_codepoint_t,
    glyphs_length: c_uint,
    zero_context: hb_bool_t,
) -> hb_bool_t;
```

Tests whether the `GSUB` lookup at `lookup_index` would trigger a substitution
on the given glyph sequence.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `lookup_index` | A `GSUB` lookup index. |
| `glyphs` | Input sequence of **glyph IDs**, borrowed for the call. |
| `glyphs_length` | Number of glyphs in the sequence. |
| `zero_context` | Non-zero means pre-context and post-context are disallowed — the sequence is treated as standing alone rather than embedded in a run. |

**Returns** — true if a substitution would be triggered, false otherwise.

**Ownership** — `glyphs` is not retained.

**Notes** — this is how HarfBuzz's own Arabic and Indic shapers probe for
`init`/`medi`/`fina` support, and how `hb_font_get_glyph_from_name`-style
tooling tests feature applicability without running a full shape. Since HarfBuzz
0.9.7.

#### `hb_ot_layout_lookup_get_glyph_alternates`

```c
unsigned
hb_ot_layout_lookup_get_glyph_alternates (hb_face_t      *face,
                                          unsigned        lookup_index,
                                          hb_codepoint_t  glyph,
                                          unsigned        start_offset,
                                          unsigned       *alternate_count  /* IN/OUT */,
                                          hb_codepoint_t *alternate_glyphs /* OUT */);
```

```rust
pub fn hb_ot_layout_lookup_get_glyph_alternates(
    face: *mut hb_face_t,
    lookup_index: c_uint,
    glyph: hb_codepoint_t,
    start_offset: c_uint,
    alternate_count: *mut c_uint,
    alternate_glyphs: *mut hb_codepoint_t,
) -> c_uint;
```

Fetches the alternates of `glyph` from the `GSUB` lookup at `lookup_index`,
beginning at `start_offset`.

**Parameters** — `alternate_count` is in/out and both it and `alternate_glyphs`
are **nullable**, following the usual convention.

**Returns** — the total number of alternates found in that lookup for that
glyph.

**Ownership** — nothing allocated.

**Notes** — for **one-to-one** GSUB substitutions this returns the substituted
glyph, so the function doubles as "what does this single-substitution lookup map
this glyph to?". The one-based index into the returned array is what you put in
`hb_feature_t::value` for a lookup-type-3 feature such as `salt` or `aalt`.
Since HarfBuzz 2.6.8.

#### `hb_ot_layout_lookup_collect_glyph_alternates`

```c
hb_bool_t
hb_ot_layout_lookup_collect_glyph_alternates (hb_face_t *face,
                                              unsigned   lookup_index,
                                              hb_map_t  *alternate_count  /* IN/OUT */,
                                              hb_map_t  *alternate_glyphs /* IN/OUT */);
```

```rust
pub fn hb_ot_layout_lookup_collect_glyph_alternates(
    face: *mut hb_face_t,
    lookup_index: c_uint,
    alternate_count: *mut hb_map_t,
    alternate_glyphs: *mut hb_map_t,
) -> hb_bool_t;
```

Collects the alternates of many glyphs from one `GSUB` lookup in a single pass —
the bulk form of `hb_ot_layout_lookup_get_glyph_alternates()`.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `alternate_count` | **In/out** map. On entry it must already contain the glyph IDs you care about as keys, with the number of alternates currently known for each as values. On return, the values are updated. |
| `alternate_glyphs` | **In/out** map. Alternate *i* of glyph *G* is stored under the key `G + (i << 24)`, for `i` in `0 … n−1` where `n` is `G`'s entry in `alternate_count`. |

**Returns** — true if alternates were collected. For one-to-one substitutions
the substituted glyph is collected; for lookups that assign multiple alternates,
all of them are. For **other lookup types nothing is done and false is
returned**.

**Ownership** — both maps are caller-owned and caller-destroyed; they are
mutated in place, never cleared.

**Notes** — the `G + (i << 24)` encoding limits it to glyph IDs below 2²⁴, which
is above the OpenType maximum of 65535, so it is not a practical constraint.
Since HarfBuzz 12.1.0.

#### `hb_ot_layout_lookup_substitute_closure`

```c
void
hb_ot_layout_lookup_substitute_closure (hb_face_t    *face,
                                        unsigned int  lookup_index,
                                        hb_set_t     *glyphs);
```

```rust
pub fn hb_ot_layout_lookup_substitute_closure(
    face: *mut hb_face_t,
    lookup_index: c_uint,
    glyphs: *mut hb_set_t,
);
```

Computes the transitive closure of glyphs needed for one lookup.

**Parameters** — `glyphs`: **in/out**. Seed it with the glyphs you already have;
the function adds every glyph the lookup can produce from them, repeatedly,
until the set stops growing.

**Returns** — nothing.

**Ownership** — the set is caller-owned and is added to, never cleared.

**Notes** — the header carries a `/*TODO , hb_bool_t inclusive */` comment: an
"inclusive" flag was contemplated and never added. Since HarfBuzz 0.9.7.

#### `hb_ot_layout_lookups_substitute_closure`

```c
void
hb_ot_layout_lookups_substitute_closure (hb_face_t      *face,
                                         const hb_set_t *lookups,
                                         hb_set_t       *glyphs);
```

```rust
pub fn hb_ot_layout_lookups_substitute_closure(
    face: *mut hb_face_t,
    lookups: *const hb_set_t,
    glyphs: *mut hb_set_t,
);
```

As `hb_ot_layout_lookup_substitute_closure()`, but over a whole set of lookups
at once, iterating to a joint fixed point.

**Parameters** — `lookups`: an input set of lookup indices, typically built by
`hb_ot_layout_collect_lookups()`. `glyphs`: in/out seed-and-result set.

**Returns** — nothing.

**Ownership** — both sets are caller-owned; `lookups` is read only, `glyphs` is
added to.

**Notes** — this is the glyph-closure step of subsetting: start from the glyphs
your text needs, close over the lookups the features you keep can reach, and the
result is the glyph set the subset must retain. Since HarfBuzz 1.8.1.

### GPOS — positioning

#### `hb_ot_layout_has_positioning`

```c
hb_bool_t hb_ot_layout_has_positioning (hb_face_t *face);
```

```rust
pub fn hb_ot_layout_has_positioning(face: *mut hb_face_t) -> hb_bool_t;
```

Tests whether the face includes any `GPOS` positioning.

**Returns** — true if the face has `GPOS` data, false otherwise.

**Ownership** — nothing allocated.

**Notes** — a false result does not mean the font cannot kern; legacy `kern`
tables are separate and are reported by `hb_ot_layout_has_kerning()` in
`hb-ot-deprecated.h`. No since-version is stated.

#### `hb_ot_layout_get_size_params`

```c
hb_bool_t
hb_ot_layout_get_size_params (hb_face_t       *face,
                              unsigned int    *design_size       /* OUT.  May be NULL */,
                              unsigned int    *subfamily_id      /* OUT.  May be NULL */,
                              hb_ot_name_id_t *subfamily_name_id /* OUT.  May be NULL */,
                              unsigned int    *range_start       /* OUT.  May be NULL */,
                              unsigned int    *range_end         /* OUT.  May be NULL */);
```

```rust
pub fn hb_ot_layout_get_size_params(
    face: *mut hb_face_t,
    design_size: *mut c_uint,
    subfamily_id: *mut c_uint,
    subfamily_name_id: *mut hb_ot_name_id_t,
    range_start: *mut c_uint,
    range_end: *mut c_uint,
) -> hb_bool_t;
```

Fetches optical-size feature data — the `size` feature from `GPOS`.

**Parameters** — **every** output pointer may be null.

| Parameter | Meaning |
| --- | --- |
| `design_size` | The design size of the face, in decipoints. |
| `subfamily_id` | Identifier of the face within the font subfamily. Zero means the face is not part of a size-range grouping, in which case the remaining fields are meaningless. |
| `subfamily_name_id` | The `name`-table name ID of the face within the subfamily. Resolve it with `hb_ot_name_get_utf8()` and friends. |
| `range_start` | Minimum of the recommended size range, in decipoints. |
| `range_end` | Maximum of the recommended size range, in decipoints. |

**Returns** — true if the data was found, false otherwise.

**Ownership** — nothing allocated; the name ID is an integer, not a string.

**Notes** — `subfamily_id` and the subfamily name pertain **only** to fonts
within a family that differ specifically in their size ranges; other ways of
differentiating fonts within a subfamily are outside the `size` feature's scope.
See the
[`size` feature documentation](https://docs.microsoft.com/en-us/typography/opentype/spec/features_pt#tag-size).
Since HarfBuzz 0.9.10.

#### `hb_ot_layout_lookup_get_optical_bound`

```c
hb_position_t
hb_ot_layout_lookup_get_optical_bound (hb_font_t      *font,
                                       unsigned        lookup_index,
                                       hb_direction_t  direction,
                                       hb_codepoint_t  glyph);
```

```rust
pub fn hb_ot_layout_lookup_get_optical_bound(
    font: *mut hb_font_t,
    lookup_index: c_uint,
    direction: hb_direction_t,
    glyph: hb_codepoint_t,
) -> hb_position_t;
```

Fetches the optical bound of a glyph positioned at the margin of text — the
adjustment a font's optical-margin-alignment (`lfbd`/`rtbd`) lookups would make.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | A **font**, since the result is in scaled units. |
| `lookup_index` | A **`GPOS`** lookup index. |
| `direction` | Which edge of the glyph to query. `HB_DIRECTION_LTR` gives the left edge, `RTL` the right, `TTB` the top, `BTT` the bottom. |
| `glyph` | Glyph ID. |

**Returns** — the adjustment value. **Negative values mean the glyph will stick
out of the margin** — which is the point of optical alignment. Zero means no
adjustment, including when the lookup does not apply to the glyph.

**Ownership** — nothing allocated.

**Notes** — pass an invalid direction and the result is zero. Since HarfBuzz
5.3.0.

### GSUB/GPOS — feature parameters

#### `hb_ot_layout_feature_get_name_ids`

```c
hb_bool_t
hb_ot_layout_feature_get_name_ids (hb_face_t       *face,
                                   hb_tag_t         table_tag,
                                   unsigned int     feature_index,
                                   hb_ot_name_id_t *label_id             /* OUT.  May be NULL */,
                                   hb_ot_name_id_t *tooltip_id           /* OUT.  May be NULL */,
                                   hb_ot_name_id_t *sample_id            /* OUT.  May be NULL */,
                                   unsigned int    *num_named_parameters /* OUT.  May be NULL */,
                                   hb_ot_name_id_t *first_param_id       /* OUT.  May be NULL */);
```

```rust
pub fn hb_ot_layout_feature_get_name_ids(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    feature_index: c_uint,
    label_id: *mut hb_ot_name_id_t,
    tooltip_id: *mut hb_ot_name_id_t,
    sample_id: *mut hb_ot_name_id_t,
    num_named_parameters: *mut c_uint,
    first_param_id: *mut hb_ot_name_id_t,
) -> hb_bool_t;
```

Fetches the `name`-table name IDs recorded in the feature parameters of a
Stylistic Set (`ssXX`) or Character Variant (`cvXX`) feature.

**Parameters** — **every** output pointer may be null.

| Parameter | Meaning |
| --- | --- |
| `label_id` | Name ID of a user-interface label for this feature. |
| `tooltip_id` | Name ID of tooltip text for this feature. |
| `sample_id` | Name ID of sample text illustrating the feature's effect. |
| `num_named_parameters` | Number of named parameters. |
| `first_param_id` | First name ID used to label the feature parameters. Must be zero if `num_named_parameters` is zero. |

**Returns** — true if the data was found, false otherwise — which is the case
for every feature that is not `ssXX` or `cvXX`, and for `ssXX`/`cvXX` features
whose font supplies no `FeatureParams`.

**Ownership** — nothing allocated. Turn a name ID into text with
`hb_ot_name_get_utf8()` / `hb_ot_name_get_utf16()` / `hb_ot_name_get_utf32()`.

**Notes** — `ssXX` features expose only `label_id` in the OpenType spec; the
richer set applies to `cvXX`. Since HarfBuzz 2.0.0.

#### `hb_ot_layout_feature_get_characters`

```c
unsigned int
hb_ot_layout_feature_get_characters (hb_face_t      *face,
                                     hb_tag_t        table_tag,
                                     unsigned int    feature_index,
                                     unsigned int    start_offset,
                                     unsigned int   *char_count /* IN/OUT.  May be NULL */,
                                     hb_codepoint_t *characters /* OUT.     May be NULL */);
```

```rust
pub fn hb_ot_layout_feature_get_characters(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    feature_index: c_uint,
    start_offset: c_uint,
    char_count: *mut c_uint,
    characters: *mut hb_codepoint_t,
) -> c_uint;
```

Fetches the characters declared as having a variant under a Character Variant
(`cvXX`) feature.

**Parameters** — `char_count` in/out, `characters` out; both **nullable**.

**Returns** — the total number of sample characters in the `cvXX` feature.

**Ownership** — nothing allocated.

**Notes** — unlike almost everything else in this header, the values written to
`characters` are genuine **Unicode codepoints**, not glyph IDs. Returns 0 for
features that are not `cvXX`. Since HarfBuzz 2.0.0.

### BASE — font extents and baselines

#### `hb_ot_layout_get_font_extents`

```c
hb_bool_t
hb_ot_layout_get_font_extents (hb_font_t         *font,
                               hb_direction_t     direction,
                               hb_tag_t           script_tag,
                               hb_tag_t           language_tag,
                               hb_font_extents_t *extents);
```

```rust
pub fn hb_ot_layout_get_font_extents(
    font: *mut hb_font_t,
    direction: hb_direction_t,
    script_tag: hb_tag_t,
    language_tag: hb_tag_t,
    extents: *mut hb_font_extents_t,
) -> hb_bool_t;
```

Fetches script- and language-specific font extents, looked up in the `BASE`
table's `MinMax` records.

**Parameters** — `extents`: out, **nullable**.

**Returns** — true if script/language-specific extents were found.

**Ownership** — nothing allocated.

**Notes** — if no such extents exist the font's **default** extents are fetched
instead, so the return value can for the most part be ignored: `extents` is
filled either way. Per-script and per-language extents carry no line-gap value,
and `line_gap` is set to zero in that case. Since HarfBuzz 8.0.0.

#### `hb_ot_layout_get_font_extents2`

```c
hb_bool_t
hb_ot_layout_get_font_extents2 (hb_font_t         *font,
                                hb_direction_t     direction,
                                hb_script_t        script,
                                hb_language_t      language,
                                hb_font_extents_t *extents);
```

```rust
pub fn hb_ot_layout_get_font_extents2(
    font: *mut hb_font_t,
    direction: hb_direction_t,
    script: hb_script_t,
    language: hb_language_t,
    extents: *mut hb_font_extents_t,
) -> hb_bool_t;
```

As `hb_ot_layout_get_font_extents()`, but takes an `hb_script_t` and an
`hb_language_t` instead of OpenType tags — it does the tag conversion for you.

**Parameters** — `language` is **nullable** (`HB_LANGUAGE_INVALID`); `extents`
is nullable.

**Returns** — true if script/language-specific extents were found; same
fill-either-way behaviour as above.

**Ownership** — nothing allocated.

**Notes** — prefer this form in new code unless you already hold OpenType tags.
Since HarfBuzz 8.0.0.

#### `hb_ot_layout_get_horizontal_baseline_tag_for_script`

```c
hb_ot_layout_baseline_tag_t
hb_ot_layout_get_horizontal_baseline_tag_for_script (hb_script_t script);
```

```rust
pub fn hb_ot_layout_get_horizontal_baseline_tag_for_script(
    script: hb_script_t,
) -> hb_ot_layout_baseline_tag_t;
```

Fetches the dominant horizontal baseline tag used by `script`.

**Parameters** — `script`: any script; unknown scripts are not an error.

**Returns** — the dominant baseline tag. In HarfBuzz 14.3.0 the mapping is:

| Result | Scripts |
| --- | --- |
| `HB_OT_LAYOUT_BASELINE_TAG_HANGING` | Bengali, Devanagari, Gujarati, Gurmukhi, Tibetan, Limbu, Syloti Nagri, Phags-pa, Meetei Mayek, Sharada, Takri, Modi, Siddham, Tirhuta, Marchen, Newa, Soyombo, Zanabazar Square, Dogra, Gunjala Gondi, Nandinagari |
| `HB_OT_LAYOUT_BASELINE_TAG_IDEO_FACE_BOTTOM_OR_LEFT` | Hangul, Han, Hiragana, Katakana, Bopomofo, Tangut, Nushu, Khitan Small Script |
| `HB_OT_LAYOUT_BASELINE_TAG_ROMAN` | everything else (the default) |

**Ownership** — nothing allocated.

**Notes** — this is a pure table lookup on the script; it never touches a font.
Use it to pick the `baseline_tag` argument for the baseline getters. Since
HarfBuzz 4.0.0.

#### `hb_ot_layout_get_baseline`

```c
hb_bool_t
hb_ot_layout_get_baseline (hb_font_t                   *font,
                           hb_ot_layout_baseline_tag_t  baseline_tag,
                           hb_direction_t               direction,
                           hb_tag_t                     script_tag,
                           hb_tag_t                     language_tag,
                           hb_position_t               *coord /* OUT.  May be NULL. */);
```

```rust
pub fn hb_ot_layout_get_baseline(
    font: *mut hb_font_t,
    baseline_tag: hb_ot_layout_baseline_tag_t,
    direction: hb_direction_t,
    script_tag: hb_tag_t,
    language_tag: hb_tag_t,
    coord: *mut hb_position_t,
) -> hb_bool_t;
```

Fetches a baseline value from the font's `BASE` table.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `baseline_tag` | Which baseline to fetch. |
| `direction` | Selects the horizontal or vertical axis of the `BASE` table. |
| `script_tag` | OpenType script tag. |
| `language_tag` | OpenType language tag. **Currently unused** by the implementation. |
| `coord` | Out, in scaled font units. **Nullable.** |

**Returns** — true if the baseline value was found in the font. On false,
`coord` is left untouched — so initialise it, or use the `_with_fallback`
variant.

**Ownership** — nothing allocated.

**Notes** — Since HarfBuzz 2.6.0.

#### `hb_ot_layout_get_baseline2`

```c
hb_bool_t
hb_ot_layout_get_baseline2 (hb_font_t                   *font,
                            hb_ot_layout_baseline_tag_t  baseline_tag,
                            hb_direction_t               direction,
                            hb_script_t                  script,
                            hb_language_t                language,
                            hb_position_t               *coord /* OUT.  May be NULL. */);
```

```rust
pub fn hb_ot_layout_get_baseline2(
    font: *mut hb_font_t,
    baseline_tag: hb_ot_layout_baseline_tag_t,
    direction: hb_direction_t,
    script: hb_script_t,
    language: hb_language_t,
    coord: *mut hb_position_t,
) -> hb_bool_t;
```

As `hb_ot_layout_get_baseline()`, but takes an `hb_script_t` and an
`hb_language_t` instead of OpenType tags.

**Parameters** — `language` is **nullable** and currently unused; `coord` is
nullable.

**Returns** — true if the baseline value was found in the font.

**Ownership** — nothing allocated.

**Notes** — Since HarfBuzz 8.0.0.

#### `hb_ot_layout_get_baseline_with_fallback`

```c
void
hb_ot_layout_get_baseline_with_fallback (hb_font_t                   *font,
                                         hb_ot_layout_baseline_tag_t  baseline_tag,
                                         hb_direction_t               direction,
                                         hb_tag_t                     script_tag,
                                         hb_tag_t                     language_tag,
                                         hb_position_t               *coord /* OUT */);
```

```rust
pub fn hb_ot_layout_get_baseline_with_fallback(
    font: *mut hb_font_t,
    baseline_tag: hb_ot_layout_baseline_tag_t,
    direction: hb_direction_t,
    script_tag: hb_tag_t,
    language_tag: hb_tag_t,
    coord: *mut hb_position_t,
);
```

Fetches a baseline value from the font, **synthesizing** it when the font does
not have one.

**Parameters** — `coord`: out. The header does **not** mark it nullable here,
unlike `hb_ot_layout_get_baseline()`; treat it as required.

**Returns** — nothing. There is no failure to report: a value is always written.

**Ownership** — nothing allocated.

**Notes** — this is the function to call for layout. The synthesis rules are
derived from the font's metrics and from the script's dominant baseline (kept in
sync with `hb_ot_layout_get_horizontal_baseline_tag_for_script()`), so results
are plausible rather than authoritative. `language_tag` is currently unused.
Since HarfBuzz 4.0.0.

#### `hb_ot_layout_get_baseline_with_fallback2`

```c
void
hb_ot_layout_get_baseline_with_fallback2 (hb_font_t                   *font,
                                          hb_ot_layout_baseline_tag_t  baseline_tag,
                                          hb_direction_t               direction,
                                          hb_script_t                  script,
                                          hb_language_t                language,
                                          hb_position_t               *coord /* OUT */);
```

```rust
pub fn hb_ot_layout_get_baseline_with_fallback2(
    font: *mut hb_font_t,
    baseline_tag: hb_ot_layout_baseline_tag_t,
    direction: hb_direction_t,
    script: hb_script_t,
    language: hb_language_t,
    coord: *mut hb_position_t,
);
```

As `hb_ot_layout_get_baseline_with_fallback()`, but takes an `hb_script_t` and
an `hb_language_t` instead of OpenType tags.

**Parameters** — `language` is **nullable** and currently unused; `coord` is
required.

**Returns** — nothing.

**Ownership** — nothing allocated.

**Notes** — the most convenient baseline entry point: it needs no tag conversion
and always produces a value. Since HarfBuzz 8.0.0.

### Documented in this section but declared elsewhere

Upstream's `hb-ot-layout` gtk-doc section lists three symbols that
`hb-ot-layout.h` does not declare. They are transcribed in the Rust modules that
own their headers, and are documented here only so that this page covers the
whole section.

#### `hb_ot_layout_script_find_language`

Declared in `hb-ot-deprecated.h`; see the entry under *GSUB/GPOS — languages*
above.

#### `hb_ot_shape_plan_collect_lookups`

Declared in `hb-ot-shape.h`, transcribed in the `ot_shape` Rust module.

```c
void
hb_ot_shape_plan_collect_lookups (hb_shape_plan_t *shape_plan,
                                  hb_tag_t         table_tag,
                                  hb_set_t        *lookup_indexes /* OUT */);
```

Computes the complete set of `GSUB` or `GPOS` lookups that are applicable under
a given shape plan, and adds their indices to `lookup_indexes`. This is the
plan-aware counterpart of `hb_ot_layout_collect_lookups()`: instead of naming
scripts, languages, and features yourself, you let a resolved
`hb_shape_plan_t` — which already knows the segment's script, language,
direction, and user features — decide. Since HarfBuzz 0.9.7.

#### `hb_ot_shape_plan_get_feature_tags`

Declared in `hb-ot-shape.h`, transcribed in the `ot_shape` Rust module.

```c
unsigned int
hb_ot_shape_plan_get_feature_tags (hb_shape_plan_t *shape_plan,
                                   unsigned int     start_offset,
                                   unsigned int    *tag_count /* IN/OUT */,
                                   hb_tag_t        *tags /* OUT */);
```

Fetches the list of OpenType feature tags enabled for a shaping plan, if
possible, using the standard `start_offset` / in-out-count convention. Returns
the total number of feature tags. This is what actually got turned on for a
segment — the union of the shaper's automatic features and the caller's user
features — as opposed to what the font merely offers. Since HarfBuzz 10.3.0.

## Usage

### Descending the GSUB table for one script and language (C)

The canonical walk. Note how each step's output index feeds the next.

```c
#include <hb.h>
#include <hb-ot.h>

static void
dump_features (hb_face_t *face, hb_script_t script, hb_language_t language)
{
  /* 1. Script/language -> OpenType tags, in preference order. */
  hb_tag_t script_tags[HB_OT_MAX_TAGS_PER_SCRIPT];
  hb_tag_t language_tags[HB_OT_MAX_TAGS_PER_LANGUAGE];
  unsigned int script_count   = HB_OT_MAX_TAGS_PER_SCRIPT;
  unsigned int language_count = HB_OT_MAX_TAGS_PER_LANGUAGE;

  hb_ot_tags_from_script_and_language (script, language,
                                       &script_count,   script_tags,
                                       &language_count, language_tags);

  /* 2. Tags -> script index. */
  unsigned int script_index;
  hb_tag_t     chosen_script;
  hb_ot_layout_table_select_script (face, HB_OT_TAG_GSUB,
                                    script_count, script_tags,
                                    &script_index, &chosen_script);
  if (script_index == HB_OT_LAYOUT_NO_SCRIPT_INDEX)
    return;                                   /* nothing at all for this script */

  /* 3. Tags -> language index.  A false return still leaves a usable
   *    default-language index behind, so we do not bail out here. */
  unsigned int language_index;
  hb_ot_layout_script_select_language (face, HB_OT_TAG_GSUB, script_index,
                                       language_count, language_tags,
                                       &language_index);

  /* 4. Language system -> feature tags.  Ask for the count first. */
  unsigned int total =
    hb_ot_layout_language_get_feature_tags (face, HB_OT_TAG_GSUB,
                                            script_index, language_index,
                                            0, NULL, NULL);

  hb_tag_t *tags = malloc (total * sizeof (hb_tag_t));
  unsigned int count = total;
  hb_ot_layout_language_get_feature_tags (face, HB_OT_TAG_GSUB,
                                          script_index, language_index,
                                          0, &count, tags);

  for (unsigned int i = 0; i < count; i++)
  {
    char buf[5] = {0};
    hb_tag_to_string (tags[i], buf);          /* NOT NUL-terminated by itself */
    printf ("%s\n", buf);
  }
  free (tags);
}
```

### Paging a large enumeration (C)

Every enumerator takes `start_offset`, so you never need to allocate the whole
list:

```c
unsigned int offset = 0;
for (;;)
{
  hb_tag_t buf[32];
  unsigned int count = 32;
  unsigned int total = hb_ot_layout_table_get_script_tags (face, HB_OT_TAG_GPOS,
                                                           offset, &count, buf);
  for (unsigned int i = 0; i < count; i++)
    handle (buf[i]);

  offset += count;
  if (offset >= total || count == 0)
    break;
}
```

### The same descent in Rust

```rust
use core::ffi::c_uint;
use core::ptr;
use harfbuzz_sys::*;

/// Returns the `GSUB` feature tags a face offers for `script` / `language`.
unsafe fn gsub_feature_tags(
    face: *mut hb_face_t,
    script: hb_script_t,
    language: hb_language_t,
) -> Vec<hb_tag_t> {
    // 1. script/language -> OpenType tags
    let mut script_tags = [HB_TAG_NONE; HB_OT_MAX_TAGS_PER_SCRIPT as usize];
    let mut language_tags = [HB_TAG_NONE; HB_OT_MAX_TAGS_PER_LANGUAGE as usize];
    let mut script_count = script_tags.len() as c_uint;
    let mut language_count = language_tags.len() as c_uint;

    unsafe {
        hb_ot_tags_from_script_and_language(
            script,
            language,
            &mut script_count,
            script_tags.as_mut_ptr(),
            &mut language_count,
            language_tags.as_mut_ptr(),
        );
    }

    // 2. tags -> script index
    let mut script_index: c_uint = HB_OT_LAYOUT_NO_SCRIPT_INDEX;
    unsafe {
        hb_ot_layout_table_select_script(
            face,
            HB_OT_TAG_GSUB,
            script_count,
            script_tags.as_ptr(),
            &mut script_index,
            ptr::null_mut(),
        );
    }
    if script_index == HB_OT_LAYOUT_NO_SCRIPT_INDEX {
        return Vec::new();
    }

    // 3. tags -> language index (a `false` return still yields a usable index)
    let mut language_index: c_uint = HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX;
    unsafe {
        hb_ot_layout_script_select_language(
            face,
            HB_OT_TAG_GSUB,
            script_index,
            language_count,
            language_tags.as_ptr(),
            &mut language_index,
        );
    }

    // 4. size the buffer, then fill it
    let total = unsafe {
        hb_ot_layout_language_get_feature_tags(
            face,
            HB_OT_TAG_GSUB,
            script_index,
            language_index,
            0,
            ptr::null_mut(),
            ptr::null_mut(),
        )
    };

    let mut tags = vec![HB_TAG_NONE; total as usize];
    let mut count = total;
    unsafe {
        hb_ot_layout_language_get_feature_tags(
            face,
            HB_OT_TAG_GSUB,
            script_index,
            language_index,
            0,
            &mut count,
            tags.as_mut_ptr(),
        );
    }
    tags.truncate(count as usize);
    tags
}
```

### Listing the alternates a `salt`/`aalt` feature offers for a glyph (Rust)

The one-based position in the alternates array is what goes in
`hb_feature_t::value`.

```rust
unsafe fn alternates_for(
    face: *mut hb_face_t,
    lookup_index: c_uint,
    glyph: hb_codepoint_t,
) -> Vec<hb_codepoint_t> {
    let total = unsafe {
        hb_ot_layout_lookup_get_glyph_alternates(
            face,
            lookup_index,
            glyph,
            0,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        )
    };

    let mut out = vec![0u32; total as usize];
    let mut count = total;
    unsafe {
        hb_ot_layout_lookup_get_glyph_alternates(
            face,
            lookup_index,
            glyph,
            0,
            &mut count,
            out.as_mut_ptr(),
        );
    }
    out.truncate(count as usize);
    out
}
```

### Glyph closure for a subsetter (C)

Collect the lookups a set of features can reach, then close the glyph set over
them:

```c
hb_set_t *lookups = hb_set_create ();
hb_tag_t  keep[] = { HB_TAG ('l','i','g','a'), HB_TAG ('k','e','r','n'), HB_TAG_NONE };

hb_ot_layout_collect_lookups (face, HB_OT_TAG_GSUB,
                              NULL /* all scripts */,
                              NULL /* all languages */,
                              keep,
                              lookups);

hb_set_t *glyphs = hb_set_create ();
/* seed with the glyphs your text needs … */
hb_set_add (glyphs, my_glyph);

hb_ot_layout_lookups_substitute_closure (face, lookups, glyphs);
/* `glyphs` now also contains everything those lookups can produce. */

hb_set_destroy (lookups);
hb_set_destroy (glyphs);
```

### Getting a usable baseline (C)

```c
hb_ot_layout_baseline_tag_t tag =
  hb_ot_layout_get_horizontal_baseline_tag_for_script (HB_SCRIPT_DEVANAGARI);

hb_position_t coord;
hb_ot_layout_get_baseline_with_fallback2 (font, tag, HB_DIRECTION_LTR,
                                          HB_SCRIPT_DEVANAGARI,
                                          HB_LANGUAGE_INVALID,
                                          &coord);
/* `coord` is always set — the fallback variant synthesizes when the font
 * has no BASE entry.  Use hb_ot_layout_get_baseline2() instead if you need
 * to know whether the value came from the font. */
```

### Reading a font's optical-size range (C)

```c
unsigned int    design_size, subfamily_id, range_start, range_end;
hb_ot_name_id_t subfamily_name_id;

if (hb_ot_layout_get_size_params (face, &design_size, &subfamily_id,
                                  &subfamily_name_id, &range_start, &range_end)
    && subfamily_id != 0)
{
  char text[128];
  unsigned int len = sizeof (text);
  hb_ot_name_get_utf8 (face, subfamily_name_id, HB_LANGUAGE_INVALID, &len, text);
  printf ("%s: %g pt, recommended %g–%g pt\n",
          text, design_size / 10.0, range_start / 10.0, range_end / 10.0);
}
```

## Pitfalls

### `hb_ot_layout_table_find_script()` returns false but still writes an index

Its `false` return means "your tag was not found", not "nothing was written".
The function has already tried `DFLT`, then `dflt`, then `latn`, and written
whichever it found. Code that does

```c
if (!hb_ot_layout_table_find_script (face, HB_OT_TAG_GSUB, tag, &idx))
  return;   /* WRONG — you just threw away a perfectly good fallback */
```

silently loses the fallback. The same asymmetry exists in
`hb_ot_layout_table_select_script()`, which returns `false` when it fell back —
so test `script_index != HB_OT_LAYOUT_NO_SCRIPT_INDEX` rather than the boolean
when what you want is "can I proceed?".

### `hb_ot_layout_script_select_language()` returning false is normal

A `false` return sets `language_index` to
`HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX`, which is a **valid index** naming the
script's `DefaultLangSys`. Most fonts have no per-language systems at all, so
false is the common case for well-formed input. Bailing out on false means you
will find no features in the majority of fonts.

### The three `0xFFFF` sentinels are not interchangeable

`HB_OT_LAYOUT_NO_SCRIPT_INDEX`, `HB_OT_LAYOUT_NO_FEATURE_INDEX`, and
`HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX` all equal `0xFFFF`, so the compiler will
not catch you comparing a feature index against the script sentinel. Worse, the
language sentinel is not an error value at all — it is a *usable* index — while
the other two are pure failure markers.

### An invalid `table_tag` fails silently

`table_tag` is validated by a `switch` that maps `HB_OT_TAG_GSUB` and
`HB_OT_TAG_GPOS` to real tables and **everything else to HarfBuzz's null
table object**. Passing `HB_OT_TAG_GDEF`, a typo, or an uninitialised variable
therefore yields zero scripts, zero features, and zero lookups — indistinguishable
from a font that genuinely has none. There is no error return to check.

### Output sets and maps are added to, never cleared

`hb_ot_layout_get_glyphs_in_class()`, `hb_ot_layout_collect_features()`,
`hb_ot_layout_collect_lookups()`, `hb_ot_layout_lookup_collect_glyphs()`, both
substitute-closure functions, and `hb_ot_layout_collect_features_map()` all
*union* their results into the object you pass. Reusing one set across a loop
accumulates every iteration's results. Call `hb_set_clear()` / `hb_map_clear()`
between uses, and check `hb_set_allocation_successful()` afterwards — these
functions have no way to report an out-of-memory condition.

### The return value is the total, `*count` is what you got

Every enumerator returns the grand total and separately reports, through
`*count`, how many elements it actually wrote. Sizing a loop with the return
value while indexing a buffer filled to `*count` reads uninitialised memory. The
safe pattern is: call once with `(0, NULL, NULL)` to get the total, allocate,
call again, then trust `*count`.

### `hb_codepoint_t` means glyph ID almost everywhere here

`hb_ot_layout_get_glyph_class()`, `hb_ot_layout_get_attach_points()`,
`hb_ot_layout_get_ligature_carets()`, `hb_ot_layout_lookup_would_substitute()`,
`hb_ot_layout_lookup_get_glyph_alternates()`, and
`hb_ot_layout_lookup_get_optical_bound()` all take **glyph IDs** despite the
Unicode-flavoured type name. The single exception is
`hb_ot_layout_feature_get_characters()`, which really does return Unicode
codepoints. Mixing the two produces plausible-looking nonsense rather than an
error.

### `_with_fallback` variants have no return value on purpose

`hb_ot_layout_get_baseline()` writes `coord` only on success and leaves it
untouched on failure — so an uninitialised `hb_position_t` stays uninitialised.
`hb_ot_layout_get_baseline_with_fallback()` always writes, which is why it
returns `void`. Choose deliberately: the fallback variants give you a usable
number, the plain ones tell you whether the font actually said anything.

### Indices are per-table, per-face, and not stable across faces

A script index obtained from `GSUB` is meaningless against `GPOS`, and both are
meaningless against a different face — including a different `hb_face_t` built
from the same file. Cache tags, not indices.

### `hb_ot_layout_lookup_get_optical_bound()` reads `GPOS`, not `GSUB`

The `lookup_index` argument is unconditionally interpreted as a `GPOS` lookup
index; there is no `table_tag` parameter. Passing a `GSUB` index returns a
meaningless number rather than failing.

### `hb_ot_layout_lookup_collect_glyph_alternates()` needs a pre-seeded map

Unlike everything else in this header, `alternate_count` is an *input* as well
as an output: it must already contain the glyph IDs you are interested in as
keys. Handing it an empty map collects nothing and looks like a font that has no
alternates.

### `hb_ot_tag_to_language()` can return null

It is the only tag-conversion function here that has a documented failure value.
`HB_LANGUAGE_INVALID` is a null pointer, and passing it on to
`hb_language_to_string()` is not safe.

### Macros

`hb-ot-layout.h` defines twelve object-like macros, all of which are transcribed
as `pub const`. Five of them (`HB_OT_TAG_BASE`, `HB_OT_TAG_GDEF`,
`HB_OT_TAG_GSUB`, `HB_OT_TAG_GPOS`, `HB_OT_TAG_JSTF`) plus
`HB_OT_TAG_DEFAULT_SCRIPT` and `HB_OT_TAG_DEFAULT_LANGUAGE` are `HB_TAG(...)`
invocations, and are reproduced with the crate's `const fn HB_TAG`. The header
defines no function-like macros, so nothing was skipped in the transcription.
