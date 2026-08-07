# Buffers

Header: `hb-buffer.h` — Rust module: `harfbuzz_sys::buffer` (glob re-exported at
the crate root).

## Overview

A **buffer** is the object you shape *with*. It plays two roles at once, which
is the single most important thing to understand about it: before shaping it
holds the **input characters** — a sequence of Unicode code points plus the
attributes that decide how they are shaped (direction, script, language, flags)
— and after shaping the very same object holds the **output glyphs**, a
sequence of glyph indices with positions and cluster values. `hb_shape` rewrites
the buffer in place. There is no separate "result" object.

The normal lifecycle is a loop:

1. `hb_buffer_create` once, and keep the buffer.
2. `hb_buffer_add_utf8` (or another `add_*` function) to put text in.
3. Set the segment properties — `hb_buffer_set_direction`,
   `hb_buffer_set_script`, `hb_buffer_set_language`, or
   `hb_buffer_guess_segment_properties` to infer them.
4. `hb_shape` (documented in `shape.md`).
5. Read the results with `hb_buffer_get_glyph_infos` and
   `hb_buffer_get_glyph_positions`.
6. `hb_buffer_reset` or `hb_buffer_clear_contents`, then go back to step 2.

Reusing one buffer across many shaping calls is the intended pattern and the
reason `hb_buffer_clear_contents` exists: the buffer keeps its allocated arrays,
so a long paragraph loop performs at most a handful of allocations in total.

The **content type** (`hb_buffer_content_type_t`) records which of the two roles
the buffer is currently in: invalid (empty), Unicode (characters), or glyphs
(shaped). You almost never set it by hand — the `add_*` functions move an empty
buffer to Unicode, `hb_shape` moves a Unicode buffer to glyphs, and
`hb_buffer_reset` / `hb_buffer_clear_contents` / `hb_buffer_set_length(0)` move
it back to invalid. Functions that only make sense in one role assert on the
content type, so calling `hb_buffer_add_utf8` on a shaped buffer is a
programming error rather than a runtime failure you can recover from.

**Clusters** are how the output is tied back to the input. Every
`hb_glyph_info_t` carries a `cluster` value, which by default is the byte or
code-unit index of the character it came from. Because shaping is many-to-many,
several glyphs can share a cluster (one character produced a base plus a mark)
and several characters can collapse into one (a ligature, which takes the
smallest of their cluster values). The **cluster level**
(`hb_buffer_cluster_level_t`) chooses how aggressively HarfBuzz merges those
values; it is a genuine semantic choice, not a performance knob, and it is
described in detail under `hb_buffer_cluster_level_t` below.

**Pre-context and post-context** are the other half of correct
text-to-glyph mapping. Shaping a run out of the middle of a paragraph is not the
same as shaping that run alone: an Arabic letter joins differently depending on
what precedes and follows it, and a combining mark at the start of a run needs
its base. The `add_*` functions therefore take a `text` pointer covering as much
text as you have plus an `item_offset` / `item_length` window naming the part
that actually enters the buffer. Everything outside the window is retained as up
to five code points of context on each side and is used by the shaper without
producing glyphs. Passing only the substring throws that information away.

Buffers are reference counted like every other HarfBuzz object
(`hb_buffer_reference` / `hb_buffer_destroy`) and carry a user-data table. They
have no public `make_immutable` — the only immutable buffer is the shared empty
one from `hb_buffer_get_empty`, on which every mutator silently does nothing.
Allocation failures are also silent: no buffer function returns null, and
instead the buffer records that it ran out of memory. `hb_buffer_allocation_successful`
is the only way to find out, and it is worth calling once after filling a buffer.

Finally, buffers can be **serialized** to and from a textual form
(`hb_buffer_serialize` / `hb_buffer_deserialize_*`) in either a plain-text or a
JSON format. This is what `hb-shape` prints, and it is the standard way to write
shaping regression tests.

## Types

### `hb_buffer_t`

```c
typedef struct hb_buffer_t hb_buffer_t;
```

```rust
crate::opaque_handle! { hb_buffer_t }
```

The main structure holding the input text and its properties before shaping, and
the output glyphs and their information after shaping. Opaque: it has no visible
body in the public header, so it exists only behind a pointer. In Rust it is a
zero-sized `#[repr(C)]` handle that cannot be constructed, copied, or sent
between threads by accident — you always hold `*mut hb_buffer_t`.

It is reference counted, starting at one from `hb_buffer_create`. Internally it
owns two parallel arrays (glyph infos and glyph positions), a
`hb_unicode_funcs_t` reference, the segment properties, the flags and cluster
level, four substitution code points, the random state, up to five code points
of context on each side, and an optional message callback.

### `hb_glyph_info_t`

```c
typedef struct hb_glyph_info_t {
  hb_codepoint_t codepoint;
  /*< private >*/
  hb_mask_t      mask;
  /*< public >*/
  uint32_t       cluster;
  /*< private >*/
  hb_var_int_t   var1;
  hb_var_int_t   var2;
} hb_glyph_info_t;
```

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_glyph_info_t {
    pub codepoint: hb_codepoint_t,
    pub mask: hb_mask_t,
    pub cluster: u32,
    pub var1: hb_var_int_t,
    pub var2: hb_var_int_t,
}
```

One entry per item in the buffer. You obtain an array of these from
`hb_buffer_get_glyph_infos`; the array belongs to the buffer.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `codepoint` | `hb_codepoint_t` | `hb_codepoint_t` (`u32`) | A Unicode code point before shaping, a glyph index in the shaped font after shaping. The name does not change with the meaning. |
| `mask` | `hb_mask_t` | `hb_mask_t` (`u32`) | **Private.** Feature masks used during shaping. Its low bits carry `hb_glyph_flags_t`; read them with `hb_glyph_info_get_glyph_flags`, never by hand. |
| `cluster` | `uint32_t` | `u32` | The index of the character in the original text this glyph corresponds to, or whatever the client passed to `hb_buffer_add`. Several infos may share a cluster (one-to-many substitution); when several characters merge into one glyph (many-to-one) the result carries the smallest of their cluster values. Combining marks share their base's cluster by default; `hb_buffer_set_cluster_level` selects finer-grained handling. |
| `var1` | `hb_var_int_t` | `hb_var_int_t` | **Private.** Shaper scratch space. |
| `var2` | `hb_var_int_t` | `hb_var_int_t` | **Private.** Shaper scratch space. |

The private fields are declared in Rust only so the layout matches C. `Debug` is
implemented by hand (a union cannot derive it) and prints only `codepoint` and
`cluster`.

### `hb_glyph_flags_t`

```c
typedef enum { /*< flags >*/
  HB_GLYPH_FLAG_UNSAFE_TO_BREAK        = 0x00000001,
  HB_GLYPH_FLAG_UNSAFE_TO_CONCAT       = 0x00000002,
  HB_GLYPH_FLAG_SAFE_TO_INSERT_TATWEEL = 0x00000004,
  HB_GLYPH_FLAG_DEFINED                = 0x00000007
} hb_glyph_flags_t;
```

```rust
pub type hb_glyph_flags_t = core::ffi::c_int;
```

Bit flags describing how a glyph relates to the text around it, encoded in the
private `mask` field and read with `hb_glyph_info_get_glyph_flags`. The C
enumeration has no sentinel and no value exceeds `0x7FFFFFFF`, so it is
transcribed as `c_int` plus constants. Since HarfBuzz 1.5.0.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_GLYPH_FLAG_UNSAFE_TO_BREAK` | 0x01 | Breaking the input text at the start of this glyph's cluster requires re-shaping both sides, because the result might differ. When the flag is *absent*, breaking there is safe: the two halves are exactly what you would get by breaking the input and shaping separately. Paragraph layout uses this to avoid re-shaping every line after line-breaking. |
| `HB_GLYPH_FLAG_UNSAFE_TO_CONCAT` | 0x02 | Changing the input text on one side of the start of this glyph's cluster may change the shaping result on the other side. Absence does **not** by itself mean concatenation is safe — only two pieces of text both clear of the flag can be concatenated safely. Implied by `HB_GLYPH_FLAG_UNSAFE_TO_BREAK`. Requires `HB_BUFFER_FLAG_PRODUCE_UNSAFE_TO_CONCAT` to be reliably produced. Since 4.0.0. |
| `HB_GLYPH_FLAG_SAFE_TO_INSERT_TATWEEL` | 0x04 | In scripts that elongate (Arabic, Mongolian, Syriac, …) it is safe to insert a U+0640 TATWEEL before this cluster. It does not say elongation *belongs* there, only that it will not disrupt shaping. Requires `HB_BUFFER_FLAG_PRODUCE_SAFE_TO_INSERT_TATWEEL`. Since 5.1.0. |
| `HB_GLYPH_FLAG_DEFINED` | 0x07 | The bitwise OR of every currently defined flag; the mask `hb_glyph_info_get_glyph_flags` applies. |

The upstream header spells out the intended `UNSAFE_TO_CONCAT` algorithm for
line breaking: iterate back from the break position to the first cluster start
that is *not* unsafe-to-concat, shape from there to the end of the line, and
check that the resulting run is also clear of the flag at its start-of-text
position; if it is, splice it in, otherwise move further back and retry. The
start of the next line is symmetric, iterating forward. One complication: the
buffer API can report flags for the start-of-text position but has no
end-of-text position, which is worked around by shaping more text than needed
and looking for the flag inside the clusters.

### `hb_glyph_position_t`

```c
typedef struct hb_glyph_position_t {
  hb_position_t  x_advance;
  hb_position_t  y_advance;
  hb_position_t  x_offset;
  hb_position_t  y_offset;
  /*< private >*/
  hb_var_int_t   var;
} hb_glyph_position_t;
```

```rust
#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_glyph_position_t {
    pub x_advance: hb_position_t,
    pub y_advance: hb_position_t,
    pub x_offset: hb_position_t,
    pub y_offset: hb_position_t,
    pub var: hb_var_int_t,
}
```

Positioning for a single glyph, in the font's scaled units (see
`hb_font_set_scale` in `font.md`). All values are relative to the current point.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `x_advance` | `hb_position_t` | `i32` | How much the line advances after drawing this glyph when setting text horizontally. |
| `y_advance` | `hb_position_t` | `i32` | How much the line advances after drawing this glyph when setting text vertically. |
| `x_offset` | `hb_position_t` | `i32` | How much the glyph moves on the x-axis before being drawn. Does **not** affect the advance. |
| `y_offset` | `hb_position_t` | `i32` | How much the glyph moves on the y-axis before being drawn. Does **not** affect the advance. |
| `var` | `hb_var_int_t` | `hb_var_int_t` | **Private.** Shaper scratch space. |

The rendering loop is therefore: draw the glyph at
`(cursor_x + x_offset, cursor_y + y_offset)`, then advance the cursor by
`(x_advance, y_advance)`.

### `hb_segment_properties_t`

```c
typedef struct hb_segment_properties_t {
  hb_direction_t  direction;
  hb_script_t     script;
  hb_language_t   language;
  /*< private >*/
  void           *reserved1;
  void           *reserved2;
} hb_segment_properties_t;
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct hb_segment_properties_t {
    pub direction: hb_direction_t,
    pub script: hb_script_t,
    pub language: hb_language_t,
    pub reserved1: *mut c_void,
    pub reserved2: *mut c_void,
}
```

The text properties of a buffer, gettable and settable as a unit.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `direction` | `hb_direction_t` | `c_int` | Text flow direction. See `hb_buffer_set_direction`. |
| `script` | `hb_script_t` | `c_int` | ISO 15924 script. See `hb_buffer_set_script`. |
| `language` | `hb_language_t` | `*const hb_language_impl_t` | BCP 47 language. See `hb_buffer_set_language`. The pointer is interned and never freed. |
| `reserved1` | `void *` | `*mut c_void` | **Private.** Must be null. |
| `reserved2` | `void *` | `*mut c_void` | **Private.** Must be null. |

`hb_segment_properties_equal` compares the reserved fields too, so a struct you
built yourself must zero them or equality and hashing will misbehave. Use
`HB_SEGMENT_PROPERTIES_DEFAULT` as the starting value.

### `hb_buffer_content_type_t`

```c
typedef enum {
  HB_BUFFER_CONTENT_TYPE_INVALID = 0,
  HB_BUFFER_CONTENT_TYPE_UNICODE,
  HB_BUFFER_CONTENT_TYPE_GLYPHS
} hb_buffer_content_type_t;
```

