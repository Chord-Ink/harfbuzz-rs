# OpenType metadata

Transcribed from `hb-ot-meta.h`. Rust module: `harfbuzz_sys::ot_meta`, glob
re-exported at the crate root.

## Overview

`hb-ot-meta.h` is HarfBuzz's reader for the OpenType `meta` table. Upstream's
own one-line description is *"Functions for fetching metadata from fonts."*
The `meta` table is a small, tag-keyed dictionary that a font may carry: each
entry is a four-byte tag paired with an arbitrary run of bytes. It has nothing
to do with shaping, layout, or glyph selection — it is a place for the font
vendor to record facts *about* the font that a font picker, a font manager, or
an installer might want to show a user.

The API is deliberately tiny: one open-ended tag type and two functions. You
enumerate which tags a face carries with `hb_ot_meta_get_entry_tags()`, and you
pull the bytes for one tag with `hb_ot_meta_reference_entry()`. There are no
objects to create and nothing to configure. The only thing you own afterwards
is the `hb_blob_t` returned by `hb_ot_meta_reference_entry()`, which you must
release with `hb_blob_destroy()`.

Two properties of the table shape the whole API. First, **HarfBuzz does not
interpret the payload.** An entry's value is returned as raw bytes in a blob;
HarfBuzz never parses it, never validates its encoding, and never NUL-terminates
it. The two tags HarfBuzz gives constants for — `dlng` and `slng` — happen to
hold ASCII text (comma-separated BCP 47-ish language/script tags such as
`ar,de,fa`), but that is the OpenType specification's rule, not something the
library enforces. Second, **the tag space is open.** A font may store any tag
at all; Apple's variant of the table defines `appl` and `bild`, and fonts in
the wild carry vendor tags such as `fslf` and `nacl`. `hb_ot_meta_tag_t` is
therefore a plain 32-bit tag with two named constants, not a closed
enumeration, and both functions accept and return unnamed tags without
complaint.

The `meta` table is per-**face**, not per-font: it is a property of the binary
typeface, unaffected by variation coordinates, point size, or anything else set
on an `hb_font_t`. HarfBuzz loads and sanity-checks it lazily the first time you
call either function on a face, then caches it for that face's lifetime. The
sanitizer requires the table's version field to be exactly 1; a table that fails
that check, or a face with no `meta` table at all, behaves identically to a face
with zero entries — no error is reported. There is no "does this face have a
`meta` table?" predicate; a zero return from `hb_ot_meta_get_entry_tags()` is
the closest thing.

Upstream compiles this entire feature out under `HB_NO_META`, which the `HB_LEAN`
and `HB_TINY` reduced-feature profiles both enable. Unlike some HarfBuzz
features, there is no stub: the two functions are simply not defined, so a
program that calls them against such a build fails to *link*, not to run. In
this crate that corresponds to the `lean` and `tiny` Cargo features; the default
configuration includes `meta` support.

Include path in C: `#include <hb-ot.h>` (the header refuses to be included
directly).

## Types

### `hb_ot_meta_tag_t`

```c
typedef enum {
/*
   HB_OT_META_TAG_APPL		= HB_TAG ('a','p','p','l'),
   HB_OT_META_TAG_BILD		= HB_TAG ('b','i','l','d'),
*/
  HB_OT_META_TAG_DESIGN_LANGUAGES	= HB_TAG ('d','l','n','g'),
  HB_OT_META_TAG_SUPPORTED_LANGUAGES	= HB_TAG ('s','l','n','g'),

  /*< private >*/
  _HB_OT_META_TAG_MAX_VALUE = HB_TAG_MAX_SIGNED /*< skip >*/
} hb_ot_meta_tag_t;
```

```rust
pub type hb_ot_meta_tag_t = core::ffi::c_int;
```

