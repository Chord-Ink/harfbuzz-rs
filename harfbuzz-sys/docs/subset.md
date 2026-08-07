# Subsetting

Headers: `hb-subset.h`, `hb-subset-serialize.h`, `hb-subset-depend.h` — Rust
module: `harfbuzz_sys::subset`. The module is gated on the crate's `subset`
Cargo feature and is **not** glob re-exported at the crate root, so its items are
reached as `harfbuzz_sys::subset::*`. A handful of items additionally need the
`experimental` feature; each is marked below.

## Overview

Subsetting reduces the codepoint coverage of a font file and removes all data
that is no longer needed. You describe the subset you want in an
`hb_subset_input_t`, hand it to `hb_subset_or_fail()` together with a source
`hb_face_t`, and get back a new face whose blob is the subsetted font file.

The pipeline has three stages, and the API exposes all three:

```
hb_subset_input_t  ──create_or_fail──▶  hb_subset_plan_t  ──execute_or_fail──▶  hb_face_t
   (what you want)                       (resolved against                       (the subset;
                                          a specific face)                        reference_blob
                                                                                  gives the bytes)
```

`hb_subset_or_fail()` is the one-call shorthand for "create plan, execute,
destroy plan". Splitting it in two is worth doing when you need the plan's glyph
mappings — which glyph IDs survived and what they became — either before or
after the font is produced.

**What is supported.** Most outline and bitmap tables: `glyf`, `CFF `, `CFF2`,
`sbix`, `COLR`, and `CBDT`/`CBLC`, including variable outlines via OpenType
variations. Notably `EBDT`/`EBLC` and `SVG ` are **not** supported. Layout
subsetting covers the OpenType Layout tables (`GSUB`, `GPOS`, `GDEF`) only;
Graphite and AAT tables are not subsetted. A font carrying Graphite or AAT tables
can still be run through the subsetter, but upstream's advice is to enable
retain-gids and configure those layout tables to pass through untouched — which
is why the default configuration simply drops them.

**Instancing is the same machinery.** Pinning a variation axis to a fixed value
with `hb_subset_input_pin_axis_location()`, or narrowing it with
`hb_subset_input_set_axis_range()`, produces a font with a smaller — possibly
empty — variation space. Pin every axis and the output is a static instance.

**Three independent tools live in these headers**, and it helps to keep them
apart:

1. The subsetter proper (`hb-subset.h`).
2. The **object-graph repacker** (`hb-subset-serialize.h`) — HarfBuzz's
   offset-overflow resolver, exposed standalone via
   `hb_subset_serialize_or_fail()` so that a caller who assembles an OpenType
   table themselves can hand HarfBuzz a graph of objects and get back packed
   bytes with all 16-bit offsets made to fit.
3. The **glyph dependency graph** (`hb-subset-depend.h`) — a static analysis of
   which glyphs pull in which other glyphs through `GSUB`, composite `glyf`
   outlines, `CFF` seacs, `COLR` layers, and `MATH` variants. Highly
   experimental.

### The default subset input is not empty

A freshly created `hb_subset_input_t` is *not* a blank slate. Its constructor
pre-populates several sets with what a typical web-font subsetter wants, and
knowing those defaults is the difference between "my subset lost all its layout"
and a correct call.

| Set | Default contents |
| --- | --- |
| Unicodes (`HB_SUBSET_SETS_UNICODE`) | empty — you must add what you want |
| Glyph indices (`HB_SUBSET_SETS_GLYPH_INDEX`) | empty |
| Name IDs (`HB_SUBSET_SETS_NAME_ID`) | 0 through 6 inclusive (copyright, family, subfamily, unique ID, full name, version, PostScript name) |
| Name language IDs (`HB_SUBSET_SETS_NAME_LANG_ID`) | `0x0409` (US English) only |
| Drop tables (`HB_SUBSET_SETS_DROP_TABLE_TAG`) | `morx`, `mort`, `kerx`, `kern`, `JSTF`, `DSIG`, `EBDT`, `EBLC`, `EBSC`, `SVG `, `PCLT`, `LTSH`, and the Graphite tables `Feat`, `Glat`, `Gloc`, `Silf`, `Sill` |
| No-subset tables (`HB_SUBSET_SETS_NO_SUBSET_TABLE_TAG`) | `gasp`, `fpgm`, `prep`, `VDMX`, `DSIG` — passed through byte-for-byte |
| Layout features (`HB_SUBSET_SETS_LAYOUT_FEATURE_TAG`) | a curated list of ~70 tags copied from fontTools' `_layout_features_groups`: `rvrn ccmp liga locl mark mkmk rlig frac numr dnom calt clig curs kern rclt valt vert vkrn vpal vrt2 ltra ltrm rtla rtlm rand jalt chws vchw halt vhal palt Harf HARF Buzz BUZZ init medi fina isol med2 fin2 fin3 cswh mset stch ljmo vjmo tjmo abvs blws abvm blwm nukt akhn rphf rkrf pref blwf half abvf pstf cfar vatu cjct pres psts haln dist` |
| Layout scripts (`HB_SUBSET_SETS_LAYOUT_SCRIPT_TAG`) | **inverted** — all scripts |
| Flags | `HB_SUBSET_FLAGS_DEFAULT` (everything off) |
| Old-to-new glyph map | empty (automatic assignment) |
| Axis pins/ranges | none |

`hb_subset_input_keep_everything()` is the escape hatch: it inverts the six
"retain" sets, clears the drop-table set, and turns on the five flags that mean
"do not throw information away". Use it when you want to *remove* a few specific
things rather than *select* a few specific things.

### Building

The `subset` Cargo feature adds HarfBuzz's subsetting sources — including
`src/graph/`, the repacker — to the amalgamation and defines `HB_HAS_SUBSET`.
No system library is needed.

The `experimental` feature defines `HB_EXPERIMENTAL_API` upstream. That matters
twice over:

- It enables the `HB_SUBSET_FLAGS_IFTB_REQUIREMENTS` and
  `HB_SUBSET_FLAGS_RETAIN_NUM_GLYPHS` flag values, the
  `hb_subset_input_to_string_or_fail` / `hb_subset_input_override_name_table`
  functions, and the four raw-CFF-outline accessors.
- `hb-config.hh` defines `HB_NO_SUBSET_DEPEND` **unless** `HB_EXPERIMENTAL_API`
  is set, so without the feature the entire `hb-subset-depend.h` API is not
  compiled into the library at all — the symbols do not exist to link against.
  (`HB_LEAN` and `HB_TINY` also define `HB_NO_SUBSET_DEPEND`, so those
  size-reduction profiles remove it even with `experimental` on.)

Upstream makes no compatibility promise for anything behind
`HB_EXPERIMENTAL_API`; the symbols may vanish in a point release.

## Types

### `hb_subset_input_t`

```c
typedef struct hb_subset_input_t hb_subset_input_t;
```

```rust
crate::opaque_handle! { hb_subset_input_t }
```

"Things that change based on the input. Characters to keep, etc." — an opaque,
reference-counted description of the subset you want: the Unicode codepoints and
glyph IDs to retain, the tables to drop or pass through, the `name` records to
keep, the layout features and scripts to keep, the variation axes to pin, an
optional explicit old-to-new glyph map, and the boolean `hb_subset_flags_t`
settings.

Create one with `hb_subset_input_create_or_fail()`, mutate it in place through
the accessors below, and release it with `hb_subset_input_destroy()`. It carries
the usual HarfBuzz user-data table. One input can be reused against many faces.

### `hb_subset_plan_t`

```c
typedef struct hb_subset_plan_t hb_subset_plan_t;
```

```rust
crate::opaque_handle! { hb_subset_plan_t }
```

"Contains information about how the subset operation will be executed, such as
mappings from the old glyph IDs to the new ones in the subset." An opaque,
reference-counted object: an `hb_subset_input_t` **resolved against a particular
`hb_face_t`**. Creating it performs glyph closure (following `cmap`, composite
outlines, `GSUB` closure, bidi mirroring, colour layers) and decides which
tables and glyphs survive and how old glyph IDs map onto new ones.

