# GLib integration

Header: `hb-glib.h` — Rust module: `harfbuzz_sys::glib`. The module is gated on
the crate's `glib` Cargo feature and is **not** glob re-exported at the crate
root, so its items are reached as `harfbuzz_sys::glib::*`.

## Overview

GLib carries its own copy of the Unicode Character Database — `g_unichar_type`,
`g_unichar_combining_class`, `g_unichar_get_mirror_char`,
`g_unichar_get_script`, `g_unichar_compose`, `g_unichar_decompose` — which
happens to be exactly the set of queries HarfBuzz's shaping engine needs.
`hb-glib.h` wires those six GLib functions into an `hb_unicode_funcs_t` so that
HarfBuzz can use GLib's data instead of the compact UCD copy it ships itself.

That is worth doing when the host application already links GLib, which on a
GTK/Pango stack it always does. You get two things: HarfBuzz's own Unicode
tables can be dropped from the binary, and every consumer in the process agrees
on one Unicode version instead of two that may drift apart between releases.

The header also carries a script-enumeration bridge in both directions and one
convenience constructor:

* `hb_glib_get_unicode_funcs()` — the GLib-backed `hb_unicode_funcs_t`.
* `hb_glib_script_to_script()` / `hb_glib_script_from_script()` — convert
  between GLib's `GUnicodeScript` and HarfBuzz's `hb_script_t`. Both are
  one-liners over GLib's own `g_unicode_script_to_iso15924` /
  `g_unicode_script_from_iso15924`, so ISO 15924 is the common ground and the
  numeric values of either enumeration are irrelevant.
* `hb_glib_blob_create()` — wrap a `GBytes` as an `hb_blob_t` without copying,
  so font data can cross between the two libraries with its lifetime intact.

**Nothing here changes how HarfBuzz shapes.** There is no GLib shaper; only the
Unicode property lookups change hands.

### Building

The `glib` Cargo feature makes `build.rs` probe for the `glib-2.0` package with
`pkg-config`, emit its link directives, define `HB_HAS_GLIB`, and add
`hb-glib.cc` to the amalgamation. GLib is a system library — without it the
feature does not build. Because the module is gated on the same feature that
compiles the sources, a declaration in `harfbuzz_sys::glib` can never refer to a
symbol left out of the archive.

## Types

### `GUnicodeScript`

```c
/* <glib.h> — GLib, not HarfBuzz */
typedef enum { G_UNICODE_SCRIPT_INVALID_CODE = -1, G_UNICODE_SCRIPT_COMMON = 0, ... } GUnicodeScript;
```

```rust
pub type GUnicodeScript = core::ffi::c_int;
```

GLib's script enumeration. It belongs to GLib, and this crate has no GLib
dependency to import it from, so it is declared locally as a plain C `int` —
which is what the C compiler gives the enumeration, since it has a negative
enumerator (`G_UNICODE_SCRIPT_INVALID_CODE = -1`) and every value fits in an
`int`. That makes the alias ABI-compatible with the `GUnicodeScript` of the
`glib`/`glib-sys` crates; pass values straight through with `as`.

The individual script values are GLib's and grow with each Unicode release, so
they are not restated here. Take them from whatever `glib` crate you use.

### `GBytes`

```c
/* <glib.h> — GLib, not HarfBuzz */
typedef struct _GBytes GBytes;
```

```rust
crate::opaque_handle! { GBytes }
```

GLib's reference-counted, immutable byte buffer. Opaque in C and opaque here,
for the same reason as `GUnicodeScript`: the crate names the type so that the
signature of `hb_glib_blob_create` can be written honestly, without binding GLib
itself. In Rust it is a zero-sized `#[repr(C)]` handle that exists only behind
`*mut GBytes`.

### `hb_unicode_funcs_t`

Defined in `hb-unicode.h`, documented with the rest of the Unicode API. An
opaque, reference-counted vtable of the six Unicode property callbacks shaping
needs.

### `hb_script_t`

Defined in `hb-common.h`. In this crate `pub type hb_script_t = c_int` holding
an ISO 15924 four-letter tag, with `HB_SCRIPT_INVALID == HB_TAG_NONE == 0`.

### `hb_blob_t`

Defined in `hb-blob.h`; see `blob.md`. `hb_glib_blob_create` returns one.

## Functions

### `hb_glib_get_unicode_funcs`

```c
hb_unicode_funcs_t *hb_glib_get_unicode_funcs (void);
```

```rust
pub fn hb_glib_get_unicode_funcs() -> *mut hb_unicode_funcs_t;
```

Fetches a Unicode-functions structure populated with the appropriate GLib
function for each method — combining class, general category, mirroring, script,
compose, and decompose.

**Parameters** — none.

