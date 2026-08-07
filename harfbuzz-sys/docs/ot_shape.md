# OpenType shaping

Source header: `hb-ot-shape.h`. Rust module: `harfbuzz_sys::ot_shape` (glob
re-exported at the crate root).

## Overview

Despite its name, this header does **not** shape text. Shaping is driven from
`hb-shape.h` (`hb_shape()`, `hb_shape_full()`) and, at a lower level, from
`hb-shape-plan.h`. What `hb-ot-shape.h` provides is *introspection of the `ot`
shaper*: a small set of entry points that let you ask what HarfBuzz's own
OpenType implementation would do, without producing a shaped buffer, plus one
build-identification helper.

There are three jobs here, and they are only loosely related to each other:

1. **Glyph closure.** `hb_ot_shape_glyphs_closure()` answers "which glyphs
   could this text possibly turn into?" It maps every character in a buffer to
   its nominal glyph and then takes the transitive closure of that set under the
   GSUB lookups the `ot` shaper would activate, so ligature components, their
   ligatures, contextual alternates, and everything reachable from those end up
   in the result. This is the primitive a subsetter or a font-packager uses to
   decide which glyphs must survive.

2. **Shape-plan introspection.** `hb_ot_shape_plan_collect_lookups()` and
   `hb_ot_shape_plan_get_feature_tags()` open up an `hb_shape_plan_t` and report
   the GSUB/GPOS lookup indices and the OpenType feature tags that plan has
   compiled. They are how you find out which features the shaper actually
   enabled for a given script/language/direction combination — the shaper's own
   defaults (`ccmp`, `locl`, `rlig`, `calt`, `kern`, `mark`, `mkmk`, plus
   script-specific ones) merged with whatever user features you supplied, minus
   any that the font does not implement and HarfBuzz has no fallback for. Both
   functions are declared here but are documented upstream under the
   `hb-ot-layout` section, because their subject matter is layout tables.

3. **Buffer-format identification.** `HB_OT_SHAPE_BUFFER_FORMAT_SERIAL` and
   `hb_ot_shape_get_buffer_format_serial()` expose a version number for the
   *private* layout of `hb_glyph_info_t` and `hb_glyph_position_t`. Ordinary
   clients never need this. It exists for code — notably HarfBuzz's own
   HarfRust/fontations integration and test harnesses — that reaches into the
   reserved members of those structs and must refuse to run against a library
   whose internal format has changed underneath it.

Nothing in this header allocates or owns an object. The two closure/collect
functions take a caller-provided `hb_set_t` as an out-parameter and **add** to
it; the two introspection functions borrow a shape plan; the serial function
takes no arguments at all. There is no reference counting to manage and nothing
to destroy that this header handed you.

Everything here lives inside upstream's `#ifndef HB_NO_OT_SHAPE` guard, so a
build configured without the OpenType shaper does not export these symbols at
all. That configuration is not what this crate builds — the vendored HarfBuzz is
compiled with the `ot` shaper enabled — but it matters if you ever link against
a third-party, size-reduced `libharfbuzz`.

## Types

This header declares no types of its own. It uses, but does not define:

| Type | Declared in | Role here |
| --- | --- | --- |
| `hb_font_t` | `hb-font.h` | Supplies the character-to-glyph mapping and, through its face, the layout tables consulted by the closure. |
| `hb_buffer_t` | `hb-buffer.h` | Input to the closure: its codepoints and its segment properties (script/language/direction). Read only; never shaped. |
| `hb_feature_t` | `hb-common.h` | One user feature setting — `tag`, `value`, and the cluster range `[start, end)`. |
| `hb_set_t` | `hb-set.h` | The out-parameter for both the glyph closure and the lookup collection. Integer set; also carries an out-of-memory flag. |
| `hb_shape_plan_t` | `hb-shape-plan.h` | The compiled plan being introspected. |
| `hb_tag_t` | `hb-common.h` | Four-byte table tag (`GSUB`/`GPOS`) on input, feature tags on output. |

## Constants

### `HB_OT_SHAPE_BUFFER_FORMAT_SERIAL`

```c
#define HB_OT_SHAPE_BUFFER_FORMAT_SERIAL 1
```

```rust
pub const HB_OT_SHAPE_BUFFER_FORMAT_SERIAL: c_uint = 1;
```

