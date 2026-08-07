//! Safe, idiomatic Rust bindings for [HarfBuzz](https://harfbuzz.github.io/),
//! the text shaping engine.
//!
//! Shaping is the step between "here is a string and a font" and "here are the
//! glyphs, and here is where each one goes". It is where ligatures form, where
//! Arabic letters take their positional forms, where Indic syllables reorder,
//! and where kerning is applied. HarfBuzz is the implementation almost
//! everything uses, and this crate is a safe wrapper over it.
//!
//! # The shortest useful program
//!
//! ```no_run
//! use harfbuzz_rs::{Face, Font, IntoShared, buffer_from, shape};
//!
//! let face = Face::from_file("font.ttf", 0)?.into_shared();
//! let font = Font::new(face);
//!
//! let output = shape(&font, buffer_from("Hello, world!")?, &[]);
//!
//! for (info, position) in output.iter() {
//!     println!("glyph {:>4}  advance {:>5}", info.glyph(), position.x_advance());
//! }
//! # Ok::<(), harfbuzz_rs::Error>(())
//! ```
//!
//! # The object model
//!
//! Four types carry the work, and they stack:
//!
//! | Type | What it is |
//! | ---- | ---------- |
//! | [`Blob`] | Bytes, with a lifetime HarfBuzz can track |
//! | [`Face`] | One font from those bytes: tables, glyphs, design units |
//! | [`Font`] | A face at a size, with variation axes pinned |
//! | [`Buffer`] | Text going in; [`GlyphBuffer`] is what comes out |
//!
//! Faces and fonts are worth caching; buffers are worth reusing. A blob can
//! back many faces, a face many fonts, and a font can shape any number of
//! buffers.
//!
//! # Mutability, sharing, and threads
//!
//! HarfBuzz objects follow one pattern: create, configure with a few setters,
//! then use without further modification. An object can be marked immutable,
//! after which setters **fail silently** — and there is no way back.
//!
//! This crate turns that runtime rule into a compile-time one. An owned
//! [`Face`] or [`Font`] is a unique handle whose setters take `&mut self`.
//! [`IntoShared::into_shared`] freezes it and hands back a [`Shared<T>`], which
//! is [`Clone`], dereferences to `&T`, and therefore reaches the accessors but
//! none of the setters. A `Shared<Face>` or `Shared<Font>` is [`Send`] and
//! [`Sync`]; the owned forms are `Send` but not `Sync`.
//!
//! ```no_run
//! # use harfbuzz_rs::{Face, Font, IntoShared};
//! let mut face = Face::from_file("font.ttf", 0)?;
//! face.set_upem(1000);            // fine: we hold the only handle
//!
//! let face = face.into_shared();  // frozen from here on
//! let clone = face.clone();       // cheap: bumps HarfBuzz's own refcount
//! // clone.set_upem(2048);        // will not compile
//! # Ok::<(), harfbuzz_rs::Error>(())
//! ```
//!
//! # Features
//!
//! Every feature maps onto one of HarfBuzz's optional sub-libraries or back
//! ends, and turns on both the C sources and the Rust that wraps them.
//!
//! | Feature | Effect |
//! | ------- | ------ |
//! | `subset` | Font subsetting and instancing |
//! | `raster` | CPU glyph rasterization |
//! | `vector` | SVG and PDF glyph output |
//! | `gpu` | GPU-oriented outline extraction |
//! | `coretext` | Apple CoreText shaper and font backend |
//! | `freetype` | FreeType font backend |
//! | `graphite2` | Graphite2 shaper |
//! | `icu`, `glib` | Unicode data from ICU or GLib |
//! | `debug` | Build HarfBuzz with debug info and frame pointers |

#![deny(missing_docs)]
#![deny(unsafe_op_in_unsafe_fn)]

/// The raw FFI bindings this crate is built on.
///
/// Reach for these when you need something the safe API does not cover yet.
/// Everything in here is `unsafe`.
pub use harfbuzz_sys as sys;

mod blob;
mod buffer;
mod direction;
mod error;
mod face;
mod feature;
mod font;
mod language;
mod object;
mod script;
mod shape;
mod tag;

#[cfg(test)]
mod testing;

pub use blob::Blob;
pub use buffer::{Buffer, GlyphBuffer, GlyphInfo, GlyphPosition, buffer_from};
pub use direction::Direction;
pub use error::{Error, Result};
pub use face::Face;
pub use feature::{Feature, Variation};
pub use font::{Font, FontExtents, GlyphExtents, points_to_scale};
pub use language::Language;
pub use object::{HarfBuzzObject, IntoShared, Shared, ThreadSafeWhenImmutable};
pub use script::Script;
pub use shape::{shape, shapers};
pub use tag::Tag;

/// The version of HarfBuzz this crate was built against.
///
/// ```
/// let (major, _minor, _micro) = harfbuzz_rs::version();
/// assert!(major >= 14);
/// ```
pub fn version() -> (u32, u32, u32) {
    let (mut major, mut minor, mut micro) = (0, 0, 0);

    // SAFETY: `hb_version` writes one `unsigned` through each pointer, and all
    // three point at live, correctly aligned locals.
    unsafe { sys::hb_version(&mut major, &mut minor, &mut micro) };

    (major, minor, micro)
}

/// Whether the HarfBuzz this crate was built against is at least the given
/// version.
pub fn version_at_least(major: u32, minor: u32, micro: u32) -> bool {
    // SAFETY: takes three plain integers and returns a boolean; no pointers.
    unsafe { sys::hb_version_atleast(major, minor, micro) != 0 }
}

#[cfg(test)]
mod tests {
    #[test]
    fn reports_the_vendored_version() {
        assert_eq!(super::version(), (14, 3, 0));
        assert!(super::version_at_least(14, 0, 0));
        assert!(!super::version_at_least(99, 0, 0));
    }
}
