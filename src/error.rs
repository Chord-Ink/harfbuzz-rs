//! The error type shared by the crate.
//!
//! HarfBuzz reports problems in three different ways, and this module folds
//! them into one:
//!
//! * **Constructors never fail.** On allocation failure they hand back an inert
//!   "empty" singleton instead of null, so the caller never has to null-check.
//!   The wrappers detect that singleton and turn it into
//!   [`Error::AllocationFailed`], because silently returning an object that
//!   ignores everything you do to it is worse than an error.
//!
//! * **`*_or_fail` constructors return null.** These are the variants upstream
//!   added for callers who would rather know; the wrappers use them wherever
//!   they exist.
//!
//! * **Accumulating failure flags.** Buffers and sets record allocation
//!   failures internally and keep going, so a long series of pushes can be
//!   checked once at the end rather than after every call.

use core::fmt;

/// Anything that can go wrong in this crate.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// HarfBuzz could not allocate memory.
    ///
    /// This also covers the case where a constructor returned the shared empty
    /// singleton, which is how HarfBuzz signals allocation failure without
    /// returning null.
    AllocationFailed,

    /// A font file could not be read, or was not a font HarfBuzz recognises.
    ///
    /// HarfBuzz does not distinguish "the file is missing" from "the file is
    /// not a font", so neither can this.
    FontLoadFailed,

    /// The requested face index does not exist in this font file.
    ///
    /// Collections such as `.ttc` hold several faces; plain `.ttf` files have
    /// exactly one, at index zero.
    NoSuchFace {
        /// The index that was asked for.
        requested: u32,
        /// How many faces the file actually contains.
        available: u32,
    },

    /// A string was not four bytes long, or held a byte outside the printable
    /// ASCII range that OpenType tags are limited to.
    InvalidTag,

    /// A string did not name a text direction.
    InvalidDirection,

    /// A string was not a usable BCP 47 language tag.
    InvalidLanguage,

    /// A string was not a usable script name or ISO 15924 tag.
    InvalidScript,

    /// A feature string such as `"kern"` or `"-liga"` could not be parsed.
    InvalidFeature,

    /// A variation string such as `"wght=700"` could not be parsed.
    InvalidVariation,

    /// The bytes handed to a buffer were not valid UTF-8, UTF-16, or UTF-32.
    InvalidText,

    /// Subsetting failed.
    ///
    /// HarfBuzz reports subsetting failure as a single boolean, so there is no
    /// finer detail to pass on.
    SubsetFailed,

    /// A shaper was requested by name that this build of HarfBuzz does not
    /// have.
    UnknownShaper,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AllocationFailed => f.write_str("HarfBuzz could not allocate memory"),
            Self::FontLoadFailed => {
                f.write_str("the font could not be read, or is not in a recognised format")
            }
            Self::NoSuchFace {
                requested,
                available,
            } => write!(
                f,
                "face index {requested} is out of range; the file contains {available} face(s)"
            ),
            Self::InvalidTag => {
                f.write_str("a tag must be one to four printable ASCII characters")
            }
            Self::InvalidDirection => {
                f.write_str("expected one of \"ltr\", \"rtl\", \"ttb\", or \"btt\"")
            }
            Self::InvalidLanguage => f.write_str("not a usable BCP 47 language tag"),
            Self::InvalidScript => f.write_str("not a usable script name or ISO 15924 tag"),
            Self::InvalidFeature => f.write_str("the feature string could not be parsed"),
            Self::InvalidVariation => f.write_str("the variation string could not be parsed"),
            Self::InvalidText => f.write_str("the text was not valid Unicode"),
            Self::SubsetFailed => f.write_str("subsetting failed"),
            Self::UnknownShaper => {
                f.write_str("this build of HarfBuzz does not include the requested shaper")
            }
        }
    }
}

impl core::error::Error for Error {}

/// A [`Result`](core::result::Result) with this crate's [`Error`].
pub type Result<T, E = Error> = core::result::Result<T, E>;
