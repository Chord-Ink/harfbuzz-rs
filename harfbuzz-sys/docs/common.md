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
