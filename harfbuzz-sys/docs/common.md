# Common types

Header: `hb-common.h` — Rust module: `harfbuzz_sys::common` (glob re-exported at
the crate root).

## Overview

`hb-common.h` is the header every other HarfBuzz header includes. It defines no
objects and owns no lifecycle of its own; what it provides is the vocabulary
that the rest of the API is written in — the scalar typedefs (`hb_codepoint_t`,
`hb_position_t`, `hb_mask_t`, `hb_bool_t`), the four-byte **tag**, the
**direction** and **script** enumerations, the interned **language** handle, the
`hb_feature_t` and `hb_variation_t` structs you hand to the shaper, the
user-data key and destroy-callback conventions shared by every reference-counted
object, and HarfBuzz's own `malloc` family.

Two design habits run through the whole file and are worth internalising early.

The first is **tags**. A `hb_tag_t` is a `uint32_t` holding four bytes, most
significant byte first, so `'k' 'e' 'r' 'n'` becomes `0x6B65726E`. HarfBuzz
identifies almost everything this way: table names, OpenType feature and script
and language-system tags, design-variation axes, baselines, and — via
`hb_script_t`, which is literally an ISO 15924 tag — scripts. Because tags are
plain integers they compare and hash for free, and `HB_TAG` builds one at
compile time. `hb_tag_from_string` pads short strings with spaces and truncates
long ones, so `hb_tag_from_string("aa", -1)` is `HB_TAG('a','a',' ',' ')`.

The second is **string conversions that never fail loudly**. `hb_tag_from_string`,
`hb_direction_from_string`, `hb_script_from_string`, and `hb_language_from_string`
all return a sentinel (`HB_TAG_NONE`, `HB_DIRECTION_INVALID`,
`HB_SCRIPT_UNKNOWN` or `HB_SCRIPT_INVALID`, `HB_LANGUAGE_INVALID`) rather than
signalling an error, and their matching is deliberately loose —
`hb_direction_from_string` looks at the first letter only, so `"l"`, `"LTR"`,
and `"left-to-right"` all give `HB_DIRECTION_LTR`. Only `hb_feature_from_string`
and `hb_variation_from_string` return a boolean.

**Languages** are the one type here with an unusual representation.
`hb_language_t` is an opaque pointer, but the objects it points at are *interned*
in a process-wide table that is never garbage-collected: an `hb_language_t`
lives for the lifetime of the process, is never freed by the caller, and can be
compared for equality with `==`. That is why `hb_segment_properties_t` can hash
one by casting the pointer to an integer.

**User-data keys and destroy callbacks** are the shared extension mechanism.
Every reference-counted HarfBuzz object has `hb_*_set_user_data` /
`hb_*_get_user_data` taking an `hb_user_data_key_t *`; HarfBuzz uses the
*address* of the key, never its contents, so the canonical key is a `static`
variable whose value nobody ever reads. `hb_destroy_func_t` is the matching
"I am done with this pointer" callback, used both by the user-data tables and by
every constructor that takes ownership of client memory.

Two things declared in this header are documented elsewhere because gtk-doc
files them under other sections: `hb_glyph_extents_t` and `hb_font_t` belong to
`font.md`, and `hb_color_t`, `HB_COLOR`, and the `hb_color_get_*` accessors
belong to `ot_color.md`. Both groups are described below anyway, since they are
part of this header's contents.

## Types

### `hb_bool_t`

```c
typedef int hb_bool_t;
```

```rust
pub type hb_bool_t = core::ffi::c_int;
```

Data type for booleans. Zero is false; any non-zero value is true. HarfBuzz
returns `1` for true in practice, but code should test `!= 0` rather than
`== 1`.

### `hb_codepoint_t`

```c
typedef uint32_t hb_codepoint_t;
```

```rust
pub type hb_codepoint_t = u32;
```

Data type for holding Unicode code points. **Also used to hold glyph IDs** —
the same type appears in `hb_glyph_info_t.codepoint` both before shaping (a
character) and after (a glyph index), and in every `hb_font_get_glyph_*`
function. Nothing in the type distinguishes the two meanings; only context does.

### `hb_position_t`

```c
typedef int32_t hb_position_t;
```

```rust
pub type hb_position_t = i32;
```

Data type for holding a single coordinate value. Contour points and other
multi-dimensional data are stored as tuples of `hb_position_t`. Signed, because
offsets and bearings go both ways. The unit depends on the font's scale (see
`hb_font_set_scale` in `font.md`): with the default scale it is font design
units; with a scale set it is 26.6-style fixed point of your choosing.

### `hb_mask_t`

```c
typedef uint32_t hb_mask_t;
```

```rust
pub type hb_mask_t = u32;
```

Data type for bitmasks. In the public API it appears only as the private `mask`
field of `hb_glyph_info_t`, whose low bits carry `hb_glyph_flags_t`; read those
with `hb_glyph_info_get_glyph_flags`.

### `hb_tag_t`

```c
typedef uint32_t hb_tag_t;
```

```rust
pub type hb_tag_t = u32;
```

Data type for tag identifiers. Tags are four-byte integers, each byte
representing a character, stored most significant byte first. They identify
tables, design-variation axes, scripts, languages, font features, and baselines
with human-readable names.

Build them with `HB_TAG` (compile time) or `hb_tag_from_string` (run time), and
take them apart with `HB_UNTAG` or `hb_tag_to_string`. The sentinel and bounds
are `HB_TAG_NONE`, `HB_TAG_MAX`, and `HB_TAG_MAX_SIGNED`.

### `hb_direction_t`

```c
typedef enum {
  HB_DIRECTION_INVALID = 0,
  HB_DIRECTION_LTR = 4,
  HB_DIRECTION_RTL,
  HB_DIRECTION_TTB,
  HB_DIRECTION_BTT
} hb_direction_t;
```

```rust
pub type hb_direction_t = core::ffi::c_int;
```

