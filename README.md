# harfbuzz-rs

Safe, idiomatic Rust bindings for [HarfBuzz](https://github.com/harfbuzz/harfbuzz)
14.3.0, the text shaping engine.

Shaping is the step between *"here is a string and a font"* and *"here are the
glyphs, and here is where each one goes"*. It is where ligatures form, where
Arabic letters take their positional forms, where Indic syllables reorder, and
where kerning is applied.

```rust
use harfbuzz_rs::{Face, Font, IntoShared, buffer_from, shape};

let face = Face::from_file("font.ttf", 0)?.into_shared();
let font = Font::new(face);

let output = shape(&font, buffer_from("Hello, world!")?, &[]);

for (info, position) in output.iter() {
    println!("glyph {:>4}  advance {:>5}", info.glyph(), position.x_advance());
}
```

## Layout

| Crate          | Purpose                                                          |
| -------------- | ---------------------------------------------------------------- |
| `harfbuzz-sys` | Raw FFI for all 38 public headers, plus a vendored HarfBuzz build |
| `harfbuzz-rs`  | The safe wrapper: ownership, lifetimes, and error handling        |

HarfBuzz is vendored as a git submodule pinned to a tagged release, so a clone
needs `--recursive`:

```sh
git clone --recursive https://github.com/Chord-Ink/harfbuzz-rs
```

## Design

**Misuse is a compile error, not a silent no-op.** HarfBuzz objects follow one
lifecycle: create, configure with a few setters, then use without further
modification. An object can be marked immutable, after which setters *fail
silently* — and there is no way back.

This crate puts that rule in the type system. An owned `Face` or `Font` is a
unique handle whose setters take `&mut self`. `into_shared()` freezes it and
returns a `Shared<T>`, which is `Clone`, dereferences to `&T`, and so reaches
the accessors but none of the setters.

```rust
let mut face = Face::from_file("font.ttf", 0)?;
face.set_upem(1000);            // fine: we hold the only handle

let face = face.into_shared();  // frozen from here on
let clone = face.clone();       // cheap: bumps HarfBuzz's own refcount
// clone.set_upem(2048);        // will not compile
```

`Buffer` and `GlyphBuffer` are separate types for the same reason: a buffer
holds code points before shaping and glyphs after, so splitting them stops you
reading glyph output before shaping, or adding text after it.

## Documentation

- [`docs/`](docs/) — the `harfbuzz-rs` API, with guides on the object model,
  ownership and threading, buffers and clusters, and sizing.
- [`harfbuzz-sys/docs/`](harfbuzz-sys/docs/) — the complete HarfBuzz **C API
  reference manual**, one page per header, plus 19 conceptual guides adapted
  from upstream's user manual.

## Features

Each feature maps onto one of HarfBuzz's optional sub-libraries or back ends,
and turns on both the C sources and the Rust that wraps them.

| Feature               | Effect                                        |
| --------------------- | --------------------------------------------- |
| `subset`              | Font subsetting and instancing                 |
| `raster`              | CPU glyph rasterization                        |
| `vector`              | SVG and PDF glyph output                       |
| `gpu`                 | GPU-oriented outline extraction                |
| `coretext`            | Apple CoreText shaper and font backend         |
| `freetype`            | FreeType font backend                          |
| `graphite2`           | Graphite2 complementary shaper                 |
| `icu`, `glib`         | Unicode data from ICU or GLib                  |
| `png`, `zlib`         | Codec support for the raster and vector output |
| `debug`               | Build HarfBuzz with debug info + frame pointers |
| `experimental`        | Upstream APIs with no compatibility promise    |
| `mini`, `lean`, `tiny`| Size profiles that **remove** public API       |

The features needing a system library (`freetype`, `graphite2`, `icu`, `glib`,
`png`, `zlib`) locate it with `pkg-config`.

## Building

HarfBuzz is C++, so the build uses `cc::Build::cpp(true)`, which reads `CXX`:

```sh
cargo build
CXX=/path/to/clang++ cargo build      # a specific compiler
```

It is compiled from upstream's `src/harfbuzz-world.cc` amalgamation — one
translation unit for the whole library — at `-Oz` with `-ffunction-sections`
and `-fdata-sections`, so the linker's dead-stripping can drop everything
unused.

### Cross-language LTO

The `lto` feature compiles HarfBuzz to LLVM bitcode with `-flto=thin` so it can
be optimized together with the Rust calling it. Its requirements are strict and
unchecked — see the comments on the feature in `harfbuzz-sys/Cargo.toml`. In
short: `CXX` must be an upstream LLVM `clang++` whose major version matches the
`LLVM version` line of `rustc -vV`, `AR` must be `llvm-ar`, and the consumer
must pass `-Clinker-plugin-lto`.

Note that on Apple targets the final link cannot currently succeed: `rustc`
translates `-Clinker-plugin-lto` into GNU-style `-Wl,-plugin-opt=`, which no
Mach-O linker accepts. Cross-language LTO is reachable on ELF targets.

### Verification

`harfbuzz-sys` checks every one of its 727 `extern "C"` declarations against the
symbol table of the archive it links:

```sh
cargo test -p harfbuzz-sys
```

An `extern` declaration is otherwise an unchecked promise — Rust only notices a
wrong name when something calls it, so a typo can sit undetected until a
downstream user hits a link error.

## Licence

MIT. HarfBuzz itself is under its own [licence](harfbuzz-sys/harfbuzz/COPYING).
The test font under `tests/fonts/` is a subset of Roboto from HarfBuzz's test
corpus, under the Apache License 2.0.