```rust
pub type hb_buffer_content_type_t = core::ffi::c_int;
```

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_BUFFER_CONTENT_TYPE_INVALID` | 0 | Initial value for a new buffer; also the state after `hb_buffer_reset`, `hb_buffer_clear_contents`, and `hb_buffer_set_length(buffer, 0)`. |
| `HB_BUFFER_CONTENT_TYPE_UNICODE` | 1 | The buffer contains input characters, before shaping. |
| `HB_BUFFER_CONTENT_TYPE_GLYPHS` | 2 | The buffer contains output glyphs, the result of shaping. |

### `hb_buffer_flags_t`

```c
typedef enum { /*< flags >*/
  HB_BUFFER_FLAG_DEFAULT                        = 0x00000000u,
  HB_BUFFER_FLAG_BOT                            = 0x00000001u,
  HB_BUFFER_FLAG_EOT                            = 0x00000002u,
  HB_BUFFER_FLAG_PRESERVE_DEFAULT_IGNORABLES    = 0x00000004u,
  HB_BUFFER_FLAG_REMOVE_DEFAULT_IGNORABLES      = 0x00000008u,
  HB_BUFFER_FLAG_DO_NOT_INSERT_DOTTED_CIRCLE    = 0x00000010u,
  HB_BUFFER_FLAG_VERIFY                         = 0x00000020u,
  HB_BUFFER_FLAG_PRODUCE_UNSAFE_TO_CONCAT       = 0x00000040u,
  HB_BUFFER_FLAG_PRODUCE_SAFE_TO_INSERT_TATWEEL = 0x00000080u,
  HB_BUFFER_FLAG_DEFINED                        = 0x000000FFu
} hb_buffer_flags_t;
```

```rust
pub type hb_buffer_flags_t = core::ffi::c_int;
```

Bit flags that change how the buffer is shaped. Since HarfBuzz 0.9.20.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_BUFFER_FLAG_DEFAULT` | 0x00 | No flags. |
| `HB_BUFFER_FLAG_BOT` | 0x01 | The buffer begins a text paragraph, so beginning-of-text handling may be applied. Should usually be set, unless you are passing only part of the text without its full context. |
| `HB_BUFFER_FLAG_EOT` | 0x02 | The buffer ends a text paragraph; the counterpart of `BOT`. |
| `HB_BUFFER_FLAG_PRESERVE_DEFAULT_IGNORABLES` | 0x04 | Characters with the Unicode `Default_Ignorable` property use the font's corresponding glyph instead of being hidden. (Hiding replaces them with the space glyph and zeroes the advance.) Takes precedence over `REMOVE_DEFAULT_IGNORABLES`. |
| `HB_BUFFER_FLAG_REMOVE_DEFAULT_IGNORABLES` | 0x08 | `Default_Ignorable` characters are removed from the glyph string instead of being hidden. Since 1.8.0. |
| `HB_BUFFER_FLAG_DO_NOT_INSERT_DOTTED_CIRCLE` | 0x10 | Do not insert a dotted circle when rendering an incorrect character sequence such as `<0905 093E>`. Since 2.4.0. |
| `HB_BUFFER_FLAG_VERIFY` | 0x20 | `hb_shape` and its variants run verification passes over the results. On failure a buffer message is sent if a message handler is installed, otherwise a message goes to standard error; either way the shaping result may be modified to show the failed output. Since 3.4.0. |
| `HB_BUFFER_FLAG_PRODUCE_UNSAFE_TO_CONCAT` | 0x40 | The shaper produces `HB_GLYPH_FLAG_UNSAFE_TO_CONCAT`. Off by default because it costs time. Since 4.0.0. |
| `HB_BUFFER_FLAG_PRODUCE_SAFE_TO_INSERT_TATWEEL` | 0x80 | The shaper produces `HB_GLYPH_FLAG_SAFE_TO_INSERT_TATWEEL`. Off by default. Since 5.1.0. |
| `HB_BUFFER_FLAG_DEFINED` | 0xFF | The bitwise OR of every currently defined flag. Since 4.4.0. |

### `hb_buffer_cluster_level_t`

```c
typedef enum {
  HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES  = 0,
  HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS = 1,
  HB_BUFFER_CLUSTER_LEVEL_CHARACTERS          = 2,
  HB_BUFFER_CLUSTER_LEVEL_GRAPHEMES           = 3,
  HB_BUFFER_CLUSTER_LEVEL_DEFAULT = HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES
} hb_buffer_cluster_level_t;
```

```rust
pub type hb_buffer_cluster_level_t = core::ffi::c_int;
```

How HarfBuzz groups cluster values, which is one aspect of how it treats
non-base characters during shaping. Since HarfBuzz 0.9.42.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES` | 0 | Non-base characters are merged into the cluster of the preceding base character, and clusters are merged again whenever they would otherwise become non-monotone. |
| `HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS` | 1 | Non-base characters initially keep their own cluster values, which are not merged into preceding base clusters. That lets HarfBuzz do extra work such as reordering runs of adjacent marks. Output is still monotone, but cluster values are more granular. |
| `HB_BUFFER_CLUSTER_LEVEL_CHARACTERS` | 2 | No grouping at all: cluster values are neither merged into base clusters nor forced monotone. The most granular level — it tells you the exact cluster of every character — but harder to consume, since clusters may appear in any order. |
| `HB_BUFFER_CLUSTER_LEVEL_GRAPHEMES` | 3 | Group by grapheme but do not enforce monotone order. Resembles the Unicode Grapheme Cluster algorithm without being identical, and makes HarfBuzz usable as a cheap implementation of it. |
| `HB_BUFFER_CLUSTER_LEVEL_DEFAULT` | 0 | Alias for `MONOTONE_GRAPHEMES`. |

`MONOTONE_GRAPHEMES` is the default only for backward compatibility with older
HarfBuzz. Upstream recommends new programs that do not need that compatibility
use `MONOTONE_CHARACTERS` instead.

### `hb_buffer_serialize_format_t`

```c
typedef enum {
  HB_BUFFER_SERIALIZE_FORMAT_TEXT    = HB_TAG('T','E','X','T'),
  HB_BUFFER_SERIALIZE_FORMAT_JSON    = HB_TAG('J','S','O','N'),
  HB_BUFFER_SERIALIZE_FORMAT_INVALID = HB_TAG_NONE
} hb_buffer_serialize_format_t;
```

```rust
pub type hb_buffer_serialize_format_t = core::ffi::c_int;
```

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_BUFFER_SERIALIZE_FORMAT_TEXT` | `HB_TAG('T','E','X','T')` = 0x54455854 | A human-readable, plain-text format. |
| `HB_BUFFER_SERIALIZE_FORMAT_JSON` | `HB_TAG('J','S','O','N')` = 0x4A534F4E | A machine-readable JSON format. |
| `HB_BUFFER_SERIALIZE_FORMAT_INVALID` | `HB_TAG_NONE` = 0 | Invalid format. Serializing with it returns 0; deserializing returns false. |

Since HarfBuzz 0.9.2.

### `hb_buffer_serialize_flags_t`

```c
typedef enum { /*< flags >*/
  HB_BUFFER_SERIALIZE_FLAG_DEFAULT        = 0x00000000u,
  HB_BUFFER_SERIALIZE_FLAG_NO_CLUSTERS    = 0x00000001u,
  HB_BUFFER_SERIALIZE_FLAG_NO_POSITIONS   = 0x00000002u,
  HB_BUFFER_SERIALIZE_FLAG_NO_GLYPH_NAMES = 0x00000004u,
  HB_BUFFER_SERIALIZE_FLAG_GLYPH_EXTENTS  = 0x00000008u,
  HB_BUFFER_SERIALIZE_FLAG_GLYPH_FLAGS    = 0x00000010u,
  HB_BUFFER_SERIALIZE_FLAG_NO_ADVANCES    = 0x00000020u,
  HB_BUFFER_SERIALIZE_FLAG_DEFINED        = 0x0000003Fu
} hb_buffer_serialize_flags_t;
```

```rust
pub type hb_buffer_serialize_flags_t = core::ffi::c_int;
```

Which glyph information the serializers write out. Since HarfBuzz 0.9.20.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_BUFFER_SERIALIZE_FLAG_DEFAULT` | 0x00 | Serialize glyph names, clusters, and positions. |
| `HB_BUFFER_SERIALIZE_FLAG_NO_CLUSTERS` | 0x01 | Do not serialize glyph clusters. |
| `HB_BUFFER_SERIALIZE_FLAG_NO_POSITIONS` | 0x02 | Do not serialize glyph position information. Forced on automatically when the buffer has no positions. |
| `HB_BUFFER_SERIALIZE_FLAG_NO_GLYPH_NAMES` | 0x04 | Emit glyph indices instead of glyph names. |
| `HB_BUFFER_SERIALIZE_FLAG_GLYPH_EXTENTS` | 0x08 | Also serialize glyph extents (`hb_glyph_extents_t`). Requires a real `font`. |
| `HB_BUFFER_SERIALIZE_FLAG_GLYPH_FLAGS` | 0x10 | Also serialize glyph flags. Since 1.5.0. |
| `HB_BUFFER_SERIALIZE_FLAG_NO_ADVANCES` | 0x20 | Do not serialize advances; offsets then reflect absolute positions. With a partial range (`start` not zero) computing absolute positions costs time proportional to `start`, so serializing in many small chunks becomes quadratic — use a larger `buf_size`. Since 1.8.0. |
| `HB_BUFFER_SERIALIZE_FLAG_DEFINED` | 0x3F | The bitwise OR of every currently defined flag. Since 4.4.0. |

### `hb_buffer_diff_flags_t`

```c
typedef enum { /*< flags >*/
  HB_BUFFER_DIFF_FLAG_EQUAL                 = 0x0000,
  HB_BUFFER_DIFF_FLAG_CONTENT_TYPE_MISMATCH = 0x0001,
  HB_BUFFER_DIFF_FLAG_LENGTH_MISMATCH       = 0x0002,
  HB_BUFFER_DIFF_FLAG_NOTDEF_PRESENT        = 0x0004,
  HB_BUFFER_DIFF_FLAG_DOTTED_CIRCLE_PRESENT = 0x0008,
  HB_BUFFER_DIFF_FLAG_CODEPOINT_MISMATCH    = 0x0010,
  HB_BUFFER_DIFF_FLAG_CLUSTER_MISMATCH      = 0x0020,
  HB_BUFFER_DIFF_FLAG_GLYPH_FLAGS_MISMATCH  = 0x0040,
  HB_BUFFER_DIFF_FLAG_POSITION_MISMATCH     = 0x0080
} hb_buffer_diff_flags_t;
```

```rust
pub type hb_buffer_diff_flags_t = core::ffi::c_int;
```

The kinds of difference `hb_buffer_diff` reports. Since HarfBuzz 1.5.0.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_BUFFER_DIFF_FLAG_EQUAL` | 0x0000 | The buffers are equal — the zero value, so `result == 0` is the "no differences" test. |
| `HB_BUFFER_DIFF_FLAG_CONTENT_TYPE_MISMATCH` | 0x0001 | The buffers have different content types. Returned alone: no further comparison is meaningful. Only reported when *both* buffers are non-empty. |
| `HB_BUFFER_DIFF_FLAG_LENGTH_MISMATCH` | 0x0002 | The buffers differ in length. Per-glyph comparison is skipped, though the reference is still scanned for dotted-circle and `.notdef`. |
| `HB_BUFFER_DIFF_FLAG_NOTDEF_PRESENT` | 0x0004 | `.notdef` (glyph 0) is present in the *reference* buffer. |
| `HB_BUFFER_DIFF_FLAG_DOTTED_CIRCLE_PRESENT` | 0x0008 | The dotted-circle glyph is present in the *reference* buffer. |
| `HB_BUFFER_DIFF_FLAG_CODEPOINT_MISMATCH` | 0x0010 | Some `hb_glyph_info_t.codepoint` differs. |
| `HB_BUFFER_DIFF_FLAG_CLUSTER_MISMATCH` | 0x0020 | Some `hb_glyph_info_t.cluster` differs. |
| `HB_BUFFER_DIFF_FLAG_GLYPH_FLAGS_MISMATCH` | 0x0040 | Some `hb_glyph_flags_t` differs. |
| `HB_BUFFER_DIFF_FLAG_POSITION_MISMATCH` | 0x0080 | Some `hb_glyph_position_t` differs by more than `position_fuzz`. |

### `hb_buffer_message_func_t`

```c
typedef hb_bool_t (*hb_buffer_message_func_t) (hb_buffer_t *buffer,
                                               hb_font_t   *font,
                                               const char  *message,
                                               void        *user_data);
```

```rust
pub type hb_buffer_message_func_t = Option<
    unsafe extern "C" fn(
        buffer: *mut hb_buffer_t,
        font: *mut hb_font_t,
        message: *const c_char,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;
```

A tracing callback invoked at each step of the shaping process. It is called
with the buffer it was set on, the font the buffer is being shaped with, and a
NUL-terminated message describing the step that is about to be performed.

**Returns** `true` to perform the step, `false` to skip it and move on to the
next one — so the callback is a filter, not just an observer.

In Rust the typedef is wrapped in `Option`, so `None` is the null pointer.
`hb_buffer_set_message_func(buffer, None, …)` clears the callback. Since
HarfBuzz 1.1.3. The whole facility is compiled out when HarfBuzz is built with
`HB_NO_BUFFER_MESSAGE`.

## Constants

