# OpenType name table

Reference for `hb-ot-name.h` — reading the human-readable strings out of an
OpenType `name` table — as transcribed in `harfbuzz_sys::ot_name`, glob
re-exported at the crate root.

## Overview

Every OpenType font carries its prose in one table: the family name, the style
name, the version string, the designer, the licence, the sample text, the
PostScript name. The `name` table stores each of those as a *name record*
identified by a four-part key — platform ID, encoding ID, language ID, name ID —
and a font typically contains the same string several times over, once per
platform and once per language it has been localised into. There is no
"the family name"; there is a Windows/Unicode/English one, possibly a
Macintosh/Roman/English one, possibly a Japanese one, and so on.

This header hides all of that behind two operations. `hb_ot_name_list_names()`
tells you which *(name ID, language)* pairs the face can produce, and the three
`hb_ot_name_get_utf*()` functions fetch the string for one such pair in the
Unicode encoding of your choice. HarfBuzz does the decoding: the first time a
face is asked about names it builds a small per-face index that maps each
record's platform and language codes to a BCP 47 `hb_language_t`, scores the
record's encoding, keeps only the best-encoded record for each *(name ID,
language)* pair, throws away records in encodings it cannot decode and records
whose language it cannot name, and sorts what is left. Everything after that is
a binary search over that index.

There are no objects to create and nothing to destroy. Both entry points take an
`hb_face_t *` and return either a borrowed array or plain integers; the face owns
all the memory, and the API never takes a reference on it. The `name` table is
face data, not font data, so a size, a variation setting, or a synthetic slant
makes no difference here — two `hb_font_t`s built from the same face report
exactly the same names.

Name IDs come in three bands. IDs 0–25 are defined by the specification and have
constants in `hb_ot_name_id_predefined_t`; ID 15 and IDs 26–255 are reserved and
should not appear in a well-formed font; IDs 256 and above are font-specific,
allocated by the font's designer, and are handed to you by *other* parts of the
API — `hb_ot_var_axis_info_t.name_id`, `hb_ot_var_named_instance_get_subfamily_name_id()`,
`hb_ot_layout_feature_get_name_ids()`, `hb_ot_color_palette_get_name_id()`,
`hb_aat_layout_feature_type_get_name_id()`, and friends. That is the reason
`hb_ot_name_id_t` is a plain `unsigned int` rather than the enumeration: an ID
you got from one of those calls is just as valid an argument as
`HB_OT_NAME_ID_FONT_FAMILY`, and it will not be one of the named constants.

Upstream compiles this entire file out under `HB_NO_NAME`, which is part of the
`HB_LEAN` / `HB_TINY` reduced-feature profiles and also implies
`HB_NO_OT_NAME_LANGUAGE`. The header still declares the functions
unconditionally, so a program can compile against it and fail to link. The
default configuration used by this crate's `build.rs` includes the whole thing.

## Types

### `hb_ot_name_id_t`

```c
typedef unsigned int hb_ot_name_id_t;
```

```rust
pub type hb_ot_name_id_t = core::ffi::c_uint;
```

An integral type representing an OpenType `name` table name identifier. Values
are the numbers stored in the table's `nameID` field, which is a `uint16`, so in
practice every meaningful value is in `0..=0xFFFF`. This is the type every
function in this header takes and the type every *other* header hands back when
it reports a name ID.

Since HarfBuzz 2.0.0, where it was introduced as `hb_name_id_t`; it took its
present name in 2.1.0.

### `hb_ot_name_id_predefined_t`

```c
typedef enum {
  HB_OT_NAME_ID_COPYRIGHT              = 0,
  /* ... */
  HB_OT_NAME_ID_VARIATIONS_PS_PREFIX   = 25,
  HB_OT_NAME_ID_INVALID                = 0xFFFF
} hb_ot_name_id_predefined_t;
```

```rust
pub type hb_ot_name_id_predefined_t = core::ffi::c_int;
```