**Returns** — a pointer to the GLib-backed `hb_unicode_funcs_t`. It is a
process-wide singleton: created on first use by HarfBuzz's lazy loader, made
immutable, cached, and released at process exit. Repeated calls return the same
pointer. Not documented as ever being null; on allocation failure the lazy
loader yields HarfBuzz's shared "empty" ufuncs object rather than null.

**Ownership** — upstream annotates the return `transfer none`. Ownership is
**not** transferred: do not call `hb_unicode_funcs_destroy` on it unless you
first took your own reference with `hb_unicode_funcs_reference`.

**Notes** — Since HarfBuzz 0.9.38. Initialisation goes through HarfBuzz's
atomic lazy loader, so concurrent first calls are safe; the object is immutable
and can be shared across threads.

One implementation detail worth knowing: HarfBuzz's
`hb_unicode_general_category_t` and GLib's `GUnicodeType` are deliberately
identical enumerations, so the general-category callback is a straight cast with
no translation table.

### `hb_glib_script_to_script`

```c
hb_script_t hb_glib_script_to_script (GUnicodeScript script);
```

```rust
pub fn hb_glib_script_to_script(script: GUnicodeScript) -> hb_script_t;
```

Fetches the `hb_script_t` that corresponds to the specified `GUnicodeScript`.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `script` | A GLib script identifier. | Not a pointer. Any `int` is accepted. |

**Returns** — the `hb_script_t` found. The whole implementation is
`(hb_script_t) g_unicode_script_to_iso15924 (script)`, so the result is
whatever ISO 15924 tag GLib reports; GLib returns the tag for `Zzzz` (unknown)
for values it does not recognise. Note that the tag is *not* run through
HarfBuzz's own canonicalisation, so it comes back exactly as GLib spells it.

**Ownership** — nothing is allocated.

**Notes** — Since HarfBuzz 0.9.38. Pure function, thread-safe.

### `hb_glib_script_from_script`

```c
GUnicodeScript hb_glib_script_from_script (hb_script_t script);
```

```rust
pub fn hb_glib_script_from_script(script: hb_script_t) -> GUnicodeScript;
```

Fetches the `GUnicodeScript` identifier that corresponds to the specified
`hb_script_t`.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `script` | A HarfBuzz script tag. | Not a pointer. |

**Returns** — the `GUnicodeScript` found. The whole implementation is
`g_unicode_script_from_iso15924 (script)`, so GLib decides: a tag GLib does not
know yields `G_UNICODE_SCRIPT_UNKNOWN`, and a malformed tag yields
`G_UNICODE_SCRIPT_INVALID_CODE` (`-1`).

**Ownership** — nothing is allocated.

**Notes** — Since HarfBuzz 0.9.38. Pure function, thread-safe.

### `hb_glib_blob_create`

```c
hb_blob_t *hb_glib_blob_create (GBytes *gbytes);
```

```rust
pub fn hb_glib_blob_create(gbytes: *mut GBytes) -> *mut hb_blob_t;
```

Creates an `hb_blob_t` from the specified `GBytes` structure, **without copying
the data**.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `gbytes` | The GLib byte buffer to wrap. | The header does not annotate nullability; the implementation calls `g_bytes_get_data` on it immediately, so treat null as forbidden. |

**Returns** — a new `hb_blob_t` wrapping the bytes. Built with
`hb_blob_create()`, so it follows that function's convention: it **never returns
null**, and a zero-length `GBytes` yields the singleton empty blob. It is not an
`_or_fail` constructor, so allocation failure is invisible.

**Ownership** — this is the interesting part, and it is symmetric:

- The blob's memory mode is `HB_MEMORY_MODE_READONLY` and the data pointer is
  GLib's, so **no copy is made**.
- The call takes its own reference with `g_bytes_ref(gbytes)` and installs
  `g_bytes_unref` as the blob's destroy callback. The `GBytes` therefore
  outlives the blob automatically; the caller keeps its own reference and
  unrefs it whenever it likes.
- Upstream annotates the return `transfer full`: the caller owns the blob and
  must release it with `hb_blob_destroy()`.

**Notes** — Since HarfBuzz 0.9.38. Upstream guards the declaration behind
`GLIB_CHECK_VERSION(2, 31, 10)`, the release that introduced `GBytes`. Every
GLib new enough to satisfy HarfBuzz's own minimum ships `GBytes` in practice, so
this crate declares the function unconditionally — calling it against an
impossibly old GLib is a link error rather than a compile error.

Because the mode is `HB_MEMORY_MODE_READONLY` and `GBytes` is immutable by
contract, this is exactly the case that mode was designed for: no copy, no
mutation, and a destroy callback that ends the borrow.

## Usage

### C: shape with GLib's Unicode data

