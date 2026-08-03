# Fonts

C header: `hb-font.h` · Rust module: `harfbuzz_sys::font` (glob re-exported at the crate root)

## Overview

A **face** (`hb_face_t`) is a typeface as it exists in a file: tables, outlines, a
units-per-em value. A **font** (`hb_font_t`) is that face *instantiated* — pinned to a
scale, and optionally to a pixel size, a point size, a set of variation-axis values, and
synthetic bold/slant parameters. Shaping consumes a font, not a face, because the
positions it produces are in the font's scaled coordinate space. Fonts are described
upstream as "very light-weight objects": creating one is cheap, and creating several from
one face is the normal way to shape the same typeface at several sizes.

Every metric question HarfBuzz asks of a font — what glyph is this code point, how wide is
that glyph, where is its origin, what does its outline look like — goes through a table of
virtual methods, the **font functions** object (`hb_font_funcs_t`). Each font has one
attached, together with an opaque `font_data` pointer that the callbacks receive. This
indirection is the extension point of the whole library: it is how the FreeType backend
(`hb-ft.h`), the CoreText backend, and any client-supplied font engine plug in. HarfBuzz
also ships built-in implementations; `hb_font_list_funcs()` reports which ones this build
contains, and `hb_font_set_funcs_using()` selects one by name.

Font objects form a **parent chain**. `hb_font_create_sub_font()` makes a child that
replicates its parent's properties, and the *default* implementation of every font-funcs
method delegates to the parent font's methods. That is what makes partial overriding
practical: install a fresh `hb_font_funcs_t` on a sub-font, set only
`glyph_h_advance_func`, and every other query still resolves through the parent. It is
also how scaling works — a sub-font at a different scale returns its parent's answers
converted into its own coordinate space.

Both `hb_font_t` and `hb_font_funcs_t` are **reference-counted, opaque objects** following
the standard HarfBuzz object protocol: `_create` returns a reference you own,
`_reference` adds one, `_destroy` removes one and frees at zero, `_get_empty` returns a
shared immutable singleton, `_set_user_data`/`_get_user_data` attach client data keyed by
the *address* of an `hb_user_data_key_t`, and `_make_immutable`/`_is_immutable` freeze the
object. Once an object is immutable, every setter on it becomes a silent no-op — this is
the single most common source of confusion in this API, because nothing reports the
failure.

The function surface splits into four layers, and it helps to keep them straight:

1. **Setters on `hb_font_funcs_t`** (`hb_font_funcs_set_*_func`) — install a callback.
2. **Dispatch functions** (`hb_font_get_h_extents`, `hb_font_get_glyph_h_advance`, …) —
   call the installed callback for a font and return its result verbatim.
3. **Direction-dispatching wrappers** (`*_for_direction`) — pick the horizontal or vertical
   variant based on an `hb_direction_t`, and supply fallbacks where a font has no data.
4. **Property accessors on `hb_font_t`** (scale, ppem, ptem, variations, synthetic bold) —
   configure the instantiation itself.

## Types

### `hb_font_t`

Opaque, reference-counted. A face at a specific size and configuration. Only ever handled
through pointers. In Rust it is an `opaque_handle!` type: zero-sized, not constructible,
`!Send`/`!Sync`.

Note that although `hb_font_t` is *forward-declared* in `hb-common.h`, all of its API lives
in `hb-font.h`, so this crate declares the type in the `font` module.

### `hb_font_funcs_t`

Opaque, reference-counted. A table of virtual methods for querying a font. A freshly
created one has every method set to HarfBuzz's lightweight default (which delegates to the
parent font); a client replaces individual entries with the `hb_font_funcs_set_*_func`
setters.

### `hb_font_extents_t`

Font-wide extents, in **scaled units** (unlike `hb_glyph_extents_t`, which the header
documents as font units).

| Field | Type | Meaning |
| --- | --- | --- |
| `ascender` | `hb_position_t` | Height of typographic ascenders. Typically positive in a coordinate system that grows up. |
| `descender` | `hb_position_t` | Depth of typographic descenders. Typically negative in a coordinate system that grows up. |
| `line_gap` | `hb_position_t` | Suggested line-spacing gap. |
| `reserved9` … `reserved1` | `hb_position_t` | Private padding, marked `/*< private >*/` in the header. Present only so the Rust struct has the same size and layout as the C one. Do not read or write them; upstream may repurpose them. |

The struct is 12 `int32_t`s — 48 bytes. Derives `Debug, Clone, Copy, PartialEq, Eq, Hash`
because every field is integral.

### Virtual method typedefs

Every callback is transcribed as `Option<unsafe extern "C" fn(...)>`, so `None` is the null
pointer. Every one of them receives, in order: the `hb_font_t` being queried, the
`font_data` pointer that was attached with `hb_font_set_funcs`, the query's own arguments,
and finally the `user_data` pointer that was supplied when *this particular method* was
installed. Those two data pointers are distinct and it is easy to conflate them.

