//! Name strings from the OpenType `name` table — `hb-ot-name.h`.
//!
//! A face carries its human-readable strings — family, subfamily, designer,
//! licence, sample text — as numbered entries in several languages. This module
//! enumerates the pairs a face offers and fetches them as UTF-8, UTF-16, or
//! UTF-32.

use core::ffi::{c_char, c_int, c_uint};

use crate::{hb_face_t, hb_language_t, hb_var_int_t};

/// The pre-defined name IDs of the OpenType `name` table.
///
/// Every string in a `name` table is filed under an identifier. Identifiers
/// 0–25 have a meaning fixed by the specification and are listed here; 26–255
/// are reserved; 256 and above are font-specific and are handed out by other
/// parts of the API — feature UI labels, variation axis and instance names,
/// colour palette labels, and so on.
///
/// For more information on these entries, see the
/// [OpenType spec](https://docs.microsoft.com/en-us/typography/opentype/spec/name#name-ids).
///
/// The C enumeration has no explicit sentinel and its largest enumerator is
/// `0xFFFF`, so it fits in an `int`.
///
/// Note that these constants are typed as this enumeration, whereas the fetch
/// functions take an [`hb_ot_name_id_t`] (a `c_uint`); a cast is needed at the
/// call site.
///
/// Since HarfBuzz 7.0.0.
pub type hb_ot_name_id_predefined_t = c_int;

/// Copyright notice.
pub const HB_OT_NAME_ID_COPYRIGHT: hb_ot_name_id_predefined_t = 0;
/// Font Family name.
pub const HB_OT_NAME_ID_FONT_FAMILY: hb_ot_name_id_predefined_t = 1;
/// Font Subfamily name.
pub const HB_OT_NAME_ID_FONT_SUBFAMILY: hb_ot_name_id_predefined_t = 2;
/// Unique font identifier.
pub const HB_OT_NAME_ID_UNIQUE_ID: hb_ot_name_id_predefined_t = 3;
/// Full font name that reflects all family and relevant subfamily descriptors.
pub const HB_OT_NAME_ID_FULL_NAME: hb_ot_name_id_predefined_t = 4;
/// Version string.
pub const HB_OT_NAME_ID_VERSION_STRING: hb_ot_name_id_predefined_t = 5;
/// PostScript name for the font.
pub const HB_OT_NAME_ID_POSTSCRIPT_NAME: hb_ot_name_id_predefined_t = 6;
/// Trademark.
pub const HB_OT_NAME_ID_TRADEMARK: hb_ot_name_id_predefined_t = 7;
/// Manufacturer Name.
pub const HB_OT_NAME_ID_MANUFACTURER: hb_ot_name_id_predefined_t = 8;
/// Designer.
pub const HB_OT_NAME_ID_DESIGNER: hb_ot_name_id_predefined_t = 9;
/// Description.
pub const HB_OT_NAME_ID_DESCRIPTION: hb_ot_name_id_predefined_t = 10;
/// URL of font vendor.
pub const HB_OT_NAME_ID_VENDOR_URL: hb_ot_name_id_predefined_t = 11;
/// URL of typeface designer.
pub const HB_OT_NAME_ID_DESIGNER_URL: hb_ot_name_id_predefined_t = 12;
/// License Description.
pub const HB_OT_NAME_ID_LICENSE: hb_ot_name_id_predefined_t = 13;
/// URL where additional licensing information can be found.
pub const HB_OT_NAME_ID_LICENSE_URL: hb_ot_name_id_predefined_t = 14;

// Identifier 15 is reserved by the OpenType specification; upstream leaves it
// commented out and so has no constant for it.

/// Typographic Family name.
pub const HB_OT_NAME_ID_TYPOGRAPHIC_FAMILY: hb_ot_name_id_predefined_t = 16;
/// Typographic Subfamily name.
pub const HB_OT_NAME_ID_TYPOGRAPHIC_SUBFAMILY: hb_ot_name_id_predefined_t = 17;
/// Compatible Full Name for MacOS.
pub const HB_OT_NAME_ID_MAC_FULL_NAME: hb_ot_name_id_predefined_t = 18;
/// Sample text.
pub const HB_OT_NAME_ID_SAMPLE_TEXT: hb_ot_name_id_predefined_t = 19;
/// PostScript CID findfont name.
pub const HB_OT_NAME_ID_CID_FINDFONT_NAME: hb_ot_name_id_predefined_t = 20;
/// WWS Family Name.
pub const HB_OT_NAME_ID_WWS_FAMILY: hb_ot_name_id_predefined_t = 21;
/// WWS Subfamily Name.
pub const HB_OT_NAME_ID_WWS_SUBFAMILY: hb_ot_name_id_predefined_t = 22;
/// Light Background Palette.
pub const HB_OT_NAME_ID_LIGHT_BACKGROUND: hb_ot_name_id_predefined_t = 23;
/// Dark Background Palette.
pub const HB_OT_NAME_ID_DARK_BACKGROUND: hb_ot_name_id_predefined_t = 24;
/// Variations PostScript Name Prefix.
pub const HB_OT_NAME_ID_VARIATIONS_PS_PREFIX: hb_ot_name_id_predefined_t = 25;

