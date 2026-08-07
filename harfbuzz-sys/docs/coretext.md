# CoreText integration

Reference for `hb-coretext.h`, transcribed in `harfbuzz-sys` as the `coretext`
module. The module is gated on the crate's `coretext` feature and is **not**
re-exported at the crate root, so its items are reached as
`harfbuzz_sys::coretext::*`.

## Overview

On Apple platforms a font usually arrives as a Core Graphics or Core Text
object rather than as a file path or a byte buffer — from `CTFontCreateWithName`,
from an `NSFont`/`UIFont`, from a font descriptor, or from the system's font
cascade. `hb-coretext.h` is the bridge between those objects and HarfBuzz's own,
and it works in both directions.

The two frameworks split the job the same way HarfBuzz does:

| Apple type | HarfBuzz type | What it holds |
| --- | --- | --- |
| `CGFontRef` (Core Graphics) | `hb_face_t` | A typeface: tables, glyph outlines, no size |
| `CTFontRef` (Core Text) | `hb_font_t` | A typeface *plus* point size, transform, variation settings |

So the API is four conversions and one switch:

* `hb_coretext_face_create()` — `CGFontRef` → `hb_face_t`.
* `hb_coretext_face_get_cg_font()` — `hb_face_t` → `CGFontRef`.
* `hb_coretext_font_create()` — `CTFontRef` → `hb_font_t`.
* `hb_coretext_font_get_ct_font()` — `hb_font_t` → `CTFontRef`.
* `hb_coretext_font_set_funcs()` — make a font answer metric and outline
  queries through Core Text instead of through HarfBuzz's own OpenType code.

Two more constructors — `hb_coretext_face_create_from_file_or_fail()` and
`hb_coretext_face_create_from_blob_or_fail()` — skip the `CGFontRef` step and
hand raw font data to Core Text's loader. They are the CoreText analogues of
`hb_face_create_from_file_or_fail()` and `hb_face_create_or_fail()`, useful when
you want Apple's parser (and Apple's opinion about what is a valid font) rather
than HarfBuzz's.

**The getters are lazy, not merely accessors.** `hb_coretext_face_get_cg_font()`
and `hb_coretext_font_get_ct_font()` work on *any* face or font, not just ones
that came from this API. If no Core Graphics / Core Text object has been
associated yet, one is manufactured on demand from the object's font data and
cached on it. Both returned references are owned by the HarfBuzz object and must
not be released by the caller.

**Shaping is still HarfBuzz's.** Nothing in this header hands shaping over to
Core Text. Faces and fonts built here are shaped by HarfBuzz's OpenType engine
and, unless you call `hb_coretext_font_set_funcs()`, use HarfBuzz's own font
functions. HarfBuzz does have a `coretext` *shaper* back end, but that is an
internal testing path used by `hb-shape` to compare HarfBuzz's output against
Core Text's, and it is unrelated to the functions on this page.

**You do not need this header for AAT.** HarfBuzz reads Apple Advanced
Typography tables — `mort`, `morx`, `kerx`, `trak`, `feat`, and the rest —
natively, on every platform. The three tag constants this header defines are a
convenience for code that inspects those tables, not a signal that CoreText
integration is required to shape AAT fonts.

**Building.** The `coretext` Cargo feature compiles HarfBuzz's CoreText sources
and adds the link directives: the `ApplicationServices` umbrella framework on
macOS, and `CoreText` + `CoreGraphics` + `CoreFoundation` on the iOS-family
targets. On a non-Apple target `build.rs` prints a warning and ignores the
feature, so the declarations in the Rust module have no symbols behind them
there.

## Types

### `CGFontRef`

```c
typedef struct CGFont *CGFontRef;   /* <CoreGraphics/CGFont.h> */
```

```rust
crate::opaque_handle! { CGFont }
pub type CGFontRef = *mut CGFont;
```

A Core Graphics font: a typeface with no size attached, and the Core Graphics
counterpart of `hb_face_t`. It is a CoreFoundation-style object — retain it with
`CGFontRetain`, release it with `CGFontRelease`.

This type belongs to Apple's frameworks, not to HarfBuzz. `harfbuzz-sys` has no
dependency that supplies it, so the module declares its own zero-sized opaque
`CGFont` stand-in. Since `CGFontRef` is a plain pointer, the alias is ABI- and
layout-compatible with the one in the `core-graphics` crate; a `.cast()`
converts between them.

