# Version

Transcribed from `hb-version.h`. Rust module: `harfbuzz_sys::version`, glob
re-exported at the crate root.

## Overview

`hb-version.h` is the smallest public header in HarfBuzz, and the only one that
declares no objects at all. Its whole job is to answer one question — *which
HarfBuzz am I talking to?* — twice over: once at compile time, through
preprocessor macros baked into the header you compiled against, and once at run
time, through three functions that report what the linked library actually is.

That duplication is the entire point. HarfBuzz ships as a shared library on most
platforms, and its headers and its `.so`/`.dylib`/`.dll` can come from different
places and different releases. A program compiled against HarfBuzz 14.3.0 can
easily find itself running against 8.5.0 on a user's machine. The macros
(`HB_VERSION_MAJOR`, `HB_VERSION_MINOR`, `HB_VERSION_MICRO`,
`HB_VERSION_STRING`, `HB_VERSION_ATLEAST`) describe the headers. The functions
(`hb_version`, `hb_version_string`, `hb_version_atleast`) describe the library.
Use the macros to decide what to *compile* — whether a newer API even exists to
call — and the functions to decide what to *execute*, or to print in a bug
report.

HarfBuzz's versioning is conventional three-component semantic-ish versioning:
major, minor, micro. The project treats its public C API and ABI as a hard
constraint, so new releases add symbols but do not remove or change them. In
practice that means a version test is nearly always an *at-least* test: "is this
new enough to have the function I want?" Both the macro and the function encode
this directly, comparing `major * 10000 + minor * 100 + micro` against the same
expression for the available version. Note the implication of that packing: a
minor or micro component of 100 or more would collide with the next component
up. HarfBuzz has never come close, but the arithmetic is not a general-purpose
version comparator.

Nothing here allocates, nothing here has a lifecycle, and nothing here needs to
be destroyed. There are no objects, no reference counts, and no user data. This
header does not depend on any HarfBuzz object type; it includes `hb-common.h`
only for `hb_bool_t` and the `HB_EXTERN`/`HB_BEGIN_DECLS` plumbing.

One caveat specific to this crate: `harfbuzz-sys` vendors the HarfBuzz sources
as a git submodule and compiles them with `build.rs`. The headers and the
library therefore come from the same tree by construction, and the compile-time
constants and the run-time functions cannot disagree. The distinction above
still matters for anyone reading HarfBuzz's own documentation, or linking this
crate against a system HarfBuzz, so both halves are transcribed faithfully.

## Types

This header declares no types of its own.

`hb_bool_t` — the return type of `hb_version_atleast` — is declared in
`hb-common.h` and lives in `crate::common`. It is a C `int`: zero is false,
anything else is true.

## Constants

| Rust constant        | Rust type | C value    | Meaning                                          |
| -------------------- | --------- | ---------- | ------------------------------------------------ |
| `HB_VERSION_MAJOR`   | `c_uint`  | `14`       | Major component of the compile-time version.     |
| `HB_VERSION_MINOR`   | `c_uint`  | `3`        | Minor component of the compile-time version.     |
| `HB_VERSION_MICRO`   | `c_uint`  | `0`        | Micro component of the compile-time version.     |
| `HB_VERSION_STRING`  | `&CStr`   | `"14.3.0"` | The compile-time version as one string.          |

Two transcription judgement calls are worth stating plainly:

* The three numeric macros are bare integer literals in C, so their C type is
  `int`. They are emitted as `c_uint` here because every function in this
  header that consumes or produces a version component uses `unsigned int`, and
  a `c_uint` constant can be passed to `hb_version_atleast` without a cast.

* `HB_VERSION_STRING` is a C string literal — a NUL-terminated `const char[7]`.
  It is emitted as `&'static CStr` (`c"14.3.0"`) rather than `&str`, so that it
  can be passed to a C API directly. Call `.to_str()` (infallible in practice
  for this value, but it still returns a `Result`) or `.to_string_lossy()` for
  a Rust string.

These constants are *not* patched by `build.rs`; they are transcribed from the
vendored header and will change when the submodule is updated.

## Functions

### Compile-time test

#### `HB_VERSION_ATLEAST`

```c
#define HB_VERSION_ATLEAST(major,minor,micro) \
	((major)*10000+(minor)*100+(micro) <= \
	 HB_VERSION_MAJOR*10000+HB_VERSION_MINOR*100+HB_VERSION_MICRO)
```

```rust
#[inline]
pub const fn HB_VERSION_ATLEAST(major: c_uint, minor: c_uint, micro: c_uint) -> bool;
```

Tests the *header* version against a minimum, as three integer components.
Evaluates to true when the compile-time version is greater than or equal to the
requested one.

The C form is a function-like macro, so it is expressible as a `const fn` and
is transcribed as one. It can be used anywhere a `const` expression is
accepted, including in `const` items and array lengths, though it cannot be
used in `#[cfg(...)]` — Rust's `cfg` predicates are not expression-evaluated.
Code that wants conditional compilation on a HarfBuzz version must drive it
from a Cargo feature or a `build.rs`-emitted `cargo::rustc-cfg`, not from this
function.

The Rust version computes in `u64` rather than `c_uint`, so that a nonsensical
argument cannot overflow. For every version number HarfBuzz has ever shipped
the result is identical to the C macro's.