| Typedef | Returns | Query-specific parameters |
| --- | --- | --- |
| `hb_font_get_font_extents_func_t` | `hb_bool_t` | `extents: *mut hb_font_extents_t` (out) |
| `hb_font_get_font_h_extents_func_t` | — | alias of the above, horizontal |
| `hb_font_get_font_v_extents_func_t` | — | alias of the above, vertical |
| `hb_font_get_nominal_glyph_func_t` | `hb_bool_t` | `unicode`, `glyph` (out) |
| `hb_font_get_variation_glyph_func_t` | `hb_bool_t` | `unicode`, `variation_selector`, `glyph` (out) |
| `hb_font_get_nominal_glyphs_func_t` | `c_uint` (count processed) | `count`, `first_unicode`, `unicode_stride`, `first_glyph` (out), `glyph_stride` |
| `hb_font_get_glyph_advance_func_t` | `hb_position_t` | `glyph` |
| `hb_font_get_glyph_h_advance_func_t` | — | alias, horizontal |
| `hb_font_get_glyph_v_advance_func_t` | — | alias, vertical |
| `hb_font_get_glyph_advances_func_t` | `void` | `count`, `first_glyph`, `glyph_stride`, `first_advance` (out), `advance_stride` |
| `hb_font_get_glyph_h_advances_func_t` | — | alias, horizontal |
| `hb_font_get_glyph_v_advances_func_t` | — | alias, vertical |
| `hb_font_get_glyph_origin_func_t` | `hb_bool_t` | `glyph`, `x` (out), `y` (out) |
| `hb_font_get_glyph_h_origin_func_t` | — | alias, horizontal |
| `hb_font_get_glyph_v_origin_func_t` | — | alias, vertical |
| `hb_font_get_glyph_origins_func_t` | `hb_bool_t` | `count`, `first_glyph`, `glyph_stride`, `first_x` (out), `x_stride`, `first_y` (out), `y_stride`. Since 11.3.0 |
| `hb_font_get_glyph_h_origins_func_t` | — | alias, horizontal. Since 11.3.0 |
| `hb_font_get_glyph_v_origins_func_t` | — | alias, vertical. Since 11.3.0 |
| `hb_font_get_glyph_kerning_func_t` | `hb_position_t` | `first_glyph`, `second_glyph` |
| `hb_font_get_glyph_h_kerning_func_t` | — | alias, horizontal (there is no `v` counterpart in this header) |
| `hb_font_get_glyph_extents_func_t` | `hb_bool_t` | `glyph`, `extents` (out) |
| `hb_font_get_glyph_contour_point_func_t` | `hb_bool_t` | `glyph`, `point_index`, `x` (out), `y` (out) |
| `hb_font_get_glyph_name_func_t` | `hb_bool_t` | `glyph`, `name` (out buffer), `size` |
| `hb_font_get_glyph_from_name_func_t` | `hb_bool_t` | `name`, `len` (`-1` = NUL-terminated), `glyph` (out) |
| `hb_font_draw_glyph_or_fail_func_t` | `hb_bool_t` | `glyph`, `draw_funcs`, `draw_data`. Since 11.2.0 |
| `hb_font_paint_glyph_or_fail_func_t` | `hb_bool_t` | `glyph`, `paint_funcs`, `paint_data`, `palette_index`, `foreground`. Since 11.2.0 |

Several of these are C `typedef`s of one another, not distinct types — for example
`hb_font_get_glyph_h_advance_func_t` and `hb_font_get_glyph_v_advance_func_t` are both
spellings of `hb_font_get_glyph_advance_func_t`. The Rust transcription preserves that:
they are type *aliases*, so the compiler will not stop you passing a horizontal callback to
the vertical setter. That mirrors C exactly, and it is a real hazard.

The plural (`*s_func_t`) variants take **strided arrays**. `first_*` points at element
zero and the corresponding `*_stride` is the byte offset from one element to the next, which
lets a caller read glyph IDs straight out of an array of structs without repacking. A
stride of `0` therefore reads or writes the same slot repeatedly — almost certainly a bug.

### Constants

| Constant | Rust type | Value | Meaning |
| --- | --- | --- | --- |
| `HB_FONT_NO_VAR_NAMED_INSTANCE` | `c_uint` | `0xFFFFFFFF` | No named-instance index is set. A font's default. Since 7.0.0. |

There are no enumerations in this header, and no function-like macros, so nothing was
skipped on those grounds.

## Functions

### Font-functions lifecycle

```c
hb_font_funcs_t *hb_font_funcs_create (void);
hb_font_funcs_t *hb_font_funcs_get_empty (void);
hb_font_funcs_t *hb_font_funcs_reference (hb_font_funcs_t *ffuncs);
void             hb_font_funcs_destroy (hb_font_funcs_t *ffuncs);
```

```rust
pub fn hb_font_funcs_create() -> *mut hb_font_funcs_t;
pub fn hb_font_funcs_get_empty() -> *mut hb_font_funcs_t;
pub fn hb_font_funcs_reference(ffuncs: *mut hb_font_funcs_t) -> *mut hb_font_funcs_t;
pub fn hb_font_funcs_destroy(ffuncs: *mut hb_font_funcs_t);
```

