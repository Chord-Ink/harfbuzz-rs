//! Font subsetting: cutting a font down to the glyphs you actually use.
//!
//! Requires the `subset` feature.
//!
//! A web page that draws twenty characters does not need a font carrying
//! thousands. Subsetting produces a new font containing only the glyphs you
//! ask for, with the layout tables rewritten to match — often an order of
//! magnitude smaller than the original.
//!
//! The unit you ask in is usually Unicode: name the characters you need and
//! HarfBuzz works out which glyphs that implies, following ligature and
//! alternate substitutions so the result still shapes correctly.
//!
//! # Examples
//!
//! ```no_run
//! use harfbuzz_rs::{Face, IntoShared};
//! use harfbuzz_rs::subset::Subset;
//!
//! let face = Face::from_file("font.ttf", 0)?.into_shared();
//!
//! let mut subset = Subset::new()?;
//! subset.keep_chars("Hello, world!".chars());
//!
//! let cut_down = subset.apply(&face)?;
//! std::fs::write("subset.ttf", cut_down.blob().as_bytes())?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use harfbuzz_sys as sys;

use crate::error::{Error, Result};
use crate::face::Face;
use crate::object::{HarfBuzzObject, Shared, harfbuzz_object};

harfbuzz_object! {
    /// What to keep when subsetting a font.
    ///
    /// Build one, tell it which characters or glyphs to retain, then hand it a
    /// face with [`Subset::apply`].
    Subset,
    raw: sys::subset::hb_subset_input_t,
    reference: sys::subset::hb_subset_input_reference,
    destroy: sys::subset::hb_subset_input_destroy,
}

impl Subset {
    /// Creates an empty subsetting request.
    ///
    /// Nothing is retained until you say so, beyond the handful of glyphs every
    /// font needs.
    pub fn new() -> Result<Self> {
        // SAFETY: takes no arguments; the `_or_fail` variant reports failure as
        // null rather than substituting an inert singleton.
        let raw = unsafe { sys::subset::hb_subset_input_create_or_fail() };

        if raw.is_null() {
            return Err(Error::AllocationFailed);
        }

        // SAFETY: `raw` is non-null and carries the reference the constructor
        // created.
        Ok(unsafe { Self::from_raw(raw) })
    }

    /// Retains the glyphs needed to render these characters.
    ///
    /// This is the usual way to subset. HarfBuzz maps each character to its
    /// glyph and then follows the font's layout tables, so ligatures and
    /// alternates reachable from your text survive too.
    pub fn keep_chars(&mut self, chars: impl IntoIterator<Item = char>) {
        // SAFETY: `self` owns a live input, and `&mut self` proves exclusivity.
        // The returned set is owned by the input and stays valid for as long as
        // it does; we only borrow it for the loop below.
        let set = unsafe { sys::subset::hb_subset_input_unicode_set(self.as_raw()) };

        for c in chars {
            // SAFETY: `set` is the live set belonging to this input.
            unsafe { sys::hb_set_add(set, c as u32) };
        }
    }

    /// Retains an inclusive range of characters.
    pub fn keep_char_range(&mut self, first: char, last: char) {
        // SAFETY: as in `keep_chars`.
        let set = unsafe { sys::subset::hb_subset_input_unicode_set(self.as_raw()) };

        // SAFETY: `set` is the live set belonging to this input.
        unsafe { sys::hb_set_add_range(set, first as u32, last as u32) };
    }

    /// Retains specific glyphs, by index.
    ///
    /// Prefer [`keep_chars`](Subset::keep_chars) unless you already know the
    /// glyph indices — asking by character lets HarfBuzz pull in everything the
    /// layout tables reach.
    pub fn keep_glyphs(&mut self, glyphs: impl IntoIterator<Item = u32>) {
        // SAFETY: as in `keep_chars`.
        let set = unsafe { sys::subset::hb_subset_input_glyph_set(self.as_raw()) };

        for glyph in glyphs {
            // SAFETY: `set` is the live set belonging to this input.
            unsafe { sys::hb_set_add(set, glyph) };
        }
    }

    /// The flags currently set.
    pub fn flags(&self) -> SubsetFlags {
        // SAFETY: `self` owns a live input.
        SubsetFlags(unsafe { sys::subset::hb_subset_input_get_flags(self.as_raw()) })
    }

    /// Replaces the flags.
    pub fn set_flags(&mut self, flags: SubsetFlags) {
        // SAFETY: `self` owns a live input; `&mut self` proves exclusivity.
        unsafe { sys::subset::hb_subset_input_set_flags(self.as_raw(), flags.0 as core::ffi::c_uint) };
    }

    /// Produces the subsetted face.
    ///
    /// The source face is left untouched; the result is a new face you own.
    pub fn apply(&self, face: &Shared<Face>) -> Result<Face> {
        // SAFETY: both `self` and `face` own live objects for the duration of
        // the call. `hb_subset_or_fail` reads them without taking ownership and
        // returns a new face carrying its own reference, or null on failure.
        let raw = unsafe { sys::subset::hb_subset_or_fail(face.as_raw(), self.as_raw()) };

        if raw.is_null() {
            return Err(Error::SubsetFailed);
        }

        // SAFETY: `raw` is non-null and carries a reference this wrapper owns.
        Ok(unsafe { Face::from_raw(raw) })
    }
}

