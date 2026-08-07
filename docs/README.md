# harfbuzz-rs

Safe, idiomatic Rust bindings for [HarfBuzz](https://harfbuzz.github.io/), the
text shaping engine.

**Shaping** is the step between *"here is a string and a font"* and *"here are
the glyphs, and here is where each one goes"*. It is where ligatures form, where
Arabic letters take their positional forms, where Indic syllables reorder, and
where kerning is applied. HarfBuzz is the implementation almost everything uses
— browsers, Android, LibreOffice, most game engines — and this crate is a safe
wrapper over it.

This crate does **not** rasterize glyphs. It tells you *which* glyph to draw and
*where*; turning a glyph ID into pixels is the job of a rasterizer such as
FreeType, Skia, or `swash`.

---

## Install

```toml
[dependencies]
harfbuzz-rs = { git = "https://github.com/Chord-Ink/harfbuzz-rs" }
```

HarfBuzz itself is vendored as a git submodule and compiled by
`harfbuzz-sys/build.rs`, so there is no system dependency to install — but a
clone needs its submodules:

```sh
git clone --recursive https://github.com/Chord-Ink/harfbuzz-rs
```

Requires Rust 1.85 or newer (the crate is edition 2024) and a working C++
toolchain.

### Features

Every feature forwards to `harfbuzz-sys`, where it switches on an optional slice
of the C library and the raw FFI module that declares it. None of them are on by
default; the safe API in this crate is entirely feature-independent, so enabling
one only adds items under [`harfbuzz_rs::sys`](#the-raw-ffi-escape-hatch).

| Feature | Effect | Needs a system library |
| --- | --- | --- |
| `subset` | Font subsetting and instancing | no |
| `raster` | CPU glyph rasterization | no |
| `vector` | SVG and PDF glyph output | no |
| `gpu` | GPU-oriented outline extraction | no |
| `coretext` | Apple CoreText shaper and font backend | Apple SDK |
| `freetype` | FreeType font backend | `freetype2` |
| `graphite2` | Graphite2 complementary shaper | `graphite2` |
| `icu` | Unicode data from ICU | `icu-uc` |
| `glib` | Unicode data from GLib | `glib-2.0` |
| `png` | PNG decoding for the raster/vector back ends | `libpng` |
| `zlib` | Compression for the vector back end | `zlib` |
| `debug` | Build HarfBuzz with debug info and frame pointers | no |
| `experimental` | Upstream APIs with no stability promise | no |

---

## The whole thing, end to end

```rust
use harfbuzz_rs::{Face, Font, IntoShared, buffer_from, points_to_scale, shape};

fn main() -> harfbuzz_rs::Result<()> {
    let face = Face::from_file("font.ttf", 0)?.into_shared();

    let mut font = Font::new(face);
    font.set_scale(points_to_scale(16), points_to_scale(16));

    let output = shape(&font, buffer_from("Hello, world!")?, &[]);

    let mut pen_x = 0;
    for (info, position) in output.iter() {
        let x = (pen_x + position.x_offset()) as f32 / 64.0;
        println!("glyph {:>4} at x={x:>7.2}", info.glyph());
        pen_x += position.x_advance();
    }

    Ok(())
}
```

Four objects do the work, and they stack: **bytes** become a `Blob`, a blob
becomes a `Face`, a face at a size becomes a `Font`, and a `Font` plus a
`Buffer` of text produces a `GlyphBuffer` of positioned glyphs.

---

## Start here

Read in this order if you are new to the crate, or to shaping:

1. **[Getting started](getting-started.md)** — zero to positioned glyphs,
   including what the numbers mean.
2. **[The object model](object-model.md)** — what each type owns, what is
   expensive, what to cache.
3. **[Ownership and threads](ownership-and-threads.md)** — the design idea at
   the centre of the crate. Read this before writing anything concurrent.
4. Then whichever of **[text and buffers](text-and-buffers.md)**,
   **[fonts and sizing](fonts-and-sizing.md)**, or **[values](values.md)** your
   problem lives in.
5. **[Errors](errors.md)** when something returns a `Result` you did not expect
   — or worse, silently does nothing.

---

## Contents

| Page | Covers |
| --- | --- |
| [getting-started.md](getting-started.md) | Loading a font, building a font at a size, shaping a string, reading advances and clusters, converting to pen positions |
| [object-model.md](object-model.md) | `Blob` → `Face` → `Font` → `Buffer`/`GlyphBuffer`; ownership graph, relative costs, caching strategy |
| [ownership-and-threads.md](ownership-and-threads.md) | HarfBuzz's create/configure/freeze lifecycle, `IntoShared`, `Shared<T>`, `ThreadSafeWhenImmutable`, `Send`/`Sync` matrix, a worked threaded example |
| [text-and-buffers.md](text-and-buffers.md) | Buffer lifecycle, adding text, segment properties, clusters, cursor placement and hit-testing, shaping fragments with context, buffer reuse |
| [fonts-and-sizing.md](fonts-and-sizing.md) | Face vs font, units-per-em, `set_scale` and 26.6 fixed point, ppem, variable-font axes, font and glyph metrics |
| [values.md](values.md) | `Tag`, `Direction`, `Script`, `Language`, `Feature`, `Variation`: construction, parsing, `Display`, and their sharp edges |
| [errors.md](errors.md) | Every `Error` variant, HarfBuzz's three failure styles, and the failures that are silent by design |

### API index

Every public item in the crate, and the page that explains it.

| Item | Kind | Page |
| --- | --- | --- |
| `Blob` | object | [object-model.md](object-model.md#blob) |
| `Face` | object | [object-model.md](object-model.md#face), [fonts-and-sizing.md](fonts-and-sizing.md) |
| `Font` | object | [fonts-and-sizing.md](fonts-and-sizing.md) |
| `Buffer`, `GlyphBuffer` | objects | [text-and-buffers.md](text-and-buffers.md) |
| `GlyphInfo`, `GlyphPosition` | output data | [text-and-buffers.md](text-and-buffers.md#reading-the-output) |
| `FontExtents`, `GlyphExtents` | metrics | [fonts-and-sizing.md](fonts-and-sizing.md#metrics) |
| `shape`, `shapers` | functions | [getting-started.md](getting-started.md#step-4-shape) |
| `buffer_from` | function | [text-and-buffers.md](text-and-buffers.md#adding-text) |
| `points_to_scale` | function | [fonts-and-sizing.md](fonts-and-sizing.md#scale-and-the-266-convention) |
| `version`, `version_at_least` | functions | [below](#version) |
| `Tag`, `Direction`, `Script`, `Language`, `Feature`, `Variation` | value types | [values.md](values.md) |
| `Error`, `Result` | errors | [errors.md](errors.md) |
| `Shared<T>`, `IntoShared` | ownership | [ownership-and-threads.md](ownership-and-threads.md) |
| `HarfBuzzObject`, `ThreadSafeWhenImmutable` | unsafe traits | [ownership-and-threads.md](ownership-and-threads.md#the-unsafe-traits) |
| `sys` | re-export | [below](#the-raw-ffi-escape-hatch) |

---

## Version

```rust
let (major, minor, micro) = harfbuzz_rs::version();
assert!(major >= 14);

assert!(harfbuzz_rs::version_at_least(14, 0, 0));
```

These report the version of the **vendored HarfBuzz** the crate was compiled
against, not the crate's own version. See
[`../harfbuzz-sys/docs/version.md`](../harfbuzz-sys/docs/version.md) for the
compile-time constants.

## The raw FFI escape hatch

The safe API deliberately covers the shaping path and little else. Everything
HarfBuzz exposes is still reachable through the re-exported `harfbuzz_sys`:

```rust
use harfbuzz_rs::{Face, HarfBuzzObject, sys};

fn table_count(face: &Face) -> u32 {
    // SAFETY: `face` owns a live `hb_face_t` for the duration of the call, and
    // this function only reads from it.
    unsafe { sys::hb_face_get_table_tags(face.as_raw(), 0, &mut 0, core::ptr::null_mut()) }
}
```

`as_raw()` comes from the [`HarfBuzzObject`](ownership-and-threads.md#the-unsafe-traits)
trait and borrows the pointer without touching the reference count — the wrapper
still owns it, so do **not** call `hb_*_destroy` on it.

The raw layer has its own reference documentation, one page per C header, under
[`../harfbuzz-sys/docs/`](../harfbuzz-sys/docs/). The most useful starting
points:

| C API page | Subject |
| --- | --- |
| [`blob.md`](../harfbuzz-sys/docs/blob.md) | `hb_blob_t`, memory modes, destroy callbacks |
| [`face.md`](../harfbuzz-sys/docs/face.md) | `hb_face_t`, tables, face builders |
| [`font.md`](../harfbuzz-sys/docs/font.md) | `hb_font_t`, font funcs, synthetic styles |
| [`buffer.md`](../harfbuzz-sys/docs/buffer.md) | `hb_buffer_t`, flags, cluster levels, serialization |
| [`shape.md`](../harfbuzz-sys/docs/shape.md) | `hb_shape`, `hb_shape_full`, shaper selection |
| [`ot_var.md`](../harfbuzz-sys/docs/ot_var.md) | Variable-font axes and named instances |
| [`draw.md`](../harfbuzz-sys/docs/draw.md) | Glyph outlines, for when you need to rasterize |
| [`guide/`](../harfbuzz-sys/docs/guide/) | Upstream's conceptual manual, including [clusters](../harfbuzz-sys/docs/guide/08-clusters.md) and [OpenType features](../harfbuzz-sys/docs/guide/09-opentype-features.md) |