The macro takes no address, allocates nothing, and has no failure mode.

### Run-time queries

#### `hb_version`

```c
void hb_version (unsigned int *major,
                 unsigned int *minor,
                 unsigned int *micro);
```

```rust
pub fn hb_version(major: *mut c_uint, minor: *mut c_uint, micro: *mut c_uint);
```

Writes the three components of the *library* version into the three
out-parameters. Returns nothing, and has no failure mode.

Ownership: none — the caller supplies the storage, HarfBuzz only writes to it.

Nullability: the header does not annotate these parameters, and does not say
whether a null pointer is tolerated for a component the caller does not care
about. Treat all three as required and pass three real addresses; that is the
only usage the header documents.

#### `hb_version_string`

```c
const char *hb_version_string (void);
```

```rust
pub fn hb_version_string() -> *const c_char;
```

Returns the *library* version as a NUL-terminated string with three
components, for example `"14.3.0"`.

Ownership: the returned pointer belongs to HarfBuzz. Do not free it, and do not
write through it. It is a pointer to static storage inside the library and
stays valid for as long as the library is loaded, so it can be cached freely.

Nullability: the header does not document a failure mode, and there is none —
this always returns a valid string.

Because the pointer is static and never freed, wrapping it is a matter of
`CStr::from_ptr` and nothing more; no copy is required unless you want an owned
`String`.

#### `hb_version_atleast`

```c
hb_bool_t hb_version_atleast (unsigned int major,
                              unsigned int minor,
                              unsigned int micro);
```

```rust
pub fn hb_version_atleast(major: c_uint, minor: c_uint, micro: c_uint) -> hb_bool_t;
```

Tests the *library* version against a minimum, as three integer components.
Returns non-zero when the library's version is greater than or equal to the
requested one, and zero otherwise. This is the run-time twin of
`HB_VERSION_ATLEAST`, and uses the same packed comparison.

It has no failure mode distinct from its answer: a zero return means "older
than requested", never "could not determine".

## Usage notes

### Threading

All three functions are pure reads of constant data. They are safe to call from
any thread, at any time, concurrently, including before any other HarfBuzz call
and after the last one. They take no locks and touch no global mutable state —
unlike, say, `hb_language_get_default`, which caches on first use and is
documented as not thread-safe. Nothing in this header needs that caveat.

### Compile-time versus run-time, concretely

The classic mistake is to guard a call with the macro when you meant the
function, or the reverse. The rule is mechanical:

* If the question is *"does this symbol exist to link against?"*, the answer is
  a compile-time question. Use `HB_VERSION_ATLEAST`.
* If the question is *"will the library I actually loaded behave the way I
  expect?"*, the answer is a run-time question. Use `hb_version_atleast`.

A program that dynamically links HarfBuzz frequently needs both: the macro to
decide whether the newer code path can be *compiled*, and the function to
decide whether it can be *taken*.

```c
#if HB_VERSION_ATLEAST(7, 0, 0)
  if (hb_version_atleast (7, 0, 0))
    use_the_new_thing ();
  else
#endif
    use_the_old_thing ();
```

In `harfbuzz-sys` the outer guard is unnecessary — the vendored sources are
compiled into the crate, so header and library are the same release — but the
pattern is worth recognising when reading upstream code or C examples.

### Reading the run-time version from Rust

```rust
use core::ffi::CStr;
use harfbuzz_sys::{hb_version, hb_version_string, HB_VERSION_STRING};

// As a string. The pointer is static, so the CStr borrow is effectively
// 'static and the unsafe block is limited to the dereference.
let s = unsafe { CStr::from_ptr(hb_version_string()) };
assert_eq!(s, HB_VERSION_STRING);

// As three numbers.
let (mut major, mut minor, mut micro) = (0, 0, 0);
unsafe { hb_version(&mut major, &mut minor, &mut micro) };
assert_eq!((major, minor, micro), (14, 3, 0));
```

Both assertions hold for this crate specifically, because it builds the
HarfBuzz it declares. They would not be safe assumptions against a system
HarfBuzz.

### Don't parse the version string

`hb_version_string` is for display: log lines, `--version` output, bug reports.
Anything that needs to *decide* something should call `hb_version_atleast` or
`hb_version` and compare integers. The header makes no promise about the
string's format beyond "three components", and splitting on `.` is a
needlessly fragile way to learn what two dedicated functions will tell you
directly.

### Don't compare versions by hand

The packed `major * 10000 + minor * 100 + micro` form is an implementation
detail of the comparison, not a version encoding you should reproduce. Use the
provided at-least tests. If you genuinely need an ordering, extract the three
components with `hb_version` and compare the tuple — in Rust, `(major, minor,
micro)` compares lexicographically for free, which is both clearer and correct
for components of 100 or more.

### Version-gating in a Cargo build

`HB_VERSION_ATLEAST` is a `const fn`, which means it works in `const` contexts
but not in `#[cfg]`. If you need whole items to appear and disappear based on
the HarfBuzz version, the options are, in order of preference: a Cargo feature
that the user opts into; a `build.rs` that emits `cargo::rustc-cfg=...` after
inspecting the vendored version; or, at the call site, a plain `if
HB_VERSION_ATLEAST(..)` — which is fine as long as both branches compile, and
they will not if the newer branch names a symbol that is absent from the
library you are linking.