The direction of a text segment or buffer. The numeric values are not
arbitrary — the four valid ones occupy `4..=7` so that the `HB_DIRECTION_IS_*`
macros can be single bit tests, and `HB_DIRECTION_REVERSE` can be an XOR with 1.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_DIRECTION_INVALID` | 0 | Initial, unset direction. Shaping cannot proceed with this. |
| `HB_DIRECTION_LTR` | 4 | Text is set horizontally from left to right. |
| `HB_DIRECTION_RTL` | 5 | Text is set horizontally from right to left. |
| `HB_DIRECTION_TTB` | 6 | Text is set vertically from top to bottom. |
| `HB_DIRECTION_BTT` | 7 | Text is set vertically from bottom to top. |

Values 1, 2, and 3 are unused and are not valid directions;
`HB_DIRECTION_IS_VALID` rejects them. A segment can be tested for horizontal or
vertical orientation, irrespective of specific direction, with
`HB_DIRECTION_IS_HORIZONTAL` and `HB_DIRECTION_IS_VERTICAL`.

### `hb_script_t`

```c
/* from hb-script-list.h, included by hb-common.h */
typedef enum { HB_SCRIPT_COMMON = HB_TAG('Z','y','y','y'), ... } hb_script_t;
```

```rust
pub type hb_script_t = core::ffi::c_int;
```

Data type for scripts. Every value is an `hb_tag_t` holding the four-letter code
defined by [ISO 15924](https://unicode.org/iso15924/) — the Script (`sc`)
property of the Unicode Character Database. The constants themselves are
declared in `hb-script-list.h` (Rust module `harfbuzz_sys::script`) and are
documented there; the four you meet most often are:

| Constant | Tag | Meaning |
| --- | --- | --- |
| `HB_SCRIPT_INVALID` | `HB_TAG_NONE` (0) | No script set. |
| `HB_SCRIPT_COMMON` | `Zyyy` | Characters belonging to no single script: spaces, digits, most punctuation. |
| `HB_SCRIPT_INHERITED` | `Zinh` | Characters taking their script from the preceding character, such as combining marks. |
| `HB_SCRIPT_UNKNOWN` | `Zzzz` | Unassigned, private-use, noncharacter, and surrogate code points. |

The Rust alias is a signed `c_int` because the C enumeration ends with two
private sentinels equal to `HB_TAG_MAX_SIGNED`, which pins the underlying type.
Those sentinels exist so that *any* `hb_tag_t` bit pattern can be stored in an
`hb_script_t` without undefined behaviour: values are not restricted to the
`HB_SCRIPT_*` constants, and text or font data can legitimately produce a tag
that has no constant.

### `hb_language_t`

```c
typedef const struct hb_language_impl_t *hb_language_t;
```

```rust
crate::opaque_handle! { hb_language_impl_t }
pub type hb_language_t = *const hb_language_impl_t;
```

Data type for languages. Each `hb_language_t` corresponds to a
[BCP 47](https://www.rfc-editor.org/info/bcp47) language tag.

**Ownership** — values are interned by HarfBuzz in a process-wide table and live
for the lifetime of the process. You never allocate one, never free one, and can
compare two for equality with `==`. `hb_language_from_string` is the only
constructor; `hb_language_to_string` gives the canonical string back.

The unset value is `HB_LANGUAGE_INVALID`, which is the null pointer, so
`if (!language)` is the idiomatic C test.

### `hb_user_data_key_t`

```c
typedef struct hb_user_data_key_t {
  /*< private >*/
  char unused;
} hb_user_data_key_t;
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct hb_user_data_key_t {
    pub unused: c_char,
}
```

Data structure for holding user-data keys.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `unused` | `char` | `c_char` | **Private padding.** Never read or written by HarfBuzz; it exists only so the struct is not empty. |

HarfBuzz uses the **address** of an `hb_user_data_key_t`, never its contents, as
the key into an object's user-data table. The canonical usage is a file-scope
`static hb_user_data_key_t my_key;` whose value nobody ever touches; the key
object must outlive every object it was used with. In Rust that is a
`static MY_KEY: hb_user_data_key_t = hb_user_data_key_t { unused: 0 };` — but
note that the C API takes `*mut hb_user_data_key_t`, so a `static mut` or a
cast is needed, and two distinct statics must not be merged by the compiler.

### `hb_destroy_func_t`

```c
typedef void (*hb_destroy_func_t) (void *user_data);
```

```rust
pub type hb_destroy_func_t = Option<unsafe extern "C" fn(user_data: *mut c_void)>;
```

A virtual method for destroy user-data callbacks: HarfBuzz calls it with the
pointer you supplied when it no longer needs that pointer.

This is the universal "you may free this now" signal across the whole API. It
appears in every `set_user_data`, in `hb_blob_create`, in
`hb_buffer_set_message_func`, in every `hb_*_funcs_set_*_func`, and so on. Two
rules hold everywhere:

- It may be null (`None` in Rust) when you have nothing to clean up.
- Functions that take a `user_data` / `destroy` pair assume ownership of
  `user_data` **unconditionally**, including on their own failure paths — they
  call `destroy(user_data)` themselves before returning an error. Freeing the
  data yourself after a failed call is a double free.

In Rust the typedef is wrapped in `Option` so that `None` is the null pointer.

### `hb_feature_t`

```c
typedef struct hb_feature_t {
  hb_tag_t      tag;
  uint32_t      value;
  unsigned int  start;
  unsigned int  end;
} hb_feature_t;
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_feature_t {
    pub tag: hb_tag_t,
    pub value: u32,
    pub start: c_uint,
    pub end: c_uint,
}
```

Holds information about a requested feature application. The feature is applied
with `value` to all glyphs in clusters between `start` (inclusive) and `end`
(exclusive). This is the struct you pass as an array to `hb_shape`.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `tag` | `hb_tag_t` | `u32` | The OpenType feature tag, e.g. `HB_TAG('k','e','r','n')`. |
| `value` | `uint32_t` | `u32` | 0 disables the feature; non-zero (usually 1) enables it. For features implemented as lookup type 3 — `salt`, for instance — the value is a **one-based index into the alternates**. |
| `start` | `unsigned int` | `c_uint` | The cluster to start applying this setting, inclusive. `HB_FEATURE_GLOBAL_START` (0) means from the start of the buffer. |
| `end` | `unsigned int` | `c_uint` | The cluster to stop applying this setting, exclusive. `HB_FEATURE_GLOBAL_END` (`UINT_MAX`) means to the end of the buffer. |

Setting `start` to `HB_FEATURE_GLOBAL_START` and `end` to
`HB_FEATURE_GLOBAL_END` specifies that the feature always applies to the entire
buffer. The range indices refer to cluster values, which for text added with
`hb_buffer_add_utf8` are byte offsets — so a range is only meaningful relative
to how the text was added.

The Rust struct derives `PartialEq`, `Eq`, and `Hash`, so features can be used
as map keys directly.

### `hb_variation_t`

```c
typedef struct hb_variation_t {
  hb_tag_t tag;
  float    value;
} hb_variation_t;
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct hb_variation_t {
    pub tag: hb_tag_t,
    pub value: c_float,
}
```

Data type for holding variation data — one axis setting of a variable font.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `tag` | `hb_tag_t` | `u32` | The variation-axis tag, e.g. `HB_TAG('w','g','h','t')`. |
| `value` | `float` | `c_float` | The value of the variation axis, in the axis's own user-space units. |

Registered OpenType variation-axis tags are listed in the
[OpenType Axis Tag Registry](https://docs.microsoft.com/en-us/typography/opentype/spec/dvaraxisreg).
Since HarfBuzz 1.4.2. Only `PartialEq` is derived, because `f32` is not `Eq`.

### `hb_color_t`

```c
typedef uint32_t hb_color_t;
```

```rust
pub type hb_color_t = u32;
```

Data type for holding colour values: eight bits per channel, RGB plus alpha
transparency. Build one with `HB_COLOR` and take it apart with
`hb_color_get_red` and friends. Since HarfBuzz 2.1.0.

The byte order is **BGRA** from most significant byte down: blue in bits 31–24,
green in 23–16, red in 15–8, alpha in 7–0. That is why `HB_COLOR` takes its
arguments in the order `(b, g, r, a)` — a genuine and frequently-hit surprise.

gtk-doc files this type, `HB_COLOR`, and the four accessors under the
`hb-ot-color` section, so they are also covered in `ot_color.md`; they are
declared in `hb-common.h`.

### `hb_glyph_extents_t`

```c
typedef struct hb_glyph_extents_t {
  hb_position_t x_bearing;
  hb_position_t y_bearing;
  hb_position_t width;
  hb_position_t height;
} hb_glyph_extents_t;
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_glyph_extents_t {
    pub x_bearing: hb_position_t,
    pub y_bearing: hb_position_t,
    pub width: hb_position_t,
    pub height: hb_position_t,
}
```

Glyph extent values, measured in font units.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `x_bearing` | `hb_position_t` | `i32` | Distance from the x-origin to the left extremum of the glyph. |
| `y_bearing` | `hb_position_t` | `i32` | Distance from the top extremum of the glyph to the y-origin. |
| `width` | `hb_position_t` | `i32` | Distance from the left extremum to the right extremum. |
| `height` | `hb_position_t` | `i32` | Distance from the top extremum to the bottom extremum. |

Note that `height` is **negative** in coordinate systems that grow up, which is
HarfBuzz's default. gtk-doc files this struct under the `hb-font` section, so it
is also covered in `font.md`; it is declared in `hb-common.h` and filled in by
`hb_font_get_glyph_extents`.

### `hb_var_int_t`

```c
typedef union _hb_var_int_t {
  uint32_t u32;
  int32_t  i32;
  uint16_t u16[2];
  int16_t  i16[2];
  uint8_t  u8[4];
  int8_t   i8[4];
} hb_var_int_t;
```

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub union hb_var_int_t {
    pub u32_: u32,
    pub i32_: i32,
    pub u16_: [u16; 2],
    pub i16_: [i16; 2],
    pub u8_: [u8; 4],
    pub i8_: [i8; 4],
}
```

A union of the integer widths HarfBuzz can store in a single 32-bit slot. It is
listed in the private subsection of the section file, and appears in the public
API only as the private `var1`/`var2` fields of `hb_glyph_info_t` and the
private `var` field of `hb_glyph_position_t` — scratch space the shaper uses.
Clients must not read or write it.

The Rust field names carry a trailing underscore (`u32_`, `i32_`, …) because
`u32` and friends are type names in Rust. Because it is a union, structs
containing it cannot derive `Debug`, which is why `hb_glyph_info_t` and
`hb_glyph_position_t` implement `Debug` by hand.

### `hb_var_num_t`

