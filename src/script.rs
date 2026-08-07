//! Unicode scripts.

use core::ffi::c_int;
use core::fmt;
use core::str::FromStr;

use harfbuzz_sys as sys;

use crate::direction::Direction;
use crate::error::{Error, Result};
use crate::tag::Tag;

/// A Unicode script, identified by its ISO 15924 code.
///
/// The script of a text run is what decides which shaper HarfBuzz uses —
/// Arabic joining, Indic reordering, and Hangul composition are all selected
/// this way — so getting it right matters more than most buffer properties. If
/// you do not know it,
/// [`Buffer::guess_segment_properties`](crate::Buffer::guess_segment_properties)
/// will infer it from the text itself.
///
/// Internally a script is its ISO 15924 tag, so [`Script::LATIN`] is the tag
/// `Latn`.
///
/// # Examples
///
/// ```
/// use harfbuzz_rs::{Direction, Script};
///
/// let arabic: Script = "Arab".parse()?;
/// assert_eq!(arabic, Script::ARABIC);
/// assert_eq!(arabic.horizontal_direction(), Direction::RightToLeft);
/// # Ok::<(), harfbuzz_rs::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Script(pub(crate) sys::hb_script_t);

impl Script {
    /// Builds a script from its raw value.
    pub(crate) const fn from_raw(raw: sys::hb_script_t) -> Self {
        Self(raw)
    }

    pub(crate) const fn to_raw(self) -> sys::hb_script_t {
        self.0
    }

    /// Builds a script from an ISO 15924 tag such as `Latn`.
    ///
    /// Unlike [`FromStr`], this accepts any four-character tag, including ones
    /// Unicode has not assigned.
    pub fn from_iso15924_tag(tag: Tag) -> Self {
        // SAFETY: takes a plain integer and returns one; no pointers involved.
        Self(unsafe { sys::hb_script_from_iso15924_tag(tag.to_raw()) })
    }

    /// The script's ISO 15924 tag.
    pub fn to_iso15924_tag(self) -> Tag {
        // SAFETY: takes a plain integer and returns one; no pointers involved.
        Tag::from_raw(unsafe { sys::hb_script_to_iso15924_tag(self.0) })
    }

    /// The direction text in this script is normally set in horizontally.
    ///
    /// Returns [`Direction::Invalid`] for scripts that are not written
    /// horizontally, and [`Direction::LeftToRight`] for scripts HarfBuzz does
    /// not recognise.
    pub fn horizontal_direction(self) -> Direction {
        // SAFETY: takes a plain integer and returns one; no pointers involved.
        Direction::from_raw(unsafe { sys::hb_script_get_horizontal_direction(self.0) })
    }
}

impl FromStr for Script {
    type Err = Error;

    /// Parses an ISO 15924 tag such as `Latn`, or a script name HarfBuzz
    /// recognises.
    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(Error::InvalidScript);
        }

        // SAFETY: `s` is a Rust string and its length is passed explicitly, so
        // HarfBuzz reads exactly those bytes.
        let raw = unsafe { sys::hb_script_from_string(s.as_ptr().cast(), s.len() as c_int) };

        if raw == Self::INVALID.0 {
            return Err(Error::InvalidScript);
        }

        Ok(Self(raw))
    }
}

impl fmt::Display for Script {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_iso15924_tag())
    }
}

impl From<Script> for Tag {
    fn from(script: Script) -> Tag {
        script.to_iso15924_tag()
    }
}

impl From<Tag> for Script {
    fn from(tag: Tag) -> Script {
        Script::from_iso15924_tag(tag)
    }
}

/// Names the scripts that come up most often, so callers rarely need a tag
/// literal.
///
/// All 177 of HarfBuzz's scripts are available in
/// [`harfbuzz_sys`](crate::sys) as `HB_SCRIPT_*` constants; anything not named
/// here can be built with [`Script::from_iso15924_tag`].
macro_rules! well_known_scripts {
    ($($name:ident = $raw:ident, $human:literal;)*) => {
        impl Script {
            $(
                #[doc = concat!("The ", $human, " script.")]
                pub const $name: Self = Self(sys::$raw);
            )*
        }
    };
}

