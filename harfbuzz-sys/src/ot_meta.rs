//! Metadata entries from a font's OpenType `meta` table — `hb-ot-meta.h`.
//!
//! The `meta` table is a tag-keyed dictionary of opaque byte strings; this
//! module lists the tags a face carries and hands back the bytes for one.

use core::ffi::{c_int, c_uint};

use crate::{HB_TAG, hb_blob_t, hb_face_t};

/// A metadata entry tag in a font's OpenType `meta` table.
///
/// Known tags are listed in the
/// [OpenType `meta` table specification](https://docs.microsoft.com/en-us/typography/opentype/spec/meta).
///
/// The alias is a signed C `int` because the C enumeration ends with a private
/// sentinel equal to [`HB_TAG_MAX_SIGNED`](crate::HB_TAG_MAX_SIGNED), which
/// pins the underlying type. That sentinel also signals that the value space is
/// open: [`hb_ot_meta_get_entry_tags`] returns whatever tags the font data
/// happens to contain, and [`hb_ot_meta_reference_entry`] accepts any tag, so
/// values outside the constants below are normal rather than exceptional.
///
/// Since HarfBuzz 2.6.0.
pub type hb_ot_meta_tag_t = c_int;

/// Design languages (`dlng`) — text, using only Basic Latin (ASCII)
/// characters, indicating languages and/or scripts for the user audiences that
/// the font was primarily designed for.
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_META_TAG_DESIGN_LANGUAGES: hb_ot_meta_tag_t =
    HB_TAG(b'd', b'l', b'n', b'g') as hb_ot_meta_tag_t;

/// Supported languages (`slng`) — text, using only Basic Latin (ASCII)
/// characters, indicating languages and/or scripts that the font is declared to
/// be capable of supporting.
///
/// Since HarfBuzz 2.6.0.
pub const HB_OT_META_TAG_SUPPORTED_LANGUAGES: hb_ot_meta_tag_t =
    HB_TAG(b's', b'l', b'n', b'g') as hb_ot_meta_tag_t;

unsafe extern "C" {
    /// Fetches the metadata entry tags present in a face, starting at
    /// `start_offset` within the face's list of entries.
    ///
    /// `entries_count` is an in/out parameter: on input the capacity of
    /// `entries`, on output how many tags were written. Both it and `entries`
    /// may be null, in which case nothing is written.
    ///
    /// Returns the total number of metadata entries in the face, regardless of
    /// how many were written into `entries`.
    ///
    /// Since HarfBuzz 2.6.0.
    pub fn hb_ot_meta_get_entry_tags(
        face: *mut hb_face_t,
        start_offset: c_uint,
        entries_count: *mut c_uint,
        entries: *mut hb_ot_meta_tag_t,
    ) -> c_uint;

    /// Fetches the metadata entry stored under `meta_tag` in a face.
    ///
    /// Returns a blob holding that entry's bytes. Release it with
    /// [`hb_blob_destroy`](crate::hb_blob_destroy).
    ///
    /// Since HarfBuzz 2.6.0.
    pub fn hb_ot_meta_reference_entry(
        face: *mut hb_face_t,
        meta_tag: hb_ot_meta_tag_t,
    ) -> *mut hb_blob_t;
}