### `CTFontRef`

```c
typedef const struct __CTFont *CTFontRef;   /* <CoreText/CTFont.h> */
```

```rust
crate::opaque_handle! { CTFont }
pub type CTFontRef = *const CTFont;
```

A Core Text font: a typeface together with a point size, a text matrix, and a
variation configuration — the Core Text counterpart of `hb_font_t`. Retain with
`CFRetain`, release with `CFRelease`. As with `CGFontRef`, the pointee type is a
local stand-in and the alias is layout-compatible with the `core-text` crate's
`CTFontRef`.

Note the constness difference between the two aliases: it mirrors Apple's
headers exactly (`CGFontRef` is a pointer to non-const, `CTFontRef` a pointer to
const). It has no ABI consequence.

### Types this header uses but does not declare

`hb-coretext.h` includes `hb.h`, so the HarfBuzz types in its signatures come
from elsewhere and are imported by the Rust module rather than redeclared:

| Type | Declared in | Rust module |
| --- | --- | --- |
| `hb_tag_t` | `hb-common.h` | `common` |
| `hb_blob_t` | `hb-blob.h` | `blob` |
| `hb_face_t` | `hb-face.h` | `face` |
| `hb_font_t` | `hb-font.h` | `font` |

## Constants

All three are `#define`d in terms of `HB_TAG()`, and all three become plain
`pub const` values in Rust. None carries a `Since:` annotation in the header.

### `HB_CORETEXT_TAG_MORT`

```c
#define HB_CORETEXT_TAG_MORT HB_TAG('m','o','r','t')
```

```rust
pub const HB_CORETEXT_TAG_MORT: hb_tag_t = HB_TAG(b'm', b'o', b'r', b't');
```

The `hb_tag_t` for the `mort` (glyph metamorphosis) table, which holds AAT
features. Numeric value `0x6D6F7274`. `mort` is the older, superseded form of
`morx`; see Apple's
[TrueType Reference Manual](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6mort.html).

### `HB_CORETEXT_TAG_MORX`

```c
#define HB_CORETEXT_TAG_MORX HB_TAG('m','o','r','x')
```

```rust
pub const HB_CORETEXT_TAG_MORX: hb_tag_t = HB_TAG(b'm', b'o', b'r', b'x');
```

The `hb_tag_t` for the `morx` (extended glyph metamorphosis) table, the AAT
counterpart of OpenType's `GSUB`. Numeric value `0x6D6F7278`. See Apple's
[TrueType Reference Manual](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6morx.html).

### `HB_CORETEXT_TAG_KERX`

```c
#define HB_CORETEXT_TAG_KERX HB_TAG('k','e','r','x')
```

```rust
pub const HB_CORETEXT_TAG_KERX: hb_tag_t = HB_TAG(b'k', b'e', b'r', b'x');
```

The `hb_tag_t` for the `kerx` (extended kerning) table, which holds AAT kerning
information — one of the AAT counterparts of OpenType's `GPOS`. Numeric value
`0x6B657278`. See Apple's
[TrueType Reference Manual](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM06/Chap6kerx.html).

## Functions

### Face creation

#### `hb_coretext_face_create`

```c
hb_face_t *
hb_coretext_face_create (CGFontRef cg_font);
```

```rust
pub fn hb_coretext_face_create(cg_font: CGFontRef) -> *mut hb_face_t;
```

Creates a face that reads its tables out of a Core Graphics font.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `cg_font` | The Core Graphics font to wrap. The header does not document whether null is allowed, and the implementation does not check — passing null yields a face on which every table lookup fails. Treat it as required. |

**Returns** — a new face. **Never null**: on allocation failure the singleton
empty face comes back instead, so you cannot detect failure by comparing against
`NULL`.

**Ownership** — the face takes its own `CGFontRetain` on `cg_font`, so the
caller may release its reference immediately after the call. The caller owns
the returned face and must release it with `hb_face_destroy()`.

**Notes** — Since HarfBuzz 0.9.10. The face is built with
`hb_face_create_for_tables()`: tables are fetched lazily, one at a time, through
`CGFontCopyTableForTag`, and each is wrapped in a read-only blob that releases
the `CFData` when it dies. A table-tag enumeration callback is installed too, so
`hb_face_get_table_tags()` works on the result — unlike a bare
`hb_face_create_for_tables()` face. The face's index is left at zero; use
`hb_face_set_index()` if you need it set.