The pre-defined name IDs. The C enumeration has no `/*< skip >*/` sentinel and
its largest enumerator is `0xFFFF`, which fits in an `int`, so the Rust
transcription is a `c_int` alias plus constants. It is an alias rather than a
Rust `enum` for the usual reason in this crate: the values are read out of font
data and out of other APIs, most legal values are *not* in the list, and a Rust
`enum` holding a value outside its variant list is undefined behaviour.

Full reference — see the
[OpenType spec](https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-ids)
for the normative wording:

| Constant | Value | Meaning |
| --- | --- | --- |
| `HB_OT_NAME_ID_COPYRIGHT` | 0 | Copyright notice. |
| `HB_OT_NAME_ID_FONT_FAMILY` | 1 | Font Family name. The legacy, four-styles-per-family name used by older Windows apps. |
| `HB_OT_NAME_ID_FONT_SUBFAMILY` | 2 | Font Subfamily name. Legacy; usually one of Regular / Italic / Bold / Bold Italic. |
| `HB_OT_NAME_ID_UNIQUE_ID` | 3 | Unique font identifier. |
| `HB_OT_NAME_ID_FULL_NAME` | 4 | Full font name that reflects all family and relevant subfamily descriptors. |
| `HB_OT_NAME_ID_VERSION_STRING` | 5 | Version string. |
| `HB_OT_NAME_ID_POSTSCRIPT_NAME` | 6 | PostScript name for the font. |
| `HB_OT_NAME_ID_TRADEMARK` | 7 | Trademark. |
| `HB_OT_NAME_ID_MANUFACTURER` | 8 | Manufacturer Name. |
| `HB_OT_NAME_ID_DESIGNER` | 9 | Designer. |
| `HB_OT_NAME_ID_DESCRIPTION` | 10 | Description. |
| `HB_OT_NAME_ID_VENDOR_URL` | 11 | URL of font vendor. |
| `HB_OT_NAME_ID_DESIGNER_URL` | 12 | URL of typeface designer. |
| `HB_OT_NAME_ID_LICENSE` | 13 | License Description. |
| `HB_OT_NAME_ID_LICENSE_URL` | 14 | URL where additional licensing information can be found. |
| — | 15 | Reserved by the specification. Upstream leaves it commented out; this crate emits no constant. |
| `HB_OT_NAME_ID_TYPOGRAPHIC_FAMILY` | 16 | Typographic Family name. The modern, unlimited-styles family name; falls back to ID 1 when absent. |
| `HB_OT_NAME_ID_TYPOGRAPHIC_SUBFAMILY` | 17 | Typographic Subfamily name. Falls back to ID 2 when absent. |
| `HB_OT_NAME_ID_MAC_FULL_NAME` | 18 | Compatible Full Name for MacOS. |
| `HB_OT_NAME_ID_SAMPLE_TEXT` | 19 | Sample text. |
| `HB_OT_NAME_ID_CID_FINDFONT_NAME` | 20 | PostScript CID `findfont` name. |
| `HB_OT_NAME_ID_WWS_FAMILY` | 21 | WWS (Weight/Width/Slope) Family Name. |
| `HB_OT_NAME_ID_WWS_SUBFAMILY` | 22 | WWS Subfamily Name. |
| `HB_OT_NAME_ID_LIGHT_BACKGROUND` | 23 | Light Background Palette. The name of a `CPAL` palette designed for light backgrounds. |
| `HB_OT_NAME_ID_DARK_BACKGROUND` | 24 | Dark Background Palette. |
| `HB_OT_NAME_ID_VARIATIONS_PS_PREFIX` | 25 | Variations PostScript Name Prefix. |
| `HB_OT_NAME_ID_INVALID` | 0xFFFF | Value to represent a nonexistent name ID. |

