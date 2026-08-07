# OpenType table fetching

Transcribed from `hb-ot-fetch.h`. Rust module: `harfbuzz_sys::ot_fetch`, glob
re-exported at the crate root.

## Overview

`hb-ot-fetch.h` is HarfBuzz's escape hatch for raw OpenType table values. Its
gtk-doc section is titled *"OpenType bit fields and numbers"*, and upstream's
own description is *"Functions for fetching various bit fields and numbers
scattered around OpenType tables. These are raw table values, and many of them
are legacy or unreliable, but applications might need them for various legacy
reasons."* That warning is the whole design brief: this header exists so you do
not have to parse `OS/2`, `head`, and `post` yourself just to read a licensing
flag or a bounding box.

The API is two functions and two tag types, and nothing else. There are no
objects, no allocation, no callbacks, and no error channel. You pass a
`hb_face_t` and a tag; you get back a `uint32_t` of bits or an `int32_t` number.
Every failure mode — missing table, table that failed sanitization, `OS/2`
version too old to carry the field, tag you made up — is reported as **zero**.
That is the same "nil object" philosophy the rest of HarfBuzz uses, applied to
scalars.

The tags themselves are worth a moment of attention: `fstp`, `fssl`, `mcst`,
`xmin` and friends are **not** OpenType table tags or feature tags. They are
HarfBuzz-invented keys that merely happen to be spelled with four characters so
they fit `HB_TAG`. Do not go looking for a `fstp` table in a font; there isn't
one. Both tag types are open-ended (their C enumerations end with a
`HB_TAG_MAX_SIGNED` sentinel), so unknown values are accepted and quietly
return zero rather than being a compile-time or run-time error.

Everything here is per-**face**. The three tables involved are properties of the
binary typeface, so variation coordinates, point size, and anything else set on
an `hb_font_t` have no effect — there is no font-level entry point and none is
needed. HarfBuzz loads and sanity-checks each table lazily the first time you
touch it, then caches it for the face's lifetime; a table that is absent or
fails sanitization is replaced by an all-zero "Null" object, which is exactly
why absence reads as zero.

When you want an *interpreted* answer rather than raw bits, prefer the
purpose-built APIs, which understand variations and fall back sensibly:

| You want | Use instead |
| --- | --- |
| Is this font bold / italic / condensed, as rendered? | `hb_style_get_value()` with `HB_STYLE_TAG_WEIGHT`, `HB_STYLE_TAG_ITALIC`, `HB_STYLE_TAG_SLANT_ANGLE`, `HB_STYLE_TAG_WIDTH` (`hb-style.h`) |
| Ascender / descender / line gap / x-height / cap-height | `hb_ot_metrics_get_position()` (`hb-ot-metrics.h`) |
| Which characters does this font actually cover? | `hb_face_collect_unicodes()` (`hb-face.h`) |
| Font extents for laying out a line | `hb_font_get_extents_for_direction()` (`hb-font.h`) |
| The bounding box of one glyph | `hb_font_get_glyph_extents()` (`hb-font.h`) |

Use `hb-ot-fetch.h` when you specifically need the byte that the font vendor
wrote — embedding permissions, the legacy `macStyle` bits a Mac-era pipeline
expects, or the `OS/2` range bitmaps a font-picker UI displays.

Include path in C: `#include <hb-ot.h>`. The header refuses to be included
directly (`#error "Include <hb-ot.h> instead."`).

Upstream compiles this whole file out under `HB_NO_OT_FETCH`, which
`HB_LEAN` — and therefore `HB_TINY` — defines. There is no stub: the two
functions are simply not defined, so calls fail to **link**, not to run. In this
crate that corresponds to the `lean` and `tiny` Cargo features; the default
configuration includes the API.

Everything in this header is new in HarfBuzz **14.3.0**.

## Types

### `hb_ot_bits_tag_t`

```c
typedef enum {
  HB_OT_BITS_TAG_FS_TYPE		= HB_TAG ('f','s','t','p'),
  HB_OT_BITS_TAG_FS_SELECTION		= HB_TAG ('f','s','s','l'),
  HB_OT_BITS_TAG_MAC_STYLE		= HB_TAG ('m','c','s','t'),
  HB_OT_BITS_TAG_IS_FIXED_PITCH		= HB_TAG ('f','x','p','t'),
  HB_OT_BITS_TAG_UNICODE_RANGE_1	= HB_TAG ('u','r','n','1'),
  HB_OT_BITS_TAG_UNICODE_RANGE_2	= HB_TAG ('u','r','n','2'),
  HB_OT_BITS_TAG_UNICODE_RANGE_3	= HB_TAG ('u','r','n','3'),
  HB_OT_BITS_TAG_UNICODE_RANGE_4	= HB_TAG ('u','r','n','4'),
  HB_OT_BITS_TAG_CODE_PAGE_RANGE_1	= HB_TAG ('c','p','r','1'),
  HB_OT_BITS_TAG_CODE_PAGE_RANGE_2	= HB_TAG ('c','p','r','2'),

  /*< private >*/
  _HB_OT_BITS_TAG_MAX_VALUE = HB_TAG_MAX_SIGNED /*< skip >*/
} hb_ot_bits_tag_t;
```

```rust
pub type hb_ot_bits_tag_t = core::ffi::c_int;
```

