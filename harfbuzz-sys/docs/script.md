# Scripts

Header: `hb-script-list.h` — Rust module: `harfbuzz_sys::script` (glob
re-exported at the crate root). Upstream has **no** `hb-script-list` gtk-doc
section; the script API is documented as part of the `hb-common` section (see
[Section coverage](#section-coverage) at the end of this page).

## Overview

A **script** in HarfBuzz is an `hb_script_t`: a 32-bit value whose bit pattern
*is* the four-character [ISO 15924](https://unicode.org/iso15924/) script code,
packed one ASCII byte per octet, most significant byte first. `HB_SCRIPT_LATIN`
is not an arbitrary ordinal — it is literally `'L'<<24 | 'a'<<16 | 't'<<8 |
'n'`, i.e. `0x4C61746E`. That identity is the whole design: converting between
an `hb_script_t` and an `hb_tag_t` is a reinterpretation with no lookup table,
and any four-byte tag can be stored in an `hb_script_t` even if HarfBuzz has
never heard of it.

Scripts enter HarfBuzz from two directions. Character properties supply one:
`hb_unicode_script()` maps a code point to its Unicode `Script` (`sc`)
property, which is what `hb_buffer_guess_segment_properties()` uses to pick a
buffer's script from its contents. Client code supplies the other: you can call
`hb_buffer_set_script()` directly when you already know what you are shaping,
which is the normal thing to do when a higher layer (an itemizer, a rich-text
engine, a `lang` attribute) has already segmented the run.

Once a buffer has a script, HarfBuzz uses it for three separate purposes.
First, it selects the **shaper**: the complex-script machinery for Arabic,
Indic, Khmer, Myanmar, Hangul, Hebrew, Thai, and the Universal Shaping Engine
is dispatched on script, so an Arabic run gets joining and an Indic run gets
reordering. Second, it is mapped to an **OpenType script tag** for `GSUB`/`GPOS`
lookup selection — a *different* four-character namespace (`deva`/`dev2`,
`latn`, `arab`, …) reached through `hb_ot_tags_from_script_and_language()` and
`hb_ot_tag_to_script()`, which are documented with the OpenType layout API, not
here. Third, when the buffer's direction has not been set, the script picks a
default **horizontal direction** through `hb_script_get_horizontal_direction()`.

Three of the values are not scripts at all but Unicode's pseudo-scripts.
`HB_SCRIPT_COMMON` (`Zyyy`) covers characters that belong to no single script —
spaces, digits, most punctuation. `HB_SCRIPT_INHERITED` (`Zinh`) covers
characters that take their script from what precedes them, chiefly combining
marks. `HB_SCRIPT_UNKNOWN` (`Zzzz`) covers unassigned, private-use,
noncharacter, and surrogate code points. HarfBuzz adds a fourth of its own,
`HB_SCRIPT_MATH` (`Zmth`), for mathematical notation. And `HB_SCRIPT_INVALID`
is numerically zero (`HB_TAG_NONE`) and means "no script set" — it is what a
freshly created buffer reports.

The type is deliberately open. The C enumeration ends with two private
sentinels, `_HB_SCRIPT_MAX_VALUE` and `_HB_SCRIPT_MAX_VALUE_SIGNED`, both equal
to `HB_TAG_MAX_SIGNED` (`0x7FFFFFFF`), whose only job is to pin the enum's
underlying type wide enough that *any* `hb_tag_t` can be stored in an
`hb_script_t` without undefined behaviour. Consequently you must never treat
`hb_script_t` as a closed set: text and font data can legitimately produce a tag
that has no `HB_SCRIPT_*` constant, and a newer HarfBuzz will produce constants
this table does not list.

## Types

### `hb_script_t`

```c
typedef enum {
  HB_SCRIPT_COMMON = HB_TAG ('Z','y','y','y'),
  /* ... 176 more ... */
  HB_SCRIPT_INVALID = HB_TAG_NONE,
  /*< private >*/
  _HB_SCRIPT_MAX_VALUE        = HB_TAG_MAX_SIGNED, /*< skip >*/
  _HB_SCRIPT_MAX_VALUE_SIGNED = HB_TAG_MAX_SIGNED  /*< skip >*/
} hb_script_t;
```

```rust
pub type hb_script_t = c_int;
```

Data type for scripts. Each value is an `hb_tag_t` holding the four-letter code
defined by ISO 15924; see also the Script (`sc`) property of the Unicode
Character Database.

In Rust this is a plain `c_int` alias plus 177 constants, not a Rust `enum`.
That is forced by the C definition and is also the correct modelling: because
the two private sentinels equal `HB_TAG_MAX_SIGNED`, the C enum's underlying
type is a **signed** 32-bit integer, and *every* `hb_tag_t` bit pattern is a
valid `hb_script_t`. A Rust `enum` would make an out-of-list tag instant
undefined behaviour.

Two consequences of the signedness are worth spelling out. Tags whose first
byte is `>= 0x80` become negative when stored in `hb_script_t`; HarfBuzz's own
constants never do, because ISO 15924 codes are ASCII, but a hostile or
corrupted font table could. And in Rust, `HB_TAG(...)` returns `hb_tag_t`
(`u32`), so every constant in `script.rs` is written with an explicit
`as hb_script_t` cast.

#### Tag encoding

`HB_TAG(c1,c2,c3,c4)` packs four bytes big-endian:

```c
#define HB_TAG(c1,c2,c3,c4) ((hb_tag_t)((((uint32_t)(c1)&0xFF)<<24) | \
                                        (((uint32_t)(c2)&0xFF)<<16) | \
                                        (((uint32_t)(c3)&0xFF)<< 8) | \
                                         ((uint32_t)(c4)&0xFF)))
```

```rust
pub const fn HB_TAG(c1: u8, c2: u8, c3: u8, c4: u8) -> hb_tag_t;
```

So `HB_SCRIPT_HAN` is `HB_TAG('H','a','n','i')` = `0x48616E69`, and
`HB_UNTAG(tag)` (or `hb_tag_to_string`) recovers the four characters. ISO 15924
codes are conventionally title case: one uppercase ASCII letter followed by
three lowercase ones.

`HB_SCRIPT_INVALID` is `HB_TAG_NONE`, i.e. `HB_TAG(0,0,0,0)` = `0`. It is the
only constant on this page that is not a printable four-character code, and it
is the one value you can meaningfully test against zero.

### `hb_direction_t` (referenced)

`hb_script_get_horizontal_direction()` returns an `hb_direction_t`, defined in
`hb-common.h` and documented with the common types. The values that can come
back from it are:

| Constant | Value | Meaning here |
| --- | ---: | --- |
| `HB_DIRECTION_INVALID` | 0 | The script is written in both directions; HarfBuzz declines to choose. |
| `HB_DIRECTION_LTR` | 4 | Left-to-right, and also the answer for every unrecognised tag. |
| `HB_DIRECTION_RTL` | 5 | Right-to-left. |

`HB_DIRECTION_TTB` (6) and `HB_DIRECTION_BTT` (7) exist but are never returned
by this function — the name says *horizontal*.

## Functions

The four functions that operate on `hb_script_t` are declared in `hb-common.h`,
not in `hb-script-list.h`; `hb-script-list.h` contains only the enum. In this
crate all four live in `harfbuzz-sys/src/script.rs` alongside the constants.

### Tag conversion

#### `hb_script_from_iso15924_tag`

```c
hb_script_t hb_script_from_iso15924_tag (hb_tag_t tag);
```

```rust
pub fn hb_script_from_iso15924_tag(tag: hb_tag_t) -> hb_script_t;
```

Converts an ISO 15924 script tag to the corresponding `hb_script_t`.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `tag` | Any four-byte tag. Not a pointer, so nullability does not apply. `HB_TAG_NONE` is accepted and handled specially. |

**Returns** — the matching script. This is not a plain cast; the implementation
does four things in order:

1. `HB_TAG_NONE` maps to `HB_SCRIPT_INVALID`.
2. The tag is case-folded to "one capital letter followed by three small
   letters" by `tag = (tag & 0xDFDFDFDF) | 0x00202020`. Matching is therefore
   **case-insensitive**: `LATN`, `latn`, and `Latn` all give `HB_SCRIPT_LATIN`.
3. A fixed alias table folds historic and variant codes onto their modern
   script:

   | Input tag | Result |
   | --- | --- |
   | `Qaai` | `HB_SCRIPT_INHERITED` (the old private-area code, still used by ICU) |
   | `Qaac` | `HB_SCRIPT_COPTIC` |
   | `Aran` | `HB_SCRIPT_ARABIC` (Nastaliq variant) |
   | `Cyrs` | `HB_SCRIPT_CYRILLIC` (Old Church Slavonic variant) |
   | `Geok` | `HB_SCRIPT_GEORGIAN` (Khutsuri) |
   | `Hans` | `HB_SCRIPT_HAN` (Simplified) |
   | `Hant` | `HB_SCRIPT_HAN` (Traditional) |
   | `Jamo` | `HB_SCRIPT_HANGUL` |
   | `Latf` | `HB_SCRIPT_LATIN` (Fraktur) |
   | `Latg` | `HB_SCRIPT_LATIN` (Gaelic) |
   | `Syre` | `HB_SCRIPT_SYRIAC` (Estrangelo) |
   | `Syrj` | `HB_SCRIPT_SYRIAC` (Western) |
   | `Syrn` | `HB_SCRIPT_SYRIAC` (Eastern) |

4. Otherwise, if the (case-folded) tag *looks* like a script code — the test is
   `(tag & 0xE0E0E0E0) == 0x40606060`, i.e. one uppercase ASCII letter followed
   by three lowercase ones — it is passed through unchanged, even if HarfBuzz
   has no constant for it. Anything else returns `HB_SCRIPT_UNKNOWN`.

**Ownership** — none; scalars in, scalar out.

**Notes** — Since HarfBuzz 0.9.2. Pure function, no allocation, thread-safe.
Step 4 is why a future ISO 15924 code works without a HarfBuzz upgrade, and
also why a garbage-but-well-formed tag will not be rejected.

#### `hb_script_from_string`

```c
hb_script_t hb_script_from_string (const char *str, int len);
```

```rust
pub fn hb_script_from_string(str_: *const c_char, len: c_int) -> hb_script_t;
```

Converts a string holding an ISO 15924 tag to the corresponding script.
Shorthand for `hb_tag_from_string()` followed by `hb_script_from_iso15924_tag()`,
so all the folding rules above apply.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `str` | The tag text. Upstream annotates it `(array length=len) (element-type uint8_t)`; nullability is **unspecified** in the header, and `hb_tag_from_string` dereferences it, so treat null as forbidden — pass `""` with `len = 0` instead. |
| `len` | Byte length of `str`, or `-1` when `str` is NUL-terminated. |

**Returns** — the script. `hb_tag_from_string` pads short input with spaces and
truncates anything past four bytes, so `"Latn"`, `"Lat"`, and `"Latnxyz"` do not
all mean the same thing: `"Lat"` becomes the tag `Lat ` (trailing space), which
fails the "looks like a script code" test and yields `HB_SCRIPT_UNKNOWN`.
Empty input yields `HB_TAG_NONE`, hence `HB_SCRIPT_INVALID`.

**Ownership** — the string is read, not retained.

**Notes** — Since HarfBuzz 0.9.2.

#### `hb_script_to_iso15924_tag`

```c
hb_tag_t hb_script_to_iso15924_tag (hb_script_t script);
```

```rust
pub fn hb_script_to_iso15924_tag(script: hb_script_t) -> hb_tag_t;
```

Converts a script to the corresponding ISO 15924 script tag. The whole
implementation is `return (hb_tag_t) script;` — a reinterpretation, nothing
more.

**Returns** — the tag. It never fails and has no error value. Note the
asymmetry with `hb_script_from_iso15924_tag`: the round trip
tag → script → tag is the identity only for tags that survive the folding
step. `Hans` goes in and `Hani` comes out; `qaai` goes in and `Zinh` comes out.
The round trip script → tag → script *is* the identity for every script.

**Ownership** — none.

**Notes** — Since HarfBuzz 0.9.2. Pair it with `hb_tag_to_string` to print a
script; that function writes exactly four bytes and does **not** NUL-terminate,
so the buffer must be at least four bytes and you must add your own terminator.

### Direction

#### `hb_script_get_horizontal_direction`

```c
hb_direction_t hb_script_get_horizontal_direction (hb_script_t script);
```

```rust
pub fn hb_script_get_horizontal_direction(script: hb_script_t) -> hb_direction_t;
```

Fetches the direction a script is written in when set horizontally.

**Returns**

- `HB_DIRECTION_RTL` for right-to-left scripts.
- `HB_DIRECTION_LTR` for left-to-right scripts — **and** for every script the
  function does not recognise, including `HB_SCRIPT_INVALID`, `HB_SCRIPT_COMMON`,
  and any tag added to ISO 15924 after your HarfBuzz build. There is no
  "unknown" answer; LTR is the fallback.
- `HB_DIRECTION_INVALID` for the four scripts that are attested in both
  directions and where HarfBuzz refuses to guess: `HB_SCRIPT_OLD_HUNGARIAN`,
  `HB_SCRIPT_OLD_ITALIC`, `HB_SCRIPT_RUNIC`, and `HB_SCRIPT_TIFINAGH`.

The complete right-to-left set, as HarfBuzz 14.3.0 has it:

`HB_SCRIPT_ADLAM`, `HB_SCRIPT_ARABIC`, `HB_SCRIPT_AVESTAN`,
`HB_SCRIPT_CHORASMIAN`, `HB_SCRIPT_CYPRIOT`, `HB_SCRIPT_ELYMAIC`,
`HB_SCRIPT_GARAY`, `HB_SCRIPT_HANIFI_ROHINGYA`, `HB_SCRIPT_HATRAN`,
`HB_SCRIPT_HEBREW`, `HB_SCRIPT_IMPERIAL_ARAMAIC`,
`HB_SCRIPT_INSCRIPTIONAL_PAHLAVI`, `HB_SCRIPT_INSCRIPTIONAL_PARTHIAN`,
`HB_SCRIPT_KHAROSHTHI`, `HB_SCRIPT_LYDIAN`, `HB_SCRIPT_MANDAIC`,
`HB_SCRIPT_MANICHAEAN`, `HB_SCRIPT_MENDE_KIKAKUI`,
`HB_SCRIPT_MEROITIC_CURSIVE`, `HB_SCRIPT_MEROITIC_HIEROGLYPHS`,
`HB_SCRIPT_NABATAEAN`, `HB_SCRIPT_NKO`, `HB_SCRIPT_OLD_NORTH_ARABIAN`,
`HB_SCRIPT_OLD_SOGDIAN`, `HB_SCRIPT_OLD_SOUTH_ARABIAN`,
`HB_SCRIPT_OLD_TURKIC`, `HB_SCRIPT_OLD_UYGHUR`, `HB_SCRIPT_PALMYRENE`,
`HB_SCRIPT_PHOENICIAN`, `HB_SCRIPT_PSALTER_PAHLAVI`, `HB_SCRIPT_SAMARITAN`,
`HB_SCRIPT_SIDETIC`, `HB_SCRIPT_SOGDIAN`, `HB_SCRIPT_SYRIAC`,
`HB_SCRIPT_THAANA`, `HB_SCRIPT_YEZIDI`.

**Ownership** — none.

**Notes** — Since HarfBuzz 0.9.2. Pure function, thread-safe. This is what
`hb_buffer_guess_segment_properties()` calls when a buffer has a script but no
direction; the `HB_DIRECTION_INVALID` cases therefore leave such a buffer with
an unset direction, which is the point — you are expected to decide.

## The `HB_SCRIPT_*` constants

Every constant below is defined as `HB_TAG(...)` of its four-character ISO 15924
code, so the **Value** column is that code's ASCII bytes packed big-endian and
is fully determined by the tag.

The **Unicode** column is the Unicode version that added the script, taken from
the trailing comment on each enumerator in `hb-script-list.h`. The **Since**
column is the HarfBuzz version, taken from the `Since:` annotations in that
file's gtk-doc block. Upstream only started annotating a HarfBuzz `Since:` with
the 0.9.30 batch; every constant marked `—` was present in the original
enumeration and carries no upstream `Since:` line, so no HarfBuzz version is
claimed for it here rather than one being invented.

177 constants, matching the 177 enumerators in `hb-script-list.h`, the 177
`@HB_SCRIPT_*` entries in its gtk-doc block, and the 177 `pub const`s in
`harfbuzz-sys/src/script.rs`. The order below is the header's order, which is
by Unicode version rather than alphabetical.

| Constant | Tag | Value | Script | Unicode | Since (HarfBuzz) |
| --- | --- | --- | --- | ---: | ---: |
| `HB_SCRIPT_COMMON` | `Zyyy` | `0x5A797979` | Common | 1.1 | — |
| `HB_SCRIPT_INHERITED` | `Zinh` | `0x5A696E68` | Inherited | 1.1 | — |
| `HB_SCRIPT_UNKNOWN` | `Zzzz` | `0x5A7A7A7A` | Unknown | 5.0 | — |
| `HB_SCRIPT_ARABIC` | `Arab` | `0x41726162` | Arabic | 1.1 | — |
| `HB_SCRIPT_ARMENIAN` | `Armn` | `0x41726D6E` | Armenian | 1.1 | — |
| `HB_SCRIPT_BENGALI` | `Beng` | `0x42656E67` | Bengali | 1.1 | — |
| `HB_SCRIPT_CYRILLIC` | `Cyrl` | `0x4379726C` | Cyrillic | 1.1 | — |
| `HB_SCRIPT_DEVANAGARI` | `Deva` | `0x44657661` | Devanagari | 1.1 | — |
| `HB_SCRIPT_GEORGIAN` | `Geor` | `0x47656F72` | Georgian | 1.1 | — |
| `HB_SCRIPT_GREEK` | `Grek` | `0x4772656B` | Greek | 1.1 | — |
| `HB_SCRIPT_GUJARATI` | `Gujr` | `0x47756A72` | Gujarati | 1.1 | — |
| `HB_SCRIPT_GURMUKHI` | `Guru` | `0x47757275` | Gurmukhi | 1.1 | — |
| `HB_SCRIPT_HANGUL` | `Hang` | `0x48616E67` | Hangul | 1.1 | — |
| `HB_SCRIPT_HAN` | `Hani` | `0x48616E69` | Han | 1.1 | — |
| `HB_SCRIPT_HEBREW` | `Hebr` | `0x48656272` | Hebrew | 1.1 | — |
| `HB_SCRIPT_HIRAGANA` | `Hira` | `0x48697261` | Hiragana | 1.1 | — |
| `HB_SCRIPT_KANNADA` | `Knda` | `0x4B6E6461` | Kannada | 1.1 | — |
| `HB_SCRIPT_KATAKANA` | `Kana` | `0x4B616E61` | Katakana | 1.1 | — |
| `HB_SCRIPT_LAO` | `Laoo` | `0x4C616F6F` | Lao | 1.1 | — |
| `HB_SCRIPT_LATIN` | `Latn` | `0x4C61746E` | Latin | 1.1 | — |
| `HB_SCRIPT_MALAYALAM` | `Mlym` | `0x4D6C796D` | Malayalam | 1.1 | — |
| `HB_SCRIPT_ORIYA` | `Orya` | `0x4F727961` | Oriya | 1.1 | — |
| `HB_SCRIPT_TAMIL` | `Taml` | `0x54616D6C` | Tamil | 1.1 | — |
| `HB_SCRIPT_TELUGU` | `Telu` | `0x54656C75` | Telugu | 1.1 | — |
| `HB_SCRIPT_THAI` | `Thai` | `0x54686169` | Thai | 1.1 | — |
| `HB_SCRIPT_TIBETAN` | `Tibt` | `0x54696274` | Tibetan | 2.0 | — |
| `HB_SCRIPT_BOPOMOFO` | `Bopo` | `0x426F706F` | Bopomofo | 3.0 | — |
| `HB_SCRIPT_BRAILLE` | `Brai` | `0x42726169` | Braille | 3.0 | — |
| `HB_SCRIPT_CANADIAN_SYLLABICS` | `Cans` | `0x43616E73` | Unified Canadian Aboriginal Syllabics | 3.0 | — |
| `HB_SCRIPT_CHEROKEE` | `Cher` | `0x43686572` | Cherokee | 3.0 | — |
| `HB_SCRIPT_ETHIOPIC` | `Ethi` | `0x45746869` | Ethiopic | 3.0 | — |
| `HB_SCRIPT_KHMER` | `Khmr` | `0x4B686D72` | Khmer | 3.0 | — |
| `HB_SCRIPT_MONGOLIAN` | `Mong` | `0x4D6F6E67` | Mongolian | 3.0 | — |
| `HB_SCRIPT_MYANMAR` | `Mymr` | `0x4D796D72` | Myanmar | 3.0 | — |
| `HB_SCRIPT_OGHAM` | `Ogam` | `0x4F67616D` | Ogham | 3.0 | — |
| `HB_SCRIPT_RUNIC` | `Runr` | `0x52756E72` | Runic | 3.0 | — |
| `HB_SCRIPT_SINHALA` | `Sinh` | `0x53696E68` | Sinhala | 3.0 | — |
| `HB_SCRIPT_SYRIAC` | `Syrc` | `0x53797263` | Syriac | 3.0 | — |
| `HB_SCRIPT_THAANA` | `Thaa` | `0x54686161` | Thaana | 3.0 | — |
| `HB_SCRIPT_YI` | `Yiii` | `0x59696969` | Yi | 3.0 | — |
| `HB_SCRIPT_DESERET` | `Dsrt` | `0x44737274` | Deseret | 3.1 | — |
| `HB_SCRIPT_GOTHIC` | `Goth` | `0x476F7468` | Gothic | 3.1 | — |
| `HB_SCRIPT_OLD_ITALIC` | `Ital` | `0x4974616C` | Old Italic | 3.1 | — |
| `HB_SCRIPT_BUHID` | `Buhd` | `0x42756864` | Buhid | 3.2 | — |
| `HB_SCRIPT_HANUNOO` | `Hano` | `0x48616E6F` | Hanunoo | 3.2 | — |
| `HB_SCRIPT_TAGALOG` | `Tglg` | `0x54676C67` | Tagalog | 3.2 | — |
| `HB_SCRIPT_TAGBANWA` | `Tagb` | `0x54616762` | Tagbanwa | 3.2 | — |
| `HB_SCRIPT_CYPRIOT` | `Cprt` | `0x43707274` | Cypriot | 4.0 | — |
| `HB_SCRIPT_LIMBU` | `Limb` | `0x4C696D62` | Limbu | 4.0 | — |
| `HB_SCRIPT_LINEAR_B` | `Linb` | `0x4C696E62` | Linear B | 4.0 | — |
| `HB_SCRIPT_OSMANYA` | `Osma` | `0x4F736D61` | Osmanya | 4.0 | — |
| `HB_SCRIPT_SHAVIAN` | `Shaw` | `0x53686177` | Shavian | 4.0 | — |
| `HB_SCRIPT_TAI_LE` | `Tale` | `0x54616C65` | Tai Le | 4.0 | — |
| `HB_SCRIPT_UGARITIC` | `Ugar` | `0x55676172` | Ugaritic | 4.0 | — |
| `HB_SCRIPT_BUGINESE` | `Bugi` | `0x42756769` | Buginese | 4.1 | — |
| `HB_SCRIPT_COPTIC` | `Copt` | `0x436F7074` | Coptic | 4.1 | — |
| `HB_SCRIPT_GLAGOLITIC` | `Glag` | `0x476C6167` | Glagolitic | 4.1 | — |
| `HB_SCRIPT_KHAROSHTHI` | `Khar` | `0x4B686172` | Kharoshthi | 4.1 | — |
| `HB_SCRIPT_NEW_TAI_LUE` | `Talu` | `0x54616C75` | New Tai Lue | 4.1 | — |
| `HB_SCRIPT_OLD_PERSIAN` | `Xpeo` | `0x5870656F` | Old Persian | 4.1 | — |
| `HB_SCRIPT_SYLOTI_NAGRI` | `Sylo` | `0x53796C6F` | Syloti Nagri | 4.1 | — |
| `HB_SCRIPT_TIFINAGH` | `Tfng` | `0x54666E67` | Tifinagh | 4.1 | — |
| `HB_SCRIPT_BALINESE` | `Bali` | `0x42616C69` | Balinese | 5.0 | — |
| `HB_SCRIPT_CUNEIFORM` | `Xsux` | `0x58737578` | Cuneiform | 5.0 | — |
| `HB_SCRIPT_NKO` | `Nkoo` | `0x4E6B6F6F` | N'Ko | 5.0 | — |
| `HB_SCRIPT_PHAGS_PA` | `Phag` | `0x50686167` | Phags-pa | 5.0 | — |
| `HB_SCRIPT_PHOENICIAN` | `Phnx` | `0x50686E78` | Phoenician | 5.0 | — |
| `HB_SCRIPT_CARIAN` | `Cari` | `0x43617269` | Carian | 5.1 | — |
| `HB_SCRIPT_CHAM` | `Cham` | `0x4368616D` | Cham | 5.1 | — |
| `HB_SCRIPT_KAYAH_LI` | `Kali` | `0x4B616C69` | Kayah Li | 5.1 | — |
| `HB_SCRIPT_LEPCHA` | `Lepc` | `0x4C657063` | Lepcha | 5.1 | — |
| `HB_SCRIPT_LYCIAN` | `Lyci` | `0x4C796369` | Lycian | 5.1 | — |
| `HB_SCRIPT_LYDIAN` | `Lydi` | `0x4C796469` | Lydian | 5.1 | — |
| `HB_SCRIPT_OL_CHIKI` | `Olck` | `0x4F6C636B` | Ol Chiki | 5.1 | — |
| `HB_SCRIPT_REJANG` | `Rjng` | `0x526A6E67` | Rejang | 5.1 | — |
| `HB_SCRIPT_SAURASHTRA` | `Saur` | `0x53617572` | Saurashtra | 5.1 | — |
| `HB_SCRIPT_SUNDANESE` | `Sund` | `0x53756E64` | Sundanese | 5.1 | — |
| `HB_SCRIPT_VAI` | `Vaii` | `0x56616969` | Vai | 5.1 | — |
| `HB_SCRIPT_AVESTAN` | `Avst` | `0x41767374` | Avestan | 5.2 | — |
| `HB_SCRIPT_BAMUM` | `Bamu` | `0x42616D75` | Bamum | 5.2 | — |
| `HB_SCRIPT_EGYPTIAN_HIEROGLYPHS` | `Egyp` | `0x45677970` | Egyptian Hieroglyphs | 5.2 | — |
| `HB_SCRIPT_IMPERIAL_ARAMAIC` | `Armi` | `0x41726D69` | Imperial Aramaic | 5.2 | — |
| `HB_SCRIPT_INSCRIPTIONAL_PAHLAVI` | `Phli` | `0x50686C69` | Inscriptional Pahlavi | 5.2 | — |
| `HB_SCRIPT_INSCRIPTIONAL_PARTHIAN` | `Prti` | `0x50727469` | Inscriptional Parthian | 5.2 | — |
| `HB_SCRIPT_JAVANESE` | `Java` | `0x4A617661` | Javanese | 5.2 | — |
| `HB_SCRIPT_KAITHI` | `Kthi` | `0x4B746869` | Kaithi | 5.2 | — |
| `HB_SCRIPT_LISU` | `Lisu` | `0x4C697375` | Lisu | 5.2 | — |
| `HB_SCRIPT_MEETEI_MAYEK` | `Mtei` | `0x4D746569` | Meetei Mayek | 5.2 | — |
| `HB_SCRIPT_OLD_SOUTH_ARABIAN` | `Sarb` | `0x53617262` | Old South Arabian | 5.2 | — |
| `HB_SCRIPT_OLD_TURKIC` | `Orkh` | `0x4F726B68` | Old Turkic | 5.2 | — |
| `HB_SCRIPT_SAMARITAN` | `Samr` | `0x53616D72` | Samaritan | 5.2 | — |
| `HB_SCRIPT_TAI_THAM` | `Lana` | `0x4C616E61` | Tai Tham | 5.2 | — |
| `HB_SCRIPT_TAI_VIET` | `Tavt` | `0x54617674` | Tai Viet | 5.2 | — |
| `HB_SCRIPT_BATAK` | `Batk` | `0x4261746B` | Batak | 6.0 | — |
| `HB_SCRIPT_BRAHMI` | `Brah` | `0x42726168` | Brahmi | 6.0 | — |
| `HB_SCRIPT_MANDAIC` | `Mand` | `0x4D616E64` | Mandaic | 6.0 | — |
| `HB_SCRIPT_CHAKMA` | `Cakm` | `0x43616B6D` | Chakma | 6.1 | — |
| `HB_SCRIPT_MEROITIC_CURSIVE` | `Merc` | `0x4D657263` | Meroitic Cursive | 6.1 | — |
| `HB_SCRIPT_MEROITIC_HIEROGLYPHS` | `Mero` | `0x4D65726F` | Meroitic Hieroglyphs | 6.1 | — |
| `HB_SCRIPT_MIAO` | `Plrd` | `0x506C7264` | Miao | 6.1 | — |
| `HB_SCRIPT_SHARADA` | `Shrd` | `0x53687264` | Sharada | 6.1 | — |
| `HB_SCRIPT_SORA_SOMPENG` | `Sora` | `0x536F7261` | Sora Sompeng | 6.1 | — |
| `HB_SCRIPT_TAKRI` | `Takr` | `0x54616B72` | Takri | 6.1 | — |
| `HB_SCRIPT_BASSA_VAH` | `Bass` | `0x42617373` | Bassa Vah | 7.0 | 0.9.30 |
| `HB_SCRIPT_CAUCASIAN_ALBANIAN` | `Aghb` | `0x41676862` | Caucasian Albanian | 7.0 | 0.9.30 |
| `HB_SCRIPT_DUPLOYAN` | `Dupl` | `0x4475706C` | Duployan | 7.0 | 0.9.30 |
| `HB_SCRIPT_ELBASAN` | `Elba` | `0x456C6261` | Elbasan | 7.0 | 0.9.30 |
| `HB_SCRIPT_GRANTHA` | `Gran` | `0x4772616E` | Grantha | 7.0 | 0.9.30 |
| `HB_SCRIPT_KHOJKI` | `Khoj` | `0x4B686F6A` | Khojki | 7.0 | 0.9.30 |
| `HB_SCRIPT_KHUDAWADI` | `Sind` | `0x53696E64` | Khudawadi | 7.0 | 0.9.30 |
| `HB_SCRIPT_LINEAR_A` | `Lina` | `0x4C696E61` | Linear A | 7.0 | 0.9.30 |
| `HB_SCRIPT_MAHAJANI` | `Mahj` | `0x4D61686A` | Mahajani | 7.0 | 0.9.30 |
| `HB_SCRIPT_MANICHAEAN` | `Mani` | `0x4D616E69` | Manichaean | 7.0 | 0.9.30 |
| `HB_SCRIPT_MENDE_KIKAKUI` | `Mend` | `0x4D656E64` | Mende Kikakui | 7.0 | 0.9.30 |
| `HB_SCRIPT_MODI` | `Modi` | `0x4D6F6469` | Modi | 7.0 | 0.9.30 |
| `HB_SCRIPT_MRO` | `Mroo` | `0x4D726F6F` | Mro | 7.0 | 0.9.30 |
| `HB_SCRIPT_NABATAEAN` | `Nbat` | `0x4E626174` | Nabataean | 7.0 | 0.9.30 |
| `HB_SCRIPT_OLD_NORTH_ARABIAN` | `Narb` | `0x4E617262` | Old North Arabian | 7.0 | 0.9.30 |
| `HB_SCRIPT_OLD_PERMIC` | `Perm` | `0x5065726D` | Old Permic | 7.0 | 0.9.30 |
| `HB_SCRIPT_PAHAWH_HMONG` | `Hmng` | `0x486D6E67` | Pahawh Hmong | 7.0 | 0.9.30 |
| `HB_SCRIPT_PALMYRENE` | `Palm` | `0x50616C6D` | Palmyrene | 7.0 | 0.9.30 |
| `HB_SCRIPT_PAU_CIN_HAU` | `Pauc` | `0x50617563` | Pau Cin Hau | 7.0 | 0.9.30 |
| `HB_SCRIPT_PSALTER_PAHLAVI` | `Phlp` | `0x50686C70` | Psalter Pahlavi | 7.0 | 0.9.30 |
| `HB_SCRIPT_SIDDHAM` | `Sidd` | `0x53696464` | Siddham | 7.0 | 0.9.30 |
| `HB_SCRIPT_TIRHUTA` | `Tirh` | `0x54697268` | Tirhuta | 7.0 | 0.9.30 |
| `HB_SCRIPT_WARANG_CITI` | `Wara` | `0x57617261` | Warang Citi | 7.0 | 0.9.30 |
| `HB_SCRIPT_AHOM` | `Ahom` | `0x41686F6D` | Ahom | 8.0 | 0.9.30 |
| `HB_SCRIPT_ANATOLIAN_HIEROGLYPHS` | `Hluw` | `0x486C7577` | Anatolian Hieroglyphs | 8.0 | 0.9.30 |
| `HB_SCRIPT_HATRAN` | `Hatr` | `0x48617472` | Hatran | 8.0 | 0.9.30 |
| `HB_SCRIPT_MULTANI` | `Mult` | `0x4D756C74` | Multani | 8.0 | 0.9.30 |
| `HB_SCRIPT_OLD_HUNGARIAN` | `Hung` | `0x48756E67` | Old Hungarian | 8.0 | 0.9.30 |
| `HB_SCRIPT_SIGNWRITING` | `Sgnw` | `0x53676E77` | SignWriting | 8.0 | 0.9.30 |
| `HB_SCRIPT_ADLAM` | `Adlm` | `0x41646C6D` | Adlam | 9.0 | 1.3.0 |
| `HB_SCRIPT_BHAIKSUKI` | `Bhks` | `0x42686B73` | Bhaiksuki | 9.0 | 1.3.0 |
| `HB_SCRIPT_MARCHEN` | `Marc` | `0x4D617263` | Marchen | 9.0 | 1.3.0 |
| `HB_SCRIPT_OSAGE` | `Osge` | `0x4F736765` | Osage | 9.0 | 1.3.0 |
| `HB_SCRIPT_TANGUT` | `Tang` | `0x54616E67` | Tangut | 9.0 | 1.3.0 |
| `HB_SCRIPT_NEWA` | `Newa` | `0x4E657761` | Newa | 9.0 | 1.3.0 |
| `HB_SCRIPT_MASARAM_GONDI` | `Gonm` | `0x476F6E6D` | Masaram Gondi | 10.0 | 1.6.0 |
| `HB_SCRIPT_NUSHU` | `Nshu` | `0x4E736875` | Nushu | 10.0 | 1.6.0 |
| `HB_SCRIPT_SOYOMBO` | `Soyo` | `0x536F796F` | Soyombo | 10.0 | 1.6.0 |
| `HB_SCRIPT_ZANABAZAR_SQUARE` | `Zanb` | `0x5A616E62` | Zanabazar Square | 10.0 | 1.6.0 |
| `HB_SCRIPT_DOGRA` | `Dogr` | `0x446F6772` | Dogra | 11.0 | 1.8.0 |
| `HB_SCRIPT_GUNJALA_GONDI` | `Gong` | `0x476F6E67` | Gunjala Gondi | 11.0 | 1.8.0 |
| `HB_SCRIPT_HANIFI_ROHINGYA` | `Rohg` | `0x526F6867` | Hanifi Rohingya | 11.0 | 1.8.0 |
| `HB_SCRIPT_MAKASAR` | `Maka` | `0x4D616B61` | Makasar | 11.0 | 1.8.0 |
| `HB_SCRIPT_MEDEFAIDRIN` | `Medf` | `0x4D656466` | Medefaidrin | 11.0 | 1.8.0 |
| `HB_SCRIPT_OLD_SOGDIAN` | `Sogo` | `0x536F676F` | Old Sogdian | 11.0 | 1.8.0 |
| `HB_SCRIPT_SOGDIAN` | `Sogd` | `0x536F6764` | Sogdian | 11.0 | 1.8.0 |
| `HB_SCRIPT_ELYMAIC` | `Elym` | `0x456C796D` | Elymaic | 12.0 | 2.4.0 |
| `HB_SCRIPT_NANDINAGARI` | `Nand` | `0x4E616E64` | Nandinagari | 12.0 | 2.4.0 |
| `HB_SCRIPT_NYIAKENG_PUACHUE_HMONG` | `Hmnp` | `0x486D6E70` | Nyiakeng Puachue Hmong | 12.0 | 2.4.0 |
| `HB_SCRIPT_WANCHO` | `Wcho` | `0x5763686F` | Wancho | 12.0 | 2.4.0 |
| `HB_SCRIPT_CHORASMIAN` | `Chrs` | `0x43687273` | Chorasmian | 13.0 | 2.6.7 |
| `HB_SCRIPT_DIVES_AKURU` | `Diak` | `0x4469616B` | Dives Akuru | 13.0 | 2.6.7 |
| `HB_SCRIPT_KHITAN_SMALL_SCRIPT` | `Kits` | `0x4B697473` | Khitan Small Script | 13.0 | 2.6.7 |
| `HB_SCRIPT_YEZIDI` | `Yezi` | `0x59657A69` | Yezidi | 13.0 | 2.6.7 |
| `HB_SCRIPT_CYPRO_MINOAN` | `Cpmn` | `0x43706D6E` | Cypro-Minoan | 14.0 | 3.0.0 |
| `HB_SCRIPT_OLD_UYGHUR` | `Ougr` | `0x4F756772` | Old Uyghur | 14.0 | 3.0.0 |
| `HB_SCRIPT_TANGSA` | `Tnsa` | `0x546E7361` | Tangsa | 14.0 | 3.0.0 |
| `HB_SCRIPT_TOTO` | `Toto` | `0x546F746F` | Toto | 14.0 | 3.0.0 |
| `HB_SCRIPT_VITHKUQI` | `Vith` | `0x56697468` | Vithkuqi | 14.0 | 3.0.0 |
| `HB_SCRIPT_MATH` | `Zmth` | `0x5A6D7468` | Mathematical notation | — | 3.4.0 |
| `HB_SCRIPT_KAWI` | `Kawi` | `0x4B617769` | Kawi | 15.0 | 5.2.0 |
| `HB_SCRIPT_NAG_MUNDARI` | `Nagm` | `0x4E61676D` | Nag Mundari | 15.0 | 5.2.0 |
| `HB_SCRIPT_GARAY` | `Gara` | `0x47617261` | Garay | 16.0 | 10.0.0 |
| `HB_SCRIPT_GURUNG_KHEMA` | `Gukh` | `0x47756B68` | Gurung Khema | 16.0 | 10.0.0 |
| `HB_SCRIPT_KIRAT_RAI` | `Krai` | `0x4B726169` | Kirat Rai | 16.0 | 10.0.0 |
| `HB_SCRIPT_OL_ONAL` | `Onao` | `0x4F6E616F` | Ol Onal | 16.0 | 10.0.0 |
| `HB_SCRIPT_SUNUWAR` | `Sunu` | `0x53756E75` | Sunuwar | 16.0 | 10.0.0 |
| `HB_SCRIPT_TODHRI` | `Todr` | `0x546F6472` | Todhri | 16.0 | 10.0.0 |
| `HB_SCRIPT_TULU_TIGALARI` | `Tutg` | `0x54757467` | Tulu-Tigalari | 16.0 | 10.0.0 |
| `HB_SCRIPT_BERIA_ERFE` | `Berf` | `0x42657266` | Beria Erfe | 17.0 | 11.5.0 |
| `HB_SCRIPT_SIDETIC` | `Sidt` | `0x53696474` | Sidetic | 17.0 | 11.5.0 |
| `HB_SCRIPT_TAI_YO` | `Tayo` | `0x5461796F` | Tai Yo | 17.0 | 11.5.0 |
| `HB_SCRIPT_TOLONG_SIKI` | `Tols` | `0x546F6C73` | Tolong Siki | 17.0 | 11.5.0 |
| `HB_SCRIPT_INVALID` | — | `0x00000000` | No script set (`HB_TAG_NONE`) | — | — |

## Usage

### Setting a buffer's script

```c
#include <hb.h>

hb_buffer_t *buf = hb_buffer_create ();
hb_buffer_add_utf8 (buf, "مرحبا", -1, 0, -1);

hb_buffer_set_script (buf, HB_SCRIPT_ARABIC);
hb_buffer_set_direction (buf, hb_script_get_horizontal_direction (HB_SCRIPT_ARABIC));
hb_buffer_set_language (buf, hb_language_from_string ("ar", -1));
```

```rust
use harfbuzz_sys::{
    HB_SCRIPT_ARABIC, hb_buffer_add_utf8, hb_buffer_create, hb_buffer_set_direction,
    hb_buffer_set_script, hb_script_get_horizontal_direction,
};

// SAFETY: `buf` is freshly created and owned here; the text pointer is valid
// for the duration of the `hb_buffer_add_utf8` call.
unsafe {
    let buf = hb_buffer_create();
    let text = "مرحبا";
    hb_buffer_add_utf8(buf, text.as_ptr().cast(), text.len() as i32, 0, -1);

    hb_buffer_set_script(buf, HB_SCRIPT_ARABIC);
    hb_buffer_set_direction(buf, hb_script_get_horizontal_direction(HB_SCRIPT_ARABIC));
}
```

If you do not know the script, call `hb_buffer_guess_segment_properties()`
instead of setting it by hand: it derives script, direction, and language from
the buffer's contents using `hb_unicode_script()` and this page's
`hb_script_get_horizontal_direction()`.

### Parsing a script from user input

```c
hb_script_t script = hb_script_from_string ("Deva", -1);
if (script == HB_SCRIPT_UNKNOWN || script == HB_SCRIPT_INVALID)
  { /* not a usable script code */ }
```

```rust
use core::ffi::c_char;
use harfbuzz_sys::{HB_SCRIPT_INVALID, HB_SCRIPT_UNKNOWN, hb_script_from_string, hb_script_t};

fn parse_script(s: &str) -> Option<hb_script_t> {
    // SAFETY: the pointer and length describe `s`, which outlives the call.
    let script = unsafe { hb_script_from_string(s.as_ptr() as *const c_char, s.len() as i32) };

    if script == HB_SCRIPT_UNKNOWN || script == HB_SCRIPT_INVALID {
        None
    } else {
        Some(script)
    }
}
```

Testing for both sentinels is the whole error check — there is no separate
failure return. Note that this rejects a *legitimately unknown* run too, since
`Zzzz` is a real Unicode script value; if you want to distinguish "the user
typed nonsense" from "the user typed `Zzzz`", compare the input text instead.

### Printing a script

```c
char buf[5];
hb_tag_to_string (hb_script_to_iso15924_tag (script), buf);
buf[4] = '\0';           /* hb_tag_to_string does NOT NUL-terminate */
printf ("%s\n", buf);
```

```rust
use core::ffi::c_char;
use harfbuzz_sys::{hb_script_t, hb_script_to_iso15924_tag, hb_tag_to_string};

fn script_to_string(script: hb_script_t) -> String {
    let mut buf = [0u8; 4];

    // SAFETY: `hb_tag_to_string` writes exactly four bytes and `buf` is four
    // bytes long. It does not NUL-terminate, which is why the buffer is sized
    // exactly and read back as a byte slice rather than a C string.
    unsafe { hb_tag_to_string(hb_script_to_iso15924_tag(script), buf.as_mut_ptr() as *mut c_char) };

    String::from_utf8_lossy(&buf).into_owned()
}
```

### Constructing a script tag at compile time

Because the encoding is just `HB_TAG`, a script HarfBuzz does not yet know
about can be built directly:

```rust
use harfbuzz_sys::{HB_TAG, hb_script_t};

// A hypothetical future ISO 15924 code. `hb_script_from_iso15924_tag` would
// pass this through unchanged, because it is one uppercase letter followed by
// three lowercase ones.
const HB_SCRIPT_FUTURE: hb_script_t = HB_TAG(b'X', b'y', b'z', b'w') as hb_script_t;
```

### Round-tripping through the Unicode functions

```rust
use harfbuzz_sys::{hb_unicode_funcs_get_default, hb_unicode_script};

// SAFETY: the default Unicode-functions structure is a process-wide singleton
// owned by HarfBuzz and is valid for the lifetime of the program.
let script = unsafe { hb_unicode_script(hb_unicode_funcs_get_default(), 'ก' as u32) };
assert_eq!(script, harfbuzz_sys::HB_SCRIPT_THAI);
```

## Pitfalls

### `hb_script_t` is not a closed set

The two private sentinels exist precisely so that arbitrary tags are
representable. Never `match` exhaustively over the constants and assume you
have covered everything, never transmute into a Rust `enum`, and never assume a
script returned by `hb_unicode_script()` on a newer HarfBuzz has a constant in
your build. Treat an unrecognised value as data to pass through, not as an
error.

### Zero means "unset", not "unknown"

`HB_SCRIPT_INVALID == 0` is what a buffer reports before anything sets a script.
`HB_SCRIPT_UNKNOWN` (`Zzzz`) is a real Unicode script value that
`hb_unicode_script()` legitimately returns for unassigned and private-use code
points. They are different conditions and both come back from
`hb_script_from_iso15924_tag` — `HB_TAG_NONE` gives the former, malformed input
gives the latter.

### `hb_script_to_iso15924_tag` is lossy in one direction only

It is a cast, so it cannot fail and cannot tell you the tag was never a real
script. The lossy step is the *other* function: `Hans`, `Hant`, `Aran`, `Qaai`,
and the rest of the alias table are collapsed on the way in and cannot be
recovered on the way out. If you need to preserve the writing-system variant
the user asked for, keep the original `hb_tag_t` yourself.

### The ISO 15924 tag is not the OpenType script tag

`HB_SCRIPT_DEVANAGARI` is `Deva`; the OpenType `GSUB`/`GPOS` script tags for the
same script are `dev2` and `deva`. `HB_SCRIPT_HAN` is `Hani`; the OpenType tag
is `hani`. The mapping is many-to-many and version-sensitive, and it lives in
`hb-ot-layout.h` (`hb_ot_tags_from_script_and_language`,
`hb_ot_tags_to_script_and_language`, `hb_ot_tag_to_script`) — do not hand-lower
the ISO tag and hope.

### `hb_tag_to_string` does not NUL-terminate

It writes exactly four bytes. A `char buf[4]` passed to `printf("%s")` reads off
the end. Size the buffer at five and terminate it yourself, or in Rust read back
a four-byte slice as shown above.

### `hb_script_from_string` pads, it does not validate

`hb_tag_from_string` right-pads with spaces and truncates at four bytes. Short
input silently becomes a different tag (`"Lat"` → `Lat `), which then fails the
shape test and gives `HB_SCRIPT_UNKNOWN`. Long input silently drops the tail.
Validate length yourself if the distinction matters.

### `HB_DIRECTION_INVALID` is a real answer

`hb_script_get_horizontal_direction` returns it for Old Hungarian, Old Italic,
Runic, and Tifinagh. Code that does
`hb_buffer_set_direction (buf, hb_script_get_horizontal_direction (script))`
will set the buffer's direction to *unset* for those four scripts. Check the
result, or let `hb_buffer_guess_segment_properties()` handle it.

### The `Since` column is about HarfBuzz, the `Unicode` column is not

The trailing `/*7.0*/`-style comments in `hb-script-list.h` are **Unicode**
versions, and the `Since:` lines in the gtk-doc block are **HarfBuzz** versions.
They differ wildly — Unicode 7.0's scripts arrived in HarfBuzz 0.9.30, Unicode
17.0's in HarfBuzz 11.5.0. Linking against an older HarfBuzz than the constant's
`Since` version fails at compile time in C and at link time never (the constants
are compile-time only), but `hb_unicode_script()` will simply never return the
value.

### Signedness in Rust

`hb_script_t` is `c_int` (signed) while `hb_tag_t` is `u32`. Every constant in
`script.rs` carries an explicit `as hb_script_t` cast, and code that mixes the
two needs one too. For ASCII script codes the top bit is always clear so the
numeric value is unaffected, but the types still differ.

### Where the header lives

`hb-script-list.h` was surgically extracted from the middle of `hb-common.h` so
that FreeType can copy the file verbatim (see harfbuzz issue #5271). It is not
independently includable — it has no real `#include`s and no
`HB_BEGIN_DECLS`/`HB_END_DECLS`, only dummy ones inside `#if 0` to satisfy
upstream's own header checks. Include `<hb.h>`.

## Section coverage

Upstream's `docs/harfbuzz-sections.txt` has no `<FILE>hb-script-list</FILE>`
section — running

```sh
sed -n '/<FILE>hb-script-list<\/FILE>/,/<\/SECTION>/p' docs/harfbuzz-sections.txt
```

produces no output. The script API is listed under `<FILE>hb-common</FILE>`
instead, which contributes these five entries:

| Section entry | Covered by |
| --- | --- |
| `hb_script_t` | [Types → `hb_script_t`](#hb_script_t) |
| `hb_script_from_iso15924_tag` | [Tag conversion](#hb_script_from_iso15924_tag) |
| `hb_script_to_iso15924_tag` | [Tag conversion](#hb_script_to_iso15924_tag) |
| `hb_script_from_string` | [Tag conversion](#hb_script_from_string) |
| `hb_script_get_horizontal_direction` | [Direction](#hb_script_get_horizontal_direction) |

The remaining `hb-common` entries (`HB_TAG`, `hb_direction_t`, `hb_language_t`,
`hb_feature_t`, and so on) belong to the common-types page, not to this one;
`HB_TAG`, `HB_TAG_NONE`, `HB_TAG_MAX_SIGNED`, and `hb_direction_t` are
nevertheless described above because the script encoding cannot be explained
without them.

The 177 `HB_SCRIPT_*` constants are gtk-doc enumerator members of `hb_script_t`
rather than standalone section entries, which is why they do not appear in the
section list; all 177 are tabulated above.