A metadata entry tag — the key half of one `meta` table record. Known tags are
listed in the
[OpenType `meta` table specification](https://docs.microsoft.com/en-us/typography/opentype/spec/meta).

You get values of this type out of `hb_ot_meta_get_entry_tags()`, and you pass
them into `hb_ot_meta_reference_entry()`. There is nothing to own or free.

The Rust transcription is a `c_int` alias plus constants rather than a Rust
`enum`, for two reasons. The C enumeration's private sentinel is
`HB_TAG_MAX_SIGNED` (`0x7FFFFFFF`), which pins the underlying C type at signed
`int`. And the value space is genuinely open: `hb_ot_meta_get_entry_tags()`
copies tags straight out of font data, so it can hand back any 32-bit value a
font contains. A Rust `enum` holding a value outside its variant list would be
undefined behaviour, so the crate does not use one.

| Constant | Tag | Value | Meaning |
| --- | --- | --- | --- |
| `HB_OT_META_TAG_DESIGN_LANGUAGES` | `dlng` | `0x646C6E67` | Design languages. Text, using only Basic Latin (ASCII) characters. Indicates languages and/or scripts for the user audiences that the font was primarily designed for. |
| `HB_OT_META_TAG_SUPPORTED_LANGUAGES` | `slng` | `0x736C6E67` | Supported languages. Text, using only Basic Latin (ASCII) characters. Indicates languages and/or scripts that the font is declared to be capable of supporting. |

Both constants are `Since: 2.6.0`, as is the type itself.

Three things the table above does not show:

- **`_HB_OT_META_TAG_MAX_VALUE` is not transcribed.** It is marked
  `/*< private >*/` and `/*< skip >*/` in the header — a type-width pin, not a
  usable value. It decides that the Rust alias is `c_int` and nothing else.
- **`HB_OT_META_TAG_APPL` and `HB_OT_META_TAG_BILD` are commented out** in the
  header, so they are not part of the API and are not transcribed. They are
  Apple's `appl` and `bild` tags, and fonts do carry them — you will see them
  come back from `hb_ot_meta_get_entry_tags()`. Build them yourself with
  `HB_TAG('a','p','p','l')` if you need to match on them.
- **Any other tag is legal.** `fslf`, `nacl`, and vendor-private tags all work
  in both functions.

Because `hb_ot_meta_tag_t` is `c_int` in Rust while `HB_TAG` returns
`hb_tag_t` (`u32`), constructing a tag on the fly needs a cast:

```rust
let appl = harfbuzz_sys::HB_TAG(b'a', b'p', b'p', b'l') as harfbuzz_sys::hb_ot_meta_tag_t;
```

The two provided constants are already cast. In C the equivalent cast is
`(hb_ot_meta_tag_t) HB_TAG ('a','p','p','l')`, which is what HarfBuzz's own
tests and the `hb-info` utility write.

## Functions

### Enumeration

#### `hb_ot_meta_get_entry_tags`

```c
HB_EXTERN unsigned int
hb_ot_meta_get_entry_tags (hb_face_t        *face,
                           unsigned int      start_offset,
                           unsigned int     *entries_count, /* IN/OUT.  May be NULL. */
                           hb_ot_meta_tag_t *entries        /* OUT.     May be NULL. */);
```

```rust
pub fn hb_ot_meta_get_entry_tags(
    face: *mut hb_face_t,
    start_offset: c_uint,
    entries_count: *mut c_uint,
    entries: *mut hb_ot_meta_tag_t,
) -> c_uint;
```

Fetches the tags of the metadata entries in a face. The header's own wording is
"Fetches all available feature types" — that is copy-paste from the AAT feature
API and is misleading; this function returns `meta` entry tags, not features.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Null is not documented and the implementation dereferences it immediately — do not pass null. `hb_face_get_empty()` is well defined and yields zero entries. |
| `start_offset` | Zero-based index of the first entry to report, in the table's own record order. Values past the end are not an error; they simply produce zero written entries. |
| `entries_count` | In/out, and **optional**. On input, the capacity of `entries`, in entries. On output, how many entries were actually written. May be null. |
| `entries` | Caller-allocated output array of at least `*entries_count` tags. May be null. |

**Returns** — the **total** number of metadata entries in the face, which is
independent of `start_offset`, of `*entries_count`, and of how many entries were
written. Zero means the face has no `meta` table, or has one that failed
sanitization, or has one with an empty record array — the three are
indistinguishable through this API. There is no error return.

**Ownership** — nothing is allocated and nothing is transferred. The function
reads `face` (loading and caching its `meta` table on first use) and writes into
memory the caller owns. Do not free anything.

**Notes**

- The tag values written are exactly the bytes stored in the font, read as
  big-endian 32-bit integers, so they compare equal to `HB_TAG(...)`
  constants. HarfBuzz does not filter, validate, sort, or deduplicate them.
- Entries come back in the table's stored order. The OpenType specification asks
  for records sorted by tag, but HarfBuzz neither requires nor enforces that —
  its lookup is a linear search — so do not rely on ordering.
- Since HarfBuzz 2.6.0.
- Thread-safe for concurrent readers of the same face. The first call may
  populate the face's cached `meta` accelerator, which is done with the same
  internal synchronisation as every other lazily loaded table.

**Clamping rules**, which the header does not spell out and which matter:

- If `start_offset` is greater than the total entry count, the written count is
  zero.
- Otherwise the written count is `min(total - start_offset, *entries_count)`.
- `*entries_count` is updated to that written count.

#### The `entries == NULL` special case

Read this before writing a "count first, then fetch" loop.

The implementation only touches the output parameters when **both**
`entries_count` and `entries` are non-null:

```cpp
if (count && entries)
{
  /* ... copy tags, and set *count to the number copied ... */
}
return table->dataMaps.len;
```

Consequences:

- `hb_ot_meta_get_entry_tags (face, 0, NULL, NULL)` is the idiomatic way to ask
  only for the total. This is what HarfBuzz's `hb-info` utility does.
- `hb_ot_meta_get_entry_tags (face, 0, &count, NULL)` also returns the total,
  but **leaves `count` untouched.** It is not set to zero and it is not set to
  the available count. If you pre-loaded `count` with a buffer capacity and then
  read it back expecting the number available, you will read back your own input
  value. Several other HarfBuzz enumerators behave the same way, but it is the
  opposite of the usual "pass NULL to size the buffer" C idiom, which is why
  upstream has a dedicated regression test for it
  (`test_nullable_output_meta_entries`).
- `hb_ot_meta_get_entry_tags (face, 0, NULL, entries)` writes nothing at all,
  because the guard requires both. Passing a buffer without a count is silently
  a no-op.

### Data retrieval

#### `hb_ot_meta_reference_entry`

```c
HB_EXTERN hb_blob_t *
hb_ot_meta_reference_entry (hb_face_t *face, hb_ot_meta_tag_t meta_tag);
```

```rust
pub fn hb_ot_meta_reference_entry(
    face: *mut hb_face_t,
    meta_tag: hb_ot_meta_tag_t,
) -> *mut hb_blob_t;
```

Fetches the metadata entry stored under `meta_tag`, as raw bytes.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to read from. Null is not documented and is dereferenced immediately — do not pass null. `hb_face_get_empty()` is well defined and yields the empty blob. |
| `meta_tag` | The tag to look up. Any 32-bit tag is accepted, not just the two named constants. Matching is exact; there is no case folding and no wildcard. |

**Returns** — a blob holding the entry's bytes, never null. Both "this face has
no `meta` table" and "this tag is not present" yield HarfBuzz's singleton empty
blob, which reports length 0. As in the rest of HarfBuzz, absence is signalled
by an empty object rather than `NULL`.

Note the ambiguity this creates: a tag that is *present but stores zero bytes*
is indistinguishable from a tag that is absent. Upstream's own test relies on
`nacl` returning length 0 for the "not present" case. If you need to
distinguish, enumerate with `hb_ot_meta_get_entry_tags()` and look for the tag
there.

**Ownership** — transfer full. **The caller owns a reference and must call
`hb_blob_destroy()`** on the result, including when it is the empty blob
(destroying the singleton is harmless and expected). The bytes themselves are
*not* copied: the returned blob is a sub-blob of the face's cached `meta` table
blob, created with `hb_blob_create_sub_blob()`. That has three follow-on
effects:

1. The sub-blob takes a reference on the whole `meta` table blob, so holding a
   short entry keeps the entire table's memory alive. Do not stash entry blobs
   indefinitely if memory matters.
2. Sub-blobs are always created with `HB_MEMORY_MODE_READONLY`, so
   `hb_blob_get_data_writable()` on the result will copy or fail rather than
   hand you the font's own bytes to scribble on.
3. Creating the sub-blob makes the parent `meta` blob immutable.

The blob's data remains valid for as long as you hold your reference, even if
the face is destroyed in the meantime, because the reference chain runs blob →
`meta` table blob → underlying font data.

**Notes**

- The payload is **not NUL-terminated** and has no guaranteed encoding. Always
  pair `hb_blob_get_data()` with `hb_blob_get_length()`.
- Lookup is a linear scan over the table's records, so cost is O(number of
  entries). That number is small in practice.
