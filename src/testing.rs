//! Shared fixtures for the crate's own tests.
//!
//! The font under `tests/fonts/` is a subset of Roboto taken from HarfBuzz's
//! own test corpus, licensed under the Apache License 2.0. It is small on
//! purpose — 8 glyphs and a weight axis — which is enough to shape real text
//! and exercise variations without adding a megabyte to the repository.
//!
//! Its character map covers `A`, `B`, and `C` only. Tests that need a glyph
//! must use those; lowercase letters map to `.notdef`.

use crate::blob::Blob;
use crate::face::Face;
use crate::object::IntoShared;
use crate::font::Font;

/// A subset of Roboto with a variable weight axis.
const VARIABLE: &[u8] = include_bytes!("../tests/fonts/Roboto-Variable.abc.ttf");

/// The raw bytes of the test font.
pub(crate) fn font_data() -> Blob {
    Blob::from_bytes(VARIABLE.to_vec()).expect("test font is valid")
}

/// A face built from the test font.
pub(crate) fn face() -> Face {
    Face::from_bytes(VARIABLE.to_vec(), 0).expect("test font is a valid face")
}

/// A font built from the test font, scaled to its design units so that shaped
/// advances come back in font units and are easy to reason about.
pub(crate) fn font() -> Font {
    Font::new(face().into_shared())
}