`hb_font_funcs_create` returns a new structure with every method at HarfBuzz's default
implementation, and transfers one reference to the caller, who must eventually
`hb_font_funcs_destroy` it. Under HarfBuzz's usual object conventions it returns the empty
singleton rather than null on allocation failure, but the header does not state that, so
treat the result as potentially null unless you have checked upstream behaviour.

`hb_font_funcs_get_empty` returns a shared, immutable singleton whose methods report
failure. It is safe to `destroy` and `reference` like any other instance. Since 0.9.2 for
all four.

`hb_font_funcs_reference` returns its argument after incrementing the count.
`hb_font_funcs_destroy` decrements; at zero the structure is freed along with every
`user_data` it holds, each released through the `destroy` callback that was registered
with it.

```c
hb_bool_t hb_font_funcs_set_user_data (hb_font_funcs_t *ffuncs, hb_user_data_key_t *key,
                                       void *data, hb_destroy_func_t destroy, hb_bool_t replace);
void     *hb_font_funcs_get_user_data (const hb_font_funcs_t *ffuncs, hb_user_data_key_t *key);
void      hb_font_funcs_make_immutable (hb_font_funcs_t *ffuncs);
hb_bool_t hb_font_funcs_is_immutable (hb_font_funcs_t *ffuncs);
```

`set_user_data` returns true on success. When `replace` is false and a value already
exists for `key`, the call fails and the new `data` is *not* stored. `get_user_data`
returns a borrowed pointer — do not free it — and returns null when nothing is stored under
that key. `make_immutable` freezes the structure; every `hb_font_funcs_set_*_func` call
afterwards is silently ignored. All since 0.9.2.

### Installing virtual methods

Every setter has the same shape:

```c
void hb_font_funcs_set_XXX_func (hb_font_funcs_t *ffuncs,
                                 hb_font_XXX_func_t func,
                                 void *user_data, hb_destroy_func_t destroy);
```

```rust
pub fn hb_font_funcs_set_XXX_func(
    ffuncs: *mut hb_font_funcs_t,
    func: hb_font_XXX_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

`user_data` is handed to `func` on every invocation. `destroy` is nullable; HarfBuzz calls
it on `user_data` when the method is replaced or the structure is destroyed. Passing
`None`/`NULL` for `func` restores the default implementation for that method. None of the
setters return anything, so a call on an immutable `ffuncs` fails silently — check
`hb_font_funcs_is_immutable` first if that matters.

| Setter | Installs | Since |
| --- | --- | --- |
| `hb_font_funcs_set_font_h_extents_func` | `hb_font_get_font_h_extents_func_t` | 1.1.2 |
| `hb_font_funcs_set_font_v_extents_func` | `hb_font_get_font_v_extents_func_t` | 1.1.2 |
| `hb_font_funcs_set_nominal_glyph_func` | `hb_font_get_nominal_glyph_func_t` | 1.2.3 |
| `hb_font_funcs_set_nominal_glyphs_func` | `hb_font_get_nominal_glyphs_func_t` | 2.0.0 |
| `hb_font_funcs_set_variation_glyph_func` | `hb_font_get_variation_glyph_func_t` | 1.2.3 |
| `hb_font_funcs_set_glyph_h_advance_func` | `hb_font_get_glyph_h_advance_func_t` | 0.9.2 |
| `hb_font_funcs_set_glyph_v_advance_func` | `hb_font_get_glyph_v_advance_func_t` | 0.9.2 |
| `hb_font_funcs_set_glyph_h_advances_func` | `hb_font_get_glyph_h_advances_func_t` | 1.8.6 |
| `hb_font_funcs_set_glyph_v_advances_func` | `hb_font_get_glyph_v_advances_func_t` | 1.8.6 |
| `hb_font_funcs_set_glyph_h_origin_func` | `hb_font_get_glyph_h_origin_func_t` | 0.9.2 |
| `hb_font_funcs_set_glyph_v_origin_func` | `hb_font_get_glyph_v_origin_func_t` | 0.9.2 |
| `hb_font_funcs_set_glyph_h_origins_func` | `hb_font_get_glyph_h_origins_func_t` | 11.3.0 |
| `hb_font_funcs_set_glyph_v_origins_func` | `hb_font_get_glyph_v_origins_func_t` | 11.3.0 |
| `hb_font_funcs_set_glyph_h_kerning_func` | `hb_font_get_glyph_h_kerning_func_t` | 0.9.2 |
| `hb_font_funcs_set_glyph_extents_func` | `hb_font_get_glyph_extents_func_t` | 0.9.2 |
| `hb_font_funcs_set_glyph_contour_point_func` | `hb_font_get_glyph_contour_point_func_t` | 0.9.2 |
| `hb_font_funcs_set_glyph_name_func` | `hb_font_get_glyph_name_func_t` | 0.9.2 |
| `hb_font_funcs_set_glyph_from_name_func` | `hb_font_get_glyph_from_name_func_t` | 0.9.2 |
| `hb_font_funcs_set_draw_glyph_or_fail_func` | `hb_font_draw_glyph_or_fail_func_t` | 11.2.0 |
| `hb_font_funcs_set_paint_glyph_or_fail_func` | `hb_font_paint_glyph_or_fail_func_t` | 11.2.0 |

Note the asymmetry: there is a `glyph_h_kerning` setter but no `glyph_v_kerning` setter in
this header, and no setter at all for the singular `hb_font_get_font_extents_func_t` — you
install the horizontal and vertical variants separately.

### Metric queries (dispatch)

These call straight into the font's installed font functions and return the result
unchanged. All take `font: *mut hb_font_t` first.

```c
hb_bool_t hb_font_get_h_extents (hb_font_t *font, hb_font_extents_t *extents);
hb_bool_t hb_font_get_v_extents (hb_font_t *font, hb_font_extents_t *extents);
```

Fetch font-wide extents for horizontal or vertical text. Return true if data was found.
The header does not promise that `*extents` is left untouched on failure; HarfBuzz's
defaults zero it, but do not rely on reading it after a false return. Since 1.1.3.

```c
hb_bool_t    hb_font_get_nominal_glyph (hb_font_t *font, hb_codepoint_t unicode,
                                        hb_codepoint_t *glyph);