```c
typedef union _hb_var_num_t {
  float    f;
  uint32_t u32;
  int32_t  i32;
  uint16_t u16[2];
  int16_t  i16[2];
  uint8_t  u8[4];
  int8_t   i8[4];
} hb_var_num_t;
```

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub union hb_var_num_t {
    pub f: c_float,
    pub u32_: u32,
    pub i32_: i32,
    pub u16_: [u16; 2],
    pub i16_: [i16; 2],
    pub u8_: [u8; 4],
    pub i8_: [i8; 4],
}
```

As `hb_var_int_t`, but also able to hold a `float`. Private, like its sibling;
it exists for internal 32-bit slots that may carry a floating-point value.

## Constants

### `HB_CODEPOINT_INVALID`

```c
#define HB_CODEPOINT_INVALID ((hb_codepoint_t) -1)
```

```rust
pub const HB_CODEPOINT_INVALID: hb_codepoint_t = u32::MAX;
```

An unused `hb_codepoint_t` value — `0xFFFFFFFF`, which is neither a valid
Unicode code point nor a plausible glyph ID. Used as "no value" wherever a code
point or glyph ID is optional; `hb_buffer_set_not_found_variation_selector_glyph`
and `hb_buffer_diff` both take it as a sentinel. Since HarfBuzz 8.0.0.

### `HB_TAG_NONE`

```c
#define HB_TAG_NONE HB_TAG(0,0,0,0)
```

```rust
pub const HB_TAG_NONE: hb_tag_t = HB_TAG(0, 0, 0, 0);
```

Unset `hb_tag_t` — zero. It is also the value of `HB_SCRIPT_INVALID` and of
`HB_BUFFER_SERIALIZE_FORMAT_INVALID`, and what `hb_tag_from_string` returns for
an empty or null string.

### `HB_TAG_MAX`

```c
#define HB_TAG_MAX HB_TAG(0xff,0xff,0xff,0xff)
```

```rust
pub const HB_TAG_MAX: hb_tag_t = HB_TAG(0xff, 0xff, 0xff, 0xff);
```

Maximum possible unsigned `hb_tag_t` — `0xFFFFFFFF`. Since HarfBuzz 0.9.26.

### `HB_TAG_MAX_SIGNED`

```c
#define HB_TAG_MAX_SIGNED HB_TAG(0x7f,0xff,0xff,0xff)
```

```rust
pub const HB_TAG_MAX_SIGNED: hb_tag_t = HB_TAG(0x7f, 0xff, 0xff, 0xff);
```

Maximum possible signed `hb_tag_t` — `0x7FFFFFFF`. This is the value of the
private sentinels at the end of the `hb_script_t` enumeration, which is what
forces that enumeration's underlying type to be at least a signed 32-bit
integer. Since HarfBuzz 0.9.33.

### `HB_LANGUAGE_INVALID`

```c
#define HB_LANGUAGE_INVALID ((hb_language_t) 0)
```

```rust
pub const HB_LANGUAGE_INVALID: hb_language_t = core::ptr::null();
```

An unset `hb_language_t`: the null pointer. Returned by
`hb_language_from_string` for an empty or null string, and the initial language
of a fresh buffer. Since HarfBuzz 0.6.0.

### `HB_FEATURE_GLOBAL_START`

```c
#define HB_FEATURE_GLOBAL_START 0
```

```rust
pub const HB_FEATURE_GLOBAL_START: c_uint = 0;
```

Special setting for `hb_feature_t.start` to apply the feature from the start of
the buffer. Since HarfBuzz 2.0.0.

### `HB_FEATURE_GLOBAL_END`

```c
#define HB_FEATURE_GLOBAL_END ((unsigned int) -1)
```

```rust
pub const HB_FEATURE_GLOBAL_END: c_uint = c_uint::MAX;
```

Special setting for `hb_feature_t.end` to apply the feature to the end of the
buffer. Since HarfBuzz 2.0.0.

## Macros

### `HB_TAG`

```c
#define HB_TAG(c1,c2,c3,c4) ((hb_tag_t)((((uint32_t)(c1)&0xFF)<<24)| \
                                        (((uint32_t)(c2)&0xFF)<<16)| \
                                        (((uint32_t)(c3)&0xFF)<<8)|   \
                                         ((uint32_t)(c4)&0xFF)))
```

```rust
#[inline]
pub const fn HB_TAG(c1: u8, c2: u8, c3: u8, c4: u8) -> hb_tag_t;
```

Constructs an `hb_tag_t` from four character literals: `HB_TAG('k','e','r','n')`
in C, `HB_TAG(b'k', b'e', b'r', b'n')` in Rust. Each argument contributes one
byte, most significant first.

In Rust this is a `const fn`, so it is usable in constant position:

```rust
use harfbuzz_sys::{HB_TAG, hb_tag_t};
const KERN: hb_tag_t = HB_TAG(b'k', b'e', b'r', b'n');
```

The C macro masks each argument with `0xFF`, so passing a value above 255 is
silently truncated rather than corrupting the neighbouring byte; the Rust
version takes `u8` and cannot be given one.

### `HB_UNTAG`

```c
#define HB_UNTAG(tag) (uint8_t)(((tag)>>24)&0xFF), (uint8_t)(((tag)>>16)&0xFF), \
                      (uint8_t)(((tag)>>8)&0xFF),  (uint8_t)((tag)&0xFF)
