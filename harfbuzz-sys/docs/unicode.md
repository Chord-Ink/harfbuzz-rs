# Unicode functions

Header: `hb-unicode.h` — Rust module: `harfbuzz_sys::unicode` (glob re-exported
at the crate root). Upstream gtk-doc section: `hb-unicode`, short description
*"Unicode character property access"*.

## Overview

Before HarfBuzz can shape anything it needs to know a handful of facts about
each character: what general category it is in, what its canonical combining
class is, which script it belongs to, what its mirrored form is, and how it
composes and decomposes under canonical equivalence. An **`hb_unicode_funcs_t`**
is the object that answers those questions. It is a small vtable — six function
pointers, each with its own `user_data` and destroy notifier — wrapped in
HarfBuzz's usual reference-counted, immutable-flagged, user-data-carrying
object.

Most programs never touch this API directly. Every `hb_buffer_t` starts out
using `hb_unicode_funcs_get_default()`, which is whatever implementation the
library was built with: HarfBuzz's own bundled Unicode Character Database
tables (`hb-ucd.cc`) normally, falling back to GLib and then ICU if UCD support
was compiled out. That default is a process-wide singleton, is effectively
immutable, and is good enough for essentially all shaping. You reach for this
header when you have a Unicode data source of your own — an application that
already links a UCD-derived library and wants one copy of the tables, an
embedded target that ships a cut-down table, or a test harness that wants to lie
about a character's script.

Supplying your own data is a three-step ritual: create a structure with
`hb_unicode_funcs_create()`, install the callbacks you care about with the
`hb_unicode_funcs_set_*_func()` setters, and hand the result to
`hb_buffer_set_unicode_funcs()`. The **parent** argument to the constructor is
what makes partial overrides cheap: the new structure starts out with a *copy*
of the parent's function table, so any method you do not install keeps behaving
exactly as the parent's did. Pass `hb_unicode_funcs_get_default()` as the parent
and override only `script`, and everything else still comes from the UCD tables.
Creating a child also takes a reference on the parent and makes the parent
immutable.

Immutability is the other half of the safety story. `hb_unicode_funcs_make_immutable()`
permanently freezes a structure; after that the setters silently do nothing —
they still run the `destroy` notifier on the `user_data` they were handed, so
nothing leaks, but they report no error. That silence is the single biggest
trap in this API: installing a callback on a structure that has already been
used as a parent, or on `hb_unicode_funcs_get_default()`, appears to succeed and
changes nothing.

Two things this API is *not*. It is not a general Unicode library — there is no
case mapping, no word breaking, no normalisation driver, and the
composition/decomposition callbacks handle only the pairwise canonical case that
HarfBuzz's normalizer needs. And it is not where scripts are defined:
`hb_script_t` and the `HB_SCRIPT_*` constants live in `hb-script-list.h`, and
`hb_unicode_script()` merely returns one.

## Types

### `hb_unicode_funcs_t`

```c
typedef struct hb_unicode_funcs_t hb_unicode_funcs_t;
```

```rust
crate::opaque_handle! { hb_unicode_funcs_t }
```

Data type containing a set of virtual methods used for accessing various
Unicode character properties. HarfBuzz provides a default function for each of
the methods; client programs can implement their own replacements for the
individual functions, as needed, and replace the default by calling the setter
for a method.

Opaque and reference counted, so you always hold `*mut hb_unicode_funcs_t`. In
Rust it is a zero-sized `#[repr(C)]` handle that cannot be constructed or
copied by accident. Internally the object holds a parent pointer, six function
pointers, six `user_data` pointers, and six destroy notifiers.

You obtain one from `hb_unicode_funcs_create()` (owned — you must destroy it),
`hb_unicode_funcs_get_default()` (borrowed singleton), `hb_unicode_funcs_get_empty()`
(borrowed singleton), `hb_unicode_funcs_get_parent()` (borrowed), or
`hb_buffer_get_unicode_funcs()` (borrowed).

### `hb_unicode_general_category_t`

Data type for the `General_Category` (`gc`) property from the Unicode Character
Database. In C it is an unnamed-value `enum` with 30 enumerators, no sentinel,
and no explicit values, so the values are simply 0–29; it fits in an `int` and
is transcribed as `pub type hb_unicode_general_category_t = core::ffi::c_int;`
plus 30 constants.

The ordering is HarfBuzz's own — alphabetical by the two-letter UCD abbreviation
within each major class (`C`, `L`, `M`, `N`, `P`, `S`, `Z`) — and matches
neither ICU's `UCharCategory` nor GLib's `GUnicodeType`. Never transmute a value
from another library into this type; map it explicitly.

