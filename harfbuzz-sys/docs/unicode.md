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
returning an `hb_bool_t`. Exactly two outputs — a canonical decomposition
mapping of length one is expressed as `a` = the singleton and `b` = 0 by
convention in HarfBuzz's own UCD implementation; the header does not specify
this, so if you write your own, match what the normalizer expects by mirroring
the built-in behaviour. Leave both out-parameters alone when returning false.