#### `hb_coretext_face_create_from_file_or_fail`

```c
hb_face_t *
hb_coretext_face_create_from_file_or_fail (const char   *file_name,
                                           unsigned int  index);
```

```rust
pub fn hb_coretext_face_create_from_file_or_fail(
    file_name: *const c_char,
    index: c_uint,
) -> *mut hb_face_t;
```

Creates a face by handing a font file to Core Text's font manager. Similar in
functionality to `hb_face_create_from_file_or_fail()`, but Core Text does the
parsing.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `file_name` | Path to a font file, in the platform's file-system representation. Required; it is passed straight to `CFURLCreateFromFileSystemRepresentation` with `strlen()`, so null is a crash, not a failure. |
| `index` | Face index within the file. The **low 16 bits** select a face inside a font collection and **must be zero** — Core Text cannot read TTC/OTC collections. The **high 16 bits**, when non-zero, select a named instance of a variable font. |

**Returns** — the new face, or `NULL` when the file cannot be read, when Core
Text finds no font descriptors in it, when the low 16 bits of `index` are
non-zero, or when the face at that index does not exist.

**Ownership** — the caller owns a non-null result and must release it with
`hb_face_destroy()`. Nothing from the file is retained by the caller; the URL
and descriptor array created internally are released before returning.

**Notes** — Since HarfBuzz 10.1.0. On success the face's index is set to the
`index` you passed, so `hb_face_get_index()` reads it back. Internally the file
is loaded through `CTFontManagerCreateFontDescriptorsFromURL`, a `CTFont` is
built from the selected descriptor, and its graphics font is passed to
`hb_coretext_face_create()`; the intermediate `CGFontRef` is released, leaving
only the face's own retain. When the low 16 bits of `index` are non-zero the
function returns early and leaks the URL and descriptor array it had already
created — avoid feeding it collection indices.

#### `hb_coretext_face_create_from_blob_or_fail`

```c
hb_face_t *
hb_coretext_face_create_from_blob_or_fail (hb_blob_t    *blob,
                                           unsigned int  index);
```

```rust
pub fn hb_coretext_face_create_from_blob_or_fail(
    blob: *mut hb_blob_t,
    index: c_uint,
) -> *mut hb_face_t;
```

Creates a face by handing font data already in memory to Core Text. Similar in
functionality to `hb_face_create_or_fail()`, but Core Text (or Core Graphics)
does the parsing.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `blob` | A blob holding the font data. Required — it is dereferenced unconditionally. |
| `index` | Face index within the data, encoded exactly as for `hb_coretext_face_create_from_file_or_fail()`: low 16 bits = collection index, must be zero; high 16 bits = named-instance index. |

**Returns** — the new face, or `NULL` when the data cannot be parsed, when the
low 16 bits of `index` are non-zero, or when the requested named instance does
not exist.

**Ownership** — the blob is made **immutable** and an extra reference is taken
on it, so its bytes are *borrowed* for as long as the resulting `CGFontRef`
lives, not copied. The reference is dropped by the data provider's release
callback when Core Graphics is done with it. The caller keeps its own blob
reference and must still destroy it. The caller owns a non-null face and must
release it with `hb_face_destroy()`.

**Notes** — Since HarfBuzz 11.0.0. On success the face's index is set to the
`index` you passed. For the plain case (no named instance) the bytes go through
`CGDataProviderCreateWithData` + `CGFontCreateWithDataProvider`. For a named
instance the data goes through `CTFontManagerCreateFontDescriptorsFromData`,
which requires a recent deployment target — iOS 11, macOS 10.15, tvOS 11,
watchOS 4, Mac Catalyst 13.1, or visionOS 1; on older deployment targets a
non-zero named-instance index always returns `NULL`. A zero-length blob is
accepted by the call but will fail during parsing.

### Font creation

#### `hb_coretext_font_create`

```c
hb_font_t *
hb_coretext_font_create (CTFontRef ct_font);
```

```rust
pub fn hb_coretext_font_create(ct_font: CTFontRef) -> *mut hb_font_t;
```

Creates a font — and, implicitly, its face — from a Core Text font.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `ct_font` | The Core Text font to wrap. Required; it is passed to `CTFontCopyGraphicsFont` without a null check. |