| Constant | Value | UCD | Meaning |
| --- | ---: | --- | --- |
| `HB_UNICODE_GENERAL_CATEGORY_CONTROL` | 0 | `Cc` | Control characters |
| `HB_UNICODE_GENERAL_CATEGORY_FORMAT` | 1 | `Cf` | Format characters |
| `HB_UNICODE_GENERAL_CATEGORY_UNASSIGNED` | 2 | `Cn` | Unassigned code points |
| `HB_UNICODE_GENERAL_CATEGORY_PRIVATE_USE` | 3 | `Co` | Private-use characters |
| `HB_UNICODE_GENERAL_CATEGORY_SURROGATE` | 4 | `Cs` | Surrogate code points |
| `HB_UNICODE_GENERAL_CATEGORY_LOWERCASE_LETTER` | 5 | `Ll` | Lowercase letters |
| `HB_UNICODE_GENERAL_CATEGORY_MODIFIER_LETTER` | 6 | `Lm` | Modifier letters |
| `HB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER` | 7 | `Lo` | Other letters |
| `HB_UNICODE_GENERAL_CATEGORY_TITLECASE_LETTER` | 8 | `Lt` | Titlecase letters |
| `HB_UNICODE_GENERAL_CATEGORY_UPPERCASE_LETTER` | 9 | `Lu` | Uppercase letters |
| `HB_UNICODE_GENERAL_CATEGORY_SPACING_MARK` | 10 | `Mc` | Spacing combining marks |
| `HB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK` | 11 | `Me` | Enclosing combining marks |
| `HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK` | 12 | `Mn` | Non-spacing combining marks |
| `HB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER` | 13 | `Nd` | Decimal digits |
| `HB_UNICODE_GENERAL_CATEGORY_LETTER_NUMBER` | 14 | `Nl` | Letter-like numbers |
| `HB_UNICODE_GENERAL_CATEGORY_OTHER_NUMBER` | 15 | `No` | Other numbers |
| `HB_UNICODE_GENERAL_CATEGORY_CONNECT_PUNCTUATION` | 16 | `Pc` | Connector punctuation |
| `HB_UNICODE_GENERAL_CATEGORY_DASH_PUNCTUATION` | 17 | `Pd` | Dash punctuation |
| `HB_UNICODE_GENERAL_CATEGORY_CLOSE_PUNCTUATION` | 18 | `Pe` | Closing punctuation |
| `HB_UNICODE_GENERAL_CATEGORY_FINAL_PUNCTUATION` | 19 | `Pf` | Final quotation punctuation |
| `HB_UNICODE_GENERAL_CATEGORY_INITIAL_PUNCTUATION` | 20 | `Pi` | Initial quotation punctuation |
| `HB_UNICODE_GENERAL_CATEGORY_OTHER_PUNCTUATION` | 21 | `Po` | Other punctuation |
| `HB_UNICODE_GENERAL_CATEGORY_OPEN_PUNCTUATION` | 22 | `Ps` | Opening punctuation |
| `HB_UNICODE_GENERAL_CATEGORY_CURRENCY_SYMBOL` | 23 | `Sc` | Currency symbols |
| `HB_UNICODE_GENERAL_CATEGORY_MODIFIER_SYMBOL` | 24 | `Sk` | Modifier symbols |
| `HB_UNICODE_GENERAL_CATEGORY_MATH_SYMBOL` | 25 | `Sm` | Math symbols |
| `HB_UNICODE_GENERAL_CATEGORY_OTHER_SYMBOL` | 26 | `So` | Other symbols |
| `HB_UNICODE_GENERAL_CATEGORY_LINE_SEPARATOR` | 27 | `Zl` | Line separators |
| `HB_UNICODE_GENERAL_CATEGORY_PARAGRAPH_SEPARATOR` | 28 | `Zp` | Paragraph separators |
| `HB_UNICODE_GENERAL_CATEGORY_SPACE_SEPARATOR` | 29 | `Zs` | Space separators |

There is no "invalid" or "unknown" member. The empty structure's stub reports
`HB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER` (`Lo`) for every code point, which is
the closest thing to a neutral answer — it makes every character look like a
plain letter to the shaper.

### `hb_unicode_combining_class_t`

Data type for the `Canonical_Combining_Class` (`ccc`) property from the Unicode
Character Database. In C it is an `enum` with explicit values ranging from 0 to
255; it fits in an `int` and is transcribed as
`pub type hb_unicode_combining_class_t = core::ffi::c_int;` plus constants.

**The constants are not exhaustive.** The header says so explicitly: *"newer
versions of Unicode may add new values. Client programs should be ready to
handle any value in the 0..254 range being returned from
`hb_unicode_combining_class()`."* The named constants cover the classes
HarfBuzz's shapers reason about; everything else is a bare number. This is
exactly why the Rust binding is an integer alias rather than a Rust `enum` —
a `match` on it must have a catch-all arm.

