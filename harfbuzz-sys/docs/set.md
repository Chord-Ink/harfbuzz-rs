# Sets

Header: `hb-set.h` — Rust module: `harfbuzz_sys::set` (glob re-exported from the crate root).

## Overview

An `hb_set_t` is HarfBuzz's mathematical set of unsigned 32-bit integers. Despite
the `hb_codepoint_t` element type, a set is not restricted to Unicode: the same
type holds glyph IDs, `hb_tag_t` values reinterpreted as integers, layout lookup
indices, feature indices, and any other collection of discrete non-negative
values the API needs to hand back or take in. It is the standard "collection of
things" currency of the non-shaping side of HarfBuzz — face and font
introspection (`hb_face_collect_unicodes`, `hb_face_collect_glyphs`), OpenType
layout queries (`hb_ot_layout_collect_lookups`), and the subsetter's input plan
all speak in sets.

Sets are heap-allocated, reference-counted objects. `hb_set_create()` returns a
set with one reference. `hb_set_reference()` takes an additional reference and
returns the same pointer; `hb_set_destroy()` drops one reference, and the set is
freed only when the count reaches zero. This is the same object protocol as
`hb_blob_t`, `hb_face_t`, and `hb_font_t`: a function that returns a set with
"transfer full" ownership hands you a reference you must eventually destroy, and
a function that merely reads a set borrows it for the duration of the call. Each
set also carries a user-data table keyed by the *address* of an
`hb_user_data_key_t`, via `hb_set_set_user_data` / `hb_set_get_user_data`.

The internal representation is a sparse bit set: contiguous runs of pages, so a
set holding "every code point from 0 to 0x10FFFF" costs about as much as one
holding a few hundred scattered values. This is why the range-oriented calls
(`hb_set_add_range`, `hb_set_next_range`) exist and why they are dramatically
faster than looping element by element over a dense set. A set can additionally
be *inverted* (`hb_set_invert`), which flips membership for the whole 32-bit
universe in constant time rather than materialising 4 billion elements; a set in
that state reports `hb_set_is_inverted() == true` and its population counts the
complement.

Because every mutating operation may need to allocate a new page, allocation
failure is not reported per call — the mutators all return `void`. Instead the
set latches a sticky failure flag, and `hb_set_allocation_successful()` reports
it. A set that has failed once stays failed and silently ignores subsequent
mutations, so the intended usage is: perform a batch of operations, then check
`hb_set_allocation_successful()` once before trusting the result.

Sets are value-comparable (`hb_set_is_equal`), orderable by containment
(`hb_set_is_subset`), hashable (`hb_set_hash`), and support the four standard
binary operations in place (`hb_set_union`, `hb_set_intersect`, `hb_set_subtract`,
`hb_set_symmetric_difference`). Iteration is via stateless cursor functions that
carry their position in an in/out `hb_codepoint_t` — there is no iterator object
to allocate or free.

## Types

### `hb_set_t`

An opaque, reference-counted set of `hb_codepoint_t` values. The header declares
it as `typedef struct hb_set_t hb_set_t;` with no visible body, so its size and
layout are private to the library and it is only ever handled through pointers.

In Rust it is declared with the crate's `opaque_handle!` macro, producing a
zero-sized `#[repr(C)]` type that cannot be constructed, copied, or moved, and
that is neither `Send` nor `Sync`. You always work with `*mut hb_set_t` or
`*const hb_set_t`.

This header declares no structs with visible bodies, no enumerations, and no
function-pointer typedefs.

## Constants

### `HB_SET_VALUE_INVALID`

```c
#define HB_SET_VALUE_INVALID HB_CODEPOINT_INVALID
```

```rust
pub const HB_SET_VALUE_INVALID: hb_codepoint_t = HB_CODEPOINT_INVALID; // 0xFFFFFFFF
```

The "unset" set value. Since HarfBuzz 0.9.21. It serves three distinct roles:

* the sentinel returned by `hb_set_get_min` / `hb_set_get_max` for an empty set;
* the "start from the beginning/end" seed for every iteration function;
* the "unbounded" marker for the `last` argument of `hb_set_del_range`.

It is defined as `HB_CODEPOINT_INVALID`, which lives in `hb-common.h` and is
imported from `crate::common` rather than redefined here. Note the consequence:
`0xFFFFFFFF` cannot be stored as an ordinary member and round-tripped through
the iteration API, because it is indistinguishable from the sentinel.

