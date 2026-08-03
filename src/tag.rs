//! Four-character OpenType tags.

use core::fmt;
use core::str::FromStr;

use harfbuzz_sys as sys;

use crate::error::{Error, Result};

/// A four-character OpenType tag.
///
/// Tags name almost everything in an OpenType font: tables (`glyf`, `GSUB`),
/// features (`kern`, `liga`), scripts (`latn`), languages (`TRK `), variation
/// axes (`wght`), and baselines. They are four bytes, conventionally printable
/// ASCII, padded on the right with spaces when the name is shorter.
///
/// The byte order is fixed: the first character is the most significant byte,
/// so `Tag::new(b"wght")` is `0x77676874`.
///
/// # Examples
///
/// ```
/// use harfbuzz_rs::Tag;
///
/// let kern = Tag::new(b"kern");
/// assert_eq!(kern.to_string(), "kern");
///
/// // Shorter names are padded with spaces, which is what OpenType expects.
/// let turkish: Tag = "TRK".parse()?;
/// assert_eq!(turkish.to_string(), "TRK ");
/// # Ok::<(), harfbuzz_rs::Error>(())
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Tag(pub(crate) sys::hb_tag_t);

impl Tag {
    /// The unset tag, all four bytes zero.
    pub const NONE: Self = Self(sys::HB_TAG_NONE);

    /// Builds a tag from exactly four bytes.
    ///
    /// This is `const`, so it can be used to define tag constants:
    ///
    /// ```
    /// # use harfbuzz_rs::Tag;
    /// const WEIGHT_AXIS: Tag = Tag::new(b"wght");
    /// ```
    pub const fn new(bytes: &[u8; 4]) -> Self {
        Self(sys::HB_TAG(bytes[0], bytes[1], bytes[2], bytes[3]))
    }

    /// Builds a tag from its raw 32-bit value.
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// The tag's raw 32-bit value.
    pub const fn to_raw(self) -> u32 {
        self.0
    }

    /// The tag's four bytes, most significant first.
    pub const fn to_bytes(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }
}

impl FromStr for Tag {
    type Err = Error;

    /// Parses a tag from one to four characters, padding on the right with
    /// spaces.
    ///
    /// This is stricter than HarfBuzz's own `hb_tag_from_string`, which
    /// silently truncates anything longer than four characters. Truncation
    /// almost always means the caller made a mistake, so it is reported here
    /// rather than hidden.
    fn from_str(s: &str) -> Result<Self> {
        let bytes = s.as_bytes();
        if bytes.is_empty() || bytes.len() > 4 {
            return Err(Error::InvalidTag);
        }

        // OpenType tags are defined over printable ASCII. Rejecting anything
        // else keeps `Display` lossless and round-trips through `to_string`.
        if bytes.iter().any(|b| !b.is_ascii_graphic() && *b != b' ') {
            return Err(Error::InvalidTag);
        }

        let mut padded = [b' '; 4];
        padded[..bytes.len()].copy_from_slice(bytes);

        Ok(Self::new(&padded))
    }
}

impl fmt::Display for Tag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.to_bytes() {
            // A tag built through `new` or `from_raw` may hold arbitrary bytes,
            // so fall back rather than emitting invalid UTF-8.
            if byte.is_ascii_graphic() || byte == b' ' {
                f.write_str(core::str::from_utf8(&[byte]).unwrap_or("?"))?;
            } else {
                f.write_str("?")?;
            }
        }

        Ok(())
    }
}

impl From<Tag> for u32 {
    fn from(tag: Tag) -> u32 {
        tag.0
    }
}

impl From<u32> for Tag {
    fn from(raw: u32) -> Tag {
        Tag(raw)
    }
}

impl From<&[u8; 4]> for Tag {
    fn from(bytes: &[u8; 4]) -> Tag {
        Tag::new(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_bytes_most_significant_first() {
        assert_eq!(Tag::new(b"wght").to_raw(), 0x77676874);
    }

    #[test]
    fn pads_short_names_with_spaces() {
        assert_eq!("TRK".parse::<Tag>().unwrap(), Tag::new(b"TRK "));
        assert_eq!("a".parse::<Tag>().unwrap(), Tag::new(b"a   "));
    }

    #[test]
    fn rejects_names_that_do_not_fit() {
        assert_eq!("toolong".parse::<Tag>(), Err(Error::InvalidTag));
        assert_eq!("".parse::<Tag>(), Err(Error::InvalidTag));
    }

    #[test]
    fn rejects_non_ascii() {
        assert_eq!("wgh\u{00e9}".parse::<Tag>(), Err(Error::InvalidTag));
    }

    #[test]
    fn round_trips_through_display() {
        for name in ["kern", "GSUB", "wght", "TRK ", "cv01"] {
            assert_eq!(name.parse::<Tag>().unwrap().to_string(), name);
        }
    }

    #[test]
    fn matches_harfbuzz_tag_parsing() {
        // The C parser is the reference for the four-character case; only the
        // over-long and empty cases deliberately differ.
        for name in ["kern", "GSUB", "wght", "cv01"] {
            // SAFETY: `name` is a Rust string with a known length, and passing
            // that length explicitly means HarfBuzz never looks for a
            // terminator past the end of the slice.
            let from_c = unsafe {
                sys::hb_tag_from_string(name.as_ptr().cast(), name.len() as core::ffi::c_int)
            };
            assert_eq!(name.parse::<Tag>().unwrap().to_raw(), from_c, "{name}");
        }
    }
}