- Since HarfBuzz 2.6.0.
- Thread-safe for concurrent readers of the same face, with the same lazy
  table-loading caveat as `hb_ot_meta_get_entry_tags()`.

## Usage

The examples below use `meta.ttf` from HarfBuzz's own test suite, which carries
five entries — `appl`, `bild`, `dlng`, `fslf`, `slng` — where `dlng` is the
8-byte string `ar,de,fa` and `slng` is the 11-byte string `ar,de,en,fa`.

### Read one known entry

C:

```c
#include <hb-ot.h>

hb_blob_t *blob = hb_ot_meta_reference_entry (face, HB_OT_META_TAG_DESIGN_LANGUAGES);

unsigned int len;
const char *data = hb_blob_get_data (blob, &len);

/* Not NUL-terminated: use the length. */
printf ("dlng: %.*s\n", (int) len, data);   /* dlng: ar,de,fa */

hb_blob_destroy (blob);
```

Rust:

```rust
use core::ffi::c_uint;
use core::slice;

use harfbuzz_sys::{
    HB_OT_META_TAG_DESIGN_LANGUAGES, hb_blob_destroy, hb_blob_get_data,
    hb_ot_meta_reference_entry,
};

unsafe {
    let blob = hb_ot_meta_reference_entry(face, HB_OT_META_TAG_DESIGN_LANGUAGES);

    let mut len: c_uint = 0;
    let data = hb_blob_get_data(blob, &mut len);

    // `data` is null only if the blob is empty; guard before building a slice.
    let bytes = if data.is_null() || len == 0 {
        &[][..]
    } else {
        slice::from_raw_parts(data as *const u8, len as usize)
    };

    let text = core::str::from_utf8(bytes).unwrap_or("");
    // text == "ar,de,fa"

    hb_blob_destroy(blob);
}
```

