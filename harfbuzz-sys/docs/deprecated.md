# Deprecated API

Transcribed from `hb-deprecated.h`. Rust module: `harfbuzz_sys::deprecated`,
glob re-exported at the crate root.

## Overview

`hb-deprecated.h` is not a subsystem. It is a graveyard with an ABI contract.
HarfBuzz treats its public C API and ABI as a hard constraint, so nothing that
was ever exported is ever removed; when a name turns out to be wrong, a callback
signature turns out to be under-powered, or a feature turns out to be unused,
the old declaration is moved into this header, marked with `HB_DEPRECATED` or
`HB_DEPRECATED_FOR`, and left to keep working forever. Everything here still
links, still runs, and still does what it always did. It is simply the wrong
thing to write in new code.

The header's own summary is one sentence: *"These API have been deprecated in
favor of newer API, or because they were deemed unnecessary."* Those two clauses
divide the contents cleanly, and the division is the fastest way to know what to
do about any given symbol.

**Deprecated in favour of newer API.** Most of the header. Four groups:

* *Renamed constants* — `HB_SCRIPT_CANADIAN_ABORIGINAL`,
  `HB_BUFFER_FLAGS_DEFAULT`, `HB_BUFFER_SERIALIZE_FLAGS_DEFAULT`, and the
  gloriously misspelled `HB_AAT_LAYOUT_FEATURE_TYPE_CURISVE_CONNECTION`. Each is
  a preprocessor alias for the correctly-named constant, with an identical
  value. Migration is a search-and-replace with no behavioural risk at all.
* *The combined glyph lookup* — `hb_font_get_glyph_func_t` and
  `hb_font_funcs_set_glyph_func`, which asked one callback to answer both
  "what glyph is this code point?" and "what glyph is this code point with this
  variation selector?". HarfBuzz 1.2.3 split it into
  `hb_font_get_nominal_glyph_func_t` and `hb_font_get_variation_glyph_func_t` so
  that the overwhelmingly common no-selector case could take a cheaper path
  (and, later, a batched `..._glyphs` path).
* *The outline and colour-glyph callbacks* — a three-generation lineage.
  `hb_font_get_glyph_shape_func_t` (4.0.0) was renamed to
  `hb_font_draw_glyph_func_t` (7.0.0), and both were superseded by
  `hb_font_draw_glyph_or_fail_func_t` (11.2.0). The same story runs for
  `hb_font_paint_glyph_func_t` → `hb_font_paint_glyph_or_fail_func_t`. The theme
  of every step is *the ability to report failure*: the old draw callback
  returns `void`, and the old paint callback's `hb_bool_t` return is thrown
  away by the compatibility shim, so neither can tell HarfBuzz "I have no
  outline for this glyph, fall back."
* *`hb_font_get_glyph_shape`* — the standalone query matching that lineage,
  replaced by `hb_font_draw_glyph_or_fail`.

**Deprecated because unnecessary.** The rest:

* *East Asian width* — `hb_unicode_eastasian_width` and its callback. HarfBuzz
  never consulted it for anything; it was a Unicode-functions slot that shaping
  did not need. The built-in implementation returns `1` for every code point.
* *Compatibility decomposition* — `hb_unicode_decompose_compatibility` and its
  callback and the `HB_UNICODE_MAX_DECOMPOSITION_LEN` buffer size. Shaping uses
  *canonical* decomposition, not compatibility decomposition. The built-in
  implementation returns `0` for every code point.
* *Vertical kerning* — `hb_font_get_glyph_v_kerning` and its setter. There is no
  such thing in OpenType; the slot only ever surfaced legacy `kern`-table data
  through a client-supplied callback, and HarfBuzz ships no default
  implementation that reads a font table for it.
* *`HB_UNICODE_COMBINING_CLASS_CCC133`* — a combining class Unicode has never
  assigned to any character. Removed from the enumeration in 7.2.0, kept as a
  macro so that exhaustive switches still compile.

