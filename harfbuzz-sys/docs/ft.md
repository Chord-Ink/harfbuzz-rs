# FreeType integration

Header: `hb-ft.h` — Rust module: `harfbuzz_sys::ft`, gated on the crate's
`freetype` feature and **not** glob re-exported at the crate root. Upstream
gtk-doc section: `hb-ft`, short description *"FreeType integration"*.

## Overview

FreeType is the font engine that loads, scales, hints and rasterizes fonts for
most of the free-software stack. HarfBuzz does not need it — it has its own
OpenType table readers in `hb-ot-font` — but a program that already renders
with FreeType usually wants HarfBuzz's shaping to agree, glyph for glyph and
unit for unit, with what FreeType is about to draw. `hb-ft.h` is the bridge.

The header does two separable things, and conflating them is the single most
common source of confusion:

1. **Face data from an `FT_Face`.** The `hb_ft_face_create*` family wraps an
   `FT_Face` in an `hb_face_t`. HarfBuzz uses the `FT_Face` purely as a way to
   reach the raw SFNT tables — either by pointing directly at the memory-mapped
   stream (`ft_face->stream->base`) when FreeType has one, or by installing a
   table-reference callback that calls `FT_Load_Sfnt_Table` on demand. The
   practical payoff is format coverage: FreeType can open WOFF, WOFF2, Type 1,
   CFF and other containers that `hb_face_create` cannot. **A font built on
   such a face still uses HarfBuzz's own native font implementation** for
   advances, extents and outlines, unless you take step 2.
2. **Font functions from FreeType.** The `hb_ft_font_create*` family, and
   `hb_ft_font_set_funcs` on a font you already have, install FreeType's
   callback table (`hb_font_funcs_t`) on an `hb_font_t`. From then on every
   `hb_font_get_*` question is answered by `FT_Load_Glyph` and friends, so
   HarfBuzz sees exactly the advances FreeType computes — including hinting,
   FreeType's transform, and its bitmap-strike selection for colour emoji.

Each family offers the same three lifecycle policies, which is what the variant
names mean:

| Variant | `FT_Face` lifetime | Who destroys the `FT_Face` |
| --- | --- | --- |
| `hb_ft_face_create` / `hb_ft_font_create` | not managed | you, *after* the HarfBuzz object is destroyed |
| `hb_ft_face_create_referenced` / `hb_ft_font_create_referenced` | `FT_Reference_Face` on entry, `FT_Done_Face` on destruction | HarfBuzz drops its reference; you drop yours whenever you like |
| `hb_ft_face_create_cached` | not managed, but the `hb_face_t` is memoized in `ft_face->generic` | you, after the last `hb_face_t` is destroyed |

Upstream's advice is blunt and worth repeating: use the `_referenced` variants
unless you know exactly why you should not. The plain `_create` variants exist
for callers who manage the `FT_Face` themselves and predate reference counting;
passing `NULL` as their `destroy` argument is a strong hint that you actually
wanted `_referenced`.

Because both objects carry mutable state — an `hb_font_t` has a scale and
variation coordinates, an `FT_Face` has a char size, a transform and design
coordinates — they can drift apart. `hb_ft_font_changed` copies the `FT_Face`'s
state into the font (call it after `FT_Set_Char_Size` or
`FT_Set_Var_Design_Coordinates`); `hb_ft_hb_font_changed` copies the font's
state back into the `FT_Face` (call it after `hb_font_set_scale` or
`hb_font_set_variations`). Since HarfBuzz 11.0.0 the second direction happens
automatically inside every glyph callback, so it is now a legacy call.

Finally, the header's own warning, first sentence of the file: *"Note: FreeType
is not thread-safe. Hence, these functions are not either."* The FreeType-backed
callbacks do serialize access to their `FT_Face` behind an internal mutex — the
same mutex `hb_ft_font_lock_face` hands you — but the `FT_Library`, and any
`FT_Face` you own and mutate yourself, are not protected by anything HarfBuzz
does.

In this crate the whole module exists only when the `freetype` feature is
enabled, because upstream compiles `hb-ft.cc` only under `HAVE_FREETYPE` and
the crate's `build.rs` resolves the system `freetype2` library through
`pkg-config` for that feature. There is no runtime probe: if the feature is off,
the symbols are simply absent.

## Types

`hb-ft.h` declares no types, no constants and no macros of its own. Everything
it names comes either from HarfBuzz's core headers or from FreeType.

### `FT_Face` (from FreeType)

```c
typedef struct FT_FaceRec_ *FT_Face;   /* freetype.h */
```

```rust
crate::opaque_handle! { FT_FaceRec_ }
pub type FT_Face = *mut FT_FaceRec_;
```

FreeType's handle to a face: one typeface out of a font file, **plus** the
currently selected size, charmap, transform and variation coordinates. Unlike
an `hb_face_t`, it is mutable and stateful, and that state is what most of this
header is about synchronizing.

`hb-ft.h` gets this type by `#include`ing `<ft2build.h>` and `FT_FREETYPE_H`, so
it belongs to FreeType, not to HarfBuzz. `harfbuzz-sys` has no FreeType
dependency to import it from, so the module declares the pointee as an opaque
handle and `FT_Face` as a pointer alias. This is ABI-identical to FreeType's own
`typedef` — both are a thin pointer — so a handle from any FreeType binding
converts with a plain pointer cast:

```rust
let hb_ft_face = their_ft_face as harfbuzz_sys::ft::FT_Face;
```

Fields of `FT_FaceRec_` are deliberately not transcribed. If you need
`units_per_EM`, `num_fixed_sizes`, `glyph`, `size` or `generic`, read them
through a real FreeType binding.

Two members do matter conceptually, because HarfBuzz touches them:

* `size` — the currently selected size. **HarfBuzz dereferences it
  unconditionally** when creating a font, so a face with no size set is a null
  dereference, not a graceful default.
* `generic` — a `void *data` plus a finalizer, provided by FreeType for client
  use. `hb_ft_face_create_cached` claims it for its cache;
  `hb_ft_face_create_from_file_or_fail` and
  `hb_ft_face_create_from_blob_or_fail` claim it to hold their internal
  `FT_Library`.