```c
#include <hb.h>
#include <hb-glib.h>

hb_buffer_t *buf = hb_buffer_create ();

/* Route Unicode property queries through GLib. The buffer takes its own
   reference; the singleton itself is never destroyed by us. */
hb_buffer_set_unicode_funcs (buf, hb_glib_get_unicode_funcs ());

hb_buffer_add_utf8 (buf, text, -1, 0, -1);
hb_buffer_guess_segment_properties (buf);   /* now uses GLib for script detection */
hb_shape (font, buf, NULL, 0);
```

Set the unicode funcs *before* `hb_buffer_guess_segment_properties()`: that call
uses the script callback to infer the buffer's script.

### C: map a font file through `GMappedFile` into a face

```c
GError    *err   = NULL;
GMappedFile *map = g_mapped_file_new ("font.ttf", FALSE, &err);
GBytes    *bytes = g_mapped_file_get_bytes (map);

hb_blob_t *blob  = hb_glib_blob_create (bytes);   /* refs `bytes`, no copy */
hb_face_t *face  = hb_face_create (blob, 0);

g_bytes_unref (bytes);        /* the blob holds its own reference */
g_mapped_file_unref (map);    /* the GBytes holds its own reference */
hb_blob_destroy (blob);       /* the face holds its own reference */

/* ... use face ... */
hb_face_destroy (face);
```

### Rust: wrapping a `GBytes` you got from a GLib crate

```rust
use core::ffi::c_void;
use harfbuzz_sys::{hb_blob_destroy, hb_blob_t};
use harfbuzz_sys::glib::{hb_glib_blob_create, GBytes};

/// Wrap a `GBytes` as a HarfBuzz blob without copying.
///
/// # Safety
/// `gbytes` must be a live, non-null `GBytes` pointer.
unsafe fn blob_from_gbytes(gbytes: *mut GBytes) -> *mut hb_blob_t {
    // SAFETY: `gbytes` is live by the caller's contract. HarfBuzz takes its own
    // g_bytes_ref, so the caller's reference stays independently owned.
    unsafe { hb_glib_blob_create(gbytes) }
}

/// # Safety
/// `blob` must have come from `blob_from_gbytes` and not been destroyed yet.
unsafe fn release(blob: *mut hb_blob_t) {
    // SAFETY: single matching release for the `transfer full` return above.
    unsafe { hb_blob_destroy(blob) };
}
```

If you use the `glib` crate, `glib::Bytes` derefs to a `*mut glib_sys::GBytes`,
which is the same pointer type after an `as` cast — the two `GBytes` handles are
ABI-identical opaque structs.

### Rust: attaching the GLib unicode funcs

```rust
use harfbuzz_sys::{hb_buffer_set_unicode_funcs, hb_buffer_t};
use harfbuzz_sys::glib::hb_glib_get_unicode_funcs;

/// # Safety
/// `buf` must be a live, non-null buffer.
unsafe fn use_glib_unicode_funcs(buf: *mut hb_buffer_t) {
    // SAFETY: the returned ufuncs is a permanently valid process-wide
    // singleton and `buf` is live by the caller's contract. The buffer takes
    // its own reference, so we must not destroy the singleton.
    unsafe { hb_buffer_set_unicode_funcs(buf, hb_glib_get_unicode_funcs()) };
}
```

## Pitfalls

- **The singleton is not yours to destroy.** `hb_glib_get_unicode_funcs()` is
  `transfer none`; destroying it without a matching reference frees an object
  HarfBuzz still owns.

- **`hb_glib_blob_create` never returns null.** It is built on `hb_blob_create`,
  not on `hb_blob_create_or_fail`, so allocation failure and a zero-length
  `GBytes` both come back as the singleton empty blob. Check
  `hb_blob_get_length()` if you need to tell an empty font from a failure.

- **Do not unref the `GBytes` "on behalf of" the blob.** The blob took its own
  reference at creation and drops it from its destroy callback. Your reference
  is yours; unref it exactly once, whenever you are done with it.

- **The blob borrows GLib's bytes.** The pointer the blob hands out
  (`hb_blob_get_data`) points into the `GBytes`. That is safe because of the
  reference the blob holds — but it also means `hb_blob_get_data_writable()`
  will have to copy, and the blob's data must never be mutated through any other
  route.

- **Script conversion is GLib's opinion, not HarfBuzz's.**
  `hb_glib_script_to_script` does not canonicalise the tag through
  `hb_script_from_string`, so what you get is literally
  `g_unicode_script_to_iso15924`'s output cast to `hb_script_t`. For unknown
  input that is the `Zzzz` tag, not `HB_SCRIPT_INVALID`.

- **Order matters when guessing segment properties**, exactly as with the ICU
  integration: set the unicode funcs before
  `hb_buffer_guess_segment_properties()`.

- **Feature gate.** Without the `glib` Cargo feature the module does not exist,
  and `harfbuzz_sys::glib` is a compile error rather than a link error.
