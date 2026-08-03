# Maps

Reference for `hb-map.h` — HarfBuzz's integer-to-integer hash map — as
transcribed in `harfbuzz_sys::map`.

## Overview

An `hb_map_t` is a hash map whose keys and values are both `hb_codepoint_t`
(a `uint32_t`). That is the entire type: there is no generic map, no other key
or value width, and no way to store a pointer or a struct in one. The natural
uses are the tables that show up all over font tooling — codepoint to glyph ID,
old glyph ID to new glyph ID, glyph ID to some small index.

Upstream is explicit that maps are a convenience rather than a load-bearing part
of the shaping API: HarfBuzz's own public entry points do not take or return
`hb_map_t` in the core headers, but the container is exported because the
library already has a good one and clients frequently want it. (The subsetting
API in `hb-subset.h` does consume maps, so if you use the `subset` feature you
will meet them there.)

Maps are ordinary HarfBuzz reference-counted objects and follow the same
lifecycle as `hb_blob_t`, `hb_face_t`, and friends: `hb_map_create` hands you
one reference, `hb_map_reference` takes an additional one, and `hb_map_destroy`
gives one back. The object is freed when the last reference is released. Like
other HarfBuzz objects a map also carries a user-data table, reachable through
`hb_map_set_user_data` and `hb_map_get_user_data`.

Two design details shape almost everything a caller has to get right. First,
allocation failure is *latched* rather than reported per call: `hb_map_set`
returns `void`, and a map that has run out of memory silently refuses further
insertions until you ask it about `hb_map_allocation_successful`. Second, the
"no value" sentinel `HB_MAP_VALUE_INVALID` is a perfectly legal value to store,
so `hb_map_get` alone cannot distinguish "absent" from "present and equal to
`(hb_codepoint_t) -1`" — that is what `hb_map_has` is for.

Unlike sets and faces, maps have no immutability flag: the header declares no
`hb_map_make_immutable` or `hb_map_is_immutable`. The only inherently
unwritable map is the singleton returned by `hb_map_get_empty`.

## Types

### `hb_map_t`

Opaque, reference-counted, heap-allocated hash map from `hb_codepoint_t` to
`hb_codepoint_t`. The header declares it as `typedef struct hb_map_t hb_map_t;`
with no visible body, so it is transcribed with the crate's `opaque_handle!`
macro and is only ever handled through `*mut hb_map_t` / `*const hb_map_t`.

```rust
pub struct hb_map_t { /* opaque */ }
```

A `const hb_map_t *` in the C signatures means the call only reads the map; a
`hb_map_t *` means it may write to it. That distinction is carried over
faithfully as `*const` versus `*mut`, and it is the quickest way to tell an
accessor from a mutator in the table below.

## Constants

| Constant               | C definition                | Rust type        | Value        | Since |
| ---------------------- | --------------------------- | ---------------- | ------------ | ----- |
| `HB_MAP_VALUE_INVALID` | `#define ... HB_CODEPOINT_INVALID` | `hb_codepoint_t` | `0xFFFFFFFF` | 1.7.7 |

```rust
pub const HB_MAP_VALUE_INVALID: hb_codepoint_t = HB_CODEPOINT_INVALID;
```

The unset map value. It is defined as exactly `HB_CODEPOINT_INVALID`, which
comes from `hb-common.h` and therefore lives in `crate::common`, not here. It
serves double duty as the value `hb_map_get` reports for a missing key.

There are no function-like macros in this header, so nothing was skipped.

## Functions

Every function below is declared in one `unsafe extern "C"` block. The C
signatures are quoted verbatim from the header; the "Since" versions come from
the gtk-doc comments upstream keeps in `hb-map.cc` alongside each
implementation, since this header carries prose only for the type and the
macro.

### Creation, references, and destruction

#### `hb_map_create`

```c
hb_map_t * hb_map_create (void);
```
```rust
pub fn hb_map_create() -> *mut hb_map_t;
```

Creates a new, empty map and returns the caller's reference to it. Release it
with `hb_map_destroy`.

Never returns null. If the object allocation fails it returns the singleton
empty map instead, which means the pointer is always safe to pass onward but
may not be a map you can actually write to. `hb_map_allocation_successful` on
the result distinguishes the two cases. Since 1.7.7.

