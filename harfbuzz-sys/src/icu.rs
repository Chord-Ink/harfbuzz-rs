//! ICU integration — `hb-icu.h`.
//!
//! Shaping needs Unicode character properties: the General Category of a code
//! point, its Canonical Combining Class, its mirrored form, its Script, and the
//! canonical composition and decomposition mappings. HarfBuzz answers those
//! questions through an [`hb_unicode_funcs_t`] and ships its own compact copy of
//! the Unicode Character Database to back the default one.
//!
//! This sub-library offers an alternative source for exactly that data: the
//! International Components for Unicode. [`hb_icu_get_unicode_funcs`] returns a
//! process-wide, immutable [`hb_unicode_funcs_t`] whose virtual methods call
//! into ICU, and `hb_buffer_set_unicode_funcs` attaches it to a buffer. The
//! usual reason to do this is consistency rather than speed: a program that
//! already links ICU gets one Unicode version — ICU's — driving line breaking,
//! normalization, bidi, *and* shaping, instead of two possibly different ones.
//!
//! The other two functions translate between the two libraries' script
//! enumerations. HarfBuzz spells a script as an ISO 15924 four-letter tag in an
//! [`hb_script_t`]; ICU spells it as a numeric `UScriptCode`. Convert with
//! [`hb_icu_script_to_script`] and [`hb_icu_script_from_script`]; both route
//! through the ISO 15924 short name, so they stay correct as ICU adds scripts.
//!
//! Nothing here changes how HarfBuzz shapes. The OpenType shaper, the font
//! machinery, and the buffer API are the same; only the answer to "what is this
//! code point?" comes from somewhere else.
//!
//! This module is compiled only when the crate's `icu` feature is enabled, which
//! is also what makes `build.rs` find ICU with `pkg-config` (the `icu-uc`
//! package), compile HarfBuzz's ICU sources, and link against it. Because of
//! that gating the module is reached as `harfbuzz_sys::icu` rather than being
//! re-exported at the crate root.

use core::ffi::c_int;

use crate::{hb_script_t, hb_unicode_funcs_t};

// ---------------------------------------------------------------------------
// Foreign types
//
// `UScriptCode` belongs to ICU, not to HarfBuzz, and this crate has no
// dependency that supplies it. It is declared here so that the signatures
// below can be written honestly.
// ---------------------------------------------------------------------------

/// ICU's script enumeration, declared as `UScriptCode` in
/// `<unicode/uscript.h>`.
///
/// A plain C enumeration whose values run from `-1` (`USCRIPT_INVALID_CODE`)
/// upwards — `0` is `USCRIPT_COMMON`, `1` is `USCRIPT_INHERITED`, and so on, in
/// the order ICU happened to add them. Because it has a negative enumerator and
/// every value fits in an `int`, C compilers give it `int` as its underlying
/// type; this alias is therefore `c_int`, which is ABI-compatible with the
/// `UScriptCode` of the `icu` and `icu_sys` crates.
///
/// Only [`USCRIPT_INVALID_CODE`] is restated below. The individual script codes
/// belong to ICU, and ICU appends new ones as Unicode grows, so take them from
/// ICU's own headers or bindings rather than from here.
pub type UScriptCode = c_int;

/// ICU's "not a script" value, `USCRIPT_INVALID_CODE`.
///
/// Both conversions in this module use it: [`hb_icu_script_to_script`] maps it
/// to `HB_SCRIPT_INVALID`, and [`hb_icu_script_from_script`] returns it when no
/// ICU script matches.
pub const USCRIPT_INVALID_CODE: UScriptCode = -1;

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

unsafe extern "C" {
    /// Converts an ICU [`UScriptCode`] into the corresponding [`hb_script_t`].
    ///
    /// The conversion goes through the script's ISO 15924 four-letter short
    /// name, so it does not depend on ICU's numeric values staying put.
    ///
    /// Returns `HB_SCRIPT_INVALID` for [`USCRIPT_INVALID_CODE`] and for any
    /// value ICU does not recognise. A script ICU knows but HarfBuzz does not
    /// comes back as the tag itself, which is what `hb_script_from_string`
    /// produces for an unknown four-letter code.
    ///
    /// The header carries no `Since:` annotation; upstream's `NEWS` lists this
    /// function under HarfBuzz 0.6.0.
    pub fn hb_icu_script_to_script(script: UScriptCode) -> hb_script_t;

    /// Converts an [`hb_script_t`] into the corresponding ICU [`UScriptCode`].
    ///
    /// The four bytes of the script tag are handed to ICU's `uscript_getCode`,
    /// so again the ISO 15924 name is the common ground rather than any numeric
    /// value.
    ///
    /// Returns [`USCRIPT_INVALID_CODE`] for `HB_SCRIPT_INVALID` and whenever
    /// ICU does not recognise the tag — including scripts too new for the
    /// linked ICU. ICU's error code is discarded, so that sentinel is the only
    /// failure signal.
    ///
    /// The header carries no `Since:` annotation; upstream's `NEWS` lists this
    /// function under HarfBuzz 0.6.0.
    pub fn hb_icu_script_from_script(script: hb_script_t) -> UScriptCode;

    /// Fetches the Unicode-functions structure backed by ICU.
    ///
    /// The returned [`hb_unicode_funcs_t`] answers combining class, general
    /// category, mirroring, script, compose, and decompose queries by calling
    /// into ICU. Attach it to a buffer with `hb_buffer_set_unicode_funcs` to
    /// make shaping use ICU's Unicode data instead of HarfBuzz's built-in
    /// tables.
    ///
    /// This is a process-wide singleton: it is built on first use, made
    /// immutable, cached, and released at exit. Ownership is not transferred —
    /// do not destroy it unless you took your own reference with
    /// `hb_unicode_funcs_reference` first. Repeated calls return the same
    /// pointer, and the lazy initialisation is thread-safe.
    ///
    /// Since HarfBuzz 0.9.38.
    pub fn hb_icu_get_unicode_funcs() -> *mut hb_unicode_funcs_t;
}
