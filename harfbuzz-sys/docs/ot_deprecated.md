# Deprecated OpenType API

Transcribed from `hb-ot-deprecated.h`. Rust module: `harfbuzz_sys::ot_deprecated`,
glob re-exported at the crate root.

## Overview

`hb-ot-deprecated.h` is the attic of HarfBuzz's OpenType API. It holds nothing
new: every item in it is a superseded spelling, a superseded type, or a
superseded function that HarfBuzz keeps exporting purely so that programs
compiled against older versions keep linking and keep behaving the same way.
There are **three `#define`s, one struct, and six functions**, and each one
names its replacement in the header itself. Upstream's rule for this file is
"add nothing, remove nothing" — items land here when a better API arrives, and
they stay forever.

The contents fall into three unrelated groups, which is worth knowing because
they migrate in three completely different ways:

- **Naming fossils** (`HB_MATH_GLYPH_PART_FLAG_EXTENDER`, `HB_OT_MATH_SCRIPT`)
  are plain aliases for constants that live in `hb-ot-math.h`. They are pure
  renames — same value, same type — except that `HB_OT_MATH_SCRIPT` is also a
  semantic trap, because for years the documentation told people to pass it to
  `hb_buffer_set_script()`, which was always wrong.
- **The pre-2.0 tag and layout lookups** (`hb_ot_layout_table_choose_script`,
  `hb_ot_layout_script_find_language`, `hb_ot_tags_from_script`,
  `hb_ot_tag_from_language`) date from when HarfBuzz assumed one script mapped
  to at most two OpenType tags and one language to exactly one. HarfBuzz 2.0
  replaced all four with two general functions — `hb_ot_layout_table_select_script()`,
  `hb_ot_layout_script_select_language()`, and `hb_ot_tags_from_script_and_language()`
  — that take counted arrays. Each deprecated function is now a thin shim that
  calls the new one; there is no separate code path, so behaviour is identical
  where the old signature can express the query at all.
- **The pre-2.2 variation-axis API** (`HB_OT_VAR_NO_AXIS_INDEX`,
  `hb_ot_var_axis_t`, `hb_ot_var_get_axes`, `hb_ot_var_find_axis`) shipped a
  struct with no axis index and no flags. Since a caller could not tell a hidden
  axis from a visible one, HarfBuzz 2.2 introduced `hb_ot_var_axis_info_t` with
  both, plus `hb_ot_var_get_axis_infos()` and `hb_ot_var_find_axis_info()`.

Nothing here allocates, nothing here is reference counted, and nothing here has
an object lifecycle of its own. Every function reads an `hb_face_t` (or nothing
at all) and writes into memory the caller already owns. There is no create, no
destroy, and no user data. That makes migration mechanical: change the call,
change the struct type, and you are done.

**The whole header is wrapped in `#ifndef HB_DISABLE_DEPRECATED`,** and so are
the implementations. In a build that defines that macro — which upstream's
`HB_LEAN` and `HB_TINY` profiles both do, and which this crate's `lean` and
`tiny` Cargo features therefore both do — these six functions are not compiled
at all. The Rust declarations still exist (the crate emits them
unconditionally), so calling one in such a build is a **link** error, not a
compile error and not a runtime error. The three `#define`s vanish from C
entirely under the same macro, though the Rust constants remain, since a
constant needs no symbol.

Include path in C: `#include <hb-ot.h>` (the header refuses to be included
directly). In Rust every item is at the crate root: `use harfbuzz_sys::…`.

### Migration at a glance

| Deprecated | Since | Deprecated in | Use instead |
| --- | --- | --- | --- |
| `HB_MATH_GLYPH_PART_FLAG_EXTENDER` | — | 2.5.1 | `HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER` |
| `HB_OT_MATH_SCRIPT` | 1.3.3 | 3.4.0 | `HB_SCRIPT_MATH` (for buffers) or `HB_OT_TAG_MATH_SCRIPT` (for tags) |
| `HB_OT_VAR_NO_AXIS_INDEX` | 1.4.2 | 2.2.0 | nothing — "do not use" |
| `hb_ot_var_axis_t` | 1.4.2 | 2.2.0 | `hb_ot_var_axis_info_t` |
| `hb_ot_layout_table_choose_script` | — | 2.0.0 | `hb_ot_layout_table_select_script` |
| `hb_ot_layout_script_find_language` | 0.6.0 | 2.0.0 | `hb_ot_layout_script_select_language` |
| `hb_ot_tags_from_script` | 0.6.0 | 2.0.0 | `hb_ot_tags_from_script_and_language` |
| `hb_ot_tag_from_language` | 0.6.0 | 2.0.0 | `hb_ot_tags_from_script_and_language` |
| `hb_ot_var_get_axes` | 1.4.2 | 2.2.0 | `hb_ot_var_get_axis_infos` |
| `hb_ot_var_find_axis` | 1.4.2 | 2.2.0 | `hb_ot_var_find_axis_info` |

In C every function carries `HB_DEPRECATED_FOR(replacement)`, which expands to
`__attribute__((deprecated("Use 'replacement' instead")))` on GCC/Clang and
`__declspec(deprecated(…))` on MSVC — so compiling a call produces a warning.
The Rust transcription mirrors that with `#[deprecated(note = "…")]` on every
item, including the three constants and the struct, which the C header does not
mark (a `#define` cannot carry an attribute).

## Constants

### `HB_MATH_GLYPH_PART_FLAG_EXTENDER`

```c
#define HB_MATH_GLYPH_PART_FLAG_EXTENDER HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER
```

```rust
#[deprecated(note = "use `HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER` instead")]
pub const HB_MATH_GLYPH_PART_FLAG_EXTENDER: hb_ot_math_glyph_part_flags_t =
    HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER;
```