```

```rust
#[inline]
pub const fn HB_UNTAG(tag: hb_tag_t) -> [u8; 4];
```

Extracts the four character literals from an `hb_tag_t`, most significant first.

The C macro expands to a **comma-separated list of four expressions**, not to a
single value — its purpose is to be spliced into an argument list, as in
`printf ("%c%c%c%c", HB_UNTAG (tag))`. The Rust equivalent cannot do that, so it
returns a `[u8; 4]` (implemented as `tag.to_be_bytes()`). This is a deliberate
divergence in shape between the two spellings. Since HarfBuzz 0.6.0.

### `HB_DIRECTION_IS_VALID`

```c
#define HB_DIRECTION_IS_VALID(dir) ((((unsigned int) (dir)) & ~3U) == 4)
```

```rust
#[inline]
pub const fn HB_DIRECTION_IS_VALID(dir: hb_direction_t) -> bool;
```

Tests whether a text direction is valid — that is, one of `HB_DIRECTION_LTR`
(4), `RTL` (5), `TTB` (6), `BTT` (7). Everything else, including
`HB_DIRECTION_INVALID` (0) and the unused values 1–3, is rejected. This is the
**only** predicate in the family that is safe to call on an arbitrary value; the
other five require a valid direction and give meaningless answers otherwise.

### `HB_DIRECTION_IS_HORIZONTAL`

```c
#define HB_DIRECTION_IS_HORIZONTAL(dir) ((((unsigned int) (dir)) & ~1U) == 4)
```

```rust
#[inline]
pub const fn HB_DIRECTION_IS_HORIZONTAL(dir: hb_direction_t) -> bool;
```

Tests whether a text direction is horizontal — `LTR` or `RTL`. **Requires that
the direction be valid.**

### `HB_DIRECTION_IS_VERTICAL`

```c
#define HB_DIRECTION_IS_VERTICAL(dir) ((((unsigned int) (dir)) & ~1U) == 6)
```

```rust
#[inline]
pub const fn HB_DIRECTION_IS_VERTICAL(dir: hb_direction_t) -> bool;
```

Tests whether a text direction is vertical — `TTB` or `BTT`. **Requires that the
direction be valid.**

### `HB_DIRECTION_IS_FORWARD`

```c
#define HB_DIRECTION_IS_FORWARD(dir) ((((unsigned int) (dir)) & ~2U) == 4)
```

```rust
#[inline]
pub const fn HB_DIRECTION_IS_FORWARD(dir: hb_direction_t) -> bool;
```

Tests whether a text direction moves forward — left to right, or top to bottom;
that is, `LTR` or `TTB`. **Requires that the direction be valid.**

### `HB_DIRECTION_IS_BACKWARD`

```c
#define HB_DIRECTION_IS_BACKWARD(dir) ((((unsigned int) (dir)) & ~2U) == 5)
```

```rust
#[inline]
pub const fn HB_DIRECTION_IS_BACKWARD(dir: hb_direction_t) -> bool;
```

Tests whether a text direction moves backward — right to left, or bottom to
top; that is, `RTL` or `BTT`. **Requires that the direction be valid.**

### `HB_DIRECTION_REVERSE`

```c
#define HB_DIRECTION_REVERSE(dir) ((hb_direction_t) (((unsigned int) (dir)) ^ 1))
```

```rust
#[inline]
pub const fn HB_DIRECTION_REVERSE(dir: hb_direction_t) -> hb_direction_t;
```

Reverses a text direction: `LTR` ↔ `RTL`, `TTB` ↔ `BTT`. **Requires that the
direction be valid** — applied to `HB_DIRECTION_INVALID` (0) it yields 1, which
is not a direction at all.

### `HB_COLOR`

```c
#define HB_COLOR(b,g,r,a) ((hb_color_t) HB_TAG ((b),(g),(r),(a)))
```

```rust
#[inline]
pub const fn HB_COLOR(b: u8, g: u8, r: u8, a: u8) -> hb_color_t;
```

Constructs an `hb_color_t` from four channel values. **Note the argument order,
which follows the C macro: blue, green, red, alpha** — not RGBA. Opaque red is
`HB_COLOR(0, 0, 255, 255)`. Since HarfBuzz 2.1.0.

## Functions

### Tags

#### `hb_tag_from_string`

```c
hb_tag_t hb_tag_from_string (const char *str, int len);
```

```rust
pub fn hb_tag_from_string(str_: *const c_char, len: c_int) -> hb_tag_t;
```

Converts a string into an `hb_tag_t`.

**Parameters**

| Parameter | Meaning | Null allowed |
| --- | --- | --- |
| `str_` | The string to convert. | yes — returns `HB_TAG_NONE` |
| `len` | Length of `str_`, or `-1` if it is NUL-terminated. | — |

**Returns** — the tag corresponding to `str_`. Valid tags are four characters:
**shorter input strings are padded with spaces, longer ones are truncated**, so
`"aa"` becomes `HB_TAG('a','a',' ',' ')` and `"kerning"` becomes
`HB_TAG('k','e','r','n')`. A null pointer, a zero `len`, or a leading NUL byte
all return `HB_TAG_NONE`. An embedded NUL also stops the scan, with the
remainder padded with spaces.

**Notes** — since HarfBuzz 0.9.2. There is no error return: every input produces
some tag.

#### `hb_tag_to_string`

```c
void hb_tag_to_string (hb_tag_t tag, char *buf);
```

```rust
pub fn hb_tag_to_string(tag: hb_tag_t, buf: *mut c_char);
```

Converts an `hb_tag_t` to a string and returns it in `buf`.

**Parameters**

| Parameter | Meaning | Null allowed |
| --- | --- | --- |
| `tag` | The tag to convert. | — |
| `buf` | Caller-allocated output. **Must have room for exactly four bytes.** | no |

**Returns** — nothing. Strings are always four characters long and the result is
**not NUL-terminated**, so a `char buf[5]` with `buf[4] = '\0'` is the usual
idiom in C, and in Rust a `[u8; 4]` that you then wrap in a `str`.

**Notes** — since HarfBuzz 0.9.5. Bytes are written unconditionally, including
the trailing spaces of a padded tag; trim them yourself if you want `"aa"` back
rather than `"aa  "`.

### Directions

#### `hb_direction_from_string`

```c
hb_direction_t hb_direction_from_string (const char *str, int len);
```

```rust
pub fn hb_direction_from_string(str_: *const c_char, len: c_int) -> hb_direction_t;
```

Converts a string to an `hb_direction_t`.

**Parameters**

| Parameter | Meaning | Null allowed |
| --- | --- | --- |
| `str_` | The string to convert. | yes — returns `HB_DIRECTION_INVALID` |
| `len` | Length of `str_`, or `-1` if it is NUL-terminated. | — |

**Returns** — the matching direction, or `HB_DIRECTION_INVALID` for an unmatched
string. **Matching is loose, case-insensitive, and applies only to the first
letter**: `"l"`, `"LTR"`, and `"left-to-right"` all give `HB_DIRECTION_LTR`, and
so does `"lizard"`. The recognised first letters are `l`, `r`, `t`, `b`.

**Notes** — since HarfBuzz 0.9.2.

#### `hb_direction_to_string`

```c
const char *hb_direction_to_string (hb_direction_t direction);
```

```rust
pub fn hb_direction_to_string(direction: hb_direction_t) -> *const c_char;
```

Converts an `hb_direction_t` to a string: `"ltr"`, `"rtl"`, `"ttb"`, `"btt"`, or
`"invalid"` for anything else.

**Returns** — a static NUL-terminated string. **Never null**, unlike most of
HarfBuzz's `*_to_string` functions.

**Ownership** — `transfer none`; the string is static and must not be freed.

**Notes** — since HarfBuzz 0.9.2.

### Scripts

#### `hb_script_from_iso15924_tag`

```c
hb_script_t hb_script_from_iso15924_tag (hb_tag_t tag);
```

```rust
pub fn hb_script_from_iso15924_tag(tag: hb_tag_t) -> hb_script_t;
```

Converts an ISO 15924 script tag to the corresponding `hb_script_t`.

**Returns** — the script. The conversion is deliberately lenient:

- Case is normalised to one capital letter followed by three small ones, so
  `"arab"`, `"ARAB"`, and `"Arab"` all work.
- A handful of historic and variant tags are folded onto their modern script:
  `Qaai` → `HB_SCRIPT_INHERITED`, `Qaac` → `HB_SCRIPT_COPTIC`, `Aran` →
  `HB_SCRIPT_ARABIC`, `Cyrs` → `HB_SCRIPT_CYRILLIC`, `Geok` →
  `HB_SCRIPT_GEORGIAN`, `Hans`/`Hant` → `HB_SCRIPT_HAN`, `Jamo` →
  `HB_SCRIPT_HANGUL`, `Latf`/`Latg` → `HB_SCRIPT_LATIN`, and
  `Syre`/`Syrj`/`Syrn` → `HB_SCRIPT_SYRIAC`.
- A tag that merely *looks* like a script code (one uppercase letter followed by
  three lowercase) is passed through unchanged, even if no constant exists for
  it — this is how new Unicode scripts work before HarfBuzz knows about them.
- `HB_TAG_NONE` returns `HB_SCRIPT_INVALID`.
- Anything else returns `HB_SCRIPT_UNKNOWN`.

**Notes** — since HarfBuzz 0.9.2. Note that the folding is lossy: `Hans` and
`Hant` both become `HB_SCRIPT_HAN`, so round-tripping through
`hb_script_to_iso15924_tag` does not give you back the original tag.

#### `hb_script_from_string`

```c
hb_script_t hb_script_from_string (const char *str, int len);
```

```rust
pub fn hb_script_from_string(str_: *const c_char, len: c_int) -> hb_script_t;
```

Converts a string representing an ISO 15924 script tag to the corresponding
script. Shorthand for `hb_tag_from_string` followed by
`hb_script_from_iso15924_tag`, and therefore inherits both the space-padding and
the leniency described above. Pass `len` as `-1` when `str_` is NUL-terminated.

**Notes** — since HarfBuzz 0.9.2. Because of the padding, `"Ar"` becomes the tag
`"Ar  "`, which does not look like a script code and so yields
`HB_SCRIPT_UNKNOWN` rather than an error.

#### `hb_script_to_iso15924_tag`

```c
hb_tag_t hb_script_to_iso15924_tag (hb_script_t script);
```

```rust
pub fn hb_script_to_iso15924_tag(script: hb_script_t) -> hb_tag_t;
```

Converts an `hb_script_t` to the corresponding ISO 15924 script tag.

**Returns** — the tag. The implementation is a plain reinterpretation of the
value, so it never fails and round-trips unchanged any tag that
`hb_script_from_iso15924_tag` passed through. Combine with `hb_tag_to_string` to
get printable characters.

**Notes** — since HarfBuzz 0.9.2.

#### `hb_script_get_horizontal_direction`

```c
hb_direction_t hb_script_get_horizontal_direction (hb_script_t script);
```

```rust
pub fn hb_script_get_horizontal_direction(script: hb_script_t) -> hb_direction_t;
```

Fetches the direction of a script when it is set horizontally.

**Returns**

| Case | Result |
| --- | --- |
| Right-to-left scripts (Arabic, Hebrew, Syriac, Thaana, …) | `HB_DIRECTION_RTL` |
| Left-to-right scripts | `HB_DIRECTION_LTR` |
| Scripts that can be written either way | `HB_DIRECTION_INVALID` |
| Unknown scripts | `HB_DIRECTION_LTR` |

**Notes** — since HarfBuzz 0.9.2. `hb_buffer_guess_segment_properties` uses this
and substitutes `HB_DIRECTION_LTR` when the answer is `HB_DIRECTION_INVALID`.
The `HB_DIRECTION_INVALID` case is a real answer, not an error: it means the
script genuinely has no single horizontal direction.

### Languages

#### `hb_language_from_string`

```c
hb_language_t hb_language_from_string (const char *str, int len);
```

```rust
pub fn hb_language_from_string(str_: *const c_char, len: c_int) -> hb_language_t;
```

Converts a string representing a BCP 47 language tag to the corresponding
`hb_language_t`.

**Parameters**

| Parameter | Meaning | Null allowed |
| --- | --- | --- |
| `str_` | The BCP 47 tag, e.g. `"en"`, `"fa-IR"`, `"zh-Hant-TW"`. | yes — returns `HB_LANGUAGE_INVALID` |
| `len` | Length of `str_`, or `-1` if NUL-terminated. | — |

**Returns** — the interned language, or `HB_LANGUAGE_INVALID` for a null,
empty, or leading-NUL string, or if interning failed for lack of memory.

**Ownership** — `transfer none`. The value is interned in a process-wide table
and lives for the lifetime of the process; never free it. Two calls with
equivalent tags return the *same* pointer, so `==` is a valid comparison.

**Notes** — since HarfBuzz 0.9.2. When `len >= 0` the implementation copies into
a 64-byte stack buffer and **truncates to 63 bytes**; pass `-1` for a
NUL-terminated string if a tag might be longer. Tags are canonicalised (case is
normalised) before interning, so `"EN"` and `"en"` give the same value. The
interning table takes a lock, so this is thread-safe, but it is not free —
convert once and cache.

#### `hb_language_to_string`

```c
const char *hb_language_to_string (hb_language_t language);
```

```rust
pub fn hb_language_to_string(language: hb_language_t) -> *const c_char;
```

Converts an `hb_language_t` to a string.

**Returns** — a NUL-terminated string representing the language, or **null** for
`HB_LANGUAGE_INVALID`. This is the canonicalised form, which may differ in case
from what you passed to `hb_language_from_string`.

**Ownership** — `transfer none`; owned by the interning table and must not be
freed by the caller. Valid for the lifetime of the process.

**Notes** — since HarfBuzz 0.9.2. The null return for the invalid language is
easy to miss and is the usual cause of a crash in code that pipes the result
straight into `printf ("%s")`.

#### `hb_language_get_default`

```c
hb_language_t hb_language_get_default (void);
```

```rust
pub fn hb_language_get_default() -> hb_language_t;
```

Fetches the default language from the current locale.

**Returns** — the locale's language as an `hb_language_t`, cached after the
first call.

**Ownership** — `transfer none`; interned, never freed.

**Notes** — since HarfBuzz 0.9.2. **Not thread-safe the first time it is
called**: it calls `setlocale (LC_CTYPE, NULL)` to fetch the current locale, and
the underlying `setlocale` is not thread-safe in many implementations. Call it
once before multiple threads can reach it. Within HarfBuzz itself it is used
only from `hb_buffer_guess_segment_properties`, which inherits the same caveat.
Subsequent calls read an atomic cache and are cheap and safe.

#### `hb_language_matches`

```c
hb_bool_t hb_language_matches (hb_language_t language,
                               hb_language_t specific);
