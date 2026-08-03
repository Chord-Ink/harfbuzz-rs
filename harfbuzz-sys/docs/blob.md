# Blobs

Header: `hb-blob.h` — Rust module: `harfbuzz_sys::blob` (glob re-exported at the crate root).

## Overview

A **blob** is HarfBuzz's container for a chunk of binary data. It pairs a
pointer and a byte length with a reference count, a memory mode, and an
optional destroy callback, so that a block of memory can be handed back and
forth between a client program and HarfBuzz without either side having to guess
who is responsible for freeing it.

Blobs are the entry point to almost everything else in HarfBuzz. You wrap a
font file in a blob and hand it to `hb_face_create` to get a face; you ask a
face for one of its tables and get another blob back. They are also used for
any other opaque binary payload the API needs to pass around — subsetting
output, `CBDT` bitmaps, `SVG ` documents, and so on. Nothing about a blob is
font-specific: it is just bytes plus a lifetime policy.

The lifetime policy is the interesting part, and it has two independent halves.
The first is the **memory mode** (`hb_memory_mode_t`), fixed at creation time,
which tells HarfBuzz whether it may copy the data, whether it may write to it,
and whether the client promises never to touch it again. The second is the
**reference count**: every blob starts with a count of one, `hb_blob_reference`
raises it, `hb_blob_destroy` lowers it, and when it hits zero the blob is freed
and the destroy callback supplied at creation is invoked on the user data. The
destroy callback is how the client learns that HarfBuzz is finally done with
the underlying bytes — that is the moment to `munmap` the file, drop the
`Vec<u8>`, or release the `Arc`.

Blobs also carry the two facilities every HarfBuzz object has: an
**immutability flag** (`hb_blob_make_immutable` / `hb_blob_is_immutable`), which
permanently forbids obtaining a writable pointer to the data, and a
**user-data table** keyed by the address of an `hb_user_data_key_t`, which lets
a client attach arbitrary side data to a blob and have it cleaned up
automatically.

Finally, note the API's two error conventions. The original functions
(`hb_blob_create`, `hb_blob_create_from_file`, `hb_blob_create_sub_blob`) never
return null: on failure they return the singleton *empty blob*, a permanently
valid zero-length object. That makes them convenient but makes errors invisible
unless you compare against `hb_blob_get_empty()` or check the length. The
later `_or_fail` variants (`hb_blob_create_or_fail`,
`hb_blob_create_from_file_or_fail`, `hb_blob_copy_writable_or_fail`) return
null on failure instead, and are what you want in a Rust wrapper that intends
to surface errors.

## Types

### `hb_memory_mode_t`

The memory mode negotiates ownership of the buffer passed to
`hb_blob_create`. In C it is an unnamed `enum` with four enumerators and no
sentinel value; the largest is `3`, so it fits in an `int` and is transcribed as
`pub type hb_memory_mode_t = core::ffi::c_int;` plus four constants.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_MEMORY_MODE_DUPLICATE` | 0 | HarfBuzz immediately makes its own copy of the data. |
| `HB_MEMORY_MODE_READONLY` | 1 | The client will never modify the data, and HarfBuzz will never modify the data. |
| `HB_MEMORY_MODE_WRITABLE` | 2 | The client made this copy solely for HarfBuzz, so HarfBuzz may modify it. |
| `HB_MEMORY_MODE_READONLY_MAY_MAKE_WRITABLE` | 3 | Read-only, but HarfBuzz may try to make the pages writable in place. |

The header's own guidance, verbatim in substance:

- In no case may the client modify memory it has passed to HarfBuzz in a blob.
  If there is any such possibility, use `HB_MEMORY_MODE_DUPLICATE` so HarfBuzz
  copies immediately.
- Use `HB_MEMORY_MODE_READONLY` otherwise, "unless you really really really
  know what you are doing".
- `HB_MEMORY_MODE_WRITABLE` is appropriate if you really made a copy of the data
  solely to pass to HarfBuzz, and are doing that just once — no reuse.
- If the font is `mmap`ed it is okay to use
  `HB_MEMORY_MODE_READONLY_MAY_MAKE_WRITABLE`, "however, using that mode
  correctly is very tricky. Use `HB_MEMORY_MODE_READONLY` instead."

`HB_MEMORY_MODE_DUPLICATE` is not stored: the implementation converts it to
`HB_MEMORY_MODE_READONLY` and then immediately forces a writable copy, so a
duplicate-mode blob ends up owning a private, writable buffer allocated with
`hb_malloc`.

### `hb_blob_t`

```c
typedef struct hb_blob_t hb_blob_t;
```

```rust
crate::opaque_handle! { hb_blob_t }
```

An opaque, reference-counted object wrapping a chunk of binary data. The struct
has no visible body in the public header, so it exists only behind a pointer.
In Rust it is a zero-sized `#[repr(C)]` handle that cannot be constructed,
copied, or sent between threads by accident — you always hold `*mut hb_blob_t`.

