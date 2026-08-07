# ICU integration

Header: `hb-icu.h` — Rust module: `harfbuzz_sys::icu`. The module is gated on
the crate's `icu` Cargo feature and is **not** glob re-exported at the crate
root, so its items are reached as `harfbuzz_sys::icu::*`.

## Overview

Shaping needs Unicode character properties. Before HarfBuzz can decide how to
reorder an Indic cluster, where to place a combining mark, or which script a run
belongs to, it must be able to ask, for each code point: what is its General
Category? its Canonical Combining Class? its mirrored form? its Script? what
does it compose to, and decompose into? HarfBuzz answers those six questions
through an `hb_unicode_funcs_t` — a vtable of six callbacks — and ships its own
compact copy of the Unicode Character Database (the built-in `ucd` provider) to
back the default one.

`hb-icu.h` offers an alternative source for exactly that data: the
International Components for Unicode. `hb_icu_get_unicode_funcs()` returns a
process-wide, immutable `hb_unicode_funcs_t` whose six virtual methods call into
ICU's `u_getIntPropertyValue`, `u_charType`, `u_charMirror`, `uscript_getScript`,
and the normalizer. Attach it to a buffer with `hb_buffer_set_unicode_funcs()`
and shaping uses ICU's Unicode data instead of HarfBuzz's.

The usual reason to do this is **consistency, not speed**. A program that
already links ICU — for line breaking, collation, bidi, normalization,
transliteration — otherwise has two independent copies of the UCD in the
process, potentially at two different Unicode versions. Routing HarfBuzz's
Unicode queries through ICU makes one library the single authority. A secondary
benefit in size-constrained builds is that HarfBuzz's own tables can be dropped
(`HB_NO_UCD`) once ICU supplies the data.

The other two functions translate between the two libraries' script
enumerations. HarfBuzz spells a script as an ISO 15924 four-letter tag packed
into an `hb_script_t`; ICU spells it as a numeric `UScriptCode`. Both
conversions route through the ISO 15924 short name rather than through any
numeric correspondence, so they stay correct as ICU adds scripts and as
HarfBuzz's own script list grows independently.

**Nothing here changes how HarfBuzz shapes.** The OpenType shaper, the font
machinery, the buffer API, and the glyph output are all unaffected; only the
answer to "what is this code point?" comes from somewhere else. There is no ICU
*shaper* back end in HarfBuzz.

### Building

The `icu` Cargo feature does three things in `build.rs`:

1. Probes for the `icu-uc` package with `pkg-config` and emits its link
   directives. ICU is a system library; without it the feature fails to build.
2. Defines `HB_HAS_ICU` and `HAVE_ICU` and adds `hb-icu.cc` to the
   amalgamation. (Upstream's `harfbuzz-world.cc` never translates `HB_HAS_ICU`
   into `HAVE_ICU`, so `build.rs` defines both explicitly.)
3. Bumps the C++ standard to C++17 when the detected ICU major version is 75 or
   later, because ICU's public headers require it from that release on, and
   keeps RTTI enabled, because ICU's headers use it.

Because the module is gated on the same feature that compiles the sources, a
declaration in `harfbuzz_sys::icu` can never refer to a symbol left out of the
archive.

## Types

### `UScriptCode`

```c
/* <unicode/uscript.h> — ICU, not HarfBuzz */
typedef enum UScriptCode { USCRIPT_INVALID_CODE = -1, USCRIPT_COMMON = 0, ... } UScriptCode;
```

```rust
pub type UScriptCode = core::ffi::c_int;
```

ICU's script enumeration. It belongs to ICU, not to HarfBuzz, and this crate has
no ICU dependency to import it from, so it is declared locally as what the C
compiler gives it: because the enumeration has a negative enumerator
(`USCRIPT_INVALID_CODE = -1`) and every value fits in an `int`, its underlying
type is `int`. The alias is therefore ABI-compatible with the `UScriptCode` of
the `icu` and `icu_sys` crates — pass values straight through with `as`.

The values run from `-1` upwards in the order ICU happened to add scripts:
`0` is `USCRIPT_COMMON`, `1` is `USCRIPT_INHERITED`, and so on. Only the
sentinel is restated in this crate; take the individual script codes from ICU's
own headers or bindings, since ICU appends new ones as Unicode grows.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `USCRIPT_INVALID_CODE` | -1 | ICU's "not a script" sentinel. |

### `hb_script_t`