```

```rust
pub fn hb_language_matches(language: hb_language_t, specific: hb_language_t) -> hb_bool_t;
```

Checks whether `specific` is the same as, or a more specific version of,
`language`. For example `"fa_IR.utf8"` is a more specific tag for `"fa"` and for
`"fa_IR"`.

**Returns** — true if the languages match. The test is a prefix comparison: true
when the two are identical, or when `language`'s string is a prefix of
`specific`'s **and** the next character in `specific` is `'\0'` or `'-'`. So
`"fa"` matches `"fa-IR"` but not `"fake"`. Either argument being
`HB_LANGUAGE_INVALID` gives false unless both are, in which case the identity
check makes it true.

**Notes** — since HarfBuzz 5.0.0. The relation is asymmetric — the general tag
goes first, the specific one second — and getting the order wrong is a common
mistake.

### Features and variations

#### `hb_feature_from_string`

```c
hb_bool_t hb_feature_from_string (const char *str, int len,
                                  hb_feature_t *feature);
```

```rust
pub fn hb_feature_from_string(
    str_: *const c_char,
    len: c_int,
    feature: *mut hb_feature_t,
) -> hb_bool_t;
```

Parses a string into an `hb_feature_t`. This is the syntax `hb-shape --features`
accepts.

**Parameters**

| Parameter | Meaning | Null allowed |
| --- | --- | --- |
| `str_` | The string to parse. | no — dereferenced when `len < 0` |
| `len` | Length of `str_`, or `-1` if NUL-terminated. | — |
| `feature` | Out-parameter initialised with the parsed values. | yes — parsing still happens, only the result is discarded |

**Returns** — true if `str_` was successfully parsed, false otherwise. **On
failure `*feature` is zeroed**, not left untouched, so a failed parse produces
`{tag: 0, value: 0, start: 0, end: 0}` rather than garbage.

**Syntax** — all valid CSS `font-feature-settings` values other than `normal`
and the global values are accepted, though not documented in the table below.
CSS string escapes are not supported. The range indices refer to the positions
*between* Unicode characters; the position before the first character is always
0. The format is Python-esque:

| Syntax | Value | Start | End | Meaning |
| --- | ---: | ---: | ---: | --- |
| `kern` | 1 | 0 | ∞ | Turn feature on |
| `+kern` | 1 | 0 | ∞ | Turn feature on |
| `-kern` | 0 | 0 | ∞ | Turn feature off |
| `kern=0` | 0 | 0 | ∞ | Turn feature off |
| `kern=1` | 1 | 0 | ∞ | Turn feature on |
| `aalt=2` | 2 | 0 | ∞ | Choose 2nd alternate |
| `kern[]` | 1 | 0 | ∞ | Turn feature on |
| `kern[:]` | 1 | 0 | ∞ | Turn feature on |
| `kern[5:]` | 1 | 5 | ∞ | Turn feature on, partial |
| `kern[:5]` | 1 | 0 | 5 | Turn feature on, partial |
| `kern[3:5]` | 1 | 3 | 5 | Turn feature on, range |
| `kern[3]` | 1 | 3 | 4 | Turn feature on, single char |
| `aalt[3:5]=2` | 2 | 3 | 5 | Turn 2nd alternate on for a range |

"∞" is `HB_FEATURE_GLOBAL_END`.

**Notes** — since HarfBuzz 0.9.5. Leading and trailing whitespace is tolerated,
but the whole string must be consumed or the parse fails.

#### `hb_feature_to_string`

```c
void hb_feature_to_string (hb_feature_t *feature,
                           char *buf, unsigned int size);
```

```rust
pub fn hb_feature_to_string(feature: *mut hb_feature_t, buf: *mut c_char, size: c_uint);
```

Converts an `hb_feature_t` into a NUL-terminated string in the format understood
by `hb_feature_from_string`.

**Parameters**

| Parameter | Meaning | Null allowed |
| --- | --- | --- |
| `feature` | The feature to convert. Not modified despite the non-`const` pointer. | no |
| `buf` | Caller-allocated output buffer. | no when `size > 0` |
| `size` | The allocated size of `buf`, in bytes. | — |

**Returns** — nothing. The client is responsible for allocating a big enough
buffer; **128 bytes is more than enough**. The result is truncated to fit and
always NUL-terminated when `size > 0`; a `size` of 0 makes the call a no-op.

**Notes** — since HarfBuzz 0.9.5. The feature value is **omitted when it is 1**
(so `{kern, 1, global}` prints as `"kern"`, not `"kern=1"`), a value of 0 is
rendered as a leading `-`, trailing spaces in the tag are trimmed, and the
string never contains whitespace.

#### `hb_variation_from_string`

```c
hb_bool_t hb_variation_from_string (const char *str, int len,
                                    hb_variation_t *variation);