### `HB_SEGMENT_PROPERTIES_DEFAULT`

```c
#define HB_SEGMENT_PROPERTIES_DEFAULT {HB_DIRECTION_INVALID, \
                                       HB_SCRIPT_INVALID, \
                                       HB_LANGUAGE_INVALID, \
                                       (void *) 0, \
                                       (void *) 0}
```

```rust
pub const HB_SEGMENT_PROPERTIES_DEFAULT: hb_segment_properties_t = hb_segment_properties_t {
    direction: HB_DIRECTION_INVALID,
    script: HB_SCRIPT_INVALID,
    language: HB_LANGUAGE_INVALID,
    reserved1: core::ptr::null_mut(),
    reserved2: core::ptr::null_mut(),
};
```

The segment properties of a freshly created buffer: everything unset. In C it is
a brace initializer, so it can only be used as an initializer; in Rust it is a
real `const` value you can assign. This is also the value a buffer's properties
return to after `hb_buffer_reset` **and** after `hb_buffer_clear_contents`.

### `HB_BUFFER_REPLACEMENT_CODEPOINT_DEFAULT`

```c
#define HB_BUFFER_REPLACEMENT_CODEPOINT_DEFAULT 0xFFFDu
```

```rust
pub const HB_BUFFER_REPLACEMENT_CODEPOINT_DEFAULT: hb_codepoint_t = 0xFFFD;
```

The default code point for replacing invalid characters in a given encoding:
U+FFFD REPLACEMENT CHARACTER. Since HarfBuzz 0.9.31.

## Macros

The three cluster-level predicates are function-like macros in C and `const fn`s
in Rust. All three require `level` to be a valid `hb_buffer_cluster_level_t`;
passing anything else shifts by an out-of-range amount. All since HarfBuzz
11.0.0.

### `HB_BUFFER_CLUSTER_LEVEL_IS_MONOTONE`

```c
#define HB_BUFFER_CLUSTER_LEVEL_IS_MONOTONE(level) \
        ((bool) ((1u << (unsigned) (level)) & \
                 ((1u << HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES) | \
                  (1u << HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS))))
```

```rust
pub const fn HB_BUFFER_CLUSTER_LEVEL_IS_MONOTONE(level: hb_buffer_cluster_level_t) -> bool;
```

True for `MONOTONE_GRAPHEMES` (0) and `MONOTONE_CHARACTERS` (1) — the levels
that force cluster values into monotone order.

### `HB_BUFFER_CLUSTER_LEVEL_IS_GRAPHEMES`

```c
#define HB_BUFFER_CLUSTER_LEVEL_IS_GRAPHEMES(level) \
        ((bool) ((1u << (unsigned) (level)) & \
                 ((1u << HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES) | \
                  (1u << HB_BUFFER_CLUSTER_LEVEL_GRAPHEMES))))
```

```rust
pub const fn HB_BUFFER_CLUSTER_LEVEL_IS_GRAPHEMES(level: hb_buffer_cluster_level_t) -> bool;
```

True for `MONOTONE_GRAPHEMES` (0) and `GRAPHEMES` (3) — the levels that merge
non-base characters into the preceding base's cluster.

### `HB_BUFFER_CLUSTER_LEVEL_IS_CHARACTERS`

```c
#define HB_BUFFER_CLUSTER_LEVEL_IS_CHARACTERS(level) \
        ((bool) ((1u << (unsigned) (level)) & \
                 ((1u << HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS) | \
                  (1u << HB_BUFFER_CLUSTER_LEVEL_CHARACTERS))))
```

```rust
pub const fn HB_BUFFER_CLUSTER_LEVEL_IS_CHARACTERS(level: hb_buffer_cluster_level_t) -> bool;
```

True for `MONOTONE_CHARACTERS` (1) and `CHARACTERS` (2) — the levels that do
*not* group by grapheme. Exactly the negation of
`HB_BUFFER_CLUSTER_LEVEL_IS_GRAPHEMES` over the four valid levels.

## Functions

### Creation, references, and destruction

#### `hb_buffer_create`

```c
hb_buffer_t *hb_buffer_create (void);
```

```rust
pub fn hb_buffer_create() -> *mut hb_buffer_t;
```

Creates a new buffer with all properties at their defaults: no content, invalid
content type, `HB_SEGMENT_PROPERTIES_DEFAULT`, `HB_BUFFER_FLAG_DEFAULT`,
`HB_BUFFER_CLUSTER_LEVEL_DEFAULT`, replacement code point U+FFFD, invisible
glyph 0, not-found glyph 0, not-found-variation-selector `HB_CODEPOINT_INVALID`,
random state 1, and the default Unicode functions.

**Parameters** — none.

**Returns** — a newly allocated buffer with a reference count of one. **Never
returns null**: if memory cannot be allocated you get the shared empty buffer
instead, on which `hb_buffer_allocation_successful` reports false.

**Ownership** — the caller owns the initial reference and must release it with
`hb_buffer_destroy`.

**Notes** — since HarfBuzz 0.9.2. Distinguishing genuine success from the
out-of-memory case requires
`hb_buffer_allocation_successful(buffer)` or a comparison against
`hb_buffer_get_empty()`.

#### `hb_buffer_create_similar`

```c
hb_buffer_t *hb_buffer_create_similar (const hb_buffer_t *src);
```

```rust
pub fn hb_buffer_create_similar(src: *const hb_buffer_t) -> *mut hb_buffer_t;
```

Creates a new buffer exactly as `hb_buffer_create` does, then copies `src`'s
*configuration* onto it.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `src` | The buffer to copy configuration from. The header does not annotate it as nullable and the implementation dereferences it, so treat null as forbidden. |

What is copied: the Unicode functions (a reference is taken), the flags, the
cluster level, the replacement code point, the invisible glyph, the not-found
glyph, and the not-found-variation-selector glyph. What is **not** copied: the
contents, the length, the content type, and the segment properties (direction,
script, language).

**Returns** — a new buffer with a reference count of one; never null.

**Ownership** — release with `hb_buffer_destroy`. `src` is unchanged apart from
the extra reference taken on its Unicode functions.

**Notes** — since HarfBuzz 3.3.0. This is the right way to spawn a worker
buffer that must behave identically to a configured one.

#### `hb_buffer_get_empty`

```c
hb_buffer_t *hb_buffer_get_empty (void);
```

```rust
pub fn hb_buffer_get_empty() -> *mut hb_buffer_t;
```

Fetches the singleton empty buffer — an inert, permanently valid, immutable
object that the error path of `hb_buffer_create` also returns.

**Returns** — the empty buffer; never null.

**Ownership** — upstream annotates the return as `transfer full`, so treat it
like any other buffer and pass it to `hb_buffer_destroy` when done. HarfBuzz's
shared null objects are inert, so referencing and destroying them are cheap
no-ops.

**Notes** — since HarfBuzz 0.9.2. Every mutator silently does nothing on it,
its length is always zero, its content type is
`HB_BUFFER_CONTENT_TYPE_INVALID`, and `hb_buffer_allocation_successful` returns
**false** for it. That last point is what makes it usable as an out-of-memory
sentinel.

#### `hb_buffer_reference`

```c
hb_buffer_t *hb_buffer_reference (hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_reference(buffer: *mut hb_buffer_t) -> *mut hb_buffer_t;
```

Increases the reference count on `buffer` by one and returns the same pointer,
which is convenient when handing a buffer to something that will take ownership.
This prevents the buffer from being destroyed until a matching
`hb_buffer_destroy`.

**Returns** — `buffer` itself.

**Ownership** — every call must be matched by exactly one `hb_buffer_destroy`.

**Notes** — since HarfBuzz 0.9.2. Reference counts are atomic in a normally
configured build.

#### `hb_buffer_destroy`

```c
void hb_buffer_destroy (hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_destroy(buffer: *mut hb_buffer_t);
```

Decreases the reference count on `buffer` by one. When it reaches zero the
buffer and all associated resources are freed: the reference on its Unicode
functions is dropped, the glyph-info and glyph-position arrays are freed, and
the message callback's destroy function is invoked on its user data.

**Returns** — nothing. There is no way to observe whether the object actually
went away.

**Notes** — since HarfBuzz 0.9.2. Tolerates the shared empty buffer.

### User data

#### `hb_buffer_set_user_data`

```c
hb_bool_t hb_buffer_set_user_data (hb_buffer_t        *buffer,
                                   hb_user_data_key_t *key,
                                   void *              data,
                                   hb_destroy_func_t   destroy,
                                   hb_bool_t           replace);
```

```rust
pub fn hb_buffer_set_user_data(
    buffer: *mut hb_buffer_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

Attaches a key/data pair to the buffer.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `buffer` | The buffer. Not annotated nullable. |
| `key` | The user-data key. HarfBuzz uses the **address** of `key`, not its contents, so the key object must outlive the buffer — a `static` is the usual choice. |
| `data` | The pointer to store. May be null. |
| `destroy` | Called with `data` when the buffer is destroyed or the entry is replaced. May be null (`None` in Rust). |
| `replace` | Whether to overwrite an existing entry stored under the same key. |

**Returns** — true on success, false otherwise (allocation failure, or a
non-`replace` call against an existing key).

**Ownership** — `data` is not copied; the destroy callback is how you learn it
is no longer needed.

**Notes** — since HarfBuzz 0.9.2. Silently fails on an immutable buffer, i.e.
on `hb_buffer_get_empty()`.

#### `hb_buffer_get_user_data`

```c
void *hb_buffer_get_user_data (const hb_buffer_t  *buffer,
                               hb_user_data_key_t *key);