/// Value to represent a nonexistent name ID.
pub const HB_OT_NAME_ID_INVALID: hb_ot_name_id_predefined_t = 0xFFFF;

/// An integral type representing an OpenType `name` table name identifier.
///
/// There are predefined name IDs — see [`hb_ot_name_id_predefined_t`] — as well
/// as name IDs returned from other API. These can be used to fetch name strings
/// from a font face.
///
/// Since HarfBuzz 2.0.0.
pub type hb_ot_name_id_t = c_uint;

/// A name ID in a particular language, as reported by
/// [`hb_ot_name_list_names`].
///
/// The pair identifies one string the face can produce: pass both back to
/// [`hb_ot_name_get_utf8`] and friends to read it.
///
/// Since HarfBuzz 2.1.0.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_ot_name_entry_t {
    /// The name ID this entry stands for.
    pub name_id: hb_ot_name_id_t,

    /// Private to HarfBuzz, which stores the record's encoding score and its
    /// index within the table here. Present so that the Rust layout matches
    /// the C one; do not read or write it.
    pub var: hb_var_int_t,

    /// The language the string is written in, as a BCP 47 tag interned by
    /// HarfBuzz.
    pub language: hb_language_t,
}

// `hb_var_int_t` is a union and so has no `Debug`; print the two public fields
// and leave the private slot out.
impl core::fmt::Debug for hb_ot_name_entry_t {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("hb_ot_name_entry_t")
            .field("name_id", &self.name_id)
            .field("language", &self.language)
            .finish_non_exhaustive()
    }
}

unsafe extern "C" {
    /// Enumerates all available name IDs and language combinations.
    ///
    /// When `num_entries` is non-null it receives the number of entries
    /// returned.
    ///
    /// The returned array is owned by `face` and should not be modified. It can
    /// be used for as long as `face` is alive.
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_name_list_names(
        face: *mut hb_face_t,
        num_entries: *mut c_uint,
    ) -> *const hb_ot_name_entry_t;

    /// Fetches a font name from the OpenType `name` table, in UTF-8.
    ///
    /// If `language` is [`HB_LANGUAGE_INVALID`](crate::HB_LANGUAGE_INVALID),
    /// English (`"en"`) is assumed. A NUL terminator is always written for
    /// convenience, and is not included in the output `text_size`.
    ///
    /// `text_size` is in/out and may be null: on input it is the capacity of
    /// `text` in bytes, including room for the terminator; on output it is the
    /// number of bytes actually written, excluding it.
    ///
    /// Returns the full length of the requested string in bytes — which may
    /// exceed the number written — or 0 if the name was not found.
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_name_get_utf8(
        face: *mut hb_face_t,
        name_id: hb_ot_name_id_t,
        language: hb_language_t,
        text_size: *mut c_uint,
        text: *mut c_char,
    ) -> c_uint;

    /// Fetches a font name from the OpenType `name` table, in UTF-16.
    ///
    /// If `language` is [`HB_LANGUAGE_INVALID`](crate::HB_LANGUAGE_INVALID),
    /// English (`"en"`) is assumed. A NUL terminator is always written for
    /// convenience, and is not included in the output `text_size`.
    ///
    /// `text_size` is in/out and may be null: on input it is the capacity of
    /// `text` in `u16` units, including room for the terminator; on output it
    /// is the number of units actually written, excluding it.
    ///
    /// Returns the full length of the requested string in `u16` units — which
    /// may exceed the number written — or 0 if the name was not found.
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_name_get_utf16(
        face: *mut hb_face_t,
        name_id: hb_ot_name_id_t,
        language: hb_language_t,
        text_size: *mut c_uint,
        text: *mut u16,
    ) -> c_uint;

    /// Fetches a font name from the OpenType `name` table, in UTF-32.
    ///
    /// If `language` is [`HB_LANGUAGE_INVALID`](crate::HB_LANGUAGE_INVALID),
    /// English (`"en"`) is assumed. A NUL terminator is always written for
    /// convenience, and is not included in the output `text_size`.
    ///
    /// `text_size` is in/out and may be null: on input it is the capacity of
    /// `text` in `u32` units, including room for the terminator; on output it
    /// is the number of units actually written, excluding it.
    ///
    /// Returns the full length of the requested string in `u32` units — that
    /// is, in Unicode code points, which may exceed the number written — or 0
    /// if the name was not found.
    ///
    /// Since HarfBuzz 2.1.0.
    pub fn hb_ot_name_get_utf32(
        face: *mut hb_face_t,
        name_id: hb_ot_name_id_t,
        language: hb_language_t,
        text_size: *mut c_uint,
        text: *mut u32,
    ) -> c_uint;
}
