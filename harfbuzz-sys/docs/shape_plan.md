# Shape plans

Reference for `hb-shape-plan.h`, transcribed in `harfbuzz-sys` as the
`shape_plan` module (re-exported at the crate root).

## Overview

A **shape plan** is HarfBuzz's record of *how* a particular text segment will be
shaped. It is the resolved answer to a set of questions that HarfBuzz would
otherwise have to re-answer on every shaping call: which shaper back end (the
built-in OpenType shaper, CoreText, Graphite2, …) is going to run, and — for the
OpenType shaper — which script/language system, which complex-script machinery,
and which lookups and feature masks apply. The inputs that determine all of this
are exactly the arguments of the creation functions: a face, the segment
properties (direction, script, language), the caller's user features, the
variation-space coordinates, and an optional preference list of shapers.

Shape plans are an internal mechanism, and most client programs never touch
them. `hb_shape()` and `hb_shape_full()` (in `hb-shape.h`) build one internally
for every call — specifically, they call `hb_shape_plan_create_cached2()` with
the font's face, the buffer's segment properties, the caller's features and the
font's normalized variation coordinates, execute it, and release it. Because
that call goes through the per-face cache, the cost is paid once per distinct
combination of inputs rather than once per shaping call.

The reasons to use this API directly are narrow but real:

* **Introspection.** `hb_shape_plan_get_shaper()` tells you which back end
  HarfBuzz picked for a given face and segment — useful for diagnostics, for
  tests, and for tools such as `hb-shape --shapers`.
* **Shaper control.** Passing an explicit `shaper_list` lets you pin shaping to
  a particular back end, or to a preference order, instead of accepting the
  built-in default order.
* **Explicit reuse.** Holding a plan yourself and calling
  `hb_shape_plan_execute()` skips the cache lookup that `hb_shape_full()`
  performs, at the cost of having to guarantee yourself that the plan matches
  the font and buffer you hand it.

**Lifecycle.** `hb_shape_plan_t` is an opaque, reference-counted HarfBuzz
object, like `hb_face_t` and `hb_font_t`. The creation functions return a plan
you own; release it with `hb_shape_plan_destroy()`. `hb_shape_plan_reference()`
takes an extra reference. There is a singleton *empty* (inert) plan, returned by
`hb_shape_plan_get_empty()` and used as the failure value of every creation
function, so creation never returns null. As with other HarfBuzz objects, you
can hang arbitrary user data off a plan with
`hb_shape_plan_set_user_data()`/`hb_shape_plan_get_user_data()`.

**Caching.** The `_cached` variants store the plan in a linked list owned by the
face, and return an additional reference to an existing equivalent plan when one
is found. Those cached plans live as long as the face does. The plain
(non-cached) variants always build a fresh plan and are never registered with
the face.

## Types

### `hb_shape_plan_t`

```c
typedef struct hb_shape_plan_t hb_shape_plan_t;
```

```rust
crate::opaque_handle! { hb_shape_plan_t }
```

Opaque, reference-counted. It has no public fields and no accessors beyond
`hb_shape_plan_get_shaper()`; everything else it knows is consumed internally by
`hb_shape_plan_execute()`. In Rust it is a zero-sized `#[repr(C)]` handle that
can only be used behind a pointer.

### Types this header uses but does not declare

`hb-shape-plan.h` includes `hb-common.h` and `hb-font.h`, so the following come
from elsewhere and are imported by the Rust module rather than redeclared:

| Type | Declared in | Rust module |
| --- | --- | --- |
| `hb_bool_t` | `hb-common.h` | `common` |
| `hb_feature_t` | `hb-common.h` | `common` |
| `hb_user_data_key_t` | `hb-common.h` | `common` |
| `hb_destroy_func_t` | `hb-common.h` | `common` |
| `hb_face_t` | `hb-face.h` | `face` |
| `hb_font_t` | `hb-font.h` | `font` |
| `hb_buffer_t` | `hb-buffer.h` | `buffer` |
| `hb_segment_properties_t` | `hb-buffer.h` | `buffer` |

## Functions

### Creation and destruction

#### `hb_shape_plan_create`

```c
hb_shape_plan_t *
hb_shape_plan_create (hb_face_t                     *face,
                      const hb_segment_properties_t *props,
                      const hb_feature_t            *user_features,
                      unsigned int                   num_user_features,
                      const char * const            *shaper_list);
```

```rust
pub fn hb_shape_plan_create(
    face: *mut hb_face_t,
    props: *const hb_segment_properties_t,
    user_features: *const hb_feature_t,
    num_user_features: c_uint,
    shaper_list: *const *const c_char,
) -> *mut hb_shape_plan_t;
```