## Functions

Every function below is declared in one `unsafe extern "C"` block in
`src/set.rs`; the Rust signatures shown omit the surrounding `pub fn`
boilerplate's `unsafe` keyword for brevity, but all of them are `unsafe` to call.

### Creation, lifetime, and user data

#### `hb_set_create`

```c
hb_set_t *hb_set_create (void);
```
```rust
pub fn hb_set_create() -> *mut hb_set_t;
```

Creates a new, initially empty set with a reference count of one. Since 0.9.2.

**Ownership:** transfer full — the caller must eventually call `hb_set_destroy`.

**Failure:** never returns null. On allocation failure it returns the singleton
empty set (`hb_set_get_empty()`), which is inert. Detect this with
`hb_set_allocation_successful()` rather than a null check.

#### `hb_set_get_empty`

```c
hb_set_t *hb_set_get_empty (void);
```
```rust
pub fn hb_set_get_empty() -> *mut hb_set_t;
```

Fetches the singleton empty set. Since 0.9.2.

**Ownership:** annotated "transfer full" upstream, so it is correct — and
harmless — to `hb_set_destroy` the result. Following HarfBuzz's nil-object
convention, reference counting on this singleton is a no-op and it is never
actually freed.

**Failure:** never returns null.

**Note:** the singleton is shared process-wide. The header is silent about
mutating it; HarfBuzz's convention is that the nil object absorbs mutations
without taking effect, so do not treat a set you obtained from this function as
a scratch buffer.

#### `hb_set_reference`

```c
hb_set_t *hb_set_reference (hb_set_t *set);
```
```rust
pub fn hb_set_reference(set: *mut hb_set_t) -> *mut hb_set_t;
```

Increases the reference count on a set and returns the same pointer. Since 0.9.2.

**Ownership:** transfer full — the returned pointer is a new reference that must
be balanced with `hb_set_destroy`.

#### `hb_set_destroy`

```c
void hb_set_destroy (hb_set_t *set);
```
```rust
pub fn hb_set_destroy(set: *mut hb_set_t);
```

Decreases the reference count on a set. When the count reaches zero the set is
destroyed and all its memory freed, and any registered user-data destroy
callbacks are invoked. Since 0.9.2.

#### `hb_set_set_user_data`

```c
hb_bool_t hb_set_set_user_data (hb_set_t           *set,
                                hb_user_data_key_t *key,
                                void               *data,
                                hb_destroy_func_t   destroy,
                                hb_bool_t           replace);
```
```rust
pub fn hb_set_set_user_data(
    set: *mut hb_set_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

Attaches a user-data key/data pair to the set. Since 0.9.2.

**Returns** true on success, false otherwise (for example, when `replace` is
false and an entry already exists under `key`, or on allocation failure).

**Ownership:** HarfBuzz stores `data` as an opaque pointer and does not copy it.
`destroy` is nullable (`None` in Rust) and is called with `data` when the entry
is replaced or when the set is destroyed. The `key` pointer is used by *address*,
so it must outlive the set — in practice a `static` `hb_user_data_key_t`.

#### `hb_set_get_user_data`

```c
void *hb_set_get_user_data (const hb_set_t     *set,
                            hb_user_data_key_t *key);