```

```rust
pub fn hb_buffer_get_user_data(
    buffer: *const hb_buffer_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

Fetches the data previously attached under `key`.

**Returns** — the stored pointer, or null when no entry is present for that key.

**Ownership** — none is transferred: the pointer belongs to whoever stored it
and must not be freed by the caller.

**Notes** — since HarfBuzz 0.9.2. Note the `const` buffer parameter.

### Resetting, capacity, and allocation

#### `hb_buffer_reset`

```c
void hb_buffer_reset (hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_reset(buffer: *mut hb_buffer_t);
```

Resets the buffer to its initial status, as if it had just been created by
`hb_buffer_create`. Concretely it restores the default Unicode functions,
`HB_BUFFER_FLAG_DEFAULT`, `HB_BUFFER_CLUSTER_LEVEL_DEFAULT`, the default
replacement code point, invisible glyph 0, not-found glyph 0, and
not-found-variation-selector `HB_CODEPOINT_INVALID`, and then does everything
`hb_buffer_clear_contents` does.

**Notes** — since HarfBuzz 0.9.2. Silently does nothing on an immutable buffer.
The allocated arrays are kept, so resetting is cheap; only the logical state is
reset. It also clears the out-of-memory flag, so a buffer that failed an
allocation becomes usable again.

#### `hb_buffer_clear_contents`

```c
void hb_buffer_clear_contents (hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_clear_contents(buffer: *mut hb_buffer_t);
```

Similar to `hb_buffer_reset`, but does **not** clear the Unicode functions and
the replacement code point. In the current implementation it also preserves the
flags, the cluster level, the invisible glyph, the not-found glyph, and the
not-found-variation-selector glyph — everything `hb_buffer_reset` restores.

What it *does* clear: the contents (length becomes zero), the content type
(back to `HB_BUFFER_CONTENT_TYPE_INVALID`), **the segment properties**
(direction, script, and language all return to
`HB_SEGMENT_PROPERTIES_DEFAULT`), the pre- and post-context, the position array
(`hb_buffer_has_positions` becomes false), the out-of-memory flag (reset to
"successful"), and the random state (back to 1).

**Notes** — since HarfBuzz 0.9.11. Silently does nothing on an immutable
buffer. This is the function to call between shaping runs in a loop; the
allocated capacity survives.

#### `hb_buffer_pre_allocate`

```c
hb_bool_t hb_buffer_pre_allocate (hb_buffer_t  *buffer,
                                  unsigned int  size);
```

```rust
pub fn hb_buffer_pre_allocate(buffer: *mut hb_buffer_t, size: c_uint) -> hb_bool_t;
```

Pre-allocates memory so the buffer can hold at least `size` items without
reallocating. Does not change the length or the contents.

**Returns** — true if the allocation succeeded, false otherwise. A false return
also latches the buffer's out-of-memory flag, so
`hb_buffer_allocation_successful` will report false afterwards.

**Notes** — since HarfBuzz 0.9.2. Purely an optimisation; the `add_*` functions
grow the buffer on demand anyway.

#### `hb_buffer_allocation_successful`

```c
hb_bool_t hb_buffer_allocation_successful (hb_buffer_t  *buffer);
```

```rust
pub fn hb_buffer_allocation_successful(buffer: *mut hb_buffer_t) -> hb_bool_t;
```

Checks whether every memory allocation the buffer has attempted so far
succeeded.

**Returns** — true if all allocations succeeded, false if any failed.

**Notes** — since HarfBuzz 0.9.2. This is the *only* error channel the buffer
API has: `hb_buffer_create` and the `add_*` functions never report failure
directly. The flag is sticky until `hb_buffer_reset` or
`hb_buffer_clear_contents`. It is also false for `hb_buffer_get_empty()`, which
is what makes the out-of-memory return of `hb_buffer_create` detectable.

### Filling the buffer

#### `hb_buffer_add`

```c
void hb_buffer_add (hb_buffer_t    *buffer,
                    hb_codepoint_t  codepoint,
                    unsigned int    cluster);
```

```rust
pub fn hb_buffer_add(buffer: *mut hb_buffer_t, codepoint: hb_codepoint_t, cluster: c_uint);
```

Appends a single character with the Unicode value `codepoint` and the initial
cluster value `cluster`.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `codepoint` | A Unicode code point. **Not validated** — it is up to the caller to pass a valid one. |
| `cluster` | The cluster value. Clusters can be anything the client wants; they are usually the index of the character in the input text, and they come back out in `hb_glyph_info_t.cluster`. |

**Notes** — since HarfBuzz 0.9.7. This function **clears the post-context**, so
interleaving it with `hb_buffer_add_utf8` calls that established context will
discard that context. It does not itself set the content type, unlike the
`add_*` bulk functions.

#### `hb_buffer_add_utf8`

```c
void hb_buffer_add_utf8 (hb_buffer_t  *buffer,
                         const char   *text,
                         int           text_length,
                         unsigned int  item_offset,
                         int           item_length);
```

```rust
pub fn hb_buffer_add_utf8(
    buffer: *mut hb_buffer_t,
    text: *const c_char,
    text_length: c_int,
    item_offset: c_uint,
    item_length: c_int,
);
```

Appends UTF-8 text to the buffer. See `hb_buffer_add_codepoints` for the
`item_offset` / `item_length` contract, which is shared by the whole family.

**Parameters**

| Parameter | Meaning | Units |
| --- | --- | --- |
| `text` | The UTF-8 text. Not required to be NUL-terminated unless `text_length` is `-1`. | bytes |
| `text_length` | Length of `text`, or `-1` if it is NUL-terminated. | bytes |
| `item_offset` | Offset of the first character to add to the buffer. Clamped to `text_length`. | bytes |
| `item_length` | Number of characters to add, or `-1` for everything from `item_offset` to the end. Clamped to what remains. | bytes |

Invalid UTF-8 sequences are replaced with the buffer's replacement code point —
see `hb_buffer_set_replacement_codepoint`.

**Cluster values** are byte offsets into `text` (not into the sub-range), so
with `item_offset = 10` the first glyph's cluster is 10, not 0.

**Context** — if the buffer is empty and `item_offset > 0`, up to five code
points immediately before `item_offset` are installed as the pre-context. After
the items are added, up to five code points after `item_offset + item_length`
are installed as the post-context. The check is written so you can supply
pre-context in one call and the text in a follow-up call.

**Notes** — since HarfBuzz 0.9.2. Silently does nothing on an immutable buffer.
Requires the buffer to be empty with an invalid content type, or already
`HB_BUFFER_CONTENT_TYPE_UNICODE`; it sets the content type to Unicode when it
adds to an empty buffer. Bails out silently if the required capacity cannot be
allocated — check `hb_buffer_allocation_successful`.

#### `hb_buffer_add_utf16`

```c
void hb_buffer_add_utf16 (hb_buffer_t    *buffer,
                          const uint16_t *text,
                          int             text_length,
                          unsigned int    item_offset,
                          int             item_length);
```

```rust
pub fn hb_buffer_add_utf16(
    buffer: *mut hb_buffer_t,
    text: *const u16,
    text_length: c_int,
    item_offset: c_uint,
    item_length: c_int,
);
```

As `hb_buffer_add_utf8`, but the input is UTF-16 and all four length/offset
values are counted in **UTF-16 code units**, not bytes and not characters.
Invalid UTF-16 (an unpaired surrogate) is replaced with the buffer's replacement
code point. Cluster values are code-unit offsets into `text`.

**Notes** — since HarfBuzz 0.9.2.

#### `hb_buffer_add_utf32`

```c
void hb_buffer_add_utf32 (hb_buffer_t    *buffer,
                          const uint32_t *text,
                          int             text_length,
                          unsigned int    item_offset,
                          int             item_length);
```

```rust
pub fn hb_buffer_add_utf32(
    buffer: *mut hb_buffer_t,
    text: *const u32,
    text_length: c_int,
    item_offset: c_uint,
    item_length: c_int,
);
```

As `hb_buffer_add_utf8`, but the input is UTF-32 and the units are code points.
Values that are not valid Unicode scalar values are replaced with the buffer's
replacement code point — this is the sanity-checking counterpart of
`hb_buffer_add_codepoints`.

**Notes** — since HarfBuzz 0.9.2.

#### `hb_buffer_add_latin1`

```c
void hb_buffer_add_latin1 (hb_buffer_t   *buffer,
                           const uint8_t *text,
                           int            text_length,
                           unsigned int   item_offset,
                           int            item_length);
```

```rust
pub fn hb_buffer_add_latin1(
    buffer: *mut hb_buffer_t,
    text: *const u8,
    text_length: c_int,
    item_offset: c_uint,
    item_length: c_int,
);
```

As `hb_buffer_add_codepoints`, but allows access only to the first 256 Unicode
code points, which fit in 8-bit strings. Each byte becomes the code point of the
same value.

**Notes** — since HarfBuzz 0.9.39. The header carries an explicit warning: this
has nothing to do with the non-Unicode Latin-1 *encoding*. Because U+0000 to
U+00FF coincide with ISO-8859-1 it happens to be the same thing for that
encoding, but it is not a general transcoder.

#### `hb_buffer_add_codepoints`

```c
void hb_buffer_add_codepoints (hb_buffer_t          *buffer,
                               const hb_codepoint_t *text,
                               int                   text_length,
                               unsigned int          item_offset,
                               int                   item_length);
```

```rust
pub fn hb_buffer_add_codepoints(
    buffer: *mut hb_buffer_t,
    text: *const hb_codepoint_t,
    text_length: c_int,
    item_offset: c_uint,
    item_length: c_int,
);
```

Appends already-decoded Unicode code points. This is the function whose
documentation the rest of the family refers to.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `text` | Array of code points. Not validated — it is up to the caller to ensure they are valid Unicode scalar values. |
| `text_length` | Length of `text` in code points, or `-1` if NUL-terminated (a zero code point terminates). |
| `item_offset` | Index of the first code point that will be appended. |
| `item_length` | How many code points to append, or `-1` for the rest of `text`. |

**Why the offset/length window matters** — when shaping part of a larger text
(a run inside a paragraph, say), pass the *whole paragraph* as `text` and
delimit the run with `item_offset` and `item_length`, rather than passing only
the substring. That is what gives HarfBuzz the full context it needs for
cross-run Arabic shaping and for correctly handling combining marks at the start
of a run.

**Notes** — since HarfBuzz 0.9.31. `hb_buffer_add_utf32` takes the same kind of
input but sanity-checks it.

#### `hb_buffer_append`

```c
void hb_buffer_append (hb_buffer_t *buffer,
                       const hb_buffer_t *source,
                       unsigned int start,
                       unsigned int end);
```

```rust
pub fn hb_buffer_append(
    buffer: *mut hb_buffer_t,
    source: *const hb_buffer_t,
    start: c_uint,
    end: c_uint,
);
```

Appends part of the contents of another buffer to this one.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `source` | The buffer to copy from. Not modified. |
| `start` | Start index into `source`. Use `0` to copy from the beginning. |
| `end` | End index into `source`, exclusive. Use `UINT_MAX` (`c_uint::MAX`) to copy through the end. Clamped to `source`'s length; if `start > end` after clamping, nothing happens. |

**Preconditions** — asserted in the implementation, so violating them aborts in
a build with assertions enabled rather than failing gracefully: neither buffer
may be mid-shaping (have pending output), the two buffers must agree on whether
they have positions, and they must have the same content type. Both of the
latter two are waived when either buffer is empty.

**Side effects** — if `buffer` was empty it adopts `source`'s content type; if
`source` has positions and `buffer` does not, `buffer` gains a zeroed position
array; `hb_segment_properties_overlay` is applied, so `buffer` inherits any
direction/script/language it does not already have from `source`; and when
`source` holds Unicode content, `buffer`'s pre-context and post-context are
rebuilt from the text surrounding the copied range.

**Notes** — since HarfBuzz 1.5.0. On integer overflow of the resulting length,
or on allocation failure, the buffer's out-of-memory flag is set and nothing is
appended.

#### `hb_buffer_set_length`

```c
hb_bool_t hb_buffer_set_length (hb_buffer_t  *buffer,
                                unsigned int  length);
```

```rust
pub fn hb_buffer_set_length(buffer: *mut hb_buffer_t, length: c_uint) -> hb_bool_t;
```

Sets the number of items in the buffer. Similar to `hb_buffer_pre_allocate`, but
any new items added at the end are cleared to zero (both infos and, if the
buffer has them, positions).

**Returns** — true if the allocation succeeded, false otherwise. On an immutable
buffer it returns `length == 0`.

**Side effects** — `length == 0` also resets the content type to
`HB_BUFFER_CONTENT_TYPE_INVALID` and clears the pre-context; any call clears the
post-context.

**Notes** — since HarfBuzz 0.9.2. Growing the buffer this way gives you zeroed
glyph infos, which are not meaningful text; this function is mainly for clients
that fill the arrays themselves.

#### `hb_buffer_get_length`

```c
unsigned int hb_buffer_get_length (const hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_get_length(buffer: *const hb_buffer_t) -> c_uint;
```

Returns the number of items in the buffer — characters before shaping, glyphs
after. Valid only as long as the buffer is not modified.

**Notes** — since HarfBuzz 0.9.2.

### Segment properties

Every setter in this group silently does nothing on an immutable buffer, and
none of them validates its argument.

#### `hb_buffer_set_direction`

```c
void hb_buffer_set_direction (hb_buffer_t    *buffer,
                              hb_direction_t  direction);
```

```rust
pub fn hb_buffer_set_direction(buffer: *mut hb_buffer_t, direction: hb_direction_t);
```

Sets the text flow direction of the buffer.

**No shaping can happen without setting the direction.** It controls the visual
direction of the output glyphs — for an RTL direction the glyphs come back
reversed — and many layout features depend on it. Note in particular that
reversing RTL text before shaping and then shaping with LTR is *not* the same as
keeping the text in logical order and shaping with RTL.

**Notes** — since HarfBuzz 0.9.2. The value is stored verbatim, including
`HB_DIRECTION_INVALID`; `hb_buffer_guess_segment_properties` is what fills an
unset direction in.

#### `hb_buffer_get_direction`

```c
hb_direction_t hb_buffer_get_direction (const hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_get_direction(buffer: *const hb_buffer_t) -> hb_direction_t;
```

Fetches the text flow direction. Returns `HB_DIRECTION_INVALID` when unset.
Since HarfBuzz 0.9.2.

#### `hb_buffer_set_script`

```c
void hb_buffer_set_script (hb_buffer_t *buffer,
                           hb_script_t  script);
```

```rust
pub fn hb_buffer_set_script(buffer: *mut hb_buffer_t, script: hb_script_t);
```

Sets the script of the buffer.

Script is crucial for choosing the proper shaping behaviour for scripts that
require it (Arabic, for instance) and for deciding which OpenType features
defined in the font are applied. Pass one of the predefined `HB_SCRIPT_*`
values, or derive one with `hb_script_from_string` / `hb_script_from_iso15924_tag`.

**Notes** — since HarfBuzz 0.9.2.

#### `hb_buffer_get_script`

```c
hb_script_t hb_buffer_get_script (const hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_get_script(buffer: *const hb_buffer_t) -> hb_script_t;
```

Fetches the script. Returns `HB_SCRIPT_INVALID` when unset. Since HarfBuzz
0.9.2.

#### `hb_buffer_set_language`

```c
void hb_buffer_set_language (hb_buffer_t   *buffer,
                             hb_language_t  language);
```

```rust
pub fn hb_buffer_set_language(buffer: *mut hb_buffer_t, language: hb_language_t);
```

Sets the language of the buffer.

Languages select which OpenType features apply, which can produce
language-specific behaviour. They are orthogonal to scripts: related concepts,
but different, and not to be confused. Use `hb_language_from_string` to convert
a BCP 47 tag into an `hb_language_t`.

**Ownership** — `hb_language_t` values are interned for the lifetime of the
process, so nothing is copied and nothing needs freeing.

**Notes** — since HarfBuzz 0.9.2.

#### `hb_buffer_get_language`

```c
hb_language_t hb_buffer_get_language (const hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_get_language(buffer: *const hb_buffer_t) -> hb_language_t;
```

Fetches the language. Returns `HB_LANGUAGE_INVALID` (null) when unset. The
returned value must not be freed by the caller. Since HarfBuzz 0.9.2.

#### `hb_buffer_set_segment_properties`

```c
void hb_buffer_set_segment_properties (hb_buffer_t *buffer,
                                       const hb_segment_properties_t *props);
```

```rust
pub fn hb_buffer_set_segment_properties(
    buffer: *mut hb_buffer_t,
    props: *const hb_segment_properties_t,
);
```

Sets the buffer's direction, script, and language in one call — a shortcut for
calling the three setters individually.

**Ownership** — `props` is copied by value; the caller keeps ownership of the
struct and may free it immediately.

**Notes** — since HarfBuzz 0.9.7. The whole struct is copied, reserved fields
included, so build it from `HB_SEGMENT_PROPERTIES_DEFAULT` or from
`hb_buffer_get_segment_properties` rather than from uninitialised memory.

#### `hb_buffer_get_segment_properties`

```c
void hb_buffer_get_segment_properties (const hb_buffer_t *buffer,
                                       hb_segment_properties_t *props);
```

```rust
pub fn hb_buffer_get_segment_properties(
    buffer: *const hb_buffer_t,
    props: *mut hb_segment_properties_t,
);
```

Copies the buffer's segment properties into `*props`. `props` is an
out-parameter and must point at writable storage; it is fully overwritten, so it
need not be initialised. Since HarfBuzz 0.9.7.

#### `hb_buffer_guess_segment_properties`

```c
void hb_buffer_guess_segment_properties (hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_guess_segment_properties(buffer: *mut hb_buffer_t);
```

Sets *unset* segment properties from the buffer's Unicode contents. Properties
that are already set are left alone. If the buffer is not empty it must have
content type `HB_BUFFER_CONTENT_TYPE_UNICODE`.

The three steps, in order:

1. If the script is `HB_SCRIPT_INVALID`, it becomes the Unicode script of the
   first character in the buffer whose script is not `HB_SCRIPT_COMMON`,
   `HB_SCRIPT_INHERITED`, or `HB_SCRIPT_UNKNOWN`.
2. If the direction is `HB_DIRECTION_INVALID`, it becomes
   `hb_script_get_horizontal_direction` of the (possibly just-guessed) script;
   if that returns `HB_DIRECTION_INVALID`, `HB_DIRECTION_LTR` is used.
3. If the language is `HB_LANGUAGE_INVALID`, it becomes the process default from
   `hb_language_get_default`.

**Notes** — since HarfBuzz 0.9.7. `hb_language_get_default` is **not thread-safe
the first time it is called**, because it calls `setlocale`; call it once from a
single thread before any other thread can reach this function. Upstream notes
the language choice may change in the future to take the script into account.
Because step 1 looks at the buffer contents, add the text *before* calling this.

#### `hb_segment_properties_equal`

```c
hb_bool_t hb_segment_properties_equal (const hb_segment_properties_t *a,
                                       const hb_segment_properties_t *b);
```

```rust
pub fn hb_segment_properties_equal(
    a: *const hb_segment_properties_t,
    b: *const hb_segment_properties_t,
) -> hb_bool_t;
```

Checks the equality of two `hb_segment_properties_t`.

**Returns** — true if *all* properties of `a` equal those of `b`. The comparison
includes the two private reserved fields, so structs that were not zero-
initialised can compare unequal despite matching direction, script, and
language.

**Notes** — since HarfBuzz 0.9.7. Language comparison is a pointer comparison,
which is correct because languages are interned.

#### `hb_segment_properties_hash`

```c
unsigned int hb_segment_properties_hash (const hb_segment_properties_t *p);
```

```rust
pub fn hb_segment_properties_hash(p: *const hb_segment_properties_t) -> c_uint;
```

Creates a hash representing `p`, suitable for use as a map key. Computed as
`(direction * 31 + script) * 31 + (uintptr) language`, so it ignores the
reserved fields even though `hb_segment_properties_equal` does not.

**Notes** — since HarfBuzz 0.9.7. Not a stable hash across processes: it mixes
in an interned pointer value.

#### `hb_segment_properties_overlay`

```c
void hb_segment_properties_overlay (hb_segment_properties_t *p,
                                    const hb_segment_properties_t *src);
```

```rust
pub fn hb_segment_properties_overlay(
    p: *mut hb_segment_properties_t,
    src: *const hb_segment_properties_t,
);
```

Fills in missing fields of `p` from `src` in a considered manner:

1. If `p` has no direction, the direction is copied from `src`.
2. If `p` and `src` now have the same direction (which may be unset) and `p` has
   no script, the script is copied.
3. If `p` and `src` have the same direction and script (either may be unset) and
   `p` has no language, the language is copied.

The cascade stops as soon as a field disagrees, so a `p` whose direction differs
from `src`'s inherits nothing at all.

**Notes** — since HarfBuzz 3.3.0. Tolerates null for either argument (returns
immediately). This is the function `hb_buffer_append` uses internally.

### Content type and Unicode functions

#### `hb_buffer_set_content_type`

```c
void hb_buffer_set_content_type (hb_buffer_t              *buffer,
                                 hb_buffer_content_type_t  content_type);
```

```rust
pub fn hb_buffer_set_content_type(
    buffer: *mut hb_buffer_t,
    content_type: hb_buffer_content_type_t,
);
```

Sets the type of the buffer's contents. **You rarely need this**, because a
number of other functions transition the content type for you:

- A newly created buffer starts at `HB_BUFFER_CONTENT_TYPE_INVALID`.
  `hb_buffer_reset`, `hb_buffer_clear_contents`, and `hb_buffer_set_length` with
  an argument of zero all return it to invalid.
- `hb_buffer_add_utf8`, `hb_buffer_add_utf16`, `hb_buffer_add_utf32`,
  `hb_buffer_add_codepoints`, and `hb_buffer_add_latin1` expect the buffer to be
  either empty with content type invalid, or already
  `HB_BUFFER_CONTENT_TYPE_UNICODE`; they set the type to Unicode when they add
  to an empty buffer.
- `hb_shape` and `hb_shape_full` expect the same, and on success set the type to
  `HB_BUFFER_CONTENT_TYPE_GLYPHS`.

The transitions are designed so that a "reset : add-text : shape" loop never has
to touch the content type manually.

**Notes** — since HarfBuzz 0.9.5. Silently does nothing on an immutable buffer.
Legitimate uses are narrow: telling HarfBuzz that a hand-filled buffer contains
glyphs, or forcing a buffer's type before `hb_buffer_serialize`.

#### `hb_buffer_get_content_type`

```c
hb_buffer_content_type_t hb_buffer_get_content_type (const hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_get_content_type(buffer: *const hb_buffer_t) -> hb_buffer_content_type_t;
```

Fetches the type of the buffer's contents. Since HarfBuzz 0.9.5.

#### `hb_buffer_set_unicode_funcs`

```c
void hb_buffer_set_unicode_funcs (hb_buffer_t        *buffer,
                                  hb_unicode_funcs_t *unicode_funcs);
```

```rust
pub fn hb_buffer_set_unicode_funcs(
    buffer: *mut hb_buffer_t,
    unicode_funcs: *mut hb_unicode_funcs_t,
);
```

Sets the Unicode-functions structure the buffer uses to look up character
properties — general category, combining class, mirroring, composition, and
decomposition. See `unicode.md`.

**Ownership** — the buffer takes a reference on `unicode_funcs` and drops its
reference on the previous one. The caller keeps its own reference and must still
destroy it.

**Notes** — since HarfBuzz 0.9.2. Passing null installs the default Unicode
functions. Silently does nothing on an immutable buffer.

#### `hb_buffer_get_unicode_funcs`

```c
hb_unicode_funcs_t *hb_buffer_get_unicode_funcs (const hb_buffer_t  *buffer);
```

```rust
pub fn hb_buffer_get_unicode_funcs(buffer: *const hb_buffer_t) -> *mut hb_unicode_funcs_t;
```

Fetches the Unicode-functions structure attached to the buffer. No ownership is
transferred — the buffer keeps its reference, so take your own with
`hb_unicode_funcs_reference` if you intend to outlive the buffer. Since
HarfBuzz 0.9.2.

### Flags and cluster level

#### `hb_buffer_set_flags`

```c
void hb_buffer_set_flags (hb_buffer_t       *buffer,
                          hb_buffer_flags_t  flags);
```

```rust
pub fn hb_buffer_set_flags(buffer: *mut hb_buffer_t, flags: hb_buffer_flags_t);
```

Sets the buffer's flags, replacing the previous set wholesale — to add a flag,
OR it into the result of `hb_buffer_get_flags`. Since HarfBuzz 0.9.7. Silently
does nothing on an immutable buffer.

#### `hb_buffer_get_flags`

```c
hb_buffer_flags_t hb_buffer_get_flags (const hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_get_flags(buffer: *const hb_buffer_t) -> hb_buffer_flags_t;
```

Fetches the buffer's flags. Since HarfBuzz 0.9.7.

#### `hb_buffer_set_cluster_level`

```c
void hb_buffer_set_cluster_level (hb_buffer_t               *buffer,
                                  hb_buffer_cluster_level_t  cluster_level);
```

```rust
pub fn hb_buffer_set_cluster_level(
    buffer: *mut hb_buffer_t,
    cluster_level: hb_buffer_cluster_level_t,
);
```

Sets the cluster level, which controls how cluster values are grouped. See
`hb_buffer_cluster_level_t` for what each level means. Since HarfBuzz 0.9.42.
Silently does nothing on an immutable buffer. Set it before shaping; changing it
afterwards does not re-group anything.

#### `hb_buffer_get_cluster_level`

```c
hb_buffer_cluster_level_t hb_buffer_get_cluster_level (const hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_get_cluster_level(buffer: *const hb_buffer_t) -> hb_buffer_cluster_level_t;
```

Fetches the buffer's cluster level. Since HarfBuzz 0.9.42.

### Substitution code points and glyphs

#### `hb_buffer_set_replacement_codepoint`

```c
void hb_buffer_set_replacement_codepoint (hb_buffer_t    *buffer,
                                          hb_codepoint_t  replacement);
```

```rust
pub fn hb_buffer_set_replacement_codepoint(
    buffer: *mut hb_buffer_t,
    replacement: hb_codepoint_t,
);
```

Sets the code point that replaces invalid entries for a given encoding when
adding text to the buffer — used by `hb_buffer_add_utf8`, `_utf16`, and
`_utf32`. Defaults to `HB_BUFFER_REPLACEMENT_CODEPOINT_DEFAULT` (U+FFFD).

**Notes** — since HarfBuzz 0.9.31. Set it before adding text; it is consulted at
add time, not at shape time. Silently does nothing on an immutable buffer.
Preserved by `hb_buffer_clear_contents`, restored to the default by
`hb_buffer_reset`.

#### `hb_buffer_get_replacement_codepoint`

```c
hb_codepoint_t hb_buffer_get_replacement_codepoint (const hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_get_replacement_codepoint(buffer: *const hb_buffer_t) -> hb_codepoint_t;
```

Fetches the replacement code point. Since HarfBuzz 0.9.31.

#### `hb_buffer_set_invisible_glyph`

```c
void hb_buffer_set_invisible_glyph (hb_buffer_t    *buffer,
                                    hb_codepoint_t  invisible);
```

```rust
pub fn hb_buffer_set_invisible_glyph(buffer: *mut hb_buffer_t, invisible: hb_codepoint_t);
```

Sets the glyph that replaces invisible characters in the shaping result. If set
to zero (the default), the glyph for U+0020 SPACE is used; any other value is
used verbatim.

**Notes** — since HarfBuzz 2.0.0. This is the glyph substituted for hidden
`Default_Ignorable` characters, so it interacts with
`HB_BUFFER_FLAG_PRESERVE_DEFAULT_IGNORABLES` and
`HB_BUFFER_FLAG_REMOVE_DEFAULT_IGNORABLES`. Silently does nothing on an
immutable buffer.

#### `hb_buffer_get_invisible_glyph`

```c
hb_codepoint_t hb_buffer_get_invisible_glyph (const hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_get_invisible_glyph(buffer: *const hb_buffer_t) -> hb_codepoint_t;
```

Fetches the invisible glyph. Since HarfBuzz 2.0.0.

#### `hb_buffer_set_not_found_glyph`

```c
void hb_buffer_set_not_found_glyph (hb_buffer_t    *buffer,
                                    hb_codepoint_t  not_found);
```

```rust
pub fn hb_buffer_set_not_found_glyph(buffer: *mut hb_buffer_t, not_found: hb_codepoint_t);
```

Sets the glyph that replaces characters not found in the font during shaping.
The not-found glyph defaults to zero, sometimes known as the `.notdef` glyph;
setting it to something else lets you tell a genuine `.notdef` in the font apart
from a lookup failure.

**Notes** — since HarfBuzz 3.1.0. Silently does nothing on an immutable buffer.

#### `hb_buffer_get_not_found_glyph`

```c
hb_codepoint_t hb_buffer_get_not_found_glyph (const hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_get_not_found_glyph(buffer: *const hb_buffer_t) -> hb_codepoint_t;
```

Fetches the not-found glyph. Since HarfBuzz 3.1.0.

#### `hb_buffer_set_not_found_variation_selector_glyph`

```c
void hb_buffer_set_not_found_variation_selector_glyph (hb_buffer_t    *buffer,
                                                       hb_codepoint_t  not_found_variation_selector);
```

```rust
pub fn hb_buffer_set_not_found_variation_selector_glyph(
    buffer: *mut hb_buffer_t,
    not_found_variation_selector: hb_codepoint_t,
);
```

Sets the glyph that replaces variation-selector characters the font does not
resolve. The default is `HB_CODEPOINT_INVALID`, in which case an unresolved
variation selector is removed from the glyph string during shaping. Setting a
real glyph retains it instead, so the client can detect the situation and react
— by trying a different font, for instance.

**Notes** — since HarfBuzz 10.0.0. Unlike its siblings, the implementation of
this setter does **not** check the immutability flag, so it writes even to the
shared empty buffer.

#### `hb_buffer_get_not_found_variation_selector_glyph`

```c
hb_codepoint_t hb_buffer_get_not_found_variation_selector_glyph (const hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_get_not_found_variation_selector_glyph(
    buffer: *const hb_buffer_t,
) -> hb_codepoint_t;
```

Fetches the glyph used for an unresolved variation selector. Since HarfBuzz
10.0.0.

#### `hb_buffer_set_random_state`

```c
void hb_buffer_set_random_state (hb_buffer_t    *buffer,
                                 unsigned        state);
```

```rust
pub fn hb_buffer_set_random_state(buffer: *mut hb_buffer_t, state: c_uint);
```

Sets the buffer's random state, which changes every time a glyph uses randomness
— the OpenType `rand` feature, for example. Together with
`hb_buffer_get_random_state` this lets you transfer the current state to a
subsequent buffer for a better randomness distribution.

**Values** — defaults to 1, including after the buffer contents are cleared. A
value of **0 disables randomness** during shaping.

**Notes** — since HarfBuzz 8.4.0. Silently does nothing on an immutable buffer.

#### `hb_buffer_get_random_state`

```c
unsigned hb_buffer_get_random_state (const hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_get_random_state(buffer: *const hb_buffer_t) -> c_uint;
```

Fetches the buffer's random state. Since HarfBuzz 8.4.0.

### Reading results

#### `hb_buffer_get_glyph_infos`

```c
hb_glyph_info_t *hb_buffer_get_glyph_infos (hb_buffer_t  *buffer,
                                            unsigned int *length);
```

```rust
pub fn hb_buffer_get_glyph_infos(
    buffer: *mut hb_buffer_t,
    length: *mut c_uint,
) -> *mut hb_glyph_info_t;
```

Returns the buffer's glyph-information array.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `length` | Out-parameter receiving the array length. May be null if you do not want it. |

**Returns** — a pointer to the array. Upstream annotates it `array
length=length`; the pointer is non-null for a normal buffer, and for a
zero-length buffer the length is what tells you there is nothing to read.

**Ownership** — `transfer none`. The array belongs to the buffer and stays valid
only as long as the buffer contents are not modified — shaping, adding text,
reversing, `hb_buffer_set_length`, `hb_buffer_clear_contents`, and destruction
all invalidate it. Do not free it.

**Notes** — since HarfBuzz 0.9.2. The array is writable, and clients do
sometimes edit it in place; that is outside what the header documents.

#### `hb_buffer_get_glyph_positions`

```c
hb_glyph_position_t *hb_buffer_get_glyph_positions (hb_buffer_t  *buffer,
                                                    unsigned int *length);
```

```rust
pub fn hb_buffer_get_glyph_positions(
    buffer: *mut hb_buffer_t,
    length: *mut c_uint,
) -> *mut hb_glyph_position_t;
```

Returns the buffer's glyph-position array.

**Parameters** — `length` is an out-parameter receiving the array length; may be
null.

**Returns** — a pointer to the array, or **null** in one specific case: if the
buffer did not already have positions *and* this is called from inside a buffer
message callback (see `hb_buffer_set_message_func`). Otherwise, if the buffer
did not have positions, they are created and initialised to zeros.

**Ownership** — `transfer none`, same invalidation rules as
`hb_buffer_get_glyph_infos`.

**Notes** — since HarfBuzz 0.9.2. Calling this has the side effect of giving the
buffer a position array, which `hb_buffer_has_positions` will then report and
which `hb_buffer_append` checks for compatibility. Note also that `length` is
set *before* the null check, so it is filled in even when null is returned.

#### `hb_buffer_has_positions`

```c
hb_bool_t hb_buffer_has_positions (hb_buffer_t  *buffer);
```

```rust
pub fn hb_buffer_has_positions(buffer: *mut hb_buffer_t) -> hb_bool_t;
```

Returns whether the buffer has glyph position data. A buffer gains position data
when `hb_buffer_get_glyph_positions` is called on it (and during shaping), and
is cleared of position data by `hb_buffer_clear_contents`.

**Notes** — since HarfBuzz 2.7.3. Useful before `hb_buffer_append`, which
requires both buffers to agree, and before `hb_buffer_normalize_glyphs`, which
asserts that positions exist.

#### `hb_glyph_info_get_glyph_flags`

```c
hb_glyph_flags_t hb_glyph_info_get_glyph_flags (const hb_glyph_info_t *info);
#define hb_glyph_info_get_glyph_flags(info) \
        ((hb_glyph_flags_t) ((unsigned int) (info)->mask & HB_GLYPH_FLAG_DEFINED))
```

```rust
pub fn hb_glyph_info_get_glyph_flags(info: *const hb_glyph_info_t) -> hb_glyph_flags_t;
```

Returns the `hb_glyph_flags_t` encoded within an `hb_glyph_info_t` — that is,
`info->mask & HB_GLYPH_FLAG_DEFINED`.

**Notes** — since HarfBuzz 1.5.0. In C the name is *both* a real exported
function and a function-like macro, and the macro normally wins, so C callers
get an inline field read. Rust binds to the exported function; the two compute
exactly the same value. `mask` is otherwise private and must not be interpreted
by hand.

### Reordering

#### `hb_buffer_reverse`

```c
void hb_buffer_reverse (hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_reverse(buffer: *mut hb_buffer_t);
```

Reverses the buffer's contents — both the info array and, if present, the
position array. Since HarfBuzz 0.9.2.

#### `hb_buffer_reverse_range`

```c
void hb_buffer_reverse_range (hb_buffer_t *buffer,
                              unsigned int start, unsigned int end);
```

```rust
pub fn hb_buffer_reverse_range(buffer: *mut hb_buffer_t, start: c_uint, end: c_uint);
```

Reverses the buffer's contents in the range `start` (inclusive) to `end`
(exclusive). Since HarfBuzz 0.9.41.

#### `hb_buffer_reverse_clusters`

```c
void hb_buffer_reverse_clusters (hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_reverse_clusters(buffer: *mut hb_buffer_t);
```

Reverses the buffer's contents, then reverses each cluster again — where a
cluster is a run of consecutive items sharing a cluster number. The net effect
is to reverse the order of clusters while keeping the glyph order within each
cluster intact. Since HarfBuzz 0.9.2.

#### `hb_buffer_normalize_glyphs`

```c
void hb_buffer_normalize_glyphs (hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_normalize_glyphs(buffer: *mut hb_buffer_t);
```

Reorders a glyph buffer to have canonical in-cluster glyph order and position.
The resulting clusters behave identically to the pre-reordering ones; this is
about making two equivalent shaping results compare equal, which is why
`hb-shape`-style testing uses it.

**Preconditions** — asserts that the buffer has positions and that its content
type is `HB_BUFFER_CONTENT_TYPE_GLYPHS`. It reads the buffer direction to decide
whether clusters run forward or backward.

**Notes** — since HarfBuzz 0.9.2. The header carries an explicit warning: this
has **nothing to do with Unicode normalization**.

### Serialization

#### `hb_buffer_serialize_format_from_string`

```c
hb_buffer_serialize_format_t hb_buffer_serialize_format_from_string (const char *str, int len);
```

```rust
pub fn hb_buffer_serialize_format_from_string(
    str_: *const c_char,
    len: c_int,
) -> hb_buffer_serialize_format_t;
```

Parses a string such as `"text"` or `"json"` into an
`hb_buffer_serialize_format_t`. Pass `len` as `-1` when the string is
NUL-terminated.

**Returns** — the parsed format. It does **not** check whether the result is a
supported format: the implementation is `hb_tag_from_string(str, len) &
~0x20202020u`, i.e. a tag with the lowercase bits cleared, so any four-character
string produces some value. Use `hb_buffer_serialize_list_formats` to learn
which ones are real.

**Notes** — since HarfBuzz 0.9.7.

#### `hb_buffer_serialize_format_to_string`

```c
const char *hb_buffer_serialize_format_to_string (hb_buffer_serialize_format_t format);
```

```rust
pub fn hb_buffer_serialize_format_to_string(
    format: hb_buffer_serialize_format_t,
) -> *const c_char;
```

Converts a serialization format to its NUL-terminated name — `"text"` or
`"json"`.

**Returns** — a static string, or **null** when `format` is not a valid format.

**Ownership** — `transfer none`; the string is static and must not be freed.

**Notes** — since HarfBuzz 0.9.7.

#### `hb_buffer_serialize_list_formats`

```c
const char **hb_buffer_serialize_list_formats (void);
```

```rust
pub fn hb_buffer_serialize_list_formats() -> *mut *const c_char;
```

Returns the list of supported buffer serialization formats: a NULL-terminated
array of C strings, currently `{"text", "json", NULL}`.

**Ownership** — `transfer none`; a static array that must not be freed.

**Notes** — since HarfBuzz 0.9.7. The Rust signature is `*mut *const c_char`
rather than C's `const char **`; the array is still read-only in practice.

#### `hb_buffer_serialize_glyphs`

```c
unsigned int hb_buffer_serialize_glyphs (hb_buffer_t *buffer,
                                         unsigned int start,
                                         unsigned int end,
                                         char *buf,
                                         unsigned int buf_size,
                                         unsigned int *buf_consumed,
                                         hb_font_t *font,
                                         hb_buffer_serialize_format_t format,
                                         hb_buffer_serialize_flags_t flags);
```

```rust
pub fn hb_buffer_serialize_glyphs(
    buffer: *mut hb_buffer_t,
    start: c_uint,
    end: c_uint,
    buf: *mut c_char,
    buf_size: c_uint,
    buf_consumed: *mut c_uint,
    font: *mut hb_font_t,
    format: hb_buffer_serialize_format_t,
    flags: hb_buffer_serialize_flags_t,
) -> c_uint;
```

Serializes the buffer's glyph content into `buf` as text, which is useful for
showing the contents of a buffer during debugging or in tests.

**Parameters**

| Parameter | Meaning | Null allowed |
| --- | --- | --- |
| `start` | Index of the first item to serialize. Clamped to `end`. | — |
| `end` | Index one past the last item. Clamped to the buffer length. | — |
| `buf` | Output string. Set to `""` immediately when `buf_size` is non-zero. | no |
| `buf_size` | The size of `buf` in bytes. | — |
| `buf_consumed` | Out-parameter set to the number of bytes written into `buf`. | yes |
| `font` | The font the buffer was shaped with, needed to read glyph names and extents. | yes — an empty font is used |
| `format` | `HB_BUFFER_SERIALIZE_FORMAT_TEXT` or `_JSON`. | — |
| `flags` | Which properties to serialize. | — |

**Returns** — the number of **items** serialized, which is not the number of
bytes. Zero when `start == end`, when the format is invalid, or when nothing fit
in `buf`. The idiomatic loop is to call repeatedly with `start` advanced by the
return value until it reaches `end`, flushing `buf` each time.

**Preconditions** — asserts the buffer's content type is
`HB_BUFFER_CONTENT_TYPE_GLYPHS`. If the buffer has no positions,
`HB_BUFFER_SERIALIZE_FLAG_NO_POSITIONS` is forced on.

**Output format** — the `text` format looks like

```text
[uni0651=0@518,0+0|uni0628=0+1897]
```

- Serialized glyphs are delimited with `[` and `]`, and separated with `|`.
- Each glyph starts with its glyph name, or its glyph index if
  `HB_BUFFER_SERIALIZE_FLAG_NO_GLYPH_NAMES` is set. Then:
  - unless `NO_CLUSTERS` is set, `=` followed by `hb_glyph_info_t.cluster`;
  - unless `NO_POSITIONS` is set, the position: `@x_offset,y_offset` if the two
    offsets are not both zero, then `+x_advance`, then `,y_advance` if the
    y-advance is not zero;
  - if `GLYPH_EXTENTS` is set, `<x_bearing,y_bearing,width,height>`.

The `json` format looks like

```json
[{"g":"uni0651","cl":0,"dx":518,"dy":0,"ax":0,"ay":0},
 {"g":"uni0628","cl":0,"dx":0,"dy":0,"ax":1897,"ay":0}]
```

with `g` the glyph name or index, `cl` the cluster (unless `NO_CLUSTERS`),
`dx`/`dy`/`ax`/`ay` the x-offset, y-offset, x-advance, and y-advance (unless
`NO_POSITIONS`), and `xb`/`yb`/`w`/`h` the extents' `x_bearing`, `y_bearing`,
`width`, and `height` (only with `GLYPH_EXTENTS`).

**Notes** — since HarfBuzz 0.9.7.

#### `hb_buffer_serialize_unicode`

```c
unsigned int hb_buffer_serialize_unicode (hb_buffer_t *buffer,
                                          unsigned int start,
                                          unsigned int end,
                                          char *buf,
                                          unsigned int buf_size,
                                          unsigned int *buf_consumed,
                                          hb_buffer_serialize_format_t format,
                                          hb_buffer_serialize_flags_t flags);
```

```rust
pub fn hb_buffer_serialize_unicode(
    buffer: *mut hb_buffer_t,
    start: c_uint,
    end: c_uint,
    buf: *mut c_char,
    buf_size: c_uint,
    buf_consumed: *mut c_uint,
    format: hb_buffer_serialize_format_t,
    flags: hb_buffer_serialize_flags_t,
) -> c_uint;
```

Serializes the buffer's *Unicode* content — that is, before shaping. Same
parameters as `hb_buffer_serialize_glyphs` minus `font`, which is not needed
because there are no glyph names.

**Output format** — the `text` format looks like

```text
<U+0651=0|U+0628=1>
```

Items are separated with `|`, code points are zero-padded four-or-more-digit
hexadecimal preceded by `U+`, and unless `NO_CLUSTERS` is set the cluster
follows a `=`. The `json` format is a list of objects with `u` (the code point
as a decimal integer) and `cl` (the cluster, unless `NO_CLUSTERS`):

```json
[{"u":1617,"cl":0},{"u":1576,"cl":1}]
```

**Returns** — the number of items serialized.

**Preconditions** — asserts the content type is
`HB_BUFFER_CONTENT_TYPE_UNICODE`.

**Notes** — since HarfBuzz 2.7.3.

#### `hb_buffer_serialize`

```c
unsigned int hb_buffer_serialize (hb_buffer_t *buffer,
                                  unsigned int start,
                                  unsigned int end,
                                  char *buf,
                                  unsigned int buf_size,
                                  unsigned int *buf_consumed,
                                  hb_font_t *font,
                                  hb_buffer_serialize_format_t format,
                                  hb_buffer_serialize_flags_t flags);
```

```rust
pub fn hb_buffer_serialize(
    buffer: *mut hb_buffer_t,
    start: c_uint,
    end: c_uint,
    buf: *mut c_char,
    buf_size: c_uint,
    buf_consumed: *mut c_uint,
    font: *mut hb_font_t,
    format: hb_buffer_serialize_format_t,
    flags: hb_buffer_serialize_flags_t,
) -> c_uint;
```

Serializes whatever the buffer holds, dispatching on the content type:
`HB_BUFFER_CONTENT_TYPE_GLYPHS` goes to `hb_buffer_serialize_glyphs`,
`HB_BUFFER_CONTENT_TYPE_UNICODE` to `hb_buffer_serialize_unicode`, and an
invalid content type to an internal fallback that emits an empty
representation.

**Returns** — the number of items serialized.

**Notes** — since HarfBuzz 2.7.3. This is the one to use when you do not know,
or do not want to care, whether the buffer has been shaped yet. Because it never
asserts on content type, it is also the safe choice inside a debugger or a
logging helper.

#### `hb_buffer_deserialize_glyphs`

```c
hb_bool_t hb_buffer_deserialize_glyphs (hb_buffer_t *buffer,
                                        const char *buf,
                                        int buf_len,
                                        const char **end_ptr,
                                        hb_font_t *font,
                                        hb_buffer_serialize_format_t format);
```

```rust
pub fn hb_buffer_deserialize_glyphs(
    buffer: *mut hb_buffer_t,
    buf: *const c_char,
    buf_len: c_int,
    end_ptr: *mut *const c_char,
    font: *mut hb_font_t,
    format: hb_buffer_serialize_format_t,
) -> hb_bool_t;
```

Parses glyphs into `buffer` from the textual representation produced by
`hb_buffer_serialize_glyphs`. The items are **appended** to whatever the buffer
already holds.

**Parameters**

| Parameter | Meaning | Null allowed |
| --- | --- | --- |
| `buf` | The string to deserialize. | no |
| `buf_len` | Its length, or `-1` if NUL-terminated. | — |
| `end_ptr` | Out-parameter receiving a pointer to the character after the last one consumed. | yes |
| `font` | Font used to resolve glyph names to glyph IDs. | yes — an empty font is used, which means names cannot be resolved |
| `format` | The format of `buf`. | — |

**Returns** — true if the full string was parsed, false otherwise. False is also
returned for an empty `buf`, for an immutable buffer, and for an unsupported
`format`; in those cases `*end_ptr` is set to `buf`.

**Side effects** — asserts the buffer's content type is glyphs (an empty buffer
qualifies), then sets it to `HB_BUFFER_CONTENT_TYPE_GLYPHS`.

