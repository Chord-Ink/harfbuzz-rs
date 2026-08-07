# Graphite2 integration

Header: `hb-graphite2.h` — Rust module: `harfbuzz_sys::graphite2`. The module is
gated on the crate's `graphite2` Cargo feature and is **not** glob re-exported at
the crate root, so its items are reached as `harfbuzz_sys::graphite2::*`.

## Overview

[Graphite](http://graphite.sil.org/) is SIL International's "smart font"
technology. Where OpenType describes shaping as a fixed pipeline of substitution
and positioning lookups that the shaping engine interprets, Graphite compiles a
small rule *program* — written in GDL, the Graphite Description Language — into
the font, and a Graphite engine executes that program over the text. Because the
font author writes the algorithm rather than selecting from a fixed vocabulary
of lookup types, Graphite can support writing systems whose behaviour OpenType
has no model for. That is why Graphite fonts are common for minority and
lesser-documented scripts.

A Graphite font carries extra tables alongside its ordinary OpenType ones:

| Table | Contents |
| --- | --- |
| `Silf` | The compiled Graphite rules — the program itself. |
| `Glat` | Glyph attributes referenced by the rules. |
| `Gloc` | Index into `Glat`. |
| `Feat` | The font's Graphite feature declarations. |
| `Sill` | Optional: per-language default feature settings. |

HarfBuzz does not implement Graphite itself. It delegates to SIL's
`libgraphite2`, which is why this is an optional back end rather than part of
the core.

### Shaping happens automatically

There is nothing in this header you have to call to get Graphite shaping. When
HarfBuzz is built with Graphite support, `graphite2` is registered in
`hb-shaper-list.hh` **ahead of** the native `ot` shaper (and after `wasm`), and
`hb_shape()` picks it for any face whose `Silf` table is present and non-empty.
A face without a `Silf` table silently falls through to the OpenType shaper,
which is the desired behaviour: enabling this back end never changes how a
non-Graphite font shapes.

The selection is literally the shaper's face-data constructor refusing to
initialise: `_hb_graphite2_shaper_face_data_create()` references
`HB_GRAPHITE2_TAG_SILF`, returns null if the blob's length is zero, and a null
face data marks the shaper as inapplicable for that face.

To go the other way and *refuse* Graphite for a particular call, pass an
explicit shaper list to `hb_shape_full()` — `{"ot", NULL}` — or reorder the
defaults process-wide with the `HB_SHAPER_LIST` environment variable.

### What this header is for

Given that, the header's job is narrow: it hands you the underlying
`libgraphite2` object so you can ask Graphite questions HarfBuzz does not
expose — enumerating a font's Graphite features with `gr_face_n_fref` and
`gr_face_fref`, listing its languages via `gr_face_n_languages`, inspecting
feature value labels, or building your own `gr_segment`. That is what
`hb_graphite2_face_get_gr_face()` is for. The header defines exactly one tag
constant, one live function, and one deprecated function.

One caveat about features: HarfBuzz maps each `hb_feature_t` you pass to shaping
onto a Graphite feature reference **by tag**, and ignores tags the font does not
declare. Graphite features are named by the font designer and are not drawn from
the OpenType feature registry, so the tags that work here are whatever the font
declares in its `Feat` table. Discovering them is the main reason to reach for
the `gr_face`.

### Building

The sources behind these declarations are compiled only when the crate's
`graphite2` feature is enabled, which also requires the system `graphite2`
library to be discoverable through `pkg-config` (package name `graphite2`).
`build.rs` probes for it, emits the link directives, and defines both
`HB_HAS_GRAPHITE` and `HAVE_GRAPHITE2` — the latter explicitly, because
upstream's `harfbuzz-world.cc` translates `HB_HAS_GRAPHITE` into `HAVE_GRAPHITE`
while every consumer in the sources tests `HAVE_GRAPHITE2`.

Note upstream's own remark in the section documentation: "Currently, the default
is to not enable `graphite2` shaping." A stock distribution HarfBuzz may or may
not have it.

## Types

### `gr_face`

```c
/* <graphite2/Font.h> — libgraphite2, not HarfBuzz */
typedef struct gr_face gr_face;
```

```rust
crate::opaque_handle! { gr_face }
```

`libgraphite2`'s counterpart of `hb_face_t`: a typeface with its compiled
Graphite rule tables loaded, and no size attached. It is the object every
`gr_face_*` call and `gr_make_seg` takes.

It belongs to SIL's library, not to HarfBuzz, and this crate has no `graphite2`
dependency to import it from — so it is declared locally as what it is in C, an
opaque struct used only behind a pointer. It is layout- and ABI-compatible with
the real definition, so a handle from a Graphite binding crate converts with a
plain pointer cast:

```rust
let gr = unsafe { harfbuzz_sys::graphite2::hb_graphite2_face_get_gr_face(face) };
let theirs = gr.cast::<their_crate::gr_face>();
```

This crate never looks inside it. A `gr_face` obtained from
`hb_graphite2_face_get_gr_face()` is owned by the `hb_face_t` it came from — do
not pass it to `gr_face_destroy`.

### `gr_font`

```c
/* <graphite2/Font.h> — libgraphite2, not HarfBuzz */
typedef struct gr_font gr_font;
```

```rust
crate::opaque_handle! { gr_font }
```

`libgraphite2`'s counterpart of `hb_font_t`: a `gr_face` at a particular
pixels-per-em size, used to scale Graphite's design-unit output. HarfBuzz no
longer creates one — since 1.4.2 it shapes with a null `gr_font` and applies its
own scaling — so the only function that mentions this type,
`hb_graphite2_font_get_gr_font()`, is deprecated and always returns null.

### `hb_face_t`, `hb_font_t`

HarfBuzz's own face and font objects, defined in `hb-face.h` and `hb-font.h`.

## Constants

### `HB_GRAPHITE2_TAG_SILF`

```c
#define HB_GRAPHITE2_TAG_SILF HB_TAG('S','i','l','f')
```

```rust
pub const HB_GRAPHITE2_TAG_SILF: hb_tag_t = HB_TAG(b'S', b'i', b'l', b'f');
```

The `hb_tag_t` for the `Silf` table, which holds a font's compiled Graphite
rules. Numeric value `0x53696C66`.

This is the table HarfBuzz tests when deciding whether a face can be shaped by
Graphite: a face whose `Silf` table is missing or zero-length gets no Graphite
back end at all and shapes with the OpenType shaper instead. Fetch it like any
other table, with `hb_face_reference_table()`, if you want to test for Graphite
support yourself.

The header's own documentation block carries no `Since:` annotation. For more
information, see <http://graphite.sil.org/>.

## Functions

### `hb_graphite2_face_get_gr_face`

```c
gr_face *hb_graphite2_face_get_gr_face (hb_face_t *face);
```

```rust
pub fn hb_graphite2_face_get_gr_face(face: *mut hb_face_t) -> *mut gr_face;
```

Fetches the Graphite2 `gr_face` corresponding to the specified `hb_face_t`.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `face` | The face to query. | The header does not annotate nullability; the implementation dereferences `face->data.graphite2` immediately, so treat null as forbidden. |

**Returns** — the cached `gr_face`, or **null**. Null is the normal answer for
an ordinary OpenType font, not an error to report. Creation fails — and this
returns null — when:

- the face has no `HB_GRAPHITE2_TAG_SILF` table, or that table is zero-length;
- `libgraphite2`'s `gr_make_face_with_ops` rejects the font;
- allocation fails.

**Ownership** — no transfer. The `gr_face` is created on first use and cached on
the `hb_face_t`; it lives exactly as long as the face does and is destroyed with
it. Do **not** call `gr_face_destroy` on it, and do not use it after the last
reference to `face` is released.

There is a subtler lifetime rule underneath. HarfBuzz builds the `gr_face` with
`gr_make_face_with_ops` and a table-fetch callback that reads through
`hb_face_reference_table()`, keeping a reference to every blob it hands out in a
per-face list. The Graphite tables are therefore *borrowed* from the face's
blobs rather than copied — so the `hb_face_t` must outlive not only the
`gr_face` but any `gr_segment` you build from it.

**Notes** — Since HarfBuzz 0.9.10. Upstream marks the function `(skip)` for
introspection. The lazy creation goes through HarfBuzz's shaper-face-data lazy
loader and the table list is maintained with an atomic compare-and-exchange, so
two threads calling this on the same face concurrently is safe: one of the two
candidate objects wins and both callers see it.

The face data is created with `gr_face_preloadAll`, so all Graphite tables are
read up front rather than on demand.

### `hb_graphite2_font_get_gr_font` (deprecated)

```c
HB_DEPRECATED_FOR (hb_graphite2_face_get_gr_face)
gr_font *hb_graphite2_font_get_gr_font (hb_font_t *font);
```

```rust
#[deprecated(note = "deprecated in HarfBuzz 1.4.2 and always returns null; \
                     use hb_graphite2_face_get_gr_face instead")]
pub fn hb_graphite2_font_get_gr_font(font: *mut hb_font_t) -> *mut gr_font;
```

**Always returns null.** HarfBuzz used to build a per-size `gr_font` for each
`hb_font_t` and hand it to `gr_make_seg`. Since HarfBuzz 1.4.2 it shapes with a
null `gr_font` and scales Graphite's design-unit output itself, using the font's
`x_scale`, `y_scale`, and the face's units-per-em — so there is no `gr_font` left
to return. The symbol is retained for binary compatibility, and returning null is
its entire implementation.

**Parameters** — `font` is ignored, including a null one.

**Returns** — null, unconditionally.

**Ownership** — nothing to release.

**Notes** — Since HarfBuzz 0.9.10; deprecated since 1.4.2. Declared inside
`#ifndef HB_DISABLE_DEPRECATED`, so a build configured with `HB_LEAN` or
`HB_TINY` (which define `HB_DISABLE_DEPRECATED`) does not export it at all —
calling it there is a link error. Upstream files this function under the
`hb-deprecated` gtk-doc section rather than `hb-graphite2`; it is documented here
because it lives in this header. Use `hb_graphite2_face_get_gr_face()` instead.

## Usage

### C: detect whether a face will shape with Graphite

```c
#include <hb.h>
#include <hb-graphite2.h>

hb_bool_t face_is_graphite (hb_face_t *face)
{
  /* Cheap: just look for the table. */
  hb_blob_t *silf = hb_face_reference_table (face, HB_GRAPHITE2_TAG_SILF);
  hb_bool_t  ok   = hb_blob_get_length (silf) != 0;
  hb_blob_destroy (silf);
  return ok;
}

hb_bool_t graphite_engine_accepted (hb_face_t *face)
{
  /* Stronger: libgraphite2 actually parsed it. */
  return hb_graphite2_face_get_gr_face (face) != NULL;
}
```

### C: enumerate the font's Graphite features

Graphite feature tags come from the font, not from a registry, so you have to
ask the font what it offers before you can request anything.

```c
#include <graphite2/Font.h>

gr_face *gr = hb_graphite2_face_get_gr_face (face);
if (gr)
{
  gr_uint16 n = gr_face_n_fref (gr);
  for (gr_uint16 i = 0; i < n; i++)
  {
    const gr_feature_ref *fref = gr_face_fref (gr, i);
    gr_uint32 tag = gr_fref_id (fref);   /* the tag to put in hb_feature_t */
    /* ... inspect gr_fref_n_values, gr_fref_value, labels, etc. ... */
  }
}
/* `gr` is owned by `face`. Do not gr_face_destroy it. */
```

### C: force the OpenType shaper on a Graphite font

```c
const char *shapers[] = { "ot", NULL };
hb_shape_full (font, buf, features, num_features, shapers);
```

Or, process-wide, `HB_SHAPER_LIST=ot` in the environment.

### Rust: reach the `gr_face`

```rust
use harfbuzz_sys::hb_face_t;
use harfbuzz_sys::graphite2::{gr_face, hb_graphite2_face_get_gr_face};

/// Returns the Graphite face cached on `face`, or `None` if this font has no
/// usable Graphite tables.
///
/// # Safety
/// `face` must be a live, non-null `hb_face_t`. The returned pointer borrows
/// from `face` and must not outlive it, and must never be passed to
/// `gr_face_destroy`.
unsafe fn graphite_face(face: *mut hb_face_t) -> Option<*mut gr_face> {
    // SAFETY: `face` is live by the caller's contract. The call only reads the
    // lazily-initialised shaper face data; it never transfers ownership.
    let gr = unsafe { hb_graphite2_face_get_gr_face(face) };
    (!gr.is_null()).then_some(gr)
}
```

### Rust: passing a font-declared feature to shaping

```rust
use harfbuzz_sys::{hb_feature_t, HB_TAG};

// The tag must be one the font's `Feat` table declares; unknown tags are
// silently ignored by the Graphite back end.
let feature = hb_feature_t {
    tag: HB_TAG(b'1', b'0', b'2', b'1'),   // example: a font-specific tag
    value: 1,
    start: 0,
    end: u32::MAX,
};
```

## Pitfalls

- **`hb_graphite2_font_get_gr_font` always returns null.** It is not a bug in
  your code and not an error condition — the function has had no implementation
  since 1.4.2. Never dereference its result.

- **A null `gr_face` is normal.** Every non-Graphite font returns null. Do not
  treat it as a failure to report; treat it as "this font has no Graphite data".

- **Do not destroy the `gr_face`.** It is owned by the `hb_face_t`. Calling
  `gr_face_destroy` on it produces a double free when the face is released.

- **The `gr_face` borrows the face's table blobs.** Anything you build from it —
  particularly a `gr_segment` — is only valid while the `hb_face_t` is alive.
  Keep a reference to the face for at least as long as you keep the `gr_face`.

- **Graphite feature tags are not OpenType feature tags.** `liga` and `kern`
  mean nothing to a Graphite font unless the designer happened to declare tags
  by those names in `Feat`. Enumerate the font's features first; anything else
  is silently dropped.

- **Enabling the back end changes shaper *selection*, not just capability.**
  Once `graphite2` is compiled in it sits ahead of `ot` in the default shaper
  list. Any font that ships both OpenType layout tables and a `Silf` table will
  now shape through Graphite by default, which may produce different output than
  before. Pass an explicit shaper list if that matters.

- **Version skew.** HarfBuzz links whatever `libgraphite2` `pkg-config` finds;
  the `gr_face` you receive is that library's object, and any `gr_*` calls you
  make must come from the same library. Mixing two Graphite builds in one
  process is undefined.

- **Feature gate.** Without the `graphite2` Cargo feature the module does not
  exist, and `harfbuzz_sys::graphite2` is a compile error rather than a link
  error.