**Returns** — a new font. Never null; on failure you get an object derived from
the empty face rather than `NULL`.

**Ownership** — the font takes its own `CFRetain` on `ct_font` and stores it, so
the caller keeps and must still release its own reference. The face created
along the way is owned by the font; if you want it, fetch it with
`hb_font_get_face()` (that does not transfer ownership). The caller owns the
returned font and must release it with `hb_font_destroy()`.

**Notes** — Since HarfBuzz 1.7.2.

* The font's **point size** is copied from `CTFontGetSize(ct_font)` into `ptem`,
  as if by `hb_font_set_ptem()`. This is what enables optical-size and `trak`
  tracking behaviour.
* The font's **variation settings** are copied from `CTFontCopyVariation()` and
  applied as if by `hb_font_set_variations()`. (Added in HarfBuzz 8.x; earlier
  releases did not do this.)
* The font's **scale is not set** beyond `hb_font_create()`'s default, which is
  the face's units-per-em. Shaping therefore produces positions in font units,
  *not* in points or pixels. Call `hb_font_set_scale()` yourself if you want
  something else.
* The font uses **HarfBuzz's own font functions**, not Core Text's. This is
  deliberate. Call `hb_coretext_font_set_funcs()` to change it.
* The stored `CTFontRef` is installed with a compare-and-exchange into the
  font's CoreText data slot, so it is the object `hb_coretext_font_get_ct_font()`
  hands back.

### Accessors

#### `hb_coretext_face_get_cg_font`

```c
CGFontRef
hb_coretext_face_get_cg_font (hb_face_t *face);
```

```rust
pub fn hb_coretext_face_get_cg_font(face: *mut hb_face_t) -> CGFontRef;
```

Fetches the `CGFontRef` associated with a face, creating one if the face does
not have one yet.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `face` | The face to query. Required. Takes a non-`const` pointer although the call is conceptually a read — it may mutate the face's cache slot. |

**Returns** — the face's Core Graphics font, or `NULL` if one could not be
produced.

**Ownership** — **transfer none.** The reference belongs to the face and is
released when the face is destroyed. Do not call `CGFontRelease` on it. If you
need it to outlive the face, take your own `CGFontRetain`. It stays valid for as
long as the face lives, and repeated calls return the same pointer.

**Notes** — Since HarfBuzz 0.9.10.

* This is a *lazy* accessor, not a plain getter. On the first call for a face
  that was made by `hb_coretext_face_create()`, it retains the `CGFontRef` the
  face was built from. For any other face — one from `hb_face_create()`,
  `hb_face_create_from_file_or_fail()`, and so on — it takes the face's
  reference blob and builds a `CGFontRef` from those bytes, then caches it.
* It therefore returns `NULL` for faces Core Graphics cannot represent: a face
  whose data is a TrueType collection (low 16 bits of the face index non-zero),
  a face built with `hb_face_create_for_tables()` that has no underlying blob, or
  a face whose bytes Core Graphics refuses.
* The lazy slot is filled with an atomic compare-and-exchange, so concurrent
  first calls are safe; a losing thread's `CGFontRef` is released and the winner's
  is returned to everyone.

#### `hb_coretext_font_get_ct_font`

```c
CTFontRef
hb_coretext_font_get_ct_font (hb_font_t *font);
```

```rust
pub fn hb_coretext_font_get_ct_font(font: *mut hb_font_t) -> CTFontRef;
```

Fetches the `CTFontRef` associated with a font, creating one if the font does
not have one yet.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | The font to query. Required. Non-`const` for the same reason as above. |

**Returns** — the font's Core Text font, or `NULL` if one could not be produced
(most often because the face has no `CGFontRef`, per the previous function).

**Ownership** — **transfer none.** The reference belongs to the font and is
released when the font is destroyed. Do not `CFRelease` it; `CFRetain` it if you
need it to outlive the font. Repeated calls return the same pointer.

**Notes** — Since HarfBuzz 0.9.10.

* For a font made by `hb_coretext_font_create()`, this returns the very
  `CTFontRef` you passed in.
* For any other font it builds one on demand: the face's `CGFontRef` (itself
  possibly created on demand) is turned into a `CTFont` at the font's `ptem`, or
  at **12 points** when `ptem` is unset or non-positive, and the font's
  normalized variation coordinates are applied to it.