**Notes** — since HarfBuzz 0.9.7.

#### `hb_buffer_deserialize_unicode`

```c
hb_bool_t hb_buffer_deserialize_unicode (hb_buffer_t *buffer,
                                         const char *buf,
                                         int buf_len,
                                         const char **end_ptr,
                                         hb_buffer_serialize_format_t format);
```

```rust
pub fn hb_buffer_deserialize_unicode(
    buffer: *mut hb_buffer_t,
    buf: *const c_char,
    buf_len: c_int,
    end_ptr: *mut *const c_char,
    format: hb_buffer_serialize_format_t,
) -> hb_bool_t;
```

Parses Unicode text into `buffer` from the representation produced by
`hb_buffer_serialize_unicode`, in the same manner as
`hb_buffer_deserialize_glyphs` and with the same return convention. Asserts the
content type is Unicode and sets it to `HB_BUFFER_CONTENT_TYPE_UNICODE`.

**Notes** — since HarfBuzz 2.7.3.

### Comparison

#### `hb_buffer_diff`

```c
hb_buffer_diff_flags_t hb_buffer_diff (hb_buffer_t *buffer,
                                       hb_buffer_t *reference,
                                       hb_codepoint_t dottedcircle_glyph,
                                       unsigned int position_fuzz);
```