`HB_OT_NAME_ID_INVALID` is the sentinel other headers return when a font does not
supply a name for something — an axis with no `axisNameID`, a feature with no UI
label, a palette with no label. It is not special to the fetch functions: asking
for name ID `0xFFFF` simply looks it up and, in any sane font, finds nothing and
returns 0.

The enumeration type itself is `Since: 7.0.0` in the header — that is when the
loose constants were gathered under a documented `typedef` — but the constants
themselves predate it. Upstream's `NEWS` records `HB_OT_NAME_ID_COPYRIGHT`
through `HB_OT_NAME_ID_VARIATIONS_PS_PREFIX` as new in 2.1.0, alongside
`hb_ot_name_entry_t` and all four functions; `HB_OT_NAME_ID_INVALID` shipped in
2.0.0 as `HB_NAME_ID_INVALID` and was renamed in 2.1.0.

### `hb_ot_name_entry_t`

```c
typedef struct hb_ot_name_entry_t {
  hb_ot_name_id_t name_id;
  /*< private >*/
  hb_var_int_t    var;
  /*< public >*/
  hb_language_t   language;
} hb_ot_name_entry_t;
```

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_ot_name_entry_t {
    pub name_id: hb_ot_name_id_t,
    pub var: hb_var_int_t,
    pub language: hb_language_t,
}
```

A structure representing a name ID in a particular language: one element of the
array returned by `hb_ot_name_list_names()`. You never construct one; you read
them out of that array and feed the pair back into a fetch function.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `name_id` | `hb_ot_name_id_t` | `c_uint` | The name ID this entry stands for. Any value the font used — predefined or font-specific. |
| `var` | `hb_var_int_t` | `hb_var_int_t` | **Private.** HarfBuzz packs the record's encoding score into `var.u16[0]` and its index within the `name` table into `var.u16[1]`. Present only so the Rust layout matches C. Do not read or write it; its contents are not API. |
| `language` | `hb_language_t` | `*const hb_language_impl_t` | The language the string is in, as an interned BCP 47 tag. Compare with `==`, or render with `hb_language_to_string()`. Entries whose platform/language codes could not be mapped to a tag are dropped from the array, so in a normal build this is never `HB_LANGUAGE_INVALID`. |

`hb_var_int_t` is a `union` and therefore has no `Debug`, so this crate derives
only `Clone` and `Copy` and supplies a hand-written `Debug` that prints
`name_id` and `language` and elides the private slot. There is no `PartialEq`:
compare the two public fields yourself.

Since HarfBuzz 2.1.0.

## Functions

### Enumeration

#### `hb_ot_name_list_names`

```c
const hb_ot_name_entry_t *
hb_ot_name_list_names (hb_face_t    *face,
                       unsigned int *num_entries /* OUT */);
