# Faces

Reference for `hb-face.h` — the font face object.

## Overview

A **face** is a single typeface selected out of a binary font file. Where a
font *family* is a design and a font *file* is a container, a face is one
concrete member: "Noto Sans Bold" as it exists inside a particular `.ttf` or
inside slot 3 of a `.ttc`. Faces sit in the middle of HarfBuzz's object stack.
Below them is `hb_blob_t`, which is just bytes plus a lifetime. Above them is
`hb_font_t`, which adds a point size, a scale, and variation coordinates. You
almost never shape with a face directly — you build a face, then build one or
more fonts from it, then shape with those.

The usual construction path is a blob plus a **face index**. The index selects
one face from a container that holds several: TrueType Collection (`.ttc`) and
Mac `dfont` files are the two that matter. Indices are zero-based. If the blob
is not a collection the index is ignored, and only the low 16 bits of the index
are ever used for face selection — the high 16 bits are reserved, and
`hb_font_create()` reads them to pick a *named instance* of a variable font.
`hb_face_get_index()` returns whatever you passed in, unmodified, high bits
included.

There is a second construction path for callers who do not have a contiguous
font file at all: `hb_face_create_for_tables()`. You supply a callback that
hands back one table at a time, and HarfBuzz asks for tables as it needs them.
This is how you wrap a font stored in a database, a resource bundle, or a
format HarfBuzz does not itself parse. The catch is that a table-callback face
cannot enumerate its own tables, so `hb_face_get_table_tags()` returns zero
unless you also install an `hb_get_table_tags_func_t` with
`hb_face_set_get_table_tags_func()`.

A third path runs the same machinery in reverse: `hb_face_builder_create()`
gives you an empty face you fill in with `hb_face_builder_add_table()`, and
`hb_face_reference_blob()` on that face serializes the accumulated tables into
a real binary font file. This is how HarfBuzz's own subsetter emits its output.

Faces are **reference counted**. Every function whose name contains `create`,
`reference`, or `get_empty` returns a face you own and must eventually pass to
`hb_face_destroy()`. Faces also carry a **mutability flag**: once
`hb_face_make_immutable()` is called, every setter on the face silently does
nothing. HarfBuzz makes a face immutable internally when a font is created from
it, which is why mutating a face after it has been used is not merely bad style
but a no-op.

Two families of creation function coexist for historical reasons.
`hb_face_create()` is the original and *never fails* — on error it returns the
singleton empty face, which parses as a valid but glyphless font.
`hb_face_create_or_fail()` and its siblings, added in HarfBuzz 10.1 and 11.0,
return `NULL` instead. New code should prefer the `_or_fail` variants; they are
the only way to distinguish "this file is not a font" from "this font has no
glyphs".

## Types

### `hb_face_t`

Opaque, reference-counted, heap-allocated. Holds the face's table data (or the
callback that produces it), its face index, and its cached upem and glyph
count. Only ever handled through a pointer.

In Rust this is an `opaque_handle!` type: zero-sized, unconstructible, `!Send`
and `!Sync`. You only ever hold `*mut hb_face_t` or `*const hb_face_t`.

Since HarfBuzz 0.9.2.

### `hb_reference_table_func_t`

```c
typedef hb_blob_t * (*hb_reference_table_func_t) (hb_face_t *face,
                                                  hb_tag_t   tag,
                                                  void      *user_data);
```

```rust
pub type hb_reference_table_func_t = Option<
    unsafe extern "C" fn(
        face: *mut hb_face_t,
        tag: hb_tag_t,
        user_data: *mut c_void,
    ) -> *mut hb_blob_t,
>;
```

The table-fetching callback for `hb_face_create_for_tables()`.

| Parameter | Meaning |
| --- | --- |
| `face` | The face the table is being fetched for. |
| `tag` | The four-byte table tag being requested. `HB_TAG_NONE` asks for the blob of the *whole face*, not a table. |
| `user_data` | The pointer passed to `hb_face_create_for_tables()`. |

**Return value.** A blob holding the table's bytes, with **ownership
transferred to HarfBuzz** — the callback must hand over a reference, and
HarfBuzz will destroy it. Return `NULL` when the table does not exist or cannot
be referenced.