The serial number of the current internal buffer format, as of the headers you
compiled against. The number increases whenever the *private* members of
`hb_glyph_info_t` and `hb_glyph_position_t` change their format — the reserved
`var1`/`var2` slots in `hb_glyph_info_t` and the `var` slot in
`hb_glyph_position_t`, which HarfBuzz uses as scratch space during shaping and
which are not part of the public contract.

Type: the header spells it as the bare literal `1`. It is transcribed as
`c_uint` because the only meaningful use is comparing it against
`hb_ot_shape_get_buffer_format_serial()`, which returns `unsigned int`.

Public glyph-info and glyph-position fields — `codepoint`, `cluster`,
`x_advance`, `y_advance`, `x_offset`, `y_offset` — are **not** covered by this
serial. If you only read those, ignore this constant entirely.

Since HarfBuzz 13.2.0.

## Functions

### Glyph closure

#### `hb_ot_shape_glyphs_closure`

```c
void
hb_ot_shape_glyphs_closure (hb_font_t          *font,
                            hb_buffer_t        *buffer,
                            const hb_feature_t *features,
                            unsigned int        num_features,
                            hb_set_t           *glyphs);
```

```rust
pub fn hb_ot_shape_glyphs_closure(
    font: *mut hb_font_t,
    buffer: *mut hb_buffer_t,
    features: *const hb_feature_t,
    num_features: c_uint,
    glyphs: *mut hb_set_t,
);
```

Computes the transitive closure of glyphs needed to shape `buffer` with `font`
under `features`, and adds them to `glyphs`. The closure is computed as a set,
not as a list: it has no order and no multiplicity, and it does not tell you
which input character produced which glyph.

What the implementation actually does, in order:

1. Builds a **cached** shape plan from `font`'s face and the buffer's segment
   properties, with the shaper list pinned to `{"ot", NULL}` and with
   `features`/`num_features` as the user features.
2. Decides whether to mirror, by testing
   `hb_script_get_horizontal_direction (buffer->props.script) == HB_DIRECTION_RTL`.
   Note that this consults the buffer's **script**, not its direction.
3. For each codepoint currently in the buffer, adds that character's nominal
   glyph to `glyphs`; and if mirroring is on and the character has a distinct
   Unicode mirror, adds the mirror's nominal glyph too. Characters the font has
   no nominal glyph for contribute nothing — not even `.notdef`.
4. Collects the plan's GSUB lookups into a temporary set and runs
   `hb_ot_layout_lookups_substitute_closure()` over `glyphs`, which iterates to a
   fixed point.
5. Destroys the temporary set and releases the plan.

**Parameters**

| Parameter | Direction | Meaning |
| --- | --- | --- |
| `font` | in | The font to work upon. Its face supplies GSUB; its funcs supply the nominal-glyph mapping. Not modified. Nullability unspecified by the header — but note that a null `font` is dereferenced immediately (`font->face`), so pass `hb_font_get_empty()` rather than `NULL` for the degenerate case. |
| `buffer` | in | Holds the input characters. Must contain **Unicode codepoints**, not glyphs. Its `props` are read directly; they are not guessed for you. Not modified. |
| `features` | in | Array of `num_features` user features, exactly as you would pass to `hb_shape()`. May be `NULL`. |
| `num_features` | in | Length of `features`; pass `0` with a null `features`. |
| `glyphs` | out | Receives the closure. **Added to, not cleared** — pass a fresh `hb_set_t` unless you deliberately want to accumulate across several calls. |

**Returns** — nothing. There is no success flag.

**Ownership** — borrows everything. The plan it builds internally is created and
destroyed inside the call. `features` is read during the call only. `glyphs`
stays yours: you created it with `hb_set_create()` and you must destroy it with
`hb_set_destroy()`.

**Notes**

- Only **GSUB** is consulted. GPOS is irrelevant to a glyph closure and is
  ignored, so `table_tag`-style selection is not offered here.
- The result reflects the `ot` shaper specifically. Even if `hb_shape()` would
  have picked `coretext` or `graphite2` for this face, the closure is HarfBuzz's
  OpenType view of it.
- Failure is silent. If the internal set allocation or the closure iteration
  runs out of memory, the call simply returns with an incomplete `glyphs`;
  check `hb_set_allocation_successful (glyphs)` afterwards if that matters.
- Thread safety: safe to call concurrently on distinct buffers and distinct
  `glyphs` sets, provided the font is not being mutated. The internal use of
  `hb_shape_plan_create_cached()` touches the face's plan cache, which is
  internally synchronised, but it also calls `hb_face_make_immutable()` on the
  font's face as a side effect of plan creation — after the first closure the
  face can no longer be modified.