| Property | Value |
| --- | --- |
| Type | `hb_ot_math_glyph_part_flags_t` (`core::ffi::c_int`) |
| Value | `0x00000001` |
| Meaning | This glyph part is an *extender*: it may be repeated as many times as needed to reach the desired length of a stretchy math glyph. |
| Deprecated | 2.5.1 |

An unprefixed spelling that predates HarfBuzz's `HB_OT_` naming convention, kept
after [issue #1734](https://github.com/harfbuzz/harfbuzz/issues/1734). It is a
literal alias — the same token, the same value, the same type — so swapping it
for `HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER` cannot change behaviour.

It is the only flag defined for `hb_ot_math_glyph_part_flags_t`, and you meet it
in the `flags` field of `hb_ot_math_glyph_part_t` values returned by
`hb_ot_math_get_glyph_assembly()`.

**Notes** — the C `#define` disappears under `HB_DISABLE_DEPRECATED`; the Rust
constant does not, because it needs no linker symbol. Nothing else about the
math API is deprecated.

### `HB_OT_MATH_SCRIPT`

```c
#define HB_OT_MATH_SCRIPT HB_OT_TAG_MATH_SCRIPT
```

```rust
#[deprecated(note = "use `HB_SCRIPT_MATH` or `HB_OT_TAG_MATH_SCRIPT` instead")]
pub const HB_OT_MATH_SCRIPT: hb_tag_t = HB_OT_TAG_MATH_SCRIPT;
```

| Property | Value |
| --- | --- |
| Type | `hb_tag_t` (`u32`) |
| Value | `HB_TAG('m','a','t','h')` = `0x6D617468` |
| Since | 1.3.3 |
| Deprecated | 3.4.0 |

The OpenType script tag `math`, under which a font's math layout features live
in `GSUB`/`GPOS`. Despite the name it is **not** an `hb_script_t` and never was.

The rename came with [PR #3417](https://github.com/harfbuzz/harfbuzz/pull/3417),
which split one badly-named constant into two correctly-named ones:

- **`HB_OT_TAG_MATH_SCRIPT`** (`hb-ot-math.h`) — the same value, `math`, for use
  wherever an OpenType script *tag* is expected: `hb_ot_layout_table_find_script()`,
  `hb_ot_layout_table_select_script()`, and friends.
- **`HB_SCRIPT_MATH`** (`hb-common.h`, since 3.4.0) — a real `hb_script_t`,
  `HB_TAG('Z','m','t','h')` = `0x5A6D7468`. `Zmth` is a *pseudo-script* code, not
  a registered ISO 15924 script, but it is what you pass to
  `hb_buffer_set_script()` to ask for math shaping.

**The pitfall this constant exists to warn about:** HarfBuzz's own documentation
once recommended `hb_buffer_set_script(buffer, HB_OT_MATH_SCRIPT)` to switch on
math shaping. That is no longer supported and never really worked — it fed a
`math` tag into a field that wants an ISO 15924-style script code. Use
`HB_SCRIPT_MATH` there instead.

### `HB_OT_VAR_NO_AXIS_INDEX`

```c
#define HB_OT_VAR_NO_AXIS_INDEX 0xFFFFFFFFu
```

```rust
#[deprecated(note = "do not use")]
pub const HB_OT_VAR_NO_AXIS_INDEX: c_uint = 0xFFFFFFFF;
```

| Property | Value |
| --- | --- |
| Type | `unsigned int` / `core::ffi::c_uint` (`u32`) |
| Value | `0xFFFFFFFF` |
| Since | 1.4.2 |
| Deprecated | 2.2.0 |

The header's own documentation for this constant is exactly two words: **"Do not
use."** It has no replacement, because the API that produced it has none — the
modern `hb_ot_var_find_axis_info()` does not report an index at all (the index is
inside `hb_ot_var_axis_info_t` instead, and is only set on success).

It does have one real meaning, and it is worth knowing when reading old code:
`hb_ot_var_find_axis()` stores `HB_OT_VAR_NO_AXIS_INDEX` into `*axis_index`
*before* it starts searching, and leaves it there if the tag is not found. So a
caller that ignored the boolean return could test
`*axis_index != HB_OT_VAR_NO_AXIS_INDEX` instead. Nothing else in HarfBuzz ever
returns this value.

## Types

### `hb_ot_var_axis_t`

```c
typedef struct hb_ot_var_axis_t {
  hb_tag_t tag;
  hb_ot_name_id_t name_id;
  float min_value;
  float default_value;
  float max_value;
} hb_ot_var_axis_t;
```

```rust
#[deprecated(note = "use `hb_ot_var_axis_info_t` instead")]
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct hb_ot_var_axis_t {
    pub tag: hb_tag_t,
    pub name_id: hb_ot_name_id_t,
    pub min_value: c_float,
    pub default_value: c_float,
    pub max_value: c_float,
}
```

The original description of one OpenType design-variation axis, as read from a
variable font's `fvar` table. You get one by passing a pointer to
`hb_ot_var_get_axes()` or `hb_ot_var_find_axis()`; the caller allocates it, the
callee fills it in, and there is nothing to free. It is a plain value type: copy
it, store it, and forget about it.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `tag` | `hb_tag_t` | `hb_tag_t` (`u32`) | The four-byte tag identifying the design variation, e.g. `wght`, `wdth`, `opsz`. Registered tags are listed in the [OpenType Axis Tag Registry](https://docs.microsoft.com/en-us/typography/opentype/spec/dvaraxisreg); fonts may also define private tags. |
| `name_id` | `hb_ot_name_id_t` | `hb_ot_name_id_t` (`c_uint`) | The `name` table Name ID that provides display names for this axis. Pass it to `hb_ot_name_get_utf8()` and friends. `HB_OT_NAME_ID_INVALID` (`0xFFFF`) means none. |
| `min_value` | `float` | `c_float` (`f32`) | Minimum value the font covers on this axis, in user (un-normalized) units — the numbers a person would type, such as `100` for thin weight. |
| `default_value` | `float` | `c_float` (`f32`) | The position on this axis corresponding to the font's default instance, in user units. |
| `max_value` | `float` | `c_float` (`f32`) | Maximum value the font covers, in user units. |

Layout: five 4-byte fields, so 20 bytes with 4-byte alignment. No padding, no
reserved words, no version field.

**Why it was replaced.** `hb_ot_var_axis_info_t` (since 2.2.0, `hb-ot-var.h`)
carries three things this struct does not:

| | `hb_ot_var_axis_t` | `hb_ot_var_axis_info_t` |
| --- | --- | --- |
| `axis_index` | absent | present — the axis's position in the face's array, which is what `hb_ot_var_normalize_coords()` and design-coordinate arrays are indexed by |
| `flags` | absent | present — notably `HB_OT_VAR_AXIS_FLAG_HIDDEN`, so a UI can skip axes the designer marked hidden |
| `reserved` | absent | present — zeroed padding that lets upstream extend the struct without breaking ABI |
| size | 20 bytes | 32 bytes |
| `tag`, `name_id`, `min_value`, `default_value`, `max_value` | present | present, identical semantics |

Because the shared fields are identical and carry the same values, converting is
a field-by-field copy.

**Value ranges, which both structs share.** HarfBuzz normalizes the ordering
before handing the values back: `default_value` is taken verbatim from the font,
then `min_value = min(default, font_min)` and `max_value = max(default, font_max)`.
A font that stores `min > default` or `max < default` therefore cannot make
HarfBuzz report an inverted range — the invariant `min ≤ default ≤ max` always
holds, so client arithmetic can rely on it. The stored values are `F16DOT16`
fixed-point and are converted to `float`.

**Notes** — Since 1.4.2, deprecated 2.2.0. The struct itself is available
whenever the header is (i.e. not under `HB_DISABLE_DEPRECATED`), but the only
functions that produce one are also deprecated, so in practice they come and go
together. In Rust the type derives `Debug, Clone, Copy` but not `PartialEq` —
three of its fields are floats.

## Functions

### OpenType layout lookups

#### `hb_ot_layout_table_choose_script`

```c
HB_DEPRECATED_FOR (hb_ot_layout_table_select_script)
HB_EXTERN hb_bool_t
hb_ot_layout_table_choose_script (hb_face_t      *face,
                                  hb_tag_t        table_tag,
                                  const hb_tag_t *script_tags,
                                  unsigned int   *script_index,
                                  hb_tag_t       *chosen_script);
```

```rust
pub fn hb_ot_layout_table_choose_script(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    script_tags: *const hb_tag_t,
    script_index: *mut c_uint,
    chosen_script: *mut hb_tag_t,
) -> hb_bool_t;
```

Selects one OpenType script from a **zero-terminated** array of candidate tags,
in order of preference, and reports where it sits in the face's `GSUB` or `GPOS`
script list. The header's own comment describes it as "like
`hb_ot_layout_table_find_script`, but takes zero-terminated array of scripts to
test".

The implementation is four lines: it walks `script_tags` to find the terminator,
computes the length, and forwards to `hb_ot_layout_table_select_script()`. The
counted-array form is the whole difference between the two functions.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Dereferenced immediately; do not pass null. `hb_face_get_empty()` is well defined and behaves as a face with no layout tables. |
| `table_tag` | `HB_OT_TAG_GSUB` (`GSUB`) or `HB_OT_TAG_GPOS` (`GPOS`). Any other tag yields HarfBuzz's null table object, i.e. no scripts found. |
| `script_tags` | Candidate script tags in descending order of preference, terminated by a zero tag (`HB_TAG_NONE`). **Must not be null** — the terminator scan dereferences it before anything else. An array holding only the terminator is legal and means "no candidates", which sends the call straight to the fallback chain. |
| `script_index` | Out, **may be null**. Receives the index of the selected script within the table's script list. |
| `chosen_script` | Out, **may be null**. Receives the tag actually selected. |

**Returns** — `true` **only if one of the tags you asked for was found.**
`false` covers three different outcomes, and you must read the outputs to tell
them apart:

| Situation | Return | `*script_index` | `*chosen_script` |
| --- | --- | --- | --- |
| A requested tag is present | `true` | its index | that tag |
| Not found, but `DFLT` is present | `false` | index of `DFLT` | `HB_TAG('D','F','L','T')` |
| Not found, but `dflt` is present | `false` | index of `dflt` | `HB_TAG('d','f','l','t')` |
| Not found, but `latn` is present | `false` | index of `latn` | `HB_TAG('l','a','t','n')` |
| Nothing at all | `false` | `HB_OT_LAYOUT_NO_SCRIPT_INDEX` (`0xFFFF`) | `HB_TAG_NONE` (`0`) |

The `dflt` fallback exists because Microsoft's specification had typos and many
shipped fonts use the lowercase spelling; the `latn` fallback exists because old
fonts put features there even when supporting other scripts. A `false` return
with a usable `*script_index` is the normal, expected case for most fonts — do
not treat it as an error.

**Ownership** — nothing is allocated, nothing is transferred. `script_tags` is
read, not retained.

**Notes**

- Deprecated 2.0.0. The header gives no `Since:`; the function predates the
  documented history.
- Thread-safe for concurrent readers of the same face, with the usual lazy
  table-loading caveat shared by all `hb_ot_layout_*` functions.
- Compiled out under `HB_DISABLE_DEPRECATED` (Cargo features `lean`, `tiny`).

#### `hb_ot_layout_script_find_language`

```c
HB_DEPRECATED_FOR (hb_ot_layout_script_select_language)
HB_EXTERN hb_bool_t
hb_ot_layout_script_find_language (hb_face_t    *face,
                                   hb_tag_t      table_tag,
                                   unsigned int  script_index,
                                   hb_tag_t      language_tag,
                                   unsigned int *language_index);
```

```rust
pub fn hb_ot_layout_script_find_language(
    face: *mut hb_face_t,
    table_tag: hb_tag_t,
    script_index: c_uint,
    language_tag: hb_tag_t,
    language_index: *mut c_uint,
) -> hb_bool_t;
```

Fetches the index of a single language system tag underneath a given script in
the face's `GSUB` or `GPOS` table. Implemented by calling
`hb_ot_layout_script_select_language()` with a one-element array.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Do not pass null. |
| `table_tag` | `HB_OT_TAG_GSUB` or `HB_OT_TAG_GPOS`. |
| `script_index` | Index of the script to search under, as returned by `hb_ot_layout_table_select_script()`, `hb_ot_layout_table_find_script()`, or the enumeration in `hb_ot_layout_table_get_script_tags()`. An out-of-range index selects HarfBuzz's null script object rather than crashing, so the call simply finds nothing. |
| `language_tag` | The OpenType language system tag to look for, e.g. `HB_TAG('T','R','K',' ')`. Taken by value. |
| `language_index` | Out, **may be null**. Receives the index of the language system within the script. |

**Returns** — `true` if `language_tag` itself is present; `false` otherwise. As
with script selection, `false` still leaves a usable index behind when a
fallback applies:

| Situation | Return | `*language_index` |
| --- | --- | --- |
| The requested tag is present | `true` | its index |
| Not found, but `dflt` is present | `false` | index of `dflt` |
| Neither is present | `false` | `HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX` (`0xFFFF`) |

Note that `HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX` is also the value you pass to
downstream functions to mean "the script's default language system", so a
`false` return is still perfectly usable input.

**Ownership** — nothing allocated, nothing transferred.

**Notes**

- Since 0.6.0, deprecated 2.0.0.
- **Not listed in upstream's gtk-doc section file.** It is a genuine exported
  symbol declared in this header, but `docs/harfbuzz-sections.txt` omits it, so
  it does not appear in HarfBuzz's published reference manual. It is documented
  here because it exists in the ABI.
- Compiled out under `HB_DISABLE_DEPRECATED`.

### Script and language tag conversion

#### `hb_ot_tags_from_script`

```c
HB_DEPRECATED_FOR (hb_ot_tags_from_script_and_language)
HB_EXTERN void
hb_ot_tags_from_script (hb_script_t  script,
                        hb_tag_t    *script_tag_1,
                        hb_tag_t    *script_tag_2);
```

```rust
pub fn hb_ot_tags_from_script(
    script: hb_script_t,
    script_tag_1: *mut hb_tag_t,
    script_tag_2: *mut hb_tag_t,
);
```

Converts an `hb_script_t` to the one or two OpenType script tags a font might
use for it. The two-output shape encodes an assumption that stopped being true.
Ten Indic scripts carry an old tag *and* a "version 2" tag (`deva`/`dev2`,
`taml`/`tml2`, `bng2`, `gjr2`, `gur2`, `knd2`, `mlm2`, `ory2`, `tel2`, `mym2`),
and HarfBuzz now also synthesises a "version 3" tag for nine of them (`dev3`,
`tml3`, … — Myanmar is the exception, since no `mym3` exists). That makes three
tags for those scripts, which is why `HB_OT_MAX_TAGS_PER_SCRIPT` is `3` and why
this function cannot express the answer.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `script` | The Unicode script to convert. `HB_SCRIPT_INVALID` maps to `DFLT`. Any script HarfBuzz has no special rule for is converted by lower-casing the first byte of its ISO 15924 code, so `HB_SCRIPT_UNKNOWN` (`Zzzz`) becomes `zzzz` rather than `DFLT`. `HB_SCRIPT_MATH` (`Zmth`) is special-cased to `math`. |
| `script_tag_1` | Out. **Must not be null** — written unconditionally. Receives the most-preferred tag. |
| `script_tag_2` | Out. **Must not be null** — written unconditionally. Receives the second tag, or `DFLT` if there is only one. |

**Returns** — nothing. There is no failure signal; a script with no mapping
simply yields `DFLT` in both slots.

**Ownership** — nothing allocated or transferred.

**Notes**

- Since 0.6.0, deprecated 2.0.0.
- Internally it calls `hb_ot_tags_from_script_and_language(script, HB_LANGUAGE_INVALID, …)`
  with a two-element buffer, then pads any unwritten slot with
  `HB_OT_TAG_DEFAULT_SCRIPT` (`DFLT`). **A third tag, if the script has one, is
  silently dropped** — and for the nine scripts that have three, the dropped one
  is the *old* tag, which is exactly the one an old font is most likely to use.
- Neither output is checked for null. Passing null for either is a crash, not a
  no-op — unlike most HarfBuzz out-parameters.
- Order is by preference, newest first: for Tamil the full list is `tml3`,
  `tml2`, `taml`, so `script_tag_1` is `tml3` and `script_tag_2` is `tml2`.
- For a script with no special mapping the list is just the old tag — e.g.
  `HB_SCRIPT_LATIN` gives `script_tag_1 == 'latn'` and `script_tag_2 == 'DFLT'`.

#### `hb_ot_tag_from_language`

```c
HB_DEPRECATED_FOR (hb_ot_tags_from_script_and_language)
HB_EXTERN hb_tag_t
hb_ot_tag_from_language (hb_language_t language);
```

```rust
pub fn hb_ot_tag_from_language(language: hb_language_t) -> hb_tag_t;
```

Converts an `hb_language_t` (a BCP 47 tag) to a single OpenType language system
tag. Like its script counterpart, the signature bakes in a one-to-one assumption
that no longer holds: `HB_OT_MAX_TAGS_PER_LANGUAGE` is `3`, and this function can
only ever return the first.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `language` | The language to convert. `HB_LANGUAGE_INVALID` (null) is accepted and yields `dflt`. Languages are interned by HarfBuzz, so the pointer is not retained in any meaningful sense. |

**Returns** — the most-preferred OpenType language tag, or
`HB_OT_TAG_DEFAULT_LANGUAGE` = `HB_TAG('d','f','l','t')` = `0x64666C74` when
there is no mapping. **There is no distinct "not found" value:** a language that
genuinely maps to `dflt` and a language HarfBuzz does not know are
indistinguishable.

**Ownership** — nothing allocated or transferred; the returned tag is a value.

**Notes**

- Since 0.6.0, deprecated 2.0.0.
- Internally: `hb_ot_tags_from_script_and_language(HB_SCRIPT_UNKNOWN, language,
  NULL, NULL, &count, tags)` with `count = 1`, then the first tag or `dflt`.
- The private-use subtag conventions the modern API honours still apply here,
  since it is the same code path: a BCP 47 tag such as `en-x-hbot-XYZ` forces the
  OpenType language tag `XYZ `. (The matching `-hbsc` script override is
  unreachable through this function, which discards script output.)
- Compiled out under `HB_DISABLE_DEPRECATED`.

### Variation axes

#### `hb_ot_var_get_axes`

```c
HB_DEPRECATED_FOR (hb_ot_var_get_axis_infos)
HB_EXTERN unsigned int
hb_ot_var_get_axes (hb_face_t        *face,
                    unsigned int      start_offset,
                    unsigned int     *axes_count /* IN/OUT */,
                    hb_ot_var_axis_t *axes_array /* OUT */);
```

```rust
pub fn hb_ot_var_get_axes(
    face: *mut hb_face_t,
    start_offset: c_uint,
    axes_count: *mut c_uint,
    axes_array: *mut hb_ot_var_axis_t,
) -> c_uint;
```

Fetches a list of all variation axes in the face, beginning at `start_offset`.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Do not pass null. A non-variable face reports zero axes. |
| `start_offset` | Zero-based index of the first axis to report, in `fvar` order. Values past the end are not an error; they produce zero written axes. |
| `axes_count` | In/out, **optional**. On input, the capacity of `axes_array`. On output, the number of axes actually written. |
| `axes_array` | Caller-allocated output array of at least `*axes_count` entries, **optional**. |

**Returns** — the **total** number of axes in the face's `fvar` table,
independent of `start_offset` and of the capacity. Zero means the face is not a
variable font (or its `fvar` failed sanitization — the two are
indistinguishable). There is no error return.

**Clamping**, which the header does not spell out:

- If `start_offset > total`, the written count is `0`.
- Otherwise the written count is `min(total - start_offset, *axes_count)`.
- `*axes_count` is updated to that written count.

**The `axes_array == NULL` special case.** The implementation only touches the
outputs when **both** `axes_count` and `axes_array` are non-null:

```cpp
if (axes_count && axes_array)
{
  /* ... copy axes, and set *axes_count to the number copied ... */
}
return axisCount;
```

So `hb_ot_var_get_axes(face, 0, NULL, NULL)` is the idiomatic "how many?" query,
while `hb_ot_var_get_axes(face, 0, &n, NULL)` returns the total but **leaves `n`
untouched** — it is neither zeroed nor set to the available count. This is the
opposite of the usual "pass NULL to size the buffer" C idiom, and it is shared
with `hb_ot_var_get_axis_infos()` and most other HarfBuzz enumerators.

**Ownership** — nothing allocated, nothing transferred. HarfBuzz writes into
memory the caller owns.

**Notes**

- Since 1.4.2, deprecated 2.2.0.
- Behaves identically to `hb_ot_var_get_axis_infos()` except that it fills the
  smaller struct: no `axis_index`, no `flags`, no `reserved`.
- Thread-safe for concurrent readers of the same face.
- Compiled out under `HB_DISABLE_DEPRECATED`, and also under `HB_NO_VAR`
  (which removes the whole variation subsystem).

#### `hb_ot_var_find_axis`

```c
HB_DEPRECATED_FOR (hb_ot_var_find_axis_info)
HB_EXTERN hb_bool_t
hb_ot_var_find_axis (hb_face_t        *face,
                     hb_tag_t          axis_tag,
                     unsigned int     *axis_index,
                     hb_ot_var_axis_t *axis_info);
```

```rust
pub fn hb_ot_var_find_axis(
    face: *mut hb_face_t,
    axis_tag: hb_tag_t,
    axis_index: *mut c_uint,
    axis_info: *mut hb_ot_var_axis_t,
) -> hb_bool_t;
```

Fetches the variation-axis information for a given axis tag.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Do not pass null. |
| `axis_tag` | The axis tag to look for, e.g. `HB_OT_TAG_VAR_AXIS_WEIGHT` (`wght`). Matching is exact; the search is a linear scan of `fvar`, since axes are not stored sorted. |
| `axis_index` | Out, **may be null**. Set to `HB_OT_VAR_NO_AXIS_INDEX` (`0xFFFFFFFF`) before the search, then to the axis's index if found. When null, the implementation substitutes an internal scratch variable, so passing null is safe. |
| `axis_info` | Out. Written **only when the axis is found** — and then written **without a null check**. See below. |

**Returns** — `true` if an axis with that tag exists in the face, `false`
otherwise. On `false`, `axis_info` is untouched and `*axis_index` (if non-null)
holds `HB_OT_VAR_NO_AXIS_INDEX`.

**Ownership** — nothing allocated, nothing transferred.

**Notes**

- Since 1.4.2, deprecated 2.2.0.
- **`axis_info` is not optional in practice.** The header marks it `(out)` with
  no nullability annotation, and the implementation dereferences it as soon as a
  match is found. Passing `NULL` is safe only for a tag the face does not have —
  which you cannot know in advance. Always pass a real struct.
- The replacement, `hb_ot_var_find_axis_info(face, axis_tag, axis_info)`, drops
  the index parameter entirely because `hb_ot_var_axis_info_t::axis_index`
  carries it.
- Compiled out under `HB_DISABLE_DEPRECATED` and under `HB_NO_VAR`.

## Usage

Every example below shows the deprecated call and the modern equivalent, because
there is no reason to write new code against this header.

### Choosing a script for a layout table

C, old:

```c
#include <hb-ot.h>

/* Zero-terminated, most preferred first. */
const hb_tag_t candidates[] = {
  HB_TAG ('d','e','v','2'),
  HB_TAG ('d','e','v','a'),
  HB_TAG_NONE                     /* terminator — do not forget it */
};

unsigned int script_index;
hb_tag_t     chosen;

hb_bool_t exact = hb_ot_layout_table_choose_script (face, HB_OT_TAG_GSUB,
                                                    candidates,
                                                    &script_index, &chosen);
if (!exact && script_index == HB_OT_LAYOUT_NO_SCRIPT_INDEX)
  /* Nothing at all, not even DFLT/dflt/latn. */;
```

C, new — pass a count instead of a terminator:

```c
const hb_tag_t candidates[] = {
  HB_TAG ('d','e','v','2'),
  HB_TAG ('d','e','v','a'),
};

hb_bool_t exact = hb_ot_layout_table_select_script (face, HB_OT_TAG_GSUB,
                                                    2, candidates,
                                                    &script_index, &chosen);
```

Rust, new:

```rust
use core::ffi::c_uint;

use harfbuzz_sys::{
    HB_OT_LAYOUT_NO_SCRIPT_INDEX, HB_OT_TAG_GSUB, HB_TAG, hb_tag_t,
    hb_ot_layout_table_select_script,
};

let candidates: [hb_tag_t; 2] = [
    HB_TAG(b'd', b'e', b'v', b'2'),
    HB_TAG(b'd', b'e', b'v', b'a'),
];

let mut script_index: c_uint = 0;
let mut chosen: hb_tag_t = 0;

unsafe {
    let exact = hb_ot_layout_table_select_script(
        face,
        HB_OT_TAG_GSUB,
        candidates.len() as c_uint,
        candidates.as_ptr(),
        &mut script_index,
        &mut chosen,
    );

    let found_anything = exact != 0 || script_index != HB_OT_LAYOUT_NO_SCRIPT_INDEX;
    let _ = found_anything;
}
```

If you must call the deprecated form from Rust, remember the terminator and
silence the lint at the call site:

```rust
#[allow(deprecated)]
unsafe {
    let candidates: [hb_tag_t; 3] = [
        HB_TAG(b'd', b'e', b'v', b'2'),
        HB_TAG(b'd', b'e', b'v', b'a'),
        harfbuzz_sys::HB_TAG_NONE,      // required terminator
    ];
    harfbuzz_sys::hb_ot_layout_table_choose_script(
        face,
        HB_OT_TAG_GSUB,
        candidates.as_ptr(),
        &mut script_index,
        &mut chosen,
    );
}
```

### Finding a language system under a script

C, old and new side by side:

```c
unsigned int language_index;

/* Old: one tag. */
hb_ot_layout_script_find_language (face, HB_OT_TAG_GSUB, script_index,
                                   HB_TAG ('T','R','K',' '), &language_index);

/* New: a counted array, tried in order. */
const hb_tag_t langs[] = { HB_TAG ('T','R','K',' '), HB_TAG ('A','Z','E',' ') };
hb_ot_layout_script_select_language (face, HB_OT_TAG_GSUB, script_index,
                                     2, langs, &language_index);

/* Newer still (7.0.0): also reports which tag matched. */
hb_tag_t chosen_language;
hb_ot_layout_script_select_language2 (face, HB_OT_TAG_GSUB, script_index,
                                      2, langs, &language_index,
                                      &chosen_language);
```

A `false` return with `language_index == HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX` is
normal and usable — it is the script's default language system.

### Mapping a script to OpenType tags

C, old:

```c
hb_tag_t t1, t2;                    /* both must be real addresses */
hb_ot_tags_from_script (HB_SCRIPT_TAMIL, &t1, &t2);
/* t1 == 'tml3', t2 == 'tml2' — and 'taml' has been silently lost. */
```

C, new — and this is the version that can return all three tags, and that also
handles the language half:

```c
hb_tag_t     script_tags[HB_OT_MAX_TAGS_PER_SCRIPT];
unsigned int script_count = HB_OT_MAX_TAGS_PER_SCRIPT;
hb_tag_t     lang_tags[HB_OT_MAX_TAGS_PER_LANGUAGE];
unsigned int lang_count = HB_OT_MAX_TAGS_PER_LANGUAGE;

hb_ot_tags_from_script_and_language (HB_SCRIPT_TAMIL,
                                     hb_language_from_string ("ta", -1),
                                     &script_count, script_tags,
                                     &lang_count,   lang_tags);
/* script_count == 3: 'tml3', 'tml2', 'taml' — nothing lost.
   lang_count language tags, likewise in preference order. */
```

Rust, new:

```rust
use core::ffi::c_uint;

use harfbuzz_sys::{
    HB_OT_MAX_TAGS_PER_LANGUAGE, HB_OT_MAX_TAGS_PER_SCRIPT, HB_SCRIPT_TAMIL, hb_tag_t,
    hb_language_from_string, hb_ot_tags_from_script_and_language,
};

let mut script_tags = [0 as hb_tag_t; HB_OT_MAX_TAGS_PER_SCRIPT as usize];
let mut script_count: c_uint = HB_OT_MAX_TAGS_PER_SCRIPT;
let mut lang_tags = [0 as hb_tag_t; HB_OT_MAX_TAGS_PER_LANGUAGE as usize];
let mut lang_count: c_uint = HB_OT_MAX_TAGS_PER_LANGUAGE;

unsafe {
    let lang = hb_language_from_string(c"ta".as_ptr(), -1);
    hb_ot_tags_from_script_and_language(
        HB_SCRIPT_TAMIL,
        lang,
        &mut script_count,
        script_tags.as_mut_ptr(),
        &mut lang_count,
        lang_tags.as_mut_ptr(),
    );
}
```

The old `hb_ot_tag_from_language()` maps onto the same call with the script half
disabled:

```c
/* Old. */
hb_tag_t lang_tag = hb_ot_tag_from_language (language);

/* New, exactly equivalent. */
hb_tag_t     tags[1];
unsigned int count = 1;
hb_ot_tags_from_script_and_language (HB_SCRIPT_UNKNOWN, language,
                                     NULL, NULL, &count, tags);
hb_tag_t lang_tag = count ? tags[0] : HB_OT_TAG_DEFAULT_LANGUAGE;
```

### Enumerating variation axes

C, old:

```c
unsigned int total = hb_ot_var_get_axes (face, 0, NULL, NULL);

hb_ot_var_axis_t *axes  = malloc (total * sizeof (hb_ot_var_axis_t));
unsigned int      count = total;
hb_ot_var_get_axes (face, 0, &count, axes);

for (unsigned int i = 0; i < count; i++)
  printf ("%c%c%c%c  %g .. %g (default %g)\n",
          HB_UNTAG (axes[i].tag),
          axes[i].min_value, axes[i].max_value, axes[i].default_value);

free (axes);
```

C, new — same shape, richer struct, and now you can skip hidden axes:

```c
unsigned int total = hb_ot_var_get_axis_count (face);   /* dedicated counter */

hb_ot_var_axis_info_t *axes = malloc (total * sizeof (hb_ot_var_axis_info_t));
unsigned int count = total;
hb_ot_var_get_axis_infos (face, 0, &count, axes);

for (unsigned int i = 0; i < count; i++)
{
  if (axes[i].flags & HB_OT_VAR_AXIS_FLAG_HIDDEN)
    continue;                                  /* not for the UI */
  /* axes[i].axis_index is the index into design-coordinate arrays. */
}

free (axes);
```

Rust, new:

```rust
use core::ffi::c_uint;

use harfbuzz_sys::{
    HB_OT_VAR_AXIS_FLAG_HIDDEN, hb_ot_var_axis_info_t, hb_ot_var_get_axis_count,
    hb_ot_var_get_axis_infos,
};

unsafe {
    let total = hb_ot_var_get_axis_count(face);

    let mut axes: Vec<hb_ot_var_axis_info_t> = Vec::with_capacity(total as usize);
    let mut count: c_uint = total;

    hb_ot_var_get_axis_infos(face, 0, &mut count, axes.as_mut_ptr());
    axes.set_len(count as usize);

    for axis in &axes {
        if axis.flags & HB_OT_VAR_AXIS_FLAG_HIDDEN != 0 {
            continue;
        }
        let tag = axis.tag.to_be_bytes();       // e.g. b"wght"
        let _ = (tag, axis.min_value, axis.default_value, axis.max_value);
    }
}
```

Note that the modern path has a dedicated counter, `hb_ot_var_get_axis_count()`,
so you never need the "call with nulls to size the buffer" trick.

### Looking up one axis by tag

```c
/* Old: two outputs, and axis_info must not be NULL. */
unsigned int     index;
hb_ot_var_axis_t axis;
if (hb_ot_var_find_axis (face, HB_OT_TAG_VAR_AXIS_WEIGHT, &index, &axis))
  clamp_weight (axis.min_value, axis.max_value);

/* New: one output; the index lives inside it. */
hb_ot_var_axis_info_t info;
if (hb_ot_var_find_axis_info (face, HB_OT_TAG_VAR_AXIS_WEIGHT, &info))
  clamp_weight (info.min_value, info.max_value);   /* info.axis_index is there too */
```

Rust:

```rust
use harfbuzz_sys::{
    HB_OT_TAG_VAR_AXIS_WEIGHT, hb_ot_var_axis_info_t, hb_ot_var_find_axis_info,
};

unsafe {
    let mut info: hb_ot_var_axis_info_t = core::mem::zeroed();
    if hb_ot_var_find_axis_info(face, HB_OT_TAG_VAR_AXIS_WEIGHT, &mut info) != 0 {
        let _ = (info.axis_index, info.min_value, info.default_value, info.max_value);
    }
}
```

### Converting an `hb_ot_var_axis_t` you already have

The shared fields are identical, so a migration shim is five assignments:

```c
static void
upgrade (const hb_ot_var_axis_t *old, unsigned int index, hb_ot_var_axis_info_t *out)
{
  out->axis_index    = index;                  /* the old struct did not carry it */
  out->tag           = old->tag;
  out->name_id       = old->name_id;
  out->flags         = 0;                      /* unknowable from the old struct */
  out->min_value     = old->min_value;
  out->default_value = old->default_value;
  out->max_value     = old->max_value;
  out->reserved      = 0;
}
```

`flags` cannot be recovered — that is precisely why the struct was replaced. If
you need to know whether an axis is hidden, you must re-query with
`hb_ot_var_get_axis_infos()` or `hb_ot_var_find_axis_info()`.

### Math shaping, correctly

```c
/* Wrong, and what the old documentation used to say: */
hb_buffer_set_script (buffer, HB_OT_MATH_SCRIPT);        /* a 'math' TAG, not a script */

/* Right, since 3.4.0: */
hb_buffer_set_script (buffer, HB_SCRIPT_MATH);           /* 'Zmth' */

/* And when you genuinely need the OpenType layout TAG, in a table lookup: */
unsigned int   script_index;
const hb_tag_t math[] = { HB_OT_TAG_MATH_SCRIPT };
hb_ot_layout_table_select_script (face, HB_OT_TAG_GSUB,
                                  1, math, &script_index, NULL);
```

The two constants are not interchangeable and never were: `HB_SCRIPT_MATH` is
`Zmth` (a script code, for buffers), `HB_OT_TAG_MATH_SCRIPT` is `math` (an
OpenType layout script tag, for `GSUB`/`GPOS` lookups).
`hb_ot_tags_from_script()` and its replacement do map one
to the other — `hb_ot_tags_from_script(HB_SCRIPT_MATH, &t1, &t2)` yields
`t1 == 'math'` — which is the only supported bridge between them.

## Pitfalls

- **`script_tags` must be zero-terminated, and must not be null.**
  `hb_ot_layout_table_choose_script()` finds the array length by scanning for a
  zero tag before it does anything else. A missing terminator walks off the end
  of your array; a null pointer crashes immediately. The replacement,
  `hb_ot_layout_table_select_script()`, takes an explicit count and has neither
  problem.
- **`false` does not mean "nothing found".** Both layout lookups here return
  `false` whenever a *fallback* was used — `DFLT`, `dflt` or `latn` for scripts,
  `dflt` for languages — and those fallbacks are the common case in real fonts.
  Check the output index against `HB_OT_LAYOUT_NO_SCRIPT_INDEX` /
  `HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX` rather than treating `false` as failure.
- **`hb_ot_tags_from_script()` silently truncates.** Nine Indic scripts map to
  three OpenType tags (`HB_OT_MAX_TAGS_PER_SCRIPT == 3`), but this function has
  only two slots. The tag it drops is the *legacy* one — `taml`, `deva`, … —
  which is precisely the tag an older font is most likely to use, so a naive
  port of this call can fail to find layout data that is right there.
- **`hb_ot_tags_from_script()` dereferences both outputs unconditionally.**
  Unlike almost every other HarfBuzz out-parameter, neither may be null.
- **`hb_ot_tag_from_language()` cannot report failure.** `dflt` is both "no
  mapping" and a legitimate result, and a language can also map to more than one
  tag, of which you only ever see the first.
- **`hb_ot_var_find_axis()` writes `axis_info` without a null check.** The
  parameter looks optional and is not. Passing `NULL` happens to survive when the
  face lacks the axis and crashes when it has it — the worst possible failure
  mode, because it depends on the font.
- **`hb_ot_var_get_axes()` writes nothing unless *both* `axes_count` and
  `axes_array` are non-null.** Passing a count pointer alone returns the total
  but leaves your variable untouched; passing an array alone is a silent no-op.
  Use `(NULL, NULL)` for a pure count, or better, use
  `hb_ot_var_get_axis_count()`.
- **Re-set `*axes_count` on every iteration** of a paged loop. It is in/out, and
  the out value is the number written, so leaving it shrinks the window each time
  round.
- **The return value of `hb_ot_var_get_axes()` is the total, not the number
  written.** It ignores `start_offset`. Loop on the written count.
- **`hb_ot_var_axis_t` cannot express hidden axes.** Anything built on it will
  show designer-hidden axes in a user interface. That is the whole reason for
  `hb_ot_var_axis_info_t`.
- **`HB_OT_MATH_SCRIPT` is a tag, not a script.** Passing it to
  `hb_buffer_set_script()` does not enable math shaping, no matter what old
  documentation says. Use `HB_SCRIPT_MATH` (`Zmth`) for buffers and
  `HB_OT_TAG_MATH_SCRIPT` (`math`) for layout-table lookups.
- **`HB_OT_VAR_NO_AXIS_INDEX` has no modern counterpart.** The header says "Do
  not use". It only ever appears as the pre-set/not-found value of
  `hb_ot_var_find_axis()`'s `axis_index`.
- **`HB_DISABLE_DEPRECATED` builds do not link.** Under this crate's `lean` or
  `tiny` features (upstream `HB_LEAN` / `HB_TINY`) all six functions are compiled
  out rather than stubbed, so a call becomes an undefined symbol at link time.
  The two variation functions additionally disappear under `HB_NO_VAR`. The Rust
  constants and the struct survive in every configuration, because they need no
  symbol.
- **In Rust, every item here is `#[deprecated]`,** including the constants and
  the struct that the C header leaves unmarked. Using one warns; wrap the call
  site in `#[allow(deprecated)]` if you truly must.
- **Face-level, not font-level.** Nothing in this header reads an `hb_font_t`,
  so variation settings and point size have no effect on any of it.
- **Null `face` is unspecified.** The header documents nothing and every
  implementation dereferences `face` immediately. Pass a real face, or
  `hb_face_get_empty()`.

## Relationship to upstream's reference manual

HarfBuzz's `docs/harfbuzz-sections.txt` has **no `hb-ot-deprecated` section**.
Everything declared here is folded into the single `hb-deprecated` section (38
entries), together with the contents of `hb-deprecated.h`, `hb-ft.h`,
`hb-directwrite.h`, `hb-aat-layout.h`, `hb-buffer.h`, `hb-unicode.h` and
`hb-font.h`. Nine of those 38 entries belong to this header and are documented
above. Two further notes:

- **`hb_ot_layout_table_find_script`** appears in the `hb-deprecated` section but
  is declared in `hb-ot-layout.h`, not here, and is *not* formally marked
  deprecated in C. It lives in this crate's `ot_layout` module; see
  `docs/ot_layout.md`.
- **`hb_ot_layout_script_find_language`** is declared in this header but is
  missing from the section list, so it does not appear in upstream's published
  manual. It is documented above regardless.
