//! OpenType features and variation-axis settings.

use core::ffi::{CStr, c_int};
use core::fmt;
use core::ops::RangeBounds;
use core::str::FromStr;

use harfbuzz_sys as sys;

use crate::error::{Error, Result};
use crate::tag::Tag;

/// A request to turn an OpenType feature on or off over a range of the buffer.
///
/// Features are the switches that control optional typographic behaviour:
/// `liga` for standard ligatures, `kern` for kerning, `smcp` for small
/// capitals, `ss01` for the first stylistic set. A font decides what each one
/// actually does, and many features are on by default — passing one explicitly
/// is how you override that.
///
/// The range is measured in *cluster values*, not glyphs or bytes. By default
/// a feature applies to the whole buffer.
///
/// # Examples
///
/// ```
/// use harfbuzz_rs::{Feature, Tag};
///
/// // Turn ligatures off everywhere.
/// let no_ligatures = Feature::new(Tag::new(b"liga"), 0, ..);
///
/// // The same thing, written the way the hb-shape tool spells it.
/// let also: Feature = "-liga".parse()?;
/// assert_eq!(no_ligatures, also);
///
/// // Select the third alternate of `salt`, over clusters 2..5 only.
/// let alternate = Feature::new(Tag::new(b"salt"), 3, 2..5);
/// # Ok::<(), harfbuzz_rs::Error>(())
/// ```
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct Feature(pub(crate) sys::hb_feature_t);

impl Feature {
    /// Builds a feature setting.
    ///
    /// `value` is zero to disable the feature and non-zero to enable it; for
    /// features implemented as an alternate list, such as `salt`, it is a
    /// one-based index into the alternates.
    ///
    /// `range` is in cluster values. Pass `..` for the whole buffer.
    pub fn new(tag: Tag, value: u32, range: impl RangeBounds<u32>) -> Self {
        use core::ops::Bound;

        let start = match range.start_bound() {
            Bound::Included(&n) => n,
            Bound::Excluded(&n) => n.saturating_add(1),
            Bound::Unbounded => sys::HB_FEATURE_GLOBAL_START,
        };

        let end = match range.end_bound() {
            Bound::Included(&n) => n.saturating_add(1),
            Bound::Excluded(&n) => n,
            Bound::Unbounded => sys::HB_FEATURE_GLOBAL_END,
        };

        Self(sys::hb_feature_t {
            tag: tag.to_raw(),
            value,
            start,
            end,
        })
    }

    /// The feature's tag.
    pub fn tag(self) -> Tag {
        Tag::from_raw(self.0.tag)
    }

    /// The value: zero disables, non-zero enables, and for alternate features a
    /// one-based index.
    pub fn value(self) -> u32 {
        self.0.value
    }

    /// The first cluster the setting applies to, inclusive.
    pub fn start(self) -> u32 {
        self.0.start
    }

    /// The cluster the setting stops applying at, exclusive.
    pub fn end(self) -> u32 {
        self.0.end
    }

    /// Whether the setting covers the entire buffer.
    pub fn is_global(self) -> bool {
        self.0.start == sys::HB_FEATURE_GLOBAL_START && self.0.end == sys::HB_FEATURE_GLOBAL_END
    }
}

impl FromStr for Feature {
    type Err = Error;

    /// Parses the syntax used by the `hb-shape` tool.
    ///
    /// Accepts forms such as `kern`, `+kern`, `-liga`, `aalt=2`, and
    /// `salt[3:5]=2`.
    fn from_str(s: &str) -> Result<Self> {
        let mut feature = sys::hb_feature_t {
            tag: 0,
            value: 0,
            start: 0,
            end: 0,
        };

        // SAFETY: `s` is a Rust string whose length is passed explicitly, and
        // `feature` is a live, correctly aligned local that HarfBuzz only
        // writes on success.
        let ok = unsafe {
            sys::hb_feature_from_string(s.as_ptr().cast(), s.len() as c_int, &mut feature)
        };

        if ok == 0 {
            return Err(Error::InvalidFeature);
        }

        Ok(Self(feature))
    }
}

impl fmt::Display for Feature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // HarfBuzz documents 128 bytes as the size that always suffices for
        // this function, and takes the buffer by pointer with an explicit
        // length so it cannot overrun.
        let mut buffer = [0u8; 128];
        let mut feature = self.0;

        // SAFETY: `feature` is a live local; `buffer` is a live local of the
        // length we pass. HarfBuzz writes at most that many bytes and
        // NUL-terminates. It takes the feature by `*mut` but does not modify
        // it, so passing a copy costs nothing.
        unsafe {
            sys::hb_feature_to_string(
                &mut feature,
                buffer.as_mut_ptr().cast(),
                buffer.len() as core::ffi::c_uint,
            )
        };

        // SAFETY: HarfBuzz NUL-terminated the buffer within its bounds, so
        // there is a terminator to find.
        let text = unsafe { CStr::from_ptr(buffer.as_ptr().cast()) };

        f.write_str(&text.to_string_lossy())
    }
}