```

```rust
pub fn hb_ot_name_list_names(
    face: *mut hb_face_t,
    num_entries: *mut c_uint,
) -> *const hb_ot_name_entry_t;
```

Enumerates all available name IDs and language combinations.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Required; the implementation dereferences it immediately to reach `face->table.name`. The header documents no behaviour for null — do not pass it. `hb_face_get_empty()` is fine and yields zero entries. |
| `num_entries` | Out parameter, **optional** — gtk-doc marks it `(out) (optional)`, and the implementation checks it for null. When non-null it receives the number of entries in the returned array. Passing null means you get a pointer with no length, which is only useful if you already know the count. |

**Returns** — a pointer to an array of `num_entries` entries. gtk-doc marks it
`(transfer none) (array length=num_entries)`. For a face with no `name` table,
or one whose table failed sanitisation, the array is empty and `*num_entries` is
0; the pointer in that case is whatever the empty internal vector holds, so
always trust the count rather than testing the pointer for null.

**Ownership** — the array is owned by `face` and should not be modified. It can
be used for as long as `face` is alive. Nothing is copied for you; nothing is
allocated for you to free. Once the last reference to the face is released the
array dangles. If you need the data to outlive the face, copy it out, or keep a
reference with `hb_face_reference()`.

**Notes** — Since HarfBuzz 2.1.0.

The array is not raw table order. HarfBuzz builds it once, lazily, on first use
and caches it on the face; construction is internally synchronised, so it is safe
to call from several threads at once. What you get back has been:

* **decoded** — each record's platform and language IDs are mapped to a BCP 47
  `hb_language_t` (platform 3 via the Microsoft language-code table, platform 1
  via the Macintosh table, platform 0 via the font's `ltag` table);
* **filtered** — records whose language could not be mapped and records in
  encodings HarfBuzz cannot decode are dropped entirely;
* **deduplicated** — where several records share a *(name ID, language)* pair,
  only the best-encoded one survives, so each pair appears exactly once;
* **sorted** — by name ID ascending, then by language tag in `strcmp` order.

That ordering is an implementation detail rather than a documented guarantee, but
it is stable within a build and it is what makes the binary search in the fetch
functions work.

The encoding preference used for deduplication, best first, is: Windows/UCS-4
(3,10), Unicode 32-bit (0,6), Unicode 32-bit BMP-only (0,4), Windows/BMP (3,1),
Unicode (0,3), (0,2), (0,1), (0,0), Windows/Symbol (3,0), and finally
Macintosh/Roman (1,0), which HarfBuzz treats as ASCII. Anything else is
unsupported and disappears.

### Fetching name strings

The three fetch functions differ only in the output encoding. Everything below —
the lookup, the language fallback, the buffer protocol, the return value —
applies to all three; only the unit of `text_size` and of the return value
changes.

#### `hb_ot_name_get_utf8`

```c
unsigned int
hb_ot_name_get_utf8 (hb_face_t       *face,
                     hb_ot_name_id_t  name_id,
                     hb_language_t    language,
                     unsigned int    *text_size /* IN/OUT */,
                     char            *text      /* OUT */);
```

```rust
pub fn hb_ot_name_get_utf8(
    face: *mut hb_face_t,
    name_id: hb_ot_name_id_t,
    language: hb_language_t,
    text_size: *mut c_uint,
    text: *mut c_char,
) -> c_uint;
```

Fetches a font name from the OpenType `name` table, returning it in UTF-8.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to read from. Required; dereferenced immediately. `hb_face_get_empty()` is safe and finds nothing. |
| `name_id` | The name identifier to fetch. Any `unsigned int`; nothing is rejected, an unknown ID simply misses. Note the Rust type mismatch with the `HB_OT_NAME_ID_*` constants — see *Pitfalls*. |
| `language` | The language to fetch the name for. `HB_LANGUAGE_INVALID` (a null `hb_language_t`) means English: the implementation substitutes `hb_language_from_string("en", 2)`. Build one with `hb_language_from_string()`, or take one straight from an `hb_ot_name_entry_t`. |
| `text_size` | In/out, **optional**. On input: the capacity of `text`, in bytes, *including* room for the NUL terminator. On output: the number of bytes actually written, *excluding* the terminator. Null, or a pointed-to value of 0, means "write nothing, just tell me the length". |
| `text` | Caller-allocated output buffer of `*text_size` bytes. Only read/written when `text_size` is non-null *and* `*text_size` is non-zero; in every other case it is untouched and may be null. |

**Returns** — the full length of the requested string in bytes, not counting the
NUL, or 0 if the name was not found. This is the length of the *whole* string,
which may be larger than what fitted in your buffer; that is how you detect
truncation and how the two-pass idiom below works.

**Ownership** — nothing. The bytes are decoded into your buffer; HarfBuzz keeps
no pointer to it and you free nothing. The face is borrowed, not referenced.

**Notes** — Since HarfBuzz 2.1.0.

The exact write behaviour, which the header states only in summary:

* If a matching record exists and `text_size` is non-null with `*text_size > 0`,
  HarfBuzz reserves one unit for the terminator and transcodes up to
  `*text_size - 1` bytes, never splitting a multi-byte sequence. It then sets
  `*text_size` to the number of bytes written and writes a NUL at that offset.
  The return value is still the full length.
* If `text_size` is null, or `*text_size` is 0, nothing is written at all — not
  even a NUL — and the return value is the full length. `text` is never
  dereferenced in this case.
* If no matching record exists: when `text_size` is non-null and `*text_size` is
  non-zero, `text[0]` is set to 0; then `*text_size` is set to 0 and the function
  returns 0. So a "not found" with a non-empty buffer still leaves you a valid
  empty C string.

Source records are stored either as UTF-16BE or, for Macintosh/Roman records, as
ASCII. Bytes that do not decode — a malformed surrogate, or any byte ≥ 0x80 in a
Macintosh/Roman record — are replaced with U+FFFD rather than causing failure.

Lookup is a binary search for an *exact* `(name_id, language)` match first. If
that misses, a second search accepts a looser match, where the font's language is
a less specific tag than the one you asked for: a request for `en-us` is
satisfied by a record tagged `en`. The relation is one-way — see *Pitfalls*.

#### `hb_ot_name_get_utf16`

```c
unsigned int
hb_ot_name_get_utf16 (hb_face_t       *face,
                      hb_ot_name_id_t  name_id,
                      hb_language_t    language,
                      unsigned int    *text_size /* IN/OUT */,
                      uint16_t        *text      /* OUT */);