### `hb_face_t` (from `hb-face.h`)

The immutable, reference-counted HarfBuzz view of a typeface: a blob of font
data plus a face index, upem, and lazily parsed tables. Returned by the five
`hb_ft_face_create*` functions; release with `hb_face_destroy`. See
`docs/face.md`.

### `hb_font_t` (from `hb-font.h`)

A face at a particular scale, with variation coordinates and a table of font
functions. Returned by `hb_ft_font_create*`; every other function in this header
takes one as its first argument. Release with `hb_font_destroy`. See
`docs/font.md`.

An `hb_font_t` is *FreeType-backed* if its font data was installed by this
header — that is, if it came from `hb_ft_font_create` or
`hb_ft_font_create_referenced`, or had `hb_ft_font_set_funcs` called on it.
**Ten of the functions here check that and silently do nothing (or return
`NULL`/`0`/`false`) for any other font.** There is no predicate to test it with;
you have to know.

### `hb_blob_t` (from `hb-blob.h`)

Reference-counted binary data. Only `hb_ft_face_create_from_blob_or_fail` takes
one. See `docs/blob.md`.

### `hb_destroy_func_t` (from `hb-common.h`)

```c
typedef void (*hb_destroy_func_t) (void *user_data);
```

```rust
pub type hb_destroy_func_t = Option<unsafe extern "C" fn(user_data: *mut c_void)>;
```

The `destroy` argument of `hb_ft_face_create` and `hb_ft_font_create`. Note the
unusual contract in this header: HarfBuzz calls it **with the `FT_Face`** as
`user_data`, not with a pointer you chose.

## Functions

### Faces from an existing `FT_Face`

#### `hb_ft_face_create`

```c
hb_face_t *hb_ft_face_create (FT_Face ft_face, hb_destroy_func_t destroy);
```

```rust
pub fn hb_ft_face_create(ft_face: FT_Face, destroy: hb_destroy_func_t) -> *mut hb_face_t;
```

Creates an `hb_face_t` that reads its tables from `ft_face`. This variant
provides no lifecycle management whatsoever.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `ft_face` | `FT_Face` | `FT_Face` | The FreeType face to read font data from. Required; the implementation dereferences `ft_face->stream` immediately, so null is a crash. |
| `destroy` | `hb_destroy_func_t` | `hb_destroy_func_t` | Nullable. Called when the returned face is destroyed, with `ft_face` as its `user_data` argument. |

**Returns** — the new face. Upstream annotates it `(transfer full)` and
documents no failure value; the code path goes through `hb_face_create` /
`hb_face_create_for_tables`, which return the immutable empty face rather than
null on allocation failure, so treat a non-null-but-empty result (zero tables,
`hb_face_get_glyph_count` of 0) as the failure signal.

**Ownership**

* The returned face carries one reference. Destroy it with `hb_face_destroy`.
* `ft_face` is **borrowed, not referenced**. HarfBuzz never calls
  `FT_Done_Face`. You must keep it alive for at least as long as the face — and
  as long as anything derived from it — and destroy it yourself afterwards.
* `destroy` does not change that: it is a *notification*, invoked with
  `ft_face`, telling you HarfBuzz is finished. You may call `FT_Done_Face` from
  inside it, which is exactly what `hb_ft_face_create_referenced` does.
* Which internal strategy is used depends on the `FT_Face`: if
  `ft_face->stream->read` is null (a memory-resident stream, the common
  `FT_New_Memory_Face` case) HarfBuzz wraps the stream's bytes in a
  `HB_MEMORY_MODE_READONLY` blob and never copies them; otherwise it installs a
  table-reference callback that calls `FT_Load_Sfnt_Table` per table and copies
  each one. Either way the bytes stay owned by FreeType.

**Notes**

* Since HarfBuzz 0.9.2.
* The face's index and upem are taken from `ft_face->face_index` and
  `ft_face->units_per_EM`.
* Fonts created from this face use HarfBuzz's *native* font implementation. Call
  `hb_ft_font_set_funcs` on them if you want FreeType metrics.
* Not thread-safe (FreeType is not).
* Upstream: *"Most often you don't want this function."*

#### `hb_ft_face_create_cached`

```c
hb_face_t *hb_ft_face_create_cached (FT_Face ft_face);
```

```rust
pub fn hb_ft_face_create_cached(ft_face: FT_Face) -> *mut hb_face_t;
```

Like `hb_ft_face_create`, but memoizes the result on the `FT_Face` itself, so
repeated calls with the same `ft_face` hand back the same `hb_face_t`.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `ft_face` | `FT_Face` | `FT_Face` | The FreeType face. Required; `ft_face->generic` is read and written. |

**Returns** — the cached face, `(transfer full)`, with a fresh reference added
for the caller by `hb_face_reference`. Same failure caveat as
`hb_ft_face_create`.

**Ownership**

* Each call returns a reference you must release with `hb_face_destroy`.
* The cache itself holds one further reference, stored in `ft_face->generic.data`
  with `hb_ft_face_finalize` as `ft_face->generic.finalizer`. FreeType runs that
  finalizer from `FT_Done_Face`, which is what finally destroys the cached face.
* `ft_face` is borrowed. You must destroy it, and only after the last
  `hb_face_t` derived from it is gone.
* **It takes over `ft_face->generic`.** If that field already holds something
  else, the existing finalizer is invoked immediately (destroying whatever it
  owned) and then overwritten.

**Notes**

* Since HarfBuzz 0.9.2.
* The internal `hb_ft_face_create` call passes a null `destroy`, so nothing
  releases the `FT_Face`.
* No locking: two threads calling this concurrently on one `FT_Face` will race
  on `generic`.

#### `hb_ft_face_create_referenced`

```c
hb_face_t *hb_ft_face_create_referenced (FT_Face ft_face);
```

```rust
pub fn hb_ft_face_create_referenced(ft_face: FT_Face) -> *mut hb_face_t;
```