| Constant | Value | Meaning |
| --- | ---: | --- |
| `HB_UNICODE_COMBINING_CLASS_NOT_REORDERED` | 0 | Spacing and enclosing marks; also many vowel and consonant signs, even if non-spacing |
| `HB_UNICODE_COMBINING_CLASS_OVERLAY` | 1 | Marks which overlay a base letter or symbol |
| `HB_UNICODE_COMBINING_CLASS_NUKTA` | 7 | Diacritic nukta marks in Brahmi-derived scripts |
| `HB_UNICODE_COMBINING_CLASS_KANA_VOICING` | 8 | Hiragana/Katakana voicing marks |
| `HB_UNICODE_COMBINING_CLASS_VIRAMA` | 9 | Viramas |
| `HB_UNICODE_COMBINING_CLASS_CCC10` | 10 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC11` | 11 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC12` | 12 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC13` | 13 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC14` | 14 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC15` | 15 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC16` | 16 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC17` | 17 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC18` | 18 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC19` | 19 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC20` | 20 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC21` | 21 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC22` | 22 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC23` | 23 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC24` | 24 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC25` | 25 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC26` | 26 | Hebrew |
| `HB_UNICODE_COMBINING_CLASS_CCC27` | 27 | Arabic |
| `HB_UNICODE_COMBINING_CLASS_CCC28` | 28 | Arabic |
| `HB_UNICODE_COMBINING_CLASS_CCC29` | 29 | Arabic |
| `HB_UNICODE_COMBINING_CLASS_CCC30` | 30 | Arabic |
| `HB_UNICODE_COMBINING_CLASS_CCC31` | 31 | Arabic |
| `HB_UNICODE_COMBINING_CLASS_CCC32` | 32 | Arabic |
| `HB_UNICODE_COMBINING_CLASS_CCC33` | 33 | Arabic |
| `HB_UNICODE_COMBINING_CLASS_CCC34` | 34 | Arabic |
| `HB_UNICODE_COMBINING_CLASS_CCC35` | 35 | Arabic |
| `HB_UNICODE_COMBINING_CLASS_CCC36` | 36 | Syriac |
| `HB_UNICODE_COMBINING_CLASS_CCC84` | 84 | Telugu |
| `HB_UNICODE_COMBINING_CLASS_CCC91` | 91 | Telugu |
| `HB_UNICODE_COMBINING_CLASS_CCC103` | 103 | Thai |
| `HB_UNICODE_COMBINING_CLASS_CCC107` | 107 | Thai |
| `HB_UNICODE_COMBINING_CLASS_CCC118` | 118 | Lao |
| `HB_UNICODE_COMBINING_CLASS_CCC122` | 122 | Lao |
| `HB_UNICODE_COMBINING_CLASS_CCC129` | 129 | Tibetan |
| `HB_UNICODE_COMBINING_CLASS_CCC130` | 130 | Tibetan |
| `HB_UNICODE_COMBINING_CLASS_CCC132` | 132 | Tibetan. Since HarfBuzz 7.2.0 |
| `HB_UNICODE_COMBINING_CLASS_ATTACHED_BELOW_LEFT` | 200 | Marks attached at the bottom left |
| `HB_UNICODE_COMBINING_CLASS_ATTACHED_BELOW` | 202 | Marks attached directly below |
| `HB_UNICODE_COMBINING_CLASS_ATTACHED_ABOVE` | 214 | Marks attached directly above |
| `HB_UNICODE_COMBINING_CLASS_ATTACHED_ABOVE_RIGHT` | 216 | Marks attached at the top right |
| `HB_UNICODE_COMBINING_CLASS_BELOW_LEFT` | 218 | Distinct marks at the bottom left |
| `HB_UNICODE_COMBINING_CLASS_BELOW` | 220 | Distinct marks directly below |
| `HB_UNICODE_COMBINING_CLASS_BELOW_RIGHT` | 222 | Distinct marks at the bottom right |
| `HB_UNICODE_COMBINING_CLASS_LEFT` | 224 | Distinct marks to the left |
| `HB_UNICODE_COMBINING_CLASS_RIGHT` | 226 | Distinct marks to the right |
| `HB_UNICODE_COMBINING_CLASS_ABOVE_LEFT` | 228 | Distinct marks at the top left |
| `HB_UNICODE_COMBINING_CLASS_ABOVE` | 230 | Distinct marks directly above |
| `HB_UNICODE_COMBINING_CLASS_ABOVE_RIGHT` | 232 | Distinct marks at the top right |
| `HB_UNICODE_COMBINING_CLASS_DOUBLE_BELOW` | 233 | Distinct marks subtending two bases |
| `HB_UNICODE_COMBINING_CLASS_DOUBLE_ABOVE` | 234 | Distinct marks extending above two bases |
| `HB_UNICODE_COMBINING_CLASS_IOTA_SUBSCRIPT` | 240 | Greek iota subscript only |
| `HB_UNICODE_COMBINING_CLASS_INVALID` | 255 | Invalid combining class |

