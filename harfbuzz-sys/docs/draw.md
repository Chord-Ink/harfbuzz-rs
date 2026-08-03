# Drawing glyph outlines

Source header: `hb-draw.h`. Rust module: `harfbuzz_sys::draw` (glob re-exported at the crate root).

## Overview

Shaping tells you *which* glyphs to draw and *where*. This header tells you what
those glyphs actually look like. It defines the callback interface HarfBuzz uses
to hand a glyph's outline to the caller, one path operation at a time.

The central object is `hb_draw_funcs_t` — a **pen**. It is a reference-counted
bag of five function pointers: move-to, line-to, quadratic-to, cubic-to, and
close-path. You create one, install your callbacks on it, and pass it (together
with an opaque `void *draw_data` of your own) to a function that produces
outlines — most commonly `hb_font_draw_glyph()` / `hb_font_draw_glyph_or_fail()`
from `hb-font.h`. HarfBuzz then walks the glyph's contours in font units, scaled
and variation-instanced according to the font, and invokes your callbacks in
order. There is no output buffer and no intermediate path object: the outline
exists only as the sequence of calls you receive.

Alongside the pen there is a small piece of shared state, `hb_draw_state_t`.
HarfBuzz owns it, updates it as it goes, and passes a pointer to it into every
callback. Your callback reads it to learn where the pen currently sits
(`current_x`, `current_y`), where the contour it is drawing began
(`path_start_x`, `path_start_y`), and whether a contour is open at all
(`path_open`). This matters because HarfBuzz's path bookkeeping is deliberately
*normalising*: a "move-to" does not immediately reach your callback — the state
is updated and the move-to is emitted lazily when the first real segment of the
contour arrives. Likewise, closing a contour whose current point has drifted
from its start point emits an implicit line-to back to the start before the
close-path callback fires. The result is that a pen only ever sees well-formed
contours: exactly one move-to, some segments, an explicitly closed loop.

Two callbacks are effectively optional in different ways. Any callback you leave
unset falls back to a no-op, so a pen that only implements move-to and line-to
will silently drop curves — that is a bug in your pen, not a feature. The one
genuine exception is **quadratic-to**: its default is not a no-op but a
converter, which elevates the quadratic to a cubic and re-emits it through your
cubic-to callback. This is the reason move-to, line-to, and cubic-to are the
three you must implement: with those three you can consume any font, TrueType
(quadratic) or CFF (cubic) alike. Implement quadratic-to only if your back end
handles quadratics natively and you want to avoid the conversion.

The header also carries three **shape helpers** — `hb_draw_line()`,
`hb_draw_rectangle()`, and `hb_draw_circle()` — added in HarfBuzz 14.2.0. These
have nothing to do with fonts. They are thin composers over the five primitive
operations that emit common geometry into any pen, so that code which already
has a pen (a rasteriser, a GPU tessellator, an SVG writer) can draw decorations
— underlines, boxes, dots — through the exact same path without a second
graphics API. The header's own comment is explicit that callers may always
hand-roll the same shapes if they need a variation.

Finally, note that `hb_draw_funcs_t` is not only consumed here. `hb-paint.h`
hands a pen to your clip-path callback; and the optional `hb-raster.h`,
`hb-vector.h`, and `hb-gpu.h` sub-libraries each *provide* a ready-made
`hb_draw_funcs_t` so you can rasterise, serialise, or upload outlines without
writing callbacks at all.

## Types

### `hb_draw_funcs_t`

Opaque, reference-counted glyph draw callbacks — the pen. Created with
`hb_draw_funcs_create()`, shared with `hb_draw_funcs_reference()`, released with
`hb_draw_funcs_destroy()`. Callbacks are installed one at a time with the
`hb_draw_funcs_set_*_func()` family; once you are done configuring it you can
freeze it with `hb_draw_funcs_make_immutable()`.

In Rust it is an `opaque_handle!` type — zero-sized, non-constructible, used
only behind `*mut hb_draw_funcs_t`.