```
```rust
pub fn hb_set_get_user_data(set: *const hb_set_t, key: *mut hb_user_data_key_t) -> *mut c_void;
```

Fetches the user data attached under `key`. Since 0.9.2.

**Returns** null if nothing is attached under that key.

**Ownership:** transfer none — the returned pointer stays owned by whoever
supplied it to `hb_set_set_user_data`.

### Error state

#### `hb_set_allocation_successful`

```c
hb_bool_t hb_set_allocation_successful (const hb_set_t *set);
```
```rust
pub fn hb_set_allocation_successful(set: *const hb_set_t) -> hb_bool_t;
```

Tests whether memory allocation for the set has succeeded so far. Since 0.9.2.
The header comment is explicit: "Returns false if allocation has failed before."

**Returns** false once any internal allocation has failed. The flag is sticky, so
this reports the health of the whole history of the set, not just the last call.
When it is false the set's contents are incomplete and must not be trusted.

### Whole-set operations

#### `hb_set_copy`

```c
hb_set_t *hb_set_copy (const hb_set_t *set);
```
```rust
pub fn hb_set_copy(set: *const hb_set_t) -> *mut hb_set_t;
```

Allocates a deep copy of the set. Since 2.8.2.

**Ownership:** transfer full — the result is a brand-new set with its own
reference count, independent of the source. Destroy it when done. Contrast with
`hb_set_reference`, which shares the same object.

**Failure:** as with `hb_set_create`, an allocation failure surfaces through
`hb_set_allocation_successful` on the result, not as null.

#### `hb_set_clear`

```c
void hb_set_clear (hb_set_t *set);
```
```rust
pub fn hb_set_clear(set: *mut hb_set_t);
```

Removes all elements from the set. Since 0.9.2. Also clears the inverted state.

#### `hb_set_is_empty`

```c
hb_bool_t hb_set_is_empty (const hb_set_t *set);
```
```rust
pub fn hb_set_is_empty(set: *const hb_set_t) -> hb_bool_t;
```

Tests whether the set contains no elements. Since 0.9.7.

#### `hb_set_invert`

```c
void hb_set_invert (hb_set_t *set);
```
```rust
pub fn hb_set_invert(set: *mut hb_set_t);
```

Inverts the contents of the set — after the call it contains exactly the values
it did not contain before, over the full `hb_codepoint_t` range. Since 3.0.0.
This is a constant-time flag flip, not a materialisation of 2³² elements.

#### `hb_set_is_inverted`

```c
hb_bool_t hb_set_is_inverted (const hb_set_t *set);
```
```rust
pub fn hb_set_is_inverted(set: *const hb_set_t) -> hb_bool_t;
```

Returns whether the set is currently in the inverted state. Since 7.0.0.

#### `hb_set_set`

```c
void hb_set_set (hb_set_t *set, const hb_set_t *other);
```
```rust
pub fn hb_set_set(set: *mut hb_set_t, other: *const hb_set_t);
```

Makes the contents of `set` equal to the contents of `other`. Since 0.9.2.
This copies the contents; the two sets remain distinct objects afterwards.

### Membership

#### `hb_set_has`

```c
hb_bool_t hb_set_has (const hb_set_t *set, hb_codepoint_t codepoint);
```
```rust
pub fn hb_set_has(set: *const hb_set_t, codepoint: hb_codepoint_t) -> hb_bool_t;
```

Tests whether `codepoint` belongs to the set. Since 0.9.2.

#### `hb_set_add`

```c
void hb_set_add (hb_set_t *set, hb_codepoint_t codepoint);
```
```rust
pub fn hb_set_add(set: *mut hb_set_t, codepoint: hb_codepoint_t);
```

Adds `codepoint` to the set. Since 0.9.2. Adding an element already present is a
no-op. Silently does nothing if the set is already in the failed-allocation
state.

#### `hb_set_add_range`

```c
void hb_set_add_range (hb_set_t *set, hb_codepoint_t first, hb_codepoint_t last);
```
```rust
pub fn hb_set_add_range(set: *mut hb_set_t, first: hb_codepoint_t, last: hb_codepoint_t);
```

Adds every element from `first` to `last`, **inclusive of both ends**. Since
0.9.7. Prefer this to a loop over `hb_set_add`: it operates on whole pages.

#### `hb_set_add_sorted_array`

```c
void hb_set_add_sorted_array (hb_set_t             *set,
                              const hb_codepoint_t *sorted_codepoints,
                              unsigned int          num_codepoints);
