//! Text direction.

use core::fmt;
use core::str::FromStr;

use harfbuzz_sys as sys;

use crate::error::{Error, Result};

/// The direction a run of text is set in.
///
/// HarfBuzz treats the four valid directions as two axes: horizontal versus
/// vertical, and forward versus backward. [`Direction::Invalid`] is the unset
/// value a fresh buffer starts with, and it is what
/// [`Buffer::guess_segment_properties`](crate::Buffer::guess_segment_properties)
/// replaces with a real direction inferred from the text's script.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Direction {
    /// Initial, unset direction.
    #[default]
    Invalid,
    /// Horizontal, left to right.
    LeftToRight,
    /// Horizontal, right to left.
    RightToLeft,
    /// Vertical, top to bottom.
    TopToBottom,
    /// Vertical, bottom to top.
    BottomToTop,
}

impl Direction {
    /// Converts from the raw C value.
    ///
    /// Anything outside the four valid directions becomes
    /// [`Direction::Invalid`], which is the same thing HarfBuzz's own
    /// `HB_DIRECTION_IS_VALID` test would conclude.
    pub(crate) fn from_raw(raw: sys::hb_direction_t) -> Self {
        match raw {
            sys::HB_DIRECTION_LTR => Self::LeftToRight,
            sys::HB_DIRECTION_RTL => Self::RightToLeft,
            sys::HB_DIRECTION_TTB => Self::TopToBottom,
            sys::HB_DIRECTION_BTT => Self::BottomToTop,
            _ => Self::Invalid,
        }
    }

    pub(crate) fn to_raw(self) -> sys::hb_direction_t {
        match self {
            Self::Invalid => sys::HB_DIRECTION_INVALID,
            Self::LeftToRight => sys::HB_DIRECTION_LTR,
            Self::RightToLeft => sys::HB_DIRECTION_RTL,
            Self::TopToBottom => sys::HB_DIRECTION_TTB,
            Self::BottomToTop => sys::HB_DIRECTION_BTT,
        }
    }

    /// Whether this is one of the four real directions.
    pub fn is_valid(self) -> bool {
        self != Self::Invalid
    }

    /// Whether text flows along the horizontal axis.
    pub fn is_horizontal(self) -> bool {
        matches!(self, Self::LeftToRight | Self::RightToLeft)
    }

    /// Whether text flows along the vertical axis.
    pub fn is_vertical(self) -> bool {
        matches!(self, Self::TopToBottom | Self::BottomToTop)
    }

    /// Whether text flows in the increasing direction along its axis — left to
    /// right, or top to bottom.
    pub fn is_forward(self) -> bool {
        matches!(self, Self::LeftToRight | Self::TopToBottom)
    }

    /// Whether text flows in the decreasing direction along its axis — right to
    /// left, or bottom to top.
    pub fn is_backward(self) -> bool {
        matches!(self, Self::RightToLeft | Self::BottomToTop)
    }

    /// The opposite direction along the same axis.
    ///
    /// [`Direction::Invalid`] reverses to itself.
    pub fn reverse(self) -> Self {
        match self {
            Self::Invalid => Self::Invalid,
            Self::LeftToRight => Self::RightToLeft,
            Self::RightToLeft => Self::LeftToRight,
            Self::TopToBottom => Self::BottomToTop,
            Self::BottomToTop => Self::TopToBottom,
        }
    }
}

impl FromStr for Direction {
    type Err = Error;

    /// Parses `"ltr"`, `"rtl"`, `"ttb"`, or `"btt"`.
    ///
    /// Matching is case-insensitive and, as in HarfBuzz, only the first
    /// character is significant — so `"l"` and `"leftwards"` both parse as
    /// left-to-right.
    fn from_str(s: &str) -> Result<Self> {
        match s.as_bytes().first().map(u8::to_ascii_lowercase) {
            Some(b'l') => Ok(Self::LeftToRight),
            Some(b'r') => Ok(Self::RightToLeft),
            Some(b't') => Ok(Self::TopToBottom),
            Some(b'b') => Ok(Self::BottomToTop),
            _ => Err(Error::InvalidDirection),
        }
    }
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Invalid => "invalid",
            Self::LeftToRight => "ltr",
            Self::RightToLeft => "rtl",
            Self::TopToBottom => "ttb",
            Self::BottomToTop => "btt",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_axes_and_polarity() {
        assert!(Direction::LeftToRight.is_horizontal());
        assert!(Direction::LeftToRight.is_forward());
        assert!(Direction::BottomToTop.is_vertical());
        assert!(Direction::BottomToTop.is_backward());
        assert!(!Direction::Invalid.is_valid());
    }

    #[test]
    fn reverses_within_its_axis() {
        assert_eq!(Direction::LeftToRight.reverse(), Direction::RightToLeft);
        assert_eq!(Direction::TopToBottom.reverse(), Direction::BottomToTop);
        assert_eq!(Direction::Invalid.reverse(), Direction::Invalid);
    }

    #[test]
    fn round_trips_through_the_raw_value() {
        for direction in [
            Direction::Invalid,
            Direction::LeftToRight,
            Direction::RightToLeft,
            Direction::TopToBottom,
            Direction::BottomToTop,
        ] {
            assert_eq!(Direction::from_raw(direction.to_raw()), direction);
        }
    }

    #[test]
    fn agrees_with_harfbuzz_on_classification() {
        for direction in [
            Direction::LeftToRight,
            Direction::RightToLeft,
            Direction::TopToBottom,
            Direction::BottomToTop,
        ] {
            let raw = direction.to_raw();
            assert_eq!(direction.is_horizontal(), sys::HB_DIRECTION_IS_HORIZONTAL(raw));
            assert_eq!(direction.is_vertical(), sys::HB_DIRECTION_IS_VERTICAL(raw));
            assert_eq!(direction.is_forward(), sys::HB_DIRECTION_IS_FORWARD(raw));
            assert_eq!(direction.is_backward(), sys::HB_DIRECTION_IS_BACKWARD(raw));
        }
    }

    #[test]
    fn parses_the_way_harfbuzz_does() {
        for (text, expected) in [
            ("ltr", Direction::LeftToRight),
            ("RTL", Direction::RightToLeft),
            ("ttb", Direction::TopToBottom),
            ("b", Direction::BottomToTop),
        ] {
            assert_eq!(text.parse::<Direction>().unwrap(), expected);

            // SAFETY: `text` is a Rust string and its length is passed
            // explicitly, so HarfBuzz never reads past the end of the slice.
            let raw = unsafe {
                sys::hb_direction_from_string(text.as_ptr().cast(), text.len() as core::ffi::c_int)
            };
            assert_eq!(Direction::from_raw(raw), expected, "{text}");
        }
    }
}
