//! Style values — italic, weight, width, optical size — read from a font — `hb-style.h`.

use core::ffi::{c_float, c_int};

use crate::{HB_TAG, hb_font_t};

/// A style axis to query with [`hb_style_get_value`].
///
/// Every value is an [`hb_tag_t`](crate::hb_tag_t) naming a design-variation
/// axis from the
/// [OpenType Design-Variation Axis Tag Registry](https://docs.microsoft.com/en-us/typography/opentype/spec/dvaraxisreg),
/// with one HarfBuzz-only addition ([`HB_STYLE_TAG_SLANT_RATIO`]).
///
/// The alias is a signed C `int` because the C enumeration ends with a private
/// sentinel equal to [`HB_TAG_MAX_SIGNED`](crate::HB_TAG_MAX_SIGNED), which
/// pins the underlying type. That sentinel also means any registered axis tag
/// can be passed here, not only the constants below — HarfBuzz looks the tag up
/// among the font's variation axes before falling back to the cases it knows.
pub type hb_style_tag_t = c_int;

/// Italic (`ital`) — varies between non-italic and italic.
///
/// A value of 0 can be read as "Roman" (non-italic); a value of 1 as fully
/// italic.
///
/// Since HarfBuzz 3.0.0.
pub const HB_STYLE_TAG_ITALIC: hb_style_tag_t = HB_TAG(b'i', b't', b'a', b'l') as hb_style_tag_t;

/// Optical size (`opsz`) — varies the design to suit different text sizes.
///
/// Non-zero. Values can be read as a text size, in points.
///
/// Since HarfBuzz 3.0.0.
pub const HB_STYLE_TAG_OPTICAL_SIZE: hb_style_tag_t =
    HB_TAG(b'o', b'p', b's', b'z') as hb_style_tag_t;

/// Slant angle (`slnt`) — varies between upright and slanted text.
///
/// Values must be greater than -90 and less than +90, and can be read as the
/// angle, in counter-clockwise degrees, of oblique slant away from whatever the
/// designer considers upright for that font design. Typical right-leaning
/// italic fonts have a negative slant angle, around -12.
///
/// Since HarfBuzz 3.0.0.
pub const HB_STYLE_TAG_SLANT_ANGLE: hb_style_tag_t =
    HB_TAG(b's', b'l', b'n', b't') as hb_style_tag_t;

/// Slant ratio (`Slnt`) — the same quantity as [`HB_STYLE_TAG_SLANT_ANGLE`],
/// expressed as a ratio rather than an angle.
///
/// Typical right-leaning italic fonts have a positive slant ratio, around 0.2.
/// Note the capital `S`: this tag is HarfBuzz's own, not a registered OpenType
/// axis.
///
/// Since HarfBuzz 3.0.0.
pub const HB_STYLE_TAG_SLANT_RATIO: hb_style_tag_t =
    HB_TAG(b'S', b'l', b'n', b't') as hb_style_tag_t;

/// Width (`wdth`) — varies the width of text from narrower to wider.
///
/// Non-zero. Values can be read as a percentage of whatever the designer
/// considers "normal width" for that font design.
///
/// Since HarfBuzz 3.0.0.
pub const HB_STYLE_TAG_WIDTH: hb_style_tag_t = HB_TAG(b'w', b'd', b't', b'h') as hb_style_tag_t;

/// Weight (`wght`) — varies stroke thickness and other design details from
/// lighter to blacker.
///
/// Values can be compared directly against `usWeightClass` in the OS/2 table,
/// or the CSS `font-weight` property.
///
/// Since HarfBuzz 3.0.0.
pub const HB_STYLE_TAG_WEIGHT: hb_style_tag_t = HB_TAG(b'w', b'g', b'h', b't') as hb_style_tag_t;

unsafe extern "C" {
    /// Fetches the value of a style axis for a font.
    ///
    /// The font's variation axes are searched for `style_tag` first. If the
    /// axis is not set there, HarfBuzz tries the default style values in the
    /// `STAT` table, and then polyfills from other tables of the font.
    ///
    /// Returns the corresponding axis value, or a default value for the style
    /// tag.
    ///
    /// Since HarfBuzz 3.0.0.
    pub fn hb_style_get_value(font: *mut hb_font_t, style_tag: hb_style_tag_t) -> c_float;
}