* The construction path has Apple-specific quirks worth knowing about: for the
  system UI fonts (`.SFNSText`, `.SFNSDisplay`) it goes through
  `CTFontCreateUIFontForLanguage` so that `trak` tracking is enabled, and it
  installs a `LastResort`-first cascade list to short-circuit Core Text's own
  font fallback, which HarfBuzz does not want.
* Same atomic lazy-init behaviour, and the same thread-safety guarantee, as
  `hb_coretext_face_get_cg_font()`.

### Font functions

#### `hb_coretext_font_set_funcs`

```c
void
hb_coretext_font_set_funcs (hb_font_t *font);
```

```rust
pub fn hb_coretext_font_set_funcs(font: *mut hb_font_t);
```

Configures a font to answer glyph queries through Core Text rather than through
HarfBuzz's own OpenType implementation.

**Parameters**

| Parameter | Meaning |
| --- | --- |
| `font` | The font to reconfigure. Required. Must not be immutable — like any `hb_font_set_funcs()` call, this is a mutation. |

**Returns** — nothing, and there is **no success indication**. See the failure
mode below.

**Ownership** — nothing is transferred. The shared Core Text font-functions
object is a process-wide immutable singleton created on first use and freed at
exit; the font does not own it.

**Notes** — Since HarfBuzz 10.1.0.

* Works on **any** font, including one whose face came from `hb_face_create()`
  and never touched Core Text. That is the point of the function: you can take
  an ordinary HarfBuzz font and give it Apple's metrics.
* Internally it creates a `CTFont`, exactly as `hb_coretext_font_get_ct_font()`
  does, with all the caveats listed there (12-point default, cascade-list
  rewrite, `NULL` for fonts Core Graphics cannot represent).
* **Failure is silent and destructive.** If the `CTFont` cannot be created the
  function installs `hb_font_funcs_get_empty()` instead — a font-functions
  object that answers nothing. The font will then map every character to no
  glyph and report zero metrics. Check `hb_coretext_font_get_ct_font()` first if
  you need to know.
* `hb_coretext_font_create()` deliberately does **not** call this; the HarfBuzz
  default (OpenType) font functions are the intended behaviour for
  CoreText-created fonts.
* The functions installed are: nominal glyph (single and batched), variation
  glyph, horizontal font extents, horizontal glyph advances, glyph extents, and
  — subject to HarfBuzz's own `HB_NO_VERTICAL`, `HB_NO_DRAW`, and
  `HB_NO_OT_FONT_GLYPH_NAMES` build switches — vertical glyph advances, vertical
  glyph origin, glyph drawing, and glyph name lookup in both directions. Notably
  absent: kerning, glyph origins for horizontal layout, and `paint`.

## Usage

### Shaping with a `CTFontRef` you already have (C)

```c
#include <hb.h>
#include <hb-coretext.h>
#include <CoreText/CoreText.h>

CTFontRef ct_font = CTFontCreateWithName (CFSTR ("Helvetica"), 24.0, NULL);

hb_font_t *font = hb_coretext_font_create (ct_font);
CFRelease (ct_font);            /* the hb_font_t took its own retain */

/* hb_coretext_font_create() leaves the scale at the face's upem, so
 * positions come back in font units. Ask for 26.6 fixed-point pixels
 * instead, matching what most renderers want. */
hb_font_set_scale (font, 24 * 64, 24 * 64);

hb_buffer_t *buf = hb_buffer_create ();
hb_buffer_add_utf8 (buf, "Hello, world", -1, 0, -1);
hb_buffer_guess_segment_properties (buf);

hb_shape (font, buf, NULL, 0);

unsigned int len;
hb_glyph_info_t     *info = hb_buffer_get_glyph_infos (buf, &len);
hb_glyph_position_t *pos  = hb_buffer_get_glyph_positions (buf, &len);

hb_buffer_destroy (buf);
hb_font_destroy (font);
```

### Wrapping a `CGFontRef` as a face (C)

```c
CGFontRef cg_font = CGFontCreateWithFontName (CFSTR ("Helvetica"));

hb_face_t *face = hb_coretext_face_create (cg_font);
CGFontRelease (cg_font);        /* the hb_face_t took its own retain */

hb_font_t *font = hb_font_create (face);
hb_face_destroy (face);         /* the font holds a reference */

/* ... shape ... */

hb_font_destroy (font);
```