hb_bool_t    hb_font_get_variation_glyph (hb_font_t *font, hb_codepoint_t unicode,
                                          hb_codepoint_t variation_selector,
                                          hb_codepoint_t *glyph);
unsigned int hb_font_get_nominal_glyphs (hb_font_t *font, unsigned int count,
                                         const hb_codepoint_t *first_unicode,
                                         unsigned int unicode_stride,
                                         hb_codepoint_t *first_glyph,
                                         unsigned int glyph_stride);
```

Character-to-glyph mapping. `hb_font_get_nominal_glyph` (since 1.2.3) must **not** be used
for code points modified by a variation selector — use `hb_font_get_variation_glyph`
(since 1.2.3) or the combined `hb_font_get_glyph` for that. `hb_font_get_nominal_glyphs`
(since 2.6.3) is the bulk form; it stops at the first code point the font does not support
and returns how many it processed, so a return less than `count` means the mapping was
truncated at that index, not that it failed wholesale.

```c
hb_position_t hb_font_get_glyph_h_advance (hb_font_t *font, hb_codepoint_t glyph);
hb_position_t hb_font_get_glyph_v_advance (hb_font_t *font, hb_codepoint_t glyph);
void          hb_font_get_glyph_h_advances (hb_font_t *font, unsigned int count,
                                            const hb_codepoint_t *first_glyph, unsigned glyph_stride,
                                            hb_position_t *first_advance, unsigned advance_stride);
void          hb_font_get_glyph_v_advances (hb_font_t *font, unsigned int count,
                                            const hb_codepoint_t *first_glyph, unsigned glyph_stride,
                                            hb_position_t *first_advance, unsigned advance_stride);
```

Advances, in scaled units. The singular forms have no failure channel — an unsupported
glyph yields whatever the font functions return, typically zero or a default advance.
Singular since 0.9.2, plural since 1.8.6. The plural forms are substantially faster for
runs, because a backend can amortize table lookups across the batch.

```c
hb_bool_t hb_font_get_glyph_h_origin (hb_font_t *font, hb_codepoint_t glyph,
                                      hb_position_t *x, hb_position_t *y);
hb_bool_t hb_font_get_glyph_v_origin (hb_font_t *font, hb_codepoint_t glyph,
                                      hb_position_t *x, hb_position_t *y);
hb_bool_t hb_font_get_glyph_h_origins (hb_font_t *font, unsigned int count,
                                       const hb_codepoint_t *first_glyph, unsigned glyph_stride,
                                       hb_position_t *first_x, unsigned x_stride,
                                       hb_position_t *first_y, unsigned y_stride);
hb_bool_t hb_font_get_glyph_v_origins (hb_font_t *font, unsigned int count, /* … same … */);
```

Glyph origins in scaled units. Singular since 0.9.2; plural since 11.3.0. The horizontal
origin is usually (0, 0) for most fonts, which is why the horizontal singular form so often
returns a zero pair.

```c
hb_position_t hb_font_get_glyph_h_kerning (hb_font_t *font,
                                           hb_codepoint_t left_glyph, hb_codepoint_t right_glyph);
```

Legacy `kern`-table kerning only, as returned by the corresponding font-funcs method.
OpenType `GPOS` kerning is *not* visible here — it is applied during shaping. Since 0.9.2.

```c
hb_bool_t hb_font_get_glyph_extents (hb_font_t *font, hb_codepoint_t glyph,
                                     hb_glyph_extents_t *extents);
hb_bool_t hb_font_get_glyph_contour_point (hb_font_t *font, hb_codepoint_t glyph,
                                           unsigned int point_index,
                                           hb_position_t *x, hb_position_t *y);
```

Per-glyph extents and individual contour points. Both return true if data was found. Since
0.9.2.

```c
hb_bool_t hb_font_get_glyph_name (hb_font_t *font, hb_codepoint_t glyph,
                                  char *name, unsigned int size);