```

```rust
pub fn hb_ot_name_get_utf16(
    face: *mut hb_face_t,
    name_id: hb_ot_name_id_t,
    language: hb_language_t,
    text_size: *mut c_uint,
    text: *mut u16,
) -> c_uint;
```

Identical to `hb_ot_name_get_utf8()` in every respect except the output
encoding: the string is written as UTF-16 in host byte order, `text_size` counts
`uint16_t` units rather than bytes, and the return value is the full length in
`uint16_t` units. A character outside the BMP costs two units, so this is *not*
a character count. The terminator is a single zero unit and is excluded from
`*text_size`.

Useful when handing names to a UTF-16 platform API — Windows `wchar_t`, Java,
JavaScript, ICU's `UChar`. Note that the source records are usually UTF-16BE, so
this path still goes through a full decode/encode round trip; it is not a memcpy.

**Returns** — full length in `u16` units, or 0 if not found.
**Ownership** — none; caller-allocated buffer, borrowed face.
**Notes** — Since HarfBuzz 2.1.0.

#### `hb_ot_name_get_utf32`

```c
unsigned int
hb_ot_name_get_utf32 (hb_face_t       *face,
                      hb_ot_name_id_t  name_id,
                      hb_language_t    language,
                      unsigned int    *text_size /* IN/OUT */,
                      uint32_t        *text      /* OUT */);
```

```rust
pub fn hb_ot_name_get_utf32(
    face: *mut hb_face_t,
    name_id: hb_ot_name_id_t,
    language: hb_language_t,
    text_size: *mut c_uint,
    text: *mut u32,
) -> c_uint;
```

Identical to `hb_ot_name_get_utf8()` except that the string is written as UTF-32
in host byte order. `text_size` counts `uint32_t` units, which for UTF-32 *is* a
Unicode code-point count, and the return value is the full length in code points.
The terminator is a single zero unit and is excluded from `*text_size`.

This is the convenient form if you want to iterate code points without decoding,
or if you are feeding the result to something that speaks `char32_t` / Rust
`char`. Every unit written is a scalar value in `0..=0x10FFFF` with surrogates
already resolved or replaced, so `char::from_u32()` on each unit will succeed.

**Returns** — full length in code points, or 0 if not found.
**Ownership** — none; caller-allocated buffer, borrowed face.
**Notes** — Since HarfBuzz 2.1.0.

## Usage

### Fetch one name into a fixed buffer

The common case, in C:

```c
char text[128];
unsigned int text_size = sizeof (text);   /* includes room for the NUL */