```
```rust
pub fn hb_set_add_sorted_array(
    set: *mut hb_set_t,
    sorted_codepoints: *const hb_codepoint_t,
    num_codepoints: c_uint,
);
```

Adds `num_codepoints` values at once. Since 4.1.0.

**Precondition:** the array must be in increasing order and must have at least
`num_codepoints` elements. Passing an unsorted array is a contract violation, not
an error HarfBuzz reports.

**Ownership:** the array is read and copied into the set; HarfBuzz does not
retain the pointer.

#### `hb_set_del`

```c
void hb_set_del (hb_set_t *set, hb_codepoint_t codepoint);
```
```rust
pub fn hb_set_del(set: *mut hb_set_t, codepoint: hb_codepoint_t);
```

Removes `codepoint` from the set. Since 0.9.2. Removing an absent element is a
no-op.

#### `hb_set_del_range`

```c
void hb_set_del_range (hb_set_t *set, hb_codepoint_t first, hb_codepoint_t last);
```
```rust
pub fn hb_set_del_range(set: *mut hb_set_t, first: hb_codepoint_t, last: hb_codepoint_t);
```

Removes every element from `first` to `last`, inclusive. Since 0.9.7.

**Special case:** if `last` is `HB_SET_VALUE_INVALID`, every value greater than
or equal to `first` is removed. This makes `hb_set_del_range(s, 0,
HB_SET_VALUE_INVALID)` equivalent to clearing the set.

### Comparison and hashing

#### `hb_set_is_equal`

```c
hb_bool_t hb_set_is_equal (const hb_set_t *set, const hb_set_t *other);
```
```rust
pub fn hb_set_is_equal(set: *const hb_set_t, other: *const hb_set_t) -> hb_bool_t;
```

Tests whether the two sets contain the same elements. Since 0.9.7. This is a
value comparison, not a pointer comparison.

#### `hb_set_hash`

```c
unsigned int hb_set_hash (const hb_set_t *set);
```
```rust
pub fn hb_set_hash(set: *const hb_set_t) -> c_uint;
```

Creates a hash representing the set. Since 4.4.0. Sets that compare equal under
`hb_set_is_equal` hash equally, which makes this usable as the hash half of a
hash-table key. The header does not promise stability of the value across
HarfBuzz versions or process runs, so do not persist it.

#### `hb_set_is_subset`

```c
hb_bool_t hb_set_is_subset (const hb_set_t *set, const hb_set_t *larger_set);
```
```rust
pub fn hb_set_is_subset(set: *const hb_set_t, larger_set: *const hb_set_t) -> hb_bool_t;
```

Tests whether `set` is a subset of `larger_set`. Since 1.8.1. This is
*non-strict*: equal sets are subsets of each other, so identical sets return
true.

### Binary set operations

All four mutate the left-hand `set` in place and leave `other` untouched. All are
since 0.9.2.

| Function | Effect |
| --- | --- |
| `hb_set_union (set, other)` | `set` ← `set` ∪ `other` |
| `hb_set_intersect (set, other)` | `set` ← `set` ∩ `other` |
| `hb_set_subtract (set, other)` | `set` ← `set` \ `other` |
| `hb_set_symmetric_difference (set, other)` | `set` ← `set` △ `other` |

```c
void hb_set_union                (hb_set_t *set, const hb_set_t *other);
void hb_set_intersect            (hb_set_t *set, const hb_set_t *other);
void hb_set_subtract             (hb_set_t *set, const hb_set_t *other);
void hb_set_symmetric_difference (hb_set_t *set, const hb_set_t *other);
```
```rust
pub fn hb_set_union(set: *mut hb_set_t, other: *const hb_set_t);
pub fn hb_set_intersect(set: *mut hb_set_t, other: *const hb_set_t);
pub fn hb_set_subtract(set: *mut hb_set_t, other: *const hb_set_t);
pub fn hb_set_symmetric_difference(set: *mut hb_set_t, other: *const hb_set_t);
```

None of them take a reference on `other`; it is borrowed for the duration of the
call only. All can fail to allocate, which surfaces through
`hb_set_allocation_successful(set)`.

### Size and extrema

#### `hb_set_get_population`

```c
unsigned int hb_set_get_population (const hb_set_t *set);
```
```rust
pub fn hb_set_get_population(set: *const hb_set_t) -> c_uint;
```

Returns the number of elements in the set. Since 0.9.7. For an inverted set this
is the size of the complement, which can be very large — up to 2³² − 1, which does
not fit the signed range of a C `int`, so treat it as unsigned throughout.

#### `hb_set_get_min` / `hb_set_get_max`

```c
hb_codepoint_t hb_set_get_min (const hb_set_t *set);
hb_codepoint_t hb_set_get_max (const hb_set_t *set);
```
```rust
pub fn hb_set_get_min(set: *const hb_set_t) -> hb_codepoint_t;
pub fn hb_set_get_max(set: *const hb_set_t) -> hb_codepoint_t;
```

Find the smallest and largest elements. Both since 0.9.7. The header states
explicitly that they return `HB_SET_VALUE_INVALID` if the set is empty.

### Iteration

All five iteration functions are stateless: the cursor lives in the caller's
`hb_codepoint_t` variables, which the callee reads and writes. That means
iteration is re-entrant and needs no allocation, but also that mutating the set
mid-iteration gives you whatever the new structure implies from the current
cursor value.

#### `hb_set_next`

```c
hb_bool_t hb_set_next (const hb_set_t *set, hb_codepoint_t *codepoint);
```
```rust
pub fn hb_set_next(set: *const hb_set_t, codepoint: *mut hb_codepoint_t) -> hb_bool_t;
```

In/out: reads the current value, writes the next element strictly greater than
it. Since 0.9.2. Seed with `HB_SET_VALUE_INVALID` to get started — the sentinel
wraps to the smallest element.

**Returns** false when there is no next value; the header does not specify what
`*codepoint` holds in that case, though in practice HarfBuzz writes
`HB_SET_VALUE_INVALID`. Do not rely on the out value when the return is false.

#### `hb_set_previous`

```c
hb_bool_t hb_set_previous (const hb_set_t *set, hb_codepoint_t *codepoint);
```
```rust
pub fn hb_set_previous(set: *const hb_set_t, codepoint: *mut hb_codepoint_t) -> hb_bool_t;
```

The mirror of `hb_set_next`: writes the largest element strictly less than the
input. Since 1.8.0. Seed with `HB_SET_VALUE_INVALID` to start from the top.
Returns false when there is no previous value.

#### `hb_set_next_range`

```c
hb_bool_t hb_set_next_range (const hb_set_t *set,
                             hb_codepoint_t *first,
                             hb_codepoint_t *last);