```

```rust
pub fn hb_variation_from_string(
    str_: *const c_char,
    len: c_int,
    variation: *mut hb_variation_t,
) -> hb_bool_t;
```

Parses a string into an `hb_variation_t`.

**Parameters** — as `hb_feature_from_string`: `len` of `-1` means
NUL-terminated, and `variation` may be null.

**Returns** — true if `str_` was successfully parsed, false otherwise. On
failure `*variation` is zeroed.

**Syntax** — a tag, optionally followed by an equals sign, followed by a number:
`wght=500`, `slnt=-7.5`, or just `wght 500`. All valid CSS
`font-variation-settings` values other than `normal` and `inherited` are also
accepted.

**Notes** — since HarfBuzz 1.4.2.

#### `hb_variation_to_string`

```c
void hb_variation_to_string (hb_variation_t *variation,
                             char *buf, unsigned int size);
```

```rust
pub fn hb_variation_to_string(variation: *mut hb_variation_t, buf: *mut c_char, size: c_uint);
```

Converts an `hb_variation_t` into a NUL-terminated string in the format
understood by `hb_variation_from_string` — `"wght=500"`.

**Returns** — nothing. As with features, the client allocates the buffer and 128
bytes is more than enough; the output is truncated to `size` and the string
never contains whitespace.

**Notes** — since HarfBuzz 1.4.2. The number is formatted with `"%g"` under a
temporarily-installed **C locale**, so the decimal separator is always `.`
regardless of the process locale — which is what makes the round trip through
`hb_variation_from_string` reliable.

### Colours

The four accessors below are declared as functions *and* defined as function-like
macros in the header, so a C caller normally gets the inline shift-and-mask; the
exported functions exist for language bindings, and Rust binds to those. Both
compute the same value. All since HarfBuzz 2.1.0, and all filed by gtk-doc under
the `hb-ot-color` section (see `ot_color.md`).

#### `hb_color_get_alpha`

```c
uint8_t hb_color_get_alpha (hb_color_t color);
#define hb_color_get_alpha(color) ((color) & 0xFF)
```

```rust
pub fn hb_color_get_alpha(color: hb_color_t) -> u8;
```

Fetches the alpha channel — bits 7–0.

#### `hb_color_get_red`

```c
uint8_t hb_color_get_red (hb_color_t color);
#define hb_color_get_red(color) (((color) >> 8) & 0xFF)
```

```rust
pub fn hb_color_get_red(color: hb_color_t) -> u8;
```

Fetches the red channel — bits 15–8.

#### `hb_color_get_green`

```c
uint8_t hb_color_get_green (hb_color_t color);
#define hb_color_get_green(color) (((color) >> 16) & 0xFF)
```

```rust
pub fn hb_color_get_green(color: hb_color_t) -> u8;
```

Fetches the green channel — bits 23–16.

#### `hb_color_get_blue`

```c
uint8_t hb_color_get_blue (hb_color_t color);
#define hb_color_get_blue(color) (((color) >> 24) & 0xFF)
```

```rust
pub fn hb_color_get_blue(color: hb_color_t) -> u8;
```

Fetches the blue channel — bits 31–24.

### Memory

These four wrap whatever allocator HarfBuzz was configured with at compile time
— typically the C library's, but a custom one can be substituted when building.
The header calls them "not of much use to clients", and that is right: you need
them only when handing a buffer to a HarfBuzz function that will free it later,
or when freeing a buffer HarfBuzz allocated for you. All since HarfBuzz 11.0.0.

#### `hb_malloc`

```c
void *hb_malloc (size_t size);
```

```rust
pub fn hb_malloc(size: usize) -> *mut c_void;
```

Allocates `size` bytes using the allocator set at compile time — typically just
`malloc`. Returns a pointer to the allocated memory, or null on failure. The
caller owns it and must release it with `hb_free`.

#### `hb_calloc`

```c
void *hb_calloc (size_t nmemb, size_t size);
```

```rust
pub fn hb_calloc(nmemb: usize, size: usize) -> *mut c_void;
```

Allocates `nmemb` elements of `size` bytes each, initialised to zero, using the
allocator set at compile time — typically just `calloc`. Returns null on
failure. Release with `hb_free`.

#### `hb_realloc`

```c
void *hb_realloc (void *ptr, size_t size);
```

```rust
pub fn hb_realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
```

Reallocates the memory pointed to by `ptr` to `size` bytes, using the allocator
set at compile time — typically just `realloc`. `ptr` must have come from
`hb_malloc`, `hb_calloc`, or `hb_realloc`. Returns null on failure, in which
case (following `realloc` semantics) the original block is still valid and still
owned by the caller.

#### `hb_free`

```c
void hb_free (void *ptr);
```

```rust
pub fn hb_free(ptr: *mut c_void);
```

Frees the memory pointed to by `ptr`, using the allocator set at compile time —
typically just `free`. Tolerates null, as `free` does.

## Preprocessor and build plumbing

These entries appear in upstream's section file under `<SUBSECTION Private>`.
They are part of the header's contents but are not API you call; several have no
Rust counterpart at all, because the concerns they address do not exist in Rust.

### `HB_BEGIN_DECLS` / `HB_END_DECLS`

```c
#ifdef __cplusplus
#  define HB_BEGIN_DECLS extern "C" {
#  define HB_END_DECLS   }
#else
#  define HB_BEGIN_DECLS
#  define HB_END_DECLS
#endif
```

Bracket every declaration in every public header so that a C++ translation unit
gets C linkage. No Rust counterpart — `unsafe extern "C" { … }` blocks in the
`-sys` crate serve the same purpose. Both are overridable: the header only
defines them `#ifndef HB_BEGIN_DECLS`, so an embedder can substitute its own.

### `HB_EXTERN`

```c
#ifndef HB_EXTERN
#define HB_EXTERN extern
#endif
```

Prefixes every exported function declaration. It defaults to plain `extern`, and
the build system redefines it to add visibility or DLL-import/export attributes
(`__attribute__((visibility("default")))`, `__declspec(dllexport)`, and so on).
Because it is guarded by `#ifndef`, a client that needs different linkage — for
example when linking HarfBuzz statically on Windows — can define it before
including `hb.h`. No Rust counterpart.

### `HB_DEPRECATED`

```c
#if defined(__GNUC__) && ((__GNUC__ > 3) || (__GNUC__ == 3 && __GNUC_MINOR__ >= 1))
#define HB_DEPRECATED __attribute__((__deprecated__))
#elif defined(_MSC_VER) && (_MSC_VER >= 1300)
#define HB_DEPRECATED __declspec(deprecated)
#else
#define HB_DEPRECATED
#endif
```

Marks a declaration deprecated on compilers that support it, and expands to
nothing elsewhere. Applied to the functions documented in `deprecated.md` and
`ot_deprecated.md`. The Rust analogue is `#[deprecated]`, which this crate
applies to the same functions.

### `HB_DEPRECATED_FOR`

```c
#if defined(__GNUC__) && ((__GNUC__ > 4) || (__GNUC__ == 4 && __GNUC_MINOR__ >= 5))
#define HB_DEPRECATED_FOR(f) __attribute__((__deprecated__("Use '" #f "' instead")))
#elif defined(_MSC_FULL_VER) && (_MSC_FULL_VER > 140050320)
#define HB_DEPRECATED_FOR(f) __declspec(deprecated("is deprecated. Use '" #f "' instead"))
#else
#define HB_DEPRECATED_FOR(f) HB_DEPRECATED
#endif
```

As `HB_DEPRECATED`, but names the replacement in the compiler's warning. Falls
back to a bare `HB_DEPRECATED` on toolchains that cannot carry a message. The
Rust analogue is `#[deprecated(note = "...")]`.

### `HB_H_IN`, `HB_OT_H_IN`, `HB_AAT_H_IN`, `HB_SUBSET_H_IN`

Single-header guards. Each umbrella header defines its marker before including
its sub-headers and `#undef`s it afterwards:

| Macro | Defined by | Guards |
| --- | --- | --- |
| `HB_H_IN` | `hb.h` | `hb-common.h`, `hb-blob.h`, `hb-buffer.h`, `hb-face.h`, `hb-font.h`, and the rest of the core |
| `HB_OT_H_IN` | `hb-ot.h` | the `hb-ot-*.h` headers |
| `HB_AAT_H_IN` | `hb-aat.h` | the `hb-aat-*.h` headers |
| `HB_SUBSET_H_IN` | `hb-subset.h` | the `hb-subset-*.h` headers |

Each guarded header starts with a check like

```c
#if !defined(HB_H_IN) && !defined(HB_NO_SINGLE_HEADER_ERROR)
#error "Include <hb.h> instead."
#endif
```

