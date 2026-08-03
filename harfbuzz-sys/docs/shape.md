# Shaping

Source header: `hb-shape.h`. Rust module: `harfbuzz_sys::shape` (glob re-exported at the crate root).

## Overview

Shaping is the central operation of HarfBuzz. It converts a run of Unicode
characters into a sequence of positioned glyphs, applying the font's OpenType
(or AAT, or Graphite) tables along the way: mapping characters to glyphs,
substituting ligatures and contextual forms, reordering marks and vowel signs,
and computing an advance and offset for each output glyph.

Shaping operates on a **buffer** (`hb_buffer_t`) and a **font**
(`hb_font_t`). A buffer holds a sequence of characters that share one font, one
text direction, one script, and one language — an "item" or "run" in the
terminology of a layout engine. Splitting text into such runs is the caller's
job, not HarfBuzz's; the buffer's direction, script, and language properties
are set before shaping (or left unset, in which case
`hb_buffer_guess_segment_properties()` is applied). Shaping *replaces* the
buffer's contents in place: on entry the buffer holds Unicode codepoints, and
on return it holds glyph indices into the font plus an array of positions.
There is no separate output object — you read the results back out of the same
buffer with `hb_buffer_get_glyph_infos()` and
`hb_buffer_get_glyph_positions()`.

This header declares only the entry points that run a shaping operation. It
does not declare any types of its own. Everything it names comes from
elsewhere: `hb_font_t` from `hb-font.h`, `hb_buffer_t` from `hb-buffer.h`,
`hb_feature_t`, `hb_bool_t` and `hb_tag_t` from `hb-common.h`. Consequently
there are no objects to create or destroy here, and no reference counting to
manage — `hb_shape()` and friends borrow the font and mutate the buffer, and
own neither.

Internally, `hb_shape_full()` is a thin front end over the shape-plan machinery
in `hb-shape-plan.h`: it builds (or looks up, from a global cache) an
`hb_shape_plan_t` for the face, buffer properties, feature set, and variation
coordinates in play, executes it, and releases it. If you shape the same kind
of text repeatedly and want explicit control over that plan — to keep it alive,
or to inspect which shaper was chosen — use the `hb-shape-plan.h` API directly.
Otherwise `hb_shape()` is the function to call; the cache makes the plan lookup
cheap.

A **shaper** is one of HarfBuzz's back ends: `ot` (its own OpenType
implementation), plus optionally `graphite2`, `coretext`, and the trivial
`fallback`. Which are compiled in depends on build options, and
`hb_shape_list_shapers()` reports the set for the running library. Normally you
let HarfBuzz pick; `hb_shape_full()` exists for the cases where you must pin a
specific back end, most often in test harnesses and shaping-comparison tools.

## Types

This header declares no types. It uses, but does not define:

| Type | Declared in | Role here |
| --- | --- | --- |
| `hb_font_t` | `hb-font.h` | The font to shape with. Supplies glyph mappings, metrics, and variation coordinates. |
| `hb_buffer_t` | `hb-buffer.h` | Input characters on entry, output glyphs and positions on return. |
| `hb_feature_t` | `hb-common.h` | One requested feature setting: `tag`, `value`, and the cluster range `[start, end)`. |
| `hb_bool_t` | `hb-common.h` | C `int`; zero is false. |
| `hb_tag_t` | `hb-common.h` | Four-byte tag; used only by the experimental justification entry point, to report a variation axis. |

## Functions

### Shaping

#### `hb_shape`

```c
void
hb_shape (hb_font_t           *font,
          hb_buffer_t         *buffer,
          const hb_feature_t  *features,
          unsigned int         num_features);
```

```rust
pub fn hb_shape(
    font: *mut hb_font_t,
    buffer: *mut hb_buffer_t,
    features: *const hb_feature_t,
    num_features: c_uint,
);
```

Shapes `buffer` with `font`, turning the buffer's Unicode content into
positioned glyphs. `features` is an optional array of `num_features` feature
settings applied on top of the shaper's defaults; when two entries carry the
same tag over overlapping cluster ranges, the entry at the **higher index**
takes precedence.

Ownership and lifetime: this borrows both arguments. It does not take a
reference on `font` and does not destroy `buffer`; the caller keeps owning
both. `features` is read during the call only — the array may live on the stack
and may be reused or freed immediately afterwards. The buffer's previous
contents are consumed and replaced.

Failure: the function returns `void`, so a shaping failure is invisible here.
It is implemented as `hb_shape_full(font, buffer, features, num_features,
NULL)` with the result discarded. Use `hb_shape_full()` if you need to know
whether shaping succeeded, and check `hb_buffer_allocation_successful()` if you
need to distinguish an out-of-memory buffer.