```
```rust
pub fn hb_set_next_range(
    set: *const hb_set_t,
    first: *mut hb_codepoint_t,
    last: *mut hb_codepoint_t,
) -> hb_bool_t;
```

Fetches the next maximal run of consecutive members greater than the current
`*last`. Since 0.9.7. `first` is purely an out-parameter; `last` is in/out and
carries the cursor. The header instructs you to pass `HB_SET_VALUE_INVALID` for
both to get started.

**Returns** false when there is no further range. The returned range is inclusive
at both ends, matching `hb_set_add_range`.

#### `hb_set_previous_range`

```c
hb_bool_t hb_set_previous_range (const hb_set_t *set,
                                 hb_codepoint_t *first,
                                 hb_codepoint_t *last);
```
```rust
pub fn hb_set_previous_range(
    set: *const hb_set_t,
    first: *mut hb_codepoint_t,
    last: *mut hb_codepoint_t,
) -> hb_bool_t;
```

The mirror: fetches the previous run of consecutive members, below the current
`*first`. Since 1.8.0. Here `first` is the in/out cursor and `last` is the
out-parameter — the roles are swapped relative to `hb_set_next_range`, which is
easy to get wrong. Pass `HB_SET_VALUE_INVALID` for both to get started.

#### `hb_set_next_many`

```c
unsigned int hb_set_next_many (const hb_set_t *set,
                               hb_codepoint_t  codepoint,
                               hb_codepoint_t *out,
                               unsigned int    size);
```
```rust
pub fn hb_set_next_many(
    set: *const hb_set_t,
    codepoint: hb_codepoint_t,
    out: *mut hb_codepoint_t,
    size: c_uint,
) -> c_uint;
```

Bulk iteration. Since 4.2.0. Finds the elements greater than `codepoint` and
writes them into `out`, stopping when the set is exhausted or `size` values have
been written, whichever comes first.

Note that `codepoint` is passed **by value**, not in/out: to continue, pass the
last value written to `out` on the next call. Pass `HB_SET_VALUE_INVALID` to
start.

**Returns** the number of values written, which is less than `size` exactly when
the set has been exhausted. `out` must have room for `size` elements; the header
carries the `(array length=size)` annotation.

## Usage notes

### Error handling is deferred, not per-call

Nothing that mutates a set returns a status. Batch your work and check once:

```c
hb_set_t *s = hb_set_create ();
hb_set_add_range (s, 0x0041, 0x005A);
hb_set_add_range (s, 0x0061, 0x007A);
hb_set_union (s, other);
if (!hb_set_allocation_successful (s)) {
  /* out of memory somewhere above; s is unusable */
}
hb_set_destroy (s);
```

Once the flag is false the set ignores further mutations, so a failed set does
not silently accumulate a half-correct answer — but it also will not tell you
where things went wrong.

### `hb_set_create` never returns null

The nil-object pattern means a null check on the constructor is dead code. This
trips up code translated from allocator-style APIs. The same applies to
`hb_set_copy` and `hb_set_get_empty`.

### Reference vs copy

`hb_set_reference` and `hb_set_copy` look similar and mean opposite things.
`hb_set_reference` gives you a second handle to the *same* set — mutations
through either handle are visible through both. `hb_set_copy` gives you an
independent snapshot. If you receive a set from a HarfBuzz getter and want to
keep it beyond the current scope, decide which one you actually need.

### Iteration idioms

Element-by-element:

```c
hb_codepoint_t cp = HB_SET_VALUE_INVALID;
while (hb_set_next (set, &cp))
  do_something (cp);