unsigned int full = hb_ot_name_get_utf8 (face,
                                         HB_OT_NAME_ID_FONT_FAMILY,
                                         HB_LANGUAGE_INVALID,   /* => "en" */
                                         &text_size,
                                         text);
if (full == 0)
  { /* no such name in this font */ }
else if (full >= sizeof (text))
  { /* truncated: `text_size` bytes were written, `full` were available */ }
else
  printf ("family: %s\n", text);
```

The same in Rust:

```rust
use core::ffi::{c_char, c_uint};
use harfbuzz_sys::{
    HB_LANGUAGE_INVALID, HB_OT_NAME_ID_FONT_FAMILY, hb_ot_name_get_utf8, hb_ot_name_id_t,
};

let mut buf = [0u8; 128];
let mut text_size = buf.len() as c_uint;

// SAFETY: `face` is a live face; `buf` and `text_size` are valid locals.
let full = unsafe {
    hb_ot_name_get_utf8(
        face,
        HB_OT_NAME_ID_FONT_FAMILY as hb_ot_name_id_t,
        HB_LANGUAGE_INVALID,
        &mut text_size,
        buf.as_mut_ptr() as *mut c_char,
    )
};

let family: Option<&str> = if full == 0 {
    None // not found, or the record is an empty string
} else {
    // `text_size` is now the number written, excluding the NUL.
    core::str::from_utf8(&buf[..text_size as usize]).ok()
};
```

`from_utf8` is safe to use on the prefix: HarfBuzz never splits a multi-byte
sequence at the buffer boundary, and invalid input has already been replaced with
U+FFFD.

### Fetch a name of unknown length, exactly

Call once with no buffer to learn the length, then once more to fill it:

```c
unsigned int len = hb_ot_name_get_utf8 (face, name_id, language, NULL, NULL);
if (len)
{
  char *text = malloc (len + 1);        /* +1 for the NUL */
  unsigned int text_size = len + 1;
  hb_ot_name_get_utf8 (face, name_id, language, &text_size, text);
  /* text_size == len here */
  ...
  free (text);
}
```

```rust
use core::ffi::{c_char, c_uint};

// Pass 1: length only. `text` is never dereferenced when `text_size` is null.
// SAFETY: `face` is a live face.
let len = unsafe {
    hb_ot_name_get_utf8(face, name_id, language, core::ptr::null_mut(), core::ptr::null_mut())
};

let name: Option<String> = if len == 0 {
    None
} else {
    let mut buf = vec![0u8; len as usize + 1]; // +1 for the NUL
    let mut text_size = buf.len() as c_uint;
    // SAFETY: `buf` has `text_size` bytes of capacity.
    unsafe {
        hb_ot_name_get_utf8(
            face,
            name_id,
            language,
            &mut text_size,
            buf.as_mut_ptr() as *mut c_char,
        )
    };
    buf.truncate(text_size as usize);
    String::from_utf8(buf).ok()
};
```

Passing `len + 1` is what makes the second call exact. Passing `len` would write
`len - 1` bytes and a NUL.

(`harfbuzz-sys` is itself `#![no_std]`; the `Vec`/`String` here belong to the
calling crate. On a `no_std` target, size a fixed array or an `alloc::vec::Vec`
the same way.)

### Enumerate everything a face offers

```c
unsigned int count;
const hb_ot_name_entry_t *entries = hb_ot_name_list_names (face, &count);

for (unsigned int i = 0; i < count; i++)
{
  char text[256];
  unsigned int text_size = sizeof (text);

  hb_ot_name_get_utf8 (face,
                       entries[i].name_id,
                       entries[i].language,
                       &text_size,
                       text);

  printf ("%u (%s): %s\n",
          entries[i].name_id,
          hb_language_to_string (entries[i].language),
          text);
}
```