Nullability: `features` may be `NULL` (pass `num_features` as 0). The header
does not specify behaviour for a null `font` or `buffer`; in practice HarfBuzz
objects are never null because failed creation returns the immutable *empty*
object rather than `NULL`, so passing `hb_font_get_empty()` or
`hb_buffer_get_empty()` is the degenerate case rather than a null pointer.

Since HarfBuzz 0.9.2.

#### `hb_shape_full`

```c
hb_bool_t
hb_shape_full (hb_font_t          *font,
               hb_buffer_t        *buffer,
               const hb_feature_t *features,
               unsigned int        num_features,
               const char * const *shaper_list);
```

```rust
pub fn hb_shape_full(
    font: *mut hb_font_t,
    buffer: *mut hb_buffer_t,
    features: *const hb_feature_t,
    num_features: c_uint,
    shaper_list: *const *const c_char,
) -> hb_bool_t;
```

As `hb_shape()`, plus control over which back end runs. If `shaper_list` is
non-null it is a null-terminated array of NUL-terminated shaper names; they are
tried in order and the first that can handle the face wins. If it is `NULL`,
HarfBuzz's default ordering is used.

Returns false if **all** shapers failed, true otherwise. Note the asymmetry
worth internalising: a true return means some shaper ran to completion, not
that the text rendered "well" — missing glyphs come back as `.notdef` (glyph 0)
with a successful return.

Ownership and lifetime: identical to `hb_shape()`. `shaper_list` and the
strings it points at are read during the call only and are not taken over; the
usual idiom is a static array of string literals.

Nullability: `features` and `shaper_list` may both be `NULL`.

Since HarfBuzz 0.9.2.

#### `hb_shape_justify` (experimental)

```c
#ifdef HB_EXPERIMENTAL_API
hb_bool_t
hb_shape_justify (hb_font_t          *font,
                  hb_buffer_t        *buffer,
                  const hb_feature_t *features,
                  unsigned int        num_features,
                  const char * const *shaper_list,
                  float               min_target_advance,
                  float               max_target_advance,
                  float              *advance,   /* IN/OUT */
                  hb_tag_t           *var_tag,   /* OUT */
                  float              *var_value  /* OUT */);
#endif
```

```rust
#[cfg(feature = "experimental")]
pub fn hb_shape_justify(
    font: *mut hb_font_t,
    buffer: *mut hb_buffer_t,
    features: *const hb_feature_t,
    num_features: c_uint,
    shaper_list: *const *const c_char,
    min_target_advance: c_float,
    max_target_advance: c_float,
    advance: *mut c_float,
    var_tag: *mut hb_tag_t,
    var_value: *mut c_float,
) -> hb_bool_t;
```

Shapes as `hb_shape_full()` does, and additionally justifies the result: it
searches for a value of a variation axis that brings the buffer's total advance
into `[min_target_advance, max_target_advance]`. The axis chosen is `jstf` if
the face has one, else `wdth`. Whether width or height is measured follows the
buffer's direction.

Parameter protocol:

| Parameter | Direction | Meaning |
| --- | --- | --- |
| `min_target_advance` | in | Lower bound of the advance to aim for. |
| `max_target_advance` | in | Upper bound of the advance to aim for. |
| `advance` | in/out | On entry, the advance of the buffer as shaped by `hb_shape_full()` if you already know it, otherwise `0.0` to have it computed. On return, the advance actually achieved. |
| `var_tag` | out | The variation-axis tag used, or `HB_TAG_NONE`. |
| `var_value` | out | The axis value settled on, or `0.0`. |

`var_tag` is set to `HB_TAG_NONE` and `var_value` to zero in the two cases
where no justification happened: the incoming advance was already within the
target range, or the face has neither a `jstf` nor a `wdth` axis. In the latter
case the buffer is still shaped and `*advance` is filled in.

Ownership and lifetime: `font` must be **mutable and exclusively yours** — the
search calls `hb_font_set_variation()` on it repeatedly and leaves it at
whichever value it stopped on. Do not pass a font you share with other code, an
immutable font, or one whose variation state you rely on afterwards. `buffer`
is shaped several times over; on return it holds the final, justified shaping.
The three out-pointers must all be valid — the implementation dereferences
`*advance` before doing anything else, so none of them may be null.

Returns false if all shapers failed, and also false on an internal allocation
failure. Upstream marks this API as experimental and expects it to change; in
the Rust crate it appears only with the `experimental` cargo feature, which
turns on `HB_EXPERIMENTAL_API` for the vendored build. Upstream additionally
compiles it out under `HB_NO_VAR`.