so including `hb-buffer.h` directly is a compile error. Define
`HB_NO_SINGLE_HEADER_ERROR` to opt out. None of this reaches Rust: the `-sys`
crate declares the symbols itself and never includes the headers at build time.

### `int8_t`, `int16_t`, `int32_t`, `int64_t`, `uint8_t`, `uint16_t`, `uint32_t`, `uint64_t`

The fixed-width integer types the rest of the API is built from. `hb-common.h`
does not define them itself in the normal case — it includes `<inttypes.h>` (or
`<sys/inttypes.h>` on AIX, or `<stdint.h>` on Visual Studio 2013) and lets the
platform provide them. The one exception is Visual Studio versions before 2010,
which shipped no `stdint.h`; for those the header typedefs all eight from the
`__int8` … `__int64` intrinsics.

They appear in the section file because gtk-doc needs to know about them, not
because HarfBuzz owns them. In Rust they map to the built-in `i8`…`u64`, and
this crate uses those names directly.

## Usage

### Tags, both ways

```c
#include <hb.h>
#include <stdio.h>

void tags (void)
{
  hb_tag_t kern = HB_TAG ('k','e','r','n');
  hb_tag_t parsed = hb_tag_from_string ("liga", -1);

  char buf[5] = {0};
  hb_tag_to_string (kern, buf);        /* writes exactly 4 bytes, no NUL */
  printf ("%s\n", buf);                /* kern */

  printf ("%c%c%c%c\n", HB_UNTAG (parsed));   /* liga */
}
```

```rust
use core::ffi::{c_char, c_int};
use harfbuzz_sys::{HB_TAG, HB_UNTAG, hb_tag_from_string, hb_tag_t, hb_tag_to_string};

const KERN: hb_tag_t = HB_TAG(b'k', b'e', b'r', b'n');

/// Render a tag as the four characters it encodes.
fn tag_to_string(tag: hb_tag_t) -> [u8; 4] {
    let mut buf = [0u8; 4];
    // SAFETY: `hb_tag_to_string` writes exactly four bytes and `buf` is four
    // bytes long. The result is not NUL-terminated, which is why the buffer is
    // sized exactly.
    unsafe { hb_tag_to_string(tag, buf.as_mut_ptr() as *mut c_char) };
    buf
}

/// Parse a tag from a Rust string without needing a NUL terminator.
fn tag_from_str(s: &str) -> hb_tag_t {
    // SAFETY: the pointer covers `s.len()` readable bytes, and passing an
    // explicit length means HarfBuzz never looks for a NUL terminator.
    unsafe { hb_tag_from_string(s.as_ptr() as *const c_char, s.len() as c_int) }
}

// `HB_UNTAG` returns an array in Rust, unlike the C macro's argument list.
const KERN_BYTES: [u8; 4] = HB_UNTAG(KERN);
```

### Building a feature list for `hb_shape`

```c
hb_feature_t features[3];

/* Turn ligatures off everywhere. */
features[0].tag   = HB_TAG ('l','i','g','a');
features[0].value = 0;
features[0].start = HB_FEATURE_GLOBAL_START;
features[0].end   = HB_FEATURE_GLOBAL_END;

/* Small caps for clusters [3, 8). */
features[1].tag   = HB_TAG ('s','m','c','p');
features[1].value = 1;
features[1].start = 3;
features[1].end   = 8;

/* Or let the parser do it. */
hb_feature_from_string ("aalt[3:5]=2", -1, &features[2]);

hb_shape (font, buffer, features, 3);
```

```rust
use harfbuzz_sys::{
    HB_FEATURE_GLOBAL_END, HB_FEATURE_GLOBAL_START, HB_TAG, hb_feature_t,
};

const FEATURES: [hb_feature_t; 2] = [
    // Ligatures off for the whole buffer.
    hb_feature_t {
        tag: HB_TAG(b'l', b'i', b'g', b'a'),
        value: 0,
        start: HB_FEATURE_GLOBAL_START,
        end: HB_FEATURE_GLOBAL_END,
    },
    // Small caps for clusters 3..8.
    hb_feature_t {
        tag: HB_TAG(b's', b'm', b'c', b'p'),
        value: 1,
        start: 3,
        end: 8,
    },
];
```

### Parsing a feature or variation string from a user

```rust
use core::ffi::{c_char, c_int};
use harfbuzz_sys::{hb_feature_from_string, hb_feature_t};

/// Parse one `hb-shape`-style feature string, e.g. `"-liga"` or `"aalt[3:5]=2"`.
fn parse_feature(s: &str) -> Option<hb_feature_t> {
    let mut feature = core::mem::MaybeUninit::<hb_feature_t>::uninit();
    // SAFETY: the pointer covers `s.len()` readable bytes; `feature` is
    // writable and is fully initialised by HarfBuzz on both the success and
    // the failure path (failure zeroes it).
    let ok = unsafe {
        hb_feature_from_string(
            s.as_ptr() as *const c_char,
            s.len() as c_int,
            feature.as_mut_ptr(),
        )
    };
    if ok != 0 {
        // SAFETY: a true return means HarfBuzz wrote a complete struct.
        Some(unsafe { feature.assume_init() })
    } else {
        None
    }
}
```

### Rendering a feature back to a string

```rust
use core::ffi::{c_char, c_uint};
use harfbuzz_sys::{hb_feature_t, hb_feature_to_string};

fn feature_to_string(mut feature: hb_feature_t) -> alloc::string::String {
    // 128 bytes is documented as more than enough.
    let mut buf = [0u8; 128];
    // SAFETY: `buf` is 128 writable bytes and we pass that as `size`; HarfBuzz
    // truncates to fit and always NUL-terminates. `feature` is not modified
    // despite the `*mut` parameter.
    unsafe {
        hb_feature_to_string(&mut feature, buf.as_mut_ptr() as *mut c_char, buf.len() as c_uint)
    };
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    alloc::string::String::from_utf8_lossy(&buf[..end]).into_owned()
}
```

### Languages: intern once, compare by pointer

```rust
use core::ffi::{CStr, c_char, c_int};
use harfbuzz_sys::{
    HB_LANGUAGE_INVALID, hb_language_from_string, hb_language_matches, hb_language_t,
    hb_language_to_string,
};

fn language(tag: &str) -> hb_language_t {
    // SAFETY: the pointer covers `tag.len()` readable bytes. The returned
    // handle is interned for the life of the process and needs no cleanup.
    unsafe { hb_language_from_string(tag.as_ptr() as *const c_char, tag.len() as c_int) }
}

fn language_name(lang: hb_language_t) -> Option<&'static str> {
    if lang == HB_LANGUAGE_INVALID {
        return None;
    }
    // SAFETY: `lang` is a valid interned language, so the returned pointer is
    // a non-null static NUL-terminated string valid for the process lifetime.
    let s = unsafe { hb_language_to_string(lang) };
    if s.is_null() {
        return None;
    }
    // SAFETY: as above — NUL-terminated and immortal.
    unsafe { CStr::from_ptr(s) }.to_str().ok()
}

fn demo() {
    let fa = language("fa");
    let fa_ir = language("fa-IR");

    // Interning means equal tags give equal pointers.
    assert_eq!(fa, language("FA"));

    // SAFETY: both handles are valid; the general tag goes first.
    assert!(unsafe { hb_language_matches(fa, fa_ir) } != 0);
    assert!(unsafe { hb_language_matches(fa_ir, fa) } == 0);
}
```

### Classifying a direction safely

```rust
use harfbuzz_sys::{
    HB_DIRECTION_IS_BACKWARD, HB_DIRECTION_IS_HORIZONTAL, HB_DIRECTION_IS_VALID,
    HB_DIRECTION_REVERSE, hb_direction_t,
};

fn describe(dir: hb_direction_t) -> &'static str {
    // Every other predicate requires a valid direction, so test this first.
    if !HB_DIRECTION_IS_VALID(dir) {
        return "invalid";
    }
    match (HB_DIRECTION_IS_HORIZONTAL(dir), HB_DIRECTION_IS_BACKWARD(dir)) {
        (true, false) => "left-to-right",
        (true, true) => "right-to-left",
        (false, false) => "top-to-bottom",
        (false, true) => "bottom-to-top",
    }
}

fn flipped(dir: hb_direction_t) -> Option<hb_direction_t> {
    HB_DIRECTION_IS_VALID(dir).then(|| HB_DIRECTION_REVERSE(dir))
}
```