Create with `hb_subset_plan_create_or_fail()`, run with
`hb_subset_plan_execute_or_fail()`, release with `hb_subset_plan_destroy()`.
Carries a user-data table. Since HarfBuzz 4.0.0.

### `hb_subset_flags_t`

```c
typedef enum { /*< flags >*/ HB_SUBSET_FLAGS_DEFAULT = 0x00000000u, ... } hb_subset_flags_t;
```

```rust
pub type hb_subset_flags_t = core::ffi::c_int;
```

List of boolean properties that can be configured on the subset input. These are
**bit flags**: combine with bitwise OR and install the whole field at once with
`hb_subset_input_set_flags()`. The C enumeration has no sentinel and no value
exceeds `0x7FFFFFFF`, so the underlying type is `int`. Since HarfBuzz 2.9.0.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_SUBSET_FLAGS_DEFAULT` | 0x0000 | All flags at their default value of false. |
| `HB_SUBSET_FLAGS_NO_HINTING` | 0x0001 | Drop hinting instructions in the produced subset. Otherwise they are retained. |
| `HB_SUBSET_FLAGS_RETAIN_GIDS` | 0x0002 | Do not modify glyph indices. Dropped glyphs keep their index as an empty glyph. |
| `HB_SUBSET_FLAGS_DESUBROUTINIZE` | 0x0004 | When subsetting a CFF font, attempt to remove subroutines from the CFF glyphs. |
| `HB_SUBSET_FLAGS_NAME_LEGACY` | 0x0008 | Retain non-Unicode `name` records in the subset. |
| `HB_SUBSET_FLAGS_SET_OVERLAPS_FLAG` | 0x0010 | Set the `OVERLAP_SIMPLE` flag on each simple glyph. |
| `HB_SUBSET_FLAGS_PASSTHROUGH_UNRECOGNIZED` | 0x0020 | Do not drop unrecognized tables; pass them through untouched. |
| `HB_SUBSET_FLAGS_NOTDEF_OUTLINE` | 0x0040 | Retain the `.notdef` glyph outline in the final subset. |
| `HB_SUBSET_FLAGS_GLYPH_NAMES` | 0x0080 | Retain the PostScript glyph names in the final subset. |
| `HB_SUBSET_FLAGS_NO_PRUNE_UNICODE_RANGES` | 0x0100 | Do not recalculate the Unicode ranges in `OS/2`. |
| `HB_SUBSET_FLAGS_NO_LAYOUT_CLOSURE` | 0x0200 | Do not perform glyph closure on layout substitution rules (`GSUB`). Since 7.2.0. |
| `HB_SUBSET_FLAGS_OPTIMIZE_IUP_DELTAS` | 0x0400 | Perform IUP delta optimization on the remaining `gvar` table's deltas. Since 8.5.0. |
| `HB_SUBSET_FLAGS_NO_BIDI_CLOSURE` | 0x0800 | Do not pull mirrored versions of input codepoints into the subset. Since 11.1.0. |
| `HB_SUBSET_FLAGS_IFTB_REQUIREMENTS` | 0x1000 | **Needs `experimental`.** Enforce requirements on the output subset so it can be used with incremental font transfer IFTB patches; primarily, forces all outline data to use long (32-bit) offsets. Upstream marks it `Since: EXPERIMENTAL`. |
| `HB_SUBSET_FLAGS_RETAIN_NUM_GLYPHS` | 0x2000 | **Needs `experimental`.** Set alongside `RETAIN_GIDS`, keeps the glyph count unchanged, appending empty glyphs at the end if necessary. Upstream marks it `Since: EXPERIMENTAL`. |
| `HB_SUBSET_FLAGS_DOWNGRADE_CFF2` | 0x4000 | When instantiating a variable font with all axes pinned, convert the output `CFF2` table to CFF1, for compatibility with renderers that do not support `CFF2`. Since 13.0.0. |
| `HB_SUBSET_FLAGS_CFF_IDENTITY_CHARSET` | 0x8000 | When subsetting a CID-keyed CFF font, use sequential identity CIDs (CID = new GID) in the output charset rather than preserving the original CIDs. Since 14.3.0. |

Note the gap: `0x1000` and `0x2000` exist only in an experimental build, so the
numeric values of the two flags above them are stable either way.

### `hb_subset_sets_t`

```c
typedef enum { HB_SUBSET_SETS_GLYPH_INDEX = 0, ... } hb_subset_sets_t;
```

```rust
pub type hb_subset_sets_t = core::ffi::c_int;
```

List of sets that can be configured on the subset input. Each value selects one
of the `hb_set_t` collections an input carries; retrieve the set with
`hb_subset_input_set()` and edit it in place. The C enumeration has no sentinel
and its largest enumerator is 7, so it fits in an `int`. Since HarfBuzz 2.9.1.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_SUBSET_SETS_GLYPH_INDEX` | 0 | The set of glyph indexes to retain in the subset. |
| `HB_SUBSET_SETS_UNICODE` | 1 | The set of Unicode codepoints to retain in the subset. |
| `HB_SUBSET_SETS_NO_SUBSET_TABLE_TAG` | 2 | Table tags for tables that should **not** be subsetted — copied through verbatim. |
| `HB_SUBSET_SETS_DROP_TABLE_TAG` | 3 | Table tags for tables that will be dropped from the subset. |
| `HB_SUBSET_SETS_NAME_ID` | 4 | The set of `name` IDs that will be retained. |
| `HB_SUBSET_SETS_NAME_LANG_ID` | 5 | The set of `name` language IDs that will be retained. |
| `HB_SUBSET_SETS_LAYOUT_FEATURE_TAG` | 6 | The set of layout feature tags that will be retained in the subset. |
| `HB_SUBSET_SETS_LAYOUT_SCRIPT_TAG` | 7 | The set of layout script tags that will be retained. Defaults to all tags. Since 5.0.0. |

All of these are `hb_set_t` collections of 32-bit values; the table-tag and
feature-tag sets hold `hb_tag_t` values, the Unicode set holds codepoints, the
name-ID sets hold `hb_ot_name_id_t` and platform-specific language IDs. Sets can
be **inverted** (`hb_set_invert`) to mean "everything", which is how
`keep_everything` works.

### `hb_subset_serialize_link_t`

```c
typedef struct hb_subset_serialize_link_t {
  unsigned int width;
  unsigned int position;
  unsigned int objidx;
} hb_subset_serialize_link_t;
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_subset_serialize_link_t {
    pub width: c_uint,
    pub position: c_uint,
    pub objidx: c_uint,
}
```

Represents a link between two objects in the object graph to be serialized — a
record of where, inside one object's bytes, an offset to another object lives.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `width` | `unsigned int` | `c_uint` | `offsetSize` in bytes — 2 for a 16-bit offset, 3 or 4 for the wider forms OpenType uses. |
| `position` | `unsigned int` | `c_uint` | Position of the offset field, in bytes, from the beginning of the subtable (i.e. from `head`). |
| `objidx` | `unsigned int` | `c_uint` | Index of the subtable this link points at, into the object array passed to `hb_subset_serialize_or_fail()`. |

Since HarfBuzz 10.2.0.

### `hb_subset_serialize_object_t`

```c
typedef struct hb_subset_serialize_object_t {
  char *head;
  char *tail;
  unsigned int num_real_links;
  hb_subset_serialize_link_t *real_links;
  unsigned int num_virtual_links;
  hb_subset_serialize_link_t *virtual_links;
} hb_subset_serialize_object_t;
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct hb_subset_serialize_object_t {
    pub head: *mut c_char,
    pub tail: *mut c_char,
    pub num_real_links: c_uint,
    pub real_links: *mut hb_subset_serialize_link_t,
    pub num_virtual_links: c_uint,
    pub virtual_links: *mut hb_subset_serialize_link_t,
}
```

