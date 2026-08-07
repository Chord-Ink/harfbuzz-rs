//! Raw bit fields and numbers scattered around OpenType tables —
//! `hb-ot-fetch.h`.
//!
//! These are unprocessed table values from `OS/2`, `head`, and `post`. Many of
//! them are legacy or unreliable, but applications may need them anyway.

use core::ffi::c_int;

use crate::{HB_TAG, hb_face_t};

/// Identifies a bit field that [`hb_ot_fetch_bits`] can fetch from a face.
///
/// The values are not OpenType table tags; they are HarfBuzz-invented keys that
/// happen to be spelled as four-character tags.
///
/// The alias is a signed C `int` because the C enumeration ends with a private
/// sentinel equal to [`HB_TAG_MAX_SIGNED`](crate::HB_TAG_MAX_SIGNED), which
/// pins the underlying type. Any other value is accepted by
/// [`hb_ot_fetch_bits`] and simply yields zero.
///
/// Since HarfBuzz 14.3.0.
pub type hb_ot_bits_tag_t = c_int;

/// `fsType` of the `OS/2` table — the font-embedding licensing bits.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_BITS_TAG_FS_TYPE: hb_ot_bits_tag_t =
    HB_TAG(b'f', b's', b't', b'p') as hb_ot_bits_tag_t;

/// `fsSelection` of the `OS/2` table — the italic/bold/regular and
/// use-typo-metrics style bits.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_BITS_TAG_FS_SELECTION: hb_ot_bits_tag_t =
    HB_TAG(b'f', b's', b's', b'l') as hb_ot_bits_tag_t;

/// `macStyle` of the `head` table — the legacy Macintosh style bits.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_BITS_TAG_MAC_STYLE: hb_ot_bits_tag_t =
    HB_TAG(b'm', b'c', b's', b't') as hb_ot_bits_tag_t;

/// `isFixedPitch` of the `post` table — zero if the font is proportionally
/// spaced, non-zero if it is monospaced.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_BITS_TAG_IS_FIXED_PITCH: hb_ot_bits_tag_t =
    HB_TAG(b'f', b'x', b'p', b't') as hb_ot_bits_tag_t;

/// `ulUnicodeRange1` of the `OS/2` table — Unicode-range coverage bits 0..=31.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_BITS_TAG_UNICODE_RANGE_1: hb_ot_bits_tag_t =
    HB_TAG(b'u', b'r', b'n', b'1') as hb_ot_bits_tag_t;

/// `ulUnicodeRange2` of the `OS/2` table — Unicode-range coverage bits 32..=63.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_BITS_TAG_UNICODE_RANGE_2: hb_ot_bits_tag_t =
    HB_TAG(b'u', b'r', b'n', b'2') as hb_ot_bits_tag_t;

/// `ulUnicodeRange3` of the `OS/2` table — Unicode-range coverage bits 64..=95.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_BITS_TAG_UNICODE_RANGE_3: hb_ot_bits_tag_t =
    HB_TAG(b'u', b'r', b'n', b'3') as hb_ot_bits_tag_t;

/// `ulUnicodeRange4` of the `OS/2` table — Unicode-range coverage bits
/// 96..=127.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_BITS_TAG_UNICODE_RANGE_4: hb_ot_bits_tag_t =
    HB_TAG(b'u', b'r', b'n', b'4') as hb_ot_bits_tag_t;

/// `ulCodePageRange1` of the `OS/2` table — legacy code-page coverage bits
/// 0..=31. Present only in `OS/2` version 1 and later.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_BITS_TAG_CODE_PAGE_RANGE_1: hb_ot_bits_tag_t =
    HB_TAG(b'c', b'p', b'r', b'1') as hb_ot_bits_tag_t;

/// `ulCodePageRange2` of the `OS/2` table — legacy code-page coverage bits
/// 32..=63. Present only in `OS/2` version 1 and later.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_BITS_TAG_CODE_PAGE_RANGE_2: hb_ot_bits_tag_t =
    HB_TAG(b'c', b'p', b'r', b'2') as hb_ot_bits_tag_t;

/// Identifies a number that [`hb_ot_fetch_number`] can fetch from a face.
///
/// The values are not OpenType table tags; they are HarfBuzz-invented keys that
/// happen to be spelled as four-character tags.
///
/// The alias is a signed C `int` because the C enumeration ends with a private
/// sentinel equal to [`HB_TAG_MAX_SIGNED`](crate::HB_TAG_MAX_SIGNED), which
/// pins the underlying type. Any other value is accepted by
/// [`hb_ot_fetch_number`] and simply yields zero.
///
/// Since HarfBuzz 14.3.0.
pub type hb_ot_number_tag_t = c_int;

/// `xMin` of the `head` table — the left edge of the union of all glyph
/// bounding boxes, in font units.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_NUMBER_TAG_FONT_X_MIN: hb_ot_number_tag_t =
    HB_TAG(b'x', b'm', b'i', b'n') as hb_ot_number_tag_t;

/// `yMin` of the `head` table — the bottom edge of the union of all glyph
/// bounding boxes, in font units.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_NUMBER_TAG_FONT_Y_MIN: hb_ot_number_tag_t =
    HB_TAG(b'y', b'm', b'i', b'n') as hb_ot_number_tag_t;

/// `xMax` of the `head` table — the right edge of the union of all glyph
/// bounding boxes, in font units.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_NUMBER_TAG_FONT_X_MAX: hb_ot_number_tag_t =
    HB_TAG(b'x', b'm', b'a', b'x') as hb_ot_number_tag_t;

/// `yMax` of the `head` table — the top edge of the union of all glyph
/// bounding boxes, in font units.
///
/// Since HarfBuzz 14.3.0.
pub const HB_OT_NUMBER_TAG_FONT_Y_MAX: hb_ot_number_tag_t =
    HB_TAG(b'y', b'm', b'a', b'x') as hb_ot_number_tag_t;

unsafe extern "C" {
    /// Fetches a bit field of a face.
    ///
    /// Returns the bit field, or zero if the font does not have it — which
    /// covers a missing or unsanitizable table, an `OS/2` version too old to
    /// carry the field, and an unrecognized `tag`.
    ///
    /// Since HarfBuzz 14.3.0.
    pub fn hb_ot_fetch_bits(face: *mut hb_face_t, tag: hb_ot_bits_tag_t) -> u32;

    /// Fetches a number of a face, in font units.
    ///
    /// Returns the number, or zero if the font does not have it — which covers
    /// a missing or unsanitizable `head` table and an unrecognized `tag`.
    ///
    /// Since HarfBuzz 14.3.0.
    pub fn hb_ot_fetch_number(face: *mut hb_face_t, tag: hb_ot_number_tag_t) -> i32;
}