Constructs a shaping plan for the combination of `face`, `props`,
`user_features` and `shaper_list`. Equivalent to `hb_shape_plan_create2()` with
no variation coordinates. Since HarfBuzz 0.9.7.

* **Returns** a new plan the caller owns; destroy it with
  `hb_shape_plan_destroy()`. **Never null** — on any failure (no shaper matched,
  allocation failure) it returns the singleton empty plan, which is still a
  valid object and still must be destroyed by the caller if you treat the return
  value uniformly.
* **`shaper_list`** is a null-terminated array of NUL-terminated shaper names,
  tried in order; the first one the face supports wins. Pass `NULL` to use
  HarfBuzz's default order. The array is not retained: only the selected
  shaper's static name is stored.
* **`user_features`** may be `NULL` when `num_user_features` is 0. The plan
  copies the feature array into its own storage, so the caller's array does not
  need to outlive the plan.
* **`props`** is copied into the plan. The header does not say whether it may be
  null; in practice it is dereferenced unconditionally, so treat it as
  **required**.
* **Side effect:** the face is made immutable (as if by
  `hb_face_make_immutable()`), and the plan keeps a raw reference to it. See
  *Usage notes*.

#### `hb_shape_plan_create2`

```c
hb_shape_plan_t *
hb_shape_plan_create2 (hb_face_t                     *face,
                       const hb_segment_properties_t *props,
                       const hb_feature_t            *user_features,
                       unsigned int                   num_user_features,
                       const int                     *coords,
                       unsigned int                   num_coords,
                       const char * const            *shaper_list);
```

```rust
pub fn hb_shape_plan_create2(
    face: *mut hb_face_t,
    props: *const hb_segment_properties_t,
    user_features: *const hb_feature_t,
    num_user_features: c_uint,
    coords: *const c_int,
    num_coords: c_uint,
    shaper_list: *const *const c_char,
) -> *mut hb_shape_plan_t;
```

The variable-font version of `hb_shape_plan_create()`: same inputs plus the
variation-space coordinates `coords`. Since HarfBuzz 1.4.0.

* `coords` holds *normalized* axis coordinates — the same representation
  `hb_font_get_var_coords_normalized()` returns, not user-space design values.
  Pass `NULL`/0 for a non-variable instance.
* Ownership, nullability and the never-null return rule are as for
  `hb_shape_plan_create()`.
* If `props->direction` is not a valid direction, the empty plan is returned
  immediately.

#### `hb_shape_plan_create_cached`

```c
hb_shape_plan_t *
hb_shape_plan_create_cached (hb_face_t                     *face,
                             const hb_segment_properties_t *props,
                             const hb_feature_t            *user_features,
                             unsigned int                   num_user_features,
                             const char * const            *shaper_list);
```

```rust
pub fn hb_shape_plan_create_cached(
    face: *mut hb_face_t,
    props: *const hb_segment_properties_t,
    user_features: *const hb_feature_t,
    num_user_features: c_uint,
    shaper_list: *const *const c_char,
) -> *mut hb_shape_plan_t;
```

Creates a plan "suitable for reuse": it first looks for an equivalent plan
already attached to the face and, if found, returns a new reference to it;
otherwise it builds one and inserts it into the face's plan list. Since
HarfBuzz 0.9.7.

* **Returns** a reference the caller owns and must destroy, exactly like the
  non-cached version. Destroying it does not evict the plan from the cache —
  the face holds its own reference.
* Cached plans are freed when the face is destroyed, so the memory grows with
  the number of distinct plan keys used against that face.
* `face` is dereferenced to reach the cache before any null check, so a null
  `face` is not usable here even though the non-cached path tolerates it.

#### `hb_shape_plan_create_cached2`

```c
hb_shape_plan_t *
hb_shape_plan_create_cached2 (hb_face_t                     *face,
                              const hb_segment_properties_t *props,
                              const hb_feature_t            *user_features,
                              unsigned int                   num_user_features,
                              const int                     *coords,
                              unsigned int                   num_coords,
                              const char * const            *shaper_list);
```

```rust
pub fn hb_shape_plan_create_cached2(
    face: *mut hb_face_t,
    props: *const hb_segment_properties_t,
    user_features: *const hb_feature_t,
    num_user_features: c_uint,
    coords: *const c_int,
    num_coords: c_uint,
    shaper_list: *const *const c_char,
) -> *mut hb_shape_plan_t;
```

The variable-font version of `hb_shape_plan_create_cached()`. The variation
coordinates participate in the cache key, so different instances of the same
variable face get different cached plans. This is the function `hb_shape_full()`
itself calls. Since HarfBuzz 1.4.0.

#### `hb_shape_plan_get_empty`

```c
hb_shape_plan_t *
hb_shape_plan_get_empty (void);
```