impl std::fmt::Debug for Subset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Subset")
            .field("flags", &self.flags())
            .finish()
    }
}

/// Options that change how subsetting rewrites the font.
///
/// Combine them with `|`.
///
/// ```
/// # use harfbuzz_rs::subset::SubsetFlags;
/// let flags = SubsetFlags::NO_HINTING | SubsetFlags::DESUBROUTINIZE;
/// assert!(flags.contains(SubsetFlags::NO_HINTING));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubsetFlags(sys::subset::hb_subset_flags_t);

impl SubsetFlags {
    /// No options: keep hinting, renumber glyphs, drop glyph names.
    pub const DEFAULT: Self = Self(sys::subset::HB_SUBSET_FLAGS_DEFAULT);
    /// Drop the hinting tables, which are often a large share of the file.
    pub const NO_HINTING: Self = Self(sys::subset::HB_SUBSET_FLAGS_NO_HINTING);
    /// Keep the original glyph indices instead of renumbering from zero.
    ///
    /// Needed when something outside the font already refers to glyphs by
    /// index, such as a PDF that has been laid out already.
    pub const RETAIN_GIDS: Self = Self(sys::subset::HB_SUBSET_FLAGS_RETAIN_GIDS);
    /// Inline CFF subroutines, which can shrink a heavily-subsetted CFF font.
    pub const DESUBROUTINIZE: Self = Self(sys::subset::HB_SUBSET_FLAGS_DESUBROUTINIZE);
    /// Keep the `post` table's glyph names.
    pub const GLYPH_NAMES: Self = Self(sys::subset::HB_SUBSET_FLAGS_GLYPH_NAMES);

    /// Whether every flag in `other` is set here.
    pub fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }
}

impl std::ops::BitOr for SubsetFlags {
    type Output = Self;

    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl std::ops::BitOrAssign for SubsetFlags {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::object::IntoShared;
    use crate::testing;

    #[test]
    fn keeps_only_what_was_asked_for() {
        let face = testing::face().into_shared();
        let original_glyphs = face.glyph_count();

        let mut subset = Subset::new().unwrap();
        subset.keep_chars("A".chars());

        let cut_down = subset.apply(&face).unwrap();

        assert!(
            cut_down.glyph_count() < original_glyphs,
            "subsetting to one character should drop glyphs ({} -> {})",
            original_glyphs,
            cut_down.glyph_count()
        );
        assert!(cut_down.glyph_count() > 0);
    }

    #[test]
    fn produces_a_font_that_still_shapes() {
        use crate::{Font, buffer_from, shape};

        let face = testing::face().into_shared();

        let mut subset = Subset::new().unwrap();
        subset.keep_chars("AB".chars());

        let font = Font::new(subset.apply(&face).unwrap().into_shared());
        let output = shape(&font, buffer_from("AB").unwrap(), &[]);

        assert_eq!(output.len(), 2);
        for (info, _) in output.iter() {
            assert!(info.glyph() > 0, "the kept characters still resolve");
        }
    }

    #[test]
    fn the_result_is_smaller_on_disk() {
        let face = testing::face().into_shared();
        let original = face.blob().len();

        let mut subset = Subset::new().unwrap();
        subset.keep_chars("A".chars());
        subset.set_flags(SubsetFlags::NO_HINTING);

        let cut_down = subset.apply(&face).unwrap();
        assert!(
            cut_down.blob().len() < original,
            "{} should be under {original}",
            cut_down.blob().len()
        );
    }

    #[test]
    fn flags_combine_and_round_trip() {
        let mut subset = Subset::new().unwrap();
        assert_eq!(subset.flags(), SubsetFlags::DEFAULT);

        let wanted = SubsetFlags::NO_HINTING | SubsetFlags::RETAIN_GIDS;
        subset.set_flags(wanted);

        assert_eq!(subset.flags(), wanted);
        assert!(subset.flags().contains(SubsetFlags::NO_HINTING));
        assert!(subset.flags().contains(SubsetFlags::RETAIN_GIDS));
        assert!(!subset.flags().contains(SubsetFlags::GLYPH_NAMES));
    }

    #[test]
    fn retaining_glyph_ids_keeps_the_original_numbering() {
        use crate::Font;

        let face = testing::face().into_shared();
        let original = Font::new(face.clone()).nominal_glyph('C').unwrap();

        let cut = |flags| {
            let mut subset = Subset::new().unwrap();
            subset.keep_chars("C".chars());
            subset.set_flags(flags);

            let face = subset.apply(&face).unwrap().into_shared();
            Font::new(face).nominal_glyph('C').unwrap()
        };

        // Retaining the ids means `C` answers to the same index as before.
        assert_eq!(cut(SubsetFlags::RETAIN_GIDS), original);

        // Without the flag, the survivors are renumbered from the start, so
        // the one glyph we kept moves down.
        assert!(cut(SubsetFlags::DEFAULT) < original);
    }
}
