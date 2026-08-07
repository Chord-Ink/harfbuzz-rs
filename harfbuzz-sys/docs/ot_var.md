# OpenType variations

Header: `hb-ot-var.h` — Rust module: `harfbuzz_sys::ot_var` (glob re-exported at
the crate root). Upstream gtk-doc section: `hb-ot-var`, "OpenType Font
Variations".

## Overview

A **variable font** is one font file that contains a continuous design space
rather than a fixed set of styles. The space is spanned by **axes** — weight,
width, optical size, slant, and any number of custom ones — each declared in the
font's `fvar` table with a tag, a minimum, a default, and a maximum. Picking a
value on every axis picks one **instance** out of that space. `hb-ot-var.h` is
the read-only introspection API for that machinery: it tells you which axes a
face has, what range each covers, which pre-baked **named instances** the
designer shipped, and how to convert between the two coordinate systems
HarfBuzz uses.

Everything in this header operates on an `hb_face_t`, because axes and named
instances are properties of the font *file*, not of any particular rendering of
it. Nothing here is created, referenced, or destroyed — there is no object with
a lifecycle, no allocation, and nothing for the caller to free. Every function
is a pure read of face data. The complementary half of the API lives in
`hb-font.h`: `hb_font_set_variations()`, `hb_font_set_var_coords_design()`,
`hb_font_set_var_coords_normalized()`, and `hb_font_set_var_named_instance()`
are what actually *move* a font through the design space. A common mistake is to
look for a setter here; there isn't one.

Two coordinate systems matter, and confusing them is the single biggest source
of bugs with this API:

* **Design coordinates** (also called user coordinates) are the numbers a human
  types: `wght = 700`, `wdth = 87.5`, `opsz = 14`. They are `float`, and their
  meaning and range are per-axis — you get the range from
  `hb_ot_var_axis_info_t`. This is the space of `hb_variation_t`,
  `hb_ot_var_named_instance_get_design_coords()`, and
  `hb_font_set_var_coords_design()`.
* **Normalized coordinates** are what the font's variation deltas are actually
  indexed by. Each axis is squashed so that its minimum maps to −1, its default
  maps to 0, and its maximum maps to +1, and the result is stored as a **2.14
  fixed-point `int`** — so `16384` means 1.0 and `-16384` means −1.0. This is
  the space of `hb_ot_var_normalize_coords()`,
  `hb_ot_var_normalize_variations()`, and
  `hb_font_set_var_coords_normalized()`.

The conversion is not a plain linear rescale. For each axis HarfBuzz clamps the
design value into `[min_value, max_value]`, then maps it piecewise-linearly
through the default: values below the default become
`(v - default) / (default - min)`, values above become
`(v - default) / (max - default)`, and the default itself becomes exactly 0.
Then, if the face has an `avar` table, its per-axis segment map is applied on
top, which lets a designer make the interpolation non-uniform (so that
"halfway between Regular and Bold" can be tuned to look halfway). An `avar`
version 2 table additionally applies a cross-axis remapping through an item
variation store, unless HarfBuzz was built with `HB_NO_AVAR2`. Only after all of
that is the value rounded to 2.14. This is why you should always normalize with
these functions rather than doing the arithmetic yourself.

**Named instances** are the fvar `InstanceRecord`s: positions in the design
space that the designer named, such as "Condensed Light Italic". Each has a
`name` table Name ID for its subfamily name, optionally a Name ID for its
PostScript name, and one design coordinate per axis. They are indexed **from
zero** by every function in this header. They are also the source of the entries
you see in a font menu, so most applications enumerate them rather than exposing
raw axes.

Upstream compiles this entire file out under `HB_NO_VAR`, a reduced-feature
build option — the header still declares the functions, so a program can compile
and then fail to link. The default configuration used by this crate's `build.rs`
includes it.

## Types

### `hb_ot_var_axis_flags_t`

```c
typedef enum { /*< flags >*/
  HB_OT_VAR_AXIS_FLAG_HIDDEN     = 0x00000001u,

  /*< private >*/
  _HB_OT_VAR_AXIS_FLAG_MAX_VALUE = HB_TAG_MAX_SIGNED /*< skip >*/
} hb_ot_var_axis_flags_t;
```

```rust
pub type hb_ot_var_axis_flags_t = core::ffi::c_int;
```