hb_bool_t hb_font_get_glyph_from_name (hb_font_t *font, const char *name, int len,
                                       hb_codepoint_t *glyph);
```

Glyph names. `hb_font_get_glyph_name` writes into a caller-provided buffer of `size`
bytes; the OpenType specification caps glyph names at 63 characters from a subset of ASCII,
so a 64-byte buffer is always enough. `hb_font_get_glyph_from_name` takes `len = -1` for a
NUL-terminated `name`. Both return true on success and false when the font has no name
data. Since 0.9.2.

### Drawing and painting

```c
hb_bool_t hb_font_draw_glyph_or_fail (hb_font_t *font, hb_codepoint_t glyph,
                                      hb_draw_funcs_t *dfuncs, void *draw_data);
hb_bool_t hb_font_paint_glyph_or_fail (hb_font_t *font, hb_codepoint_t glyph,
                                       hb_paint_funcs_t *pfuncs, void *paint_data,
                                       unsigned int palette_index, hb_color_t foreground);
void      hb_font_draw_glyph (hb_font_t *font, hb_codepoint_t glyph,
                              hb_draw_funcs_t *dfuncs, void *draw_data);
void      hb_font_paint_glyph (hb_font_t *font, hb_codepoint_t glyph,
                               hb_paint_funcs_t *pfuncs, void *paint_data,
                               unsigned int palette_index, hb_color_t foreground);
```

These do not return geometry. They *call back* into the `hb_draw_funcs_t` or
`hb_paint_funcs_t` you supply — move-to, line-to, cubic-to, close-path for drawing; a much
richer set of gradient and layer operations for painting — passing your `draw_data` or
`paint_data` to each callback.

`hb_font_draw_glyph_or_fail` (since 11.2.0) returns false when the font has no outline for
the glyph. `hb_font_draw_glyph` (since 7.0.0) is the older name for the same operation with
no return value; the header calls it "an older alias", and it is implemented as a discarded
call to the `_or_fail` form. It is not formally deprecated, so both remain exported.

`hb_font_paint_glyph_or_fail` (since 11.2.0) succeeds when the glyph has `COLRv0` paint
layers, a `COLRv1` paint graph, or a bitmap image the font's callbacks render successfully;
it returns false when there is no color data, and the caller is expected to fall back to
`hb_font_draw_glyph_or_fail`. `hb_font_paint_glyph` (since 7.0.0) performs that fallback
for you, painting the monochrome outline when color painting fails, hence its `void` return.

`palette_index` selects one of the font's `CPAL` palettes and is `0` for a font with a
single palette. `foreground` is an unpremultiplied `hb_color_t`, used wherever the color
graph refers to the "text color".

Synthetic bold and synthetic slant are applied by
`hb_font_draw_glyph_or_fail`, so shapes coming out of these functions already reflect those
settings.

### Direction-dispatching wrappers

```c
hb_bool_t hb_font_get_glyph (hb_font_t *font, hb_codepoint_t unicode,
                             hb_codepoint_t variation_selector, hb_codepoint_t *glyph);
void hb_font_get_extents_for_direction (hb_font_t *font, hb_direction_t direction,
                                        hb_font_extents_t *extents);
void hb_font_get_glyph_advance_for_direction (hb_font_t *font, hb_codepoint_t glyph,
                                              hb_direction_t direction,
                                              hb_position_t *x, hb_position_t *y);
void hb_font_get_glyph_advances_for_direction (hb_font_t *font, hb_direction_t direction,
                                               unsigned int count,
                                               const hb_codepoint_t *first_glyph, unsigned glyph_stride,
                                               hb_position_t *first_advance, unsigned advance_stride);
void hb_font_get_glyph_origin_for_direction (hb_font_t *font, hb_codepoint_t glyph,
                                             hb_direction_t direction,
                                             hb_position_t *x, hb_position_t *y);
void hb_font_add_glyph_origin_for_direction (hb_font_t *font, hb_codepoint_t glyph,
                                             hb_direction_t direction,
                                             hb_position_t *x, hb_position_t *y);
void hb_font_subtract_glyph_origin_for_direction (hb_font_t *font, hb_codepoint_t glyph,
                                                  hb_direction_t direction,
                                                  hb_position_t *x, hb_position_t *y);
void hb_font_get_glyph_kerning_for_direction (hb_font_t *font,
                                              hb_codepoint_t first_glyph, hb_codepoint_t second_glyph,
                                              hb_direction_t direction,
                                              hb_position_t *x, hb_position_t *y);
hb_bool_t hb_font_get_glyph_extents_for_origin (hb_font_t *font, hb_codepoint_t glyph,
                                                hb_direction_t direction,
                                                hb_glyph_extents_t *extents);
hb_bool_t hb_font_get_glyph_contour_point_for_origin (hb_font_t *font, hb_codepoint_t glyph,
                                                      unsigned int point_index,
                                                      hb_direction_t direction,
                                                      hb_position_t *x, hb_position_t *y);