#### `hb_map_get_empty`

```c
hb_map_t * hb_map_get_empty (void);
```
```rust
pub fn hb_map_get_empty() -> *mut hb_map_t;
```

Fetches the singleton empty map — HarfBuzz's "nil object" for this type. It is
statically allocated and inert: reference counting on it is a no-op, and
`hb_map_set` on it is silently discarded rather than crashing. Upstream still
documents the return as a full transfer, so the tidy thing is to balance it
with `hb_map_destroy` like any other reference; doing so costs nothing.

Never returns null. Since 1.7.7.

#### `hb_map_reference`

```c
hb_map_t * hb_map_reference (hb_map_t *map);
```
```rust
pub fn hb_map_reference(map: *mut hb_map_t) -> *mut hb_map_t;
```

Increments the reference count and returns the same pointer, so it can be used
inline (`store(hb_map_reference(map))`). Each successful call must be matched by
one `hb_map_destroy`. Since 1.7.7.

#### `hb_map_destroy`

```c
void hb_map_destroy (hb_map_t *map);
```
```rust
pub fn hb_map_destroy(map: *mut hb_map_t);
```

Decrements the reference count. At zero the map is destroyed, its user-data
destroy callbacks run, and all of its memory is freed. The pointer must not be
used afterwards. Since 1.7.7.

### User data

#### `hb_map_set_user_data`

```c
hb_bool_t hb_map_set_user_data (hb_map_t           *map,
                                hb_user_data_key_t *key,
                                void *              data,
                                hb_destroy_func_t   destroy,
                                hb_bool_t           replace);
```
```rust
pub fn hb_map_set_user_data(
    map: *mut hb_map_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

Attaches `data` to the map under `key`. HarfBuzz keys on the *address* of the
`hb_user_data_key_t`, so the key object must outlive the map — a `static` is
the usual choice.

`destroy` is nullable (`None` in Rust) and is called with `data` when the map is
destroyed or when the entry is replaced. `replace` controls whether an existing
entry under the same key is overwritten; when it is false and an entry already
exists, the call fails. Returns true on success, false otherwise (including on
allocation failure). The map does not copy `data`; it stores the pointer.
Since 1.7.7.

#### `hb_map_get_user_data`

```c
void * hb_map_get_user_data (const hb_map_t     *map,
                             hb_user_data_key_t *key);
```
```rust
pub fn hb_map_get_user_data(map: *const hb_map_t, key: *mut hb_user_data_key_t) -> *mut c_void;
```

Fetches the data previously attached under `key`. Ownership is not transferred:
do not free the result, and do not use it after the map is destroyed (the
`destroy` callback will already have run). Returns null when nothing is attached
under that key. Since 1.7.7.

### Whole-map operations

#### `hb_map_allocation_successful`

```c
hb_bool_t hb_map_allocation_successful (const hb_map_t *map);
```
```rust
pub fn hb_map_allocation_successful(map: *const hb_map_t) -> hb_bool_t;
```

Returns false if any allocation this map attempted has ever failed — the
header's own comment reads "Returns false if allocation has failed before". The
flag is sticky: once tripped, the map refuses further insertions and stays in
the error state until it is cleared or destroyed. This is the only error channel
the mutating functions have, since they all return `void`. Since 1.7.7.

#### `hb_map_copy`

```c
hb_map_t * hb_map_copy (const hb_map_t *map);
```
```rust
pub fn hb_map_copy(map: *const hb_map_t) -> *mut hb_map_t;
```

Allocates a deep copy of `map`'s contents and returns the caller's reference to
it; release it with `hb_map_destroy`. The source is unchanged, and user data is
not copied. Never returns null — on allocation failure you get the singleton
empty map, so check `hb_map_allocation_successful` (or compare populations) if
a silently-empty copy would be a problem. Since 4.4.0.

#### `hb_map_clear`

```c
void hb_map_clear (hb_map_t *map);
```
```rust
pub fn hb_map_clear(map: *mut hb_map_t);
```

Removes every key/value pair, leaving the map empty and reusable. Since 1.7.7.

#### `hb_map_is_empty`

```c
hb_bool_t hb_map_is_empty (const hb_map_t *map);
```
```rust
pub fn hb_map_is_empty(map: *const hb_map_t) -> hb_bool_t;
```

True when the map holds no pairs. Equivalent to
`hb_map_get_population(map) == 0`, but says what you mean. Since 1.7.7.

#### `hb_map_get_population`

```c
unsigned int hb_map_get_population (const hb_map_t *map);
```
```rust
pub fn hb_map_get_population(map: *const hb_map_t) -> c_uint;
```

The number of key/value pairs currently stored. Since 1.7.7.

#### `hb_map_is_equal`

```c
hb_bool_t hb_map_is_equal (const hb_map_t *map,
                           const hb_map_t *other);