Represents an object in the object graph to be serialized. The object's own bytes
are the half-open range `head .. tail`.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `head` | `char *` | `*mut c_char` | Start of object data. |
| `tail` | `char *` | `*mut c_char` | End of object data (exclusive). |
| `num_real_links` | `unsigned int` | `c_uint` | Number of offset fields in the object. |
| `real_links` | `hb_subset_serialize_link_t *` | `*mut hb_subset_serialize_link_t` | Array of offset info. Each entry names a location inside `head..tail` that HarfBuzz will overwrite with the resolved offset. |
| `num_virtual_links` | `unsigned int` | `c_uint` | Number of objects that must be packed **after** the current object in the final serialized order. |
| `virtual_links` | `hb_subset_serialize_link_t *` | `*mut hb_subset_serialize_link_t` | Array of virtual link info. Virtual links write nothing into the bytes; they only constrain packing order. |

Since HarfBuzz 10.2.0.

### `hb_subset_depend_t` — needs `experimental`

```c
typedef struct hb_subset_depend_t hb_subset_depend_t;   /* #ifndef HB_NO_SUBSET_DEPEND */
```

```rust
crate::opaque_handle! { hb_subset_depend_t }
```

Data type for holding glyph dependency graphs. Built from a face with
`hb_subset_depend_from_face_or_fail()` and released with
`hb_subset_depend_destroy()`. Reference-counted like every HarfBuzz object.

The header carries an explicit warning: **"Highly experimental API. Subject to
change."** The whole type sits inside `#ifndef HB_NO_SUBSET_DEPEND`, which
`hb-config.hh` defines unless `HB_EXPERIMENTAL_API` is set.

Since HarfBuzz 14.3.0.

### `hb_subset_depend_entry_t` — needs `experimental`

```c
typedef struct {
  hb_tag_t                      table_tag;
  hb_codepoint_t                dependent;
  hb_tag_t                      layout_tag;
  hb_codepoint_t                ligature_set_index;
  hb_codepoint_t                context_set_index;
  hb_subset_depend_edge_flags_t flags;
} hb_subset_depend_entry_t;
```

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_subset_depend_entry_t {
    pub table_tag: hb_tag_t,
    pub dependent: hb_codepoint_t,
    pub layout_tag: hb_tag_t,
    pub ligature_set_index: hb_codepoint_t,
    pub context_set_index: hb_codepoint_t,
    pub flags: hb_subset_depend_edge_flags_t,
}
```

A single dependency edge returned by `hb_subset_depend_lookup_glyph()`.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `table_tag` | `hb_tag_t` | `u32` | Source table — `GSUB`, `glyf`, `CFF `, `COLR`, or `MATH`. |
| `dependent` | `hb_codepoint_t` | `u32` | Target glyph ID: the glyph this edge pulls in. |
| `layout_tag` | `hb_tag_t` | `u32` | Feature tag for `GSUB` edges; zero otherwise. |
| `ligature_set_index` | `hb_codepoint_t` | `u32` | Index into the sets array for ligature component glyphs, or `HB_CODEPOINT_INVALID` if this is not a ligature edge. Resolve with `hb_subset_depend_lookup_set()`. |
| `context_set_index` | `hb_codepoint_t` | `u32` | Index into the sets array for context requirement glyphs, or `HB_CODEPOINT_INVALID` if none. Resolve with `hb_subset_depend_lookup_set()`. |
| `flags` | `hb_subset_depend_edge_flags_t` | `c_int` | Edge flags; see below. |

Since HarfBuzz 14.3.0.

### `hb_subset_depend_edge_flags_t`

```c
typedef enum { /*< flags >*/ HB_SUBSET_DEPEND_EDGE_FLAG_NONE = 0x00u, ... } hb_subset_depend_edge_flags_t;
```

```rust
pub type hb_subset_depend_edge_flags_t = core::ffi::c_int;
```

Flags on dependency edges that mark edges which may produce **expected
over-approximation** when computing closure via the depend graph, relative to
`hb_ot_layout_lookups_substitute_closure()`. They exist so a caller can
distinguish known limitations of static dependency analysis from bugs.

Note that unlike the type and function declarations, this enumeration is defined
*outside* the `HB_NO_SUBSET_DEPEND` guard in the header, so it exists in every
build; this crate nevertheless keeps the whole depend surface behind
`experimental`, since the functions that produce these values do not exist
otherwise. Bit flags; largest enumerator is 2, so the underlying type is `int`.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_SUBSET_DEPEND_EDGE_FLAG_NONE` | 0x00 | No flags set. |
| `HB_SUBSET_DEPEND_EDGE_FLAG_FROM_CONTEXT_POSITION` | 0x01 | Edge from a multi-position contextual rule (`Context` or `ChainContext` with `inputCount > 1`). Depend extraction records edges from what glyphs could *statically* be at each position per input coverage or class. At runtime the lookups within the rule are applied sequentially: a lookup at an earlier position may transform the glyph at a later position, and two lookups at the same position may interact so that one produces a glyph another immediately consumes as an "intermediate". A glyph matching the static coverage may therefore not persist when the rule actually fires, so this edge may not trigger during closure. |
| `HB_SUBSET_DEPEND_EDGE_FLAG_FROM_NESTED_CONTEXT` | 0x02 | Edge from a lookup invoked within another contextual lookup. The outer context's requirements are not propagated to this edge, so it may fire even when those requirements are not met. |

Since HarfBuzz 14.3.0.

### Types from elsewhere

`hb_face_t` (`hb-face.h`), `hb_blob_t` (`hb-blob.h`), `hb_set_t` (`hb-set.h`),
`hb_map_t` (`hb-map.h`), `hb_tag_t` / `hb_codepoint_t` / `hb_bool_t` /
`hb_destroy_func_t` / `hb_user_data_key_t` (`hb-common.h`), and
`hb_ot_name_id_t` (`hb-ot-name.h`) all appear in these signatures and are
documented on their own pages.

## Functions

### Subset input: lifecycle

#### `hb_subset_input_create_or_fail`

```c
hb_subset_input_t *hb_subset_input_create_or_fail (void);
```

```rust
pub fn hb_subset_input_create_or_fail() -> *mut hb_subset_input_t;
```

Creates a new subset input object, pre-populated with the defaults tabulated in
the overview.

**Parameters** — none.