```

Each of these inspects `direction` and calls the horizontal or vertical variant. Because a
direction covers both axes, the scalar results become an (x, y) pair: an advance for LTR
text sets `*x` and zeroes `*y`, and vice versa for TTB.

`hb_font_get_glyph` (since 0.9.2) is the convenience form of the two glyph-lookup
functions: it calls `hb_font_get_nominal_glyph` when `variation_selector` is `0` and
`hb_font_get_variation_glyph` otherwise.

`hb_font_add_glyph_origin_for_direction` and `hb_font_subtract_glyph_origin_for_direction`
(both since 0.9.2) modify `*x`/`*y` **in place**, adding or subtracting the glyph's origin.
They are the standard way to convert between origin-relative and baseline-relative
coordinates, most often used in pairs around a metric query.

Since versions: `hb_font_get_extents_for_direction` 1.1.3;
`hb_font_get_glyph_advances_for_direction` 1.8.6; the rest 0.9.2.

```c
void      hb_font_glyph_to_string (hb_font_t *font, hb_codepoint_t glyph, char *s, unsigned int size);
hb_bool_t hb_font_glyph_from_string (hb_font_t *font, const char *s, int len, hb_codepoint_t *glyph);
```

The lenient string forms, mostly for tooling and debugging. `hb_font_glyph_to_string`
always writes something: if the glyph has no name, it synthesizes `gidDDD` with `DDD` the
glyph ID. `hb_font_glyph_from_string` parses `gidDDD` and `uniUUUU` forms as well as real
glyph names, and takes `len = -1` for NUL-terminated input. Both since 0.9.2.

### Font lifecycle

```c
hb_font_t *hb_font_create (hb_face_t *face);
hb_font_t *hb_font_create_sub_font (hb_font_t *parent);
hb_font_t *hb_font_get_empty (void);
hb_font_t *hb_font_reference (hb_font_t *font);
void       hb_font_destroy (hb_font_t *font);
```

`hb_font_create` transfers one reference to the caller and takes its own reference on
`face` — the caller may destroy the face immediately afterwards and the font stays valid.
There is one subtlety worth knowing: if the face's index (as passed to `hb_face_create`)
has non-zero top 16 bits, those bits minus one are fed to
`hb_font_set_var_named_instance`, so creating a face with index `(instance+1) << 16 | idx`
gives you a font pre-loaded with that named instance of a variable font.

`hb_font_create_sub_font` takes a reference on `parent` and replicates its properties. The
resulting font's default font functions delegate to `parent`.

`hb_font_get_empty` returns the shared immutable empty font. All since 0.9.2.

```c
hb_bool_t    hb_font_set_user_data (hb_font_t *font, hb_user_data_key_t *key, void *data,
                                    hb_destroy_func_t destroy, hb_bool_t replace);
void        *hb_font_get_user_data (const hb_font_t *font, hb_user_data_key_t *key);
void         hb_font_make_immutable (hb_font_t *font);
hb_bool_t    hb_font_is_immutable (hb_font_t *font);
unsigned int hb_font_get_serial (hb_font_t *font);
void         hb_font_changed (hb_font_t *font);
```

User data and immutability behave exactly as for `hb_font_funcs_t`. `hb_font_get_serial`
(since 4.4.0) returns a counter that increases every time a setter changes the font;
comparing serials is the cheap way to notice a font has been reconfigured.
`hb_font_changed` (since 4.4.0) bumps that counter manually, telling HarfBuzz that the
*underlying* font data changed behind its back and that internal caches must be discarded.

### Properties

```c
void       hb_font_set_parent (hb_font_t *font, hb_font_t *parent);
hb_font_t *hb_font_get_parent (hb_font_t *font);
void       hb_font_set_face (hb_font_t *font, hb_face_t *face);
hb_face_t *hb_font_get_face (hb_font_t *font);
```

Both getters return **borrowed** pointers — the font keeps its reference, and the caller
must not destroy the result without first calling `hb_font_reference` /
`hb_face_reference`. `hb_font_set_parent` since 1.0.5, `hb_font_set_face` since 1.4.3, the
getters since 0.9.2.

```c
void        hb_font_set_funcs (hb_font_t *font, hb_font_funcs_t *klass,
                               void *font_data, hb_destroy_func_t destroy);