```
```rust
pub fn hb_map_is_equal(map: *const hb_map_t, other: *const hb_map_t) -> hb_bool_t;
```

True when both maps contain exactly the same key/value pairs. Insertion order
is irrelevant. Neither argument is modified. Since 4.3.0.

#### `hb_map_hash`

```c
unsigned int hb_map_hash (const hb_map_t *map);
```
```rust
pub fn hb_map_hash(map: *const hb_map_t) -> c_uint;
```

Computes a hash of the map's contents, suitable for using a map as a key in
another hash table together with `hb_map_is_equal`. Maps that compare equal hash
equal. The value is content-derived, so it changes as the map is mutated, and
the header makes no promise that it is stable across HarfBuzz versions — do not
persist it. Since 4.4.0.

#### `hb_map_update`

```c
void hb_map_update (hb_map_t *map,
                    const hb_map_t *other);
```
```rust
pub fn hb_map_update(map: *mut hb_map_t, other: *const hb_map_t);
```

Adds the contents of `other` into `map`, in the sense of Python's
`dict.update`: keys present in both take their value from `other`. `other` is
read-only and is not consumed. Failures during the merge show up through
`hb_map_allocation_successful`. Since 7.0.0.

### Per-entry access

#### `hb_map_set`

```c
void hb_map_set (hb_map_t       *map,
                 hb_codepoint_t  key,
                 hb_codepoint_t  value);
```
```rust
pub fn hb_map_set(map: *mut hb_map_t, key: hb_codepoint_t, value: hb_codepoint_t);
```

Stores `value` under `key`, replacing whatever was there. Returns nothing: an
insertion that runs out of memory is reported only by
`hb_map_allocation_successful`, and a set on the singleton empty map is
discarded. Since 1.7.7.

#### `hb_map_get`

```c
hb_codepoint_t hb_map_get (const hb_map_t *map,
                           hb_codepoint_t  key);
```
```rust
pub fn hb_map_get(map: *const hb_map_t, key: hb_codepoint_t) -> hb_codepoint_t;
```

Fetches the value stored under `key`, or `HB_MAP_VALUE_INVALID` when the key is
absent. Since `HB_MAP_VALUE_INVALID` is itself a storable value, use
`hb_map_has` when the difference matters. Since 1.7.7.

#### `hb_map_del`

```c
void hb_map_del (hb_map_t       *map,
                 hb_codepoint_t  key);
```
```rust
pub fn hb_map_del(map: *mut hb_map_t, key: hb_codepoint_t);
```

Removes `key` and its value. Deleting a key that is not present is a harmless
no-op. Since 1.7.7.

#### `hb_map_has`

```c
hb_bool_t hb_map_has (const hb_map_t *map,
                      hb_codepoint_t  key);
```
```rust
pub fn hb_map_has(map: *const hb_map_t, key: hb_codepoint_t) -> hb_bool_t;
```

True when `key` has a value in the map. This is the only reliable presence
test. Since 1.7.7.

### Iteration and extraction

#### `hb_map_next`

```c
hb_bool_t hb_map_next (const hb_map_t *map,
                       int *idx,
                       hb_codepoint_t *key,
                       hb_codepoint_t *value);
```
```rust
pub fn hb_map_next(
    map: *const hb_map_t,
    idx: *mut c_int,
    key: *mut hb_codepoint_t,
    value: *mut hb_codepoint_t,
) -> hb_bool_t;
```

Fetches the next key/value pair. `idx` is in/out iterator state owned by the
caller: initialise it to `-1` — the header's comment is "Pass -1 in for idx to
get started" — and pass the same variable back each time. `key` and `value` are
out-parameters written only when the call returns true; when it returns false
the iteration is over and `idx` has been reset to `-1`, so the same variable can
start a fresh pass.

The order in which pairs come back is undefined, and modifying the map part-way
through an iteration is undefined behaviour. The header does not say whether
`key` or `value` may be null, so assume both are required. Since 7.0.0.

#### `hb_map_keys`

```c
void hb_map_keys (const hb_map_t *map,
                  hb_set_t *keys);