```

Range-by-range, which is what you want for anything remotely dense:

```c
hb_codepoint_t first = HB_SET_VALUE_INVALID, last = HB_SET_VALUE_INVALID;
while (hb_set_next_range (set, &first, &last))
  do_something_with_range (first, last);   /* inclusive both ends */
```

Bulk, for filling a buffer:

```c
hb_codepoint_t buf[512];
hb_codepoint_t cursor = HB_SET_VALUE_INVALID;
unsigned n;
while ((n = hb_set_next_many (set, cursor, buf, 512))) {
  process (buf, n);
  cursor = buf[n - 1];
}
```

In Rust the same loop, on the raw bindings:

```rust
use harfbuzz_sys::{HB_SET_VALUE_INVALID, hb_codepoint_t, hb_set_next, hb_set_t};

unsafe fn collect(set: *const hb_set_t, out: &mut alloc::vec::Vec<hb_codepoint_t>) {
    let mut cp: hb_codepoint_t = HB_SET_VALUE_INVALID;
    while unsafe { hb_set_next(set, &mut cp) } != 0 {
        out.push(cp);
    }
}
```

### Inverted sets and iteration

`hb_set_invert` makes a set that may contain nearly 2³² elements. Iterating it
one element at a time will not finish in reasonable time. Always use
`hb_set_next_range` on a set that might be inverted, and check
`hb_set_is_inverted` if your algorithm cannot cope with an effectively unbounded
set. Similarly, `hb_set_get_max` on an inverted set is likely to be `0xFFFFFFFE`
rather than anything meaningful about your data.

### `0xFFFFFFFF` is not usable as a member

Because `HB_SET_VALUE_INVALID` doubles as the iteration sentinel and the
empty-set return of `hb_set_get_min` / `hb_set_get_max`, the value `0xFFFFFFFF`
cannot be distinguished from "no value" through those APIs. `hb_set_has` will
still answer honestly about it, but no iteration loop will yield it. Do not use
sets to carry data where that value is meaningful.

### Ranges are inclusive

`hb_set_add_range`, `hb_set_del_range`, `hb_set_next_range`, and
`hb_set_previous_range` all use closed intervals `[first, last]`. Callers coming
from Rust's half-open `a..b` habit need to subtract one.

### The two `_range` iterators swap their cursor parameter

`hb_set_next_range` advances via `last`; `hb_set_previous_range` advances via
`first`. Seeding both parameters with `HB_SET_VALUE_INVALID`, as the header
instructs, makes this harmless at the start of a loop, but it matters if you
resume iteration from a computed position.

### Threading

The header says nothing about thread safety, and `hb_set_t` follows HarfBuzz's
general object rules rather than having any special guarantee. Reference counting
is atomic, so `hb_set_reference` / `hb_set_destroy` from multiple threads is safe;
concurrent *mutation* of the same set is not. The Rust `hb_set_t` handle is
deliberately `!Send` and `!Sync` for this reason, and a safe wrapper is where
`Send`/`Sync` should be opted back into with a justification.

### Constant-time operations are not free

`hb_set_get_population` is not necessarily O(1) — it may walk pages. In a hot
loop, prefer `hb_set_is_empty` when all you need is emptiness, and cache the
population if you need the count more than once.

## Cross-references

* `hb_codepoint_t`, `HB_CODEPOINT_INVALID`, `hb_bool_t`, `hb_user_data_key_t`,
  and `hb_destroy_func_t` come from `hb-common.h` (`crate::common`).
* `hb_map_t` in `hb-map.h` is the key/value counterpart to this header's set,
  and `hb_map_keys` / `hb_map_values` fill `hb_set_t`s.
* `hb_face_collect_unicodes`, `hb_face_collect_variation_selectors`, and
  `hb_face_collect_glyphs` in `hb-face.h`, plus most of `hb-ot-layout.h`'s
  `collect_*` calls, are the main producers of sets.
* `hb-subset.h` consumes sets as the primary description of what to keep.