void        hb_font_set_funcs_data (hb_font_t *font, void *font_data, hb_destroy_func_t destroy);
hb_bool_t   hb_font_set_funcs_using (hb_font_t *font, const char *name);
const char **hb_font_list_funcs (void);
```

`hb_font_set_funcs` (since 0.9.2) attaches a font-functions structure *and* the
`font_data` pointer every callback in it will receive, plus the `destroy` callback that
releases `font_data`. If `font` is immutable the call does nothing — except that HarfBuzz
still invokes `destroy(font_data)` right away, so ownership is not leaked. Passing a null
`klass` is not addressed by the header.

`hb_font_set_funcs_data` (since 0.9.2) swaps only the data pointer. The header's comment is
"Be *very* careful with this function!", and the reason is that the already-installed
callbacks will now be handed a pointer of possibly the wrong type.

`hb_font_set_funcs_using` (since 11.0.0) selects a built-in implementation by name,
returning true if it was found. A null or empty `name` selects the default (first working)
implementation, which can itself be overridden with the `HB_FONT_FUNCS` environment
variable. `hb_font_list_funcs` (since 11.0.0) returns a null-terminated array of
NUL-terminated strings naming the available implementations; the array is owned by HarfBuzz
and must not be modified or freed.

```c
void hb_font_set_scale (hb_font_t *font, int x_scale, int y_scale);
void hb_font_get_scale (hb_font_t *font, int *x_scale, int *y_scale);
void hb_font_set_ppem (hb_font_t *font, unsigned int x_ppem, unsigned int y_ppem);
void hb_font_get_ppem (hb_font_t *font, unsigned int *x_ppem, unsigned int *y_ppem);
void  hb_font_set_ptem (hb_font_t *font, float ptem);
float hb_font_get_ptem (hb_font_t *font);
```

**Scale** is the one you almost always have to set, and the one most often set wrongly.
`hb_position_t` is a 32-bit integer, so fractional precision has to come from the scale
factor itself: pick a fixed-point denominator (64 and 256 are conventional) and multiply.
For size 20 with 1/64 precision, call `hb_font_set_scale(font, 20 * 64, 20 * 64)`, then
divide every `hb_position_t` you get back by 64. What "size 20" means — pixels, points,
millimetres — is entirely the client's business; HarfBuzz never interprets it. The default
scale equals the face's units-per-em, giving an "unscaled" font whose positions are in font
units. Since 0.9.2.

**PPEM** drives pixel-size-specific adjustments to shaping and drawing. Mostly unused; a
zero value means "no hinting in that direction". Since 0.9.2.

**PTEM** is the point size per em, used by CoreText for optical sizing. Zero means "not
set", which is the default. There are 72 points in an inch. Since 1.6.0.

```c
hb_bool_t hb_font_is_synthetic (hb_font_t *font);
void  hb_font_set_synthetic_bold (hb_font_t *font, float x_embolden, float y_embolden, hb_bool_t in_place);
void  hb_font_get_synthetic_bold (hb_font_t *font, float *x_embolden, float *y_embolden, hb_bool_t *in_place);
void  hb_font_set_synthetic_slant (hb_font_t *font, float slant);
float hb_font_get_synthetic_slant (hb_font_t *font);
```

`hb_font_is_synthetic` (since 11.2.0) reports whether either synthetic setting is non-zero.

Synthetic **bold** (since 7.0.0) offsets the contour points of the glyph shape. Positive
values embolden, negative values thin; typical values are 0.01 to 0.05, and the default is
zero. When `in_place` is false, advance widths are widened to match; when true they are
left alone, which simulates a variable-font *grade* axis rather than a weight change.

Synthetic **slant** (since 3.3.0) is a graphical skew applied at rendering time, expressed
as a ratio — a 20% slant is `0.2`. HarfBuzz needs to be told about it so that shaping
results, metrics, and style values match the slanted rendering, and shapes returned by
`hb_font_draw_glyph_or_fail` are slanted to match.

### Variations

```c
void hb_font_set_variations (hb_font_t *font, const hb_variation_t *variations,
                             unsigned int variations_length);
void hb_font_set_variation (hb_font_t *font, hb_tag_t tag, float value);
void hb_font_set_var_coords_design (hb_font_t *font, const float *coords, unsigned int coords_length);
const float *hb_font_get_var_coords_design (hb_font_t *font, unsigned int *length);
void hb_font_set_var_coords_normalized (hb_font_t *font, const int *coords, unsigned int coords_length);
const int   *hb_font_get_var_coords_normalized (hb_font_t *font, unsigned int *length);
void         hb_font_set_var_named_instance (hb_font_t *font, unsigned int instance_index);
unsigned int hb_font_get_var_named_instance (hb_font_t *font);
```

Three ways to say the same thing, in decreasing friendliness: tagged
`hb_variation_t` values, design-space coordinates (the axis's own units, e.g. `wght = 700`),
and normalized coordinates (2.14 fixed point, i.e. `-1.0` to `1.0` scaled by 16384).

All three setters are **absolute, not incremental**: each one overrides every existing
variation, and any axis not covered by the call reverts to its default value. That is the
single most common variable-font bug in HarfBuzz code — calling `hb_font_set_variations`
twice with one axis each leaves only the second axis set. `hb_font_set_variation` (since
7.1.0) is the exception in spirit: it changes one axis and preserves the rest. The header
warns that it is expensive to call repeatedly, so batch through `hb_font_set_variations`
(since 1.4.2) when setting several axes.

`hb_font_set_var_coords_design` since 1.4.2, `hb_font_set_var_coords_normalized` since
1.4.2. Both copy their input; the caller's array can be freed immediately.

The two getters write the coordinate count into `*length` and return a **borrowed** array —
do not free it, and treat it as invalidated by the next call that modifies the font's
variations. Both may return null when no coordinates are set. If variations were set
through the *normalized* setter, `hb_font_get_var_coords_design` returns NaN values, since
the design coordinates were never computed. `hb_font_get_var_coords_design` since 3.3.0,
`hb_font_get_var_coords_normalized` since 1.4.2.

`hb_font_set_var_named_instance` (since 2.6.0) sets design coordinates from an `fvar` named
instance index; `hb_font_get_var_named_instance` (since 7.0.0) returns the current index or
`HB_FONT_NO_VAR_NAMED_INSTANCE`.

## Usage notes

### Immutability swallows errors

`hb_font_make_immutable` and `hb_font_funcs_make_immutable` are one-way. Afterwards, every
setter in this header returns `void` *and does nothing*. Nothing warns you. HarfBuzz itself
makes fonts immutable in some paths, so a font handed to you by a library may already be
frozen. When a `hb_font_set_scale` mysteriously has no effect, check
`hb_font_is_immutable` first.

The one place immutability still does something visible is `hb_font_set_funcs` and
`hb_font_set_funcs_data`: on an immutable font they invoke `destroy(font_data)` immediately
so the caller's data is not leaked. A `destroy` callback firing "too early" is therefore a
signal that the font was frozen.

### The two data pointers

A callback receives `font_data` (attached once per font via `hb_font_set_funcs`) *and*
`user_data` (attached per method via the individual setter). They are different pointers
with different lifetimes and different `destroy` callbacks. Mixing them up compiles fine —
both are `void *` — and crashes at run time.

### Scale before shaping

```c
hb_face_t *face = hb_face_create (blob, 0);
hb_font_t *font = hb_font_create (face);
hb_face_destroy (face);                       /* font holds its own reference */

