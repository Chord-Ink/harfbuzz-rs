# OpenType font funcs

Transcribed from `hb-ot-font.h`. Rust module: `harfbuzz_sys::ot_font`, glob
re-exported at the crate root.

## Overview

`hb-ot-font.h` is one of the smallest public headers in HarfBuzz. It declares no
types, no constants and no macros — just one function, `hb_ot_font_set_funcs`.
Its upstream description is short enough to quote whole: *"Functions for using
OpenType fonts with `hb_shape()`. Note that fonts returned by
`hb_font_create()` default to using these functions, so most clients would
never need to call these functions directly."*

To understand why the header exists at all, you need the shape of `hb-font.h`.
An `hb_font_t` does not know how to answer questions about itself. Every
question — *what glyph is U+0041? how wide is glyph 36? what is its outline?
what colour layers does it have?* — is dispatched through a table of callbacks,
an `hb_font_funcs_t`, together with an opaque `font_data` pointer that those
callbacks interpret. That indirection is what lets HarfBuzz shape text using
FreeType, CoreText, DirectWrite, or a font format HarfBuzz has never heard of:
you supply the callbacks, HarfBuzz supplies the shaping.

`hb-ot-font` is HarfBuzz's own implementation of that callback table, reading
the binary OpenType tables in the face directly — `cmap` for character mapping,
`hmtx`/`vmtx` for advances, `glyf`/`CFF`/`CFF2` for outlines, `COLR`, `sbix`,
`CBDT` and `SVG` for colour, `gvar` and `VARC` for variations, `post` for glyph
names. It is fast, it is self-contained, it needs no external font library, and
it is the reason `harfbuzz-sys` can shape text with no dependency beyond a C++
compiler. Upstream's own `CONFIG.md` calls using it "highly recommended".

This header therefore does one thing: it lets you *install* that
implementation, explicitly, on a font you already have. Because
`hb_font_create` installs it for you (via `hb_font_set_funcs_using(font,
NULL)`), calling it on a fresh font is a redundant no-op in effect — slightly
wasteful, because it discards a perfectly good callback table and per-font
cache and builds new ones, but harmless. The call earns its keep in exactly
three situations: putting a font *back* on the native implementation after some
other back end was installed; forcing the native implementation on a build or
process where the `HB_FONT_FUNCS` environment variable has changed the default;
and installing it on a sub-font, which by default inherits from its parent
rather than reading the face.

The whole of `hb-ot-font.cc` is wrapped in `#ifndef HB_NO_OT_FONT`. A build that
defines `HB_NO_OT_FONT` — directly, or via `HB_NO_OT`, which implies it — has no
such symbol at all, and linking against it fails. Upstream's `CONFIG.md` names
the one situation where that is reasonable: an embedded client that provides
font functions itself, typically FreeType through `hb-ft.h`, on every font it
creates. `harfbuzz-sys` never defines either macro, so for this crate the
function is always present.

## Types

This header declares no types of its own.

### `hb_font_t`

The single parameter type. Declared in `hb-font.h`, transcribed in
`crate::font`, and re-exported at the crate root. It is an opaque,
reference-counted object representing a face at a particular scale and
configuration. See `docs/font.md` for its full API.

Two properties of `hb_font_t` matter for this header:

* A font holds a `hb_font_funcs_t *klass`, a `void *user_data` (the
  `font_data` those callbacks receive), and a `hb_destroy_func_t destroy` for
  that data. `hb_ot_font_set_funcs` sets all three at once.
* A font can be made immutable with `hb_font_make_immutable`, after which
  every setter — including this one — silently does nothing.

### `hb_font_funcs_t`

Not a parameter, but the thing this function installs. Also declared in
`hb-font.h`. `hb-ot-font` builds exactly one of these per process: a lazily
created, immutable, atexit-freed singleton shared by every font that uses the
native implementation. You never see the pointer, and you must not try to
destroy it.

## Functions

### `hb_ot_font_set_funcs`

```c
void hb_ot_font_set_funcs (hb_font_t *font);
```

```rust
pub fn hb_ot_font_set_funcs(font: *mut hb_font_t);
```

Sets the font functions to use when working with `font` to HarfBuzz's native
OpenType implementation. This is the default for fonts newly created with
`hb_font_create`.

**Parameters**

| Name   | C type       | Rust type        | Meaning                                                                 |
| ------ | ------------ | ---------------- | ----------------------------------------------------------------------- |
| `font` | `hb_font_t *`| `*mut hb_font_t` | The font to modify, in place. See nullability below.                    |

Nullability: the header does not annotate the parameter, and the gtk-doc
comment does not mention null. The shipped implementation dereferences
`font->face` before it does anything else, so a null pointer is a
dereference-of-null, not a graceful no-op. Treat `font` as required. The
singleton empty font from `hb_font_get_empty()` is a valid pointer but is
immutable, so passing it is safe and does nothing.