**Returns** — a new subset input, or **null** on failure (allocation failure, or
the constructor's own set allocations failing). Unlike the older blob API there
is no "empty object" fallback.

**Ownership** — `transfer full`: the caller owns the result and must release it
with `hb_subset_input_destroy()`.

**Notes** — Since HarfBuzz 1.8.0.

#### `hb_subset_input_reference`

```c
hb_subset_input_t *hb_subset_input_reference (hb_subset_input_t *input);
```

```rust
pub fn hb_subset_input_reference(input: *mut hb_subset_input_t) -> *mut hb_subset_input_t;
```

Increases the reference count on `input` and returns the same pointer, which
makes it convenient to use inline.

**Parameters** — `input`: the object to reference. Upstream marks the function
`(skip)`; nullability is unspecified, but HarfBuzz's `hb_object_reference`
tolerates the shared null objects.

**Returns** — `input`.

**Ownership** — every call must be matched by an `hb_subset_input_destroy()`.

**Notes** — Since HarfBuzz 1.8.0. Reference counts are atomic in a normally
configured build.

#### `hb_subset_input_destroy`

```c
void hb_subset_input_destroy (hb_subset_input_t *input);
```

```rust
pub fn hb_subset_input_destroy(input: *mut hb_subset_input_t);
```

Decreases the reference count on `input`; when it reaches zero the object is
destroyed and all its memory freed, and any user-data destroy callbacks run.

**Parameters** — `input`: the object to release.

**Returns** — nothing. There is no way to observe whether the object actually
went away.

**Notes** — Since HarfBuzz 1.8.0.

#### `hb_subset_input_set_user_data`

```c
hb_bool_t hb_subset_input_set_user_data (hb_subset_input_t  *input,
                                         hb_user_data_key_t *key,
                                         void               *data,
                                         hb_destroy_func_t   destroy,
                                         hb_bool_t           replace);
```

```rust
pub fn hb_subset_input_set_user_data(
    input: *mut hb_subset_input_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

Attaches a user-data key/data pair to the given subset input object.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `input` | The object to annotate. | — |
| `key` | The user-data key. HarfBuzz uses the **address** of `key`, not its contents, so the key object must outlive the input; a `static` is the usual choice. | — |
| `data` | Pointer to the user data. | May be null. |
| `destroy` | Called with `data` when the input is destroyed or the value is replaced. | Nullable — upstream annotates it `(nullable)`. |
| `replace` | Whether to overwrite existing data stored under the same key. | — |

**Returns** — `true` on success, `false` otherwise (allocation failure, or a
non-replace call against an existing key).

**Ownership** — HarfBuzz takes responsibility for `data` only through the
`destroy` callback you supply.

**Notes** — Since HarfBuzz 2.9.0. Upstream marks the function `(skip)`.

#### `hb_subset_input_get_user_data`

```c
void *hb_subset_input_get_user_data (const hb_subset_input_t *input,
                                     hb_user_data_key_t      *key);
```

```rust
pub fn hb_subset_input_get_user_data(
    input: *const hb_subset_input_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

Fetches the user data associated with the specified key, attached to the
specified subset input object. Note the `const` input parameter.

**Returns** — the stored pointer, or null when no entry is present for that key.

**Ownership** — `transfer none`: the pointer belongs to whoever stored it and
must not be freed by the caller.

**Notes** — Since HarfBuzz 2.9.0. Upstream marks the function `(skip)`.

### Subset input: what to keep

#### `hb_subset_input_keep_everything`

```c
void hb_subset_input_keep_everything (hb_subset_input_t *input);
```

```rust
pub fn hb_subset_input_keep_everything(input: *mut hb_subset_input_t);
```

Configures the input object to keep everything in the font face — all Unicodes,
glyphs, names, layout items, glyph names, and so on. The input can be tailored
afterwards by the caller.

**Parameters** — `input`: the object to reconfigure.

**Returns** — nothing.

**What it actually does**, precisely, because the difference matters:

1. Clears and then **inverts** six sets — `UNICODE`, `GLYPH_INDEX`, `NAME_ID`,
   `NAME_LANG_ID`, `LAYOUT_FEATURE_TAG`, `LAYOUT_SCRIPT_TAG` — so each means
   "everything".
2. **Clears** the drop-table set, so no table is dropped.
3. Replaces the flag field wholesale with
   `NOTDEF_OUTLINE | GLYPH_NAMES | NAME_LEGACY | NO_PRUNE_UNICODE_RANGES | PASSTHROUGH_UNRECOGNIZED`.

Note what it does **not** touch: the no-subset-table set keeps its defaults
(`gasp`, `fpgm`, `prep`, `VDMX`, `DSIG`), the old-to-new glyph map is left alone,
and any axis pins already configured survive. Note also that step 3 *replaces*
the flags, so any flag you set before calling this is lost.

**Notes** — Since HarfBuzz 7.0.0. This is the natural starting point when you
want to remove a few specific things rather than select a few specific things —
and it is exactly what `hb_subset_preprocess()` uses internally.

#### `hb_subset_input_unicode_set`

```c
hb_set_t *hb_subset_input_unicode_set (hb_subset_input_t *input);
```

```rust
pub fn hb_subset_input_unicode_set(input: *mut hb_subset_input_t) -> *mut hb_set_t;
```

Gets the set of Unicode codepoints to retain; the caller should modify the set as
needed. Equivalent to `hb_subset_input_set(input, HB_SUBSET_SETS_UNICODE)`.

**Returns** — a pointer to the input's `hb_set_t` of Unicode codepoints. Empty on
a fresh input.

**Ownership** — `transfer none`. The set belongs to the input; edit it in place
with the `hb_set_*` functions and do **not** call `hb_set_destroy()` on it. It
stays valid as long as you hold a reference to the input.

**Notes** — Since HarfBuzz 1.8.0. This is the set most callers populate: add the
codepoints your text needs and let the subsetter's closure find the glyphs.

#### `hb_subset_input_glyph_set`

```c
hb_set_t *hb_subset_input_glyph_set (hb_subset_input_t *input);
```

```rust
pub fn hb_subset_input_glyph_set(input: *mut hb_subset_input_t) -> *mut hb_set_t;
```

Gets the set of glyph IDs to retain; the caller should modify the set as needed.
Equivalent to `hb_subset_input_set(input, HB_SUBSET_SETS_GLYPH_INDEX)`.

**Returns** — a pointer to the input's `hb_set_t` of glyph IDs. Empty on a fresh
input.

**Ownership** — `transfer none`, exactly as for the Unicode set.

**Notes** — Since HarfBuzz 1.8.0. The glyph set and the Unicode set are additive:
the retained glyph set is the closure of (glyphs named here) ∪ (glyphs the
`cmap` maps the requested codepoints to).

#### `hb_subset_input_set`

```c
hb_set_t *hb_subset_input_set (hb_subset_input_t *input, hb_subset_sets_t set_type);
```

```rust
pub fn hb_subset_input_set(
    input: *mut hb_subset_input_t,
    set_type: hb_subset_sets_t,
) -> *mut hb_set_t;
```

Gets the set of the specified type — the general form of the two accessors above,
and the only way to reach the other six sets.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `input` | The object to query. | — |
| `set_type` | Which set to retrieve. | **Must be one of the eight `hb_subset_sets_t` values (0–7).** The implementation is `input->sets_iter()[set_type]` with no bounds check, so an out-of-range value reads out of bounds. |

**Returns** — a pointer to the requested `hb_set_t`.

**Ownership** — `transfer none`. Edit in place; never destroy.

**Notes** — Since HarfBuzz 2.9.1.

#### `hb_subset_input_old_to_new_glyph_mapping`

```c
hb_map_t *hb_subset_input_old_to_new_glyph_mapping (hb_subset_input_t *input);
```

```rust
pub fn hb_subset_input_old_to_new_glyph_mapping(input: *mut hb_subset_input_t) -> *mut hb_map_t;
```

Returns a map that can be used to provide an **explicit** mapping from old to new
glyph IDs in the produced subset. The caller should populate the map as desired.

**Returns** — a pointer to the input's `hb_map_t`. Empty on a fresh input, which
means "let the subsetter assign IDs automatically".

**Ownership** — `transfer none`. The map belongs to the input; populate it with
`hb_map_set()` and do not destroy it.

**Rules when you do populate it**

- The mapping must be **unique**: no two original glyph IDs may map to the same
  new ID.
- `HB_SUBSET_FLAGS_RETAIN_GIDS` cannot also be enabled.
- Retained glyphs not named in the mapping are assigned IDs **above** the highest
  ID in the mapping.
- Non-monotonic mappings are accepted and applied, but may result in unsorted
  `Coverage` tables. Some consumers — OTS, for one — reject those. Prefer a
  monotonic mapping where possible.

**Notes** — Since HarfBuzz 7.3.0.

#### `hb_subset_input_get_flags`

```c
hb_subset_flags_t hb_subset_input_get_flags (hb_subset_input_t *input);
```

```rust
pub fn hb_subset_input_get_flags(input: *mut hb_subset_input_t) -> hb_subset_flags_t;
```

Gets all of the subsetting flags in the input object, as a bit field of
`hb_subset_flags_t` values.

**Returns** — the current bit field.

**Notes** — Since HarfBuzz 2.9.0.

#### `hb_subset_input_set_flags`

```c
void hb_subset_input_set_flags (hb_subset_input_t *input, unsigned value);
```

```rust
pub fn hb_subset_input_set_flags(input: *mut hb_subset_input_t, value: c_uint);
```

Sets **all** of the flags in the input object to the values specified by the bit
field. This *replaces* the whole field rather than merging into it; combine
`hb_subset_flags_t` values with bitwise OR, and read-modify-write through
`hb_subset_input_get_flags()` if you only want to add one.

**Parameters** — note the asymmetry: this takes an `unsigned` while the getter
returns the enumeration type. In Rust that means `c_uint` in and `c_int` out, so
a round trip needs a cast.

**Returns** — nothing. No validation is performed: unknown bits are stored and
ignored.

**Notes** — Since HarfBuzz 2.9.0.

### Instancing: pinning and narrowing variation axes

All five of these functions store into one per-input table keyed by axis tag,
holding a `(min, default, max)` triple in **user-space design coordinates**.
Pinning is just the degenerate case where all three are equal. Setting an axis
twice replaces the earlier entry. Upstream marks the first five `(skip)`.

Every function that takes a `face` looks the axis up in that face's `fvar` table,
so the same input configured against a different face may behave differently.

#### `hb_subset_input_pin_all_axes_to_default`

```c
hb_bool_t hb_subset_input_pin_all_axes_to_default (hb_subset_input_t *input, hb_face_t *face);
```

```rust
pub fn hb_subset_input_pin_all_axes_to_default(
    input: *mut hb_subset_input_t,
    face: *mut hb_face_t,
) -> hb_bool_t;
```

Pins all variation axes to their default locations in the given subset input
object — the shortest route from a variable font to a static instance at its
default position. The `CFF2` table, if present, will be de-subroutinized.

**Parameters** — `input`, and `face` whose `fvar` supplies the axis list and
default values. Nullability is unspecified; both are dereferenced.

**Returns** — `true` on success, `false` otherwise. **False includes the case
where `face` has no variation axes at all** (`hb_ot_var_get_axis_count()`
returns 0), so a false return is not necessarily an error — check whether the
face is variable first if you care.

Also returns false on allocation failure of the temporary axis-info array or of
the input's axis table.

**Notes** — Since HarfBuzz 8.3.1. Compiled only when HarfBuzz is built with
variation support (`#ifndef HB_NO_VAR`); the `HB_LEAN`/`HB_TINY` profiles define
`HB_NO_VAR` and therefore remove all five axis functions.

#### `hb_subset_input_pin_axis_to_default`

```c
hb_bool_t hb_subset_input_pin_axis_to_default (hb_subset_input_t *input,
                                               hb_face_t         *face,
                                               hb_tag_t           axis_tag);
```

```rust
pub fn hb_subset_input_pin_axis_to_default(
    input: *mut hb_subset_input_t,
    face: *mut hb_face_t,
    axis_tag: hb_tag_t,
) -> hb_bool_t;
```

Pins one variation axis to its default location. The `CFF2` table, if present,
will be de-subroutinized.

**Parameters** — `axis_tag` is a four-byte axis tag such as `wght`, `wdth`,
`opsz`.

**Returns** — `true` on success; `false` in particular when `face` has no axis
with that tag (`hb_ot_var_find_axis_info()` fails), or on allocation failure.

**Notes** — Since HarfBuzz 6.0.0.

#### `hb_subset_input_pin_axis_location`

```c
hb_bool_t hb_subset_input_pin_axis_location (hb_subset_input_t *input,
                                             hb_face_t         *face,
                                             hb_tag_t           axis_tag,
                                             float              axis_value);
```

```rust
pub fn hb_subset_input_pin_axis_location(
    input: *mut hb_subset_input_t,
    face: *mut hb_face_t,
    axis_tag: hb_tag_t,
    axis_value: c_float,
) -> hb_bool_t;
```

Pins one variation axis to a fixed location.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `input` | The input to configure. | — |
| `face` | Supplies the `fvar` axis record. | — |
| `axis_tag` | Tag of the axis to be pinned. | Must exist in `fvar`. |
| `axis_value` | Location on the axis to pin at, in **user-space design coordinates** (e.g. 700 for `wght`, not a normalized -1..1 value). | **Clamped** into the axis's `fvar` minimum and maximum — an out-of-range value is silently pulled to the nearest bound rather than rejected. |

**Returns** — `true` on success; `false` in particular when `face` has no axis
with tag `axis_tag`.

**Notes** — Since HarfBuzz 6.0.0. The `CFF2` table, if present, will be
de-subroutinized. Pinning **every** axis is what makes
`HB_SUBSET_FLAGS_DOWNGRADE_CFF2` applicable.

#### `hb_subset_input_set_axis_range`

```c
hb_bool_t hb_subset_input_set_axis_range (hb_subset_input_t *input,
                                          hb_face_t         *face,
                                          hb_tag_t           axis_tag,
                                          float              axis_min_value,
                                          float              axis_max_value,
                                          float              axis_def_value);
```

```rust
pub fn hb_subset_input_set_axis_range(
    input: *mut hb_subset_input_t,
    face: *mut hb_face_t,
    axis_tag: hb_tag_t,
    axis_min_value: c_float,
    axis_max_value: c_float,
    axis_def_value: c_float,
) -> hb_bool_t;
```

Restricts the range of variation on an axis — *partial* instancing. The output
font keeps the axis but with a narrower design space.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `axis_min_value` | New minimum. | **NaN means "keep the face's existing `fvar` minimum"**. Clamped into the `fvar` range. |
| `axis_max_value` | New maximum. | NaN means "keep the existing maximum". Clamped into the `fvar` range. |
| `axis_def_value` | New default. | NaN means "keep the existing default". Clamped into the *new* min/max range. |

The order of operations, from the implementation: NaN substitution first, then
`min > max` is rejected outright, then min and max are clamped into the `fvar`
range, then the default is clamped into the resulting new range. In upstream's
words: "If the `fvar` axis default value is not within the new range, the new
default value will be changed to the new min or max value, whichever is closer to
the `fvar` axis default."

**Returns** — `true` on success; `false` when the face has no such axis, or when
`min > max` after NaN substitution.

**Notes** — Since HarfBuzz 8.5.0. Note that `float` NaN is the "leave alone"
sentinel — in Rust write `f32::NAN`, and remember that `NaN != NaN`, so never
test these values with `==`.

#### `hb_subset_input_get_axis_range`

```c
hb_bool_t hb_subset_input_get_axis_range (hb_subset_input_t *input,
                                          hb_tag_t           axis_tag,
                                          float             *axis_min_value,
                                          float             *axis_max_value,
                                          float             *axis_def_value);
```

```rust
pub fn hb_subset_input_get_axis_range(
    input: *mut hb_subset_input_t,
    axis_tag: hb_tag_t,
    axis_min_value: *mut c_float,
    axis_max_value: *mut c_float,
    axis_def_value: *mut c_float,
) -> hb_bool_t;
```

Gets the axis range assigned by previous calls to
`hb_subset_input_set_axis_range()` — or to any of the pin functions, since they
all write the same table.

**Parameters** — the three out-pointers receive the configured minimum, maximum,
and default. Note that the *implementation writes through all three
unconditionally* when it returns true, so **null out-pointers are not safe**
even though the pattern elsewhere in HarfBuzz tolerates them. Pass real
addresses for all three.

**Returns** — `true` if a range has been set for this axis tag, `false`
otherwise. Nothing is written when it returns false.

**Notes** — Since HarfBuzz 8.5.0. Takes no `face`: it reports what you
configured, not what the font offers.

#### `hb_subset_axis_range_from_string`

```c
hb_bool_t hb_subset_axis_range_from_string (const char *str, int len,
                                            float *axis_min_value,
                                            float *axis_max_value,
                                            float *axis_def_value);
```

```rust
pub fn hb_subset_axis_range_from_string(
    str_: *const c_char,
    len: c_int,
    axis_min_value: *mut c_float,
    axis_max_value: *mut c_float,
    axis_def_value: *mut c_float,
) -> hb_bool_t;
```

Parses a string into a subset axis range (min, def, max) — the parser behind the
`hb-subset` command-line tool's `--instance=wght=300:500` syntax.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `str` | The axis-position string. | Dereferenced immediately (`strlen` when `len < 0`); treat null as forbidden. |
| `len` | Length of `str`, or **`-1`** when `str` is NUL-terminated. | Any negative value is treated as "NUL-terminated". |
| `axis_min_value` | Out: parsed minimum. | Written on success only. |
| `axis_max_value` | Out: parsed maximum. | Written on success only. |
| `axis_def_value` | Out: parsed default. | Written on success only. |

**Accepted syntax**

| Input | min | def | max |
| --- | --- | --- | --- |
| `400` (a bare value) | 400 | 400 | 400 |
| `drop` | NaN | NaN | NaN |
| `300:500` (two parts) | 300 | **NaN** | 500 |
| `300:400:500` (three parts) | 300 | 400 | 500 |
| `:300:500` (empty component) | NaN (= existing) | 300 | 500 |
| `300::500` | 300 | NaN (= existing) | 500 |

An empty component means "keep the existing value for that part", and is
reported as NaN — which is exactly the sentinel
`hb_subset_input_set_axis_range()` consumes, so the two functions compose
directly.

**Returns** — `true` if `str` is successfully parsed, `false` otherwise (a
non-numeric component, or a count other than 1, 2, or 3).

**Notes** — Since HarfBuzz 10.2.0. Parsing uses HarfBuzz's own locale-independent
double parser.

#### `hb_subset_axis_range_to_string`

```c
void hb_subset_axis_range_to_string (hb_subset_input_t *input,
                                     hb_tag_t axis_tag,
                                     char *buf,
                                     unsigned size);
```

```rust
pub fn hb_subset_axis_range_to_string(
    input: *mut hb_subset_input_t,
    axis_tag: hb_tag_t,
    buf: *mut c_char,
    size: c_uint,
);
```

Converts the axis range currently configured for `axis_tag` into a
NUL-terminated string in the `min:def:max` format understood by
`hb_subset_axis_range_from_string()`.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `input` | The input to read from. | — |
| `axis_tag` | Which axis to format. | — |
| `buf` | Caller-allocated output buffer. | The caller is responsible for allocating a big enough buffer; **128 bytes is more than enough**. |
| `size` | The allocated size of `buf`, in bytes. | A `size` of 0 makes the function return immediately without writing. |

**Returns** — nothing, and there is **no success indication**. Nothing at all is
written when `size` is zero or when no range has been configured for `axis_tag`,
so the buffer keeps whatever it held before. Zero the first byte yourself before
the call if you need to detect that.

The output is always the three-part `%g:%g:%g` form and is truncated (with a
NUL) to `size - 1` bytes if it does not fit. Formatting is done under the `C`
locale, so the decimal separator is always `.`.

**Notes** — Since HarfBuzz 10.2.0.

### Running the subset

#### `hb_subset_or_fail`

```c
hb_face_t *hb_subset_or_fail (hb_face_t *source, const hb_subset_input_t *input);
```

```rust
pub fn hb_subset_or_fail(
    source: *mut hb_face_t,
    input: *const hb_subset_input_t,
) -> *mut hb_face_t;
```

Subsets a font according to the provided input. This is the one-call entry point;
internally it is exactly `plan_create_or_fail` → `plan_execute_or_fail` →
`plan_destroy`.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `source` | Font face data to be subset. | **Null is tolerated** and returns null. |
| `input` | Input to use for the subsetting. Taken by `const` pointer — the call does not modify it. | Null is tolerated and returns null. |

**Returns** — a new `hb_face_t` for the subset, or **null** if the subset
operation fails, if either argument is null, or if the source face has no glyphs.

**Ownership** — the caller owns the result and must release it with
`hb_face_destroy()`. The input is neither consumed nor referenced beyond the
call, so you may reuse it against another face immediately.

**Getting the bytes out** — the returned face is a *built* face, not a wrapper
around a blob. Call `hb_face_reference_blob()` on it to serialize the subsetted
font file into an `hb_blob_t`, then `hb_blob_get_data()` for the pointer and
length. That blob is a new reference you must destroy.

**Notes** — Since HarfBuzz 2.9.0.

#### `hb_subset_plan_create_or_fail`

```c
hb_subset_plan_t *hb_subset_plan_create_or_fail (hb_face_t               *face,
                                                 const hb_subset_input_t *input);
```

```rust
pub fn hb_subset_plan_create_or_fail(
    face: *mut hb_face_t,
    input: *const hb_subset_input_t,
) -> *mut hb_subset_plan_t;
```

Computes a plan for subsetting `face` according to `input`. The plan describes
which tables and glyphs should be retained. This is where glyph closure happens,
so it is the expensive half of the operation.

**Parameters** — `face` and `input`; nullability is unspecified and both are
dereferenced by the plan constructor, so treat null as forbidden here (unlike
`hb_subset_or_fail`, which checks).

**Returns** — a new subset plan, or **null** if creating the plan fails (which
includes any error accumulated in the plan constructor, not only allocation
failure).

**Ownership** — `transfer full`: destroy with `hb_subset_plan_destroy()`. The
plan holds its own references to what it needs from `face`.

**Notes** — Since HarfBuzz 4.0.0. Splitting plan creation from execution lets
you inspect the glyph mappings *before* paying for serialization, and lets you
execute one plan more than once.

#### `hb_subset_plan_execute_or_fail`

```c
hb_face_t *hb_subset_plan_execute_or_fail (hb_subset_plan_t *plan);
```

```rust
pub fn hb_subset_plan_execute_or_fail(plan: *mut hb_subset_plan_t) -> *mut hb_face_t;
```

Executes the provided subsetting plan: walks the source face's table directory,
skips the tables the plan says to drop, subsets or copies the rest, and assembles
the result.

**Parameters** — `plan`. **Null is tolerated** and returns null, as is a plan
that is already in an error state.

**Returns** — on success, a reference to the generated font subset; **null** if
the subsetting operation fails.

**Ownership** — the caller owns the result and must release it with
`hb_face_destroy()`. The plan is not consumed — destroy it separately.

**Notes** — Since HarfBuzz 4.0.0.

#### `hb_subset_preprocess`

```c
hb_face_t *hb_subset_preprocess (hb_face_t *source);
```

```rust
pub fn hb_subset_preprocess(source: *mut hb_face_t) -> *mut hb_face_t;
```

Preprocesses the face and attaches data that will be needed by the subsetter, so
that future subsetting operations can reuse the precomputed data and run faster.
This is the right thing to do when you will produce many subsets of the same
font — the usual web-font-service shape.

Internally it creates an input, calls `keep_everything()` on it, sets two
internal flags (`attach_accelerator_data`, and `force_long_loca` so that glyph
bytes can be stored unpadded and the future subset can skip the trim-padding
step), and runs `hb_subset_or_fail()`.

**Parameters** — `source`: the face to preprocess.

**Returns** — a new `hb_face_t`. **Never null**: on any failure it returns
`hb_face_reference(source)` — a new reference to the *original* face — so the
result is always usable and a failure is invisible.

**Ownership** — the caller owns the returned face and must release it with
`hb_face_destroy()`, whether it is the preprocessed face or a new reference to
the source.

**Lifetime warning**, quoted in substance from the header: the preprocessed face
may contain sub-blobs that reference the memory backing the source
`hb_face_t`. If that memory is not owned by the source face, it must live at
least as long as the returned face.

**Notes** — Since HarfBuzz 6.0.0. See upstream's
`docs/subset-preprocessing.md` for the rationale.

### Subset plan: mappings and lifecycle

#### `hb_subset_plan_old_to_new_glyph_mapping`

```c
hb_map_t *hb_subset_plan_old_to_new_glyph_mapping (const hb_subset_plan_t *plan);
```

```rust
pub fn hb_subset_plan_old_to_new_glyph_mapping(plan: *const hb_subset_plan_t) -> *mut hb_map_t;
```

Returns the mapping between glyphs in the original font and glyphs in the subset
that will be produced by `plan`.

**Returns** — a pointer to the plan's `hb_map_t`. Keys are source glyph IDs,
values are subset glyph IDs. Available as soon as the plan exists — you do not
have to execute it first.

**Ownership** — `transfer none`. The map belongs to the plan; do not destroy it,
and do not use it after the plan's last reference is dropped.

**Notes** — Since HarfBuzz 4.0.0.

#### `hb_subset_plan_new_to_old_glyph_mapping`

```c
hb_map_t *hb_subset_plan_new_to_old_glyph_mapping (const hb_subset_plan_t *plan);
```

```rust
pub fn hb_subset_plan_new_to_old_glyph_mapping(plan: *const hb_subset_plan_t) -> *mut hb_map_t;
```

The reverse map: subset glyph ID → original glyph ID.

**Ownership** — `transfer none`, as above.

**Notes** — Since HarfBuzz 4.0.0.

#### `hb_subset_plan_unicode_to_old_glyph_mapping`

```c
hb_map_t *hb_subset_plan_unicode_to_old_glyph_mapping (const hb_subset_plan_t *plan);
```

```rust
pub fn hb_subset_plan_unicode_to_old_glyph_mapping(plan: *const hb_subset_plan_t) -> *mut hb_map_t;
```

Returns the mapping between codepoints in the original font and the associated
glyph ID **in the original font** — the `cmap` restricted to the requested
codepoints. Note that this is codepoint → *old* GID; compose it with the
old-to-new map if you want codepoint → new GID.

**Ownership** — `transfer none`.

**Notes** — Since HarfBuzz 4.0.0.

#### `hb_subset_plan_reference`

```c
hb_subset_plan_t *hb_subset_plan_reference (hb_subset_plan_t *plan);
```

```rust
pub fn hb_subset_plan_reference(plan: *mut hb_subset_plan_t) -> *mut hb_subset_plan_t;
```

Increases the reference count on `plan` and returns it. Every call must be
matched by an `hb_subset_plan_destroy()`. Upstream marks the function `(skip)`.
Since HarfBuzz 4.0.0.

#### `hb_subset_plan_destroy`

```c
void hb_subset_plan_destroy (hb_subset_plan_t *plan);
```

```rust
pub fn hb_subset_plan_destroy(plan: *mut hb_subset_plan_t);
```

Decreases the reference count on `plan`; when it reaches zero the plan is
destroyed and all its memory freed — including the three maps above. Since
HarfBuzz 4.0.0.

#### `hb_subset_plan_set_user_data`

```c
hb_bool_t hb_subset_plan_set_user_data (hb_subset_plan_t   *plan,
                                        hb_user_data_key_t *key,
                                        void               *data,
                                        hb_destroy_func_t   destroy,
                                        hb_bool_t           replace);
```

```rust
pub fn hb_subset_plan_set_user_data(
    plan: *mut hb_subset_plan_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

Attaches a user-data key/data pair to the given subset plan object. Semantics are
identical to `hb_subset_input_set_user_data()`: the *address* of `key` is what
identifies the slot, `destroy` is nullable and runs when the plan is destroyed or
the value replaced, and `replace` decides whether an existing entry is
overwritten. Returns `true` on success, `false` otherwise. Upstream marks it
`(skip)`. Since HarfBuzz 4.0.0.

#### `hb_subset_plan_get_user_data`

```c
void *hb_subset_plan_get_user_data (const hb_subset_plan_t *plan,
                                    hb_user_data_key_t     *key);
```

```rust
pub fn hb_subset_plan_get_user_data(
    plan: *const hb_subset_plan_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

Fetches the user data attached to the plan under the specified key. Returns null
when no entry is present. `transfer none` — ownership stays with whoever stored
it. Upstream marks it `(skip)`. Since HarfBuzz 4.0.0.

### The object-graph repacker

#### `hb_subset_serialize_or_fail`

```c
hb_blob_t *hb_subset_serialize_or_fail (hb_tag_t                      table_tag,
                                        hb_subset_serialize_object_t *hb_objects,
                                        unsigned                      num_hb_objs);
```

```rust
pub fn hb_subset_serialize_or_fail(
    table_tag: hb_tag_t,
    hb_objects: *mut hb_subset_serialize_object_t,
    num_hb_objs: c_uint,
) -> *mut hb_blob_t;
```

Given the input object-graph info, repacks a table to eliminate offset overflows
and serializes it into a continuous array of bytes.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `table_tag` | Tag of the table being packed, needed to allow table-specific optimizations (extension promotion in `GSUB`/`GPOS`, for instance). | Pass **`HB_TAG_NONE`** to disable table-specific optimizations. |
| `hb_objects` | Raw array of `hb_subset_serialize_object_t` describing the graph. | The implementation indexes it `num_hb_objs` times without a null check; treat null as forbidden unless `num_hb_objs` is 0. |
| `num_hb_objs` | Number of objects in the array. | — |

**Graph conventions** — the objects are pushed onto an internal vector *after* a
leading null placeholder, so link `objidx` values are **1-based** with respect to
the caller's array: `objidx = 1` names `hb_objects[0]`. The last object in the
array is treated as the root of the graph. The repacker is run with a maximum of
20 rounds and with the "recalculate extensions" behaviour enabled.

**Returns** — a new `hb_blob_t` holding the serialized table, or **null** if the
serializing attempt fails (unresolvable overflows, or allocation failure).

**Ownership** — the caller owns the blob and must release it with
`hb_blob_destroy()`. The input objects are read, not consumed: the `head`/`tail`
buffers and the link arrays stay the caller's, must stay valid for the duration
of the call, and are not freed by HarfBuzz. Note that `head` is `char *`, not
`const char *`, because the packer writes the resolved offsets back into the
caller's bytes.

**Notes** — Since HarfBuzz 10.2.0. This is the standalone entry point to the same
repacker the subsetter uses internally; see upstream's `docs/repacker.md` and
`docs/serializer.md`.

### Experimental: subset-input extras

Everything in this section needs the crate's `experimental` feature, which
defines `HB_EXPERIMENTAL_API`. Upstream marks each with `XSince: EXPERIMENTAL`
and makes no compatibility promise.

#### `hb_subset_input_to_string_or_fail` — needs `experimental`

```c
hb_blob_t *hb_subset_input_to_string_or_fail (hb_subset_input_t *input);
```

```rust
#[cfg(feature = "experimental")]
pub fn hb_subset_input_to_string_or_fail(input: *mut hb_subset_input_t) -> *mut hb_blob_t;
```

Produces a command-line string representation of the given subset input, suitable
for use with the `hb-subset` command-line tool — flags become `--no-hinting`,
`--retain-gids` and so on, sets become `--unicodes=…`, `--layout-features=…`,
`--drop-tables=…`, and configured axis ranges become `--variations=…`.

**Parameters** — `input`. **Null is tolerated** and returns null, as is an input
already in an error state.

**Returns** — a new `hb_blob_t` containing the command-line string, or **null**
on failure. The blob is created `HB_MEMORY_MODE_WRITABLE` over a `hb_malloc`ed
buffer with `hb_free` as its destroy callback.

**Ownership** — destroy with `hb_blob_destroy()`.

**Notes** — Primarily a debugging and reproduction aid: it lets you turn an
in-process configuration into a shell command you can rerun.

#### `hb_subset_input_override_name_table` — needs `experimental`

```c
hb_bool_t hb_subset_input_override_name_table (hb_subset_input_t *input,
                                               hb_ot_name_id_t    name_id,
                                               unsigned           platform_id,
                                               unsigned           encoding_id,
                                               unsigned           language_id,
                                               const char        *name_str,
                                               int                str_len);
```

```rust
#[cfg(feature = "experimental")]
pub fn hb_subset_input_override_name_table(
    input: *mut hb_subset_input_t,
    name_id: hb_ot_name_id_t,
    platform_id: c_uint,
    encoding_id: c_uint,
    language_id: c_uint,
    name_str: *const c_char,
    str_len: c_int,
) -> hb_bool_t;
```

Overrides the name string of the `name` record identified by `name_id`,
`platform_id`, `encoding_id`, and `language_id`. If a record with that identity
does not exist, it is created and inserted into the `name` table.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `name_id` | Name ID of the record. | — |
| `platform_id` | Platform ID of the record. | `1` is Macintosh and triggers the ASCII restriction below; `3` is Windows. |
| `encoding_id` | Encoding ID of the record. | — |
| `language_id` | Language ID of the record. | — |
| `name_str` | New value for the string. | **Null means "remove this record"** — pass null to delete. |
| `str_len` | Size of `name_str`, or **`-1`** when it is NUL-terminated. | Forced to 0 when `name_str` is null. |

**Returns** — `true` on success; `false` when allocation fails, and — the
surprising case — `false` when `platform_id == 1` and `name_str` contains any
non-ASCII character. Upstream: "for mac platform, we only support `name_str`
with all ascii characters, `name_str` with non-ascii characters will be ignored."

**Ownership** — the string is **copied** into the input; `name_str` need not
outlive the call.

**Notes** — the override is stored on the input and applied during subsetting,
so it composes with `HB_SUBSET_SETS_NAME_ID` — a record you override must also be
in the retained name-ID set to appear in the output.

#### Raw CFF outline access — needs `experimental`

Four accessors that hand back untouched bytes from a face's CFF tables. All four
take a source `hb_face_t` and are independent of any subset input or plan; they
are grouped here only because they live in `hb-subset.h` under a "Raw outline
data access" comment.

```c
hb_blob_t *hb_subset_cff_get_charstring_data   (hb_face_t *face, hb_codepoint_t glyph);
hb_blob_t *hb_subset_cff_get_charstrings_index (hb_face_t *face);
hb_blob_t *hb_subset_cff2_get_charstring_data  (hb_face_t *face, hb_codepoint_t glyph);
hb_blob_t *hb_subset_cff2_get_charstrings_index(hb_face_t *face);
```

```rust
#[cfg(feature = "experimental")]
pub fn hb_subset_cff_get_charstring_data(face: *mut hb_face_t, glyph: hb_codepoint_t) -> *mut hb_blob_t;
#[cfg(feature = "experimental")]
pub fn hb_subset_cff_get_charstrings_index(face: *mut hb_face_t) -> *mut hb_blob_t;
#[cfg(feature = "experimental")]
pub fn hb_subset_cff2_get_charstring_data(face: *mut hb_face_t, glyph: hb_codepoint_t) -> *mut hb_blob_t;
#[cfg(feature = "experimental")]
pub fn hb_subset_cff2_get_charstrings_index(face: *mut hb_face_t) -> *mut hb_blob_t;
```

- **`hb_subset_cff_get_charstring_data`** — returns the raw outline data from the
  `CFF ` table associated with the given glyph index.
- **`hb_subset_cff_get_charstrings_index`** — returns the raw `CharStrings INDEX`
  from the `CFF ` table.
- **`hb_subset_cff2_get_charstring_data`** — the `CFF2` equivalent of the first.
- **`hb_subset_cff2_get_charstrings_index`** — the `CFF2` equivalent of the
  second.

**Ownership** — each returns a blob the caller must release with
`hb_blob_destroy()`. The header does not document the failure behaviour; expect
the empty blob or null for a face with no CFF table and check the length.

**Notes** — these are the plumbing behind incremental-font-transfer experiments,
which need to move individual charstrings around without re-encoding them.

### Experimental: the glyph dependency graph

Everything in this section needs the crate's `experimental` feature. This is a
stronger requirement than elsewhere: `hb-config.hh` defines `HB_NO_SUBSET_DEPEND`
unless `HB_EXPERIMENTAL_API` is set, so without it these symbols are **not
compiled into the library at all**. The header's own warning: *"This API is
highly experimental and subject to change. It is not enabled by default and
should not be used in production code without understanding the stability
implications."*

A dependency graph records which glyphs reference or produce which other glyphs
through OpenType mechanisms — character mapping, glyph substitution, composite
construction, colour layering, math variants. It lets you find all glyphs
reachable from a given input set, which is useful for subsetting, coverage
analysis, font-delivery optimization, and working out which glyphs are needed to
render specific characters.

#### `hb_subset_depend_from_face_or_fail` — needs `experimental`

```c
hb_subset_depend_t *hb_subset_depend_from_face_or_fail (hb_face_t *face);
```

```rust
#[cfg(feature = "experimental")]
pub fn hb_subset_depend_from_face_or_fail(face: *mut hb_face_t) -> *mut hb_subset_depend_t;
```

Calculates the dependencies between glyphs in the supplied face. Dependency
information is extracted from the `GSUB`, `glyf`, `CFF `, `COLR`, and `MATH`
tables.

**UVS (Unicode Variation Sequence) dependencies are *not* included** — handle
those separately with `hb_font_get_variation_glyph()`. `VARC` is likewise not yet
covered.

**Parameters** — `face`: font face to collect dependencies from.

**Returns** — a new depend object, or **null** if creation failed (out of memory,
or an invalid face).

**Ownership** — `transfer full`: destroy with `hb_subset_depend_destroy()`. The
graph is computed eagerly at creation time and sized to the face's glyph count,
so building it is the expensive step.

**Notes** — Since HarfBuzz 14.3.0.

#### `hb_subset_depend_lookup_glyph` — needs `experimental`

```c
unsigned int hb_subset_depend_lookup_glyph (hb_subset_depend_t       *depend,
                                            hb_codepoint_t            gid,
                                            unsigned int              start_offset,
                                            unsigned int             *entry_count, /* IN/OUT */
                                            hb_subset_depend_entry_t *entries      /* OUT */);
```

```rust
#[cfg(feature = "experimental")]
pub fn hb_subset_depend_lookup_glyph(
    depend: *mut hb_subset_depend_t,
    gid: hb_codepoint_t,
    start_offset: c_uint,
    entry_count: *mut c_uint,
    entries: *mut hb_subset_depend_entry_t,
) -> c_uint;
```

Retrieves dependency edges for a glyph, following the standard HarfBuzz
array-getter pattern.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `depend` | The depend object. | — |
| `gid` | Glyph ID to retrieve dependencies from. | — |
| `start_offset` | Offset of the first entry to retrieve. | An offset at or past the total yields zero entries. |
| `entry_count` | In: number of entries to fill. Out: number actually filled. | Optional — pass null to query the total count without filling anything. |
| `entries` | Array to fill with dependency edge data. | Optional; **may be null only if `entry_count` is also null**. |

**Returns** — the **total** number of dependency edges for `gid`, always,
regardless of `start_offset` or `entry_count`.

**Ownership** — nothing is allocated; the caller owns `entries`.

**Notes** — Since HarfBuzz 14.3.0.

#### `hb_subset_depend_lookup_set` — needs `experimental`

```c
hb_bool_t hb_subset_depend_lookup_set (hb_subset_depend_t *depend,
                                       hb_codepoint_t      index,
                                       hb_set_t           *out /* OUT */);
```

```rust
#[cfg(feature = "experimental")]
pub fn hb_subset_depend_lookup_set(
    depend: *mut hb_subset_depend_t,
    index: hb_codepoint_t,
    out: *mut hb_set_t,
) -> hb_bool_t;
```

Gets all glyphs in the set identified by `index`, copying them into `out`.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `depend` | The depend object. | — |
| `index` | The set index, taken from the `ligature_set_index` or `context_set_index` field of an `hb_subset_depend_entry_t`. | `HB_CODEPOINT_INVALID` means "no set" and yields false. |
| `out` | A caller-owned set to copy into. | Its previous contents are **replaced**, not merged. |

**Returns** — `true` if there is such a set, `false` otherwise.

**Ownership** — `out` stays the caller's; create it with `hb_set_create()` and
release it with `hb_set_destroy()`.

**Notes** — Since HarfBuzz 14.3.0.

#### `hb_subset_depend_destroy` — needs `experimental`

```c
void hb_subset_depend_destroy (hb_subset_depend_t *depend);
```

```rust
#[cfg(feature = "experimental")]
pub fn hb_subset_depend_destroy(depend: *mut hb_subset_depend_t);
```

Decreases the reference count on `depend`; when it reaches zero the object is
destroyed and all its memory freed. Since HarfBuzz 14.3.0.

<!-- PART6 -->