Since HarfBuzz 4.0.0.

### `hb_draw_state_t`

The current drawing state, passed by pointer into every callback. Public
`#[repr(C)]` struct.

| Field | C type | Rust type | Meaning |
| --- | --- | --- | --- |
| `path_open` | `hb_bool_t` | `hb_bool_t` (`c_int`) | Whether there is an open path. |
| `path_start_x` | `float` | `c_float` | X component of the start of the current path. |
| `path_start_y` | `float` | `c_float` | Y component of the start of the current path. |
| `current_x` | `float` | `c_float` | X component of the current point. |
| `current_y` | `float` | `c_float` | Y component of the current point. |
| `reserved1`…`reserved7` | `hb_var_num_t` | `hb_var_num_t` | Private padding, marked `/*< private >*/` in the header. Do not read or write. |

The seven reserved slots exist to keep `sizeof(hb_draw_state_t)` stable if
HarfBuzz ever needs to track more state, since the struct is allocated by the
caller (or by HarfBuzz's own draw session) and passed across the ABI boundary.

`hb_var_num_t` is a union, so it has no `Debug`. The Rust struct therefore
derives only `Clone` and `Copy` and carries a hand-written `Debug` that prints
the five public fields and elides the padding. No `PartialEq`/`Eq`/`Hash`: the
struct contains floats and unions.

Since HarfBuzz 4.0.0.

### `hb_draw_line_cap_t`

End-cap shape for `hb_draw_line()`.

```c
typedef enum {
  HB_DRAW_LINE_CAP_BUTT   = 0,
  HB_DRAW_LINE_CAP_SQUARE = 1,
} hb_draw_line_cap_t;
```

```rust
pub type hb_draw_line_cap_t = c_int;
pub const HB_DRAW_LINE_CAP_BUTT: hb_draw_line_cap_t = 0;
pub const HB_DRAW_LINE_CAP_SQUARE: hb_draw_line_cap_t = 1;
```

| Constant | Value | Meaning |
| --- | --- | --- |
| `HB_DRAW_LINE_CAP_BUTT` | 0 | No cap; the line ends exactly at its endpoint. |
| `HB_DRAW_LINE_CAP_SQUARE` | 1 | Square cap; each endpoint is extended along the line direction by half the local stroke width. |

Underlying type: the C enumeration has no `_MAX_VALUE` sentinel and its largest
enumerator is 1, so it fits in `int` — hence `c_int`.

Since HarfBuzz 14.2.0.

### Callback typedefs

All five are wrapped in `Option` on the Rust side so that `None` is the null
pointer, which is what the setters accept to mean "restore the default". Every
one receives the pen itself (`dfuncs`), your per-draw `draw_data`, the mutable
`st`, the operation's coordinates, and the per-callback `user_data` that was
registered with the setter.

| Typedef | Extra parameters beyond `dfuncs`, `draw_data`, `st`, `user_data` |
| --- | --- |
| `hb_draw_move_to_func_t` | `to_x`, `to_y` |
| `hb_draw_line_to_func_t` | `to_x`, `to_y` |
| `hb_draw_quadratic_to_func_t` | `control_x`, `control_y`, `to_x`, `to_y` |
| `hb_draw_cubic_to_func_t` | `control1_x`, `control1_y`, `control2_x`, `control2_y`, `to_x`, `to_y` |
| `hb_draw_close_path_func_t` | *(none)* |

For example:

```c
typedef void (*hb_draw_cubic_to_func_t) (hb_draw_funcs_t *dfuncs, void *draw_data,
                                         hb_draw_state_t *st,
                                         float control1_x, float control1_y,
                                         float control2_x, float control2_y,
                                         float to_x, float to_y,
                                         void *user_data);
```

```rust
pub type hb_draw_cubic_to_func_t = Option<
    unsafe extern "C" fn(
        dfuncs: *mut hb_draw_funcs_t,
        draw_data: *mut c_void,
        st: *mut hb_draw_state_t,
        control1_x: c_float,
        control1_y: c_float,
        control2_x: c_float,
        control2_y: c_float,
        to_x: c_float,
        to_y: c_float,
        user_data: *mut c_void,
    ),
>;
```

All return `void`: a pen cannot report failure back to HarfBuzz. If your pen
can fail (allocation, I/O), record the failure in your `draw_data` and check it
after the drawing call returns.

Since HarfBuzz 4.0.0 for all five.

### Constants

#### `HB_DRAW_STATE_DEFAULT`

```c
#define HB_DRAW_STATE_DEFAULT {0, 0.f, 0.f, 0.f, 0.f, {0}, {0}, {0}, {0}, {0}, {0}, {0}}
```

```rust
pub const HB_DRAW_STATE_DEFAULT: hb_draw_state_t = hb_draw_state_t {
    path_open: 0,
    path_start_x: 0.0,
    /* … all zero … */
};
```

The default `hb_draw_state_t` at the start of glyph drawing: no open path, pen
at the origin, padding zeroed. The C form is a brace initialiser, so it is
transcribed as a `pub const` value rather than a macro. Use it to initialise any
`hb_draw_state_t` you allocate yourself before calling the `hb_draw_*`
primitives directly.

## Functions

### Object lifecycle

#### `hb_draw_funcs_create`

```c
hb_draw_funcs_t * hb_draw_funcs_create (void);
```

```rust
pub fn hb_draw_funcs_create() -> *mut hb_draw_funcs_t;
```

Creates a new draw-functions object with a reference count of one, with all five
callbacks set to their defaults.

**Never returns null.** If allocation fails, HarfBuzz returns the singleton empty
object instead — the same pointer `hb_draw_funcs_get_empty()` returns. That
object is permanently immutable, so a subsequent `hb_draw_funcs_set_*_func()`
call will silently do nothing. If you need to distinguish out-of-memory from
success, compare the result against `hb_draw_funcs_get_empty()`, or check
`hb_draw_funcs_is_immutable()` right after creation.

Ownership: the caller owns the returned reference and must release it with
`hb_draw_funcs_destroy()`.

Since HarfBuzz 4.0.0.

#### `hb_draw_funcs_get_empty`

```c
hb_draw_funcs_t * hb_draw_funcs_get_empty (void);
```

```rust
pub fn hb_draw_funcs_get_empty() -> *mut hb_draw_funcs_t;
```

Fetches the singleton empty draw-functions object: every callback is the default
(no-ops, plus the quadratic-to converter), and it is permanently immutable.
Never null. It is a static object with a static reference count, so calling
`hb_draw_funcs_destroy()` on it is harmless and calling `hb_draw_funcs_reference()`
on it is unnecessary — but doing either is consistent and safe, which is why the
gtk-doc annotation calls it `(transfer full)`.

Useful as a "draw nothing" pen when you want to run the drawing machinery for its
side effects only.

Since HarfBuzz 7.0.0.

#### `hb_draw_funcs_reference`

```c
hb_draw_funcs_t * hb_draw_funcs_reference (hb_draw_funcs_t *dfuncs);
```

```rust
pub fn hb_draw_funcs_reference(dfuncs: *mut hb_draw_funcs_t) -> *mut hb_draw_funcs_t;
```

Increases the reference count by one and returns the same pointer, so it chains.
Prevents the object from being destroyed until a matching
`hb_draw_funcs_destroy()`. Never null for a non-null argument.

Since HarfBuzz 4.0.0.

#### `hb_draw_funcs_destroy`

```c
void hb_draw_funcs_destroy (hb_draw_funcs_t *dfuncs);
```

```rust
pub fn hb_draw_funcs_destroy(dfuncs: *mut hb_draw_funcs_t);
```

Decreases the reference count by one. At zero, the object and all associated
resources are freed — and this is where the `destroy` callbacks registered
alongside each drawing callback finally run, each with the `user_data` it was
paired with. Any user-data attached with `hb_draw_funcs_set_user_data()` is
released here too.

Since HarfBuzz 4.0.0.

### Installing callbacks

All five setters share one shape:

```c
void hb_draw_funcs_set_XXX_func (hb_draw_funcs_t     *dfuncs,
                                 hb_draw_XXX_func_t   func,
                                 void                *user_data,
                                 hb_destroy_func_t    destroy);
```

```rust
pub fn hb_draw_funcs_set_XXX_func(
    dfuncs: *mut hb_draw_funcs_t,
    func: hb_draw_XXX_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

where `XXX` is one of `move_to`, `line_to`, `quadratic_to`, `cubic_to`,
`close_path`. All are Since HarfBuzz 4.0.0, and all return `void` — they cannot
report failure.

Semantics common to all five, taken from the header and the implementation:

* **`user_data` is per-callback**, not per-object. Each of the five slots has its
  own `user_data` and its own `destroy`. This is separate from, and unrelated to,
  the key/value store reached through `hb_draw_funcs_set_user_data()`.
* **`destroy` may be null**, and is the only way HarfBuzz will ever release
  `user_data`. It is called when the callback is replaced, when the object is
  destroyed, or immediately in the two "nothing to install" cases below.
* **A null `func` resets the slot to its default.** For move-to, line-to,
  cubic-to and close-path the default is a no-op; for quadratic-to the default is
  the quadratic-to-cubic converter. When `func` is null, any `user_data` and
  `destroy` you passed are consumed right away — `destroy(user_data)` is called
  before the function returns — rather than being stored.
* **On an immutable object the call is a silent no-op**, but `destroy(user_data)`
  is still called, so nothing leaks. There is no return value to tell you the set
  did not take effect; check `hb_draw_funcs_is_immutable()` first if it matters.
* **Replacing an existing callback** runs the previous slot's `destroy` on the
  previous slot's `user_data` before installing the new pair.
* Under memory pressure the internal `user_data`/`destroy` side tables may fail
  to allocate, in which case the setter aborts and calls `destroy(user_data)` on
  the *incoming* pair. Again, silently — a failed set is indistinguishable from a
  successful one at the API level.

Ownership summary: `func` is a plain function pointer and is copied. `user_data`
is stored as-is and never copied or interpreted; HarfBuzz will keep it alive for
as long as the slot holds it and will hand it back to `destroy` exactly once.

### Attaching user data

#### `hb_draw_funcs_set_user_data`

```c
hb_bool_t hb_draw_funcs_set_user_data (hb_draw_funcs_t    *dfuncs,
                                       hb_user_data_key_t *key,
                                       void               *data,
                                       hb_destroy_func_t   destroy,
                                       hb_bool_t           replace);
```

```rust
pub fn hb_draw_funcs_set_user_data(
    dfuncs: *mut hb_draw_funcs_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

The standard HarfBuzz key/value store, keyed by the *address* of an
`hb_user_data_key_t` (its contents are irrelevant, so a `static` of that type is
the idiom). `destroy` — which may be null — is called with `data` when the object
is destroyed or the value is replaced. `replace` decides whether an existing
entry under the same key is overwritten.

Returns true on success, false otherwise (allocation failure, or an existing
entry with `replace` false).

Distinct from the per-callback `user_data` described above; use this when several
independent pieces of code need to hang state off the same pen.

Since HarfBuzz 7.0.0.

#### `hb_draw_funcs_get_user_data`

```c
void * hb_draw_funcs_get_user_data (const hb_draw_funcs_t *dfuncs,
                                    hb_user_data_key_t    *key);
```

```rust
pub fn hb_draw_funcs_get_user_data(
    dfuncs: *const hb_draw_funcs_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

Fetches the value stored under `key`, or null if there is none. Ownership stays
with the pen — do not free the returned pointer, and do not use it after the pen
is destroyed. Note the `const` on `dfuncs` here, which the setter does not have.

Since HarfBuzz 7.0.0.

### Immutability

#### `hb_draw_funcs_make_immutable`

```c
void hb_draw_funcs_make_immutable (hb_draw_funcs_t *dfuncs);
```

```rust
pub fn hb_draw_funcs_make_immutable(dfuncs: *mut hb_draw_funcs_t);
```

Freezes the object. After this, every `hb_draw_funcs_set_*_func()` call is a
no-op (still honouring `destroy`, as described above). Idempotent. There is no
way to un-freeze.

The point is publication safety: once a pen is immutable, sharing it across
threads or handing it to library code cannot lead to a torn read of its callback
table.

Since HarfBuzz 4.0.0.

#### `hb_draw_funcs_is_immutable`

```c
hb_bool_t hb_draw_funcs_is_immutable (hb_draw_funcs_t *dfuncs);
```

```rust
pub fn hb_draw_funcs_is_immutable(dfuncs: *mut hb_draw_funcs_t) -> hb_bool_t;
```

Returns true if the object is immutable. Note the parameter is `hb_draw_funcs_t *`,
not `const hb_draw_funcs_t *`, matching the rest of HarfBuzz's `is_immutable`
family.

Since HarfBuzz 4.0.0.

### Driving a pen directly

These five are the operations HarfBuzz itself performs while walking a glyph.
They are public so that you can drive a pen yourself — to re-emit a stored path,
to synthesise geometry, or to build the shape helpers below. Each takes the pen,
your `draw_data`, and a pointer to an `hb_draw_state_t` **you** own and must
initialise from `HB_DRAW_STATE_DEFAULT`.

All are Since HarfBuzz 4.0.0 and return `void`.

#### `hb_draw_move_to`

```c
void hb_draw_move_to (hb_draw_funcs_t *dfuncs, void *draw_data,
                      hb_draw_state_t *st,
                      float to_x, float to_y);
```

```rust
pub fn hb_draw_move_to(
    dfuncs: *mut hb_draw_funcs_t,
    draw_data: *mut c_void,
    st: *mut hb_draw_state_t,
    to_x: c_float,
    to_y: c_float,
);
```

Starts a new contour. If a path is currently open it is closed first (with the
implicit line-to described under `hb_draw_close_path`). Then `to_x`/`to_y` become
the current point.

**Your move-to callback does not run yet.** It runs lazily, when the first
segment of the new contour is drawn. A move-to that is never followed by a
line-to, curve-to, or close therefore produces no callbacks at all — which is the
right behaviour, since an isolated point is not a contour.

#### `hb_draw_line_to`

```c
void hb_draw_line_to (hb_draw_funcs_t *dfuncs, void *draw_data,
                      hb_draw_state_t *st,
                      float to_x, float to_y);
```

```rust
pub fn hb_draw_line_to(
    dfuncs: *mut hb_draw_funcs_t,
    draw_data: *mut c_void,
    st: *mut hb_draw_state_t,
    to_x: c_float,
    to_y: c_float,
);
```

Opens the path if it is not open (emitting the deferred move-to at the current
point), invokes the line-to callback, and advances the current point.

#### `hb_draw_quadratic_to`

```c
void hb_draw_quadratic_to (hb_draw_funcs_t *dfuncs, void *draw_data,
                           hb_draw_state_t *st,
                           float control_x, float control_y,
                           float to_x, float to_y);
```

```rust
pub fn hb_draw_quadratic_to(
    dfuncs: *mut hb_draw_funcs_t,
    draw_data: *mut c_void,
    st: *mut hb_draw_state_t,
    control_x: c_float,
    control_y: c_float,
    to_x: c_float,
    to_y: c_float,
);
```

Opens the path if needed, then draws a quadratic Bézier from the current point
through the control point to the target. If no quadratic-to callback is
installed, HarfBuzz converts the curve to a cubic — control points at one third
and two thirds along, computed in `double` so the result does not depend on how
the compiler evaluates `float` expressions — and calls the cubic-to callback
instead.

#### `hb_draw_cubic_to`

```c
void hb_draw_cubic_to (hb_draw_funcs_t *dfuncs, void *draw_data,
                       hb_draw_state_t *st,
                       float control1_x, float control1_y,
                       float control2_x, float control2_y,
                       float to_x, float to_y);
```

```rust
pub fn hb_draw_cubic_to(
    dfuncs: *mut hb_draw_funcs_t,
    draw_data: *mut c_void,
    st: *mut hb_draw_state_t,
    control1_x: c_float,
    control1_y: c_float,
    control2_x: c_float,
    control2_y: c_float,
    to_x: c_float,
    to_y: c_float,
);
```

Opens the path if needed, then draws a cubic Bézier from the current point
through the two control points to the target.

#### `hb_draw_close_path`

```c
void hb_draw_close_path (hb_draw_funcs_t *dfuncs, void *draw_data,
                         hb_draw_state_t *st);
```

```rust
pub fn hb_draw_close_path(
    dfuncs: *mut hb_draw_funcs_t,
    draw_data: *mut c_void,
    st: *mut hb_draw_state_t,
);
```

Closes the open contour. If the current point differs from the contour's start
point, an implicit line-to back to the start is emitted first; then the
close-path callback runs. Afterwards `path_open` is cleared and all four
coordinate fields are reset to zero.

Calling it with no path open is harmless: the state is simply reset.

### Shape helpers

Convenience composers over the five primitives. They emit ordinary path
operations into whatever pen you give them — nothing font-specific happens — so
a pen cannot tell whether a contour came from a glyph or from one of these. All
are Since HarfBuzz 14.2.0 and return `void`.

For `hb_draw_rectangle()` and `hb_draw_circle()` the `stroke_width` parameter
selects between filled and stroked: a **positive finite** value is the width of
the stroked outline, and **NaN** means filled. Zero, negative, and infinite
values draw nothing at all.

#### `hb_draw_line`

```c
void hb_draw_line (hb_draw_funcs_t *dfuncs, void *draw_data,
                   hb_draw_state_t *st,
                   float x0, float y0, float w0,
                   float x1, float y1, float w1,
                   hb_draw_line_cap_t cap);
```

```rust
pub fn hb_draw_line(
    dfuncs: *mut hb_draw_funcs_t,
    draw_data: *mut c_void,
    st: *mut hb_draw_state_t,
    x0: c_float,
    y0: c_float,
    w0: c_float,
    x1: c_float,
    y1: c_float,
    w1: c_float,
    cap: hb_draw_line_cap_t,
);
```

Emits a tapered line segment as a filled trapezoid: one move-to, three line-tos,
one close-path, wound counter-clockwise.

`w0` and `w1` are the **full** stroke widths at the start and end points. They
may differ, giving a taper, or match, giving a uniform stroke. Passing NaN for
`w1` reuses `w0`, so you need not repeat the value for the common uniform case.

With `HB_DRAW_LINE_CAP_SQUARE` each endpoint is pushed outward along the line
direction by half its local stroke width, which is what lets four `hb_draw_line()`
calls form a closed rectangle with no notches at the corners.
`HB_DRAW_LINE_CAP_BUTT` leaves the endpoints where you put them.

A zero-length segment (`x0 == x1 && y0 == y1`) draws nothing — the direction is
undefined, so there is no trapezoid to emit.

#### `hb_draw_rectangle`

```c
void hb_draw_rectangle (hb_draw_funcs_t *dfuncs, void *draw_data,
                        hb_draw_state_t *st,
                        float x, float y,
                        float w, float h,
                        float stroke_width);
```

```rust
pub fn hb_draw_rectangle(
    dfuncs: *mut hb_draw_funcs_t,
    draw_data: *mut c_void,
    st: *mut hb_draw_state_t,
    x: c_float,
    y: c_float,
    w: c_float,
    h: c_float,
    stroke_width: c_float,
);
```

Emits an axis-aligned rectangle whose corner is `x`/`y`. `w` and `h` may be
negative — the rectangle simply extends the other way, and the implementation
normalises before doing stroke arithmetic.

* **Filled** (`stroke_width` is NaN): one closed counter-clockwise contour. A
  zero `w` or `h` draws nothing, since the area is empty.
* **Stroked** (`stroke_width` finite and positive): two contours. An outer
  rectangle grown by `stroke_width/2` on each side, wound counter-clockwise, and
  — when the inner rectangle still has positive extent — an inner rectangle
  shrunk by the same amount, wound *clockwise* so that a non-zero or even-odd
  fill rule cuts the hole out. The stroke is centred on the nominal edges. A
  zero `w` or `h` is still meaningful when stroking: the hole collapses and you
  get a single filled bar.

Performance note carried from upstream: a stroked rectangle's bounding box covers
the full outer rectangle, so a fragment-shader-based pen runs the shader for every
interior pixel even though only the outline contributes coverage. For thin
outlines around a large interior, four `hb_draw_line()` calls with square caps
are considerably cheaper.

#### `hb_draw_circle`

```c
void hb_draw_circle (hb_draw_funcs_t *dfuncs, void *draw_data,
                     hb_draw_state_t *st,
                     float cx, float cy,
                     float r,
                     float stroke_width);
```

```rust
pub fn hb_draw_circle(
    dfuncs: *mut hb_draw_funcs_t,
    draw_data: *mut c_void,
    st: *mut hb_draw_state_t,
    cx: c_float,
    cy: c_float,
    r: c_float,
    stroke_width: c_float,
);
```

Emits a circle centred on `cx`/`cy`, approximated by four cubic Béziers — one per
quadrant, using the standard `(4/3)(√2 − 1) ≈ 0.5522847` control-point offset,
which holds the maximum radial error to about 2.7 × 10⁻⁴ of `r`.

* `r <= 0` draws nothing.
* **Filled** (`stroke_width` is NaN): one closed counter-clockwise contour of
  radius `r`.
* **Stroked** (`stroke_width` finite and positive): an outer contour at
  `r + stroke_width/2` counter-clockwise, plus — when `r - stroke_width/2` is
  still positive — an inner contour clockwise, cutting the hole. A stroke wider
  than the diameter therefore degenerates to a filled disc of the outer radius.

## Usage notes

### Writing a pen

The minimum viable pen implements move-to, line-to, cubic-to, and close-path, and
lets HarfBuzz convert quadratics for it:

```c
typedef struct { char *buf; size_t len, cap; } svg_t;

static void
svg_move_to (hb_draw_funcs_t *dfuncs, void *draw_data,
             hb_draw_state_t *st,
             float to_x, float to_y,
             void *user_data)
{
  svg_t *svg = (svg_t *) draw_data;
  appendf (svg, "M%g,%g", (double) to_x, (double) to_y);
}
/* ... line_to, cubic_to, close_path likewise ... */

hb_draw_funcs_t *dfuncs = hb_draw_funcs_create ();
hb_draw_funcs_set_move_to_func    (dfuncs, svg_move_to,    NULL, NULL);
hb_draw_funcs_set_line_to_func    (dfuncs, svg_line_to,    NULL, NULL);
hb_draw_funcs_set_cubic_to_func   (dfuncs, svg_cubic_to,   NULL, NULL);
hb_draw_funcs_set_close_path_func (dfuncs, svg_close_path, NULL, NULL);
hb_draw_funcs_make_immutable (dfuncs);

svg_t svg = {0};
hb_font_draw_glyph (font, glyph, dfuncs, &svg);

hb_draw_funcs_destroy (dfuncs);
```

Note the two different opaque pointers. `draw_data` is per-*drawing-call* and is
whatever you pass to `hb_font_draw_glyph()`; `user_data` is per-*callback* and is
whatever you passed to the corresponding setter. Most pens use `draw_data` for
the output sink and leave `user_data` null.

### Do not write to `hb_draw_state_t`

The state belongs to HarfBuzz. Your callbacks receive `hb_draw_state_t *`,
non-const, because the C API has no better way to pass a mutable-by-the-owner
struct through — not as an invitation to modify it. Mutating `current_x` or
`path_open` from a callback will desynchronise HarfBuzz's path bookkeeping and
produce malformed contours. Read it, do not write it.

When you drive a pen yourself with the `hb_draw_*` primitives, you own the state
object: allocate it, initialise it from `HB_DRAW_STATE_DEFAULT`, pass the same
pointer to every call in the sequence, and call `hb_draw_close_path()` at the end
so the last contour is not left dangling.

### Coordinates

Coordinates arriving from `hb_font_draw_glyph()` are `float`, already scaled by
the font's scale and instanced for its variation coordinates. They are *not*
`hb_position_t` — no 26.6 fixed point here. The Y axis grows upward, as in font
design space, so most rendering back ends need a flip.

The shape helpers make no assumption about the coordinate system: they are pure
arithmetic on whatever numbers you hand them, so "counter-clockwise" above means
counter-clockwise in a Y-up system.

### Reuse and immutability

A pen holds no per-glyph state — all of that lives in `hb_draw_state_t` and in
your `draw_data` — so one pen can serve any number of glyphs, fonts, and threads.
The recommended pattern is: create, configure, `hb_draw_funcs_make_immutable()`,
then share freely. HarfBuzz objects are not internally locked, and an immutable
pen is the only kind that is safe to use from several threads at once, since
nothing can then mutate the callback table underneath a reader. The reference
count itself is atomic, so `hb_draw_funcs_reference()`/`destroy()` from multiple
threads is fine.

`draw_data`, by contrast, is per-call and must not be shared between concurrent
drawing calls unless you synchronise it yourself.

### Failure is invisible

Almost everything in this header returns `void`. There is no way for a pen to
report an error, no way for a setter to report that it did nothing, and
`hb_draw_funcs_create()` substitutes a shared empty object rather than returning
null on allocation failure. Build the checks you need yourself:

* Compare the result of `hb_draw_funcs_create()` against
  `hb_draw_funcs_get_empty()` if you must detect OOM.
* Call `hb_draw_funcs_is_immutable()` before a setter if a failed set would be a
  correctness problem.
* Track your pen's own errors in `draw_data` and inspect them after drawing.
* Use `hb_font_draw_glyph_or_fail()` (from `hb-font.h`) rather than
  `hb_font_draw_glyph()` when you need to know whether the glyph had an outline
  at all.

### Rust-side reminders

Every function here is `unsafe`, and this crate adds nothing on top of the C
contract. In particular:

- Callback typedefs are `Option<unsafe extern "C" fn(...)>`. Pass
  `Some(my_callback)` to install, `None` to reset the slot to its default. Your
  callback must be declared `unsafe extern "C"` and **must not unwind** — a panic
  crossing back into C is undefined behaviour, so wrap fallible bodies in
  `catch_unwind` or keep them panic-free.
- `hb_destroy_func_t` is likewise an `Option`; pass `None` when there is nothing
  to free.
- `hb_draw_state_t` is `Copy` but is passed by pointer. Do not copy it out,
  modify the copy, and expect HarfBuzz to notice — and do not modify the original
  either.
- `HB_DRAW_STATE_DEFAULT` is a `const` value, not a macro; write
  `let mut st = HB_DRAW_STATE_DEFAULT;` and pass `&raw mut st`.
- `stroke_width` sentinels are floating-point: use `f32::NAN` for "filled", and
  remember that `f32::NAN` compares unequal to itself — test with `is_nan()`, not
  `==`.
- `hb_bool_t` is `c_int`. Compare against `0`, do not transmute to `bool`.