```rust
pub fn hb_shape_plan_get_empty() -> *mut hb_shape_plan_t;
```

Fetches the singleton empty (inert) shaping plan. Since HarfBuzz 0.9.7.

* Never null. It is an immortal object: reference and destroy calls on it are
  no-ops, so it is safe — and conventional — to treat it like any other plan.
* Executing it always fails; `hb_shape_plan_get_shaper()` on it returns `NULL`.

#### `hb_shape_plan_reference`

```c
hb_shape_plan_t *
hb_shape_plan_reference (hb_shape_plan_t *shape_plan);
```

```rust
pub fn hb_shape_plan_reference(shape_plan: *mut hb_shape_plan_t) -> *mut hb_shape_plan_t;
```

Increases the reference count and returns the same pointer, for convenient
chaining. Since HarfBuzz 0.9.7.

#### `hb_shape_plan_destroy`

```c
void
hb_shape_plan_destroy (hb_shape_plan_t *shape_plan);
```

```rust
pub fn hb_shape_plan_destroy(shape_plan: *mut hb_shape_plan_t);
```

Decreases the reference count. When it reaches zero the plan is destroyed,
freeing all memory (including the copied user-feature array) and running any
user-data destroy callbacks. Since HarfBuzz 0.9.7.

### User data

#### `hb_shape_plan_set_user_data`

```c
hb_bool_t
hb_shape_plan_set_user_data (hb_shape_plan_t    *shape_plan,
                             hb_user_data_key_t *key,
                             void *              data,
                             hb_destroy_func_t   destroy,
                             hb_bool_t           replace);
```

```rust
pub fn hb_shape_plan_set_user_data(
    shape_plan: *mut hb_shape_plan_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

Attaches a user-data key/data pair to the plan. Since HarfBuzz 0.9.7.

* `key` identifies the slot **by address**, so it is normally a `static`
  `hb_user_data_key_t`. It must outlive the plan.
* `destroy` may be null (`None` in Rust); when non-null it is called with `data`
  when the plan is destroyed, or when the entry is replaced.
* `replace` selects whether an existing entry under the same key is overwritten.
  When it is false and an entry exists, the call fails.
* **Returns** true on success, false otherwise (for example allocation failure,
  or a refused replace). Setting user data on the empty plan fails.

#### `hb_shape_plan_get_user_data`

```c
void *
hb_shape_plan_get_user_data (const hb_shape_plan_t *shape_plan,
                             hb_user_data_key_t    *key);