impl fmt::Debug for Feature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Feature({self})")
    }
}

/// A setting for one axis of a variable font.
///
/// Variable fonts expose named axes — `wght` for weight, `wdth` for width,
/// `slnt` for slant, `opsz` for optical size — each with a range the font
/// declares. A `Variation` pins one axis to a value inside that range.
///
/// # Examples
///
/// ```
/// use harfbuzz_rs::{Tag, Variation};
///
/// let bold = Variation::new(Tag::new(b"wght"), 700.0);
/// assert_eq!(bold, "wght=700".parse()?);
/// # Ok::<(), harfbuzz_rs::Error>(())
/// ```
#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Variation(pub(crate) sys::hb_variation_t);

impl Variation {
    /// Pins `tag` to `value`.
    ///
    /// A value outside the axis's declared range is clamped by the font, not
    /// rejected here.
    pub fn new(tag: Tag, value: f32) -> Self {
        Self(sys::hb_variation_t {
            tag: tag.to_raw(),
            value,
        })
    }

    /// The axis tag.
    pub fn tag(self) -> Tag {
        Tag::from_raw(self.0.tag)
    }

    /// The value on that axis.
    pub fn value(self) -> f32 {
        self.0.value
    }
}

impl FromStr for Variation {
    type Err = Error;

    /// Parses `axis=value`, for example `wght=700`.
    fn from_str(s: &str) -> Result<Self> {
        let mut variation = sys::hb_variation_t { tag: 0, value: 0.0 };

        // SAFETY: `s` is a Rust string whose length is passed explicitly, and
        // `variation` is a live, correctly aligned local that HarfBuzz only
        // writes on success.
        let ok = unsafe {
            sys::hb_variation_from_string(s.as_ptr().cast(), s.len() as c_int, &mut variation)
        };

        if ok == 0 {
            return Err(Error::InvalidVariation);
        }

        Ok(Self(variation))
    }
}

impl fmt::Display for Variation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer = [0u8; 128];
        let mut variation = self.0;

        // SAFETY: as in `Feature::fmt` — live local buffer, explicit length,
        // and HarfBuzz NUL-terminates within it.
        unsafe {
            sys::hb_variation_to_string(
                &mut variation,
                buffer.as_mut_ptr().cast(),
                buffer.len() as core::ffi::c_uint,
            )
        };

        // SAFETY: the buffer was NUL-terminated within its bounds.
        let text = unsafe { CStr::from_ptr(buffer.as_ptr().cast()) };

        f.write_str(&text.to_string_lossy())
    }
}

impl fmt::Debug for Variation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Variation({self})")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_covering_the_whole_buffer() {
        let feature = Feature::new(Tag::new(b"kern"), 1, ..);

        assert!(feature.is_global());
        assert_eq!(feature.start(), sys::HB_FEATURE_GLOBAL_START);
        assert_eq!(feature.end(), sys::HB_FEATURE_GLOBAL_END);
    }

    #[test]
    fn converts_rust_ranges_to_harfbuzz_bounds() {
        let half_open = Feature::new(Tag::new(b"salt"), 2, 2..5);
        assert_eq!((half_open.start(), half_open.end()), (2, 5));

        let inclusive = Feature::new(Tag::new(b"salt"), 2, 2..=5);
        assert_eq!((inclusive.start(), inclusive.end()), (2, 6));

        assert!(!half_open.is_global());
    }

    #[test]
    fn parses_the_hb_shape_syntax() {
        assert_eq!(
            "-liga".parse::<Feature>().unwrap(),
            Feature::new(Tag::new(b"liga"), 0, ..)
        );
        assert_eq!(
            "kern".parse::<Feature>().unwrap(),
            Feature::new(Tag::new(b"kern"), 1, ..)
        );

        let ranged: Feature = "salt[2:5]=3".parse().unwrap();
        assert_eq!(ranged.tag(), Tag::new(b"salt"));
        assert_eq!(ranged.value(), 3);
    }

    #[test]
    fn round_trips_a_feature_through_its_string_form() {
        for text in ["-liga", "aalt=2"] {
            let feature: Feature = text.parse().unwrap();
            assert_eq!(feature.to_string().parse::<Feature>().unwrap(), feature);
        }
    }

    #[test]
    fn parses_and_prints_variations() {
        let bold: Variation = "wght=700".parse().unwrap();

        assert_eq!(bold.tag(), Tag::new(b"wght"));
        assert_eq!(bold.value(), 700.0);
        assert_eq!(bold, Variation::new(Tag::new(b"wght"), 700.0));
    }

    #[test]
    fn rejects_nonsense() {
        assert_eq!("".parse::<Variation>(), Err(Error::InvalidVariation));
    }
}