`HB_UNICODE_COMBINING_CLASS_INVALID` (255) is a HarfBuzz sentinel, not a Unicode
value — the UCD only assigns classes in 0..=254. It is never returned by
`hb_unicode_combining_class()` for a well-behaved implementation.

### `HB_UNICODE_MAX`

```c
#define HB_UNICODE_MAX 0x10FFFFu
```

```rust
pub const HB_UNICODE_MAX: hb_codepoint_t = 0x10FFFF;
```

Maximum valid Unicode code point. Since HarfBuzz 1.9.0. Note the type
difference: the C macro is `unsigned`, and the Rust constant is typed
`hb_codepoint_t` (= `u32`), which is what the callbacks take.

Nothing in this header range-checks against it — it is provided so that clients
implementing their own callbacks can validate their input, and so that table
sizes can be written down without a magic number.

### Callback typedefs

All six share a shape: they take the `hb_unicode_funcs_t` they were installed
on, the query, and the `user_data` pointer that was passed to their setter. In
Rust every one is an `Option<unsafe extern "C" fn(...)>`, so `None` is the
null function pointer — which is what you pass to a setter to *restore the
parent's implementation*.

None of the six carries a `Since:` annotation upstream; they are as old as the
structure itself.

#### `hb_unicode_general_category_func_t`

```c
typedef hb_unicode_general_category_t (*hb_unicode_general_category_func_t)
    (hb_unicode_funcs_t *ufuncs, hb_codepoint_t unicode, void *user_data);
```

```rust
pub type hb_unicode_general_category_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_unicode_general_category_t,
>;
```

Should retrieve the General Category property for a specified code point.
Returns the `hb_unicode_general_category_t` of `unicode`. There is no failure
signal — return the nearest sensible category (the built-in stub uses
`HB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER`).

#### `hb_unicode_combining_class_func_t`

```c
typedef hb_unicode_combining_class_t (*hb_unicode_combining_class_func_t)
    (hb_unicode_funcs_t *ufuncs, hb_codepoint_t unicode, void *user_data);
```

```rust
pub type hb_unicode_combining_class_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_unicode_combining_class_t,
>;
```

Should retrieve the Canonical Combining Class (`ccc`) property for a specified
code point. Returns any value in 0..=254; 0
(`HB_UNICODE_COMBINING_CLASS_NOT_REORDERED`) is the answer for the vast majority
of characters and is what the built-in stub returns.

#### `hb_unicode_mirroring_func_t`

```c
typedef hb_codepoint_t (*hb_unicode_mirroring_func_t)
    (hb_unicode_funcs_t *ufuncs, hb_codepoint_t unicode, void *user_data);
```

```rust
pub type hb_unicode_mirroring_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_codepoint_t,
>;
```

Should retrieve the Bi-Directional Mirroring Glyph code point for a specified
code point — the `Bidi_Mirroring_Glyph` (`bmg`) property. The header is explicit:
*"If a code point does not have a specified Bi-Directional Mirroring Glyph
defined, the method should return the original code point."* Identity, not zero,
is the "no answer" answer. HarfBuzz uses this when shaping an RTL run so that
`(` renders as `)`.

#### `hb_unicode_script_func_t`

```c
typedef hb_script_t (*hb_unicode_script_func_t)
    (hb_unicode_funcs_t *ufuncs, hb_codepoint_t unicode, void *user_data);
```

```rust
pub type hb_unicode_script_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_script_t,
>;
```

Should retrieve the Script (`sc`) property for a specified code point. Returns
an `hb_script_t`; see [Scripts](script.md). `HB_SCRIPT_UNKNOWN` (`Zzzz`) is the
correct answer for unassigned, private-use, noncharacter, and surrogate code
points, and is what the built-in stub returns for everything.

#### `hb_unicode_compose_func_t`

```c
typedef hb_bool_t (*hb_unicode_compose_func_t)
    (hb_unicode_funcs_t *ufuncs, hb_codepoint_t a, hb_codepoint_t b,
     hb_codepoint_t *ab, void *user_data);
```

```rust
pub type hb_unicode_compose_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        a: hb_codepoint_t,
        b: hb_codepoint_t,
        ab: *mut hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;
```

Should compose a sequence of two input code points **by canonical
equivalence**, writing the composed code point through the `ab` out-parameter if
successful, and returning an `hb_bool_t` indicating whether the composition
happened. Only the pairwise canonical case is in scope — no compatibility
composition, no multi-character sequences. Leave `*ab` alone when returning
false; callers must not read it.

#### `hb_unicode_decompose_func_t`