## Functions

### Creation

#### `hb_blob_create`

```c
hb_blob_t *hb_blob_create (const char *data, unsigned int length,
                           hb_memory_mode_t mode, void *user_data,
                           hb_destroy_func_t destroy);
```

```rust
pub fn hb_blob_create(
    data: *const c_char,
    length: c_uint,
    mode: hb_memory_mode_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
) -> *mut hb_blob_t;
```

Creates a blob wrapping `length` bytes at `data`, with the given memory mode.
`destroy` may be null; when non-null it is called with `user_data` once
HarfBuzz no longer needs the data.

**Never returns null.** On failure — including the case `length == 0` — it
returns the singleton empty blob. `destroy` is still called in those cases, so
the callee always assumes ownership of `user_data` regardless of the outcome.
Release the result with `hb_blob_destroy`. Since HarfBuzz 0.9.2.

#### `hb_blob_create_or_fail`

```c
hb_blob_t *hb_blob_create_or_fail (const char *data, unsigned int length,
                                   hb_memory_mode_t mode, void *user_data,
                                   hb_destroy_func_t destroy);
```

```rust
pub fn hb_blob_create_or_fail(
    data: *const c_char,
    length: c_uint,
    mode: hb_memory_mode_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
) -> *mut hb_blob_t;
```

Same as `hb_blob_create`, but returns **null** on failure instead of the empty
blob. Two behavioural differences beyond that:

- A zero `length` is not an error here: you get a freshly allocated empty blob,
  not the shared singleton. Code that compares a blob pointer against
  `hb_blob_get_empty()` will therefore behave differently between the two
  constructors.
- `destroy` is still called with `user_data` when creation fails, so ownership
  of `user_data` transfers unconditionally.

Release the result with `hb_blob_destroy`. Since HarfBuzz 2.8.2.

#### `hb_blob_create_from_file`

```c
hb_blob_t *hb_blob_create_from_file (const char *file_name);
```

```rust
pub fn hb_blob_create_from_file(file_name: *const c_char) -> *mut hb_blob_t;
```

Reads the named binary font file into a blob. The filename is passed straight
to the system on every platform except Windows, where it is interpreted as
UTF-8 and falls back to the system code page only if it is not valid UTF-8.

**Never returns null**: on any failure — missing file, permission denied,
allocation failure — you get the singleton empty blob, and there is no error
detail. Release with `hb_blob_destroy`. Since HarfBuzz 1.7.7.

#### `hb_blob_create_from_file_or_fail`

```c
hb_blob_t *hb_blob_create_from_file_or_fail (const char *file_name);
```

```rust
pub fn hb_blob_create_from_file_or_fail(file_name: *const c_char) -> *mut hb_blob_t;
```

As above, but returns **null** on failure. This is the one to bind in a safe
wrapper, since it is the only file-loading entry point that distinguishes "empty
file" from "could not read the file". Release with `hb_blob_destroy`.
Since HarfBuzz 2.8.2.

### Derived blobs

#### `hb_blob_create_sub_blob`

```c
hb_blob_t *hb_blob_create_sub_blob (hb_blob_t *parent,
                                    unsigned int offset, unsigned int length);
```

```rust
pub fn hb_blob_create_sub_blob(
    parent: *mut hb_blob_t,
    offset: c_uint,
    length: c_uint,
) -> *mut hb_blob_t;
```