### Going the other way: shape with HarfBuzz, draw with Core Text (C)

```c
/* Any hb_font_t will do — it does not have to have come from this header. */
CTFontRef ct_font = hb_coretext_font_get_ct_font (font);
if (!ct_font)
  return;                       /* Core Text cannot represent this font */

/* Do NOT CFRelease ct_font: it is owned by `font`. Retain it if it must
 * outlive `font`. */
CGGlyph  glyph = (CGGlyph) info[i].codepoint;
CGPoint  point = CGPointMake (x + pos[i].x_offset / 64.0,
                              y + pos[i].y_offset / 64.0);
CTFontDrawGlyphs (ct_font, &glyph, &point, 1, cg_context);
```

### Loading a font file through Core Text (C)

```c
/* index 0: first (and only) face, default variable-font instance. */
hb_face_t *face =
  hb_coretext_face_create_from_file_or_fail ("/Library/Fonts/Skia.ttf", 0);
if (!face)
  { /* unreadable, or not a font Core Text accepts */ }

/* Named instance 2 of a variable font: high 16 bits. */
hb_face_t *inst =
  hb_coretext_face_create_from_file_or_fail ("/Library/Fonts/Skia.ttf",
                                             (2u << 16));
```

### Rust: a `CTFontRef` from the `core-text` crate

Both crates spell the type as a plain pointer, so a `cast()` is the whole
conversion.

```rust
use core::ptr;
use harfbuzz_sys::coretext::{CTFontRef, hb_coretext_font_create};
use harfbuzz_sys::{
    hb_buffer_add_utf8, hb_buffer_create, hb_buffer_destroy,
    hb_buffer_get_glyph_infos, hb_buffer_get_glyph_positions,
    hb_buffer_guess_segment_properties, hb_font_destroy, hb_font_set_scale,
    hb_shape,
};

// `ct` is a `core_text::font::CTFont`; `.as_concrete_TypeRef()` yields the
// framework pointer, which is layout-compatible with our alias.
let ct_font: CTFontRef = ct.as_concrete_TypeRef().cast();

unsafe {
    let font = hb_coretext_font_create(ct_font);
    // `font` took its own CFRetain; `ct` still owns the caller's reference.

    hb_font_set_scale(font, 24 * 64, 24 * 64);

    let buf = hb_buffer_create();
    hb_buffer_add_utf8(buf, c"Hello, world".as_ptr(), -1, 0, -1);
    hb_buffer_guess_segment_properties(buf);

    hb_shape(font, buf, ptr::null(), 0);

    let mut len = 0;
    let infos = hb_buffer_get_glyph_infos(buf, &mut len);
    let positions = hb_buffer_get_glyph_positions(buf, &mut len);
    let infos = core::slice::from_raw_parts(infos, len as usize);
    let positions = core::slice::from_raw_parts(positions, len as usize);

    hb_buffer_destroy(buf);
    hb_font_destroy(font);
}
```

### Rust: borrowing the `CTFontRef` back out for rendering

```rust
use harfbuzz_sys::coretext::hb_coretext_font_get_ct_font;

unsafe {
    let ct_font = hb_coretext_font_get_ct_font(font);
    if ct_font.is_null() {
        // Core Text could not build a font for this face.
        return;
    }
    // Borrowed, not owned: do not CFRelease. If it must outlive `font`,
    // CFRetain it (and wrap it in whatever RAII type your CF crate provides).
    render_with_core_text(ct_font);
}
```

### Rust: Core Text metrics instead of HarfBuzz's

```rust
use harfbuzz_sys::coretext::{hb_coretext_font_get_ct_font, hb_coretext_font_set_funcs};

unsafe {
    // Guard first: set_funcs fails silently by installing the empty funcs.
    if hb_coretext_font_get_ct_font(font).is_null() {
        // Leave the font on HarfBuzz's own font functions.
    } else {
        hb_coretext_font_set_funcs(font);
    }
}
```

### Rust: checking for AAT tables by tag

```rust
use harfbuzz_sys::coretext::{HB_CORETEXT_TAG_KERX, HB_CORETEXT_TAG_MORX};
use harfbuzz_sys::{hb_blob_destroy, hb_blob_get_length, hb_face_reference_table};

unsafe {
    for tag in [HB_CORETEXT_TAG_MORX, HB_CORETEXT_TAG_KERX] {
        let blob = hb_face_reference_table(face, tag);
        let present = hb_blob_get_length(blob) != 0;
        hb_blob_destroy(blob);
        println!("{:?}: {present}", harfbuzz_sys::HB_UNTAG(tag));
    }
}
```