hb_font_set_scale (font, 16 * 64, 16 * 64);   /* 16pt, 1/64 units */
hb_font_set_ptem  (font, 16.0f);              /* optical sizing, CoreText */

hb_shape (font, buffer, NULL, 0);
/* every hb_position_t in buffer is now in 1/64ths */

hb_font_destroy (font);
```

Forgetting `hb_font_set_scale` is not an error: the font keeps its default scale of one
unit per font-design unit, and you get positions in font units (typically 1000 or 2048 per
em). That is a legitimate mode — the "unscaled" font — but it surprises people who expected
pixels.

### Sub-fonts are the overriding mechanism

To adjust just one metric without reimplementing a font backend:

```c
hb_font_t *sub = hb_font_create_sub_font (parent);
hb_font_funcs_t *ffuncs = hb_font_funcs_create ();
hb_font_funcs_set_glyph_h_advance_func (ffuncs, my_advance, my_data, my_destroy);
hb_font_set_funcs (sub, ffuncs, my_font_data, my_font_data_destroy);
hb_font_funcs_destroy (ffuncs);   /* `sub` holds its own reference now */
```

Every method other than `glyph_h_advance` falls through to the default implementation,
which asks `parent`. Note the `hb_font_funcs_destroy` immediately after
`hb_font_set_funcs`: the font took a reference, so the creator's reference should be
released.

### Horizontal and vertical callbacks are the same type

`hb_font_funcs_set_glyph_h_advance_func` and `hb_font_funcs_set_glyph_v_advance_func` take
the same C type, and therefore the same Rust type alias. Nothing prevents installing a
horizontal implementation as the vertical one. The same applies to the extents, origin, and
plural variants. Name your callbacks distinctly.

### Recursion through the parent chain

Because the default methods call the parent font's methods, a font whose parent is itself
(or a cycle via `hb_font_set_parent`) will recurse. HarfBuzz has internal guards, but the
header says nothing about it — do not build parent cycles.

### Threading

The header specifies no threading rules. HarfBuzz's general convention is that immutable
objects are safe to share between threads and mutable ones are not, and that reference
counts are atomic in a thread-safe build. Concurrent mutation of one `hb_font_t` is
unsafe. The `hb_font_t` and `hb_font_funcs_t` handles in this crate are `!Send` and
`!Sync`, so a safe wrapper must opt back in explicitly and document why.

### `_or_fail` versus the older names

Prefer `hb_font_draw_glyph_or_fail` and `hb_font_paint_glyph_or_fail` in new code: they
tell you whether the font actually had data. `hb_font_draw_glyph` silently draws nothing
for a glyph with no outline, and `hb_font_paint_glyph` silently substitutes the monochrome
outline for a glyph with no color data. Neither is deprecated, and both remain exported
symbols, but the failure signal is usually worth having.

### Unspecified behaviour

The header does not state the following, so a safe wrapper should not assume it:

- Whether any function accepts null for its `font` or `ffuncs` argument. (HarfBuzz's
  general null-object convention suggests it tolerates them, but it is not documented
  here.)
- Whether out-parameters are written on failure. Read them only after a true return.
- What `hb_font_create` returns when allocation fails.
- Whether `hb_font_set_funcs` accepts a null `klass`.
- Whether `hb_font_get_glyph_name` NUL-terminates its output when the name is exactly
  `size` bytes long.