Returns a blob representing the range of bytes `[offset, offset + length)`
within `parent`. The sub-blob is **always** created with
`HB_MEMORY_MODE_READONLY`, deliberately: even when the parent is writable, a
sub-blob's user must not be able to modify the parent's data, because that data
may be shared among several sub-blobs. The parent data is not expected to
change, and modifying it is undefined behaviour.

Side effects worth knowing:

- **`parent` is made immutable** by this call. There is no way to undo that.
- The sub-blob takes a reference on `parent` and drops it when the sub-blob is
  destroyed, so the parent outlives every sub-blob derived from it. No copying
  takes place — the sub-blob points into the parent's buffer.
- `length` is clamped to what actually remains after `offset`.

**Never returns null**: you get the empty blob if something failed, if `length`
is zero, if `parent` is null, or if `offset` is at or beyond the end of the
parent's data. Release with `hb_blob_destroy`. Since HarfBuzz 0.9.2.

#### `hb_blob_copy_writable_or_fail`

```c
hb_blob_t *hb_blob_copy_writable_or_fail (hb_blob_t *blob);
```

```rust
pub fn hb_blob_copy_writable_or_fail(blob: *mut hb_blob_t) -> *mut hb_blob_t;
```

Makes a writable copy of `blob` — a fresh blob owning its own duplicate of the
bytes, independent of the original and of the original's immutability flag.
Returns null if allocation failed. Release with `hb_blob_destroy`.
Since HarfBuzz 1.8.0.

The header does not say whether `blob` may be null; the implementation
dereferences it immediately, so treat null as forbidden.

### The empty blob

#### `hb_blob_get_empty`

```c
hb_blob_t *hb_blob_get_empty (void);
```

```rust
pub fn hb_blob_get_empty() -> *mut hb_blob_t;
```

Returns the singleton empty blob: a permanently valid, zero-length, immutable
blob that the error paths of the non-`_or_fail` constructors also return. It is
never null. Upstream's annotation marks the return as `transfer full`, so the
conventional thing is to treat it like any other blob and pass it to
`hb_blob_destroy` when done; HarfBuzz's shared null objects are inert, so
referencing and destroying them are cheap no-ops. Since HarfBuzz 0.9.2.

### Reference counting

#### `hb_blob_reference`

```c
hb_blob_t *hb_blob_reference (hb_blob_t *blob);
```

```rust
pub fn hb_blob_reference(blob: *mut hb_blob_t) -> *mut hb_blob_t;
```

Increases the reference count on `blob` and returns the same pointer, which
makes it convenient to use inline when handing a blob to something that will
take ownership. Every call must be matched by a `hb_blob_destroy`.
Since HarfBuzz 0.9.2.

#### `hb_blob_destroy`

```c
void hb_blob_destroy (hb_blob_t *blob);
```

```rust
pub fn hb_blob_destroy(blob: *mut hb_blob_t);
```

Decreases the reference count on `blob`; when it reaches zero the blob is
destroyed and all of its memory freed, and the destroy callback the blob was
created with is invoked on its user data if it has not been called already.
Returns nothing — there is no way to observe whether the object actually went
away. Since HarfBuzz 0.9.2.

### User data

#### `hb_blob_set_user_data`

```c
hb_bool_t hb_blob_set_user_data (hb_blob_t *blob, hb_user_data_key_t *key,
                                 void *data, hb_destroy_func_t destroy,
                                 hb_bool_t replace);
```