```rust
use core::ffi::c_uint;
use harfbuzz_sys::{hb_ot_name_entry_t, hb_ot_name_list_names};

let mut count: c_uint = 0;
// SAFETY: `face` is a live face; `count` is a valid local.
let ptr = unsafe { hb_ot_name_list_names(face, &mut count) };

// SAFETY: HarfBuzz guarantees `count` initialised entries at `ptr`, owned by
// `face` and valid for as long as `face` lives. Trust `count`, not `ptr`.
let entries: &[hb_ot_name_entry_t] = if count == 0 {
    &[]
} else {
    unsafe { core::slice::from_raw_parts(ptr, count as usize) }
};

for entry in entries {
    // entry.name_id is already an hb_ot_name_id_t; entry.language is ready to
    // pass straight back into a fetch call.
    let _ = (entry.name_id, entry.language);
}
```

Because the array is sorted by name ID and then by language, all the languages
for one name ID are contiguous — handy if you want to build a per-ID map of
localisations in one pass.

### Ask for a specific language

```c
hb_language_t ja = hb_language_from_string ("ja", -1);

unsigned int text_size = sizeof (text);
unsigned int full = hb_ot_name_get_utf8 (face, HB_OT_NAME_ID_FULL_NAME,
                                         ja, &text_size, text);
if (!full)
{
  /* No Japanese full name (and no less-specific match). Fall back: */
  text_size = sizeof (text);
  full = hb_ot_name_get_utf8 (face, HB_OT_NAME_ID_FULL_NAME,
                              HB_LANGUAGE_INVALID, &text_size, text);
}
```

```rust
use harfbuzz_sys::hb_language_from_string;

// SAFETY: the string literal is NUL-terminated, and -1 tells HarfBuzz so.
let ja = unsafe { hb_language_from_string(c"ja".as_ptr(), -1) };
```

There is no "give me whatever language you have" mode. If you want a
best-effort name regardless of language, ask for your preferred language, then
fall back to English, then — if you really need something — scan
`hb_ot_name_list_names()` for any entry with the ID you want.

### Preferring the typographic family name

Modern family naming lives in IDs 16/17, with 1/2 as the legacy fallback. The
usual recipe:

```c
static unsigned int
get_family (hb_face_t *face, char *buf, unsigned int size)
{
  unsigned int n = size;
  unsigned int full = hb_ot_name_get_utf8 (face, HB_OT_NAME_ID_TYPOGRAPHIC_FAMILY,
                                           HB_LANGUAGE_INVALID, &n, buf);
  if (full)
    return full;

  n = size;
  return hb_ot_name_get_utf8 (face, HB_OT_NAME_ID_FONT_FAMILY,
                              HB_LANGUAGE_INVALID, &n, buf);
}
```

Note that `n` must be reset before the second call: the first call overwrote it
with 0.

## Pitfalls

### The constants and the parameter have different Rust types

`HB_OT_NAME_ID_*` are `hb_ot_name_id_predefined_t` (`c_int`, per this crate's
enum rule), while the fetch functions take `hb_ot_name_id_t` (`c_uint`). Rust
will not coerce between them, so a call site needs a cast:

```rust
hb_ot_name_get_utf8(face, HB_OT_NAME_ID_FONT_FAMILY as hb_ot_name_id_t, ...)
```

The same applies when comparing a name ID from another API against the sentinel:
`if axis.name_id == HB_OT_NAME_ID_INVALID as hb_ot_name_id_t`. In C the
enumeration converts implicitly and this problem does not exist, so C examples
copied verbatim will not compile.

### `text_size` counts the terminator on the way in but not on the way out

This asymmetry is the single most common bug with this API. On input,
`*text_size` is the buffer's capacity and HarfBuzz immediately subtracts one to
reserve the NUL. On output, `*text_size` is the number of units written, with the
NUL sitting just past them. So:

* A buffer sized to exactly the value returned by the length-only call will
  truncate by one unit. Allocate `len + 1`.
* `*text_size` after the call is a valid slice length for the text alone —
  do not add one when slicing.