- Since HarfBuzz 0.9.2.

### Shape-plan introspection

Both functions in this group are declared in `hb-ot-shape.h` but appear under
the `hb-ot-layout` section of upstream's `harfbuzz-sections.txt`.

#### `hb_ot_shape_plan_collect_lookups`

```c
void
hb_ot_shape_plan_collect_lookups (hb_shape_plan_t *shape_plan,
                                  hb_tag_t         table_tag,
                                  hb_set_t        *lookup_indexes /* OUT */);
```

```rust
pub fn hb_ot_shape_plan_collect_lookups(
    shape_plan: *mut hb_shape_plan_t,
    table_tag: hb_tag_t,
    lookup_indexes: *mut hb_set_t,
);
```

Computes the complete set of GSUB or GPOS lookups that are applicable under
`shape_plan`, and adds their **indices** — not the lookups themselves — to
`lookup_indexes`. Those indices are positions in the face's lookup list for the
selected table, and are what `hb_ot_layout_lookup_*()` in `hb-ot-layout.h`
takes.

**Parameters**

| Parameter | Direction | Meaning |
| --- | --- | --- |
| `shape_plan` | in | The plan to query. Every plan built by `hb_shape_plan_create*()` carries a compiled `ot` map regardless of which shaper it ultimately selected, so this works even on a plan whose chosen shaper is `coretext` or `graphite2`. Not modified. |
| `table_tag` | in | `HB_OT_TAG_GSUB` or `HB_OT_TAG_GPOS`. **Any other value is a silent no-op**: the function returns without touching `lookup_indexes`, with no error signalled. |
| `lookup_indexes` | out | Receives the lookup indices. **Added to, not cleared.** |

**Returns** — nothing.

**Ownership** — borrows the plan; does not take a reference on it and does not
destroy it. `lookup_indexes` remains the caller's to destroy.

**Notes**

- The result is the union across all shaping *stages*: HarfBuzz applies lookups
  in ordered stages, and the set flattens that ordering away. If you need the
  order, this is not the API for it.
- On `hb_shape_plan_get_empty()` the call is well-defined and adds nothing.
- No allocation-failure signal; check `hb_set_allocation_successful()` if the
  set matters.
- Since HarfBuzz 0.9.7.

#### `hb_ot_shape_plan_get_feature_tags`

```c
unsigned int
hb_ot_shape_plan_get_feature_tags (hb_shape_plan_t *shape_plan,
                                   unsigned int     start_offset,
                                   unsigned int    *tag_count, /* IN/OUT */
                                   hb_tag_t        *tags /* OUT */);
```

```rust
pub fn hb_ot_shape_plan_get_feature_tags(
    shape_plan: *mut hb_shape_plan_t,
    start_offset: c_uint,
    tag_count: *mut c_uint,
    tags: *mut hb_tag_t,
) -> c_uint;
```

Fetches the list of OpenType feature tags enabled for a shaping plan. This is
HarfBuzz's standard paged-array accessor shape: `start_offset` picks the window,
`tag_count` is in/out, and the return value is the grand total.

**Parameters**

| Parameter | Direction | Meaning |
| --- | --- | --- |
| `shape_plan` | in | The plan to query. Not modified. |
| `start_offset` | in | Index of the first feature to retrieve. May exceed the total, in which case `*tag_count` comes back as `0`. |
| `tag_count` | in/out | On entry, the capacity of `tags`. On return, `min(total - start_offset, capacity)`, which may be `0`. May be `NULL`, in which case nothing is written and only the return value is meaningful. |
| `tags` | out | Array of at least `*tag_count` entries, filled with the feature tags. May be `NULL` — `*tag_count` is still clamped, so a non-null `tag_count` with a null `tags` is a legal "how many are in this window?" query. |

**Returns** — the total number of feature tags in the plan, independent of
`start_offset` and of `tag_count`. Call once with `tag_count = NULL` to size a
buffer, then again to fill it.

**Ownership** — borrows the plan. `tags` is caller-allocated storage; the values
written are plain 32-bit tags with no ownership attached.

**Notes**

- The tags come back **sorted by tag value ascending**. The plan stores its
  feature map as a sorted vector keyed on the tag, so the order is neither
  application order nor the order you passed your user features in.