Flags describing one variation axis, stored in
[`hb_ot_var_axis_info_t::flags`](#hb_ot_var_axis_info_t). This is a **bit
field**: test with `&`, combine with `|`. HarfBuzz copies the raw 16-bit
`axisFlags` value straight out of the font's fvar `AxisRecord`, so bits that
have no constant here can and do appear — treat unknown bits as reserved rather
than assuming the value equals one of the constants.

| Constant | Value | Meaning |
| --- | --- | --- |
| `HB_OT_VAR_AXIS_FLAG_HIDDEN` | `0x00000001` | The axis should not be exposed directly in user interfaces. The designer marked it as an internal/parametric axis. |

The Rust transcription is a `c_int` alias plus constants rather than a Rust
`enum`, for the reason that applies throughout this crate: the value comes from
font data, and a Rust `enum` holding a value outside its variant list is
undefined behaviour. The underlying type is signed `int` because the C
enumeration's private sentinel is `HB_TAG_MAX_SIGNED` (`0x7FFFFFFF`), which fits
in an `int`.

Since HarfBuzz 2.2.0.

### `hb_ot_var_axis_info_t`

```c
typedef struct hb_ot_var_axis_info_t {
  unsigned int           axis_index;
  hb_tag_t               tag;
  hb_ot_name_id_t        name_id;
  hb_ot_var_axis_flags_t flags;
  float                  min_value;
  float                  default_value;
  float                  max_value;
  /*< private >*/
  unsigned int           reserved;
} hb_ot_var_axis_info_t;
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct hb_ot_var_axis_info_t {
    pub axis_index: c_uint,
    pub tag: hb_tag_t,
    pub name_id: hb_ot_name_id_t,
    pub flags: hb_ot_var_axis_flags_t,
    pub min_value: c_float,
    pub default_value: c_float,
    pub max_value: c_float,
    pub reserved: c_uint,
}
```

A plain value type describing one variation axis. **The caller allocates it** —
on the stack, in an array, wherever — and passes a pointer to
[`hb_ot_var_get_axis_infos`](#hb_ot_var_get_axis_infos) or
[`hb_ot_var_find_axis_info`](#hb_ot_var_find_axis_info), which fill it in. It
contains no pointers, owns nothing, and there is nothing to destroy. Copying it
is a plain memcpy; it stays valid after the face is destroyed.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `axis_index` | `unsigned int` | `c_uint` | Zero-based index of this axis in the face's axis array. This is the index into the coordinate arrays used by `hb_ot_var_normalize_coords()`, `hb_font_set_var_coords_design()`, and friends. |
| `tag` | `hb_tag_t` | `hb_tag_t` (`u32`) | Four-byte tag identifying the design variation, e.g. `wght`. Registered tags are in the [OpenType Design-Variation Axis Tag Registry](https://docs.microsoft.com/en-us/typography/opentype/spec/dvaraxisreg); the five most common have constants below. |
| `name_id` | `hb_ot_name_id_t` | `hb_ot_name_id_t` (`c_uint`) | `name` table Name ID for the axis's display name. Resolve it with `hb_ot_name_get_utf8()` / `hb_ot_name_get_utf16()` / `hb_ot_name_get_utf32()`. May be `HB_OT_NAME_ID_INVALID` (`0xFFFF`) or name an entry the `name` table does not actually contain. |
| `flags` | `hb_ot_var_axis_flags_t` | `hb_ot_var_axis_flags_t` (`c_int`) | Bit field; see above. Copied verbatim from the font. |
| `min_value` | `float` | `c_float` | Minimum design-space value the font covers on this axis. |
| `default_value` | `float` | `c_float` | Design-space value that corresponds to the font's default instance. Normalizes to exactly 0. |
| `max_value` | `float` | `c_float` | Maximum design-space value the font covers on this axis. |
| `reserved` | `unsigned int` | `c_uint` | **Private.** HarfBuzz sets it to 0 when filling the struct. Do not read it, do not rely on it, do not use it as padding of your own. It is `pub` in Rust only because the field must exist for the layout to match. |

The three value fields are in **un-normalized, user scale** — the header says so
explicitly. `min_value <= default_value <= max_value` holds for any font that
passes HarfBuzz's fvar sanitization.

The Rust struct derives `Debug, Clone, Copy` but not `PartialEq`/`Eq`/`Hash`,
because three of its fields are floats and the crate reserves those derives for
structs whose every field is integral.

Since HarfBuzz 2.2.0.

## Constants

The five registered axis tags that HarfBuzz spells out. They are ordinary
`hb_tag_t` values — there is nothing special about them beyond convenience, and
any other four-byte tag (`GRAD`, `XTRA`, a foundry's own) is equally valid
everywhere a tag is accepted.

| Constant | C definition | Value | Axis |
| --- | --- | --- | --- |
| `HB_OT_TAG_VAR_AXIS_ITALIC` | `HB_TAG('i','t','a','l')` | `0x6974616C` | Roman/italic. |
| `HB_OT_TAG_VAR_AXIS_OPTICAL_SIZE` | `HB_TAG('o','p','s','z')` | `0x6F70737A` | Optical size. Supersedes the OpenType `size` feature. |
| `HB_OT_TAG_VAR_AXIS_SLANT` | `HB_TAG('s','l','n','t')` | `0x736C6E74` | Slant. |
| `HB_OT_TAG_VAR_AXIS_WIDTH` | `HB_TAG('w','d','t','h')` | `0x77647468` | Width. |
| `HB_OT_TAG_VAR_AXIS_WEIGHT` | `HB_TAG('w','g','h','t')` | `0x77676874` | Weight. |

```rust
pub const HB_OT_TAG_VAR_AXIS_ITALIC: hb_tag_t = HB_TAG(b'i', b't', b'a', b'l');
pub const HB_OT_TAG_VAR_AXIS_OPTICAL_SIZE: hb_tag_t = HB_TAG(b'o', b'p', b's', b'z');
pub const HB_OT_TAG_VAR_AXIS_SLANT: hb_tag_t = HB_TAG(b's', b'l', b'n', b't');
pub const HB_OT_TAG_VAR_AXIS_WIDTH: hb_tag_t = HB_TAG(b'w', b'd', b't', b'h');
pub const HB_OT_TAG_VAR_AXIS_WEIGHT: hb_tag_t = HB_TAG(b'w', b'g', b'h', b't');
```

In C these are `#define`s expanding to `HB_TAG(...)`; in Rust they are `const`
items built with the `const fn` `HB_TAG`, so they can be used in patterns of
constants, array initializers, and other `const` contexts.

The header gives no `Since:` for these five macros. `HB_OT_TAG_VAR_AXIS_ITALIC`,
`_SLANT`, `_OPTICAL_SIZE`, `_WIDTH`, and `_WEIGHT` have been present since the
header was introduced in HarfBuzz 1.4.2.

Note that these are the same four tags as `HB_STYLE_TAG_ITALIC`,
`HB_STYLE_TAG_OPTICAL_SIZE`, `HB_STYLE_TAG_SLANT_ANGLE`, `HB_STYLE_TAG_WIDTH`,
and `HB_STYLE_TAG_WEIGHT` in `hb-style.h`, but the types differ:
`hb_style_tag_t` is a `c_int`, `hb_tag_t` is a `u32`. Cast when crossing over.

## Functions

Every function takes `hb_face_t *face` as a non-const pointer even though none
of them modify the face. That is HarfBuzz's house style, not a hint that the
face is mutated. In Rust they are all `*mut hb_face_t`.

The header documents nullability for none of the pointer parameters. Where the
implementation clearly tolerates null — the `_count` / array out-parameters — it
is called out below. `face` itself is dereferenced immediately in every
function; passing null is undefined. HarfBuzz's idiom is that failed creation
yields the empty object rather than `NULL`, and `hb_face_get_empty()` is
perfectly safe to pass here (it has no `fvar`, so you get "not variable"
answers).

### Face-level queries

#### `hb_ot_var_has_data`

```c
HB_EXTERN hb_bool_t
hb_ot_var_has_data (hb_face_t *face);
```

```rust
pub fn hb_ot_var_has_data(face: *mut hb_face_t) -> hb_bool_t;
```

Tests whether a face includes any OpenType variation data in the `fvar` table.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Must be non-null. Not modified. |

**Returns** — `true` (non-zero) if `fvar` data was found, `false` (0) otherwise.
Internally this checks that the face's sanitized `fvar` table has a non-zero
version, so a face whose `fvar` is missing, truncated, or malformed reports
`false`. There is no separate error channel: "no fvar" and "broken fvar" are
indistinguishable.

**Ownership** — borrows `face` for the duration of the call. Takes no reference,
returns no allocation.

**Notes** — Since HarfBuzz 1.4.2. This is the cheapest "is this a variable
font?" test. `hb_ot_var_get_axis_count() != 0` answers the same question and
also gives you the count, so if you need the count anyway, skip this call. The
first call on a face may lazily load and sanitize the `fvar` table; that
loading is internally synchronized and cached on the face, so concurrent calls
from several threads are safe.

#### `hb_ot_var_get_axis_count`

```c
HB_EXTERN unsigned int
hb_ot_var_get_axis_count (hb_face_t *face);
```

```rust
pub fn hb_ot_var_get_axis_count(face: *mut hb_face_t) -> c_uint;
```

Fetches the number of OpenType variation axes included in the face.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Must be non-null. Not modified. |

**Returns** — the number of variation axes defined, i.e. fvar's `axisCount`.
`0` for a non-variable face. This number is the required length of **every**
coordinate array in this API and in `hb-font.h`'s variation setters.

**Ownership** — borrows `face`. Nothing allocated.

**Notes** — Since HarfBuzz 1.4.2. Cheap and cached; no need to memoize it
yourself. Returns the same value as `hb_ot_var_get_axis_infos()` and
`hb_ot_var_named_instance_get_design_coords()` do.

### Axis enumeration

#### `hb_ot_var_get_axis_infos`

```c
HB_EXTERN unsigned int
hb_ot_var_get_axis_infos (hb_face_t             *face,
                          unsigned int           start_offset,
                          unsigned int          *axes_count /* IN/OUT */,
                          hb_ot_var_axis_info_t *axes_array /* OUT */);
```

```rust
pub fn hb_ot_var_get_axis_infos(
    face: *mut hb_face_t,
    start_offset: c_uint,
    axes_count: *mut c_uint,
    axes_array: *mut hb_ot_var_axis_info_t,
) -> c_uint;
```

Fetches a list of all variation axes in the face, beginning at the offset
provided.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Must be non-null. Not modified. |
| `start_offset` | Zero-based index of the first axis to return. A value greater than the axis count is not an error: nothing is written and `*axes_count` becomes 0. |
| `axes_count` | In/out, **nullable**. On input, the capacity of `axes_array` in elements. On output, how many elements were actually written — `min(capacity, axis_count - start_offset)`, possibly 0. |
| `axes_array` | Out, caller-allocated, **nullable**. Receives that many `hb_ot_var_axis_info_t` values. |

`axes_count` and `axes_array` are only honoured **together**. The implementation
is `if (axes_count && axes_array) { ... }`, so if either is null nothing is
written *and `*axes_count` is left completely untouched*. Passing a non-null
`axes_count` with a null `axes_array` does **not** give you the total in
`*axes_count`; use the return value for that.

**Returns** — the total number of variation axes in the face, regardless of
`start_offset`, of the capacity you passed, and of how many were written. This
is the same value `hb_ot_var_get_axis_count()` returns.

**Ownership** — borrows `face`; writes into caller-owned memory. The filled-in
structs contain no pointers and outlive the face.

**Notes** — Since HarfBuzz 2.2.0. Supersedes the deprecated
`hb_ot_var_get_axes()`. Each written entry has its `axis_index` set to its true
index in the face (`start_offset + i`), not to its position in your array — so
paginated reads still carry correct indices. `reserved` is zeroed.

#### `hb_ot_var_find_axis_info`

```c
HB_EXTERN hb_bool_t
hb_ot_var_find_axis_info (hb_face_t             *face,
                          hb_tag_t               axis_tag,
                          hb_ot_var_axis_info_t *axis_info);
```

```rust
pub fn hb_ot_var_find_axis_info(
    face: *mut hb_face_t,
    axis_tag: hb_tag_t,
    axis_info: *mut hb_ot_var_axis_info_t,
) -> hb_bool_t;
```

Fetches the variation-axis information corresponding to the specified axis tag
in the face.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Must be non-null. Not modified. |
| `axis_tag` | The tag of the axis to look for, e.g. `HB_OT_TAG_VAR_AXIS_WEIGHT`. Any four-byte tag is accepted; unknown tags simply are not found. |
| `axis_info` | Out, caller-allocated. Filled in **only if the function returns true**. Nullability is unspecified by the header; the implementation dereferences it on a hit, so pass a real pointer. |

**Returns** — `true` if an axis with that tag exists in the face, `false`
otherwise. **On `false`, `*axis_info` is not written at all** — it keeps
whatever garbage it had. Always branch on the return value before reading the
struct.

**Ownership** — borrows `face`; writes into caller-owned memory.

**Notes** — Since HarfBuzz 2.2.0. Supersedes the deprecated
`hb_ot_var_find_axis()`, which also reported the axis index through a separate
out-parameter; the index now arrives inside `axis_info.axis_index`. The lookup
is a **linear** scan of the axis array (fvar axes are stored in font order, not
sorted by tag), so resolving many tags on the same face is O(tags × axes) — for
more than two or three, enumerate once with `hb_ot_var_get_axis_infos()` and
build your own map.

### Named instances

#### `hb_ot_var_get_named_instance_count`

```c
HB_EXTERN unsigned int
hb_ot_var_get_named_instance_count (hb_face_t *face);
```

```rust
pub fn hb_ot_var_get_named_instance_count(face: *mut hb_face_t) -> c_uint;
```

Fetches the number of named instances included in the face.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Must be non-null. Not modified. |

**Returns** — the number of named instances defined, i.e. fvar's
`instanceCount`. `0` for a non-variable face, and also `0` for a variable face
whose designer shipped no instance records (legal, if unusual). Valid instance
indices are `0 ..= count - 1`.

**Ownership** — borrows `face`. Nothing allocated.

**Notes** — Since HarfBuzz 2.2.0.

#### `hb_ot_var_named_instance_get_subfamily_name_id`

```c
HB_EXTERN hb_ot_name_id_t
hb_ot_var_named_instance_get_subfamily_name_id (hb_face_t   *face,
                                                unsigned int instance_index);
```

```rust
pub fn hb_ot_var_named_instance_get_subfamily_name_id(
    face: *mut hb_face_t,
    instance_index: c_uint,
) -> hb_ot_name_id_t;
```

Fetches the `name` table Name ID that provides display names for the "Subfamily
name" defined for the given named instance — the "Condensed Light Italic" part
of the style menu entry.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Must be non-null. Not modified. |
| `instance_index` | Zero-based index of the instance, `< hb_ot_var_get_named_instance_count()`. Out-of-range is not an error. |

**Returns** — the Name ID found for the Subfamily name, or
`HB_OT_NAME_ID_INVALID` (`0xFFFF`, from `hb-ot-name.h`) if `instance_index` is
out of range. Note that a valid-looking ID is not a promise that the `name`
table actually has that entry; feed it to `hb_ot_name_get_utf8()` and check the
length you get back.

**Ownership** — borrows `face`; returns a plain integer.

**Notes** — Since HarfBuzz 2.2.0. This is the ID for the *subfamily* only. The
full display name is conventionally the family name (`name` ID 16 or 1) plus
this subfamily name.

#### `hb_ot_var_named_instance_get_postscript_name_id`

```c
HB_EXTERN hb_ot_name_id_t
hb_ot_var_named_instance_get_postscript_name_id (hb_face_t   *face,
                                                 unsigned int instance_index);
```

```rust
pub fn hb_ot_var_named_instance_get_postscript_name_id(
    face: *mut hb_face_t,
    instance_index: c_uint,
) -> hb_ot_name_id_t;
```

Fetches the `name` table Name ID that provides display names for the "PostScript
name" defined for the given named instance.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Must be non-null. Not modified. |
| `instance_index` | Zero-based index of the instance. Out-of-range is not an error. |

**Returns** — the Name ID found for the PostScript name, or
`HB_OT_NAME_ID_INVALID` (`0xFFFF`). You get the invalid value in **two** distinct
situations: the index is out of range, or — much more commonly — the font's
fvar `instanceSize` is too small to carry the optional `postScriptNameID` field,
which is legal and widespread. Do not treat `HB_OT_NAME_ID_INVALID` here as a
sign of a broken font.

**Ownership** — borrows `face`; returns a plain integer.

**Notes** — Since HarfBuzz 2.2.0.

#### `hb_ot_var_named_instance_get_design_coords`

```c
HB_EXTERN unsigned int
hb_ot_var_named_instance_get_design_coords (hb_face_t    *face,
                                            unsigned int  instance_index,
                                            unsigned int *coords_length, /* IN/OUT */
                                            float        *coords         /* OUT */);
```

```rust
pub fn hb_ot_var_named_instance_get_design_coords(
    face: *mut hb_face_t,
    instance_index: c_uint,
    coords_length: *mut c_uint,
    coords: *mut c_float,
) -> c_uint;
```

Fetches the design-space coordinates corresponding to the given named instance.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to inspect. Must be non-null. Not modified. |
| `instance_index` | Zero-based index of the instance. |
| `coords_length` | In/out, **nullable**. On input, the capacity of `coords`. On output, how many coordinates were written — `min(capacity, axis_count)` — or `0` if `instance_index` is out of range. |
| `coords` | Out, caller-allocated, **nullable**. Receives that many `float` design coordinates, in axis order (index *i* is the axis whose `axis_index` is *i*). |

Writing happens only when `coords_length` is non-null **and** `*coords_length`
is non-zero **and** `coords` is non-null. Unlike
`hb_ot_var_get_axis_infos()`, an out-of-range `instance_index` *does* write
`*coords_length = 0` when that pointer is non-null.

**Returns** — the number of variation axes in the face — i.e. the number of
coordinates this instance conceptually has — or **`0` if `instance_index` is out
of range**. A zero return therefore means either a bad index or a face with no
axes; combined with `*coords_length`, it is the only failure signal this
function offers.

**Ownership** — borrows `face`; writes into caller-owned memory. The floats are
copies.

**Notes** — Since HarfBuzz 2.2.0. The values are **design** coordinates in user
scale, converted from the font's 16.16 fixed point, so they are directly
comparable to `min_value` / `default_value` / `max_value` and directly usable
with `hb_font_set_var_coords_design()`. To apply an instance to a font, prefer
`hb_font_set_var_named_instance()`, which does this lookup for you.

### Coordinate conversion

#### `hb_ot_var_normalize_variations`

```c
HB_EXTERN void
hb_ot_var_normalize_variations (hb_face_t            *face,
                                const hb_variation_t *variations, /* IN */
                                unsigned int          variations_length,
                                int                  *coords, /* OUT */
                                unsigned int          coords_length);
```

```rust
pub fn hb_ot_var_normalize_variations(
    face: *mut hb_face_t,
    variations: *const hb_variation_t,
    variations_length: c_uint,
    coords: *mut c_int,
    coords_length: c_uint,
);
```

Normalizes all of the coordinates in the given list of variation axes: turns a
sparse, tag-keyed list of design values into the dense, index-keyed array of
normalized values that a font wants.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face whose axes define the mapping. Must be non-null. Not modified. |
| `variations` | Input array of `hb_variation_t` (`{ tag, value }`), read-only. May be null only if `variations_length` is 0. |
| `variations_length` | Number of entries in `variations`. `0` is legal and yields an all-default result. |
| `coords` | Out, caller-allocated, must have room for `coords_length` `int`s. Must be non-null if `coords_length` is non-zero. |
| `coords_length` | Length of `coords`. Should be `hb_ot_var_get_axis_count(face)`. |

**Returns** — nothing. There is no error channel; every failure mode is silent.

Semantics, in order:

1. `coords[0 .. coords_length]` is zeroed — so this is an **overwrite, not a
   merge**. Any axis you do not mention lands on its default (normalized 0).
2. For each variation, the axis with that tag is looked up
   (`hb_ot_var_find_axis_info()`, a linear scan). If the tag names no axis of
   this face, or the axis's index is `>= coords_length`, **the entry is silently
   ignored**.
3. Otherwise the value is clamped to the axis range, normalized piecewise-
   linearly about the default, and stored in 16.16 at `coords[axis_index]`.
4. The face's `avar` mapping is applied across the whole array.
5. Every entry is rounded from 16.16 down to **2.14**.

Later entries win over earlier ones for the same tag.

**Ownership** — borrows `face` and `variations`; writes into caller-owned
memory. Nothing is retained after the call returns.

**Notes** — Since HarfBuzz 1.4.2. This is exactly the conversion
`hb_font_set_variations()` performs internally, so if your goal is to configure
a font, call that instead. Use this function when you need the normalized array
itself — to cache it, to compare two settings, to hand to
`hb_font_set_var_coords_normalized()` on many fonts, or to key a shaping cache.

#### `hb_ot_var_normalize_coords`

```c
HB_EXTERN void
hb_ot_var_normalize_coords (hb_face_t   *face,
                            unsigned int coords_length,
                            const float *design_coords,     /* IN */
                            int         *normalized_coords  /* OUT */);
```

```rust
pub fn hb_ot_var_normalize_coords(
    face: *mut hb_face_t,
    coords_length: c_uint,
    design_coords: *const c_float,
    normalized_coords: *mut c_int,
);
```

Normalizes a dense array of design-space coordinates into normalized
coordinates. The minimum and maximum values for each axis are mapped to the
interval [−1, 1], with the default axis value mapped to 0. Any additional
scaling defined in the face's
[`avar` table](https://docs.microsoft.com/en-us/typography/opentype/spec/avar)
is also applied.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face whose axes define the mapping. Must be non-null. Not modified. |
| `coords_length` | Length of **both** arrays. Must equal `hb_ot_var_get_axis_count(face)`; see below. |
| `design_coords` | Input array of `coords_length` design values, positionally matched to axes (entry *i* is the axis with `axis_index == i`). Read-only. |
| `normalized_coords` | Out, caller-allocated, `coords_length` `int`s. May alias nothing else; the two arrays must not overlap. |

**Returns** — nothing, and no error channel.

Per the header: *"`coords_length` must be the same as the number of axes in the
face, as for example returned by `hb_ot_var_get_axis_count()`. Otherwise, the
behavior is undefined."* Concretely, a shorter array leaves the trailing axes at
whatever `hb_font_set_var_coords_normalized()` later defaults them to, and a
longer one indexes past the axis array. The current implementation happens to
bounds-check that index and normalize the extra entries to 0, but the header
declares this undefined — do not build on it.

The output has 14 bits of fixed-point sub-integer precision as the OpenType
specification requires: `16384` is 1.0, `-16384` is −1.0, `0` is the default.

**Ownership** — borrows `face` and `design_coords`; writes into caller-owned
memory.

**Notes** — Since HarfBuzz 1.4.2. Unlike
`hb_ot_var_normalize_variations()`, this function is **positional**: there are
no tags, so you must supply a value for every axis, in order. Get the order from
`hb_ot_var_get_axis_infos()`. Also unlike that function, it does not zero the
output first — it writes every entry unconditionally. Values outside an axis's
range are clamped, silently.

## Usage

### Is this face variable, and what are its axes?

The canonical two-call idiom: ask for the count with both out-parameters null,
allocate, then ask again.

```c
#include <hb.h>
#include <hb-ot.h>

unsigned int count = hb_ot_var_get_axis_infos (face, 0, NULL, NULL);
if (count == 0)
  return;  /* not a variable font */

hb_ot_var_axis_info_t *axes = calloc (count, sizeof (hb_ot_var_axis_info_t));
unsigned int written = count;
hb_ot_var_get_axis_infos (face, 0, &written, axes);

for (unsigned int i = 0; i < written; i++)
  printf ("%c%c%c%c%s  %g .. %g (default %g)\n",
          HB_UNTAG (axes[i].tag),
          (axes[i].flags & HB_OT_VAR_AXIS_FLAG_HIDDEN) ? " [hidden]" : "",
          (double) axes[i].min_value,
          (double) axes[i].max_value,
          (double) axes[i].default_value);

free (axes);
```

In Rust (the `-sys` crate itself is `no_std`; these examples are ordinary
client code and use `alloc`/`std`):

```rust
use core::ffi::c_uint;

use harfbuzz_sys::{
    HB_OT_VAR_AXIS_FLAG_HIDDEN, HB_UNTAG, hb_face_t, hb_ot_var_axis_info_t,
    hb_ot_var_get_axis_infos,
};

/// Reads every variation axis of `face`.
///
/// # Safety
/// `face` must be a valid, non-null face pointer.
unsafe fn axes(face: *mut hb_face_t) -> Vec<hb_ot_var_axis_info_t> {
    // Both out-parameters null: this call only reports the total.
    let total = unsafe {
        hb_ot_var_get_axis_infos(face, 0, core::ptr::null_mut(), core::ptr::null_mut())
    };
    if total == 0 {
        return Vec::new();
    }

    // All-zero is a valid value for this struct (integers and floats only),
    // so a zeroed buffer avoids any uninitialized-memory question.
    let mut buf: Vec<hb_ot_var_axis_info_t> = vec![unsafe { core::mem::zeroed() }; total as usize];
    let mut written: c_uint = total;
    unsafe { hb_ot_var_get_axis_infos(face, 0, &mut written, buf.as_mut_ptr()) };
    buf.truncate(written as usize);
    buf
}

for axis in unsafe { axes(face) } {
    let [a, b, c, d] = HB_UNTAG(axis.tag);
    let hidden = axis.flags & HB_OT_VAR_AXIS_FLAG_HIDDEN != 0;
    println!(
        "{}{}{}{}{}  {} .. {} (default {})",
        a as char, b as char, c as char, d as char,
        if hidden { " [hidden]" } else { "" },
        axis.min_value, axis.max_value, axis.default_value,
    );
}
```

### Look up one axis and clamp a user-supplied value

```c
hb_ot_var_axis_info_t info;
if (hb_ot_var_find_axis_info (face, HB_OT_TAG_VAR_AXIS_WEIGHT, &info))
  {
    float wanted = 900.f;
    if (wanted < info.min_value) wanted = info.min_value;
    if (wanted > info.max_value) wanted = info.max_value;
    /* `wanted` is now inside the font's actual weight range. */
  }
else
  {
    /* This font has no `wght` axis at all. `info` was NOT written. */
  }
```

```rust
use core::mem::MaybeUninit;

use harfbuzz_sys::{
    HB_OT_TAG_VAR_AXIS_WEIGHT, hb_ot_var_axis_info_t, hb_ot_var_find_axis_info,
};

let mut info = MaybeUninit::<hb_ot_var_axis_info_t>::uninit();
let found = unsafe { hb_ot_var_find_axis_info(face, HB_OT_TAG_VAR_AXIS_WEIGHT, info.as_mut_ptr()) };
if found != 0 {
    // Only now is it sound to assume_init.
    let info = unsafe { info.assume_init() };
    let wanted = 900.0_f32.clamp(info.min_value, info.max_value);
    let _ = wanted;
}
```

### List the named instances a font menu should show

```c
unsigned int n = hb_ot_var_get_named_instance_count (face);
unsigned int naxes = hb_ot_var_get_axis_count (face);
hb_language_t lang = hb_language_from_string ("en", -1);

for (unsigned int i = 0; i < n; i++)
  {
    char name[128];
    unsigned int name_len = sizeof name;
    hb_ot_name_id_t nid = hb_ot_var_named_instance_get_subfamily_name_id (face, i);
    hb_ot_name_get_utf8 (face, nid, lang, &name_len, name);   /* NUL-terminated */

    float *coords = calloc (naxes, sizeof (float));
    unsigned int got = naxes;
    hb_ot_var_named_instance_get_design_coords (face, i, &got, coords);

    printf ("%u: %s @", i, name);
    for (unsigned int j = 0; j < got; j++)
      printf (" %g", (double) coords[j]);
    printf ("\n");
    free (coords);
  }
```

Applying one of those instances to a font is a one-liner — you do not have to
copy the coordinates yourself:

```c
hb_font_t *font = hb_font_create (face);
hb_font_set_var_named_instance (font, 3);   /* zero-based, same index space */
```

### Normalize design coordinates yourself

Use this when you want the normalized array as data, not just as a side effect
on a font.

```c
unsigned int naxes = hb_ot_var_get_axis_count (face);

/* Sparse, by tag. */
hb_variation_t vars[] = {
  { HB_OT_TAG_VAR_AXIS_WEIGHT, 700.f },
  { HB_OT_TAG_VAR_AXIS_WIDTH,   87.5f },
};
int *norm = calloc (naxes, sizeof (int));
hb_ot_var_normalize_variations (face, vars, 2, norm, naxes);
/* norm[i] is 2.14: 16384 == 1.0, 0 == that axis's default. */

hb_font_set_var_coords_normalized (font, norm, naxes);
free (norm);
```

```c
/* Dense, positional — one float per axis, in axis order. */
float design[]  = { 700.f, 87.5f };            /* naxes == 2 for this face */
int   norm2[2];
hb_ot_var_normalize_coords (face, 2, design, norm2);
```

The Rust equivalent of the sparse form:

```rust
use harfbuzz_sys::{
    HB_OT_TAG_VAR_AXIS_WEIGHT, HB_OT_TAG_VAR_AXIS_WIDTH, hb_font_set_var_coords_normalized,
    hb_ot_var_get_axis_count, hb_ot_var_normalize_variations, hb_variation_t,
};

let naxes = unsafe { hb_ot_var_get_axis_count(face) };
let vars = [
    hb_variation_t { tag: HB_OT_TAG_VAR_AXIS_WEIGHT, value: 700.0 },
    hb_variation_t { tag: HB_OT_TAG_VAR_AXIS_WIDTH, value: 87.5 },
];

let mut norm = vec![0_i32; naxes as usize];
unsafe {
    hb_ot_var_normalize_variations(
        face,
        vars.as_ptr(),
        vars.len() as core::ffi::c_uint,
        norm.as_mut_ptr(),
        naxes,
    );
    hb_font_set_var_coords_normalized(font, norm.as_ptr(), naxes);
}
```

Converting a normalized value back to a human-readable fraction is just
`coord as f32 / 16384.0`.

### Building a tag → axis map for repeated lookups

`hb_ot_var_find_axis_info()` scans linearly, so resolving a dozen tags means a
dozen scans. Enumerate once instead:

```rust
use std::collections::HashMap;

use harfbuzz_sys::{hb_ot_var_axis_info_t, hb_tag_t};

let by_tag: HashMap<hb_tag_t, hb_ot_var_axis_info_t> =
    unsafe { axes(face) }.into_iter().map(|a| (a.tag, a)).collect();
```

## Pitfalls

**The two-call idiom needs *both* out-parameters null.**
`hb_ot_var_get_axis_infos()` only touches `*axes_count` when `axes_count` **and**
`axes_array` are both non-null. Calling it with `&count, NULL` hoping to learn
the total leaves your variable unchanged. The total is the **return value**, and
it is the total for the whole face — not affected by `start_offset` and not the
number of elements written. Read the written count out of `*axes_count`, not out
of the return.

**`hb_ot_var_find_axis_info()` leaves the struct untouched on failure.** It does
not zero it, does not set a sentinel tag, does nothing. Reading `axis_info`
without checking the return value reads uninitialized memory — which in Rust is
undefined behaviour, hence the `MaybeUninit` dance above. (The deprecated
`hb_ot_var_find_axis()` did write `HB_OT_VAR_NO_AXIS_INDEX` into its
`axis_index` out-parameter; the replacement dropped that behaviour along with
the parameter.)

**Design and normalized coordinates are not interchangeable.** `int` arrays are
always 2.14 normalized; `float` arrays are always design values.
`hb_font_set_var_coords_design()` takes floats,
`hb_font_set_var_coords_normalized()` takes ints, and the compiler will not
catch a swap on the Rust side either, since both are raw pointers. A normalized
coordinate of `700` is not "weight 700" — it is 0.0427, essentially the default.

**`hb_ot_var_normalize_variations()` overwrites, it does not merge.** It zeroes
the whole output array first. To adjust one axis of an existing setting, keep
the full `hb_variation_t` list around and re-normalize the whole thing; do not
call it twice into the same buffer expecting the results to accumulate.

**Unknown axis tags fail silently.** A variation naming an axis the face does
not have is dropped without any diagnostic, and so is one whose axis index is
beyond `coords_length`. If you need to know whether a user's `wght=700` request
was actually honoured, check with `hb_ot_var_find_axis_info()` first.

**Out-of-range values are clamped, not rejected.** Asking for `wght = 5000` on
an axis that stops at 900 gets you 900, with no indication that anything was
adjusted. Clamp against `min_value` / `max_value` yourself if you want to tell
the user.

**`coords_length` must match the axis count exactly** for
`hb_ot_var_normalize_coords()` — the header calls anything else undefined. Get
the number from `hb_ot_var_get_axis_count()` for the same face; do not assume it
from a previous face, and re-check it if the face can change.

**Two different "invalid" answers from the named-instance getters.**
`hb_ot_var_named_instance_get_subfamily_name_id()` and
`..._get_postscript_name_id()` both return `HB_OT_NAME_ID_INVALID` (`0xFFFF`)
for an out-of-range index, and the PostScript one *also* returns it for
perfectly good instances in fonts whose fvar records simply omit the optional
PostScript name field. You cannot distinguish the two cases from the return
value alone — validate the index against
`hb_ot_var_get_named_instance_count()` first. Meanwhile
`hb_ot_var_named_instance_get_design_coords()` signals a bad index by returning
**0** and, if you passed a non-null `coords_length`, setting `*coords_length` to
0.

**A Name ID is not a name.** The axis `name_id` and both instance name-ID
accessors hand back an index into the `name` table. The table may not contain
that ID at all, or may not have it in the language you asked for.
`hb_ot_name_get_utf8()` returns the full length of the requested string, or **0
if it was not found** — check that before using the buffer, and be ready to
synthesize a name from the axis tag instead. Passing `HB_LANGUAGE_INVALID` as
the language means "assume English", not "any language".

**Named-instance indices are zero-based *here* but one-based in the face
index.** This API, and `hb_font_set_var_named_instance()`, count instances from
0. The named-instance selector encoded in the top 16 bits of the `index`
argument to `hb_face_create()` counts from 1, because 0 there means "no
instance" — `hb_font_create()` subtracts one before forwarding it. Mixing the
two conventions is an easy off-by-one.

**`HB_OT_VAR_AXIS_FLAG_HIDDEN` is advisory.** It means "do not put a slider for
this in the UI", not "this axis cannot be set". Hidden axes still count towards
`hb_ot_var_get_axis_count()`, still occupy a slot in every coordinate array, and
still work when you set them.

**`flags` may carry bits you do not know.** HarfBuzz copies fvar's `axisFlags`
through verbatim. Test individual bits; never compare `flags` for equality
against `HB_OT_VAR_AXIS_FLAG_HIDDEN`.

**Variations live on the font, not the face.** Nothing in this header changes
anything. Two `hb_font_t`s made from one face can sit at different points in the
design space. If you cache anything derived from a font's variation settings,
invalidate it whenever `hb_font_set_variations()`,
`hb_font_set_variation()`, `hb_font_set_var_coords_design()`,
`hb_font_set_var_coords_normalized()`, or `hb_font_set_var_named_instance()` is
called; there is no change notification. Setters on an immutable font are
silently ignored.

**`HB_NO_VAR` builds have no symbols at all here.** The whole implementation
file is inside `#ifndef HB_NO_VAR`, but the header declares the functions
unconditionally, so a reduced-feature build fails at link time rather than at
compile time. This crate's default build includes variation support.

**Thread safety.** All ten functions are read-only with respect to the face and
are safe to call concurrently from multiple threads on the same face. The first
call may lazily load and sanitize `fvar` or `avar`; that caching is internally
synchronized. As always, do not race these against code that is still mutating
the face.

**Rust-specific reminders.**

- Every function is `unsafe` and takes `*mut hb_face_t` even though it only
  reads. This crate adds no null checks.
- `hb_ot_var_axis_info_t` has no `Default` and derives no `PartialEq` — three of
  its fields are floats. Compare fields explicitly, and remember the private
  `reserved` field when you construct one by hand (you normally shouldn't; let
  HarfBuzz fill it in).
- `hb_ot_var_axis_flags_t` is `c_int` while `hb_tag_t` is `u32`. The axis-tag
  constants in this module are `hb_tag_t`, so they need no cast for the
  functions here, but they do need `as hb_style_tag_t` if you pass them to
  `hb_style_get_value()`.
- Passing `&mut count` where C wants `unsigned int *` is fine; passing
  `core::ptr::null_mut()` is how you spell C's `NULL` for both out-parameters.

## Related, but not declared in this header

These symbols carry `hb_ot_var` names but live in `hb-ot-deprecated.h` (gtk-doc
section `hb-ot-deprecated`), and are transcribed in this crate's `ot_deprecated`
module:

| Symbol | Status |
| --- | --- |
| `hb_ot_var_axis_t` | Deprecated in 2.2.0. Use [`hb_ot_var_axis_info_t`](#hb_ot_var_axis_info_t). Lacks `axis_index` and `flags`. |
| `hb_ot_var_get_axes()` | Deprecated in 2.2.0. Use [`hb_ot_var_get_axis_infos`](#hb_ot_var_get_axis_infos). |
| `hb_ot_var_find_axis()` | Deprecated in 2.2.0. Use [`hb_ot_var_find_axis_info`](#hb_ot_var_find_axis_info). |
| `HB_OT_VAR_NO_AXIS_INDEX` | Deprecated in 2.2.0 (`0xFFFFFFFFu`). Only ever written by `hb_ot_var_find_axis()`. |

And the variation setters this header deliberately does not provide, all in
`hb-font.h`: `hb_font_set_variations()`, `hb_font_set_variation()`,
`hb_font_set_var_coords_design()`, `hb_font_get_var_coords_design()`,
`hb_font_set_var_coords_normalized()`, `hb_font_get_var_coords_normalized()`,
`hb_font_set_var_named_instance()`, `hb_font_get_var_named_instance()`, and
`HB_FONT_NO_VAR_NAMED_INSTANCE`. `hb_ot_name_get_utf8()` and
`HB_OT_NAME_ID_INVALID` are in `hb-ot-name.h`; `hb_variation_t` and
`hb_variation_from_string()` are in `hb-common.h`.