### Enumerate every entry

The two-pass idiom: ask for the total with both output parameters null, then
fetch in one go.

C:

```c
unsigned int total = hb_ot_meta_get_entry_tags (face, 0, NULL, NULL);
if (total)
{
  hb_ot_meta_tag_t *tags = malloc (total * sizeof (hb_ot_meta_tag_t));
  unsigned int count = total;

  hb_ot_meta_get_entry_tags (face, 0, &count, tags);
  /* count is now min(total, capacity) == total */

  for (unsigned int i = 0; i < count; i++)
  {
    hb_blob_t *blob = hb_ot_meta_reference_entry (face, tags[i]);
    printf ("%c%c%c%c\t%.*s\n",
            HB_UNTAG (tags[i]),
            (int) hb_blob_get_length (blob),
            hb_blob_get_data (blob, NULL));
    hb_blob_destroy (blob);
  }

  free (tags);
}
```

Rust:

```rust
use core::ffi::c_uint;
use core::ptr;

use harfbuzz_sys::{hb_ot_meta_get_entry_tags, hb_ot_meta_tag_t};

unsafe {
    let total = hb_ot_meta_get_entry_tags(face, 0, ptr::null_mut(), ptr::null_mut());

    let mut tags: Vec<hb_ot_meta_tag_t> = vec![0; total as usize];
    let mut count: c_uint = total;

    if total != 0 {
        hb_ot_meta_get_entry_tags(face, 0, &mut count, tags.as_mut_ptr());
        tags.truncate(count as usize);
    }

    for tag in &tags {
        let bytes = (*tag as u32).to_be_bytes();
        // e.g. b"dlng"
        let _ = bytes;
    }
}
```