**Returns** — nothing. There is no success/failure channel at all; see Notes.

**Ownership**

* `font` is borrowed for the duration of the call. No reference is taken, and
  the caller still owns its reference and must still call `hb_font_destroy`.
* The function allocates a private per-font state object (an internal
  `hb_ot_font_t` holding the face's parsed-table accessor plus advance and
  origin caches) and attaches it to the font as the font data, along with a
  destroy callback. The font owns it from then on; it is freed when the font is
  destroyed or when the font's funcs/data are replaced again.
* **Any font data previously attached to `font` is destroyed.** Installing font
  funcs is a single atomic replacement of `(klass, font_data, destroy)`; the
  old data's destroy callback runs before the new data is stored. If you
  attached your own `font_data` with `hb_font_set_funcs` or
  `hb_font_set_funcs_data`, this call throws it away. User data attached with
  `hb_font_set_user_data` — a different, key-based mechanism — is *not*
  affected.
* The `hb_font_funcs_t` it installs is a process-global immutable singleton
  owned by HarfBuzz. It is reference-counted internally by the font; do not
  destroy it, and do not attempt to mutate it (it is immutable, so setters on
  it would be no-ops anyway).

**Notes**

* Since HarfBuzz 0.9.28.
* Requires a build without `HB_NO_OT_FONT`. `harfbuzz-sys` always builds it.
* **Silent failure, twice over.** The function returns `void`, and both of its
  failure modes are invisible:
  1. If allocating the per-font state fails (out of memory), the function
     returns immediately and the font keeps whatever funcs it had.
  2. If the font is immutable, the underlying `hb_font_set_funcs` is a no-op —
     it destroys the freshly allocated state (so nothing leaks) and returns
     without changing the font.
  In both cases you get no indication. If you need certainty, use
  `hb_font_set_funcs_using(font, "ot")` instead, which returns `hb_bool_t`.
* Not thread-safe with respect to `font`: it mutates the font object. Follow
  the usual HarfBuzz rule — configure a font on one thread, call
  `hb_font_make_immutable`, and only then share it. The funcs singleton it
  installs is created under an atomic compare-exchange, so concurrent first
  calls from several threads are safe in a multi-threaded build.
* Calling it twice is harmless: the second call replaces the first call's state
  object with a fresh one, freeing the first (and discarding its warmed caches).

#### What the native implementation actually provides

The header does not say, but this is the practical question, so here is the
callback table `hb-ot-font` installs, as built in `hb-ot-font.cc`. These are the
callbacks your `hb_font_get_*` / `hb_font_draw_*` / `hb_font_paint_*` calls end
up in.

| Installed callback         | Where the answer comes from, in order                                            | Build guard                        |
| -------------------------- | -------------------------------------------------------------------------------- | ---------------------------------- |
| `nominal_glyph`            | `cmap`                                                                            | always                             |
| `nominal_glyphs` (batch)   | `cmap`                                                                            | always                             |
| `variation_glyph`          | `cmap` format 14 (Unicode variation sequences)                                    | always                             |
| `font_h_extents`           | `OS/2` typo metrics when the *use typo metrics* flag is set, else `hhea`; `MVAR` deltas applied | always              |
| `glyph_h_advances` (batch) | `hmtx`; with non-default variation coords, `HVAR`, else `gvar` + `glyf` phantom points; half the upem if `hmtx` is missing | always |
| `font_v_extents`           | `vhea` ascender/descender/line gap; `MVAR` deltas applied                          | unless `HB_NO_VERTICAL`            |
| `glyph_v_advances` (batch) | `vmtx`; `VVAR` or `gvar` + `glyf` under variation                                 | unless `HB_NO_VERTICAL`            |
| `glyph_v_origins` (batch)  | `VORG` (+`VVAR`), else `vmtx` + `glyf` phantom points (+`gvar`), else glyph extents centred against the font ascender | unless `HB_NO_VERTICAL` |
| `glyph_extents`            | `sbix`, `CBDT`, `COLR`, `VARC`, `glyf`, `CFF2`, `CFF` — first hit wins            | always                             |
| `draw_glyph_or_fail`       | `VARC`, `glyf` (+`gvar`), `CFF2`, `CFF` — first hit wins                           | unless `HB_NO_DRAW`                |
| `paint_glyph_or_fail`      | `COLR`, `SVG`, `CBDT`, `sbix` — first hit wins                                     | unless `HB_NO_PAINT`               |
| `glyph_name`               | `post`, then `CFF` charset                                                        | unless `HB_NO_OT_FONT_GLYPH_NAMES` |
| `glyph_from_name`          | `post`, then `CFF` charset                                                        | unless `HB_NO_OT_FONT_GLYPH_NAMES` |