```c
typedef hb_bool_t (*hb_unicode_decompose_func_t)
    (hb_unicode_funcs_t *ufuncs, hb_codepoint_t ab,
     hb_codepoint_t *a, hb_codepoint_t *b, void *user_data);
```

```rust
pub type hb_unicode_decompose_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        ab: hb_codepoint_t,
        a: *mut hb_codepoint_t,
        b: *mut hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;
```

Should decompose an input code point by canonical equivalence, writing the two
resulting code points through the `a` and `b` out-parameters if successful, and
returning an `hb_bool_t`. Exactly two outputs, always. The header does not say
how to express a *singleton* canonical decomposition, but the bundled
implementation in `hb-ucd.cc` sets `*a` to the single code point and `*b` to
`0`, and HarfBuzz's normalizer is written against that behaviour — match it.
Leave both out-parameters alone when returning false.

## Functions

### Obtaining a structure

#### `hb_unicode_funcs_get_default`

```c
hb_unicode_funcs_t *hb_unicode_funcs_get_default (void);
```

```rust
pub fn hb_unicode_funcs_get_default() -> *mut hb_unicode_funcs_t;
```

Fetches a pointer to the default Unicode-functions structure — the one used
when no functions are explicitly set on an `hb_buffer_t`.

**Returns** — never null. Which implementation you get is decided at build
time, in this order:

1. HarfBuzz's own bundled UCD tables (`hb_ucd_get_unicode_funcs()`), unless
   `HB_NO_UCD` or `HB_NO_UNICODE_FUNCS` is defined.
2. GLib (`hb_glib_get_unicode_funcs()`), if built with `HAVE_GLIB`.
3. ICU (`hb_icu_get_unicode_funcs()`), if built with `HAVE_ICU` and
   `HAVE_ICU_BUILTIN`.