The bit fields that `hb_ot_fetch_bits()` can fetch. You never receive a value of
this type from HarfBuzz — you only pass one in — so there is nothing to own and
nothing to free.

The Rust transcription is a `c_int` alias plus constants rather than a Rust
`enum`. The C enumeration's private sentinel is `HB_TAG_MAX_SIGNED`
(`0x7FFFFFFF`), which pins the underlying C type at signed `int`, and the value
space is open: `hb_ot_fetch_bits()` accepts any 32-bit value and returns zero
for anything it does not recognise. A Rust `enum` holding a value outside its
variant list would be undefined behaviour, so the crate does not use one.

| Constant | Tag | Value | Source | Field width | Meaning |
| --- | --- | --- | --- | --- | --- |
| `HB_OT_BITS_TAG_FS_TYPE` | `fstp` | `0x66737470` | `OS/2` | `uint16` | `fsType` — embedding/licensing permissions |
| `HB_OT_BITS_TAG_FS_SELECTION` | `fssl` | `0x6673736C` | `OS/2` | `uint16` | `fsSelection` — style bits (italic, bold, regular, use-typo-metrics, WWS, oblique …) |
| `HB_OT_BITS_TAG_MAC_STYLE` | `mcst` | `0x6D637374` | `head` | `uint16` | `macStyle` — legacy Macintosh style bits |
| `HB_OT_BITS_TAG_IS_FIXED_PITCH` | `fxpt` | `0x66787074` | `post` | `uint32` | `isFixedPitch` — 0 if proportionally spaced, non-zero if monospaced |
| `HB_OT_BITS_TAG_UNICODE_RANGE_1` | `urn1` | `0x75726E31` | `OS/2` | `uint32` | `ulUnicodeRange1` — Unicode coverage bits 0–31 |
| `HB_OT_BITS_TAG_UNICODE_RANGE_2` | `urn2` | `0x75726E32` | `OS/2` | `uint32` | `ulUnicodeRange2` — Unicode coverage bits 32–63 |
| `HB_OT_BITS_TAG_UNICODE_RANGE_3` | `urn3` | `0x75726E33` | `OS/2` | `uint32` | `ulUnicodeRange3` — Unicode coverage bits 64–95 |
| `HB_OT_BITS_TAG_UNICODE_RANGE_4` | `urn4` | `0x75726E34` | `OS/2` | `uint32` | `ulUnicodeRange4` — Unicode coverage bits 96–127 |
| `HB_OT_BITS_TAG_CODE_PAGE_RANGE_1` | `cpr1` | `0x63707231` | `OS/2` v1+ | `uint32` | `ulCodePageRange1` — legacy code-page coverage bits 0–31 |
| `HB_OT_BITS_TAG_CODE_PAGE_RANGE_2` | `cpr2` | `0x63707232` | `OS/2` v1+ | `uint32` | `ulCodePageRange2` — legacy code-page coverage bits 32–63 |

`_HB_OT_BITS_TAG_MAX_VALUE` is marked `/*< private >*/` and `/*< skip >*/` in
the header. It is a type-width pin, not a usable value, and is not transcribed;
its only effect on the Rust side is choosing `c_int`.

Because `hb_ot_bits_tag_t` is `c_int` in Rust while `HB_TAG` returns `hb_tag_t`
(`u32`), building a tag on the fly needs a cast — the provided constants are
already cast:

```rust
let tag = harfbuzz_sys::HB_TAG(b'f', b's', b't', b'p') as harfbuzz_sys::hb_ot_bits_tag_t;
```

Four of the fields are 16 bits wide in the font, so the top half of the returned
`uint32_t` is always zero for `fsType`, `fsSelection`, and `macStyle`. The other
six are genuinely 32 bits and routinely have the high bit set — which is why
`hb_ot_fetch_bits()` returns an **unsigned** value.

#### Bit layouts

