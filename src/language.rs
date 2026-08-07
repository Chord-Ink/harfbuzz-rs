//! BCP 47 language tags.

use core::ffi::{CStr, c_int};
use core::fmt;
use core::str::FromStr;

use harfbuzz_sys as sys;

use crate::error::{Error, Result};

/// A language, corresponding to a BCP 47 language tag such as `en-GB`.
///
/// HarfBuzz interns languages: every distinct tag is canonicalised and stored
/// once, for the lifetime of the process. A `Language` is therefore a plain
/// copyable handle that never needs freeing, and two languages are equal
/// exactly when their interned pointers match.
///
/// # Examples
///
/// ```
/// use harfbuzz_rs::Language;
///
/// let english: Language = "en-GB".parse()?;
/// assert_eq!(english.to_string(), "en-gb");
///
/// // Canonicalisation means case does not matter.
/// assert_eq!("EN-gb".parse::<Language>()?, english);
/// # Ok::<(), harfbuzz_rs::Error>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Language(pub(crate) sys::hb_language_t);

// SAFETY: an `hb_language_t` points into HarfBuzz's intern table. Entries are
// written once, never mutated, and never freed, so the pointer stays valid and
// its target immutable for the rest of the process. Sharing or moving one
// between threads therefore cannot race.
unsafe impl Send for Language {}
unsafe impl Sync for Language {}

impl Language {
    /// The language HarfBuzz derives from the process locale.
    ///
    /// The first call inspects the environment and caches the result, and that
    /// first call is not thread-safe in HarfBuzz. Call it once during start-up
    /// if several threads might reach it.
    pub fn default_from_locale() -> Self {
        // SAFETY: takes no arguments, and returns an interned language that
        // lives for the rest of the process.
        Self(unsafe { sys::hb_language_get_default() })
    }

    /// Whether this is the unset language.
    pub fn is_invalid(self) -> bool {
        self.0 == sys::HB_LANGUAGE_INVALID
    }

    /// Whether `self` is a more general tagging than `specific`.
    ///
    /// A language matches a more specific one that starts with it and continues
    /// with `-`: `en` matches `en-GB`, but `en-GB` does not match `en`.
    pub fn matches(self, specific: Language) -> bool {
        // SAFETY: both handles are either null (the invalid language, which
        // HarfBuzz accepts) or interned pointers that remain valid.
        unsafe { sys::hb_language_matches(self.0, specific.0) != 0 }
    }
}

impl FromStr for Language {
    type Err = Error;

    /// Interns a BCP 47 tag.
    ///
    /// HarfBuzz lowercases the tag and canonicalises it, so the round trip
    /// through [`Display`](fmt::Display) is not byte-for-byte identical to the
    /// input. An empty string has no valid interpretation and is rejected.
    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Err(Error::InvalidLanguage);
        }

        // SAFETY: `s` is a Rust string and its length is passed explicitly, so
        // HarfBuzz reads exactly those bytes and never looks for a terminator
        // past the end. The returned pointer is interned and outlives us.
        let raw = unsafe { sys::hb_language_from_string(s.as_ptr().cast(), s.len() as c_int) };

        if raw == sys::HB_LANGUAGE_INVALID {
            return Err(Error::InvalidLanguage);
        }

        Ok(Self(raw))
    }
}

impl fmt::Display for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_invalid() {
            return f.write_str("invalid");
        }

        // SAFETY: the handle is a live interned language, so HarfBuzz returns a
        // pointer to its NUL-terminated canonical spelling, owned by the intern
        // table and valid for the rest of the process.
        let text = unsafe { CStr::from_ptr(sys::hb_language_to_string(self.0)) };

        f.write_str(&text.to_string_lossy())
    }
}

impl fmt::Debug for Language {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Language({self})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonicalises_and_interns() {
        let a: Language = "en-GB".parse().unwrap();
        let b: Language = "EN-gb".parse().unwrap();

        // Interning means equal tags really are the same pointer.
        assert_eq!(a, b);
        assert_eq!(a.to_string(), "en-gb");
    }

    #[test]
    fn rejects_the_empty_tag() {
        assert_eq!("".parse::<Language>(), Err(Error::InvalidLanguage));
    }

    #[test]
    fn matches_only_more_specific_tags() {
        let general: Language = "en".parse().unwrap();
        let specific: Language = "en-GB".parse().unwrap();

        assert!(general.matches(specific));
        assert!(!specific.matches(general));
    }

    #[test]
    fn reports_a_default_from_the_locale() {
        assert!(!Language::default_from_locale().is_invalid());
    }
}