- The list is the *effective* feature set: shaper defaults plus your user
  features, minus features the font implements in neither GSUB nor GPOS and for
  which HarfBuzz has no fallback implementation, minus features explicitly
  disabled (`value = 0`) or dropped for want of mask bits. So a tag's presence
  means "this will actually do something"; a tag's absence after you requested
  it means the font does not have it.
- Each tag appears once, even though a feature can be registered against both
  GSUB and GPOS and across several stages.
- On a library built with `HB_NO_OT_SHAPE`, the function is not exported at all;
  its `#else` branch (write `0`, return `0`) is unreachable because the whole
  translation unit is compiled out.
- Since HarfBuzz 10.3.0.

### Build identification

#### `hb_ot_shape_get_buffer_format_serial`

```c
unsigned int
hb_ot_shape_get_buffer_format_serial (void);
```

```rust
pub fn hb_ot_shape_get_buffer_format_serial() -> c_uint;
```

Returns the serial number of the internal buffer format that the *linked
library* was built with. The implementation is a one-line
`return HB_OT_SHAPE_BUFFER_FORMAT_SERIAL;`, compiled into the library — which is
exactly the point: comparing the returned value against the
`HB_OT_SHAPE_BUFFER_FORMAT_SERIAL` your own translation unit sees tells you
whether your headers and the shared library agree on the private layout of
`hb_glyph_info_t` and `hb_glyph_position_t`.

**Parameters** — none.

**Returns** — the current buffer-format serial number. `1` for HarfBuzz 13.2.0
through 14.3.0.

**Ownership** — none; a plain integer.

**Notes**

- Thread-safe, side-effect free, constant for the life of the process.
- Because this crate compiles the HarfBuzz sources it vendors, the constant and
  the function can never disagree here. The check earns its keep only when the
  crate is reconfigured to link a system `libharfbuzz`.
- Since HarfBuzz 13.2.0.

## Usage

### Glyph closure for a subsetter (C)

```c
hb_blob_t *blob = hb_blob_create_from_file_or_fail ("NotoSans-Regular.ttf");
hb_face_t *face = hb_face_create (blob, 0);
hb_font_t *font = hb_font_create (face);

hb_buffer_t *buf = hb_buffer_create ();
hb_buffer_add_utf8 (buf, "office affluent", -1, 0, -1);
hb_buffer_guess_segment_properties (buf);   /* required: props are read as-is */

hb_set_t *glyphs = hb_set_create ();
hb_ot_shape_glyphs_closure (font, buf, NULL, 0, glyphs);

if (!hb_set_allocation_successful (glyphs))
  /* out of memory: the closure is incomplete */;

hb_codepoint_t g = HB_SET_VALUE_INVALID;
while (hb_set_next (glyphs, &g))
  printf ("keep glyph %u\n", g);

hb_set_destroy (glyphs);
hb_buffer_destroy (buf);
hb_font_destroy (font);
hb_face_destroy (face);
hb_blob_destroy (blob);
```

The closure is a superset of what shaping this exact string produces: it
includes every glyph any GSUB lookup could reach from the input glyphs, so for
`"office"` you get `f`, `i`, `c`, `e`, `o` *and* `fi`, `ffi`, and any further
ligature or alternate built from those.

### Glyph closure (Rust)

```rust
use core::ffi::c_char;
use core::ptr;

use harfbuzz_sys::*;

unsafe fn closure_for(font: *mut hb_font_t, text: &str) -> *mut hb_set_t {
    let buf = hb_buffer_create();
    hb_buffer_add_utf8(
        buf,
        text.as_ptr() as *const c_char,
        text.len() as i32,
        0,
        text.len() as i32,
    );
    hb_buffer_guess_segment_properties(buf);

    let glyphs = hb_set_create();
    hb_ot_shape_glyphs_closure(font, buf, ptr::null(), 0, glyphs);

    hb_buffer_destroy(buf);
    glyphs // caller destroys with hb_set_destroy
}

unsafe fn print_closure(glyphs: *mut hb_set_t) {
    if hb_set_allocation_successful(glyphs) == 0 {
        return; // out of memory; contents are incomplete
    }
    let mut g: hb_codepoint_t = HB_SET_VALUE_INVALID;
    while hb_set_next(glyphs, &mut g) != 0 {
        println!("glyph {g}");
    }
}
```

Note that `hb_buffer_add_utf8` takes byte lengths as `c_int`, and that the
`item_offset`/`item_length` pair is what controls context; passing the full
range as above is the common case.