### Paged enumeration

`start_offset` lets you walk the list with a fixed-size buffer. HarfBuzz's
`hb-info` utility takes this to the extreme and fetches one tag at a time:

```c
unsigned int count = hb_ot_meta_get_entry_tags (face, 0, NULL, NULL);
for (unsigned int i = 0; i < count; i++)
{
  hb_ot_meta_tag_t tag;
  unsigned int len = 1;
  hb_ot_meta_get_entry_tags (face, i, &len, &tag);
  /* ... use tag ... */
}
```

A more typical page loop, with the clamping behaviour made explicit:

```c
unsigned int total  = hb_ot_meta_get_entry_tags (face, 0, NULL, NULL);
unsigned int offset = 0;

while (offset < total)
{
  hb_ot_meta_tag_t page[8];
  unsigned int n = 8;                     /* must be re-set every iteration */

  hb_ot_meta_get_entry_tags (face, offset, &n, page);
  if (!n) break;                          /* defensive: offset past the end */

  for (unsigned int i = 0; i < n; i++)
    handle (page[i]);

  offset += n;
}
```

Re-setting `n` each time round is essential: the call overwrites it with the
number written, so leaving it at the previous result shrinks the window on every
iteration.

### Look up a tag with no constant

Both Apple's tags and vendor-private ones work fine; you just have to build the
tag yourself.

C:

```c
hb_blob_t *appl = hb_ot_meta_reference_entry (face, (hb_ot_meta_tag_t) HB_TAG ('a','p','p','l'));
hb_blob_destroy (appl);

/* Or from a string, e.g. a command-line argument: */
hb_ot_meta_tag_t tag = (hb_ot_meta_tag_t) hb_tag_from_string ("fslf", -1);
hb_blob_t *blob = hb_ot_meta_reference_entry (face, tag);
hb_blob_destroy (blob);
```

Rust:

```rust
use harfbuzz_sys::{HB_TAG, hb_blob_destroy, hb_ot_meta_reference_entry, hb_ot_meta_tag_t};

const APPL: hb_ot_meta_tag_t = HB_TAG(b'a', b'p', b'p', b'l') as hb_ot_meta_tag_t;

unsafe {
    let blob = hb_ot_meta_reference_entry(face, APPL);
    hb_blob_destroy(blob);
}
```

There is no `hb_ot_meta_tag_from_string()`; use `hb_tag_from_string()` and cast.

### Testing whether a face declares support for a language

`slng` is a comma-separated ASCII list, so this is string work, not HarfBuzz
work — and it is advisory metadata, not a `cmap` coverage test.