```rust
pub fn hb_buffer_diff(
    buffer: *mut hb_buffer_t,
    reference: *mut hb_buffer_t,
    dottedcircle_glyph: hb_codepoint_t,
    position_fuzz: c_uint,
) -> hb_buffer_diff_flags_t;
```

Compares the contents of two buffers and reports the kinds of difference found.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `buffer` | The buffer under test. |
| `reference` | The buffer to compare against. Only *this* one is scanned for `.notdef` and dotted circle. |
| `dottedcircle_glyph` | The glyph ID of U+25CC DOTTED CIRCLE, or `(hb_codepoint_t) -1` (`HB_CODEPOINT_INVALID`). |
| `position_fuzz` | The allowed absolute difference in position values before `HB_BUFFER_DIFF_FLAG_POSITION_MISMATCH` is reported. |

**Returns** — a bitwise OR of `hb_buffer_diff_flags_t`;
`HB_BUFFER_DIFF_FLAG_EQUAL` (zero) means no differences. Passing
`HB_CODEPOINT_INVALID` for `dottedcircle_glyph` suppresses
`HB_BUFFER_DIFF_FLAG_DOTTED_CIRCLE_PRESENT` and
`HB_BUFFER_DIFF_FLAG_NOTDEF_PRESENT` entirely, which is what most callers who
only want to compare two buffers should do.

