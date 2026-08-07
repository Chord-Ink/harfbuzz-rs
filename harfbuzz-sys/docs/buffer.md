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