Three deliberate omissions are worth knowing:

* **No horizontal origin callback.** `hb_font_get_glyph_h_origin` therefore
  falls through to the parent font and ends at the empty font's nil
  implementation, which reports `(0, 0)`. That is the correct horizontal origin
  for OpenType, so this is an optimisation, not a gap.
* **Only the batch forms of advances and origins are installed.** HarfBuzz's
  default singular callbacks dispatch to the batch callback with a count of
  one, so `hb_font_get_glyph_h_advance` works exactly as you would expect —
  just slightly less efficiently than asking for a whole run at once.
* **No contour-point callback.** `hb_font_get_glyph_contour_point` is not
  implemented by `hb-ot-font` (the registration line is commented out
  upstream); it falls back to the parent chain and generally fails. Use the
  draw API instead.

#### Relationship to `hb_font_set_funcs_using`

Since HarfBuzz 11.0.0, `hb-font.h` offers a name-based front door to the same
implementations:

```c
hb_bool_t hb_font_set_funcs_using (hb_font_t *font, const char *name);
const char **hb_font_list_funcs (void);
```

The name for this header's implementation is `"ot"`, and
`hb_font_set_funcs_using(font, "ot")` calls `hb_ot_font_set_funcs` directly.
The other registered names — each present only if that back end was compiled
in — are `"ft"`, `"fontations"`, `"coretext"` and `"directwrite"`;
`hb_font_list_funcs` enumerates what this build actually has.

Two differences make `hb_font_set_funcs_using` the better call when you can use
it:

* It returns `hb_bool_t`, so an immutable font or an unknown name is reported
  rather than swallowed. (It does not report an out-of-memory failure inside
  the chosen setter, though.)
* It exists on all builds, whereas `hb_ot_font_set_funcs` disappears under
  `HB_NO_OT_FONT`.

Passing `NULL` or `""` as the name selects the default: the value of the
`HB_FONT_FUNCS` environment variable if set, otherwise the first back end in
the built-in list that successfully installs itself — which is `"ot"` whenever
`hb-ot-font` is compiled in. That is precisely what `hb_font_create` does.

## Usage

### C: shape with the native implementation, explicitly

```c
#include <hb.h>
#include <hb-ot.h>

hb_blob_t *blob = hb_blob_create_from_file_or_fail ("Roboto.ttf");
hb_face_t *face = hb_face_create (blob, 0);
hb_font_t *font = hb_font_create (face);

/* Already the default — this is the belt-and-braces version, useful if the
   process might have HB_FONT_FUNCS set in its environment. */
hb_ot_font_set_funcs (font);

hb_font_set_scale (font, 20 * 64, 20 * 64);

hb_buffer_t *buf = hb_buffer_create ();
hb_buffer_add_utf8 (buf, "Hello", -1, 0, -1);
hb_buffer_guess_segment_properties (buf);
hb_shape (font, buf, NULL, 0);

/* ... read hb_buffer_get_glyph_infos / _positions ... */

hb_buffer_destroy (buf);
hb_font_destroy (font);
hb_face_destroy (face);
hb_blob_destroy (blob);
```

Note the include: `hb-ot-font.h` refuses to be included directly (it
`#error`s unless `HB_OT_H_IN` is defined). Include `<hb-ot.h>`. In Rust this is
irrelevant — everything is one crate.

### C: restore the native implementation after using another back end

```c
hb_ft_font_set_funcs (font);       /* now backed by FreeType */
/* ... */
hb_ot_font_set_funcs (font);       /* back to HarfBuzz's own tables */
```

The FreeType state attached by `hb_ft_font_set_funcs` is destroyed by the
second call, and the `FT_Face` it wrapped is released according to whatever
ownership `hb-ft.h` was given.

### C: force the native implementation on a sub-font

```c
hb_font_t *sub = hb_font_create_sub_font (parent);
/* By default `sub` has no funcs of its own: every query walks up to `parent`,
   with scaling applied at each hop. */
hb_ot_font_set_funcs (sub);
/* Now `sub` reads the face directly and ignores `parent`'s funcs entirely. */
```

### Rust: the same flow through `harfbuzz-sys`

```rust
use harfbuzz_sys::*;

unsafe {
    let path = c"Roboto.ttf";
    let blob = hb_blob_create_from_file_or_fail(path.as_ptr());
    assert!(!blob.is_null(), "font file not found or unreadable");

    let face = hb_face_create(blob, 0);
    let font = hb_font_create(face);

    // Redundant on a fresh font, but explicit and cheap.
    hb_ot_font_set_funcs(font);
    hb_font_set_scale(font, 20 * 64, 20 * 64);

    // Prove the native implementation is answering: look up a glyph.
    let mut glyph: hb_codepoint_t = 0;
    let found = hb_font_get_nominal_glyph(font, 'A' as hb_codepoint_t, &mut glyph);
    assert_ne!(found, 0);

    let advance = hb_font_get_glyph_h_advance(font, glyph);
    let _ = advance;

    hb_font_destroy(font);
    hb_face_destroy(face);
    hb_blob_destroy(blob);
}
```