Like `hb_ft_face_create`, but takes a FreeType reference so the `FT_Face`
cannot be released too early. This is the variant to use.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `ft_face` | `FT_Face` | `FT_Face` | The FreeType face. Required. |

**Returns** — the new face, `(transfer full)`. Same failure caveat as
`hb_ft_face_create`.

**Ownership**

* Calls `FT_Reference_Face(ft_face)` on entry and passes `FT_Done_Face` as the
  destroy callback, so the `FT_Face` outlives the `hb_face_t` automatically.
* You still own your own `FT_Face` reference and should `FT_Done_Face` it when
  *you* are finished — the point is that the order no longer matters.
* Destroy the returned face with `hb_face_destroy`.

**Notes**

* Since HarfBuzz 0.9.38.
* Upstream: *"Use this version unless you know you have good reasons not to."*
* Fonts built on the result still use HarfBuzz's native font implementation.

### Faces from a file or blob, loaded by FreeType

These two do not need an `FT_Face` at all: they create one internally, along
with a private `FT_Library`, and tie both to the returned face's lifetime.
Their reason to exist is that FreeType understands containers HarfBuzz does not
— notably **WOFF and WOFF2**.

#### `hb_ft_face_create_from_file_or_fail`

```c
hb_face_t *hb_ft_face_create_from_file_or_fail (const char   *file_name,
                                                unsigned int  index);
```

```rust
pub fn hb_ft_face_create_from_file_or_fail(
    file_name: *const c_char,
    index: c_uint,
) -> *mut hb_face_t;
```

Opens a font file with `FT_New_Face` and wraps the result.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `file_name` | `const char *` | `*const c_char` | NUL-terminated path, passed straight to `FT_New_Face`. Required. Encoding is whatever FreeType does on the platform — unlike `hb_blob_create_from_file`, HarfBuzz applies no UTF-8 handling of its own. |
| `index` | `unsigned int` | `c_uint` | Face index within the file; `0` for a single-face font. FreeType's index also encodes named-instance selection in its high 16 bits. |

**Returns** — the new face, or **`NULL`** if the file cannot be read, holds no
face at `index`, no `FT_Library` could be created, or wrapping produced an
immutable face — which is how the code detects that it got HarfBuzz's nil
empty face back instead of a real one.

**Ownership**

* `(transfer full)`: destroy with `hb_face_destroy`.
* The internal `FT_Face` is created, referenced by the face
  (`hb_ft_face_create_referenced`), and dropped by the caller's side
  immediately, so the face owns the only remaining reference.
* The internal `FT_Library` is stashed in `ft_face->generic` with a finalizer,
  so it is released when the last `FT_Face` reference goes. Nothing for you to
  clean up.
* You never see either FreeType object. `hb_ft_font_get_ft_face` will not give
  it to you either, because the *face*, not the font, is what holds it.

**Notes**

* Since HarfBuzz 10.1.0.
* Functionally parallel to `hb_face_create_from_file_or_fail`, but with
  FreeType doing the parsing.
* Fonts created from the returned face use HarfBuzz's native font
  implementation until you call `hb_ft_font_set_funcs`.

#### `hb_ft_face_create_from_blob_or_fail`

```c
hb_face_t *hb_ft_face_create_from_blob_or_fail (hb_blob_t    *blob,
                                                unsigned int  index);
```

```rust
pub fn hb_ft_face_create_from_blob_or_fail(
    blob: *mut hb_blob_t,
    index: c_uint,
) -> *mut hb_face_t;
```

The in-memory counterpart: `FT_New_Memory_Face` over the blob's bytes.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `blob` | `hb_blob_t *` | `*mut hb_blob_t` | Font data. Required — the implementation calls `hb_blob_make_immutable` on it without a null check. |
| `index` | `unsigned int` | `c_uint` | Face index within the blob. |

**Returns** — the new face, or **`NULL`** if the blob is not valid font data,
no `FT_Library` could be created, wrapping produced an immutable face (the nil
empty face, as above), or attaching the blob to the face failed.

**Ownership**

* `(transfer full)`: destroy with `hb_face_destroy`.
* **`blob` is made immutable** — permanently — as a side effect.
* HarfBuzz takes its own reference on `blob` and attaches it to the face under a
  private user-data key, because the internal `FT_Face` keeps pointing into its
  bytes. Your reference is still yours: destroy it normally.
* The internal `FT_Face` and `FT_Library` are managed exactly as in
  `hb_ft_face_create_from_file_or_fail`.

**Notes**

* Since HarfBuzz 11.0.0.
* Functionally parallel to `hb_face_create_or_fail`, but FreeType parses, so
  WOFF/WOFF2 blobs work.
* On the failure path where user-data attachment fails, HarfBuzz cleans up
  both its blob reference and the face before returning null.

### Fonts backed by FreeType

#### `hb_ft_font_create`

```c
hb_font_t *hb_ft_font_create (FT_Face ft_face, hb_destroy_func_t destroy);
```

```rust
pub fn hb_ft_font_create(ft_face: FT_Face, destroy: hb_destroy_func_t) -> *mut hb_font_t;
```

Creates an `hb_font_t` from an `FT_Face`, already wired to FreeType font
functions. No lifecycle management.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `ft_face` | `FT_Face` | `FT_Face` | The FreeType face. Required, **and it must already have a size set** — HarfBuzz reads `ft_face->size->metrics` unconditionally. |
| `destroy` | `hb_destroy_func_t` | `hb_destroy_func_t` | Nullable. Passed through to the internal `hb_ft_face_create`, so it fires when the font's underlying face is destroyed, with `ft_face` as its argument. |

**Returns** — the new font, `(transfer full)`. Same "no explicit failure value"
caveat as `hb_ft_face_create`; an out-of-memory failure inside
`hb_font_set_funcs` leaves the font with empty funcs rather than reporting.

**Ownership**

* Destroy with `hb_font_destroy`. The intermediate `hb_face_t` it creates is
  owned by the font — it is destroyed by the constructor after
  `hb_font_create` takes its own reference, so you never see it and must not
  destroy it. Retrieve it later with `hb_font_get_face` if you need it.