There are no objects here: nothing to create, nothing to reference-count,
nothing to destroy. Every function in this header operates on an object owned by
another module — an `hb_font_funcs_t`, an `hb_unicode_funcs_t`, or an
`hb_font_t` — and the lifecycle rules are those modules', not this one's. The
one lifecycle rule that *is* this header's own concern is the `destroy`
callback convention shared by every setter below: **`destroy` is always called
with `user_data` exactly once, even when the setter does nothing**. See
[Pitfalls](#pitfalls).

One structural note for anyone reading upstream sources. The whole header sits
inside `#ifndef HB_DISABLE_DEPRECATED`, a build-time switch that removes all of
it from a size-constrained build. This crate does not offer that switch (the
`mini`/`lean`/`tiny` features do not set it), so every symbol below is declared
unconditionally and is present in the compiled archive. If you ever link
`harfbuzz-sys` against a *system* HarfBuzz built with `-DHB_DISABLE_DEPRECATED`,
these declarations will compile and then fail at link time.

Finally, a scope warning. Upstream's gtk-doc section named `hb-deprecated`
gathers deprecated symbols from **six** headers, not one. Only 24 of its 38
entries are declared in `hb-deprecated.h` itself; the remaining 14 live in
`hb-ot-deprecated.h`, `hb-ot-layout.h`, `hb-ft.h`, `hb-graphite2.h`, and
`hb-directwrite.h`, and therefore belong to other Rust modules in this crate.
They are documented in [Symbols from other
headers](#symbols-from-other-headers) so that this page covers the section
completely, with each one's real home called out.

## Types

Every type in this header is a function-pointer typedef — a "virtual method" of
either `hb_font_funcs_t` or `hb_unicode_funcs_t`. All of them are transcribed as
`Option<unsafe extern "C" fn(...)>` so that `None` is the null pointer, which
HarfBuzz accepts for every callback slot.

There are no structs and no enumerations in `hb-deprecated.h`. (The one
deprecated struct in the section list, `hb_ot_var_axis_t`, belongs to
`hb-ot-deprecated.h`; it is documented [below](#hb_ot_var_axis_t).)

### `hb_font_get_glyph_func_t`

```c
typedef hb_bool_t (*hb_font_get_glyph_func_t) (hb_font_t *font, void *font_data,
                                               hb_codepoint_t unicode, hb_codepoint_t variation_selector,
                                               hb_codepoint_t *glyph,
                                               void *user_data);
```
```rust
pub type hb_font_get_glyph_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        unicode: hb_codepoint_t,
        variation_selector: hb_codepoint_t,
        glyph: *mut hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;
```

The combined nominal-and-variation glyph lookup. Install it with
`hb_font_funcs_set_glyph_func`.

| Parameter            | C type            | Rust type              | Meaning                                                                     |
| -------------------- | ----------------- | ---------------------- | --------------------------------------------------------------------------- |
| `font`               | `hb_font_t *`     | `*mut hb_font_t`       | The font being queried. Never null.                                          |
| `font_data`          | `void *`          | `*mut c_void`          | The font's user data, as given to `hb_font_set_funcs`. May be null.          |
| `unicode`            | `hb_codepoint_t`  | `u32`                  | The Unicode code point to look up.                                           |
| `variation_selector` | `hb_codepoint_t`  | `u32`                  | The variation selector, or `0` for "none". See the note below.               |
| `glyph`              | `hb_codepoint_t *`| `*mut hb_codepoint_t`  | Out-parameter: write the glyph ID here. Never null.                          |
| `user_data`          | `void *`          | `*mut c_void`          | The `user_data` passed to the setter. May be null.                           |

**Returns** — `true` (non-zero) if a glyph was found and written to `glyph`;
`false` (zero) otherwise. On `false` the contents of `*glyph` are unspecified;
callers must not read it.

**The zero convention is load-bearing.** HarfBuzz's compatibility shim installs
this one callback in *both* modern slots. The nominal-glyph trampoline invokes
it with `variation_selector = 0`; the variation-glyph trampoline passes the real
selector through. So an implementation must treat `0` as "plain lookup, no
selector" rather than as a request for U+0000.

**Notes** — deprecated in HarfBuzz 1.2.3. Replaced by
`hb_font_get_nominal_glyph_func_t` plus `hb_font_get_variation_glyph_func_t`.

### `hb_unicode_eastasian_width_func_t`

```c
typedef unsigned int (*hb_unicode_eastasian_width_func_t) (hb_unicode_funcs_t *ufuncs,
                                                           hb_codepoint_t      unicode,
                                                           void               *user_data);
```
```rust
pub type hb_unicode_eastasian_width_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
        user_data: *mut c_void,
    ) -> c_uint,
>;
```

A virtual method of `hb_unicode_funcs_t` reporting the East Asian display width
of a code point — conceptually the `wcwidth`-style column count, 1 for narrow
and 2 for wide.

| Parameter   | C type                | Rust type                 | Meaning                                                    |
| ----------- | --------------------- | ------------------------- | ---------------------------------------------------------- |
| `ufuncs`    | `hb_unicode_funcs_t *`| `*mut hb_unicode_funcs_t` | The Unicode-functions object being queried. Never null.    |
| `unicode`   | `hb_codepoint_t`      | `u32`                     | The code point to measure.                                 |
| `user_data` | `void *`              | `*mut c_void`             | The `user_data` passed to the setter. May be null.         |

**Returns** — the width. The header does not define a range or a failure value.

**Notes** — deprecated in HarfBuzz 2.0.0. HarfBuzz never calls this function
internally; installing an implementation affects nothing but your own calls to
`hb_unicode_eastasian_width`. There is no replacement, because there was never a
consumer.

### `hb_unicode_decompose_compatibility_func_t`

```c
typedef unsigned int (*hb_unicode_decompose_compatibility_func_t) (hb_unicode_funcs_t *ufuncs,
                                                                   hb_codepoint_t      u,
                                                                   hb_codepoint_t     *decomposed,
                                                                   void               *user_data);
```
```rust
pub type hb_unicode_decompose_compatibility_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        u: hb_codepoint_t,
        decomposed: *mut hb_codepoint_t,
        user_data: *mut c_void,
    ) -> c_uint,
>;
```

A virtual method of `hb_unicode_funcs_t` that fully decomposes a code point to
its Unicode *compatibility* decomposition (NFKD), writing the result into a
caller-allocated array.

| Parameter    | C type                | Rust type                 | Meaning                                                                    |
| ------------ | --------------------- | ------------------------- | -------------------------------------------------------------------------- |
| `ufuncs`     | `hb_unicode_funcs_t *`| `*mut hb_unicode_funcs_t` | The Unicode-functions object. Never null.                                   |
| `u`          | `hb_codepoint_t`      | `u32`                     | The code point to decompose.                                                |
| `decomposed` | `hb_codepoint_t *`    | `*mut hb_codepoint_t`     | Out-parameter: an array of at least `HB_UNICODE_MAX_DECOMPOSITION_LEN` code points, allocated by the caller. Never null. |
| `user_data`  | `void *`              | `*mut c_void`             | The `user_data` passed to the setter. May be null.                          |

**Returns** — the number of code points written, or `0` if `u` has no
compatibility decomposition.

**Buffer contract** — the header is explicit and it matters: *"The Unicode
standard guarantees that a buffer of length `HB_UNICODE_MAX_DECOMPOSITION_LEN`
codepoints will always be sufficient for any compatibility decomposition plus a
terminating value of 0. Consequently, `decomposed` must be allocated by the
caller to be at least this length. Implementations of this function type must
ensure that they do not write past the provided array."* An implementation that
overruns corrupts the caller's stack; there is no length parameter to check
against.

**Notes** — deprecated in HarfBuzz 2.0.0. Shaping uses canonical decomposition
(`hb_unicode_decompose`), never compatibility decomposition, so nothing inside
HarfBuzz calls this. No replacement exists; use a Unicode library such as ICU if
you need NFKD.

### `hb_font_get_glyph_v_kerning_func_t`

```c
typedef hb_font_get_glyph_kerning_func_t hb_font_get_glyph_v_kerning_func_t;
```
```rust
pub type hb_font_get_glyph_v_kerning_func_t = hb_font_get_glyph_kerning_func_t;
```

A virtual method of `hb_font_funcs_t` returning the kerning adjustment for a
glyph pair in vertical text. It is a bare alias for
`hb_font_get_glyph_kerning_func_t`, which is declared in `hb-font.h` and lives
in `crate::font` — so the alias is re-exported, not redeclared, and its
parameter names are the horizontal ones:

```rust
// from crate::font
pub type hb_font_get_glyph_kerning_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        first_glyph: hb_codepoint_t,
        second_glyph: hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_position_t,
>;
```

| Parameter      | C type           | Rust type        | Meaning                                                          |
| -------------- | ---------------- | ---------------- | ---------------------------------------------------------------- |
| `font`         | `hb_font_t *`    | `*mut hb_font_t` | The font being queried. Never null.                               |
| `font_data`    | `void *`         | `*mut c_void`    | The font's user data. May be null.                                |
| `first_glyph`  | `hb_codepoint_t` | `u32`            | In vertical use, the **top** glyph of the pair.                   |
| `second_glyph` | `hb_codepoint_t` | `u32`            | In vertical use, the **bottom** glyph of the pair.                |
| `user_data`    | `void *`         | `*mut c_void`    | The `user_data` passed to the setter. May be null.                |

**Returns** — the kerning adjustment, in scaled font units (`hb_position_t`,
an `i32`).

**Notes** — the gtk-doc block for this typedef carries no `Deprecated:` tag of
its own, but its only setter, `hb_font_funcs_set_glyph_v_kerning_func`, is
marked deprecated as of 2.0.0, and the typedef is in `hb-deprecated.h`; the Rust
transcription marks it `#[deprecated]` on that basis. OpenType has no vertical
kerning mechanism, so there is no replacement — this slot only ever existed to
surface legacy data through a client callback.

### `hb_font_get_glyph_shape_func_t`

```c
typedef void (*hb_font_get_glyph_shape_func_t) (hb_font_t *font, void *font_data,
                                                hb_codepoint_t glyph,
                                                hb_draw_funcs_t *draw_funcs, void *draw_data,
                                                void *user_data);
```
```rust
pub type hb_font_get_glyph_shape_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        glyph: hb_codepoint_t,
        draw_funcs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        user_data: *mut c_void,
    ),
>;
```

A virtual method of `hb_font_funcs_t` that emits a glyph's outline by calling
into a `hb_draw_funcs_t`.

| Parameter    | C type              | Rust type              | Meaning                                                                 |
| ------------ | ------------------- | ---------------------- | ----------------------------------------------------------------------- |
| `font`       | `hb_font_t *`       | `*mut hb_font_t`       | The font being queried. Never null.                                      |
| `font_data`  | `void *`            | `*mut c_void`          | The font's user data. May be null.                                       |
| `glyph`      | `hb_codepoint_t`    | `u32`                  | The glyph ID to draw.                                                    |
| `draw_funcs` | `hb_draw_funcs_t *` | `*mut hb_draw_funcs_t` | The draw callbacks to send `move_to`/`line_to`/`curve_to`/`close_path` to. Never null. |
| `draw_data`  | `void *`            | `*mut c_void`          | Pass this back as the `draw_data` of every `draw_funcs` call. May be null.|
| `user_data`  | `void *`            | `*mut c_void`          | The `user_data` passed to the setter. May be null.                       |

**Returns** — nothing. This is the defect that got the type deprecated: an
implementation cannot signal "no outline available", so HarfBuzz cannot fall
back and the caller cannot distinguish an empty glyph from a failure.

**Notes** — since HarfBuzz 4.0.0, deprecated in 7.0.0 in favour of
`hb_font_draw_glyph_func_t` (a pure rename), and in 11.2.0 in favour of
`hb_font_draw_glyph_or_fail_func_t` (which returns `hb_bool_t`).

### `hb_font_draw_glyph_func_t`

```c
typedef void (*hb_font_draw_glyph_func_t) (hb_font_t *font, void *font_data,
                                           hb_codepoint_t glyph,
                                           hb_draw_funcs_t *draw_funcs, void *draw_data,
                                           void *user_data);
```
```rust
pub type hb_font_draw_glyph_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        glyph: hb_codepoint_t,
        draw_funcs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        user_data: *mut c_void,
    ),
>;
```

Structurally identical to `hb_font_get_glyph_shape_func_t` — same parameters,
same meanings, same `void` return, same inability to report failure. HarfBuzz
7.0.0 introduced it purely to fix the name ("draw" rather than "get shape"), and
11.2.0 deprecated it for `hb_font_draw_glyph_or_fail_func_t`, which finally
returns `hb_bool_t`.

Because the two typedefs are the same function-pointer type in C, a callback
written for one can be installed through either setter; see
[`hb_font_funcs_set_draw_glyph_func`](#hb_font_funcs_set_draw_glyph_func).

**Notes** — since HarfBuzz 7.0.0, deprecated in 11.2.0. The header's
`Deprecated:` line names the replacement as `hb_font_draw_glyph_func_or_fail_t`,
which is a typo; the real symbol is `hb_font_draw_glyph_or_fail_func_t`.

### `hb_font_paint_glyph_func_t`

```c
typedef hb_bool_t (*hb_font_paint_glyph_func_t) (hb_font_t *font, void *font_data,
                                                 hb_codepoint_t glyph,
                                                 hb_paint_funcs_t *paint_funcs, void *paint_data,
                                                 unsigned int palette_index,
                                                 hb_color_t foreground,
                                                 void *user_data);
```
```rust
pub type hb_font_paint_glyph_func_t = Option<
    unsafe extern "C" fn(
        font: *mut hb_font_t,
        font_data: *mut c_void,
        glyph: hb_codepoint_t,
        paint_funcs: *mut hb_paint_funcs_t,
        paint_data: *mut c_void,
        palette_index: c_uint,
        foreground: hb_color_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;
```

A virtual method of `hb_font_funcs_t` that paints a colour glyph by calling into
a `hb_paint_funcs_t`.

| Parameter       | C type               | Rust type               | Meaning                                                                       |
| --------------- | -------------------- | ----------------------- | ------------------------------------------------------------------------------ |
| `font`          | `hb_font_t *`        | `*mut hb_font_t`        | The font being queried. Never null.                                            |
| `font_data`     | `void *`             | `*mut c_void`           | The font's user data. May be null.                                             |
| `glyph`         | `hb_codepoint_t`     | `u32`                   | The glyph ID to paint.                                                         |
| `paint_funcs`   | `hb_paint_funcs_t *` | `*mut hb_paint_funcs_t` | The paint callbacks to drive. Never null.                                      |
| `paint_data`    | `void *`             | `*mut c_void`           | Pass this back as the `paint_data` of every `paint_funcs` call. May be null.   |
| `palette_index` | `unsigned int`       | `c_uint`                | Which `CPAL` colour palette to resolve palette entries against.                |
| `foreground`    | `hb_color_t`         | `u32`                   | The text colour, for palette entry `0xFFFF` ("use foreground").                |
| `user_data`     | `void *`             | `*mut c_void`           | The `user_data` passed to the setter. May be null.                             |

**Returns** — `hb_bool_t` in the signature, **but HarfBuzz discards it.** The
shim installed by `hb_font_funcs_set_paint_glyph_func` calls your callback,
ignores the result, and reports `true` to the modern
`hb_font_paint_glyph_or_fail_func_t` slot unconditionally. Returning `false`
does not produce a fallback; it produces nothing.

**Notes** — since HarfBuzz 7.0.0, deprecated in 11.2.0 in favour of
`hb_font_paint_glyph_or_fail_func_t`, whose return value is honoured.

## Constants

Six constants, all of them `#define`s. Four are aliases whose entire purpose is
to keep old spellings compiling; two are values.

| Constant                                        | C definition                                          | Rust type                       | Value                     | Deprecated |
| ----------------------------------------------- | ----------------------------------------------------- | ------------------------------- | ------------------------- | ---------- |
| `HB_SCRIPT_CANADIAN_ABORIGINAL`                 | `HB_SCRIPT_CANADIAN_SYLLABICS`                        | `hb_script_t`                   | `0x43616E73` (`'Cans'`)   | 0.9.20     |
| `HB_BUFFER_FLAGS_DEFAULT`                       | `HB_BUFFER_FLAG_DEFAULT`                              | `hb_buffer_flags_t`             | `0x00000000`              | 0.9.20     |
| `HB_BUFFER_SERIALIZE_FLAGS_DEFAULT`             | `HB_BUFFER_SERIALIZE_FLAG_DEFAULT`                    | `hb_buffer_serialize_flags_t`   | `0x00000000`              | 0.9.20     |
| `HB_AAT_LAYOUT_FEATURE_TYPE_CURISVE_CONNECTION` | `HB_AAT_LAYOUT_FEATURE_TYPE_CURSIVE_CONNECTION`       | `hb_aat_layout_feature_type_t`  | `2`                       | 8.3.0      |
| `HB_UNICODE_COMBINING_CLASS_CCC133`             | `133`                                                 | `hb_unicode_combining_class_t`  | `133`                     | 7.2.0      |
| `HB_UNICODE_MAX_DECOMPOSITION_LEN`              | `(18+1)`                                              | `c_uint`                        | `19`                      | 2.0.0      |

No function-like macros appear in this header, so nothing was skipped in
transcription.

### `HB_SCRIPT_CANADIAN_ABORIGINAL`

```c
#define HB_SCRIPT_CANADIAN_ABORIGINAL		HB_SCRIPT_CANADIAN_SYLLABICS
```
```rust
pub const HB_SCRIPT_CANADIAN_ABORIGINAL: hb_script_t = HB_SCRIPT_CANADIAN_SYLLABICS;
```

The Unified Canadian Aboriginal Syllabics script, under its old name. ISO 15924
kept the tag `Cans` and changed only the English label, so the numeric value —
`HB_TAG('C','a','n','s')` — is unchanged. `HB_SCRIPT_CANADIAN_SYLLABICS` comes
from `hb-common.h` and lives in `crate::script`. Deprecated 0.9.20.

### `HB_BUFFER_FLAGS_DEFAULT`

```c
#define HB_BUFFER_FLAGS_DEFAULT			HB_BUFFER_FLAG_DEFAULT
```
```rust
pub const HB_BUFFER_FLAGS_DEFAULT: hb_buffer_flags_t = HB_BUFFER_FLAG_DEFAULT;
```

Zero — no buffer flags set. The plural `FLAGS` was dropped from the enumerator
name in 0.9.20 (the *type* is still plural: `hb_buffer_flags_t`).
`HB_BUFFER_FLAG_DEFAULT` comes from `hb-buffer.h` and lives in `crate::buffer`.

### `HB_BUFFER_SERIALIZE_FLAGS_DEFAULT`

```c
#define HB_BUFFER_SERIALIZE_FLAGS_DEFAULT	HB_BUFFER_SERIALIZE_FLAG_DEFAULT
```
```rust
pub const HB_BUFFER_SERIALIZE_FLAGS_DEFAULT: hb_buffer_serialize_flags_t =
    HB_BUFFER_SERIALIZE_FLAG_DEFAULT;
```

Zero — serialize everything, omit nothing. Same rename, same release.
`HB_BUFFER_SERIALIZE_FLAG_DEFAULT` comes from `hb-buffer.h` and lives in
`crate::buffer`.

### `HB_AAT_LAYOUT_FEATURE_TYPE_CURISVE_CONNECTION`

```c
#define HB_AAT_LAYOUT_FEATURE_TYPE_CURISVE_CONNECTION HB_AAT_LAYOUT_FEATURE_TYPE_CURSIVE_CONNECTION
```
```rust
pub const HB_AAT_LAYOUT_FEATURE_TYPE_CURISVE_CONNECTION: hb_aat_layout_feature_type_t =
    HB_AAT_LAYOUT_FEATURE_TYPE_CURSIVE_CONNECTION;
```

AAT feature type 2, "Cursive Connection". The original name transposed two
letters — `CURISVE` — and 8.3.0 shipped the corrected spelling while keeping the
typo as an alias. The correct constant comes from `hb-aat-layout.h` and lives in
`crate::aat_layout`. Deprecated 8.3.0.

### `HB_UNICODE_COMBINING_CLASS_CCC133`

```c
#define HB_UNICODE_COMBINING_CLASS_CCC133 133
```
```rust
pub const HB_UNICODE_COMBINING_CLASS_CCC133: hb_unicode_combining_class_t = 133;
```

Tibetan combining class 133 — a value the Unicode Character Database has never
assigned to any character. HarfBuzz 7.2.0 removed it from the
`hb_unicode_combining_class_t` enumeration (upstream PR 4207) and re-added it
here as a plain macro so that code naming the whole `CCC1xx` range still
compiles. Its neighbours `CCC130` and `CCC132` remain in the live enumeration in
`crate::unicode`.

Nothing produces this value. A `hb_unicode_combining_class_func_t` may still
*return* it, since the type is an integer alias and any `int` is representable —
which is exactly why this crate transcribes HarfBuzz enumerations as aliases
rather than Rust `enum`s.

### `HB_UNICODE_MAX_DECOMPOSITION_LEN`

```c
#define HB_UNICODE_MAX_DECOMPOSITION_LEN (18+1) /* codepoints */
```
```rust
pub const HB_UNICODE_MAX_DECOMPOSITION_LEN: c_uint = 18 + 1;
```

The size, in code points, of the buffer
`hb_unicode_decompose_compatibility` requires: 18 for the longest compatibility
decomposition Unicode defines, plus one for the terminating zero. See Unicode
6.1 for the derivation of 18.

Transcribed as `c_uint` for consistency with the crate's other count-style
constants (`HB_OT_MAX_TAGS_PER_SCRIPT` and friends). In Rust, a fixed array
declaration therefore needs the literal length or a `usize` cast:
`[0u32; HB_UNICODE_MAX_DECOMPOSITION_LEN as usize]`. Deprecated 2.0.0, along
with the only API that consumes it.

## Functions

Eleven functions, all declared in one `unsafe extern "C"` block. The C
signatures below are quoted verbatim from `hb-deprecated.h`; the `Since`
versions and the behavioural detail come from the gtk-doc comments upstream
keeps beside each implementation in `hb-font.cc` and `hb-unicode.cc`, because
the header itself carries prose for only some of them.

Every setter shares one contract, stated once here and referenced below:

> **The `destroy` contract.** `destroy` may be null. If it is not null, it is
> called exactly once with `user_data` — when the callback is replaced, when the
> owning object is destroyed, **or immediately, if the setter declines to do
> anything** (because the object is immutable, or because an internal allocation
> failed). The caller never has to free `user_data` itself, and must never
> assume the setter succeeded.

### Font-functions setters

#### `hb_font_funcs_set_glyph_func`

```c
HB_DEPRECATED_FOR (hb_font_funcs_set_nominal_glyph_func and hb_font_funcs_set_variation_glyph_func)
HB_EXTERN void
hb_font_funcs_set_glyph_func (hb_font_funcs_t *ffuncs,
			      hb_font_get_glyph_func_t func,
			      void *user_data, hb_destroy_func_t destroy);
```
```rust
pub fn hb_font_funcs_set_glyph_func(
    ffuncs: *mut hb_font_funcs_t,
    func: hb_font_get_glyph_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Installs a combined nominal/variation glyph-lookup callback.

| Parameter   | Meaning                                                                                             |
| ----------- | --------------------------------------------------------------------------------------------------- |
| `ffuncs`    | The font-functions object to modify. Must not be null. Must not be immutable — see below.            |
| `func`      | The callback. Unlike the modern setters, this one does **not** treat null as "reset to default": it stores `func` in a heap trampoline that later dereferences it unconditionally, so passing null is unsafe. |
| `user_data` | Opaque pointer handed back to `func` on every call. May be null.                                     |
| `destroy`   | Called with `user_data` when no longer needed. May be null.                                          |

**Returns** — nothing. There is no way to learn whether the call took effect.

**Ownership** — `user_data` is owned by `destroy` under the shared contract
above. Note the subtlety this function creates: internally it registers *two*
callbacks (nominal and variation) sharing one reference-counted trampoline, so
`destroy` runs once, after both registrations have been released — not twice,
and not on the first one.

**Notes** — since HarfBuzz 0.9.2, deprecated in 1.2.3. Because it writes both
the nominal and the variation slots, calling it after
`hb_font_funcs_set_nominal_glyph_func` silently overwrites that setting, and
vice versa. Mixing old and new on the same `hb_font_funcs_t` is a bug.

#### `hb_font_funcs_set_glyph_v_kerning_func`

```c
HB_EXTERN void
hb_font_funcs_set_glyph_v_kerning_func (hb_font_funcs_t *ffuncs,
					hb_font_get_glyph_v_kerning_func_t func,
					void *user_data, hb_destroy_func_t destroy);
```
```rust
pub fn hb_font_funcs_set_glyph_v_kerning_func(
    ffuncs: *mut hb_font_funcs_t,
    func: hb_font_get_glyph_v_kerning_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Installs the vertical-kerning callback.

| Parameter   | Meaning                                                                          |
| ----------- | -------------------------------------------------------------------------------- |
| `ffuncs`    | The font-functions object to modify. Must not be null.                            |
| `func`      | The callback. Null restores the built-in default, which delegates to the parent font. |
| `user_data` | Opaque pointer handed back to `func`. May be null.                                |
| `destroy`   | Called with `user_data` when no longer needed. May be null.                       |

**Returns** — nothing.

**Ownership** — the shared `destroy` contract.

**Notes** — since HarfBuzz 0.9.2, deprecated in 2.0.0. There is no replacement:
OpenType defines no vertical kerning, and this slot only ever surfaced legacy
`kern`-table data supplied by the client. The installed callback is read by
`hb_font_get_glyph_v_kerning` and by HarfBuzz's internal vertical-advance
plumbing; with no callback installed, both report zero.

#### `hb_font_funcs_set_glyph_shape_func`

```c
HB_DEPRECATED_FOR (hb_font_funcs_set_draw_glyph_or_fail_func)
HB_EXTERN void
hb_font_funcs_set_glyph_shape_func (hb_font_funcs_t *ffuncs,
				    hb_font_get_glyph_shape_func_t func,
				    void *user_data, hb_destroy_func_t destroy);
```
```rust
pub fn hb_font_funcs_set_glyph_shape_func(
    ffuncs: *mut hb_font_funcs_t,
    func: hb_font_get_glyph_shape_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Installs an outline-drawing callback.

| Parameter   | Meaning                                                              |
| ----------- | -------------------------------------------------------------------- |
| `ffuncs`    | The font-functions object to modify. Must not be null.                |
| `func`      | The callback. See the shim note below.                                |
| `user_data` | Opaque pointer handed back to `func`. May be null.                    |
| `destroy`   | Called with `user_data` when no longer needed. May be null.           |

**Returns** — nothing.

**Ownership** — the shared `destroy` contract, with one extra failure path:
this setter heap-allocates a closure to hold `func`/`user_data`/`destroy`, and
if that allocation fails it calls `destroy` and returns without installing
anything.

**Notes** — since HarfBuzz 4.0.0, deprecated in 7.0.0. What actually happens is
that your callback is wrapped in a shim which invokes it and then returns `true`
to the modern `hb_font_draw_glyph_or_fail_func_t` slot. So this setter and
`hb_font_funcs_set_draw_glyph_func` write the *same* slot and overwrite each
other, and both are overwritten by
`hb_font_funcs_set_draw_glyph_or_fail_func`. Upstream implements all three of
these setters through one shared static function; they differ only in the
declared type of `func`.

#### `hb_font_funcs_set_draw_glyph_func`

```c
HB_DEPRECATED_FOR (hb_font_funcs_set_draw_glyph_or_fail_func)
HB_EXTERN void
hb_font_funcs_set_draw_glyph_func (hb_font_funcs_t *ffuncs,
                                   hb_font_draw_glyph_func_t func,
                                   void *user_data, hb_destroy_func_t destroy);
```
```rust
pub fn hb_font_funcs_set_draw_glyph_func(
    ffuncs: *mut hb_font_funcs_t,
    func: hb_font_draw_glyph_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Identical to `hb_font_funcs_set_glyph_shape_func` in every respect — same
parameters, same shim, same slot, same failure paths — differing only in the
declared callback type and in when it was deprecated.

**Notes** — since HarfBuzz 7.0.0, deprecated in 11.2.0 in favour of
`hb_font_funcs_set_draw_glyph_or_fail_func`. Migrating means changing the
callback to return `hb_bool_t` and returning `false` when the glyph has no
outline, which lets HarfBuzz fall back instead of silently drawing nothing.

#### `hb_font_funcs_set_paint_glyph_func`

```c
HB_DEPRECATED_FOR (hb_font_funcs_set_paint_glyph_or_fail_func)
HB_EXTERN void
hb_font_funcs_set_paint_glyph_func (hb_font_funcs_t *ffuncs,
                                    hb_font_paint_glyph_func_t func,
                                    void *user_data, hb_destroy_func_t destroy);
```
```rust
pub fn hb_font_funcs_set_paint_glyph_func(
    ffuncs: *mut hb_font_funcs_t,
    func: hb_font_paint_glyph_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Installs a colour-glyph painting callback.

| Parameter   | Meaning                                                        |
| ----------- | -------------------------------------------------------------- |
| `ffuncs`    | The font-functions object to modify. Must not be null.          |
| `func`      | The callback. Its return value is discarded — see below.        |
| `user_data` | Opaque pointer handed back to `func`. May be null.              |
| `destroy`   | Called with `user_data` when no longer needed. May be null.     |

**Returns** — nothing.

**Ownership** — the shared `destroy` contract, plus the same
allocation-failure path as the draw setters.

**Notes** — since HarfBuzz 7.0.0, deprecated in 11.2.0 in favour of
`hb_font_funcs_set_paint_glyph_or_fail_func`. The shim calls your callback,
**throws away its `hb_bool_t`**, and reports `true` to the modern slot. A
callback that returns `false` to mean "this glyph is not a colour glyph" will
not get the monochrome fallback it is asking for; HarfBuzz will believe the
glyph was painted.

### Font queries

#### `hb_font_get_glyph_v_kerning`

```c
HB_EXTERN hb_position_t
hb_font_get_glyph_v_kerning (hb_font_t *font,
			     hb_codepoint_t top_glyph, hb_codepoint_t bottom_glyph);
```
```rust
pub fn hb_font_get_glyph_v_kerning(
    font: *mut hb_font_t,
    top_glyph: hb_codepoint_t,
    bottom_glyph: hb_codepoint_t,
) -> hb_position_t;
```

Fetches the kerning adjustment for a vertically-stacked glyph pair.

| Parameter      | Meaning                                                    |
| -------------- | ---------------------------------------------------------- |
| `font`         | The font to query. Must not be null.                        |
| `top_glyph`    | Glyph ID of the upper glyph in the pair.                    |
| `bottom_glyph` | Glyph ID of the lower glyph in the pair.                    |

**Returns** — the adjustment in scaled font units. Zero means "no kerning",
and is also what you get when no callback is installed; the two cases are
indistinguishable.

**Ownership** — none. Nothing is allocated, nothing must be freed.

**Notes** — since HarfBuzz 0.9.2, deprecated in 2.0.0. Upstream's own comment:
*"It handles legacy kerning only (as returned by the corresponding
`hb_font_funcs_t` function)."* HarfBuzz's built-in font functions provide no
implementation that reads a font table, so unless you installed a callback with
`hb_font_funcs_set_glyph_v_kerning_func`, this always returns zero. The
horizontal counterpart, `hb_font_get_glyph_h_kerning`, is *not* deprecated and
lives in `crate::font`.

#### `hb_font_get_glyph_shape`

```c
HB_DEPRECATED_FOR (hb_font_draw_glyph_or_fail)
HB_EXTERN void
hb_font_get_glyph_shape (hb_font_t *font,
			 hb_codepoint_t glyph,
			 hb_draw_funcs_t *dfuncs, void *draw_data);
```
```rust
pub fn hb_font_get_glyph_shape(
    font: *mut hb_font_t,
    glyph: hb_codepoint_t,
    dfuncs: *mut hb_draw_funcs_t,
    draw_data: *mut c_void,
);
```

Draws a glyph's outline, delivering it as a sequence of calls to `dfuncs`.

| Parameter   | Meaning                                                                                  |
| ----------- | ---------------------------------------------------------------------------------------- |
| `font`      | The font to draw from. Must not be null.                                                  |
| `glyph`     | The glyph ID to draw.                                                                     |
| `dfuncs`    | The draw callbacks to invoke. Must not be null.                                            |
| `draw_data` | Passed as the `draw_data` argument of every `dfuncs` callback. May be null.                |

**Returns** — nothing. A glyph with no outline, or a font that cannot produce
one, is indistinguishable from an empty glyph: you simply receive no callbacks.

**Ownership** — none; `dfuncs` is borrowed for the duration of the call and its
reference count is not touched.

**Notes** — since HarfBuzz 4.0.0, deprecated in 7.0.0. Upstream's gtk-doc says
"use `hb_font_draw_glyph()` instead", but the `HB_DEPRECATED_FOR` attribute in
the header names `hb_font_draw_glyph_or_fail` — follow the header, because
`hb_font_draw_glyph` has itself since been superseded for the same reason (no
return value). Both replacements live in `crate::font`.

### Unicode-functions setters

#### `hb_unicode_funcs_set_eastasian_width_func`

```c
HB_EXTERN HB_DEPRECATED void
hb_unicode_funcs_set_eastasian_width_func (hb_unicode_funcs_t *ufuncs,
					   hb_unicode_eastasian_width_func_t func,
					   void *user_data, hb_destroy_func_t destroy);
```
```rust
pub fn hb_unicode_funcs_set_eastasian_width_func(
    ufuncs: *mut hb_unicode_funcs_t,
    func: hb_unicode_eastasian_width_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Installs the East Asian width callback.

| Parameter   | Meaning                                                                         |
| ----------- | ------------------------------------------------------------------------------- |
| `ufuncs`    | The Unicode-functions object to modify. Must not be null.                        |
| `func`      | The callback. Null resets the slot to the parent object's implementation.        |
| `user_data` | Opaque pointer handed back to `func`. May be null.                               |
| `destroy`   | Called with `user_data` when no longer needed. May be null.                      |

**Returns** — nothing.

**Ownership** — the shared `destroy` contract. If `ufuncs` is immutable, the
call is a no-op and `destroy` fires immediately.

**Notes** — since HarfBuzz 0.9.2, deprecated in 2.0.0. Installing a callback
here has no effect on shaping whatsoever; the only reader is
`hb_unicode_eastasian_width`.

#### `hb_unicode_funcs_set_decompose_compatibility_func`

```c
HB_EXTERN HB_DEPRECATED void
hb_unicode_funcs_set_decompose_compatibility_func (hb_unicode_funcs_t *ufuncs,
						   hb_unicode_decompose_compatibility_func_t func,
						   void *user_data, hb_destroy_func_t destroy);
```
```rust
pub fn hb_unicode_funcs_set_decompose_compatibility_func(
    ufuncs: *mut hb_unicode_funcs_t,
    func: hb_unicode_decompose_compatibility_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Installs the compatibility-decomposition callback.

| Parameter   | Meaning                                                                         |
| ----------- | ------------------------------------------------------------------------------- |
| `ufuncs`    | The Unicode-functions object to modify. Must not be null.                        |
| `func`      | The callback. Null resets the slot to the parent object's implementation.        |
| `user_data` | Opaque pointer handed back to `func`. May be null.                               |
| `destroy`   | Called with `user_data` when no longer needed. May be null.                      |

**Returns** — nothing.

**Ownership** — the shared `destroy` contract.

**Notes** — since HarfBuzz 0.9.2, deprecated in 2.0.0. Nothing inside HarfBuzz
reads this slot; shaping uses `hb_unicode_decompose` (canonical decomposition)
instead.

### Unicode queries

#### `hb_unicode_eastasian_width`

```c
HB_EXTERN HB_DEPRECATED unsigned int
hb_unicode_eastasian_width (hb_unicode_funcs_t *ufuncs,
			    hb_codepoint_t unicode);
```
```rust
pub fn hb_unicode_eastasian_width(
    ufuncs: *mut hb_unicode_funcs_t,
    unicode: hb_codepoint_t,
) -> c_uint;
```

Fetches the East Asian display width of a code point.

| Parameter | Meaning                                                    |
| --------- | ---------------------------------------------------------- |
| `ufuncs`  | The Unicode-functions object to query. Must not be null.     |
| `unicode` | The code point to measure.                                  |

**Returns** — whatever the installed callback returns. **The built-in
implementation returns `1` for every code point**, so unless you installed your
own callback this function is a constant.

**Ownership** — none.

**Notes** — since HarfBuzz 0.9.2, deprecated in 2.0.0. The header's own
description is *"Don't use. Not used by HarfBuzz."* There is no replacement in
HarfBuzz; if you need real East Asian width data, consult ICU or the Unicode
`EastAsianWidth.txt` property directly.

#### `hb_unicode_decompose_compatibility`

```c
HB_EXTERN HB_DEPRECATED unsigned int
hb_unicode_decompose_compatibility (hb_unicode_funcs_t *ufuncs,
				    hb_codepoint_t      u,
				    hb_codepoint_t     *decomposed);
```
```rust
pub fn hb_unicode_decompose_compatibility(
    ufuncs: *mut hb_unicode_funcs_t,
    u: hb_codepoint_t,
    decomposed: *mut hb_codepoint_t,
) -> c_uint;
```

Fetches the compatibility decomposition of a code point.

| Parameter    | Meaning                                                                                                             |
| ------------ | ------------------------------------------------------------------------------------------------------------------- |
| `ufuncs`     | The Unicode-functions object to query. Must not be null.                                                             |
| `u`          | The code point to decompose.                                                                                         |
| `decomposed` | Out-parameter: caller-allocated array of at least `HB_UNICODE_MAX_DECOMPOSITION_LEN` code points. Must not be null.   |

**Returns** — the length of the decomposition, or `0` if there is none. On
return, `decomposed[len]` is set to `0`, so the array is also usable as a
zero-terminated sequence. **The built-in implementation returns `0` for every
code point.**

**Post-processing you should know about.** HarfBuzz does not hand the callback's
answer back untouched. If the callback reports a length of exactly 1 and
`decomposed[0] == u` — a self-decomposition, which is meaningless — HarfBuzz
rewrites `decomposed[0]` to `0` and returns `0`. Then it writes the terminating
zero at `decomposed[len]`. That terminator is why the buffer needs 19 slots for
an 18-code-point maximum.

**Ownership** — none; the caller owns `decomposed` throughout.

**Notes** — since HarfBuzz 0.9.2, deprecated in 2.0.0. Note that this function
has *no* gtk-doc block in `hb-deprecated.h` — upstream keeps it in
`hb-unicode.cc` — which is why the header alone looks silent about it.

## Symbols from other headers

These 14 entries appear under upstream's `hb-deprecated` gtk-doc section but are
declared elsewhere. They are documented here for completeness, with the module
that owns each one. Do not expect to find them in `crate::deprecated`; they are
transcribed by the module named beside them, and this crate's glob re-exports at
the root mean the *import path* is the same either way (`harfbuzz_sys::X`).

### OpenType deprecations — `hb-ot-deprecated.h` (module `ot_deprecated`)

#### `HB_MATH_GLYPH_PART_FLAG_EXTENDER`

```c
#define HB_MATH_GLYPH_PART_FLAG_EXTENDER HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER
```

Alias for `HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER` (`0x00000001`, type
`hb_ot_math_glyph_part_flags_t`), which marks a glyph part in a `MATH` table
assembly as stretchable filler. The name was missing its `OT_` infix.
Deprecated 2.5.1. The replacement lives in `crate::ot_math`.

#### `HB_OT_MATH_SCRIPT`

```c
#define HB_OT_MATH_SCRIPT HB_OT_TAG_MATH_SCRIPT
```

Alias for `HB_OT_TAG_MATH_SCRIPT` — the OpenType script tag `'math'`
(`0x6D617468`, type `hb_tag_t`). Since 1.3.3, deprecated 3.4.0. Upstream adds a
pointed note: earlier documentation told you to pass this to
`hb_buffer_set_script()` to enable maths shaping, and *that no longer works*.
Use `HB_SCRIPT_MATH` (`'Zmth'`, in `crate::script`) for the buffer, and
`HB_OT_TAG_MATH_SCRIPT` (in `crate::ot_math`) where an OpenType table tag is
wanted. The two are different values with different jobs.

#### `hb_ot_layout_table_choose_script`

```c
HB_DEPRECATED_FOR (hb_ot_layout_table_select_script)
HB_EXTERN hb_bool_t
hb_ot_layout_table_choose_script (hb_face_t      *face,
				  hb_tag_t        table_tag,
				  const hb_tag_t *script_tags,
				  unsigned int   *script_index,
				  hb_tag_t       *chosen_script);
```

Selects the first of a list of candidate script tags that the face's `GSUB` or
`GPOS` table actually contains. `script_tags` is a **zero-terminated** array —
that is the only difference from the replacement, which takes an explicit
`script_count`. Upstream implements this by scanning for the terminator and
delegating to `hb_ot_layout_table_select_script`. Deprecated 2.0.0. Replacement:
`hb_ot_layout_table_select_script`, in `crate::ot_layout`.

#### `hb_ot_tags_from_script`

```c
HB_DEPRECATED_FOR (hb_ot_tags_from_script_and_language)
HB_EXTERN void
hb_ot_tags_from_script (hb_script_t  script,
			hb_tag_t    *script_tag_1,
			hb_tag_t    *script_tag_2);
```

Converts an `hb_script_t` to at most two OpenType script tags, writing them
through the two out-parameters. Unfilled slots receive `HB_OT_TAG_DEFAULT_SCRIPT`
(`'DFLT'`). The fixed arity is the problem: some scripts map to three tags, which
this signature cannot express. Since 0.6.0, deprecated 2.0.0. Replacement:
`hb_ot_tags_from_script_and_language`, in `crate::ot_layout`.

#### `hb_ot_tag_from_language`

```c
HB_DEPRECATED_FOR (hb_ot_tags_from_script_and_language)
HB_EXTERN hb_tag_t
hb_ot_tag_from_language (hb_language_t language);
```

Converts an `hb_language_t` to a single OpenType language-system tag, returning
`HB_OT_TAG_DEFAULT_LANGUAGE` (`'dflt'`) when there is no mapping. Same problem:
a language can map to several tags. Since 0.6.0, deprecated 2.0.0. Replacement:
`hb_ot_tags_from_script_and_language`, in `crate::ot_layout`.

#### `HB_OT_VAR_NO_AXIS_INDEX`

```c
#define HB_OT_VAR_NO_AXIS_INDEX		0xFFFFFFFFu
```

A sentinel "no such axis" index, type `c_uint`. The header's own description is
simply *"Do not use."* Since 1.4.2, deprecated 2.2.0. No replacement: the
replacement API, `hb_ot_var_find_axis_info`, reports absence with a `false`
return instead of a sentinel.

#### `hb_ot_var_axis_t`

```c
typedef struct hb_ot_var_axis_t {
  hb_tag_t tag;
  hb_ot_name_id_t name_id;
  float min_value;
  float default_value;
  float max_value;
} hb_ot_var_axis_t;
```

The old variation-axis description struct.

| Field           | C type            | Rust type         | Meaning                                                          |
| --------------- | ----------------- | ----------------- | ---------------------------------------------------------------- |
| `tag`           | `hb_tag_t`        | `u32`             | The axis tag, e.g. `'wght'`.                                      |
| `name_id`       | `hb_ot_name_id_t` | `c_uint`          | `name`-table ID of the axis's human-readable label.               |
| `min_value`     | `float`           | `c_float`         | Minimum value of the axis.                                        |
| `default_value` | `float`           | `c_float`         | Default value of the axis.                                        |
| `max_value`     | `float`           | `c_float`         | Maximum value of the axis.                                        |

Since 1.4.2, deprecated 2.2.0. Replacement: `hb_ot_var_axis_info_t` in
`crate::ot_var`, which adds an `axis_index` field and an `hb_ot_var_axis_flags_t`
`flags` field (carrying `HB_OT_VAR_AXIS_FLAG_HIDDEN`) — neither of which this
struct can express.

#### `hb_ot_var_get_axes`

```c
HB_DEPRECATED_FOR (hb_ot_var_get_axis_infos)
HB_EXTERN unsigned int
hb_ot_var_get_axes (hb_face_t        *face,
		    unsigned int      start_offset,
		    unsigned int     *axes_count /* IN/OUT */,
		    hb_ot_var_axis_t *axes_array /* OUT */);
```

Fetches a page of the face's variation axes, starting at `start_offset`.
`axes_count` is in/out — on input the capacity of `axes_array`, on output the
number actually written. Both `axes_count` and `axes_array` are nullable, in
which case the function only reports the total. Returns the total number of
axes in the face. Since 1.4.2, deprecated 2.2.0. Replacement:
`hb_ot_var_get_axis_infos`, in `crate::ot_var`.

#### `hb_ot_var_find_axis`

```c
HB_DEPRECATED_FOR (hb_ot_var_find_axis_info)
HB_EXTERN hb_bool_t
hb_ot_var_find_axis (hb_face_t        *face,
		     hb_tag_t          axis_tag,
		     unsigned int     *axis_index,
		     hb_ot_var_axis_t *axis_info);
```

Looks up one axis by tag, writing its index through `axis_index` and its
description through `axis_info`. Returns `true` if found. Since 1.4.2,
deprecated 2.2.0. Replacement: `hb_ot_var_find_axis_info`, in `crate::ot_var`
(which folds the index into the returned `hb_ot_var_axis_info_t` and so needs
only one out-parameter).

> `hb-ot-deprecated.h` also declares `hb_ot_layout_script_find_language`
> (deprecated for `hb_ot_layout_script_select_language`), which upstream's
> section list omits. It belongs to `crate::ot_deprecated` alongside the entries
> above.

### Not actually deprecated — `hb-ot-layout.h` (module `ot_layout`)

#### `hb_ot_layout_table_find_script`

```c
HB_EXTERN hb_bool_t
hb_ot_layout_table_find_script (hb_face_t    *face,
				hb_tag_t      table_tag,
				hb_tag_t      script_tag,
				unsigned int *script_index /* OUT */);
```

Fetches the index of a given script tag in the face's `GSUB` or `GPOS` table;
returns `true` if the script is present. `table_tag` is `HB_OT_TAG_GSUB` or
`HB_OT_TAG_GPOS`.

This one is listed under the `hb-deprecated` section but carries **no**
`HB_DEPRECATED` attribute and no `Deprecated:` tag anywhere — it is a live,
supported function declared in `hb-ot-layout.h`, and it is transcribed without
`#[deprecated]` in `crate::ot_layout`. It is presumably listed here because its
sibling `hb_ot_layout_table_choose_script` is deprecated. Treat the section
listing as a filing accident, not as guidance.

### Platform back ends

Each of these is gated behind the Cargo feature that compiles the corresponding
back end, so the Rust declaration exists only when that feature is on.

#### `hb_ft_font_get_face`

```c
HB_DEPRECATED_FOR (hb_ft_font_get_ft_face)
HB_EXTERN FT_Face
hb_ft_font_get_face (hb_font_t *font);
```

Fetches the `FT_Face` behind an `hb_font_t` created by `hb_ft_font_create` or
`hb_ft_font_create_referenced`. Returns the face, or `NULL` if `font` was not
created by the FreeType integration. The FreeType face is owned by the
`hb_font_t`; do not call `FT_Done_Face` on it. Since 0.9.2, deprecated 10.4.0 —
a pure rename, for symmetry with `hb_ft_face_get_ft_face`. Header `hb-ft.h`,
module `ft`, feature `freetype`.

#### `hb_graphite2_font_get_gr_font`

```c
HB_DEPRECATED_FOR (hb_graphite2_face_get_gr_face)
HB_EXTERN gr_font *
hb_graphite2_font_get_gr_font (hb_font_t *font);
```

**Always returns `NULL`.** Upstream stopped keeping a per-font Graphite object
in 1.4.2 and reduced this function to a stub; the `HB_UNUSED` on its parameter
in `hb-graphite2.cc` says the rest. Since 0.9.10, deprecated 1.4.2. Use
`hb_graphite2_face_get_gr_face` on the font's face instead. Header
`hb-graphite2.h`, module `graphite2`, feature `graphite2`.

#### `hb_directwrite_face_get_font_face`

```c
HB_DEPRECATED_FOR (hb_directwrite_face_get_dw_font_face)
HB_EXTERN IDWriteFontFace *
hb_directwrite_face_get_font_face (hb_face_t *face);
```

Fetches the DirectWrite `IDWriteFontFace` associated with an `hb_face_t`. Since
2.5.0, deprecated 10.4.0 — a rename that inserts `dw_` for consistency with the
rest of the DirectWrite API. The COM object is owned by the face; the function
does not `AddRef`. Header `hb-directwrite.h`, Windows only. `harfbuzz-sys`
exposes no DirectWrite feature, so this symbol has no Rust module here.

#### `hb_directwrite_font_get_dw_font`

```c
HB_DEPRECATED
HB_EXTERN IDWriteFont *
hb_directwrite_font_get_dw_font (hb_font_t *font);
```

**Always returns `NULL`.** HarfBuzz 11.0.0 stopped tracking an `IDWriteFont` (as
opposed to an `IDWriteFontFace`) and reduced this to a stub. Since 10.3.0,
deprecated 11.0.0 with no replacement named; if you want the face-level object,
call `hb_directwrite_font_get_dw_font_face`. Header `hb-directwrite.h`, Windows
only; likewise absent from this crate.

## Usage

The only good use of this header is *reading* it — recognising a symbol in
someone else's code and knowing what to replace it with. The examples below are
therefore paired: the deprecated form, then the supported one.

### Migrating a glyph-lookup callback

The old way installs one callback for both cases:

```c
static hb_bool_t
my_get_glyph (hb_font_t *font, void *font_data,
              hb_codepoint_t unicode, hb_codepoint_t vs,
              hb_codepoint_t *glyph, void *user_data)
{
  my_font_t *f = (my_font_t *) font_data;
  if (vs) {
    *glyph = my_lookup_with_selector (f, unicode, vs);
    if (*glyph) return true;
    /* fall through: variation selectors degrade to the base character */
  }
  *glyph = my_lookup (f, unicode);
  return *glyph != 0;
}

hb_font_funcs_t *ffuncs = hb_font_funcs_create ();
hb_font_funcs_set_glyph_func (ffuncs, my_get_glyph, NULL, NULL);   /* deprecated */
```

The new way splits it, and the split is where the performance came from — the
nominal path is the hot one, and HarfBuzz can also use the batched
`hb_font_funcs_set_nominal_glyphs_func` when you provide it:

```c
static hb_bool_t
my_nominal_glyph (hb_font_t *font, void *font_data,
                  hb_codepoint_t unicode,
                  hb_codepoint_t *glyph, void *user_data)
{
  *glyph = my_lookup ((my_font_t *) font_data, unicode);
  return *glyph != 0;
}

static hb_bool_t
my_variation_glyph (hb_font_t *font, void *font_data,
                    hb_codepoint_t unicode, hb_codepoint_t vs,
                    hb_codepoint_t *glyph, void *user_data)
{
  *glyph = my_lookup_with_selector ((my_font_t *) font_data, unicode, vs);
  return *glyph != 0;
}

hb_font_funcs_t *ffuncs = hb_font_funcs_create ();
hb_font_funcs_set_nominal_glyph_func   (ffuncs, my_nominal_glyph,   NULL, NULL);
hb_font_funcs_set_variation_glyph_func (ffuncs, my_variation_glyph, NULL, NULL);
```

Note the behavioural difference: with the split callbacks, returning `false`
from the variation function makes HarfBuzz retry the nominal function on its
own. The combined callback had to implement that fallback itself, as above.

In Rust:

```rust
use core::ffi::c_void;
use core::ptr;
use harfbuzz_sys::{
    hb_bool_t, hb_codepoint_t, hb_font_funcs_create, hb_font_funcs_set_nominal_glyph_func,
    hb_font_funcs_set_variation_glyph_func, hb_font_t,
};

unsafe extern "C" fn nominal_glyph(
    _font: *mut hb_font_t,
    _font_data: *mut c_void,
    unicode: hb_codepoint_t,
    glyph: *mut hb_codepoint_t,
    _user_data: *mut c_void,
) -> hb_bool_t {
    let g = my_lookup(unicode);
    unsafe { *glyph = g };
    (g != 0) as hb_bool_t
}

unsafe extern "C" fn variation_glyph(
    _font: *mut hb_font_t,
    _font_data: *mut c_void,
    unicode: hb_codepoint_t,
    vs: hb_codepoint_t,
    glyph: *mut hb_codepoint_t,
    _user_data: *mut c_void,
) -> hb_bool_t {
    let g = my_lookup_vs(unicode, vs);
    unsafe { *glyph = g };
    (g != 0) as hb_bool_t
}

let ffuncs = unsafe { hb_font_funcs_create() };
unsafe {
    hb_font_funcs_set_nominal_glyph_func(ffuncs, Some(nominal_glyph), ptr::null_mut(), None);
    hb_font_funcs_set_variation_glyph_func(ffuncs, Some(variation_glyph), ptr::null_mut(), None);
}
```

### Migrating outline extraction

Deprecated, and unable to report failure:

```c
static void
my_draw (hb_font_t *font, void *font_data, hb_codepoint_t glyph,
         hb_draw_funcs_t *dfuncs, void *draw_data, void *user_data)
{
  my_emit_outline (font_data, glyph, dfuncs, draw_data);   /* no way to say "I can't" */
}

hb_font_funcs_set_draw_glyph_func (ffuncs, my_draw, NULL, NULL);   /* deprecated */
hb_font_get_glyph_shape (font, gid, dfuncs, draw_data);            /* deprecated */
```

Supported, with a `hb_bool_t` on both ends:

```c
static hb_bool_t
my_draw_or_fail (hb_font_t *font, void *font_data, hb_codepoint_t glyph,
                 hb_draw_funcs_t *dfuncs, void *draw_data, void *user_data)
{
  if (!my_has_outline (font_data, glyph))
    return false;                       /* HarfBuzz can now fall back */
  my_emit_outline (font_data, glyph, dfuncs, draw_data);
  return true;
}

hb_font_funcs_set_draw_glyph_or_fail_func (ffuncs, my_draw_or_fail, NULL, NULL);

if (!hb_font_draw_glyph_or_fail (font, gid, dfuncs, draw_data))
  handle_missing_outline (gid);
```

The paint side migrates the same way: `hb_font_funcs_set_paint_glyph_func` →
`hb_font_funcs_set_paint_glyph_or_fail_func`, with the callback's `hb_bool_t`
now actually consulted.

### Migrating the renamed constants

Purely mechanical, and safe to do with a text editor:

| Replace                                         | With                                            |
| ----------------------------------------------- | ----------------------------------------------- |
| `HB_SCRIPT_CANADIAN_ABORIGINAL`                 | `HB_SCRIPT_CANADIAN_SYLLABICS`                  |
| `HB_BUFFER_FLAGS_DEFAULT`                       | `HB_BUFFER_FLAG_DEFAULT`                        |
| `HB_BUFFER_SERIALIZE_FLAGS_DEFAULT`             | `HB_BUFFER_SERIALIZE_FLAG_DEFAULT`              |
| `HB_AAT_LAYOUT_FEATURE_TYPE_CURISVE_CONNECTION` | `HB_AAT_LAYOUT_FEATURE_TYPE_CURSIVE_CONNECTION` |
| `HB_MATH_GLYPH_PART_FLAG_EXTENDER`              | `HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER`           |
| `HB_OT_MATH_SCRIPT`                             | `HB_OT_TAG_MATH_SCRIPT`, or `HB_SCRIPT_MATH` for `hb_buffer_set_script` |

Every row but the last is value-identical. The last is not — read
[`HB_OT_MATH_SCRIPT`](#hb_ot_math_script) before changing it.

### Calling a deprecated function from Rust anyway

Sometimes you must — reproducing a bug, or matching another implementation's
behaviour. `#[deprecated]` is a warning, not an error, and it is scoped:

```rust
use harfbuzz_sys::{hb_unicode_eastasian_width, hb_unicode_funcs_get_default};

#[allow(deprecated)]
fn width_of(cp: u32) -> u32 {
    unsafe { hb_unicode_eastasian_width(hb_unicode_funcs_get_default(), cp) }
}
```

This returns `1` for every input, because nothing has installed a real
implementation. That is the correct expectation, not a bug.

### The buffer for a compatibility decomposition

If you do call `hb_unicode_decompose_compatibility`, size the buffer from the
constant, and remember the `as usize`:

```rust
use harfbuzz_sys::{
    HB_UNICODE_MAX_DECOMPOSITION_LEN, hb_unicode_decompose_compatibility,
    hb_unicode_funcs_get_default,
};

#[allow(deprecated)]
const MAX_DECOMP: usize = HB_UNICODE_MAX_DECOMPOSITION_LEN as usize;

/// Fills `buf` with the compatibility decomposition of `cp` and returns the
/// prefix that was written. The buffer must be the full 19 slots: HarfBuzz
/// writes a terminating zero past the last code point.
#[allow(deprecated)]
fn nfkd(cp: u32, buf: &mut [u32; MAX_DECOMP]) -> &[u32] {
    let len = unsafe {
        hb_unicode_decompose_compatibility(hb_unicode_funcs_get_default(), cp, buf.as_mut_ptr())
    };
    &buf[..len as usize]
}
```

With HarfBuzz's built-in Unicode functions this always yields an empty slice.

## Pitfalls

### `destroy` fires even when the setter does nothing

Every setter on this page runs `destroy(user_data)` and returns early if the
target object is immutable, and the draw/paint setters do the same if their
internal allocation fails. None of them return a value, so **there is no way to
observe that your callback was not installed** — only that your `destroy` ran
sooner than you expected.

This bites hardest when `user_data` is a `Box::into_raw`'d Rust value: the
`destroy` callback will reclaim it correctly, but the callback that was supposed
to use it never got registered, and shaping will silently use the default
implementation. Make objects immutable *after* configuring them, never before,
and check `hb_font_funcs_is_immutable` if you did not create the object
yourself.

### Old and new setters fight over the same slot

`hb_font_funcs_set_glyph_shape_func`, `hb_font_funcs_set_draw_glyph_func`, and
`hb_font_funcs_set_draw_glyph_or_fail_func` all write the **same** internal
slot; upstream routes the first two through one shared implementation. Whichever
you call last wins. The same is true of `hb_font_funcs_set_paint_glyph_func`
versus `hb_font_funcs_set_paint_glyph_or_fail_func`, and of
`hb_font_funcs_set_glyph_func` versus the nominal/variation pair.

There is no diagnostic. A partially-migrated codebase that calls both the old
and the new setter on one `hb_font_funcs_t` will get whichever ran last, which
depends on link order or initialisation order and can differ between builds.
Migrate a font-functions object all at once.

### The paint callback's return value is thrown away

`hb_font_paint_glyph_func_t` is declared to return `hb_bool_t`, so it looks like
it can decline to paint. It cannot: HarfBuzz's compatibility shim calls it,
discards the result, and reports success. If your colour-glyph implementation
depends on returning `false` to trigger the monochrome fallback, that fallback
will never happen through this API. Move to
`hb_font_funcs_set_paint_glyph_or_fail_func`.

### Silence is the failure mode for outlines

`hb_font_get_glyph_shape` returns `void`, and so do
`hb_font_get_glyph_shape_func_t` and `hb_font_draw_glyph_func_t`. A glyph with
no outline, a glyph ID out of range, and a font whose draw implementation gave
up all look identical from the caller's side: no callbacks arrive. If you are
debugging "nothing renders", switch to `hb_font_draw_glyph_or_fail` first — its
`false` return is often the entire answer.

### Two functions are permanent stubs

`hb_graphite2_font_get_gr_font` and `hb_directwrite_font_get_dw_font` return
`NULL` unconditionally in current HarfBuzz. They are not "deprecated but
working"; they are deprecated *and* gutted. Code that null-checks their result
and falls back will appear to work; code that assumes success will crash. Both
have face-level replacements.

### Zero means "no kerning" and also "no implementation"

`hb_font_get_glyph_v_kerning` returns `0` when no vertical-kerning callback is
installed, which is the normal state of every HarfBuzz font, since no built-in
font-functions implementation reads a font table for it. A zero return therefore
tells you nothing. There is no `has_data` query for this slot.

### `HB_OT_MATH_SCRIPT` is not a drop-in rename

It is the *only* alias in the section whose migration is not value-preserving in
every context. It equals `HB_OT_TAG_MATH_SCRIPT` = `'math'`, an OpenType table
tag. Old documentation told readers to pass it to `hb_buffer_set_script()`;
that path no longer works, and the correct buffer script is `HB_SCRIPT_MATH` =
`'Zmth'`. Renaming the constant without reading its context can silently change
which script HarfBuzz shapes with.

### `HB_UNICODE_MAX_DECOMPOSITION_LEN` includes the terminator

It is 19, not 18: 18 code points of decomposition plus one slot for the
terminating zero that HarfBuzz writes at `decomposed[len]`. Allocating 18 is an
off-by-one buffer overflow that only triggers on the longest decompositions in
Unicode — which is to say, almost never in testing, and eventually in
production.

### `#[deprecated]` scoping in this crate

`crate::deprecated` carries a module-level `#![allow(deprecated)]`, because its
own declarations reference each other (a deprecated setter takes a deprecated
callback type) and would otherwise warn during the crate's own compilation. That
allowance does **not** propagate: every use of these items outside the module
produces the normal deprecation warning, with the `note` naming the replacement
and the HarfBuzz version in which it landed.

### The header can be compiled out

Everything on this page sits inside `#ifndef HB_DISABLE_DEPRECATED` upstream.
`harfbuzz-sys` never defines that macro, so all symbols exist in the vendored
build. If you point this crate at a system HarfBuzz that *was* built with
deprecation disabled, these declarations compile and then fail to link — with
undefined-symbol errors, not a helpful message.

## Section checklist

Upstream's `hb-deprecated` gtk-doc section lists **38** symbols
(`docs/harfbuzz-sections.txt`). All 38 are documented on this page: **24** are
declared in `hb-deprecated.h` and transcribed in `crate::deprecated` (6
constants, 7 callback typedefs, 11 functions); the other **14** are declared in
five other headers and are noted above with their real homes.

| Symbol                                          | Declared in            | Rust module      | On this page                                                       |
| ----------------------------------------------- | ---------------------- | ---------------- | ------------------------------------------------------------------ |
| `HB_BUFFER_FLAGS_DEFAULT`                       | `hb-deprecated.h`      | `deprecated`     | [Constants](#hb_buffer_flags_default)                               |
| `HB_BUFFER_SERIALIZE_FLAGS_DEFAULT`             | `hb-deprecated.h`      | `deprecated`     | [Constants](#hb_buffer_serialize_flags_default)                     |
| `HB_SCRIPT_CANADIAN_ABORIGINAL`                 | `hb-deprecated.h`      | `deprecated`     | [Constants](#hb_script_canadian_aboriginal)                         |
| `hb_font_funcs_set_glyph_func`                  | `hb-deprecated.h`      | `deprecated`     | [Functions](#hb_font_funcs_set_glyph_func)                          |
| `hb_font_get_glyph_func_t`                      | `hb-deprecated.h`      | `deprecated`     | [Types](#hb_font_get_glyph_func_t)                                  |
| `HB_MATH_GLYPH_PART_FLAG_EXTENDER`              | `hb-ot-deprecated.h`   | `ot_deprecated`  | [Other headers](#hb_math_glyph_part_flag_extender)                  |
| `HB_OT_MATH_SCRIPT`                             | `hb-ot-deprecated.h`   | `ot_deprecated`  | [Other headers](#hb_ot_math_script)                                 |
| `hb_ot_layout_table_choose_script`              | `hb-ot-deprecated.h`   | `ot_deprecated`  | [Other headers](#hb_ot_layout_table_choose_script)                  |
| `hb_ot_layout_table_find_script`                | `hb-ot-layout.h`       | `ot_layout`      | [Other headers](#hb_ot_layout_table_find_script) — not deprecated    |
| `hb_ot_tag_from_language`                       | `hb-ot-deprecated.h`   | `ot_deprecated`  | [Other headers](#hb_ot_tag_from_language)                           |
| `hb_ot_tags_from_script`                        | `hb-ot-deprecated.h`   | `ot_deprecated`  | [Other headers](#hb_ot_tags_from_script)                            |
| `HB_OT_VAR_NO_AXIS_INDEX`                       | `hb-ot-deprecated.h`   | `ot_deprecated`  | [Other headers](#hb_ot_var_no_axis_index)                           |
| `hb_ot_var_axis_t`                              | `hb-ot-deprecated.h`   | `ot_deprecated`  | [Other headers](#hb_ot_var_axis_t)                                  |
| `hb_ot_var_find_axis`                           | `hb-ot-deprecated.h`   | `ot_deprecated`  | [Other headers](#hb_ot_var_find_axis)                               |
| `hb_ot_var_get_axes`                            | `hb-ot-deprecated.h`   | `ot_deprecated`  | [Other headers](#hb_ot_var_get_axes)                                |
| `hb_unicode_eastasian_width_func_t`             | `hb-deprecated.h`      | `deprecated`     | [Types](#hb_unicode_eastasian_width_func_t)                         |
| `hb_unicode_eastasian_width`                    | `hb-deprecated.h`      | `deprecated`     | [Functions](#hb_unicode_eastasian_width)                            |
| `hb_unicode_funcs_set_eastasian_width_func`     | `hb-deprecated.h`      | `deprecated`     | [Functions](#hb_unicode_funcs_set_eastasian_width_func)             |
| `HB_UNICODE_MAX_DECOMPOSITION_LEN`              | `hb-deprecated.h`      | `deprecated`     | [Constants](#hb_unicode_max_decomposition_len)                      |
| `hb_unicode_decompose_compatibility_func_t`     | `hb-deprecated.h`      | `deprecated`     | [Types](#hb_unicode_decompose_compatibility_func_t)                 |
| `hb_unicode_decompose_compatibility`            | `hb-deprecated.h`      | `deprecated`     | [Functions](#hb_unicode_decompose_compatibility)                    |
| `hb_unicode_funcs_set_decompose_compatibility_func` | `hb-deprecated.h`  | `deprecated`     | [Functions](#hb_unicode_funcs_set_decompose_compatibility_func)     |
| `HB_UNICODE_COMBINING_CLASS_CCC133`             | `hb-deprecated.h`      | `deprecated`     | [Constants](#hb_unicode_combining_class_ccc133)                     |
| `hb_font_funcs_set_glyph_v_kerning_func`        | `hb-deprecated.h`      | `deprecated`     | [Functions](#hb_font_funcs_set_glyph_v_kerning_func)                |
| `hb_font_get_glyph_shape`                       | `hb-deprecated.h`      | `deprecated`     | [Functions](#hb_font_get_glyph_shape)                               |
| `hb_font_get_glyph_shape_func_t`                | `hb-deprecated.h`      | `deprecated`     | [Types](#hb_font_get_glyph_shape_func_t)                            |
| `hb_font_funcs_set_glyph_shape_func`            | `hb-deprecated.h`      | `deprecated`     | [Functions](#hb_font_funcs_set_glyph_shape_func)                    |
| `hb_font_draw_glyph_func_t`                     | `hb-deprecated.h`      | `deprecated`     | [Types](#hb_font_draw_glyph_func_t)                                 |
| `hb_font_funcs_set_draw_glyph_func`             | `hb-deprecated.h`      | `deprecated`     | [Functions](#hb_font_funcs_set_draw_glyph_func)                     |
| `hb_font_paint_glyph_func_t`                    | `hb-deprecated.h`      | `deprecated`     | [Types](#hb_font_paint_glyph_func_t)                                |
| `hb_font_funcs_set_paint_glyph_func`            | `hb-deprecated.h`      | `deprecated`     | [Functions](#hb_font_funcs_set_paint_glyph_func)                    |
| `hb_font_get_glyph_v_kerning`                   | `hb-deprecated.h`      | `deprecated`     | [Functions](#hb_font_get_glyph_v_kerning)                           |
| `hb_font_get_glyph_v_kerning_func_t`            | `hb-deprecated.h`      | `deprecated`     | [Types](#hb_font_get_glyph_v_kerning_func_t)                        |
| `HB_AAT_LAYOUT_FEATURE_TYPE_CURISVE_CONNECTION` | `hb-deprecated.h`      | `deprecated`     | [Constants](#hb_aat_layout_feature_type_curisve_connection)         |
| `hb_directwrite_face_get_font_face`             | `hb-directwrite.h`     | (Windows only)   | [Other headers](#hb_directwrite_face_get_font_face)                 |
| `hb_directwrite_font_get_dw_font`               | `hb-directwrite.h`     | (Windows only)   | [Other headers](#hb_directwrite_font_get_dw_font)                   |
| `hb_ft_font_get_face`                           | `hb-ft.h`              | `ft`             | [Other headers](#hb_ft_font_get_face)                               |
| `hb_graphite2_font_get_gr_font`                 | `hb-graphite2.h`       | `graphite2`      | [Other headers](#hb_graphite2_font_get_gr_font)                     |