impl Script {
    /// An unset or unrecognised script.
    pub const INVALID: Self = Self(sys::HB_SCRIPT_INVALID);
    /// Characters used across scripts, such as spaces and most punctuation.
    pub const COMMON: Self = Self(sys::HB_SCRIPT_COMMON);
    /// Characters that take the script of the preceding character.
    pub const INHERITED: Self = Self(sys::HB_SCRIPT_INHERITED);
    /// Assigned to unknown code points.
    pub const UNKNOWN: Self = Self(sys::HB_SCRIPT_UNKNOWN);
}

well_known_scripts! {
    ARABIC = HB_SCRIPT_ARABIC, "Arabic";
    ARMENIAN = HB_SCRIPT_ARMENIAN, "Armenian";
    BENGALI = HB_SCRIPT_BENGALI, "Bengali";
    CYRILLIC = HB_SCRIPT_CYRILLIC, "Cyrillic";
    DEVANAGARI = HB_SCRIPT_DEVANAGARI, "Devanagari";
    GEORGIAN = HB_SCRIPT_GEORGIAN, "Georgian";
    GREEK = HB_SCRIPT_GREEK, "Greek";
    GUJARATI = HB_SCRIPT_GUJARATI, "Gujarati";
    GURMUKHI = HB_SCRIPT_GURMUKHI, "Gurmukhi";
    HAN = HB_SCRIPT_HAN, "Han";
    HANGUL = HB_SCRIPT_HANGUL, "Hangul";
    HEBREW = HB_SCRIPT_HEBREW, "Hebrew";
    HIRAGANA = HB_SCRIPT_HIRAGANA, "Hiragana";
    KANNADA = HB_SCRIPT_KANNADA, "Kannada";
    KATAKANA = HB_SCRIPT_KATAKANA, "Katakana";
    KHMER = HB_SCRIPT_KHMER, "Khmer";
    LAO = HB_SCRIPT_LAO, "Lao";
    LATIN = HB_SCRIPT_LATIN, "Latin";
    MALAYALAM = HB_SCRIPT_MALAYALAM, "Malayalam";
    MYANMAR = HB_SCRIPT_MYANMAR, "Myanmar";
    ORIYA = HB_SCRIPT_ORIYA, "Oriya";
    SINHALA = HB_SCRIPT_SINHALA, "Sinhala";
    SYRIAC = HB_SCRIPT_SYRIAC, "Syriac";
    TAMIL = HB_SCRIPT_TAMIL, "Tamil";
    TELUGU = HB_SCRIPT_TELUGU, "Telugu";
    THAANA = HB_SCRIPT_THAANA, "Thaana";
    THAI = HB_SCRIPT_THAI, "Thai";
    TIBETAN = HB_SCRIPT_TIBETAN, "Tibetan";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_its_iso15924_tag() {
        assert_eq!(Script::LATIN.to_iso15924_tag(), Tag::new(b"Latn"));
        assert_eq!(Script::ARABIC.to_string(), "Arab");
    }

    #[test]
    fn round_trips_through_a_tag() {
        for script in [Script::LATIN, Script::ARABIC, Script::HAN, Script::THAI] {
            assert_eq!(Script::from_iso15924_tag(script.to_iso15924_tag()), script);
        }
    }

    #[test]
    fn knows_which_scripts_run_right_to_left() {
        assert_eq!(Script::ARABIC.horizontal_direction(), Direction::RightToLeft);
        assert_eq!(Script::HEBREW.horizontal_direction(), Direction::RightToLeft);
        assert_eq!(Script::LATIN.horizontal_direction(), Direction::LeftToRight);
    }

    #[test]
    fn parses_tags_and_names() {
        assert_eq!("Arab".parse::<Script>().unwrap(), Script::ARABIC);
        assert_eq!("Latn".parse::<Script>().unwrap(), Script::LATIN);
        assert_eq!("".parse::<Script>(), Err(Error::InvalidScript));
    }
}
