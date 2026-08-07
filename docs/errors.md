# Errors

HarfBuzz reports problems in three different ways, none of them exceptions and
none of them a `Result`. This crate folds all three into one `Error` enum — and,
just as importantly, tells you which failures stay silent because HarfBuzz gives
the wrapper nothing to detect.

- [`Error` and `Result`](#error-and-result)
- [Every variant](#every-variant)
- [HarfBuzz's three failure styles](#harfbuzzs-three-failure-styles)
- [The failures that stay silent](#the-failures-that-stay-silent)
- [Handling them](#handling-them)

---

## `Error` and `Result`

```rust,ignore
pub type Result<T, E = Error> = core::result::Result<T, E>;

#[non_exhaustive]
pub enum Error { /* … */ }
```

`Error` derives `Debug`, `Clone`, `PartialEq`, and `Eq`, implements `Display`,
and implements `core::error::Error` — so it drops straight into `anyhow`,
`thiserror`, `Box<dyn Error>`, and `?` in a `main` that returns
`Result<(), Box<dyn Error>>`.

It is `#[non_exhaustive]`: a `match` over it needs a `_` arm, and new variants
can appear in a minor release.

## Every variant

| Variant | Raised by | Means | What to do |
| --- | --- | --- | --- |
| `AllocationFailed` | `Blob::from_bytes`, `buffer_from` | HarfBuzz could not allocate — or a constructor returned the inert "empty" singleton, which is how it signals allocation failure without returning null | Nothing local will help. Propagate. |
| `FontLoadFailed` | `Blob::from_file`, `Face::new`, `Face::from_file`, `Face::from_bytes` | The file could not be read, **or** the bytes are not a font HarfBuzz recognises. The C API does not distinguish the two, so neither can this | Check the path exists and the data is a real font; fall back to another font |
| `NoSuchFace { requested, available }` | `Face::new` and its convenience wrappers | The face index is past the end of a collection. `available` is the true count | Clamp the index, or iterate `0..blob.face_count()` |
| `InvalidTag` | `"…".parse::<Tag>()` | Not one to four printable-ASCII characters | Fix the literal — this is a programming error |
| `InvalidDirection` | `"…".parse::<Direction>()` | The first character was not `l`, `r`, `t`, or `b` | As above |
| `InvalidLanguage` | `"…".parse::<Language>()` | Only ever the empty string | As above |
| `InvalidScript` | `"…".parse::<Script>()` | Only ever the empty string — see the [gotchas](values.md#script) | As above |
| `InvalidFeature` | `"…".parse::<Feature>()` | Not `hb-shape` feature syntax | As above |
| `InvalidVariation` | `"…".parse::<Variation>()` | Not `axis=value` | As above |
| `InvalidText` | reserved for text-decoding paths | Bytes were not valid UTF-8, UTF-16, or UTF-32 | Decode before pushing; `push_str` takes `&str`, so it cannot happen there |
| `SubsetFailed` | reserved for the `subset` feature | HarfBuzz reports subsetting failure as one boolean, with no detail | Retry with a smaller request, or ship the whole font |
| `UnknownShaper` | reserved for shaper selection | A shaper was requested by name that this build does not have | Check `shapers()` for what is available |

The last three describe failure modes the safe API does not currently expose a
path to; they exist so that adding those paths does not need a breaking change.
`shapers()` lists what the current build has:

```rust
use harfbuzz_rs::shapers;

fn main() {
    // "ot" — HarfBuzz's own OpenType shaper — is always present.
    assert!(shapers().contains(&"ot"));
    println!("{:?}", shapers());
}
```

## HarfBuzz's three failure styles

Understanding these explains why some calls return `Result` and others cannot.

### 1. Constructors that never fail

`hb_buffer_create()`, `hb_font_create()`, and friends never return null. On
allocation failure they hand back a shared, inert **"empty" singleton** — an
object that accepts every call and ignores it. The intent is that C callers need
not null-check.

For Rust that is worse than an error: you would get a `Font` that silently
shapes nothing. Where the wrapper can detect the singleton it converts it into
`Error::AllocationFailed`; where the operation cannot fail for any other reason
— `Buffer::new`, `Font::new` — the constructor is infallible and returns the
value directly.

```rust
use harfbuzz_rs::{Buffer, Face, Font, IntoShared};

fn main() -> harfbuzz_rs::Result<()> {
    let face = Face::from_file("font.ttf", 0)?;   // fallible: I/O and parsing
    let font = Font::new(face.into_shared());     // infallible
    let buffer = Buffer::new();                   // infallible

    let _ = (font, buffer);
    Ok(())
}
```

### 2. `*_or_fail` constructors

Upstream added `hb_blob_create_or_fail` and `hb_face_create_or_fail` for callers
who would rather know. They return null on failure. The wrapper uses them
wherever they exist, which is why `Blob::from_bytes`, `Blob::from_file`, and
`Face::new` return `Result`.

`Face::new` goes one step further: HarfBuzz cannot say *why* it failed, so the
wrapper checks the blob's face count and reports the common mistake precisely.

```rust
use harfbuzz_rs::{Blob, Error, Face};

fn main() -> harfbuzz_rs::Result<()> {
    let blob = Blob::from_file("font.ttf")?;

    match Face::new(&blob, 7) {
        Ok(face) => println!("{} glyphs", face.glyph_count()),
        Err(Error::NoSuchFace { requested, available }) => {
            println!("asked for face {requested}, file has {available}");
        }
        Err(other) => return Err(other),
    }

    Ok(())
}
```

A face count of zero means the data is not a font at all, so that case is
reported as `FontLoadFailed` rather than a misleading `NoSuchFace`.

### 3. Accumulating failure flags

Buffers record allocation failures internally and keep going, so a long series
of pushes can be checked once at the end rather than after every call. That is
what `Buffer::allocation_successful()` is for:

```rust
use harfbuzz_rs::{Buffer, Error};

fn build(lines: &[&str]) -> harfbuzz_rs::Result<Buffer> {
    let mut buffer = Buffer::new();

    for line in lines {
        buffer.push_str(line);
    }

    // One check for the whole batch.
    if !buffer.allocation_successful() {
        return Err(Error::AllocationFailed);
    }

    buffer.guess_segment_properties();
    Ok(buffer)
}
```

`buffer_from` does exactly this for the single-string case, which is why it
returns `Result` even though `push_str` does not.

## The failures that stay silent

Some things cannot fail loudly, and knowing which is the difference between
debugging for five minutes and debugging for a day.

| Situation | What happens | How to detect it |
| --- | --- | --- |
| Setter called on a frozen object | **Compile error** — this crate's central design choice | The compiler. See [ownership-and-threads.md](ownership-and-threads.md) |
| Character not in the font | Shapes to glyph `0` (`.notdef`) | `info.glyph() == 0` after shaping, or `Font::nominal_glyph(ch).is_none()` before |
| Missing font table | `Face::table` returns an **empty blob**, not an error | `face.table(tag).is_empty()` |
| Font has no glyph names | `Font::glyph_name` returns `None` | Common in subset and web fonts; not a failure |
| Feature the font does not implement | Silently ignored | No way to tell from the output |
| Variation axis the font does not have | Silently ignored | Check for an `fvar` table: `!face.table(Tag::new(b"fvar")).is_empty()` |
| Variation value out of range | **Clamped** by the font | Read the axis range through the raw API |
| Shaping with unset segment properties | Wrong output, no error | `guess_segment_properties()`, or set them explicitly |
| Buffer cleared, properties forgotten | Second run shapes with no properties | Re-guess or re-set on every pass — see [text-and-buffers.md](text-and-buffers.md#reuse) |
| Two `push_str` calls | Cluster numbering restarts | Push the run in one call |
| Glyph extents for a nonexistent glyph | `None` | `Option`, not `Result` |
| Font with no vertical metrics | `Font::extents()` returns zeroes | `extents.ascender == 0` |

The two worth building a habit around:

```rust
use harfbuzz_rs::{Font, buffer_from, shape};

/// Whether `font` can render every character of `text` — the check to make
/// before committing to a font in a fallback chain.
fn covers(font: &Font, text: &str) -> harfbuzz_rs::Result<bool> {
    let output = shape(font, buffer_from(text)?, &[]);
    Ok(output.infos().iter().all(|info| info.glyph() != 0))
}
```

```rust
use harfbuzz_rs::{Face, Tag};

fn is_variable(face: &Face) -> bool {
    // A missing table is an empty blob, never an error.
    !face.table(Tag::new(b"fvar")).is_empty()
}
```

## Handling them

### With `?`

`Error` implements `core::error::Error`, so it converts into `Box<dyn Error>`
and into `anyhow::Error` automatically:

```rust
use harfbuzz_rs::{Face, Font, IntoShared, buffer_from, shape};

fn run() -> Result<(), Box<dyn core::error::Error>> {
    let face = Face::from_file("font.ttf", 0)?.into_shared();
    let font = Font::new(face);

    let output = shape(&font, buffer_from("hello")?, &[]);
    println!("{} glyphs", output.len());
    Ok(())
}
```

Inside the crate's own idiom, `harfbuzz_rs::Result<T>` is the short form:

```rust
use harfbuzz_rs::{Face, Result};

fn load(path: &str) -> Result<Face> {
    Face::from_file(path, 0)
}
```

### Matching, with the `#[non_exhaustive]` arm

```rust
use harfbuzz_rs::{Error, Face};

fn load_or_fallback(path: &str, fallback: &str) -> harfbuzz_rs::Result<Face> {
    match Face::from_file(path, 0) {
        Ok(face) => Ok(face),

        // A missing or unreadable font: try the fallback.
        Err(Error::FontLoadFailed) => Face::from_file(fallback, 0),

        // A collection index out of range: take the first face instead.
        Err(Error::NoSuchFace { .. }) => Face::from_file(path, 0),

        // `Error` is `#[non_exhaustive]`, so this arm is required.
        Err(other) => Err(other),
    }
}
```

### Distinguishing user input from programmer error

The `Invalid*` variants come from parsing string literals you almost always
wrote yourself. If the string is a constant, prefer the constructors that cannot
fail:

```rust
use harfbuzz_rs::{Feature, Tag, Variation};

fn main() {
    // Fallible: parsed at run time.
    let _from_config: Feature = "-liga".parse().expect("literal is valid");

    // Infallible: checked at compile time.
    let _direct = Feature::new(Tag::new(b"liga"), 0, ..);
    let _axis = Variation::new(Tag::new(b"wght"), 700.0);
}
```

`Tag::new` takes a `&[u8; 4]`, so a wrong-length literal is a compile error
rather than an `Error::InvalidTag` at run time.

---

The C-level conventions behind all of this — the empty singletons, the
`_or_fail` variants, and the buffer's failure flag — are described in
[`../harfbuzz-sys/docs/blob.md`](../harfbuzz-sys/docs/blob.md),
[`../harfbuzz-sys/docs/face.md`](../harfbuzz-sys/docs/face.md), and
[`../harfbuzz-sys/docs/buffer.md`](../harfbuzz-sys/docs/buffer.md).