**On the `HB_TAG_NONE` request.** If your backing store cannot produce the whole
font as one contiguous blob, return `NULL` for `HB_TAG_NONE` and install an
`hb_get_table_tags_func_t` on the face. `hb_face_reference_blob()` will then
enumerate the tags and assemble a face blob from the individual table blobs
itself.

Since HarfBuzz 0.9.2.

### `hb_get_table_tags_func_t`

```c
typedef unsigned int (*hb_get_table_tags_func_t) (const hb_face_t *face,
                                                  unsigned int     start_offset,
                                                  unsigned int    *table_count, /* IN/OUT */
                                                  hb_tag_t        *table_tags   /* OUT */,
                                                  void            *user_data);
```

```rust
pub type hb_get_table_tags_func_t = Option<
    unsafe extern "C" fn(
        face: *const hb_face_t,
        start_offset: c_uint,
        table_count: *mut c_uint,
        table_tags: *mut hb_tag_t,
        user_data: *mut c_void,
    ) -> c_uint,
>;
```

The table-enumerating callback behind `hb_face_get_table_tags()`.

| Parameter | Meaning |
| --- | --- |
| `face` | The face being enumerated. |
| `start_offset` | Index of the first tag to report — this is a paging API. |
| `table_count` | In: capacity of `table_tags`. Out: how many tags were actually written, possibly zero. |
| `table_tags` | Output array, length `*table_count`. Documented as nullable. |
| `user_data` | The pointer passed to `hb_face_set_get_table_tags_func()`. |

**Return value.** The *total* number of tables in the face — not the number
written this call — or zero if the face cannot be enumerated at all.

Since HarfBuzz 10.0.0.

## Functions

### Inspecting a blob before committing

#### `hb_face_count`

```c
unsigned int hb_face_count (hb_blob_t *blob);
```
```rust
pub fn hb_face_count(blob: *mut hb_blob_t) -> c_uint;
```

Fetches the number of faces in a blob: 1 for a plain `.ttf` or `.otf`, N for a
collection. Does not create a face and does not take a reference on `blob`.

Use it to bound a face-index loop, or to reject a file before paying for a
parse. Since HarfBuzz 1.7.7.

### Creation and destruction

#### `hb_face_create`

```c
hb_face_t * hb_face_create (hb_blob_t *blob, unsigned int index);
```
```rust
pub fn hb_face_create(blob: *mut hb_blob_t, index: c_uint) -> *mut hb_face_t;
```

Constructs a face from a blob and a face index.

* **Never returns null.** On any failure — unparseable data, index out of
  range, allocation failure — it returns the singleton empty face. The only way
  to detect this is to compare against `hb_face_get_empty()`, or to check
  `hb_face_is_immutable()` (the empty face is immutable from birth), or to use
  `hb_face_create_or_fail()` instead.
* **Reference semantics.** The face takes its own reference on `blob`; the
  caller keeps theirs and should still destroy it. The blob is *not* copied.
* **Ownership.** The caller owns the returned face and must call
  `hb_face_destroy()`.
* **Index handling.** Ignored for non-collections. Only the low 16 bits select
  a face. The full value is retrievable via `hb_face_get_index()`, and the high
  16 bits are later consumed by `hb_font_create()` to select a named instance.

Since HarfBuzz 0.9.2.

#### `hb_face_create_or_fail`

```c
hb_face_t * hb_face_create_or_fail (hb_blob_t *blob, unsigned int index);
```
```rust
pub fn hb_face_create_or_fail(blob: *mut hb_blob_t, index: c_uint) -> *mut hb_face_t;
```

As `hb_face_create()`, but **returns `NULL`** when the blob contains no usable
face at `index`. Ownership and blob-reference behaviour are otherwise
identical. Prefer this over `hb_face_create()` in new code.

Since HarfBuzz 10.1.0.

#### `hb_face_create_or_fail_using`

```c
hb_face_t * hb_face_create_or_fail_using (hb_blob_t    *blob,
                                          unsigned int  index,
                                          const char   *loader_name);
```
```rust
pub fn hb_face_create_or_fail_using(
    blob: *mut hb_blob_t,
    index: c_uint,
    loader_name: *const c_char,
) -> *mut hb_face_t;
```

Creates a face through a **named face loader**. Pass `NULL` or `""` for
`loader_name` to take the first available loader — `loader_name` is explicitly
nullable.