* `ft_face` is borrowed, exactly as in `hb_ft_face_create`. You destroy it,
  after the font.
* The font's scale is initialized from the `FT_Face`'s size metrics
  (`hb_ft_font_changed` is called internally), and its load flags default to
  `FT_LOAD_DEFAULT | FT_LOAD_NO_HINTING`.
* Whether the charmap is a symbol charmap is captured *at creation time* from
  `ft_face->charmap`, and drives a Windows-compatible U+F000 fallback in the
  nominal-glyph callback.

**Notes**

* Since HarfBuzz 0.9.2.
* Upstream: *"Set face size on ft-face before creating hb-font from it.
  Otherwise hb-ft would NOT pick up the font size correctly."*
* Most programs should use `hb_ft_font_create_referenced`.

#### `hb_ft_font_create_referenced`

```c
hb_font_t *hb_ft_font_create_referenced (FT_Face ft_face);
```

```rust
pub fn hb_ft_font_create_referenced(ft_face: FT_Face) -> *mut hb_font_t;
```

The recommended constructor: same as `hb_ft_font_create`, but it calls
`FT_Reference_Face` on entry and `FT_Done_Face` when the font's face is
destroyed.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `ft_face` | `FT_Face` | `FT_Face` | The FreeType face, with a size already set. Required. |

**Returns** — the new font, `(transfer full)`. Destroy with `hb_font_destroy`.

**Ownership**

* HarfBuzz holds its own FreeType reference for as long as the font (strictly,
  its face) lives. Your reference remains yours; destruction order no longer
  matters.

**Notes**

* Since HarfBuzz 0.9.38.
* Upstream: *"Use this version unless you know you have good reasons not to."*
* Set the size on `ft_face` first — same rule as `hb_ft_font_create`.

#### `hb_ft_font_set_funcs`

```c
void hb_ft_font_set_funcs (hb_font_t *font);
```

```rust
pub fn hb_ft_font_set_funcs(font: *mut hb_font_t);
```

Switches an existing font over to FreeType font functions, *creating an
`FT_Face` internally to do it*. This is how you get FreeType metrics on a font
whose face came from `hb_face_create` — no `FT_Face` required from you.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `font` | `hb_font_t *` | `*mut hb_font_t` | The font to reconfigure, in place. Required; the implementation dereferences `font->face` immediately. |

**Returns** — nothing. See Notes: it can fail invisibly, three ways.

**Ownership**

* `font` is borrowed; no reference is taken.
* It calls `hb_face_reference_blob(font->face)` to get the face's bytes, builds
  an `FT_Face` over them with `FT_New_Memory_Face`, and creates (or reuses) a
  process-wide lazily initialized `FT_Library`. The blob is hooked to the
  `FT_Face`'s `generic` field and the `FT_Library` to the blob's user data, so
  everything is released when the font drops its font data. Nothing for you to
  free.
* **Any font data previously attached to `font` is destroyed** — this is a
  single atomic replacement of the font's `(funcs, font_data, destroy)` triple,
  and the outgoing data's destroy callback runs. User data attached with
  `hb_font_set_user_data` is a different mechanism and survives.
* Load flags are preserved if the font was already FreeType-backed, and
  otherwise reset to `FT_LOAD_DEFAULT | FT_LOAD_NO_HINTING`.
* It selects `FT_ENCODING_MS_SYMBOL` if present, else `FT_ENCODING_UNICODE`.

**Notes**

* Since HarfBuzz 1.0.5.
* Fonts from `hb_ft_font_create*` are already configured; this call is not
  needed for them (it would rebuild everything).
* **Silent failure.** Before doing anything, the function sets the font's funcs
  to `hb_font_funcs_get_empty()`. If the `FT_Library` cannot be created, if
  `FT_New_Memory_Face` fails, or if `hb_blob_set_user_data` fails, it returns
  early and the font is left with *empty* funcs — every glyph query then fails,
  and shaping produces `.notdef`. Verify with a probe such as
  `hb_font_get_nominal_glyph(font, 'A', &g)`.
* An immutable font is not special-cased here, but the underlying
  `hb_font_set_funcs` is a no-op on immutable fonts, so the call quietly does
  nothing.
* Upstream note: after modifying the `hb_font_t` you should call
  `hb_ft_hb_font_changed` to push the change into the internal `FT_Face` — no
  longer required as of HarfBuzz 11.0.0.
* Consider `hb_font_set_funcs_using(font, "ft")` (HarfBuzz 11.0.0+, in
  `hb-font.h`) instead: it selects the same implementation by name and returns
  `hb_bool_t`.

### Reaching the underlying `FT_Face`

All four of these work only on FreeType-backed fonts — the implementation
compares the font's destroy callback against hb-ft's own — and return
`NULL`/do nothing otherwise.

#### `hb_ft_font_get_ft_face`

```c
FT_Face hb_ft_font_get_ft_face (hb_font_t *font);
```

```rust
pub fn hb_ft_font_get_ft_face(font: *mut hb_font_t) -> FT_Face;
```

Fetches the `FT_Face` a FreeType-backed font is using.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `font` | `hb_font_t *` | `*mut hb_font_t` | The font. Required — dereferenced without a null check. |

**Returns** — the `FT_Face`, or `NULL` if `font` is not FreeType-backed.
Annotated `(nullable)` and `(skip)` upstream (it is not exposed to language
bindings via gobject-introspection).

**Ownership** — borrowed. Ownership stays with the font; do not `FT_Done_Face`
it, and do not use it after the font is destroyed.

**Notes**

* Since HarfBuzz 10.4.0. The older name is `hb_ft_font_get_face`.
* **No lock is taken.** If HarfBuzz might touch the same face concurrently — or
  reentrantly, from a draw or paint callback — use `hb_ft_font_lock_face`.
* The returned face's size and variation state reflect the last synchronization,
  not necessarily the font's current state; call `hb_ft_hb_font_changed` first
  if that matters.

#### `hb_ft_font_lock_face`