4. The empty structure, only in a `HB_NO_UNICODE_FUNCS` build. Any other
   configuration that reaches this point is a compile error upstream
   ("Could not find any Unicode functions implementation, you have to provide
   your own").

**Ownership** — upstream annotates the return `(transfer none)`. The caller does
**not** own a reference: this is a process-wide singleton owned by HarfBuzz.
Do not destroy it. If you want to keep it beyond the immediate call, take your
own reference with `hb_unicode_funcs_reference()` and match it with a destroy.

**Notes** — Since HarfBuzz 0.9.2. Treat it as immutable: it may already have
been frozen by having been used as a parent, and in any case mutating a
process-wide singleton is never what you want. Use it as a *parent* instead.

#### `hb_unicode_funcs_create`

```c
hb_unicode_funcs_t *hb_unicode_funcs_create (hb_unicode_funcs_t *parent);
```

```rust
pub fn hb_unicode_funcs_create(parent: *mut hb_unicode_funcs_t) -> *mut hb_unicode_funcs_t;
```

Creates a new `hb_unicode_funcs_t` structure of Unicode functions, with a
reference count of one.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `parent` | Upstream annotates this `(nullable)`. Null is replaced by `hb_unicode_funcs_get_empty()`, so the structure always has a parent. |

**Returns** — the new structure, `(transfer full)`. **Never null**: on allocation
failure it returns the singleton empty structure instead, so the only way to
detect failure is to compare against `hb_unicode_funcs_get_empty()`.

**Ownership** — the caller owns the returned reference and must release it with
`hb_unicode_funcs_destroy()`. The call takes its own reference on `parent`,
released when the child is destroyed, so the parent outlives the child.

**Side effects** — `parent` is made **immutable** by this call, permanently.
There is no way to undo that, and creating a child is therefore a one-way door
for the parent.

**Inheritance is a snapshot, not a delegation.** The implementation copies the
parent's entire function table and its `user_data` table into the child at
creation time (it deliberately does *not* copy the destroy notifiers, since the
parent still owns those). So a method you never set on the child calls the
parent's function pointer directly — there is no per-call chain walk, and
because the parent is now immutable it can never change underneath you.

**Notes** — Since HarfBuzz 0.9.2.

#### `hb_unicode_funcs_get_empty`

```c
hb_unicode_funcs_t *hb_unicode_funcs_get_empty (void);
```

```rust
pub fn hb_unicode_funcs_get_empty() -> *mut hb_unicode_funcs_t;
```

Fetches the singleton empty Unicode-functions structure. Never null.

Its six methods are inert stubs with fixed answers:

| Method | Stub result |
| --- | --- |
| general category | `HB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER` (`Lo`) |
| combining class | `HB_UNICODE_COMBINING_CLASS_NOT_REORDERED` (0) |
| mirroring | the input code point, unchanged |
| script | `HB_SCRIPT_UNKNOWN` (`Zzzz`) |
| compose | always `false` |
| decompose | always `false` |

Its `parent` pointer is null, so `hb_unicode_funcs_get_parent()` on it returns
itself.

**Ownership** — upstream annotates the return `(transfer full)`, so the
conventional thing is to treat it like any other structure and destroy it when
done; HarfBuzz's shared null objects are inert, so referencing and destroying
them are cheap no-ops.

**Notes** — Since HarfBuzz 0.9.2. This is also the value returned by
`hb_unicode_funcs_create()` on allocation failure, and by
`hb_unicode_funcs_get_default()` in a `HB_NO_UNICODE_FUNCS` build.

### Reference counting

#### `hb_unicode_funcs_reference`

```c
hb_unicode_funcs_t *hb_unicode_funcs_reference (hb_unicode_funcs_t *ufuncs);
```

```rust
pub fn hb_unicode_funcs_reference(ufuncs: *mut hb_unicode_funcs_t) -> *mut hb_unicode_funcs_t;
```

Increases the reference count on a Unicode-functions structure and returns the
same pointer, which makes it convenient to use inline when handing the structure
to something that will take ownership. Every call must be matched by a
`hb_unicode_funcs_destroy()`.

**Notes** — Since HarfBuzz 0.9.2. Marked `(skip)` for language bindings
upstream. Reference counts are atomic in a normally configured build.

#### `hb_unicode_funcs_destroy`

```c
void hb_unicode_funcs_destroy (hb_unicode_funcs_t *ufuncs);
```

```rust
pub fn hb_unicode_funcs_destroy(ufuncs: *mut hb_unicode_funcs_t);
```

Decreases the reference count on a Unicode-functions structure. When the count
reaches zero the structure is destroyed and all of its memory freed. On the way
down it invokes each installed callback's `destroy` notifier on that callback's
`user_data`, then releases the reference it holds on its parent — which can
cascade.

**Returns** — nothing. There is no way to observe whether the object actually
went away.

**Notes** — Since HarfBuzz 0.9.2. Marked `(skip)` upstream. Tolerates the shared
singletons, which are inert.

### User data

#### `hb_unicode_funcs_set_user_data`

```c
hb_bool_t hb_unicode_funcs_set_user_data (hb_unicode_funcs_t *ufuncs,
                                          hb_user_data_key_t *key,
                                          void *              data,
                                          hb_destroy_func_t   destroy,
                                          hb_bool_t           replace);
```

```rust
pub fn hb_unicode_funcs_set_user_data(
    ufuncs: *mut hb_unicode_funcs_t,
    key: *mut hb_user_data_key_t,
    data: *mut c_void,
    destroy: hb_destroy_func_t,
    replace: hb_bool_t,
) -> hb_bool_t;
```

Attaches a key/data pair to the structure. HarfBuzz uses the *address* of `key`,
not its contents, so the key object must outlive the structure — a `static` is
the usual choice. `destroy` may be null; when non-null it is called with `data`
when the structure is destroyed or when the entry is replaced. `replace` selects
whether an existing entry stored under the same key is overwritten.

**Returns** — true on success, false otherwise (allocation failure, or a
non-replace call against an existing key).

**Notes** — Since HarfBuzz 0.9.2. This is orthogonal to the per-callback
`user_data` passed to the setters; do not confuse the two.

#### `hb_unicode_funcs_get_user_data`

```c
void *hb_unicode_funcs_get_user_data (const hb_unicode_funcs_t *ufuncs,
                                      hb_user_data_key_t       *key);
```

```rust
pub fn hb_unicode_funcs_get_user_data(
    ufuncs: *const hb_unicode_funcs_t,
    key: *mut hb_user_data_key_t,
) -> *mut c_void;
```

Fetches the data previously attached under `key`. Note the `const` structure
parameter. Ownership is not transferred (`(transfer none)`): the returned
pointer belongs to whoever stored it and must not be freed by the caller.
Returns null when no entry is present for that key.

**Notes** — Since HarfBuzz 0.9.2.

### Immutability

#### `hb_unicode_funcs_make_immutable`

```c
void hb_unicode_funcs_make_immutable (hb_unicode_funcs_t *ufuncs);
```

```rust
pub fn hb_unicode_funcs_make_immutable(ufuncs: *mut hb_unicode_funcs_t);
```

Makes the specified Unicode-functions structure immutable. **One-way** — there
is no `make_mutable`. Afterwards every `hb_unicode_funcs_set_*_func()` call
silently does nothing except run the `destroy` notifier on the `user_data` it
was handed (so nothing leaks, and nothing changes).

Applied implicitly by `hb_unicode_funcs_create()` to its `parent` argument.

**Notes** — Since HarfBuzz 0.9.2. Idempotent. Freeze a structure before sharing
it across threads; that is the discipline the whole object model is built for.

#### `hb_unicode_funcs_is_immutable`

```c
hb_bool_t hb_unicode_funcs_is_immutable (hb_unicode_funcs_t *ufuncs);
```

```rust
pub fn hb_unicode_funcs_is_immutable(ufuncs: *mut hb_unicode_funcs_t) -> hb_bool_t;
```

Tests whether the structure is immutable; true if it is. Since HarfBuzz 0.9.2.
This is the *only* way to find out that a setter is going to be a no-op, since
the setters themselves return `void`.

### Inheritance

#### `hb_unicode_funcs_get_parent`

```c
hb_unicode_funcs_t *hb_unicode_funcs_get_parent (hb_unicode_funcs_t *ufuncs);
```

```rust
pub fn hb_unicode_funcs_get_parent(ufuncs: *mut hb_unicode_funcs_t) -> *mut hb_unicode_funcs_t;
```

Fetches the parent of the structure.

**Returns** — **never null**: a structure with no parent (the empty singleton)
reports `hb_unicode_funcs_get_empty()`.

**Ownership** — the child keeps owning the returned reference. This is a borrow,
not a new reference; do not destroy it unless you call
`hb_unicode_funcs_reference()` on it first.

**Notes** — Since HarfBuzz 0.9.2. The parent is guaranteed immutable, because
`hb_unicode_funcs_create()` froze it.

### Installing callbacks

All six setters have the same shape, the same ownership rules, and the same
failure mode. They are documented once here and then listed individually.

```c
void hb_unicode_funcs_set_<name>_func (hb_unicode_funcs_t     *ufuncs,
                                       hb_unicode_<name>_func_t func,
                                       void                   *user_data,
                                       hb_destroy_func_t       destroy);
```

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `ufuncs` | The structure to modify. Nullability is unspecified in the header; the implementation dereferences it, so treat null as forbidden. |
| `func` | The callback. Upstream annotates it `(closure user_data) (destroy destroy) (scope notified)`. Passing null (`None` in Rust) is legal and **restores the parent's implementation** — see below. |
| `user_data` | Opaque pointer handed back to `func` on every call. |
| `destroy` | Upstream annotates it `(nullable)`. Called with `user_data` when the callback is replaced or the structure is destroyed. |

**Returns** — nothing. There is no success/failure signal.

**Ownership** — the callee takes ownership of `user_data` unconditionally, in
every path:

- Normal case: `destroy(user_data)` runs when the method is replaced or the
  structure is destroyed.
- `ufuncs` is immutable: the setter runs `destroy(user_data)` immediately and
  returns without changing anything.
- `func` is null: the setter runs `destroy(user_data)` immediately, then sets
  the method back to the parent's function pointer *and the parent's*
  `user_data`, with no destroy notifier of its own.

So a Rust wrapper that leaks a `Box` into `user_data` before the call is
correct in all three cases; one that frees the `Box` afterwards is a double
free.

**Notes** — all six are Since HarfBuzz 0.9.2.

#### `hb_unicode_funcs_set_general_category_func`

```rust
pub fn hb_unicode_funcs_set_general_category_func(
    ufuncs: *mut hb_unicode_funcs_t,
    func: hb_unicode_general_category_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Sets the implementation function for `hb_unicode_general_category_func_t`.

#### `hb_unicode_funcs_set_combining_class_func`

```rust
pub fn hb_unicode_funcs_set_combining_class_func(
    ufuncs: *mut hb_unicode_funcs_t,
    func: hb_unicode_combining_class_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Sets the implementation function for `hb_unicode_combining_class_func_t`.

#### `hb_unicode_funcs_set_mirroring_func`

```rust
pub fn hb_unicode_funcs_set_mirroring_func(
    ufuncs: *mut hb_unicode_funcs_t,
    func: hb_unicode_mirroring_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Sets the implementation function for `hb_unicode_mirroring_func_t`.

#### `hb_unicode_funcs_set_script_func`

```rust
pub fn hb_unicode_funcs_set_script_func(
    ufuncs: *mut hb_unicode_funcs_t,
    func: hb_unicode_script_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Sets the implementation function for `hb_unicode_script_func_t`.

#### `hb_unicode_funcs_set_compose_func`

```rust
pub fn hb_unicode_funcs_set_compose_func(
    ufuncs: *mut hb_unicode_funcs_t,
    func: hb_unicode_compose_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Sets the implementation function for `hb_unicode_compose_func_t`.

#### `hb_unicode_funcs_set_decompose_func`

```rust
pub fn hb_unicode_funcs_set_decompose_func(
    ufuncs: *mut hb_unicode_funcs_t,
    func: hb_unicode_decompose_func_t,
    user_data: *mut c_void,
    destroy: hb_destroy_func_t,
);
```

Sets the implementation function for `hb_unicode_decompose_func_t`.

### Querying properties

These six dispatch straight through the structure's vtable. None of them
validates `unicode` against `HB_UNICODE_MAX`; that is the callback's problem.
None of them tolerates a null `ufuncs`.

#### `hb_unicode_general_category`

```c
hb_unicode_general_category_t hb_unicode_general_category (hb_unicode_funcs_t *ufuncs,
                                                           hb_codepoint_t unicode);
```

```rust
pub fn hb_unicode_general_category(
    ufuncs: *mut hb_unicode_funcs_t,
    unicode: hb_codepoint_t,
) -> hb_unicode_general_category_t;
```

Retrieves the General Category (`gc`) property of code point `unicode`. Returns
the `hb_unicode_general_category_t`; there is no error value. Since HarfBuzz
0.9.2.

#### `hb_unicode_combining_class`

```c
hb_unicode_combining_class_t hb_unicode_combining_class (hb_unicode_funcs_t *ufuncs,
                                                         hb_codepoint_t unicode);
```

```rust
pub fn hb_unicode_combining_class(
    ufuncs: *mut hb_unicode_funcs_t,
    unicode: hb_codepoint_t,
) -> hb_unicode_combining_class_t;
```

Retrieves the Canonical Combining Class (`ccc`) property of code point
`unicode`. **The result may be any value in 0..=254**, not only the named
constants — the header says so in a `<note>`. Since HarfBuzz 0.9.2.

#### `hb_unicode_mirroring`

```c
hb_codepoint_t hb_unicode_mirroring (hb_unicode_funcs_t *ufuncs,
                                     hb_codepoint_t unicode);
```

```rust
pub fn hb_unicode_mirroring(
    ufuncs: *mut hb_unicode_funcs_t,
    unicode: hb_codepoint_t,
) -> hb_codepoint_t;
```

Retrieves the Bi-directional Mirroring Glyph code point defined for `unicode`.
Code points with no mirroring glyph come back **unchanged**, so `result ==
unicode` is the "no mirror" signal, not zero. Since HarfBuzz 0.9.2.

#### `hb_unicode_script`

```c
hb_script_t hb_unicode_script (hb_unicode_funcs_t *ufuncs,
                               hb_codepoint_t unicode);
```

```rust
pub fn hb_unicode_script(
    ufuncs: *mut hb_unicode_funcs_t,
    unicode: hb_codepoint_t,
) -> hb_script_t;
```

Retrieves the `hb_script_t` script to which code point `unicode` belongs.
Unassigned, private-use, noncharacter, and surrogate code points give
`HB_SCRIPT_UNKNOWN`; a newer HarfBuzz may return a script value your build has
no constant for, so do not treat the result as a closed set. Since HarfBuzz
0.9.2.

#### `hb_unicode_compose`

```c
hb_bool_t hb_unicode_compose (hb_unicode_funcs_t *ufuncs,
                              hb_codepoint_t      a,
                              hb_codepoint_t      b,
                              hb_codepoint_t     *ab);
```

```rust
pub fn hb_unicode_compose(
    ufuncs: *mut hb_unicode_funcs_t,
    a: hb_codepoint_t,
    b: hb_codepoint_t,
    ab: *mut hb_codepoint_t,
) -> hb_bool_t;
```

Fetches the composition of a sequence of two Unicode code points, by calling the
composition function of `ufuncs`.

**Parameters** — `a` and `b` are the two code points to compose; `ab` is an
out-parameter, annotated `(out)`. The header does not mark it nullable and the
built-in implementations write through it unconditionally on success, so pass a
real pointer.

**Returns** — true if `a` and `b` composed, false otherwise. `*ab` is only
meaningful when the call returns true; on false it is left untouched, so
initialise it or ignore it.

**Notes** — Since HarfBuzz 0.9.2. Canonical composition only.

#### `hb_unicode_decompose`

```c
hb_bool_t hb_unicode_decompose (hb_unicode_funcs_t *ufuncs,
                                hb_codepoint_t      ab,
                                hb_codepoint_t     *a,
                                hb_codepoint_t     *b);
```

```rust
pub fn hb_unicode_decompose(
    ufuncs: *mut hb_unicode_funcs_t,
    ab: hb_codepoint_t,
    a: *mut hb_codepoint_t,
    b: *mut hb_codepoint_t,
) -> hb_bool_t;
```

Fetches the decomposition of a Unicode code point, by calling the decomposition
function of `ufuncs`.

**Parameters** — `ab` is the code point to decompose; `a` and `b` are
out-parameters, both annotated `(out)` and neither marked nullable.

**Returns** — true if `ab` was decomposed, false otherwise. `*a` and `*b` are
only meaningful when the call returns true. With the bundled UCD implementation
a singleton decomposition sets `*b` to `0`.

**Notes** — Since HarfBuzz 0.9.2. Canonical decomposition only; the
compatibility variant, `hb_unicode_decompose_compatibility()`, was deprecated in
HarfBuzz 2.0.0 and lives in `hb-deprecated.h`.