```
```rust
pub fn hb_map_keys(map: *const hb_map_t, keys: *mut hb_set_t);
```

Adds every key of `map` to the set `keys`. Note "adds": the set is not cleared
first, so accumulating from several maps is the natural usage and a single-map
snapshot needs `hb_set_clear` beforehand. `hb_set_t` belongs to `hb-set.h`,
which this header includes; in Rust it comes from the `set` module. Since 7.0.0.

#### `hb_map_values`

```c
void hb_map_values (const hb_map_t *map,
                    hb_set_t *values);
```
```rust
pub fn hb_map_values(map: *const hb_map_t, values: *mut hb_set_t);
```

Adds every value of `map` to the set `values`, with the same additive
semantics. Because a set stores each element once, duplicate values collapse and
the resulting population can be smaller than the map's — this is, incidentally,
a cheap way to ask whether a mapping is injective. Since 7.0.0.

## Usage notes

### Error handling is out-of-band

`hb_map_set`, `hb_map_del`, `hb_map_clear`, and `hb_map_update` all return
`void`. The only way to learn that a map ran out of memory is
`hb_map_allocation_successful`, and because the failure latches, one check after
a batch of insertions is enough:

```c
hb_map_t *m = hb_map_create ();
for (unsigned i = 0; i < n; i++)
  hb_map_set (m, gids[i], i);
if (!hb_map_allocation_successful (m))
  { /* out of memory: the map is incomplete */ }
```

The same check catches the case where `hb_map_create` fell back to the singleton
empty map, so it covers both failure modes at once.

### The sentinel is a real value

```c
hb_map_set (m, 42, HB_MAP_VALUE_INVALID);
hb_map_get (m, 42);   /* HB_MAP_VALUE_INVALID */
hb_map_get (m, 43);   /* HB_MAP_VALUE_INVALID, too */
hb_map_has (m, 42);   /* true  */
hb_map_has (m, 43);   /* false */
```

If your value domain can include `0xFFFFFFFF`, every lookup needs `hb_map_has`
first. If it cannot, `hb_map_get` alone is fine and is one hash lookup cheaper.

### Iterating

```rust
let mut idx: c_int = -1;
let mut key: hb_codepoint_t = 0;
let mut value: hb_codepoint_t = 0;
// SAFETY: `map` is a live map; `idx`, `key`, and `value` are valid locals.
while unsafe { hb_map_next(map, &mut idx, &mut key, &mut value) } != 0 {
    // `key` and `value` hold the current pair.
}
```

Three things to keep straight: `idx` must start at `-1` and must not be touched
between calls; the order is undefined, so never rely on it for reproducible
output (sort the keys yourself, or extract them into an `hb_set_t`, which *is*
ordered); and mutating the map inside the loop is undefined behaviour, so
collect the keys you want to change and apply the changes after the loop.

### Nullability and thread safety

The header declares nothing about null pointers, so treat every `hb_map_t *`
parameter as required. The wider HarfBuzz object convention — visible in
`hb-object.hh` in the vendored sources — is that `*_reference` and `*_destroy`
ignore null while accessors dereference unconditionally, but that is an
implementation detail of the version compiled here, not a documented guarantee
of this header.

Likewise the header says nothing about threading. HarfBuzz's general rule
applies: reference counting is atomic, so handing a reference to another thread
is fine, but the map contents are not internally synchronised. Concurrent
`hb_map_set`/`hb_map_del` on the same map, or a read racing a write, needs your
own lock. Concurrent reads of a map nobody is writing are fine.

### Relationship to `hb_set_t`

Maps and sets are separate objects with parallel APIs, and this header is the
only bridge between them: `hb_map_keys` and `hb_map_values` project a map into a
set. There is no reverse constructor — building a map from a set means iterating
the set and calling `hb_map_set` yourself.