### A user-data key in Rust

HarfBuzz keys on the *address*, so what matters is that the static has a unique,
stable address and outlives every object it is attached to.

```rust
use core::ffi::c_void;
use harfbuzz_sys::{hb_blob_t, hb_blob_set_user_data, hb_user_data_key_t};

// The value is never read; only the address matters. `static mut` is used
// because the C API takes `*mut hb_user_data_key_t`.
static mut MY_KEY: hb_user_data_key_t = hb_user_data_key_t { unused: 0 };

unsafe extern "C" fn drop_boxed_u32(data: *mut c_void) {
    // SAFETY: `data` is the pointer we leaked below, and HarfBuzz calls this
    // exactly once.
    drop(unsafe { alloc::boxed::Box::from_raw(data as *mut u32) });
}

/// # Safety
/// `blob` must be live.
unsafe fn attach(blob: *mut hb_blob_t, value: u32) -> bool {
    let data = alloc::boxed::Box::into_raw(alloc::boxed::Box::new(value)) as *mut c_void;
    // SAFETY: `blob` is live; `MY_KEY`'s address is stable for the whole
    // program; ownership of `data` transfers to HarfBuzz unconditionally, so
    // there is nothing to free here even if this returns false.
    unsafe {
        hb_blob_set_user_data(blob, &raw mut MY_KEY, data, Some(drop_boxed_u32), 1) != 0
    }
}
```

### Colours

```rust
use harfbuzz_sys::{HB_COLOR, hb_color_get_alpha, hb_color_get_blue, hb_color_get_green,
                   hb_color_get_red, hb_color_t};

// Note the argument order: blue, green, red, alpha.
const OPAQUE_RED: hb_color_t = HB_COLOR(0x00, 0x00, 0xFF, 0xFF);

fn channels(c: hb_color_t) -> (u8, u8, u8, u8) {
    // SAFETY: these are pure functions over an integer; there is no pointer
    // and no state involved.
    unsafe {
        (
            hb_color_get_red(c),
            hb_color_get_green(c),
            hb_color_get_blue(c),
            hb_color_get_alpha(c),
        )
    }
}
```

## Pitfalls

### `HB_COLOR` takes B, G, R, A — in that order

It is defined as `HB_TAG(b, g, r, a)`, so blue occupies the most significant
byte. Writing `HB_COLOR(r, g, b, a)` compiles fine and produces a colour with
red and blue swapped. The accessors are named correctly, so the mistake usually
surfaces as blue text where red was intended.

### `hb_tag_to_string` does not NUL-terminate

It writes exactly four bytes. A `char buf[4]` passed to `printf ("%s")` reads
past the end. Use `char buf[5] = {0}` in C, or a `[u8; 4]` you convert
explicitly in Rust.

### `hb_tag_from_string` pads and truncates instead of failing

`"aa"` becomes `HB_TAG('a','a',' ',' ')` and `"kerning"` becomes
`HB_TAG('k','e','r','n')`. Neither is an error. Tags produced from user input
should be round-tripped through `hb_tag_to_string` and compared if you care.
Only a null pointer, a zero length, or a leading NUL give `HB_TAG_NONE`.

### `hb_direction_from_string` matches on the first letter only

`"lizard"` is `HB_DIRECTION_LTR` and `"backwards"` is `HB_DIRECTION_BTT`. If the
string comes from a user or a config file and you want validation, compare the
round trip against `hb_direction_to_string`.

### Five of the six direction predicates require a valid direction

`HB_DIRECTION_IS_HORIZONTAL`, `_IS_VERTICAL`, `_IS_FORWARD`, `_IS_BACKWARD`, and
`HB_DIRECTION_REVERSE` are bit tricks that assume the value is in `4..=7`.
Applied to `HB_DIRECTION_INVALID` (0) they give confidently wrong answers —
`HB_DIRECTION_REVERSE(HB_DIRECTION_INVALID)` is 1, which is not a direction.
Gate on `HB_DIRECTION_IS_VALID` first.

### `hb_language_to_string` returns null for the invalid language

Every other `*_to_string` in this header returns a string for every input
(`hb_direction_to_string` gives `"invalid"`). This one returns null when passed
`HB_LANGUAGE_INVALID`, which is exactly the value you get from a failed
`hb_language_from_string`. Check before dereferencing.

### `hb_language_from_string` truncates at 63 bytes when given a length

When `len >= 0` the implementation copies into a 64-byte stack buffer and clamps
the length to 63. Long BCP 47 tags — with several extensions or a private-use
subtag — are silently cut short and intern as a different language. Pass `-1`
with a NUL-terminated string to avoid the copy and the limit.

### `hb_language_matches` is asymmetric

The first argument is the general tag, the second the specific one:
`hb_language_matches(fa, fa_IR)` is true, `hb_language_matches(fa_IR, fa)` is
false. The parameter names in the header (`language`, `specific`) are the only
hint, and swapping them yields a plausible-looking but always-false test.

### `hb_language_get_default` is not thread-safe on first use

It calls `setlocale (LC_CTYPE, NULL)`, which is not thread-safe in many C
libraries. Call it once during start-up from a single thread. Everything that
reaches it inherits the hazard — most importantly
`hb_buffer_guess_segment_properties`.

### Languages are never freed, so do not intern unbounded input

The interning table grows for the life of the process and is only released at
exit. Interning attacker-controlled or unbounded strings is an unbounded memory
leak. Validate or bound the set of language tags you convert.

### Failed feature and variation parses zero the output

`hb_feature_from_string` and `hb_variation_from_string` write a zeroed struct on
failure rather than leaving the output untouched. A caller that ignores the
boolean return ends up with `{tag: 0, value: 0, start: 0, end: 0}` — a feature
with tag `HB_TAG_NONE` — which `hb_shape` accepts and quietly does nothing with.
Always check the return value.

### `hb_feature_to_string` omits a value of 1

`{kern, 1, global}` renders as `"kern"`, not `"kern=1"`, and value 0 renders as
`"-kern"`. That round-trips correctly through `hb_feature_from_string`, but
string comparison against a hand-written `"kern=1"` will not match.

### `hb_script_t` is not a closed set

Values are ISO 15924 tags, and any four-byte value that *looks* like one is
passed through by `hb_script_from_iso15924_tag`. Font and text data can
legitimately produce a script with no `HB_SCRIPT_*` constant, which is precisely
why this crate transcribes the enumeration as a `c_int` alias plus constants
rather than a Rust `enum` — a Rust `enum` holding an undeclared discriminant is
undefined behaviour. Match on it with a `_` arm.

### Script folding is lossy

`Hans` and `Hant` both become `HB_SCRIPT_HAN`; `Syre`, `Syrj`, and `Syrn` all
become `HB_SCRIPT_SYRIAC`. `hb_script_to_iso15924_tag` returns the folded tag,
so the round trip does not preserve the original spelling. Keep the original
string if you need it.

### `HB_UNTAG` has a different shape in C and Rust

In C it expands to four comma-separated expressions meant to be spliced into an
argument list; in Rust it returns a `[u8; 4]`. Code translated mechanically from
C — `printf("%c%c%c%c", HB_UNTAG(tag))` — has no direct Rust equivalent.

### The `hb_malloc` family is not interchangeable with the system allocator

HarfBuzz may have been built with a custom allocator. Memory from `hb_malloc`
must be released with `hb_free`, and memory from the system `malloc` must not be
passed to `hb_free`. In Rust the same applies to Rust's global allocator: never
`hb_free` a pointer that came from a `Box`, and never `Box::from_raw` a pointer
that came from `hb_malloc`.

### `hb_glyph_extents_t.height` is usually negative

In a coordinate system that grows up — HarfBuzz's default — the height measured
from the top extremum down to the bottom extremum is negative. Code that assumes
positive dimensions will compute empty or inverted boxes.

### The user-data key is an address, not a value

Two `hb_user_data_key_t` objects with identical contents are still different
keys, and the same object used from two modules is the same key. The key must
outlive every object it was attached to — a local variable is a dangling key —
and in Rust you must make sure the compiler does not merge two identical
`static`s into one address.

### Destroy callbacks own their data even when the call fails

Every API that takes a `user_data` / `destroy` pair calls `destroy(user_data)`
itself on its error paths. Freeing the data after a failed
`hb_blob_create_or_fail` or `hb_buffer_set_message_func` is a double free. Leak
the allocation into the call and let the callback own it from that moment on.