Loaders differ in what formats they accept. The header calls out the example:
the FreeType loader (`"ft"`) can load WOFF and WOFF2 when FreeType was built
with those features, while the OpenType loader (`"ot"`) cannot. Enumerate the
available names with `hb_face_list_loaders()`.

Returns `NULL` if the loader fails to load the face. The caller owns a non-null
result and must destroy it. Since HarfBuzz 11.0.0.

#### `hb_face_create_from_file_or_fail`

```c
hb_face_t * hb_face_create_from_file_or_fail (const char *file_name, unsigned int index);
```
```rust
pub fn hb_face_create_from_file_or_fail(
    file_name: *const c_char,
    index: c_uint,
) -> *mut hb_face_t;
```

A thin wrapper around `hb_blob_create_from_file_or_fail()` followed by
`hb_face_create_or_fail()`. The intermediate blob is memory-mapped where the
platform allows it, and is owned by the returned face.

Returns `NULL` if the file cannot be read *or* if it holds no face at `index` —
the two failures are not distinguishable from the return value alone. The
caller owns a non-null result and must destroy it. `file_name` is a
NUL-terminated path in the platform's filesystem encoding.

Since HarfBuzz 10.1.0.

#### `hb_face_create_from_file_or_fail_using`

```c
hb_face_t * hb_face_create_from_file_or_fail_using (const char   *file_name,
                                                    unsigned int  index,
                                                    const char   *loader_name);
```
```rust
pub fn hb_face_create_from_file_or_fail_using(
    file_name: *const c_char,
    index: c_uint,
    loader_name: *const c_char,
) -> *mut hb_face_t;
```

The file-path counterpart of `hb_face_create_or_fail_using()`. `loader_name` is
nullable with the same meaning. Returns `NULL` if the file cannot be read or
the loader fails. Since HarfBuzz 11.0.0.

#### `hb_face_create_for_tables`

```c
hb_face_t * hb_face_create_for_tables (hb_reference_table_func_t  reference_table_func,
                                       void                      *user_data,
                                       hb_destroy_func_t          destroy);
```
```rust
pub fn hb_face_create_for_tables(
    reference_table_func: hb_reference_table_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
) -> *mut hb_face_t;
```

Creates a face that pulls tables through a callback rather than parsing a
contiguous font file.

* `destroy` is invoked on `user_data` when the face no longer needs it — which
  is at face destruction, so `user_data` must outlive every use of the face.
  `destroy` is nullable.
* The header warns that `hb_face_get_table_tags()` **does not work** on a face
  made this way. Fix that by calling `hb_face_set_get_table_tags_func()` on the
  result.
* The caller owns the returned face and must destroy it.

Since HarfBuzz 0.9.2.

#### `hb_face_get_empty`

```c
hb_face_t * hb_face_get_empty (void);
```
```rust
pub fn hb_face_get_empty() -> *mut hb_face_t;
```

Fetches the singleton empty face: no tables, no glyphs, immutable, upem 1000.
Documented as `transfer full`, so the returned pointer may be passed to
`hb_face_destroy()` like any other face; the singleton is inert and is never
actually freed. Useful as a non-null placeholder and as the value
`hb_face_create()` returns on failure.

Since HarfBuzz 0.9.2.

#### `hb_face_reference` / `hb_face_destroy`

```c
hb_face_t * hb_face_reference (hb_face_t *face);
void        hb_face_destroy   (hb_face_t *face);
```
```rust
pub fn hb_face_reference(face: *mut hb_face_t) -> *mut hb_face_t;
pub fn hb_face_destroy(face: *mut hb_face_t);
```

Increment and decrement the reference count. `hb_face_reference()` returns the
same pointer it was given, which makes it convenient in expressions.
`hb_face_destroy()` frees the face and all of its memory once the count hits
zero — including releasing its reference on the backing blob and invoking any
`destroy` callback registered for table or table-tag user data.

Since HarfBuzz 0.9.2.

### User data

#### `hb_face_set_user_data`