```c
FT_Face hb_ft_font_lock_face (hb_font_t *font);
```

```rust
pub fn hb_ft_font_lock_face(font: *mut hb_font_t) -> FT_Face;
```

Takes the font's internal `FT_Face` mutex and returns the face, so you can call
FreeType on it directly without racing HarfBuzz's own callbacks.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `font` | `hb_font_t *` | `*mut hb_font_t` | The font. Required. |

**Returns** — the locked `FT_Face`, or `NULL` if `font` is not FreeType-backed.
`(nullable) (transfer none) (skip)`.

**Ownership** — borrowed, and additionally *locked*. You hold a mutex until you
call `hb_ft_font_unlock_face`.

**Notes**

* Since HarfBuzz 2.6.5.
* **When it returns `NULL`, no lock was taken** — do not call
  `hb_ft_font_unlock_face` in that case.
* The lock is not recursive. While you hold it, do not call any HarfBuzz API
  that would go through the same font's glyph callbacks (`hb_shape`,
  `hb_font_get_glyph_extents`, …) — that deadlocks.
* Keep the critical section short, and remember that HarfBuzz's own callbacks
  may mutate the face's size/transform when they notice the font changed.

#### `hb_ft_font_unlock_face`

```c
void hb_ft_font_unlock_face (hb_font_t *font);
```

```rust
pub fn hb_ft_font_unlock_face(font: *mut hb_font_t);
```

Releases the lock taken by `hb_ft_font_lock_face`.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `font` | `hb_font_t *` | `*mut hb_font_t` | The same font passed to `hb_ft_font_lock_face`. Required. |

**Returns** — nothing.

**Ownership** — nothing changes hands; the `FT_Face` you were given must not be
used after this returns.

**Notes**

* Since HarfBuzz 2.6.5.
* A no-op for fonts that are not FreeType-backed.
* Unbalanced calls are undefined: unlocking a mutex you do not hold is a bug at
  the pthread level, not something HarfBuzz checks.

#### `hb_ft_font_get_face` (deprecated)

```c
HB_DEPRECATED_FOR (hb_ft_font_get_ft_face)
FT_Face hb_ft_font_get_face (hb_font_t *font);
```

```rust
#[deprecated(note = "use `hb_ft_font_get_ft_face` instead")]
pub fn hb_ft_font_get_face(font: *mut hb_font_t) -> FT_Face;
```

The original name for `hb_ft_font_get_ft_face`; the implementation is a
one-line forward to it. Renamed because "face" was ambiguous between `FT_Face`
and `hb_face_t`.

**Parameters / Returns / Ownership** — identical to `hb_ft_font_get_ft_face`.

**Notes**

* Since HarfBuzz 0.9.2. Deprecated since HarfBuzz 10.4.0.
* Declared inside `#ifndef HB_DISABLE_DEPRECATED`, so a build that defines that
  macro drops the declaration — but the symbol is still compiled and exported,
  and `harfbuzz-sys` never defines it. The Rust module declares it
  unconditionally with `#[deprecated]`.
* Upstream files this symbol under the `hb-deprecated` gtk-doc section rather
  than `hb-ft`, even though it is declared in `hb-ft.h`.

### Load flags

#### `hb_ft_font_set_load_flags`

```c
void hb_ft_font_set_load_flags (hb_font_t *font, int load_flags);
```

```rust
pub fn hb_ft_font_set_load_flags(font: *mut hb_font_t, load_flags: c_int);
```

Sets the flags hb-ft passes to `FT_Load_Glyph` for this font.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `font` | `hb_font_t *` | `*mut hb_font_t` | The font. Required. |
| `load_flags` | `int` | `c_int` | FreeType's `FT_LOAD_*` constants, ORed. Not HarfBuzz constants — this crate does not define them; take them from FreeType's headers or a FreeType binding. See FreeType's [`FT_LOAD_XXX` documentation](https://freetype.org/freetype2/docs/reference/ft2-glyph_retrieval.html#ft_load_xxx). |

**Returns** — nothing.

**Ownership** — none; a plain setter on the font's private hb-ft state.

**Notes**

* Since HarfBuzz 1.0.5.
* **No-op, silently, if the font is immutable or not FreeType-backed.**
* The default is `FT_LOAD_DEFAULT | FT_LOAD_NO_HINTING` (numerically `0 | 2`).
  hb-ft deliberately works unhinted: it abuses no-hinting mode to get
  unrounded, font-space-like results. Turning hinting on makes HarfBuzz's
  advances match a *hinted* rasterization instead — which is what you want if
  you are rendering hinted, and wrong if you are not.
* Some flags are forced regardless: the draw callback ORs in
  `FT_LOAD_NO_BITMAP`, and the paint callback ORs in `FT_LOAD_COLOR` (plus
  `FT_LOAD_NO_SVG` on FreeType ≥ 2.13.1).
* Changing load flags does not invalidate the advance cache; the cache is keyed
  on the font's serial, which this does not bump.

#### `hb_ft_font_get_load_flags`

```c
int hb_ft_font_get_load_flags (hb_font_t *font);
```

```rust
pub fn hb_ft_font_get_load_flags(font: *mut hb_font_t) -> c_int;
```

Fetches the current `FT_Load_Glyph` flags.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `font` | `hb_font_t *` | `*mut hb_font_t` | The font. Required. |

**Returns** — the flags, or `0` if the font is not FreeType-backed. Upstream
documents that as *"FT_Load_Glyph flags found, or 0"*; note that `0` is also
`FT_LOAD_DEFAULT`, so the error value is indistinguishable from a legitimate
setting.

**Ownership** — none.

**Notes**

* Since HarfBuzz 1.0.5.

### Keeping the two objects in sync

#### `hb_ft_font_changed`

```c
void hb_ft_font_changed (hb_font_t *font);
```

```rust
pub fn hb_ft_font_changed(font: *mut hb_font_t);
```

`FT_Face` → `hb_font_t`. Call it after you have changed the size or the
variation coordinates on the underlying `FT_Face`, to make the font agree.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `font` | `hb_font_t *` | `*mut hb_font_t` | The font whose `FT_Face` you changed. Required. |