### Rust: prefer the checked spelling when you need to know it worked

```rust
use harfbuzz_sys::{hb_font_set_funcs_using, hb_font_t};

unsafe fn use_native_funcs(font: *mut hb_font_t) -> bool {
    hb_font_set_funcs_using(font, c"ot".as_ptr()) != 0
}
```

A `false` return means the font was immutable, or that this build has no `"ot"`
back end. `hb_ot_font_set_funcs` cannot tell you either of those things.

### Rust: checking whether a font still has default funcs

There is no getter for a font's `hb_font_funcs_t`, so you cannot ask a font
which implementation it is using. If that matters, track it yourself, or
observe it indirectly — a font with no working funcs returns `0` glyphs for
everything:

```rust
unsafe {
    let mut glyph = 0;
    if hb_font_get_nominal_glyph(font, 'A' as hb_codepoint_t, &mut glyph) == 0 {
        // No usable font funcs (or the face genuinely lacks a cmap entry).
    }
}
```

## Pitfalls

**It silently destroys your font data.** This is the sharpest edge in the
header. If you called `hb_font_set_funcs(font, my_funcs, my_data, my_destroy)`
and later call `hb_ot_font_set_funcs(font)`, `my_destroy(my_data)` runs and
`my_data` is gone. Ordering matters: install the native funcs first, then your
own. Note the distinction from `hb_font_set_user_data`, whose key/value pairs
survive any number of funcs changes.

**It cannot fail loudly.** The `void` return hides both out-of-memory and
"the font is immutable". A font that was made immutable before this call keeps
its old behaviour with no diagnostic. Use `hb_font_set_funcs_using(font, "ot")`
when you need a signal, and remember that even that does not surface OOM.

**Null `font` is not tolerated.** Many HarfBuzz functions accept null and
substitute a nil object; this one does not — the implementation dereferences
`font->face` immediately. The header documents nothing about null, so this is
observed behaviour rather than a promise, but the observation is unambiguous.

**Calling it on a sub-font changes semantics, not just performance.** A
sub-font created with `hb_font_create_sub_font` has no funcs of its own, and
every query walks up to the parent with scaling applied on the way down.
Installing `hb-ot-font` on the sub-font cuts that chain: the sub-font now reads
the face directly and the parent's callbacks — including any customisation you
installed there — are ignored. That is occasionally what you want and
frequently a surprise.

**The name is misleading in one direction.** "OT font" does not mean "only
works with `.otf`". `hb-ot-font` handles TrueType outlines (`glyf`) and CFF/CFF2
outlines alike, plus bitmap and colour formats, plus variable fonts. Conversely
it does not mean "does OpenType *shaping*" — that is `hb-ot-shape`, an
unrelated subsystem selected by the shaper list, not by the font funcs.

**Do not touch the installed funcs object.** It is a process-wide immutable
singleton, freed at exit by HarfBuzz. There is no getter for it in this header,
and even if you obtain it through other means, mutating it would silently fail
and destroying it would corrupt every other font using it.

**Re-installing throws away warm caches.** The per-font state includes advance
and origin caches keyed to the font's variation-coordinate serial. Calling
`hb_ot_font_set_funcs` again allocates a fresh state and drops the populated
caches. Do not call it per shaping operation, or in a loop; call it once at
font setup.

**The header cannot be included on its own in C.** `hb-ot-font.h` begins with
an `#error` unless `HB_OT_H_IN` (or `HB_NO_SINGLE_HEADER_ERROR`) is defined.
Include `<hb-ot.h>`. Rust users of `harfbuzz-sys` are unaffected: the symbol is
re-exported at the crate root alongside everything else.

**Reduced builds can remove it, or hollow it out.** Under `HB_NO_OT_FONT` (also
implied by `HB_NO_OT`) the entire translation unit vanishes and so does this
symbol; `hb_font_list_funcs()` will not list `"ot"` either. Milder profiles keep
the function but shrink what it installs: `HB_LEAN` — which `HB_TINY` implies —
defines `HB_NO_OT_FONT_GLYPH_NAMES`, `HB_NO_VERTICAL` and `HB_NO_VAR`, and,
unless a rasterising back end is enabled, `HB_NO_COLOR`, `HB_NO_DRAW` and
`HB_NO_PAINT`. Consult the build-guard column of the callback table above before
assuming a callback is there. Code that must work against arbitrary HarfBuzz
builds should go through `hb_font_set_funcs_using` and check the result.
`harfbuzz-sys` defines none of these macros, so within this crate the function
and its full callback set are guaranteed present.