Defined in `hb-common.h` and documented on the shaping pages, not here. In this
crate it is `pub type hb_script_t = c_int` holding an ISO 15924 tag, with
`HB_SCRIPT_INVALID` equal to `HB_TAG_NONE` (zero).

### `hb_unicode_funcs_t`

Defined in `hb-unicode.h`. An opaque, reference-counted table of the six Unicode
property callbacks that shaping needs. `hb_icu_get_unicode_funcs()` returns one
of these; everything you do with it afterwards — `hb_buffer_set_unicode_funcs`,
`hb_unicode_funcs_reference`, `hb_unicode_funcs_destroy` — is the ordinary
`hb-unicode.h` API.

## Functions

### `hb_icu_get_unicode_funcs`

```c
hb_unicode_funcs_t *hb_icu_get_unicode_funcs (void);
```

```rust
pub fn hb_icu_get_unicode_funcs() -> *mut hb_unicode_funcs_t;
```

Fetches a Unicode-functions structure populated with the appropriate ICU
function for each method: combining class, general category, mirroring, script,
compose, and decompose.

**Parameters** — none.

**Returns** — a pointer to the ICU-backed `hb_unicode_funcs_t`. It is a
process-wide singleton: built on first use by a lazy loader, made immutable,
cached, and released at process exit (unless `HB_NO_ATEXIT`). Repeated calls
return the same pointer. Not documented as ever being null; the lazy loader
falls back to HarfBuzz's shared "empty" ufuncs object on allocation failure
rather than returning null.

**Ownership** — upstream annotates the return `transfer none`. Ownership is
**not** transferred: do not call `hb_unicode_funcs_destroy` on it unless you
first took your own reference with `hb_unicode_funcs_reference`.

**Notes** — Since HarfBuzz 0.9.38. The lazy initialisation goes through
HarfBuzz's atomic lazy-loader, so concurrent first calls from several threads
are safe. The object is immutable, so it can be shared freely across threads.

### `hb_icu_script_to_script`

```c
hb_script_t hb_icu_script_to_script (UScriptCode script);
```

```rust
pub fn hb_icu_script_to_script(script: UScriptCode) -> hb_script_t;
```

Fetches the `hb_script_t` that corresponds to the specified `UScriptCode`.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `script` | An ICU script code. | Not a pointer. Any `int` is accepted; unknown values are handled, see below. |

**Returns** — the corresponding `hb_script_t`. The implementation special-cases
`USCRIPT_INVALID_CODE`, returning `HB_SCRIPT_INVALID`; otherwise it calls
`uscript_getShortName()` and feeds the result to `hb_script_from_string()`. So:

- an ICU code ICU does not recognise yields a null short name and comes back as
  `HB_SCRIPT_INVALID`;
- a script ICU knows but HarfBuzz does not comes back as the four-letter tag
  itself, which is what `hb_script_from_string` produces for an unrecognised
  ISO 15924 code — a valid, usable `hb_script_t`, just not one of the named
  `HB_SCRIPT_*` constants.

**Ownership** — nothing is allocated; the return is a plain integer.

**Notes** — the header carries no `Since:` annotation and neither does the
implementation's gtk-doc block; upstream's `NEWS` lists this function under
HarfBuzz 0.6.0. Pure function, thread-safe.

### `hb_icu_script_from_script`

```c
UScriptCode hb_icu_script_from_script (hb_script_t script);
```

```rust
pub fn hb_icu_script_from_script(script: hb_script_t) -> UScriptCode;
```

Fetches the `UScriptCode` that corresponds to the specified `hb_script_t`.

**Parameters**

| Parameter | Meaning | Null / range |
| --- | --- | --- |
| `script` | A HarfBuzz script tag. | Not a pointer. `HB_SCRIPT_INVALID` is handled explicitly. |

**Returns** — the matching `UScriptCode`, or `USCRIPT_INVALID_CODE` on failure.
The implementation unpacks the tag into a five-byte NUL-terminated buffer and
hands it to ICU's `uscript_getCode()`, requesting one result. ICU's `UErrorCode`
is **discarded**, so `USCRIPT_INVALID_CODE` is the only failure signal you get —
and you get it for `HB_SCRIPT_INVALID`, for a tag ICU does not recognise, and for
scripts too new for the linked ICU alike.

**Ownership** — nothing is allocated.

**Notes** — the header carries no `Since:` annotation; upstream's `NEWS` lists
this function under HarfBuzz 0.6.0. Pure function, thread-safe.

## Usage

### C: shape with ICU's Unicode data