**Returns** — nothing.

**Ownership** — none; mutates `font` in place.

**Notes**

* Since HarfBuzz 1.0.5.
* Silently does nothing if `font` is not FreeType-backed.
* Concretely it: recomputes `hb_font_set_scale` from
  `ft_face->size->metrics.{x,y}_scale × units_per_EM` (rounded 16.16); reads
  the face's blend coordinates with `FT_Get_Var_Blend_Coordinates` and
  republishes them via `hb_font_set_var_coords_normalized` (clearing them if
  all are zero); clears the advance cache; and records the font's current
  serial as "already synchronized".
* Note the asymmetry with `hb_ft_hb_font_changed`: this one has no
  "did anything change?" fast path and no return value.
* It does *not* touch ppem — hb-ft works in a no-hinting model where ppem stays
  unset.

#### `hb_ft_hb_font_changed`

```c
hb_bool_t hb_ft_hb_font_changed (hb_font_t *font);
```

```rust
pub fn hb_ft_hb_font_changed(font: *mut hb_font_t) -> hb_bool_t;
```

`hb_font_t` → `FT_Face`. The mirror image: call it after changing the scale or
variations on the `hb_font_t`, to push that into the `FT_Face`.

**Parameters**

| Name | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `font` | `hb_font_t *` | `*mut hb_font_t` | The font you changed. Required. |

**Returns** — `true` if the `FT_Face` was actually updated; `false` if nothing
had changed since the last synchronization, or if `font` is not FreeType-backed.

**Ownership** — none; mutates the `FT_Face` in place, under the font's internal
lock.

**Notes**

* Since HarfBuzz 4.4.0.
* **As of HarfBuzz 11.0.0 you no longer need to call this**: every hb-ft glyph
  callback runs the same serial check first and updates the face on demand.
  Upstream says so explicitly in the function's own documentation.
* Cheap when nothing changed — it compares `font->serial` against a cached
  value and returns immediately.
* What it pushes: `FT_Set_Char_Size(|x_scale|, |y_scale|)`; if that fails and
  the face has fixed sizes (a bitmap or colour-emoji font) it selects the
  largest strike and compensates with an `FT_Set_Transform` matrix; negative
  scales are folded into that matrix too; and design variation coordinates are
  pushed with `FT_Set_Var_Design_Coordinates`.

## What FreeType font functions actually implement

Not in the header, but it is the practical question once you call
`hb_ft_font_set_funcs`. The callback table hb-ft installs — one immutable,
lazily created, atexit-freed singleton shared by every FreeType-backed font —
is:

| Installed callback | FreeType call behind it | Build guard |
| --- | --- | --- |
| `nominal_glyph` | `FT_Get_Char_Index`, plus a symbol-font fallback that retries at `U+F000 + cp` (and Arabic PUA maps for the simplified/traditional Arabic font pages) | always |
| `nominal_glyphs` (batch) | `FT_Get_Char_Index` in a loop; stops at the first miss and lets HarfBuzz retry singly | always |
| `variation_glyph` | `FT_Face_GetCharVariantIndex` | always |
| `font_h_extents` | `ft_face->size->metrics` ascender/descender/height | always |
| `glyph_h_advances` (batch) | `FT_Load_Glyph` + `metrics.horiAdvance`, through a 16-bit-keyed advance cache | always |
| `glyph_v_advance` | `FT_Load_Glyph` + `metrics.vertAdvance` | unless `HB_NO_VERTICAL` |
| `glyph_v_origin` | `FT_Load_Glyph` + `horiBearingX/Y` and `vertBearingX/Y` | unless `HB_NO_VERTICAL` |
| `glyph_h_kerning` | `FT_Get_Kerning` (`FT_KERNING_UNFITTED` when ppem is 0) | unless `HB_NO_OT_SHAPE_FALLBACK` |
| `glyph_extents` | `FT_Load_Glyph` + `metrics`, transform-compensated; **returns false for `COLR` glyphs**, since FreeType does not report their extents | always |
| `glyph_contour_point` | `FT_Load_Glyph` + `outline.points[i]` | always |
| `glyph_name` | `FT_Get_Glyph_Name` | always |
| `glyph_from_name` | `FT_Get_Name_Index` | always |
| `draw_glyph_or_fail` | `FT_Load_Glyph` (with `FT_LOAD_NO_BITMAP`) + `FT_Outline_Decompose`; fails for non-outline glyphs | unless `HB_NO_DRAW` |
| `paint_glyph_or_fail` | `FT_Get_Color_Glyph_Paint` / `FT_Get_Color_Glyph_Layer` for `COLR`, else BGRA bitmap strikes | unless `HB_NO_PAINT`, FreeType ≥ 2.13.0 |

Two absences matter. There is **no `font_v_extents`**, so vertical font extents
fall through to the parent font chain. And there is **no `glyph_h_origin`**,
which is correct for OpenType (the origin is `(0, 0)`) and therefore an
optimization rather than a gap.

`harfbuzz-sys` defines none of the `HB_NO_*` macros in a default build, so the
full table above is what you get — subject to the FreeType version you link.

## Usage

### C: shape with FreeType metrics, the recommended way

```c
#include <hb.h>
#include <hb-ft.h>
#include <ft2build.h>
#include FT_FREETYPE_H

FT_Library ft_library;
FT_Init_FreeType (&ft_library);

FT_Face ft_face;
FT_New_Face (ft_library, "Roboto.ttf", 0, &ft_face);

/* MUST happen before hb_ft_font_create_referenced(). */
FT_Set_Char_Size (ft_face, 0, 20 * 64, 72, 72);

hb_font_t *font = hb_ft_font_create_referenced (ft_face);

hb_buffer_t *buf = hb_buffer_create ();
hb_buffer_add_utf8 (buf, "Hello", -1, 0, -1);
hb_buffer_guess_segment_properties (buf);
hb_shape (font, buf, NULL, 0);

/* ... read hb_buffer_get_glyph_infos / _positions ... */

hb_buffer_destroy (buf);
hb_font_destroy (font);

/* Order no longer matters: the font took its own FreeType reference. */
FT_Done_Face (ft_face);
FT_Done_FreeType (ft_library);
```