```c
hb_bool_t hb_face_set_user_data (hb_face_t          *face,
                                 hb_user_data_key_t *key,
                                 void               *data,
                                 hb_destroy_func_t   destroy,
                                 hb_bool_t           replace);
```
```rust
pub fn hb_face_set_user_data(
    face: *mut hb_face_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

Attaches an arbitrary pointer to the face under `key`. HarfBuzz uses the
*address* of the `hb_user_data_key_t` as the identity, never its contents, so
the key must be a stable long-lived object — typically a `static`.

`destroy` (nullable) is called on `data` when the face is destroyed or the
entry is replaced. `replace` decides whether an existing entry under the same
key is overwritten; with `replace` false and an entry present, the call fails.

Returns true on success, false otherwise. Since HarfBuzz 0.9.2.

#### `hb_face_get_user_data`

```c
void * hb_face_get_user_data (const hb_face_t *face, hb_user_data_key_t *key);
```
```rust
pub fn hb_face_get_user_data(
    face: *const hb_face_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

Fetches the pointer stored under `key`, or `NULL` if there is none. The face
retains ownership — do not free the result. Note that `face` is `const` but
`key` is not, an asymmetry inherited from the C API.

Since HarfBuzz 0.9.2.

### Mutability

#### `hb_face_make_immutable` / `hb_face_is_immutable`

```c
void      hb_face_make_immutable (hb_face_t *face);
hb_bool_t hb_face_is_immutable   (hb_face_t *face);
```
```rust
pub fn hb_face_make_immutable(face: *mut hb_face_t);
pub fn hb_face_is_immutable(face: *mut hb_face_t) -> hb_bool_t;
```

Freezes a face, and tests whether it is frozen. Once immutable, every setter
(`hb_face_set_index`, `hb_face_set_upem`, `hb_face_set_glyph_count`,
`hb_face_set_get_table_tags_func`) becomes a silent no-op — they return `void`,
so there is no error to check. Immutability is one-way; there is no
corresponding "make mutable".

Note that `hb_face_is_immutable()` takes a non-const pointer even though it only
reads.

Since HarfBuzz 0.9.2.

### Table access

#### `hb_face_reference_table`

```c
hb_blob_t * hb_face_reference_table (const hb_face_t *face, hb_tag_t tag);
```
```rust
pub fn hb_face_reference_table(face: *const hb_face_t, tag: hb_tag_t) -> *mut hb_blob_t;
```

Fetches a blob covering one table of the face.

* **Never returns null.** A missing table, or a table that cannot be
  referenced, yields the *empty blob*. Test with `hb_blob_get_length()`, not
  against `NULL`.
* **Ownership.** The caller owns the returned blob and must call
  `hb_blob_destroy()`. The blob typically points into the face's own memory
  rather than copying, and holds a reference that keeps that memory alive.

Since HarfBuzz 0.9.2.

#### `hb_face_reference_blob`

```c
hb_blob_t * hb_face_reference_blob (hb_face_t *face);
```
```rust
pub fn hb_face_reference_blob(face: *mut hb_face_t) -> *mut hb_blob_t;
```

Fetches a blob for the *entire* face as a binary font file.

For a face built from a blob this is essentially free — it references the
original data. For a face built from table callbacks, HarfBuzz asks the
reference-table callback for `HB_TAG_NONE`; if that fails, it **serializes** a
new font file from the individual table blobs, which requires that
`hb_face_get_table_tags()` work on the face. If neither route succeeds, the
empty blob is returned. Never null.

This is also the "compile" step for builder faces — see
`hb_face_builder_create()`.

The caller owns the returned blob and must call `hb_blob_destroy()`. Since
HarfBuzz 0.9.2.

### Face properties

#### `hb_face_set_index` / `hb_face_get_index`

```c
void         hb_face_set_index (hb_face_t *face, unsigned int index);
unsigned int hb_face_get_index (const hb_face_t *face);
```
```rust
pub fn hb_face_set_index(face: *mut hb_face_t, index: c_uint);
pub fn hb_face_get_index(face: *const hb_face_t) -> c_uint;
```

Get and set the recorded face index. The setter is a no-op on an immutable
face, and the header is emphatic that **changing the index does not change the
face** — it only changes what the getter reports. The face's actual table data
was fixed at creation time. Indices within a collection are zero-based.

Since HarfBuzz 0.9.2.

#### `hb_face_set_upem` / `hb_face_get_upem`

```c
void         hb_face_set_upem (hb_face_t *face, unsigned int upem);
unsigned int hb_face_get_upem (const hb_face_t *face);
```
```rust
pub fn hb_face_set_upem(face: *mut hb_face_t, upem: c_uint);
pub fn hb_face_get_upem(face: *const hb_face_t) -> c_uint;
```

Get and set units-per-em, the design grid the face's outlines are drawn on.
Typical values are 1000 (PostScript-flavoured) and 2048 (TrueType-flavoured);
OpenType permits anything from 16 to 16,384.

The getter reads the value from the `head` table on first use and caches it.
The setter exists for faces whose table data does not carry a usable `head`,
and the header describes it as needed "in rare circumstances". It is a no-op on
an immutable face.

Since HarfBuzz 0.9.2.

#### `hb_face_set_glyph_count` / `hb_face_get_glyph_count`

```c
void         hb_face_set_glyph_count (hb_face_t *face, unsigned int glyph_count);
unsigned int hb_face_get_glyph_count (const hb_face_t *face);
```
```rust
pub fn hb_face_set_glyph_count(face: *mut hb_face_t, glyph_count: c_uint);
pub fn hb_face_get_glyph_count(face: *const hb_face_t) -> c_uint;
```

Get and set the number of glyphs in the face. The getter derives it from the
`maxp` table and caches it; valid glyph IDs are `0 ..= glyph_count - 1`. Like
the upem setter, the setter is for rare cases and is a no-op on an immutable
face.

Since HarfBuzz 0.9.7.

### Table enumeration

#### `hb_face_set_get_table_tags_func`

```c
void hb_face_set_get_table_tags_func (hb_face_t                *face,
                                      hb_get_table_tags_func_t  func,
                                      void                     *user_data,
                                      hb_destroy_func_t         destroy);
```
```rust
pub fn hb_face_set_get_table_tags_func(
    face: *mut hb_face_t,
    func: hb_get_table_tags_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Installs the table-enumerating callback. `destroy` (nullable) is called on
`user_data` when `func` is no longer needed. Needed only for faces created with
`hb_face_create_for_tables()`; blob-backed faces already know their tables.
No-op on an immutable face.

Since HarfBuzz 10.0.0.

#### `hb_face_get_table_tags`

```c
unsigned int hb_face_get_table_tags (const hb_face_t *face,
                                     unsigned int     start_offset,
                                     unsigned int    *table_count, /* IN/OUT */
                                     hb_tag_t        *table_tags   /* OUT */);
```
```rust
pub fn hb_face_get_table_tags(
    face: *const hb_face_t,
    start_offset: c_uint,
    table_count: *mut c_uint,
    table_tags: *mut hb_tag_t,
) -> c_uint;
```

Retrieves a page of the face's table tags, starting at `start_offset`.
`*table_count` is the buffer capacity going in and the number written coming
out. `table_tags` is documented as nullable — pass `NULL` with a zeroed
`table_count` to query only the total.

Returns the **total** number of tables in the face, or **zero** if the tables
cannot be listed at all — the case for a `hb_face_create_for_tables()` face
with no table-tags callback installed.

Since HarfBuzz 1.6.0.

### Character coverage

These four fill caller-provided `hb_set_t` / `hb_map_t` objects. None of them
clear the output first, so they *add* to whatever is already there — pass a
fresh set if you want only this face's coverage. The face is not modified
despite the non-const pointer.

#### `hb_face_collect_unicodes`

```c
void hb_face_collect_unicodes (hb_face_t *face, hb_set_t *out);
```
```rust
pub fn hb_face_collect_unicodes(face: *mut hb_face_t, out: *mut hb_set_t);
```

Adds every Unicode codepoint the face covers to `out`. Since HarfBuzz 1.9.0.

#### `hb_face_collect_nominal_glyph_mapping`

```c
void hb_face_collect_nominal_glyph_mapping (hb_face_t *face,
                                            hb_map_t  *mapping,
                                            hb_set_t  *unicodes);
```
```rust
pub fn hb_face_collect_nominal_glyph_mapping(
    face: *mut hb_face_t,
    mapping: *mut hb_map_t,
    unicodes: *mut hb_set_t,
);
```

Adds the face's codepoint-to-nominal-glyph mapping to `mapping`, and optionally
the covered codepoints to `unicodes`. **`unicodes` is nullable**; pass `NULL`
if you only want the map. Cheaper than calling `hb_face_collect_unicodes()`
separately when you need both.

"Nominal" here means the plain `cmap` mapping, before any shaping, ligature
substitution, or variation-selector handling.

Since HarfBuzz 7.0.0.

#### `hb_face_collect_variation_selectors`

```c
void hb_face_collect_variation_selectors (hb_face_t *face, hb_set_t *out);
```
```rust
pub fn hb_face_collect_variation_selectors(face: *mut hb_face_t, out: *mut hb_set_t);
```

Adds every Unicode Variation Selector the face has a `cmap` format-14 entry for
to `out`. Since HarfBuzz 1.9.0.

#### `hb_face_collect_variation_unicodes`

```c
void hb_face_collect_variation_unicodes (hb_face_t      *face,
                                         hb_codepoint_t  variation_selector,
                                         hb_set_t       *out);
```
```rust
pub fn hb_face_collect_variation_unicodes(
    face: *mut hb_face_t,
    variation_selector: hb_codepoint_t,
    out: *mut hb_set_t,
);
```

Adds every base codepoint the face supports **in combination with**
`variation_selector` to `out`. Pair it with
`hb_face_collect_variation_selectors()` to walk the whole variation-sequence
table.

Since HarfBuzz 1.9.0.

### Builder faces

#### `hb_face_builder_create`

```c
hb_face_t * hb_face_builder_create (void);
```
```rust
pub fn hb_face_builder_create() -> *mut hb_face_t;
```

Creates an empty face to be assembled table by table. Add tables with
`hb_face_builder_add_table()`, then call `hb_face_reference_blob()` on the face
to compile everything into a binary font file.

The result is a normal `hb_face_t` — destroy it with `hb_face_destroy()` — but
only builder faces accept the two functions below. Since HarfBuzz 1.9.0.

#### `hb_face_builder_add_table`

```c
hb_bool_t hb_face_builder_add_table (hb_face_t *face, hb_tag_t tag, hb_blob_t *blob);
```
```rust
pub fn hb_face_builder_add_table(
    face: *mut hb_face_t,
    tag: hb_tag_t,
    blob: *mut hb_blob_t,
) -> hb_bool_t;
```

Adds the table `tag`, with contents `blob`, to a builder face.

* `face` **must** come from `hb_face_builder_create()`. Called on any other
  face it returns false and does nothing.
* The builder takes its own reference on `blob`; the caller keeps theirs. The
  data is not copied.
* Adding a tag that is already present replaces the previous blob.
* Returns false on allocation failure or an invalid tag.

Since HarfBuzz 1.9.0.

#### `hb_face_builder_sort_tables`

```c
void hb_face_builder_sort_tables (hb_face_t *face, const hb_tag_t *tags);
```
```rust
pub fn hb_face_builder_sort_tables(face: *mut hb_face_t, tags: *const hb_tag_t);
```

Sets the order in which tables are written when a builder face is serialized.
`tags` is an array terminated by `HB_TAG_NONE`. Tables not named in `tags` are
written after those that are, in HarfBuzz's default order. Returns `void` and
silently does nothing if `face` is not a builder face.

Useful when a downstream consumer expects a particular physical table order —
some font validators and some legacy rasterizers do.

Since HarfBuzz 5.3.0.

## Usage notes

### Two failure conventions in one header

This header mixes both HarfBuzz failure styles, and the difference is easy to
miss:

| Function | On failure |
| --- | --- |
| `hb_face_create` | Returns the empty face — never null |
| `hb_face_create_for_tables` | Returns a face (empty on allocation failure) |
| `hb_face_builder_create` | Returns a face (empty on allocation failure) |
| `hb_face_get_empty` | n/a |
| `hb_face_create_or_fail` | Returns `NULL` |
| `hb_face_create_or_fail_using` | Returns `NULL` |
| `hb_face_create_from_file_or_fail` | Returns `NULL` |
| `hb_face_create_from_file_or_fail_using` | Returns `NULL` |
| `hb_face_reference_table` | Returns the empty blob — never null |
| `hb_face_reference_blob` | Returns the empty blob — never null |

The empty-object convention means a null check on `hb_face_create()` is dead
code. Validate with `hb_face_get_glyph_count() > 0`, or just use an `_or_fail`
constructor.

### The blob is referenced, not copied

```c
hb_blob_t *blob = hb_blob_create_from_file_or_fail ("Font.ttf");
hb_face_t *face = hb_face_create_or_fail (blob, 0);
hb_blob_destroy (blob);   /* correct: the face holds its own reference */
```

Destroying the blob immediately after creating the face is not only safe but
idiomatic. The reverse — freeing the *underlying memory* a blob was built over
while a face still lives — is a use-after-free, because nothing was copied.
Blobs created with `HB_MEMORY_MODE_READONLY` over caller-owned memory are the
usual way to get this wrong.

### Enumerating the faces in a collection

```c
unsigned int n = hb_face_count (blob);
for (unsigned int i = 0; i < n; i++)
{
  hb_face_t *face = hb_face_create_or_fail (blob, i);
  if (!face) continue;
  /* ... */
  hb_face_destroy (face);
}
```

Note the index passed here is a *face* index. If you also want a named instance
of a variable font, that goes in the **high** 16 bits and is interpreted by
`hb_font_create()`, not by the face.

### Paging through table tags

`hb_face_get_table_tags()` is a paging API; the return value is the total, and
`*table_count` is what actually fits.

```c
hb_tag_t tags[32];
unsigned int offset = 0, count, total;
do {
  count = sizeof (tags) / sizeof (tags[0]);
  total = hb_face_get_table_tags (face, offset, &count, tags);
  for (unsigned int i = 0; i < count; i++) { /* ... */ }
  offset += count;
} while (count && offset < total);
```

Guard on `count` as well as `offset < total`: a return of zero means "cannot
enumerate", and looping on `total` alone would spin forever.

### Immutability is silent

Every setter here returns `void`. There is no way to learn that
`hb_face_set_upem()` was ignored except by calling `hb_face_is_immutable()`
first, or by reading the value back. HarfBuzz freezes a face when a font is
created from it, so the practical rule is: **finish configuring a face before
you create any font from it.**

### Threading

The header says nothing about thread safety, so treat these rules as the
minimum HarfBuzz's object model guarantees rather than as documented promises.
Reference counting is atomic, so `hb_face_reference()` and `hb_face_destroy()`
may be called from any thread. An *immutable* face is safe to read from several
threads concurrently — this is the intended way to share one face across worker
threads. A mutable face is not: concurrent setters, or a setter racing a
getter, are data races. Make a face immutable before publishing it to other
threads.

`hb_face_set_user_data()` and `hb_face_get_user_data()` are internally locked
and remain usable on an immutable face.

### Round-tripping a font through the builder

```c
hb_face_t *builder = hb_face_builder_create ();

hb_blob_t *head = hb_face_reference_table (src, HB_TAG ('h','e','a','d'));
hb_face_builder_add_table (builder, HB_TAG ('h','e','a','d'), head);
hb_blob_destroy (head);
/* ... more tables ... */

hb_blob_t *font_file = hb_face_reference_blob (builder);  /* serializes */
/* write font_file to disk */
hb_blob_destroy (font_file);
hb_face_destroy (builder);
```

`hb_face_reference_blob()` is the compile step; until you call it, no font file
exists. Calling it does not consume the builder — you can add more tables and
compile again.

### What the header does not say

* Whether `blob` may be `NULL` in `hb_face_create()` is unspecified in the
  header. (The implementation substitutes the empty blob, but that is not part
  of the documented contract.)
* Whether `face` may be `NULL` in the accessors is unspecified. HarfBuzz's
  general convention is that passing `NULL` where an object is expected is a
  programming error, except for `destroy` functions.
* The lifetime of the strings returned by `hb_face_list_loaders()` is
  documented only as "owned by HarfBuzz" — treat them as valid for the process
  lifetime and never free them or the array.
* Nothing in the header states whether the coverage collectors clear their
  output arguments. They do not, but you should pass fresh containers rather
  than rely on it.

### Macros not transcribed

The header defines no function-like macros and no value macros; nothing was
skipped. The `HB_EXTERN`, `HB_BEGIN_DECLS`, and `HB_END_DECLS` tokens are
linkage plumbing from `hb-common.h` with no Rust equivalent.