```c
#include <hb.h>
#include <hb-icu.h>

hb_buffer_t *buf = hb_buffer_create ();

/* Route every Unicode property query through ICU. The buffer takes its own
   reference; the singleton itself is never destroyed by us. */
hb_buffer_set_unicode_funcs (buf, hb_icu_get_unicode_funcs ());

hb_buffer_add_utf8 (buf, text, -1, 0, -1);
hb_buffer_guess_segment_properties (buf);   /* now uses ICU for script detection */
hb_shape (font, buf, NULL, 0);
```

Set the unicode funcs *before* `hb_buffer_guess_segment_properties()`, because
that is the call that uses the script callback to infer the buffer's script.

### C: cross the two script enumerations

```c
UScriptCode  icu_script = uscript_getScript (0x0915, &err);   /* DEVANAGARI LETTER KA */
hb_script_t  hb_script  = hb_icu_script_to_script (icu_script);

/* and back */
UScriptCode  round_trip = hb_icu_script_from_script (hb_script);
```

### Rust: attach the ICU funcs to a buffer

```rust
use harfbuzz_sys::{hb_buffer_set_unicode_funcs, hb_buffer_t};
use harfbuzz_sys::icu::hb_icu_get_unicode_funcs;

/// Make `buf` answer Unicode property queries through ICU.
///
/// # Safety
/// `buf` must be a live, non-null buffer.
unsafe fn use_icu_unicode_funcs(buf: *mut hb_buffer_t) {
    // SAFETY: the returned ufuncs is a permanently valid process-wide
    // singleton, and `buf` is a live buffer by the caller's contract.
    // `hb_buffer_set_unicode_funcs` takes its own reference, so we must not
    // destroy the singleton afterwards.
    unsafe {
        let ufuncs = hb_icu_get_unicode_funcs();
        hb_buffer_set_unicode_funcs(buf, ufuncs);
    }
}
```

### Rust: converting scripts

```rust
use harfbuzz_sys::{hb_script_t, HB_SCRIPT_INVALID};
use harfbuzz_sys::icu::{
    hb_icu_script_from_script, hb_icu_script_to_script, UScriptCode, USCRIPT_INVALID_CODE,
};

fn script_from_icu(code: UScriptCode) -> Option<hb_script_t> {
    // SAFETY: no pointers are involved; the call cannot fail unsafely.
    let script = unsafe { hb_icu_script_to_script(code) };
    (script != HB_SCRIPT_INVALID).then_some(script)
}

fn script_to_icu(script: hb_script_t) -> Option<UScriptCode> {
    // SAFETY: as above.
    let code = unsafe { hb_icu_script_from_script(script) };
    (code != USCRIPT_INVALID_CODE).then_some(code)
}
```

If you already depend on the `icu` or `icu_sys` crate, its `UScriptCode` is the
same `int` underneath, so `code as i32` / `i32 as icu::UScriptCode` is the whole
conversion.

## Pitfalls

- **The singleton is not yours to destroy.** `hb_icu_get_unicode_funcs()` is
  `transfer none`. Calling `hb_unicode_funcs_destroy` on it without a matching
  `hb_unicode_funcs_reference` will eventually free an object HarfBuzz still
  believes it owns.

- **Order matters when guessing segment properties.**
  `hb_buffer_guess_segment_properties()` reads the script through the buffer's
  unicode funcs. Setting the ICU funcs after that call leaves you with a script
  guessed from HarfBuzz's own tables.

- **`hb_icu_script_from_script` swallows ICU's error code.** There is no way to
  distinguish "ICU has never heard of this tag" from "ICU ran out of memory".
  `USCRIPT_INVALID_CODE` covers both.

- **Round-tripping is not guaranteed to be lossless in either direction.** A
  script HarfBuzz knows but the linked ICU does not maps to
  `USCRIPT_INVALID_CODE`; a script ICU knows but HarfBuzz does not maps to a raw
  four-letter tag. Both are the correct behaviour, but neither round-trips.

- **This is a Unicode-data back end, not a shaper.** If you were looking for
  ICU's own layout engine, it is not here and HarfBuzz does not call it.

- **Two Unicode versions can still disagree.** Swapping in ICU's data fixes
  disagreements between HarfBuzz and the rest of *your* stack only if the rest
  of your stack is that same ICU. Mixing ICU-backed HarfBuzz with a separate
  GLib-backed component reintroduces the problem the integration was meant to
  solve.

- **Feature gate.** Without the `icu` Cargo feature the module does not exist,
  and `harfbuzz_sys::icu` is a compile error rather than a link error — which is
  the intended behaviour.