### The return value is the full length, not what was written

`hb_ot_name_get_utf8()` returns how long the string *is*, not how much of it you
got. To detect truncation, compare the return value with the buffer capacity you
passed in (`full >= capacity` means truncated), or with `*text_size` after the
call (`full != *text_size` means truncated). Using the return value as a slice
length on a buffer that was too small reads uninitialised memory.

### Zero is ambiguous, and `*text_size` is clobbered on failure

A return of 0 means "no record matched" *or* "the record exists and is the empty
string". Nothing distinguishes them. Separately, on the not-found path HarfBuzz
sets `*text_size` to 0 and writes a NUL into `text[0]` — so an in/out variable you
intended to reuse for a second call has been reset, and you must reload it with
the buffer capacity before every call (see the family-name example above).

### Language matching is one-way

The fallback search asks whether the *font's* tag is a less specific version of
*yours*. A font that tags a record `en` will answer a request for `en`, `en-us`,
or `en-gb`. A font that tags a record only `en-us` will answer a request for
`en-us` but **not** a request for `en` — and therefore not the default
`HB_LANGUAGE_INVALID` request either, since that becomes exactly `en`. If a name
you can see in a font inspector comes back as "not found", check the exact tag in
`hb_ot_name_list_names()` output before assuming the font is broken.

Matching is also textual, on the interned tag string, with `-` as the only
recognised separator.

### `HB_LANGUAGE_INVALID` means English, not "any"

Passing a null language is not a wildcard. It is rewritten to `en` before the
lookup, so a font localised only into Japanese will return nothing for it.

### The returned array belongs to the face

`hb_ot_name_list_names()` hands back interior pointers into a cache owned by the
face. Do not free it, do not write through it, and do not let it outlive the
face — including the case where a Rust wrapper drops the face while a
`&[hb_ot_name_entry_t]` derived from it is still alive. There is no reference
taken on your behalf.

Also, always take the length from `num_entries`. On a face with no `name` table
the count is 0 and the pointer is not guaranteed to be null, so a null check is
not a substitute.

### The `var` field is not yours

`hb_ot_name_entry_t.var` is `/*< private >*/` in the header and is public in Rust
only because `#[repr(C)]` layout demands it. It currently holds an encoding score
and a table index. Reading it is meaningless across versions; writing it into an
entry you then pass around corrupts nothing (the fetch functions take the fields
individually) but tells you nothing either.

### Macintosh/Roman records lose their accents

HarfBuzz treats platform 1, encoding 0 records as ASCII, not as Mac Roman. Any
byte ≥ 0x80 in such a record decodes to U+FFFD. Fonts that only ship a
Macintosh/Roman name for a non-English language will therefore produce mojibake
rather than the intended text. There is nothing in this API to work around it;
prefer the Windows/Unicode record, which HarfBuzz already does when both exist.

### Names are face data, not font data

Nothing here takes an `hb_font_t`. Setting variations, `ptem`, or a synthetic
slant cannot change what these functions return. In particular, the name of the
*named instance* a variable font is currently set to is not available through
this header — you get it from `hb_ot_var_named_instance_get_subfamily_name_id()`
and then pass that ID here.

### Thread safety

Both entry points only read. The per-face name index is built lazily on first
use behind HarfBuzz's internal lazy-loader lock, so concurrent first calls from
several threads are safe, and every call thereafter is a pure read of immutable
data. What is not safe is racing these calls against something that destroys the
face.

### Reduced-feature builds

Under `HB_NO_NAME` the whole implementation file is compiled out and these four
symbols do not exist in the library, even though the header still declares them —
a link error rather than a compile error. `HB_NO_NAME` also implies
`HB_NO_OT_NAME_LANGUAGE`; a build with just that latter macro would map every
record's language to `HB_LANGUAGE_INVALID`, and since such entries are filtered
out, `hb_ot_name_list_names()` would report zero entries for every face. This
crate's default build enables both.