HarfBuzz returns these words verbatim; it neither validates nor masks them. The
layouts below come from the OpenType specification (and, where noted, from
HarfBuzz's own internal enumerations for the same fields).

**`fsType` (`HB_OT_BITS_TAG_FS_TYPE`)** — embedding permissions. The whole
value, not individual bits, is the licence: `0x0000` means *Installable
Embedding*, the most permissive setting.

| Mask | Meaning |
| --- | --- |
| `0x0000` | Installable embedding — no restriction |
| `0x0002` | Restricted License embedding — must not be embedded without permission |
| `0x0004` | Preview & Print embedding |
| `0x0008` | Editable embedding |
| `0x0100` | No subsetting |
| `0x0200` | Bitmap embedding only |
| `0x0001`, `0x0010`–`0x0080`, `0x0400`–`0x8000` | Reserved, should be zero |

Bits 0–3 are meant to be mutually exclusive, but fonts in the wild break that
rule; HarfBuzz's own test suite exercises a font (`Zycon.ttf`) whose `fsType`
is `1`, a reserved bit. Treat the value as untrusted data.

**`fsSelection` (`HB_OT_BITS_TAG_FS_SELECTION`)** — style bits. These names
match HarfBuzz's internal `OT::OS2::selection_flag_t`:

| Mask | Name | Meaning |
| --- | --- | --- |
| `0x0001` | `ITALIC` | Font is italic |
| `0x0002` | `UNDERSCORE` | Glyphs are underscored |
| `0x0004` | `NEGATIVE` | Glyphs have their foreground and background reversed |
| `0x0008` | `OUTLINED` | Outline (hollow) glyphs |
| `0x0010` | `STRIKEOUT` | Glyphs are overstruck |
| `0x0020` | `BOLD` | Font is bold |
| `0x0040` | `REGULAR` | Font is neither italic nor bold; mutually exclusive with the two above |
| `0x0080` | `USE_TYPO_METRICS` | Use `sTypoAscender`/`sTypoDescender`/`sTypoLineGap` for default line spacing |
| `0x0100` | `WWS` | The family name is a "weight/width/slope"-only family name |
| `0x0200` | `OBLIQUE` | Font is oblique rather than truly italic |
| `0x0400`–`0x8000` | — | Reserved, should be zero |

**`macStyle` (`HB_OT_BITS_TAG_MAC_STYLE`)** — the `head` table's legacy style
bits. These names match HarfBuzz's internal `OT::head::mac_style_flag_t`:

| Mask | Name |
| --- | --- |
| `0x0001` | `BOLD` |
| `0x0002` | `ITALIC` |
| `0x0004` | `UNDERLINE` |
| `0x0008` | `OUTLINE` |
| `0x0010` | `SHADOW` |
| `0x0020` | `CONDENSED` |
| `0x0040` | `EXPANDED` |
| `0x0080`–`0x8000` | Reserved, should be zero |

`macStyle` and `fsSelection` are supposed to agree about bold and italic. They
frequently do not — HarfBuzz's test suite includes a face (`adwaita.ttf`) whose
`fsSelection` says `REGULAR | USE_TYPO_METRICS` while its `macStyle` is `0`,
which is consistent, and the tests deliberately compare the two fields
separately because there is no guarantee in general. When they disagree, most
modern shapers and layout engines trust `fsSelection`.

**`isFixedPitch` (`HB_OT_BITS_TAG_IS_FIXED_PITCH`)** — this is not really a
bitmap. The `post` table stores a 32-bit integer that is `0` for a
proportionally spaced font and non-zero for a monospaced one. Test it as
`!= 0`, never `== 1`: the specification does not fix the non-zero value, and
HarfBuzz's own test asserts only `!= 0`.

**`ulUnicodeRange1`–`4` (`HB_OT_BITS_TAG_UNICODE_RANGE_1`–`_4`)** — a 128-bit
bitmap of Unicode blocks the font claims to support. Global bit *n* lives in
range word `n / 32`, at bit `n % 32` (least significant bit first). So range 1
holds bits 0–31, range 2 holds 32–63, range 3 holds 64–95, range 4 holds 96–127.

Landmark assignments (see the OpenType `OS/2` specification for the full 128-bit
table):

| Global bit | Word | Block |
| --- | --- | --- |
| 0 | range 1, `0x00000001` | Basic Latin |
| 1 | range 1, `0x00000002` | Latin-1 Supplement |
| 2 | range 1, `0x00000004` | Latin Extended-A |
| 3 | range 1, `0x00000008` | Latin Extended-B |
| 7 | range 1, `0x00000080` | Greek and Coptic |
| 9 | range 1, `0x00000200` | Cyrillic |
| 11 | range 1, `0x00000800` | Hebrew |
| 13 | range 1, `0x00002000` | Arabic |
| 31 | range 1, `0x80000000` | General Punctuation |
| 57 | range 2, `0x02000000` | Non-Plane 0 — the font covers characters outside the BMP |

This bitmap is advisory metadata: it is what the vendor claimed, not what the
`cmap` contains. **Do not use it as a coverage test.** Use
`hb_face_collect_unicodes()`, or `hb_font_get_nominal_glyph()` for a single
character.

**`ulCodePageRange1`–`2` (`HB_OT_BITS_TAG_CODE_PAGE_RANGE_1`, `_2`)** — a 64-bit
bitmap of legacy Windows/DOS code pages, present only in `OS/2` version 1 and
later. Global bit *n* lives in word `n / 32` at bit `n % 32`, so range 1 holds
bits 0–31 and range 2 holds bits 32–63.

Range 1 (`cpr1`), the ANSI and Windows code pages:

| Bit | Mask | Code page |
| --- | --- | --- |
| 0 | `0x00000001` | 1252 — Latin 1 |
| 1 | `0x00000002` | 1250 — Latin 2, Eastern Europe |
| 2 | `0x00000004` | 1251 — Cyrillic |
| 3 | `0x00000008` | 1253 — Greek |
| 4 | `0x00000010` | 1254 — Turkish |
| 5 | `0x00000020` | 1255 — Hebrew |
| 6 | `0x00000040` | 1256 — Arabic |
| 7 | `0x00000080` | 1257 — Windows Baltic |
| 8 | `0x00000100` | 1258 — Vietnamese |
| 16 | `0x00010000` | 874 — Thai |
| 17 | `0x00020000` | 932 — JIS/Japan |
| 18 | `0x00040000` | 936 — Chinese, simplified |
| 19 | `0x00080000` | 949 — Korean Wansung |
| 20 | `0x00100000` | 950 — Chinese, traditional |
| 21 | `0x00200000` | 1361 — Korean Johab |
| 29 | `0x20000000` | Macintosh character set (US Roman) |
| 30 | `0x40000000` | OEM character set |
| 31 | `0x80000000` | Symbol character set |

Range 2 (`cpr2`) covers the OEM/DOS code pages. Bits 32–47 (the low half of the
word) are reserved for OEM use, and bits 48–63 (the high half) are the
assignments — bit 63 (`0x80000000` within the word) is code page 437 (US), bit
62 is 850 (WE/Latin 1), bit 61 is 708 (Arabic ASMO), bit 60 is 737 (Greek), bit
59 is 775 (MS-DOS Baltic), bit 58 is 852 (Latin 2), and so on down to bit 48,
code page 869 (IBM Greek).

Fonts predating `OS/2` version 1 have no code-page ranges at all, and HarfBuzz
returns `0` for both words in that case — its test suite pins this behaviour
with a version-0 font (`Zycon.ttf`).

### `hb_ot_number_tag_t`

```c
typedef enum {
  HB_OT_NUMBER_TAG_FONT_X_MIN		= HB_TAG ('x','m','i','n'),
  HB_OT_NUMBER_TAG_FONT_Y_MIN		= HB_TAG ('y','m','i','n'),
  HB_OT_NUMBER_TAG_FONT_X_MAX		= HB_TAG ('x','m','a','x'),
  HB_OT_NUMBER_TAG_FONT_Y_MAX		= HB_TAG ('y','m','a','x'),

  /*< private >*/
  _HB_OT_NUMBER_TAG_MAX_VALUE = HB_TAG_MAX_SIGNED /*< skip >*/
} hb_ot_number_tag_t;
```

```rust
pub type hb_ot_number_tag_t = core::ffi::c_int;
```

The numbers that `hb_ot_fetch_number()` can fetch. As with `hb_ot_bits_tag_t`,
you only ever pass values in, there is nothing to own, the enumeration is open,
and the `HB_TAG_MAX_SIGNED` sentinel pins the Rust alias at `c_int`.

All four entries are the `head` table's font-wide bounding box — the union of
every glyph's bounding box — expressed in **font units**.

| Constant | Tag | Value | Source | Field type | Meaning |
| --- | --- | --- | --- | --- | --- |
| `HB_OT_NUMBER_TAG_FONT_X_MIN` | `xmin` | `0x786D696E` | `head` | `int16` | `xMin` — left edge of the font bounding box |
| `HB_OT_NUMBER_TAG_FONT_Y_MIN` | `ymin` | `0x796D696E` | `head` | `int16` | `yMin` — bottom edge |
| `HB_OT_NUMBER_TAG_FONT_X_MAX` | `xmax` | `0x786D6178` | `head` | `int16` | `xMax` — right edge |
| `HB_OT_NUMBER_TAG_FONT_Y_MAX` | `ymax` | `0x796D6178` | `head` | `int16` | `yMax` — top edge |

`_HB_OT_NUMBER_TAG_MAX_VALUE` is `/*< private >*/` and `/*< skip >*/`, so it is
not transcribed.

Two consequences of the fields being `int16` while the function returns
`int32_t`: the values always land in −32768…32767, and they are sign-extended,
so a negative `yMin` (very common — descenders sit below the baseline) comes
back negative rather than as a large unsigned number.

Font units are relative to the face's units-per-em; divide by
`hb_face_get_upem()` to get em-relative numbers, or multiply by the font's scale
divided by upem to get the same value in the units
`hb_font_get_glyph_extents()` uses.

The name says `FONT_X_MIN`, but this is face data: it is fixed at the default
instance of a variable font and does not track `hb_font_set_variations()`.

## Functions

### Bit fields

#### `hb_ot_fetch_bits`

```c
HB_EXTERN uint32_t
hb_ot_fetch_bits (hb_face_t        *face,
                  hb_ot_bits_tag_t  tag);
```

```rust
pub fn hb_ot_fetch_bits(face: *mut hb_face_t, tag: hb_ot_bits_tag_t) -> u32;
```

Fetches a bit field of `face`.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to read. The header documents nothing about null; the implementation dereferences `face` immediately (`face->table.OS2`, …), so **do not pass null** — behaviour is unspecified. `hb_face_get_empty()` is well defined and returns `0` for every tag. |
| `tag` | Which bit field to fetch. One of the ten `HB_OT_BITS_TAG_*` constants; any other 32-bit value is accepted and returns `0`. There is no range check and no diagnostic. |

**Returns** — the raw field, zero-extended into a `uint32_t`, or **zero** if the
font does not have it. "Does not have it" folds together five distinct
situations, and the API cannot tell them apart:

1. The source table (`OS/2`, `head`, or `post`) is absent from the font.
2. The source table is present but failed HarfBuzz's sanitization, so the
   all-zero Null object is used instead.
3. The `OS/2` table is version 0, which has no `ulCodePageRange1`/`2`.
4. The field is genuinely zero in the font — `fsType == 0` (installable
   embedding), `macStyle == 0` (regular), `isFixedPitch == 0` (proportional).
5. `tag` is not one of the recognised constants.

Case 4 is the one that bites: **zero is a meaningful value for most of these
fields**, so you cannot use it as an error signal. If you need to know whether
the table exists at all, ask separately, e.g. with
`hb_face_reference_table(face, HB_TAG('O','S','/','2'))` and checking
`hb_blob_get_length()`.

**Ownership** — nothing is allocated and nothing is transferred. The face is
only read; the caller keeps its reference and destroys the face as usual. There
is nothing to free.

**Notes**

- Since HarfBuzz 14.3.0.
- The first call for a given table on a given face loads and sanitizes that
  table; subsequent calls hit the face's cache. Sanitization failure is
  permanent for that face and silent.
- `head` sanitization requires `version.major == 1` **and**
  `magicNumber == 0x5F0F3CF5`. A font that gets the magic number wrong yields
  `macStyle == 0` from this function and `0` from every
  `hb_ot_fetch_number()` tag.
- `post` sanitization accepts versions 1.0, 2.0, and 3.0 only. The deprecated
  version 2.5 (`0x00025000`) is rejected, so such a font reports
  `isFixedPitch == 0` regardless of what it actually says.
- `OS/2` sanitization checks the fixed part and then each version tail that the
  declared version claims; a truncated v1 tail rejects the whole table, taking
  `fsType` and `fsSelection` down with it.
- Thread-safe. Concurrent calls on the same face are fine; the lazy table
  loaders use the same atomic initialisation as every other HarfBuzz table.
- Face-level only. Variation coordinates, named instances, and font scale have
  no effect on the result.
- Compiled out under `HB_NO_OT_FETCH` (implied by `HB_LEAN` and `HB_TINY`;
  the `lean` and `tiny` Cargo features here) — a link error, not a run-time
  failure.

### Numbers

#### `hb_ot_fetch_number`

```c
HB_EXTERN int32_t
hb_ot_fetch_number (hb_face_t          *face,
                    hb_ot_number_tag_t  tag);
```

```rust
pub fn hb_ot_fetch_number(face: *mut hb_face_t, tag: hb_ot_number_tag_t) -> i32;
```

Fetches a number of `face`, in font units.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to read. As with `hb_ot_fetch_bits()`, null is undocumented and dereferenced immediately — do not pass it. `hb_face_get_empty()` returns `0` for every tag. |
| `tag` | Which number to fetch. One of the four `HB_OT_NUMBER_TAG_FONT_*` constants; any other 32-bit value returns `0`. |

**Returns** — the value in font units, sign-extended from the table's `int16`
into an `int32_t`, or **zero** if the font does not have it. All four tags read
the `head` table, so the failure cases reduce to three:

1. The font has no `head` table.
2. The `head` table failed sanitization (wrong major version or wrong magic
   number) and the all-zero Null object is used.
3. `tag` is not one of the four recognised constants.

And, again, **zero is a legal value**: `xMin == 0` is perfectly normal for a
font whose leftmost ink starts at the origin. A face with an entirely empty
bounding box (all four zero) is indistinguishable from a face with no `head`
table.

**Ownership** — nothing is allocated and nothing is transferred; the face is
read only.

**Notes**

- Since HarfBuzz 14.3.0.
- Font units. Combine with `hb_face_get_upem()` to normalise.
- The bounding box is the union over all glyphs in the face, so it is usually
  much larger than any single glyph's extents and is a poor proxy for line
  height. For line layout use `hb_font_get_extents_for_direction()`; for a
  single glyph use `hb_font_get_glyph_extents()`.
- Static: it describes the default instance of a variable font, and does not
  respond to `hb_font_set_variations()`. HarfBuzz's subsetter recomputes these
  values when it writes a new `head`, but reading through this API never does.
- Same lazy-load, caching, thread-safety, and `HB_NO_OT_FETCH` notes as
  `hb_ot_fetch_bits()`.

## Usage

### Checking embedding permissions

The single most common reason to reach for this header: a document exporter that
must honour a font's `fsType` before embedding it.

C:

```c
#include <hb-ot.h>

typedef enum { EMBED_OK, EMBED_PREVIEW_PRINT, EMBED_EDITABLE, EMBED_FORBIDDEN }
embed_perm_t;

static embed_perm_t
embedding_permission (hb_face_t *face)
{
  uint32_t fs_type = hb_ot_fetch_bits (face, HB_OT_BITS_TAG_FS_TYPE);

  /* Installable embedding is the *absence* of restriction bits. */
  if ((fs_type & 0x000Fu) == 0)          return EMBED_OK;
  if (fs_type & 0x0002u)                 return EMBED_FORBIDDEN;   /* restricted */
  if (fs_type & 0x0008u)                 return EMBED_EDITABLE;
  if (fs_type & 0x0004u)                 return EMBED_PREVIEW_PRINT;

  return EMBED_FORBIDDEN;                /* reserved bits set: be conservative */
}

static hb_bool_t
may_subset (hb_face_t *face)
{
  return !(hb_ot_fetch_bits (face, HB_OT_BITS_TAG_FS_TYPE) & 0x0100u);
}
```

Rust:

```rust
use harfbuzz_sys::{HB_OT_BITS_TAG_FS_TYPE, hb_face_t, hb_ot_fetch_bits};

const FS_TYPE_RESTRICTED: u32 = 0x0002;
const FS_TYPE_PREVIEW_PRINT: u32 = 0x0004;
const FS_TYPE_EDITABLE: u32 = 0x0008;
const FS_TYPE_NO_SUBSETTING: u32 = 0x0100;
const FS_TYPE_BITMAP_ONLY: u32 = 0x0200;

pub enum Embedding {
    Installable,
    PreviewAndPrint,
    Editable,
    Forbidden,
}

/// # Safety
/// `face` must be a live `hb_face_t`.
pub unsafe fn embedding_permission(face: *mut hb_face_t) -> Embedding {
    let fs_type = unsafe { hb_ot_fetch_bits(face, HB_OT_BITS_TAG_FS_TYPE) };

    if fs_type & 0x000F == 0 {
        Embedding::Installable
    } else if fs_type & FS_TYPE_RESTRICTED != 0 {
        Embedding::Forbidden
    } else if fs_type & FS_TYPE_EDITABLE != 0 {
        Embedding::Editable
    } else if fs_type & FS_TYPE_PREVIEW_PRINT != 0 {
        Embedding::PreviewAndPrint
    } else {
        Embedding::Forbidden
    }
}

/// # Safety
/// `face` must be a live `hb_face_t`.
pub unsafe fn may_subset(face: *mut hb_face_t) -> bool {
    unsafe { hb_ot_fetch_bits(face, HB_OT_BITS_TAG_FS_TYPE) & FS_TYPE_NO_SUBSETTING == 0 }
}

/// # Safety
/// `face` must be a live `hb_face_t`.
pub unsafe fn bitmap_embedding_only(face: *mut hb_face_t) -> bool {
    unsafe { hb_ot_fetch_bits(face, HB_OT_BITS_TAG_FS_TYPE) & FS_TYPE_BITMAP_ONLY != 0 }
}
```

Note that a font with no `OS/2` table reports `fs_type == 0`, i.e. *installable*.
That is the permissive default, which may or may not be what your legal team
wants; if it is not, check for the table's presence separately.

### Reading the style bits

C:

```c
uint32_t sel = hb_ot_fetch_bits (face, HB_OT_BITS_TAG_FS_SELECTION);
uint32_t mac = hb_ot_fetch_bits (face, HB_OT_BITS_TAG_MAC_STYLE);

hb_bool_t italic     = (sel & 0x0001u) != 0;
hb_bool_t bold       = (sel & 0x0020u) != 0;
hb_bool_t regular    = (sel & 0x0040u) != 0;
hb_bool_t typo_metrics = (sel & 0x0080u) != 0;
hb_bool_t oblique    = (sel & 0x0200u) != 0;

hb_bool_t condensed  = (mac & 0x0020u) != 0;
hb_bool_t expanded   = (mac & 0x0040u) != 0;
```

`USE_TYPO_METRICS` is the one bit here that changes rendering: when it is set,
line spacing should come from `sTypoAscender`/`sTypoDescender`/`sTypoLineGap`
rather than from `usWinAscent`/`usWinDescent`. HarfBuzz already honours it
inside `hb_ot_metrics_get_position()` and `hb_font_get_extents_for_direction()`,
so read it only if you are re-implementing that logic.

Rust:

```rust
use harfbuzz_sys::{
    HB_OT_BITS_TAG_FS_SELECTION, HB_OT_BITS_TAG_MAC_STYLE, hb_face_t, hb_ot_fetch_bits,
};

const SEL_ITALIC: u32 = 1 << 0;
const SEL_BOLD: u32 = 1 << 5;
const SEL_REGULAR: u32 = 1 << 6;
const SEL_USE_TYPO_METRICS: u32 = 1 << 7;
const SEL_WWS: u32 = 1 << 8;
const SEL_OBLIQUE: u32 = 1 << 9;

const MAC_BOLD: u32 = 1 << 0;
const MAC_ITALIC: u32 = 1 << 1;
const MAC_CONDENSED: u32 = 1 << 5;
const MAC_EXPANDED: u32 = 1 << 6;

/// # Safety
/// `face` must be a live `hb_face_t`.
pub unsafe fn describe(face: *mut hb_face_t) -> String {
    let sel = unsafe { hb_ot_fetch_bits(face, HB_OT_BITS_TAG_FS_SELECTION) };
    let mac = unsafe { hb_ot_fetch_bits(face, HB_OT_BITS_TAG_MAC_STYLE) };

    let mut parts = Vec::new();
    if sel & SEL_REGULAR != 0 { parts.push("regular"); }
    if sel & SEL_BOLD != 0 { parts.push("bold"); }
    if sel & SEL_ITALIC != 0 { parts.push("italic"); }
    if sel & SEL_OBLIQUE != 0 { parts.push("oblique"); }
    if sel & SEL_WWS != 0 { parts.push("wws"); }
    if sel & SEL_USE_TYPO_METRICS != 0 { parts.push("use-typo-metrics"); }
    if mac & MAC_CONDENSED != 0 { parts.push("condensed"); }
    if mac & MAC_EXPANDED != 0 { parts.push("expanded"); }

    // macStyle and fsSelection can disagree; report it rather than pick.
    let mac_says_bold = mac & MAC_BOLD != 0;
    let mac_says_italic = mac & MAC_ITALIC != 0;
    if mac_says_bold != (sel & SEL_BOLD != 0) || mac_says_italic != (sel & SEL_ITALIC != 0) {
        parts.push("(macStyle disagrees with fsSelection)");
    }

    parts.join(" ")
}
```

For a variable font, prefer `hb_style_get_value(font, HB_STYLE_TAG_WEIGHT)` and
`hb_style_get_value(font, HB_STYLE_TAG_ITALIC)`: those follow the variation
coordinates set on the font, while these bits describe the default instance
only.

### Detecting a monospaced font

```c
hb_bool_t monospaced = hb_ot_fetch_bits (face, HB_OT_BITS_TAG_IS_FIXED_PITCH) != 0;
```

```rust
use harfbuzz_sys::{HB_OT_BITS_TAG_IS_FIXED_PITCH, hb_ot_fetch_bits};

let monospaced = unsafe { hb_ot_fetch_bits(face, HB_OT_BITS_TAG_IS_FIXED_PITCH) } != 0;
```

Compare against zero, not against `1`. And remember this is a claim, not a
measurement: fonts occasionally lie in both directions. Measuring a couple of
advance widths with `hb_font_get_glyph_h_advance()` is the reliable check.

### The font bounding box

C:

```c
int32_t x_min = hb_ot_fetch_number (face, HB_OT_NUMBER_TAG_FONT_X_MIN);
int32_t y_min = hb_ot_fetch_number (face, HB_OT_NUMBER_TAG_FONT_Y_MIN);
int32_t x_max = hb_ot_fetch_number (face, HB_OT_NUMBER_TAG_FONT_X_MAX);
int32_t y_max = hb_ot_fetch_number (face, HB_OT_NUMBER_TAG_FONT_Y_MAX);

unsigned int upem = hb_face_get_upem (face);

/* Em-relative, e.g. for a CSS/PDF-style descriptor. */
double left   = (double) x_min / upem;
double bottom = (double) y_min / upem;
double right  = (double) x_max / upem;
double top    = (double) y_max / upem;
```

Rust:

```rust
use harfbuzz_sys::{
    HB_OT_NUMBER_TAG_FONT_X_MAX, HB_OT_NUMBER_TAG_FONT_X_MIN, HB_OT_NUMBER_TAG_FONT_Y_MAX,
    HB_OT_NUMBER_TAG_FONT_Y_MIN, hb_face_get_upem, hb_face_t, hb_ot_fetch_number,
};

pub struct BBox {
    pub x_min: i32,
    pub y_min: i32,
    pub x_max: i32,
    pub y_max: i32,
}

/// # Safety
/// `face` must be a live `hb_face_t`.
pub unsafe fn font_bbox(face: *mut hb_face_t) -> BBox {
    unsafe {
        BBox {
            x_min: hb_ot_fetch_number(face, HB_OT_NUMBER_TAG_FONT_X_MIN),
            y_min: hb_ot_fetch_number(face, HB_OT_NUMBER_TAG_FONT_Y_MIN),
            x_max: hb_ot_fetch_number(face, HB_OT_NUMBER_TAG_FONT_X_MAX),
            y_max: hb_ot_fetch_number(face, HB_OT_NUMBER_TAG_FONT_Y_MAX),
        }
    }
}

/// # Safety
/// `face` must be a live `hb_face_t`.
pub unsafe fn font_bbox_em(face: *mut hb_face_t) -> [f64; 4] {
    let b = unsafe { font_bbox(face) };
    let upem = unsafe { hb_face_get_upem(face) } as f64;
    [
        b.x_min as f64 / upem,
        b.y_min as f64 / upem,
        b.x_max as f64 / upem,
        b.y_max as f64 / upem,
    ]
}
```

With HarfBuzz's `adwaita.ttf` test font this yields `(51, -250, 1238, 950)`; with
`SourceSansPro-Regular.abc.otf`, `(-454, -293, 2159, 968)`. The negative `yMin`
values are descenders, and they demonstrate why the return type is signed.

### Dumping the coverage bitmaps

```c
static void
dump_ranges (hb_face_t *face)
{
  static const hb_ot_bits_tag_t urange[4] = {
    HB_OT_BITS_TAG_UNICODE_RANGE_1, HB_OT_BITS_TAG_UNICODE_RANGE_2,
    HB_OT_BITS_TAG_UNICODE_RANGE_3, HB_OT_BITS_TAG_UNICODE_RANGE_4,
  };

  for (unsigned int w = 0; w < 4; w++)
  {
    uint32_t bits = hb_ot_fetch_bits (face, urange[w]);
    for (unsigned int b = 0; b < 32; b++)
      if (bits & (1u << b))
        printf ("Unicode range bit %u set\n", w * 32 + b);
  }

  printf ("code pages: %08X %08X\n",
          hb_ot_fetch_bits (face, HB_OT_BITS_TAG_CODE_PAGE_RANGE_1),
          hb_ot_fetch_bits (face, HB_OT_BITS_TAG_CODE_PAGE_RANGE_2));
}
```

```rust
use harfbuzz_sys::{
    HB_OT_BITS_TAG_UNICODE_RANGE_1, HB_OT_BITS_TAG_UNICODE_RANGE_2,
    HB_OT_BITS_TAG_UNICODE_RANGE_3, HB_OT_BITS_TAG_UNICODE_RANGE_4, hb_face_t,
    hb_ot_bits_tag_t, hb_ot_fetch_bits,
};

const UNICODE_RANGES: [hb_ot_bits_tag_t; 4] = [
    HB_OT_BITS_TAG_UNICODE_RANGE_1,
    HB_OT_BITS_TAG_UNICODE_RANGE_2,
    HB_OT_BITS_TAG_UNICODE_RANGE_3,
    HB_OT_BITS_TAG_UNICODE_RANGE_4,
];

/// Returns the font's claimed Unicode coverage as a 128-bit bitmap.
///
/// # Safety
/// `face` must be a live `hb_face_t`.
pub unsafe fn unicode_range_bits(face: *mut hb_face_t) -> u128 {
    let mut bits: u128 = 0;
    for (word, tag) in UNICODE_RANGES.iter().enumerate() {
        bits |= (unsafe { hb_ot_fetch_bits(face, *tag) } as u128) << (32 * word);
    }
    bits
}

/// Is the "Non-Plane 0" bit (global bit 57) set?
///
/// # Safety
/// `face` must be a live `hb_face_t`.
pub unsafe fn claims_supplementary_planes(face: *mut hb_face_t) -> bool {
    unsafe { unicode_range_bits(face) } & (1u128 << 57) != 0
}
```

With `NotoSans-Bold.ttf` from HarfBuzz's test suite the four words come back as
`0xE00002FF`, `0x4000201F`, `0x08000029`, `0x00100000`.

### Using an unrecognised tag

Both functions accept any 32-bit value and return zero, so a generic dispatcher
does not need to guard its input:

```c
/* Returns 0 — 'xxxx' is not a known bits tag. */
uint32_t v = hb_ot_fetch_bits (face, (hb_ot_bits_tag_t) HB_TAG ('x','x','x','x'));
```

```rust
use harfbuzz_sys::{HB_TAG, hb_ot_bits_tag_t, hb_ot_fetch_bits};

let unknown = HB_TAG(b'x', b'x', b'x', b'x') as hb_ot_bits_tag_t;
let v = unsafe { hb_ot_fetch_bits(face, unknown) }; // == 0
```

There is no `hb_ot_bits_tag_from_string()`; if you are driving this from a
command line, use `hb_tag_from_string()` and cast.

## Pitfalls

- **Zero means "absent *or* zero".** Every failure path — missing table, failed
  sanitization, `OS/2` version too old, unknown tag — returns `0`, and `0` is
  also a perfectly ordinary value for `fsType`, `macStyle`, `isFixedPitch`,
  `xMin`, and the range bitmaps. There is no error channel and no "has this
  table?" predicate. If the distinction matters, probe the table directly with
  `hb_face_reference_table()` and check the blob's length.
- **`fsType == 0` reads as "installable embedding" even for a font with no
  `OS/2` table.** For a licence check this fails *open*. Decide deliberately
  whether that is the behaviour you want.
- **The tags are not OpenType tags.** `fstp`, `fssl`, `mcst`, `fxpt`, `urn1`…,
  `cpr1`…, `xmin`… are HarfBuzz-invented keys. Do not confuse them with table
  tags, feature tags, or variation-axis tags, and do not expect
  `hb_face_reference_table(face, HB_TAG('f','s','t','p'))` to work.
- **`hb_ot_bits_tag_t` and `hb_ot_number_tag_t` are signed `c_int` in Rust**,
  because the C sentinel is `HB_TAG_MAX_SIGNED`. `HB_TAG` returns `u32`, so
  constructing a tag on the fly needs an `as hb_ot_bits_tag_t` cast. The
  provided constants are already cast.
- **`hb_ot_fetch_bits` returns unsigned, `hb_ot_fetch_number` returns signed.**
  Mixing them up matters: the range bitmaps routinely have bit 31 set, and
  `yMin` is routinely negative.
- **Test `isFixedPitch` with `!= 0`, not `== 1`.** The `post` field is a
  `uint32` whose non-zero value is unspecified.
- **Code-page ranges do not exist before `OS/2` version 1.** A version-0 font
  returns `0` for both `cpr1` and `cpr2`, which is indistinguishable from a font
  that declares no code pages.
- **The Unicode range bitmap is not a coverage test.** It is vendor-declared
  metadata that fonts get wrong all the time. Use
  `hb_face_collect_unicodes()` or `hb_font_get_nominal_glyph()` to find out what
  a font can actually render.
- **`macStyle` and `fsSelection` can disagree** about bold and italic. Neither
  field wins automatically; modern engines generally trust `fsSelection`.
- **These values ignore variations.** They are face-level and describe the
  default instance. `hb_font_set_variations()` and named instances have no
  effect. For variation-aware style information use `hb_style_get_value()`.
- **Sanitization failures are silent and sticky.** A `head` table with the wrong
  magic number, or a `post` table at the deprecated version 2.5, is replaced by
  an all-zero Null object for the entire lifetime of the face. Nothing is
  logged, and there is no way to re-try.
- **Null `face` is unspecified.** The header says nothing; the implementation
  dereferences `face` immediately. Pass a real face, or `hb_face_get_empty()`
  (which is well defined and returns `0` for every tag).
- **The `head` bounding box is not line metrics.** It is the union over every
  glyph in the face — including obscure ones with huge extents — so it usually
  overstates the space text needs. Use
  `hb_font_get_extents_for_direction()` for layout.
- **`HB_NO_OT_FETCH` builds do not link.** Under `HB_LEAN` or `HB_TINY` (the
  `lean` and `tiny` Cargo features here) both functions are compiled out
  entirely rather than stubbed, so calls become undefined symbols at link time.
- **Everything here is new in 14.3.0.** There is no fallback for older
  HarfBuzz; guard with `HB_VERSION_ATLEAST(14, 3, 0)` if you need to build
  against both.