```rust
pub fn hb_blob_set_user_data(
    blob: *mut hb_blob_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

Attaches a key/data pair to the blob. HarfBuzz uses the *address* of `key`, not
its contents, so the key object must outlive the blob — a `static` is the usual
choice. `destroy` may be null; when non-null it is called with `data` when the
blob is destroyed or when the entry is replaced. `replace` selects whether an
existing entry stored under the same key is overwritten.

Returns true on success, false otherwise (allocation failure, or a non-replace
call against an existing key). Since HarfBuzz 0.9.2.

#### `hb_blob_get_user_data`

```c
void *hb_blob_get_user_data (const hb_blob_t *blob, hb_user_data_key_t *key);
```

```rust
pub fn hb_blob_get_user_data(
    blob: *const hb_blob_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

Fetches the data previously attached under `key`. Note the `const` blob
parameter — this is one of the few blob functions that takes a `*const
hb_blob_t`. Ownership is not transferred: the returned pointer belongs to
whoever stored it and must not be freed by the caller. Returns null when no
entry is present for that key. Since HarfBuzz 0.9.2.

### Immutability

#### `hb_blob_make_immutable`

```c
void hb_blob_make_immutable (hb_blob_t *blob);
```

```rust
pub fn hb_blob_make_immutable(blob: *mut hb_blob_t);
```

Marks the blob immutable. This is one-way: there is no
`hb_blob_make_mutable`. After this, `hb_blob_get_data_writable` always fails.
Note that `hb_blob_create_sub_blob` and face creation apply this to their input
implicitly. Since HarfBuzz 0.9.2.

#### `hb_blob_is_immutable`

```c
hb_bool_t hb_blob_is_immutable (hb_blob_t *blob);
```

```rust
pub fn hb_blob_is_immutable(blob: *mut hb_blob_t) -> hb_bool_t;
```

Tests whether a blob is immutable; true if it is. Since HarfBuzz 0.9.2.

### Data access

#### `hb_blob_get_length`

```c
unsigned int hb_blob_get_length (hb_blob_t *blob);
```

```rust
pub fn hb_blob_get_length(blob: *mut hb_blob_t) -> c_uint;
```

Fetches the length of the blob's data in bytes. Since HarfBuzz 0.9.2.

#### `hb_blob_get_data`

```c
const char *hb_blob_get_data (hb_blob_t *blob, unsigned int *length);
```

```rust
pub fn hb_blob_get_data(blob: *mut hb_blob_t, length: *mut c_uint) -> *const c_char;
```

Fetches the blob's bytes. `length` is an out-parameter that receives the byte
count; it may be null if you do not want it. The returned pointer is **nullable**
per upstream's annotation, and no ownership is transferred — the bytes belong to
the blob and stay valid only as long as you hold a reference to it. Do not free
them. Since HarfBuzz 0.9.2.

Note that `char` is signed on most platforms; in Rust the return is
`*const c_char`, so building a byte slice means casting to `*const u8`.

#### `hb_blob_get_data_writable`

```c
char *hb_blob_get_data_writable (hb_blob_t *blob, unsigned int *length);
```

```rust
pub fn hb_blob_get_data_writable(blob: *mut hb_blob_t, length: *mut c_uint) -> *mut c_char;
```

Tries to make the blob's data writable — possibly by copying it, or by
re-protecting its pages when the mode is
`HB_MEMORY_MODE_READONLY_MAY_MAKE_WRITABLE` — and returns a pointer to it.

Fails, returning null, if the blob has been made immutable or if memory
allocation fails; `length`, when non-null, is set to zero in that case and to
the byte count on success. Ownership is not transferred: do not free the
pointer, and treat it as invalid once you drop your reference to the blob.
Since HarfBuzz 0.9.2.

A successful call can **replace the blob's buffer**: if a copy was needed,
HarfBuzz allocates a new buffer, invokes the original destroy callback, and
switches the blob's mode to `HB_MEMORY_MODE_WRITABLE`. Any pointer previously
returned by `hb_blob_get_data` is therefore potentially dangling afterwards.

## Usage notes

### The two error conventions

Prefer the `_or_fail` variants everywhere you can. The older constructors fold
every failure into the empty blob, which means a typo in a filename produces a
zero-length font rather than an error, and the mistake only surfaces much later
as a face with no tables. If you must use the older ones, the check is either
`hb_blob_get_length(blob) == 0` or `blob == hb_blob_get_empty()` — but note the
latter does not work with `hb_blob_create_or_fail`, which allocates a *fresh*
empty blob for zero-length input rather than returning the singleton.

### The destroy callback owns `user_data` unconditionally

Both `hb_blob_create` and `hb_blob_create_or_fail` call `destroy(user_data)`
themselves when they bail out early. So once you have called either function,
`user_data` is HarfBuzz's problem in every code path — you must not free it
yourself on the error path, and you must not assume the callback runs only at
blob destruction time. Concretely, a Rust wrapper that leaks a `Box` into
`user_data` before the call is correct; one that frees the `Box` when the call
returns null is a double free.

### Size limit

The header says nothing about a maximum size, but the implementation of
`hb_blob_create_or_fail` rejects any `length` of `2^31` or more, and the
non-`mmap` file-reading fallback refuses files beyond roughly 512 MiB. The
`mmap` path has no such limit. This is implementation behaviour, not a
documented guarantee.

### Nullability of the blob argument

The header does not annotate any parameter. From the implementation:
`hb_blob_create_sub_blob` explicitly tolerates a null `parent` (returning the
empty blob), and `hb_blob_destroy` tolerates null the way `free` does. The
accessors dereference their blob argument directly, so treat null as forbidden
for `hb_blob_get_length`, `hb_blob_get_data`, `hb_blob_get_data_writable`, and
`hb_blob_copy_writable_or_fail`. Passing the empty blob is always safe.

### Immutability is contagious and permanent

Anything that derives from a blob tends to freeze it: `hb_blob_create_sub_blob`
makes its parent immutable, and creating a face from a blob does the same. Once
frozen, `hb_blob_get_data_writable` will never succeed on that blob again. If
you need a writable view afterwards, take a copy with
`hb_blob_copy_writable_or_fail`.

### Threading

The header is silent on thread safety. HarfBuzz's object reference counts are
atomic in a normally configured build, so `hb_blob_reference` and
`hb_blob_destroy` may be called from multiple threads; mutating operations
(`hb_blob_set_user_data`, `hb_blob_get_data_writable`) and the immutable flag
are not documented as synchronised, and should be confined to one thread or
guarded externally. The safest discipline is the usual one: make a blob
immutable before sharing it.

### Worked example: wrapping an owned Rust buffer

The idiomatic pattern is to leak the allocation into `user_data` and free it
from the destroy callback, so that HarfBuzz's reference count — not Rust's
scope — decides when the bytes go away.

```rust
use core::ffi::{c_char, c_void};
use harfbuzz_sys::{
    hb_blob_create_or_fail, hb_blob_destroy, hb_blob_t, HB_MEMORY_MODE_READONLY,
};

unsafe extern "C" fn drop_boxed_slice(user_data: *mut c_void) {
    drop(unsafe { Box::from_raw(user_data as *mut Box<[u8]>) });
}

fn blob_from_bytes(bytes: Box<[u8]>) -> *mut hb_blob_t {
    let len = bytes.len() as u32;
    let ptr = bytes.as_ptr() as *const c_char;

    // Move the Box itself onto the heap so the callback can reconstruct it.
    let owner = Box::into_raw(Box::new(bytes)) as *mut c_void;

    // On failure HarfBuzz calls drop_boxed_slice for us and returns null,
    // so there is nothing to clean up here.
    unsafe {
        hb_blob_create_or_fail(
            ptr,
            len,
            HB_MEMORY_MODE_READONLY,
            owner,
            Some(drop_boxed_slice),
        )
    }
}

unsafe fn release(blob: *mut hb_blob_t) {
    unsafe { hb_blob_destroy(blob) };
}
```

### Worked example: reading a blob's bytes

```rust
use core::ffi::c_uint;
use harfbuzz_sys::{hb_blob_get_data, hb_blob_t};

unsafe fn blob_bytes<'a>(blob: *mut hb_blob_t) -> &'a [u8] {
    let mut len: c_uint = 0;
    let data = unsafe { hb_blob_get_data(blob, &mut len) };
    if data.is_null() || len == 0 {
        return &[];
    }
    // Valid only while a reference to `blob` is held.
    unsafe { core::slice::from_raw_parts(data as *const u8, len as usize) }
}
```

### Choosing a memory mode, in practice

- Bytes you own and will keep using elsewhere → `HB_MEMORY_MODE_DUPLICATE`.
- Bytes that live for the whole program, or that you own and will keep alive
  behind a destroy callback → `HB_MEMORY_MODE_READONLY`.
- A buffer allocated purely to hand over, once → `HB_MEMORY_MODE_WRITABLE`
  (pair it with a destroy callback that frees the buffer).
- `mmap`ed font files → `HB_MEMORY_MODE_READONLY` unless you have measured a
  reason to prefer `HB_MEMORY_MODE_READONLY_MAY_MAKE_WRITABLE`, which the header
  itself warns is very tricky to use correctly.