### C: resize a font that FreeType owns

```c
FT_Set_Char_Size (ft_face, 0, 32 * 64, 72, 72);
hb_ft_font_changed (font);        /* FT_Face -> hb_font_t */
```

### C: resize a font that HarfBuzz owns

```c
hb_font_set_scale (font, 32 * 64, 32 * 64);
hb_ft_hb_font_changed (font);     /* hb_font_t -> FT_Face; optional since 11.0.0 */
```

### C: load a WOFF2 file HarfBuzz cannot parse itself

```c
hb_face_t *face = hb_ft_face_create_from_blob_or_fail (woff2_blob, 0);
if (!face) { /* not valid font data */ }

hb_font_t *font = hb_font_create (face);
hb_font_set_scale (font, 20 * 64, 20 * 64);
/* Native HarfBuzz font funcs; add hb_ft_font_set_funcs(font) for FreeType ones. */

hb_font_destroy (font);
hb_face_destroy (face);
```

### C: put an existing font on FreeType, without an `FT_Face`

```c
hb_blob_t *blob = hb_blob_create_from_file_or_fail ("Roboto.ttf");
hb_face_t *face = hb_face_create (blob, 0);
hb_font_t *font = hb_font_create (face);
hb_font_set_scale (font, 20 * 64, 20 * 64);

hb_ft_font_set_funcs (font);   /* creates its own FT_Face internally */

hb_codepoint_t g;
if (!hb_font_get_nominal_glyph (font, 'A', &g))
  /* hb_ft_font_set_funcs failed silently, or the font has no 'A' */;
```

### C: call FreeType directly on a font's face, safely

```c
FT_Face ft = hb_ft_font_lock_face (font);
if (ft) {
    FT_Load_Glyph (ft, gid, hb_ft_font_get_load_flags (font));
    /* ... render ft->glyph ... */
    hb_ft_font_unlock_face (font);
}
```

### Rust: the recommended construction, with a FreeType binding

`harfbuzz-sys` has no FreeType dependency, so bring your own and cast the
handle. The cast is a plain pointer cast — both sides are thin pointers to an
opaque record.

```rust
use harfbuzz_sys::ft::{FT_Face, hb_ft_font_create_referenced};
use harfbuzz_sys::{hb_font_destroy, hb_font_t};

/// `their_ft_face` is whatever your FreeType crate calls an `FT_Face`.
unsafe fn font_from_ft_face(their_ft_face: *mut core::ffi::c_void) -> *mut hb_font_t {
    let ft_face = their_ft_face as FT_Face;

    // The FT_Face must already have a size set: HarfBuzz dereferences
    // ft_face->size unconditionally.
    unsafe { hb_ft_font_create_referenced(ft_face) }
}

unsafe fn release(font: *mut hb_font_t) {
    unsafe { hb_font_destroy(font) };
}
```

### Rust: no `FT_Face` at all — load a WOFF2 through FreeType

```rust
use core::ffi::c_char;
use harfbuzz_sys::ft::hb_ft_face_create_from_file_or_fail;
use harfbuzz_sys::{hb_face_destroy, hb_face_t};

unsafe fn load_woff2(path: &core::ffi::CStr) -> Option<*mut hb_face_t> {
    let face = unsafe {
        hb_ft_face_create_from_file_or_fail(path.as_ptr() as *const c_char, 0)
    };
    if face.is_null() { None } else { Some(face) }
}
```

Everything FreeType allocates — the `FT_Face`, and a private `FT_Library` —
is released when you `hb_face_destroy` the result.

### Rust: switch an existing font onto FreeType, and verify it worked

`hb_ft_font_set_funcs` returns nothing and fails silently, so probe afterwards.

```rust
use harfbuzz_sys::ft::hb_ft_font_set_funcs;
use harfbuzz_sys::{hb_codepoint_t, hb_font_get_nominal_glyph, hb_font_t};

/// Returns false if the switch left the font with empty font functions.
unsafe fn use_freetype_funcs(font: *mut hb_font_t) -> bool {
    unsafe { hb_ft_font_set_funcs(font) };

    let mut glyph: hb_codepoint_t = 0;
    unsafe { hb_font_get_nominal_glyph(font, 'A' as hb_codepoint_t, &mut glyph) != 0 }
}
```

If you are on HarfBuzz 11.0.0 or newer, prefer the checked spelling from
`hb-font.h`, which reports failure directly:

```rust
use harfbuzz_sys::{hb_font_set_funcs_using, hb_font_t};

unsafe fn use_freetype_funcs_checked(font: *mut hb_font_t) -> bool {
    unsafe { hb_font_set_funcs_using(font, c"ft".as_ptr()) != 0 }
}
```

### Rust: hinted metrics

```rust
use harfbuzz_sys::ft::{hb_ft_font_get_load_flags, hb_ft_font_set_load_flags};
use harfbuzz_sys::hb_font_t;

// FreeType's own constants; harfbuzz-sys does not define them.
const FT_LOAD_DEFAULT: i32 = 0;
const FT_LOAD_NO_HINTING: i32 = 1 << 1;
const FT_LOAD_TARGET_LIGHT: i32 = 1 << 16;

unsafe fn enable_light_hinting(font: *mut hb_font_t) {
    let flags = unsafe { hb_ft_font_get_load_flags(font) };
    let flags = (flags & !FT_LOAD_NO_HINTING) | FT_LOAD_DEFAULT | FT_LOAD_TARGET_LIGHT;
    unsafe { hb_ft_font_set_load_flags(font, flags) };
}
```

Only do this if you are also rasterizing with hinting; otherwise HarfBuzz's
positions will not match your rendering.

### Rust: borrow the `FT_Face` under the lock