## Pitfalls

**The getters are not free, and they mutate.** `hb_coretext_face_get_cg_font()`
and `hb_coretext_font_get_ct_font()` read like cheap accessors but will parse
font data and call into Core Text on their first invocation, caching the result
on the object. Do not call them in a hot loop expecting a pointer load, and do
not assume a `const` face or font is safe to hand them concurrently with other
first-touch operations (it is, thanks to the atomic slot, but the object *is*
being written to).

**Neither getter transfers ownership.** Releasing the returned `CGFontRef` or
`CTFontRef` will over-release it and crash later, when the HarfBuzz object
drops its own reference. Conversely, holding one past the death of its face or
font is a use-after-free unless you retained it.

**Collections are not supported.** Core Text cannot read TTC/OTC files. Both
`_or_fail` constructors return `NULL` when the low 16 bits of `index` are
non-zero, and `hb_coretext_face_get_cg_font()` returns `NULL` for a face whose
index selects a collection member. Use HarfBuzz's own `hb_face_create*`
functions for collections.

**`index` packs two different numbers.** Low 16 bits = collection index (must be
zero here); high 16 bits = named-instance index of a variable font. Passing a
"face index" of 1 meaning "second face in the collection" silently fails;
passing `1 << 16` means "first named instance". Named-instance loading also
requires a recent deployment target when going through a blob.

**`hb_coretext_face_create()` cannot fail visibly.** It returns the singleton
empty face on allocation failure and accepts a null `CGFontRef` without
complaint. If you need to know whether you got something usable, check the face
afterwards — for example that `hb_face_get_table_tags()` reports a non-zero
count, or that `hb_face_reference_table()` for a required table is non-empty.
The two `_or_fail` constructors do return `NULL`, which is why they are the
better default.

**`hb_coretext_font_set_funcs()` degrades to nothing.** When it cannot build a
`CTFont` it installs the *empty* font functions rather than leaving the existing
ones in place. The font then shapes to notdef/zero everywhere. Because the
function returns `void`, the only way to see this coming is to check
`hb_coretext_font_get_ct_font()` first.

**Scale is not size.** `hb_coretext_font_create()` sets `ptem` from the
`CTFontRef`'s point size but leaves the scale at the face's units-per-em, so
shaped positions are in font units. That is easy to mistake for "Core Text gave
me pixels". Call `hb_font_set_scale()` explicitly.

**The blob is borrowed, not copied.**
`hb_coretext_face_create_from_blob_or_fail()` makes the blob immutable and
references it; Core Graphics reads directly out of its memory for the lifetime
of the resulting font. If the blob wraps memory you own with
`HB_MEMORY_MODE_WRITABLE` or a custom destroy callback, that memory must remain
valid and unmodified for as long as the face lives.

**AAT does not require this header.** HarfBuzz shapes `morx`/`kerx` fonts
natively on Linux and Windows too. Reaching for CoreText integration because a
font is AAT is a mistake; reach for it because you already have Apple font
objects, or because you specifically want Apple's parser or Apple's metrics.

**The `coretext` shaper is a different thing.** HarfBuzz's shaper back end named
`"coretext"` — the one `hb_shape_plan_get_shaper()` can report and `hb-shape
--shapers=coretext` selects — hands shaping wholesale to Core Text for
comparison testing. It is unrelated to the functions on this page, and enabling
this crate's `coretext` feature does not make ordinary `hb_shape()` calls use
it.

**Platform gating.** The `coretext` Cargo feature is silently ignored by
`build.rs` on non-Apple targets (with a `cargo::warning`), because Cargo
features are additive and an unrelated crate in the graph may have switched it
on. On such a target `harfbuzz_sys::coretext` still exists as a module but none
of its functions have symbols behind them — referencing one is a link error.

**Macros.** Beyond the three tag constants, `hb-coretext.h` defines only its
include guard and the `HB_BEGIN_DECLS`/`HB_END_DECLS`/`HB_EXTERN` boilerplate,
so nothing was skipped in the transcription. The header's `TARGET_OS_IPHONE`
conditional selects which Apple umbrella header to include and does not affect
any declaration.