No `Since:` version — the header carries `XSince: EXPERIMENTAL`.

### Back-end discovery

#### `hb_shape_list_shapers`

```c
const char **
hb_shape_list_shapers (void);
```

```rust
pub fn hb_shape_list_shapers() -> *mut *const c_char;
```

Retrieves the list of shapers supported by this build of HarfBuzz, as a
null-terminated array of NUL-terminated ASCII names — typically some subset of
`"graphite2"`, `"coretext"`, `"ot"`, `"fallback"`, in the default trial order.

Ownership: the array and every string in it are owned by HarfBuzz and must not
be modified or freed. It is built once, lazily, and released at process exit.
Never null — on allocation failure the implementation returns a static array
containing only the terminator, so the result is always safe to walk. The names
are exactly the ones `hb_shape_full()` accepts in `shaper_list`.

Since HarfBuzz 0.9.2.

## Usage notes

### The buffer is both input and output

The most common first mistake is expecting a return value carrying glyphs.
There is none. The flow is:

```c
hb_buffer_t *buf = hb_buffer_create ();
hb_buffer_add_utf8 (buf, text, -1, 0, -1);
hb_buffer_guess_segment_properties (buf);      /* or set direction/script/language yourself */

hb_shape (font, buf, NULL, 0);

unsigned int len;
hb_glyph_info_t     *info = hb_buffer_get_glyph_infos (buf, &len);
hb_glyph_position_t *pos  = hb_buffer_get_glyph_positions (buf, &len);
/* ... consume ... */

hb_buffer_destroy (buf);
```

A buffer that has been shaped holds glyphs, not characters. To shape different
text with the same buffer, call `hb_buffer_reset()` (or `hb_buffer_clear_contents()`,
which keeps the segment properties) and add the new text.

### Empty buffers

`hb_shape_full()` returns true immediately for a buffer of length zero, without
consulting any shaper. Do not read a true return as evidence that a shaper ran.

### Features

`hb_feature_t` values are usually built with `hb_feature_from_string()`, which
accepts the `hb-shape` command-line syntax — `"kern"`, `"-liga"`,
`"aalt[3:5]=2"`. The `start`/`end` fields are **cluster** indices, matching the
cluster values you assigned when adding text to the buffer, not byte or glyph
offsets. `HB_FEATURE_GLOBAL_START` and `HB_FEATURE_GLOBAL_END` mean "the whole
buffer". Later array entries override earlier ones where their ranges overlap
on the same tag, which is what makes it possible to layer a global setting and
then a local exception.

Font *variations* are not passed here; they are set on the font
(`hb_font_set_variations()`) before shaping.

### Threading

HarfBuzz objects are not internally locked. Two threads may shape concurrently
provided they use distinct buffers, and distinct fonts *or* a font that neither
mutates. `hb_shape()` and `hb_shape_full()` themselves do not modify the font,
so a single immutable font (see `hb_font_make_immutable()`) shared across
threads is the normal pattern; the shape-plan cache they consult is internally
synchronised. `hb_shape_justify()` is the exception — it writes variation
coordinates to the font, so it needs a font of its own.

### Choosing a shaper

Prefer `hb_shape()`. Pin a shaper only when you have a reason to, and prefer
naming a list over a single entry so there is a fallback:

```c
static const char * const shapers[] = { "ot", "fallback", NULL };
hb_bool_t ok = hb_shape_full (font, buf, NULL, 0, shapers);
```

Passing a name that this build does not support simply means that entry is
skipped; if none of the listed shapers can handle the face, the call returns
false and the buffer's contents are left in whatever state the attempt reached.
Validate names against `hb_shape_list_shapers()` if they come from
configuration or a command line.

### Verification mode

If the buffer has `HB_BUFFER_FLAG_VERIFY` set, `hb_shape_full()` keeps a copy of
the input text and runs consistency checks on the output — monotonic clusters,
sane cluster mapping — turning a true result into false when they fail. This
costs an extra buffer allocation and a second pass, so it is a debugging aid,
not a production setting.

### Rust-side reminders

Every function here is `unsafe`, and this crate adds nothing on top of the C
contract. In particular:

- `shaper_list` is `*const *const c_char` — a pointer to a null-terminated
  array. Build it from a `[*const c_char; N]` whose last element is
  `core::ptr::null()`, and keep that array alive across the call.
- Pass `core::ptr::null()` and `0` for `features`/`num_features` when you have
  no feature overrides.
- `hb_shape_list_shapers()` returns `*mut *const c_char`; walk it until you hit
  a null element, and never free it.
- `hb_bool_t` is `c_int`. Compare against `0`, do not transmute to `bool`.