Comparison proceeds in stages: a content-type mismatch (reported only when both
buffers are non-empty) is returned alone; a length mismatch skips the per-glyph
comparison but still scans the reference for the two special glyphs; equal
lengths are compared glyph by glyph with each differing aspect reported.

**Notes** — since HarfBuzz 1.5.0. Neither buffer is modified despite the
non-`const` parameters.

### Tracing

#### `hb_buffer_set_message_func`

```c
void hb_buffer_set_message_func (hb_buffer_t *buffer,
                                 hb_buffer_message_func_t func,
                                 void *user_data, hb_destroy_func_t destroy);
```

```rust
pub fn hb_buffer_set_message_func(
    buffer: *mut hb_buffer_t,
    func: hb_buffer_message_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Installs the `hb_buffer_message_func_t` implementation for this buffer. The
callback is then invoked at each step of shaping with a message describing the
step, and its return value decides whether the step runs.

**Parameters**

| Parameter | Meaning | Null allowed |
| --- | --- | --- |
| `func` | The callback. Passing null (`None`) clears any installed callback and its user data. | yes |
| `user_data` | Data passed to `func`. | yes |
| `destroy` | Called with `user_data` when it is no longer needed — when the buffer is destroyed, or when the callback is replaced. | yes |

**Ownership** — the callee assumes ownership of `user_data` unconditionally:
if the buffer is immutable, or if the call happens from *inside* a message
callback, the function calls `destroy(user_data)` immediately and returns
without installing anything. Any previously installed `destroy` is invoked
before the new callback is stored.

**Notes** — since HarfBuzz 1.1.3. Compiled out entirely when HarfBuzz is built
with `HB_NO_BUFFER_MESSAGE`, in which case the symbol does not exist. Inside the
callback, `hb_buffer_get_glyph_positions` may return null (see above).

#### `hb_buffer_changed`

```c
void hb_buffer_changed (hb_buffer_t *buffer);
```

```rust
pub fn hb_buffer_changed(buffer: *mut hb_buffer_t);
```

Called by a message callback *after* it has modified the buffer's glyph indices,
to update HarfBuzz's internal caches (the shaping digest that shaping uses to
skip lookups).

**Notes** — since HarfBuzz 13.0.0. Does nothing when called from outside a
message callback, so it is safe but useless elsewhere.

## Usage

### Shaping a string, in C

```c
#include <hb.h>

void shape_once (hb_font_t *font, const char *utf8)
{
  hb_buffer_t *buf = hb_buffer_create ();

  hb_buffer_add_utf8 (buf, utf8, -1, 0, -1);
  hb_buffer_guess_segment_properties (buf);   /* or set the three by hand */

  hb_shape (font, buf, NULL, 0);

  if (!hb_buffer_allocation_successful (buf)) {
    hb_buffer_destroy (buf);
    return;                                    /* out of memory */
  }

  unsigned int len;
  hb_glyph_info_t     *info = hb_buffer_get_glyph_infos (buf, &len);
  hb_glyph_position_t *pos  = hb_buffer_get_glyph_positions (buf, &len);

  int x = 0, y = 0;
  for (unsigned int i = 0; i < len; i++) {
    draw_glyph (font, info[i].codepoint, x + pos[i].x_offset, y + pos[i].y_offset);
    x += pos[i].x_advance;
    y += pos[i].y_advance;
  }

  hb_buffer_destroy (buf);
}
```

### Shaping a string, in Rust

```rust
use core::ffi::{c_char, c_uint};
use harfbuzz_sys::{
    hb_buffer_add_utf8, hb_buffer_allocation_successful, hb_buffer_create, hb_buffer_destroy,
    hb_buffer_get_glyph_infos, hb_buffer_get_glyph_positions,
    hb_buffer_guess_segment_properties, hb_font_t, hb_glyph_info_t, hb_glyph_position_t, hb_shape,
};

/// Shape `text` with `font` and return one (glyph id, cluster, x_advance) per glyph.
///
/// # Safety
/// `font` must be a live `hb_font_t`.
unsafe fn shape(font: *mut hb_font_t, text: &str) -> Option<alloc::vec::Vec<(u32, u32, i32)>> {
    // SAFETY: `hb_buffer_create` takes no arguments and never returns null.
    let buf = unsafe { hb_buffer_create() };

    // SAFETY: `buf` is live; `text` is valid UTF-8 of the given byte length, and
    // -1 for `item_length` means "to the end".
    unsafe {
        hb_buffer_add_utf8(
            buf,
            text.as_ptr() as *const c_char,
            text.len() as i32,
            0,
            -1,
        );
        hb_buffer_guess_segment_properties(buf);
        hb_shape(font, buf, core::ptr::null(), 0);
    }

    // SAFETY: `buf` is live. This is the only error channel the buffer API has.
    if unsafe { hb_buffer_allocation_successful(buf) } == 0 {
        // SAFETY: `buf` is live and owned by us.
        unsafe { hb_buffer_destroy(buf) };
        return None;
    }

    let mut len: c_uint = 0;
    // SAFETY: `buf` is live; both calls fill `len` with the same count, and the
    // arrays they return are valid until `buf` is modified or destroyed.
    let (infos, positions) = unsafe {
        let i = hb_buffer_get_glyph_infos(buf, &mut len);
        let p = hb_buffer_get_glyph_positions(buf, &mut len);
        (i, p)
    };

    let mut out = alloc::vec::Vec::with_capacity(len as usize);
    for k in 0..len as usize {
        // SAFETY: `k` is in bounds of both arrays, whose length is `len`.
        let (info, pos): (&hb_glyph_info_t, &hb_glyph_position_t) =
            unsafe { (&*infos.add(k), &*positions.add(k)) };
        out.push((info.codepoint, info.cluster, pos.x_advance));
    }

    // SAFETY: `buf` is live and owned by us; `out` no longer borrows from it.
    unsafe { hb_buffer_destroy(buf) };
    Some(out)
}
```

### Reusing one buffer across a paragraph

The whole point of `hb_buffer_clear_contents` is that the allocated arrays
survive it. Remember that it also clears direction, script, and language, so
re-establish them on every iteration.

```rust
use harfbuzz_sys::{
    HB_DIRECTION_LTR, HB_SCRIPT_LATIN, hb_buffer_add_utf8, hb_buffer_clear_contents,
    hb_buffer_set_direction, hb_buffer_set_script, hb_buffer_t, hb_font_t, hb_shape,
};