### Which features did the shaper enable? (C)

```c
hb_segment_properties_t props = HB_SEGMENT_PROPERTIES_DEFAULT;
props.direction = HB_DIRECTION_LTR;
props.script    = HB_SCRIPT_ARABIC;
props.language  = hb_language_from_string ("ar", -1);

hb_shape_plan_t *plan =
  hb_shape_plan_create (face, &props, NULL, 0, NULL);

/* 1. size */
unsigned int total = hb_ot_shape_plan_get_feature_tags (plan, 0, NULL, NULL);

/* 2. fetch, in one window */
hb_tag_t *tags  = malloc (total * sizeof (hb_tag_t));
unsigned int n  = total;
hb_ot_shape_plan_get_feature_tags (plan, 0, &n, tags);

for (unsigned int i = 0; i < n; i++)
  {
    char s[5] = {0};
    hb_tag_to_string (tags[i], s);
    printf ("%s\n", s);   /* sorted ascending by tag */
  }

free (tags);
hb_shape_plan_destroy (plan);
```

Paging works the same way as elsewhere in HarfBuzz — loop while
`start_offset < total`, resetting `n` to your buffer capacity on every
iteration.

### Which features did the shaper enable? (Rust)

```rust
use core::ffi::c_uint;
use core::ptr;

use harfbuzz_sys::*;

/// Returns every feature tag the plan enabled, sorted ascending.
unsafe fn plan_feature_tags(plan: *mut hb_shape_plan_t) -> Vec<hb_tag_t> {
    let total = hb_ot_shape_plan_get_feature_tags(plan, 0, ptr::null_mut(), ptr::null_mut());

    let mut tags = vec![0 as hb_tag_t; total as usize];
    let mut count: c_uint = total;
    hb_ot_shape_plan_get_feature_tags(plan, 0, &mut count, tags.as_mut_ptr());
    tags.truncate(count as usize);
    tags
}

fn tag_string(tag: hb_tag_t) -> String {
    HB_UNTAG(tag).iter().map(|&b| b as char).collect()
}
```

`HB_UNTAG` is the crate's `const fn` form of the C macro; it returns the four
bytes most-significant first.

### Which lookups will run? (C)

```c
hb_set_t *gsub = hb_set_create ();
hb_set_t *gpos = hb_set_create ();

hb_ot_shape_plan_collect_lookups (plan, HB_OT_TAG_GSUB, gsub);
hb_ot_shape_plan_collect_lookups (plan, HB_OT_TAG_GPOS, gpos);

printf ("%u substitution lookups, %u positioning lookups\n",
        hb_set_get_population (gsub),
        hb_set_get_population (gpos));

/* Feed the indices to hb-ot-layout.h, e.g. to grow a glyph closure: */
hb_ot_layout_lookups_substitute_closure (face, gsub, glyphs);

hb_set_destroy (gpos);
hb_set_destroy (gsub);
```

`HB_OT_TAG_GSUB` and `HB_OT_TAG_GPOS` come from `hb-ot-layout.h`; they are
simply `HB_TAG('G','S','U','B')` and `HB_TAG('G','P','O','S')`. In Rust you can
write `HB_TAG(b'G', b'S', b'U', b'B')` directly.

### Guarding access to private buffer members (C)

```c
if (hb_ot_shape_get_buffer_format_serial () != HB_OT_SHAPE_BUFFER_FORMAT_SERIAL)
  {
    fprintf (stderr,
             "libharfbuzz internal buffer format changed "
             "(built against %u, running %u); refusing to poke at var1/var2\n",
             HB_OT_SHAPE_BUFFER_FORMAT_SERIAL,
             hb_ot_shape_get_buffer_format_serial ());
    return EXIT_FAILURE;
  }
```

In Rust the same check is:

```rust
unsafe {
    assert_eq!(
        hb_ot_shape_get_buffer_format_serial(),
        HB_OT_SHAPE_BUFFER_FORMAT_SERIAL,
        "harfbuzz internal buffer format mismatch",
    );
}
```

Only reach for this if you are reading the reserved members of
`hb_glyph_info_t`/`hb_glyph_position_t`. For the public fields it is noise.

## Pitfalls

**The out-parameter sets are additive.** Neither
`hb_ot_shape_glyphs_closure()` nor `hb_ot_shape_plan_collect_lookups()` clears
the `hb_set_t` you hand it. Reusing one set across calls silently unions the
results — occasionally what you want, more often a bug. Call `hb_set_clear()`
between uses, or pass a fresh set.