```c
static hb_bool_t
declares_language (hb_face_t *face, const char *bcp47)
{
  hb_blob_t *blob = hb_ot_meta_reference_entry (face, HB_OT_META_TAG_SUPPORTED_LANGUAGES);
  unsigned int len;
  const char *data = hb_blob_get_data (blob, &len);

  hb_bool_t found = false;
  /* Tokenise `data`/`len` on ',' and compare; remember: no NUL terminator. */
  ...

  hb_blob_destroy (blob);
  return found;
}
```

If you actually want to know whether a face can render a language, use the
`cmap` coverage APIs (`hb_face_collect_unicodes()`) or the OpenType layout
language-system queries (`hb_ot_layout_table_select_script()` and friends).
`slng` is what the vendor claimed, which is a different question.

## Pitfalls

- **`entries_count` is not updated when `entries` is NULL.** The write is
  guarded on both pointers being non-null. Pass both nulls to get just the
  total; do not pass a count pointer alone and expect it to be filled in. See
  the dedicated section above.
- **The return value is the total, not the number written.** It ignores
  `start_offset` entirely. `hb_ot_meta_get_entry_tags (face, 99, &n, buf)` on a
  five-entry face still returns 5, with `n` set to 0. Loop on the written count,
  not the return value.
- **Re-set `*entries_count` before every call** in a loop. It is in/out, and the
  out value is the number written.
- **The payload is not NUL-terminated and not validated.** Nothing guarantees
  `dlng`/`slng` contain ASCII, or contain anything sensible at all. Always use
  the length from `hb_blob_get_length()` / `hb_blob_get_data()`, and treat
  UTF-8 decoding as fallible.
- **Empty blob means "absent *or* empty".** A missing tag, a face with no
  `meta` table, and a present-but-zero-length entry all return a zero-length
  blob. `hb_blob_get_data()` on the empty blob may hand back a pointer you must
  not read past length zero — check the length first.
- **You must destroy the returned blob.** `hb_ot_meta_reference_entry` is a
  `reference_*` function: transfer full. Forgetting `hb_blob_destroy()` leaks
  both the sub-blob and its reference to the whole `meta` table.
- **Entry blobs pin the whole table.** The result is a sub-blob of the `meta`
  table blob, not a copy, so an 8-byte `dlng` string can keep a multi-kilobyte
  table resident. Copy the bytes out if you intend to keep them for a long time.
- **Do not expect writable data.** The sub-blob is read-only, and taking it also
  makes the underlying `meta` blob immutable.
- **`hb_ot_meta_tag_t` is signed in Rust (`c_int`) but tags are unsigned.** Cast
  through `u32` before formatting or comparing with `hb_tag_t` values;
  `(tag as u32).to_be_bytes()` is the Rust equivalent of `HB_UNTAG`.
- **The header's own doc comment for `hb_ot_meta_get_entry_tags` says "feature
  types".** It is wrong — copied from the AAT feature enumerator. There are no
  features here.
- **`HB_OT_META_TAG_APPL` and `HB_OT_META_TAG_BILD` do not exist.** They are
  commented out in the header, yet fonts do carry `appl` and `bild` entries, so
  they show up in enumeration output. Construct them with `HB_TAG` if you need
  them.
- **Ordering is not guaranteed.** Records are returned in stored order, and
  HarfBuzz does not require the sorted order the specification asks for.
- **Silent absence, silent corruption.** A `meta` table whose version field is
  not 1 fails sanitization and is treated exactly like a missing table. There is
  no way to tell the two apart, and no diagnostic.
- **`HB_NO_META` builds do not link.** Under the `lean` or `tiny` Cargo
  features (upstream `HB_LEAN` / `HB_TINY`), both functions are compiled out
  entirely rather than stubbed, so calls become undefined symbols at link time.
- **Face-level, not font-level.** Nothing here reads an `hb_font_t`, so
  variation settings and point size have no effect. Pass the face.
- **Null `face` is unspecified.** The header documents nothing, and the
  implementation dereferences `face` immediately. Pass a real face, or
  `hb_face_get_empty()`.