/// # Safety
/// `buf` and `font` must be live.
unsafe fn shape_runs(buf: *mut hb_buffer_t, font: *mut hb_font_t, runs: &[&str]) {
    for run in runs {
        // SAFETY: `buf` is live. Clearing resets contents *and* segment
        // properties, so both must be set again below.
        unsafe {
            hb_buffer_clear_contents(buf);
            hb_buffer_set_direction(buf, HB_DIRECTION_LTR);
            hb_buffer_set_script(buf, HB_SCRIPT_LATIN);
            hb_buffer_add_utf8(
                buf,
                run.as_ptr() as *const core::ffi::c_char,
                run.len() as i32,
                0,
                -1,
            );
            hb_shape(font, buf, core::ptr::null(), 0);
        }
        // ... read the results here, before the next iteration invalidates them.
    }
}
```

### Shaping a run with its surrounding context

Pass the whole paragraph and delimit the run, so the shaper sees the joining
context on both sides. Cluster values then come back as offsets into the
paragraph, which is usually what you want anyway.

```rust
use core::ffi::c_char;
use harfbuzz_sys::{hb_buffer_add_utf8, hb_buffer_t};

/// Add `paragraph[run]` to `buf`, keeping the rest of `paragraph` as context.
///
/// # Safety
/// `buf` must be live and empty; `run` must be a byte range of `paragraph`
/// that falls on character boundaries.
unsafe fn add_run(buf: *mut hb_buffer_t, paragraph: &str, run: core::ops::Range<usize>) {
    // SAFETY: the pointer covers `paragraph.len()` valid UTF-8 bytes, and the
    // offset/length window lies inside it. Text outside the window becomes up
    // to five code points of pre- and post-context.
    unsafe {
        hb_buffer_add_utf8(
            buf,
            paragraph.as_ptr() as *const c_char,
            paragraph.len() as i32,
            run.start as u32,
            (run.end - run.start) as i32,
        );
    }
}
```

### Serializing a buffer for debugging or tests

`hb_buffer_serialize` returns an item count, not a byte count, so the loop
advances `start` by the return value until the whole range is consumed.

```rust
use core::ffi::{c_char, c_uint};
use harfbuzz_sys::{
    HB_BUFFER_SERIALIZE_FLAG_DEFAULT, HB_BUFFER_SERIALIZE_FORMAT_TEXT, hb_buffer_get_length,
    hb_buffer_serialize, hb_buffer_t, hb_font_t,
};

/// # Safety
/// `buf` must be live; `font` may be null.
unsafe fn serialize(buf: *mut hb_buffer_t, font: *mut hb_font_t) -> alloc::string::String {
    // SAFETY: `buf` is live.
    let len = unsafe { hb_buffer_get_length(buf) };

    let mut out = alloc::string::String::new();
    let mut scratch = [0u8; 1024];
    let mut start: c_uint = 0;

    while start < len {
        let mut consumed: c_uint = 0;
        // SAFETY: `scratch` is `scratch.len()` writable bytes; `consumed`
        // receives the number of bytes actually written.
        let items = unsafe {
            hb_buffer_serialize(
                buf,
                start,
                len,
                scratch.as_mut_ptr() as *mut c_char,
                scratch.len() as c_uint,
                &mut consumed,
                font,
                HB_BUFFER_SERIALIZE_FORMAT_TEXT,
                HB_BUFFER_SERIALIZE_FLAG_DEFAULT,
            )
        };
        if items == 0 {
            break; // nothing fit, or the format was invalid
        }
        out.push_str(core::str::from_utf8(&scratch[..consumed as usize]).unwrap_or(""));
        start += items;
    }
    out
}
```

### Mapping glyphs back to characters

Clusters are the only supported way to relate output to input. With the default
cluster level and LTR text, cluster values are non-decreasing, so a run of
glyphs sharing a cluster is one indivisible unit for cursor placement and
selection:

```rust
/// Group `(glyph, cluster)` pairs into runs sharing a cluster value.
fn cluster_runs(glyphs: &[(u32, u32)]) -> alloc::vec::Vec<(u32, usize)> {
    let mut runs: alloc::vec::Vec<(u32, usize)> = alloc::vec::Vec::new();
    for &(_, cluster) in glyphs {
        match runs.last_mut() {
            Some((c, n)) if *c == cluster => *n += 1,
            _ => runs.push((cluster, 1)),
        }
    }
    runs
}
```

For RTL text the glyphs come back in visual order, so cluster values are
non-*increasing* instead. Under `HB_BUFFER_CLUSTER_LEVEL_CHARACTERS` and
`HB_BUFFER_CLUSTER_LEVEL_GRAPHEMES` no monotonicity is guaranteed at all, and
code that assumes sorted clusters will silently misbehave.

### Choosing a cluster level, in practice

- Text editing, cursor movement, hit testing, and anything that must not split a
  grapheme → `HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES` (the default).
- New code that wants precise character-to-glyph mapping while keeping sorted
  clusters → `HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS`, which upstream
  recommends for programs that do not need backward compatibility.
- Analysis tools that need the exact cluster of every character and can cope
  with unsorted output → `HB_BUFFER_CLUSTER_LEVEL_CHARACTERS`.
- Using HarfBuzz as a cheap grapheme segmenter → `HB_BUFFER_CLUSTER_LEVEL_GRAPHEMES`.

## Pitfalls

### Nothing returns null, and almost nothing reports failure

`hb_buffer_create` returns the shared empty buffer instead of null when it
cannot allocate, and the `add_*` functions return `void`. A buffer that ran out
of memory therefore looks exactly like a buffer with no text in it. The only
signal is `hb_buffer_allocation_successful`, and it is worth calling once after
filling and shaping. Note it returns **false** for `hb_buffer_get_empty()`, so
the same check covers the constructor's failure path.

### `hb_buffer_clear_contents` clears more than the name suggests

The header says only that it "does not clear the Unicode functions and the
replacement code point", which reads as though everything else is preserved.
What it actually clears includes the **segment properties**: after
`hb_buffer_clear_contents` the direction, script, and language are all unset
again. A reuse loop that sets them once outside the loop will shape the first
run correctly and every later run with guessed-or-invalid properties. Set them
inside the loop, or call `hb_buffer_guess_segment_properties` each time.

### The empty buffer swallows every mutation

`hb_buffer_get_empty()` returns an immutable object. Every setter, every `add_*`
function, `hb_buffer_reset`, and `hb_buffer_clear_contents` check the
immutability flag and return silently. If a buffer pointer somehow became the
empty buffer — the usual cause is an unchecked `hb_buffer_create` under memory
pressure — all subsequent work is silently discarded. The one documented
exception is `hb_buffer_set_not_found_variation_selector_glyph`, which does not
check the flag.

### Pointers from the getters are invalidated by any modification

The arrays returned by `hb_buffer_get_glyph_infos` and
`hb_buffer_get_glyph_positions` point straight into the buffer. Shaping again,
adding text, reversing, `hb_buffer_set_length`, `hb_buffer_clear_contents`,
`hb_buffer_reset`, and destruction all invalidate them, and growth reallocates
them. In Rust this is the classic FFI lifetime trap: a slice built with
`slice::from_raw_parts` has an unbounded lifetime and will happily outlive the
data. Copy what you need out before the next call.

### `item_offset` and `item_length` are in encoding units, not characters

For `hb_buffer_add_utf8` they are bytes; for `hb_buffer_add_utf16`, UTF-16 code
units; for `hb_buffer_add_utf32`, `hb_buffer_add_codepoints`, and
`hb_buffer_add_latin1`, code points. Cluster values come back in the same units
and are offsets into the whole `text` array, not into the sub-range — with
`item_offset = 10` the first cluster is 10. Getting this wrong produces
plausible-looking output that maps to the wrong characters.

### Passing only the substring loses shaping context

Slicing a paragraph and passing just the run to `hb_buffer_add_utf8` looks
equivalent to passing the paragraph with an offset, and is not. Arabic letters
join across run boundaries and combining marks need their base; the context
window (five code points on each side) is how HarfBuzz sees them. The pre-context
is only installed when the buffer is empty and `item_offset > 0`, so a second
`add_*` call into a non-empty buffer will not add pre-context. `hb_buffer_add`
clears the post-context outright.

### The content-type transitions are assertions, not errors

`hb_buffer_add_utf8` on a shaped buffer, `hb_buffer_serialize_glyphs` on a
Unicode buffer, and `hb_buffer_normalize_glyphs` without positions all hit
internal asserts rather than returning an error code. In a release build the
behaviour is undefined rather than diagnosed. Reset or clear the buffer between
roles, and prefer `hb_buffer_serialize`, which dispatches on the content type
instead of asserting.

### `hb_buffer_append` has assert-level preconditions too

The two buffers must agree on content type and on whether they have positions,
unless one of them is empty, and neither may be mid-shaping. Calling
`hb_buffer_get_glyph_positions` on one buffer and not the other is enough to
make them disagree.

### `hb_buffer_guess_segment_properties` is not thread-safe the first time

It calls `hb_language_get_default`, which calls `setlocale` on first use, and
`setlocale` is not thread-safe in many implementations. Call
`hb_language_get_default()` once from a single thread during start-up before any
worker thread can reach the guess function. Also remember that it only fills in
properties that are *unset*, and that it inspects the buffer contents — so add
the text first.

### Serialization returns items, not bytes

The `hb_buffer_serialize*` functions return the number of buffer items written,
while `buf_consumed` reports the byte count. Treating the return value as a byte
count truncates or overruns. A return of zero means nothing was written: an
empty range, an invalid format, or a `buf_size` too small for even one item —
the three are indistinguishable.

### `hb_buffer_serialize_format_from_string` accepts nonsense

It is `hb_tag_from_string` with the lowercase bits masked off, and performs no
validation. `hb_buffer_serialize_format_from_string("wibble", -1)` returns a
non-invalid-looking value that no serializer supports, and serializing with it
silently returns 0. Validate against `hb_buffer_serialize_list_formats` if the
string comes from a user.

### `mask` is private, and the flags are only in its low bits

Read glyph flags with `hb_glyph_info_get_glyph_flags`. The rest of `mask` holds
feature bits that mean nothing outside the shaper, and `var1`/`var2` are
scratch space whose contents after shaping are unspecified.

### `HB_GLYPH_FLAG_UNSAFE_TO_CONCAT` is off unless you ask for it

It is not produced unless `HB_BUFFER_FLAG_PRODUCE_UNSAFE_TO_CONCAT` was set on
the buffer during shaping, and the same is true of
`HB_GLYPH_FLAG_SAFE_TO_INSERT_TATWEEL` and
`HB_BUFFER_FLAG_PRODUCE_SAFE_TO_INSERT_TATWEEL`. An absent flag therefore does
not mean "safe"; it may mean "not computed".

### `hb_segment_properties_t` equality includes the private fields

`hb_segment_properties_equal` compares `reserved1` and `reserved2`, while
`hb_segment_properties_hash` ignores them. A struct built by hand without
zeroing those fields can hash equal but compare unequal — which breaks hash
tables keyed on segment properties. Start from `HB_SEGMENT_PROPERTIES_DEFAULT`
or from `hb_buffer_get_segment_properties`.

### Threading

The header is silent on thread safety. Reference counts are atomic in a normally
configured build, so `hb_buffer_reference` and `hb_buffer_destroy` may be called
from multiple threads, but a buffer is a mutable working object and everything
else — adding text, setting properties, shaping, reading the arrays — must be
confined to one thread at a time. The usual discipline is one buffer per thread,
created up front and reused. Note also the `hb_language_get_default` caveat
above.

### `hb_buffer_normalize_glyphs` is not Unicode normalization

Despite the name it reorders glyphs within clusters into a canonical order so
that two equivalent shaping results compare equal. It has nothing to do with
NFC, NFD, or any Unicode normalization form.