```rust
use harfbuzz_sys::ft::{FT_Face, hb_ft_font_lock_face, hb_ft_font_unlock_face};
use harfbuzz_sys::hb_font_t;

unsafe fn with_ft_face<R>(font: *mut hb_font_t, f: impl FnOnce(FT_Face) -> R) -> Option<R> {
    let ft_face = unsafe { hb_ft_font_lock_face(font) };
    if ft_face.is_null() {
        // Not a FreeType-backed font — and no lock was taken, so do NOT unlock.
        return None;
    }

    let result = f(ft_face);

    unsafe { hb_ft_font_unlock_face(font) };
    Some(result)
}
```

Do not call any HarfBuzz API on the same font inside the closure: the lock is
not recursive and the glyph callbacks take it.

## Pitfalls

**Set the `FT_Face` size before creating a font.** `hb_ft_font_create` and
`hb_ft_font_create_referenced` read `ft_face->size->metrics` with no null check
and no default. A face straight out of `FT_New_Face` has a size record, but a
zeroed one, so the resulting font gets a scale of 0 and every advance comes
back 0 — or worse. This is the header's own first note, and it is easy to miss
because there is no error.

**Faces and fonts are different bridges.** `hb_ft_face_create*` gives you
*data*; it does **not** give you FreeType metrics. Fonts built on those faces
use HarfBuzz's native implementation until you call `hb_ft_font_set_funcs`.
Conversely `hb_ft_font_create*` does both at once. If your advances do not match
FreeType's, this is almost always why.

**Ten functions silently ignore non-hb-ft fonts.** `hb_ft_font_get_ft_face`,
`hb_ft_font_get_face`, `hb_ft_font_lock_face` return `NULL`;
`hb_ft_font_unlock_face`, `hb_ft_font_set_load_flags`, `hb_ft_font_changed` do
nothing; `hb_ft_font_get_load_flags` returns `0`; `hb_ft_hb_font_changed`
returns `false`. The test is an identity comparison against hb-ft's internal
destroy callback, and there is no public predicate for it. Passing a font from
`hb_font_create` to any of these is a no-op you will not be told about.

**`hb_ft_font_get_load_flags` cannot distinguish failure from
`FT_LOAD_DEFAULT`.** Both are `0`.

**`hb_ft_font_lock_face` returning `NULL` means you must not unlock.** The
implementation returns before taking the mutex in that case; an unmatched
`hb_ft_font_unlock_face` unlocks a mutex you never held.

**The lock is not recursive, and HarfBuzz takes it.** While holding
`hb_ft_font_lock_face`, calling `hb_shape` or any `hb_font_get_*` on the same
font deadlocks. The one exception baked into HarfBuzz itself is the paint
callback, which deliberately drops the lock before invoking client callbacks so
that a draw callback can call back into the face.

**`hb_ft_font_set_funcs` destroys your font data.** It replaces the font's
`(funcs, font_data, destroy)` triple atomically, running the old destroy
callback. If you attached custom `font_data` with `hb_font_set_funcs`, install
FreeType first and your own second. User data from `hb_font_set_user_data` is
unaffected.

**`hb_ft_font_set_funcs` fails silently and leaves the font *empty*.** It sets
`hb_font_funcs_get_empty()` before trying, so a failure to create the
`FT_Library` or the `FT_Face` leaves a font on which every query fails and
shaping yields `.notdef` runs. Probe with `hb_font_get_nominal_glyph`, or use
`hb_font_set_funcs_using(font, "ft")`, which returns a boolean.

**`hb_ft_face_create_cached` hijacks `ft_face->generic`.** If your own code, or
another library, uses that field on the same `FT_Face`, this call runs its
finalizer and overwrites it. There is no way to ask whether it is safe. The same
field is also claimed by `hb_ft_face_create_from_file_or_fail` and
`hb_ft_face_create_from_blob_or_fail` for their internal `FT_Library`.

**`hb_ft_face_create_from_blob_or_fail` makes your blob immutable, forever.**
That is irreversible, and it means a subsequent `hb_blob_get_data_writable` on
that blob fails.

**Colour glyphs report no extents.** hb-ft's `glyph_extents` callback returns
false for any glyph FreeType recognizes as a `COLR` glyph, because FreeType does
not expose their bounds. `hb_font_get_glyph_extents` therefore fails on colour
emoji under FreeType font functions, where HarfBuzz's native implementation
would have answered.

**hb-ft is unhinted by default, on purpose.** The default load flags include
`FT_LOAD_NO_HINTING`, and hb-ft never sets ppem, because it abuses no-hinting
mode to approximate font-space arithmetic. Enabling hinting changes every
advance HarfBuzz reports. Do it only to match a hinted rasterizer.

**`hb_ft_hb_font_changed` is legacy but `hb_ft_font_changed` is not.** Only the
`hb_font_t` → `FT_Face` direction became automatic in HarfBuzz 11.0.0. If you
mutate the `FT_Face` behind HarfBuzz's back, you must still call
`hb_ft_font_changed` yourself; nothing detects that.

**Nothing here is thread-safe.** FreeType is not, so neither are these
functions. The internal per-font mutex protects only HarfBuzz's own use of its
`FT_Face`. Two threads sharing one `FT_Face` through separate `hb_font_t`s, or
touching one `FT_Library` concurrently, are on their own. The usual HarfBuzz
discipline — configure on one thread, `hb_font_make_immutable`, then share —
does not fully apply, because hb-ft's callbacks mutate the `FT_Face` even for
read-only queries.

**Symbol-font behaviour is captured at creation time.** Whether the U+F000
fallback is active is decided from `ft_face->charmap` when the font data is
created. Calling `FT_Select_Charmap` afterwards does not update it.

**The `FT_Face` type in this crate is a stand-in.** It is ABI-correct but
structurally empty, so you cannot read `units_per_EM`, `glyph`, or anything else
from it here. Use a real FreeType binding and cast between the two pointer
types.

**Everything in this module is behind the `freetype` feature.** Without it,
`harfbuzz_sys::ft` does not exist, and upstream's `hb-ft.cc` is not compiled at
all — the symbols are absent from the archive rather than failing at run time.
The feature also requires the system `freetype2` package to be visible to
`pkg-config` at build time.