**A bad `table_tag` is silent.** `hb_ot_shape_plan_collect_lookups()` accepts
any `hb_tag_t`, and everything that is not `GSUB` or `GPOS` returns immediately
having done nothing. There is no return value and no error flag, so a typo'd tag
looks exactly like a font with no lookups. Build the tag from the
`HB_OT_TAG_GSUB`/`HB_OT_TAG_GPOS` constants rather than from a string.

**The closure reads the buffer's properties, not your intent.**
`hb_ot_shape_glyphs_closure()` uses `buffer->props` verbatim. If you never
called `hb_buffer_set_direction()`/`set_script()`/`set_language()` or
`hb_buffer_guess_segment_properties()`, the script is `HB_SCRIPT_INVALID`, the
plan is built for an unknown script, no mirroring happens, and the closure will
quietly be too small. Set the properties first, every time.

**Mirroring keys off the script, not the direction.** The decision to add
Unicode mirrored forms is
`hb_script_get_horizontal_direction (props.script) == HB_DIRECTION_RTL`. Forcing
`direction = HB_DIRECTION_RTL` on a Latin buffer does *not* turn mirroring on.

**The buffer must still hold characters.** The closure walks
`info[i].codepoint` and feeds each value to the font's nominal-glyph mapping. If
you pass a buffer that has already been through `hb_shape()`, those codepoints
are glyph indices, and you will get a meaningless closure with no diagnostic.

**Characters without a glyph vanish.** Unmapped characters are skipped rather
than contributing `.notdef`. If you need to know that some input was unsupported,
compare populations yourself; the closure will not tell you.

**The closure only covers GSUB.** No positioning lookup, no `MATH` variant, no
COLR layer, no `CPAL` palette entry, and no composite-glyph component is
followed. A subsetter needs `hb-subset.h` (or at minimum a `glyf` component
walk) on top of this; the closure is one input to that decision, not the whole
answer.

**The closure pins the `ot` shaper.** It passes `{"ot", NULL}` as the shaper
list. On a face that `hb_shape()` would hand to CoreText or Graphite, the
closure still describes HarfBuzz's OpenType interpretation, which may differ.

**Plan creation freezes the face.** Both the closure (internally) and any plan
you build for the introspection functions call `hb_face_make_immutable()`.
Subsequent `hb_face_set_upem()`, `hb_face_set_index()`, and friends become
no-ops. Finish configuring the face before you introspect it.

**`hb_ot_shape_plan_get_feature_tags()` returns a total, not a count.** The
return value ignores both `start_offset` and `*tag_count`. The number of tags
actually written is in `*tag_count`. Sizing a loop off the return value while
reading `tags[0..total]` overruns your buffer whenever you passed a smaller
capacity.

**Feature tags come back sorted, not in application order.** The plan's feature
map is a tag-sorted vector. Do not read the sequence as "the order the shaper
applies these in" — for that you need the lookup indices, and even those are
returned as an unordered set.

**Requested features can disappear.** A user feature that the font implements in
neither GSUB nor GPOS, and for which HarfBuzz has no fallback, never makes it
into the plan's map, so it will not appear in the tag list. That is the intended
signal that the request had no effect — but it means the tag list is not an echo
of your input.

**No allocation-failure signal anywhere.** Both `void` functions can run out of
memory mid-way and return normally. `hb_set_allocation_successful()` on your
out-set is the only way to notice.

**The serial is about private members only.** `HB_OT_SHAPE_BUFFER_FORMAT_SERIAL`
changing does not mean `codepoint`, `cluster`, or the position fields moved —
those are ABI-stable public API. Checking the serial for ordinary shaping code
adds a failure mode without adding safety.

**These symbols can be absent.** Everything here is inside upstream's
`#ifndef HB_NO_OT_SHAPE`. A `libharfbuzz` built for size with the OpenType
shaper disabled exports none of them, and you get a link error rather than a
runtime fallback. This crate's vendored build always includes them.

**Rust-side reminders.** Every function here is `unsafe` and this crate adds
nothing to the C contract: `hb_bool_t` is a `c_int` to compare against `0`, not
a `bool`; pass `core::ptr::null()`/`null_mut()` for the optional pointers rather
than a dangling one; and remember that `glyphs`/`lookup_indexes` are `*mut
hb_set_t` values you created and must destroy yourself.