```

```rust
pub fn hb_shape_plan_get_user_data(
    shape_plan: *const hb_shape_plan_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

Fetches the data previously attached under `key`. Since HarfBuzz 0.9.7.

* **Returns** the stored pointer, or `NULL` when no entry exists. Ownership does
  not transfer — the plan still owns the data and will run `destroy` on it.
* Note the asymmetry with the setter: the plan is `const` here, the key is not.

### Shaping and introspection

#### `hb_shape_plan_execute`

```c
hb_bool_t
hb_shape_plan_execute (hb_shape_plan_t    *shape_plan,
                       hb_font_t          *font,
                       hb_buffer_t        *buffer,
                       const hb_feature_t *features,
                       unsigned int        num_features);
```

```rust
pub fn hb_shape_plan_execute(
    shape_plan: *mut hb_shape_plan_t,
    font: *mut hb_font_t,
    buffer: *mut hb_buffer_t,
    features: *const hb_feature_t,
    num_features: c_uint,
) -> hb_bool_t;
```

Runs the plan's selected shaper over `buffer` using `font` and `features`. Since
HarfBuzz 0.9.7.

* **Returns** true on success, false otherwise. On success, a buffer whose
  content type was Unicode is switched to glyphs, exactly as `hb_shape()` would
  leave it. An empty buffer succeeds trivially.
* **Returns false** when the plan is the empty/inert plan, and when the selected
  shaper has no per-font data for `font`.
* **Preconditions** (checked with `assert` in debug builds, undefined behaviour
  otherwise): the buffer must not be immutable; its content type must be
  Unicode; `font`'s face must be the very face the plan was created for; and the
  buffer's segment properties must equal the `props` the plan was created with.
* `features` is the per-call feature list, separate from the `user_features`
  baked into the plan. It is read, not retained.
* The buffer is mutated in place; nothing is allocated for the caller to free.

#### `hb_shape_plan_get_shaper`

```c
const char *
hb_shape_plan_get_shaper (hb_shape_plan_t *shape_plan);
```

```rust
pub fn hb_shape_plan_get_shaper(shape_plan: *mut hb_shape_plan_t) -> *const c_char;
```

Fetches the name of the shaper this plan selected — `"ot"`, `"coretext"`,
`"graphite2"`, `"fallback"`, and so on. Since HarfBuzz 0.9.7.

* **Returns** a NUL-terminated static string owned by HarfBuzz; do not free it,
  and it stays valid past the plan's lifetime.
* **Can return `NULL`**: the empty plan has no shaper. Any plan that was built
  successfully has one, because failing to select a shaper is itself a
  construction failure that yields the empty plan.
* Takes a non-`const` pointer although it only reads.

## Usage notes

### The face becomes immutable

Creating any plan for a face makes that face immutable. Anything you intend to
configure on the face — an `hb_face_set_upem()`, `hb_face_set_glyph_count()`, a
table replacement — must happen *before* the first plan (and therefore before
the first `hb_shape()` call on any font derived from it). After that, setters on
the face are silently ignored.

### Creation never returns null, but may return nothing useful

Every creation function funnels its error paths into
`hb_shape_plan_get_empty()`. That means:

* You cannot detect failure by comparing against `NULL`.
* You *can* detect it by comparing the result against
  `hb_shape_plan_get_empty()`, or by checking that
  `hb_shape_plan_get_shaper()` is non-null.
* Unconditionally destroying the result is correct either way.

### The plan is bound to one face and one set of segment properties

`hb_shape_plan_execute()` asserts that `font`'s face is the plan's face and that
the buffer's `props` equal the plan's. Reusing a plan across a direction change,
a script change, or a different face is a programming error, not a slow path —
in release builds the assertions are compiled out and the result is undefined.
When in doubt, use `hb_shape_full()`, which re-derives the plan for you.

### What actually enters the cache key

For the `_cached` variants the key is: the segment properties, the variation
coordinates, the selected shaper, and the user features. User features are
compared by tag, by value, and by *whether* the range is global — not by the
exact `start`/`end` cluster values. Two requests whose only difference is which
cluster range a non-global feature covers therefore hit the same cached plan.
This is intentional (the plan encodes which lookups exist, not the ranges), but
it means you cannot treat a cached plan as a record of the exact feature ranges
you asked for.

Cached plans are never evicted; they are freed with the face. Do not feed
unbounded, caller-derived feature combinations into `_cached` variants for a
long-lived face.

### Threading

Reference counting on HarfBuzz objects is atomic, and the face's plan list is
inserted into with a compare-and-swap retry loop, so calling the `_cached`
constructors concurrently on the same face is safe; two threads racing on the
same key may briefly build two plans, one of which is discarded. A plan is
read-only during `hb_shape_plan_execute()`, so the same plan may be executed
from several threads at once — but the `hb_buffer_t` and `hb_font_t` you pass
must not be shared between them. `hb_shape_plan_set_user_data()` mutates the
plan and follows the same rules as user data on any other HarfBuzz object.

### Worked example: which shaper will be used?

```c
hb_segment_properties_t props = HB_SEGMENT_PROPERTIES_DEFAULT;
props.direction = HB_DIRECTION_LTR;
props.script    = HB_SCRIPT_ARABIC;
props.language  = hb_language_from_string ("ar", -1);

hb_shape_plan_t *plan =
  hb_shape_plan_create_cached (face, &props, NULL, 0, NULL);

const char *shaper = hb_shape_plan_get_shaper (plan);
printf ("shaper: %s\n", shaper ? shaper : "(none — plan construction failed)");

hb_shape_plan_destroy (plan);
```

### Worked example: pinning the OpenType shaper, then executing

```rust
use core::ffi::{c_char, c_uint};
use core::ptr;

// A null-terminated list of null-terminated shaper names.
let ot: &[u8] = b"ot\0";
let shaper_list: [*const c_char; 2] = [ot.as_ptr().cast::<c_char>(), ptr::null()];

unsafe {
    hb_buffer_guess_segment_properties(buffer);

    let mut props = core::mem::zeroed();
    hb_buffer_get_segment_properties(buffer, &mut props);

    let mut num_coords: c_uint = 0;

    let plan = hb_shape_plan_create_cached2(
        hb_font_get_face(font),
        &props,
        ptr::null(),
        0,
        hb_font_get_var_coords_normalized(font, &mut num_coords),
        num_coords,
        shaper_list.as_ptr(),
    );

    let ok = hb_shape_plan_execute(plan, font, buffer, ptr::null(), 0);

    hb_shape_plan_destroy(plan);
}
```

Note the order: the buffer's properties must be *guessed and read back* (via
`hb_buffer_guess_segment_properties()` / `hb_buffer_get_segment_properties()`)
before building the plan, or the plan's `props` will not match the buffer's at
execute time.

### Macros

`hb-shape-plan.h` defines no preprocessor macros beyond its include guard and
the `HB_BEGIN_DECLS`/`HB_END_DECLS`/`HB_EXTERN` boilerplate, so nothing was
skipped in the transcription.
