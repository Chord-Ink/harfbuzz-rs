//! Compile-time and run-time library version information — `hb-version.h`.

use core::ffi::{CStr, c_char, c_uint};

use crate::hb_bool_t;

/// The major component of the library version available at compile-time.
///
/// C declares this as a plain integer literal. It is spelled `c_uint` here to
/// match [`hb_version`] and [`hb_version_atleast`], which are the only places
/// the value is ever used.
pub const HB_VERSION_MAJOR: c_uint = 14;

/// The minor component of the library version available at compile-time.
pub const HB_VERSION_MINOR: c_uint = 3;

/// The micro component of the library version available at compile-time.
pub const HB_VERSION_MICRO: c_uint = 0;

/// A string containing the library version available at compile-time.
///
/// C declares this as a string literal, so it is transcribed as a
/// NUL-terminated [`CStr`] and can be handed straight to a C API. Call
/// [`CStr::to_str`] for the Rust string `"14.3.0"`.
pub const HB_VERSION_STRING: &CStr = c"14.3.0";

/// Tests the library version at compile-time against a minimum value, given as
/// three integer components.
///
/// This is the Rust equivalent of the C `HB_VERSION_ATLEAST` macro, usable in a
/// `const` context:
///
/// ```
/// # use harfbuzz_sys::HB_VERSION_ATLEAST;
/// const HAS_DRAW_API: bool = HB_VERSION_ATLEAST(4, 0, 0);
/// ```
///
/// Because this crate builds the HarfBuzz sources it vendors, the compile-time
/// version and the run-time version always agree; see [`hb_version_atleast`]
/// for the run-time test that matters when linking against a system library.
#[inline]
pub const fn HB_VERSION_ATLEAST(major: c_uint, minor: c_uint, micro: c_uint) -> bool {
    // Widened to 64 bits so that the comparison cannot overflow for absurd
    // arguments. For every real version number the result is identical to C's.
    const fn encode(major: c_uint, minor: c_uint, micro: c_uint) -> u64 {
        (major as u64) * 10000 + (minor as u64) * 100 + (micro as u64)
    }

    encode(major, minor, micro) <= encode(HB_VERSION_MAJOR, HB_VERSION_MINOR, HB_VERSION_MICRO)
}

unsafe extern "C" {
    /// Returns library version as three integer components.
    ///
    /// All three out-parameters are written; none may be null.
    pub fn hb_version(major: *mut c_uint, minor: *mut c_uint, micro: *mut c_uint);

    /// Returns library version as a string with three components.
    ///
    /// The returned pointer is owned by HarfBuzz, is valid for the lifetime of
    /// the library, and must not be freed.
    pub fn hb_version_string() -> *const c_char;

    /// Tests the library version against a minimum value, given as three
    /// integer components.
    ///
    /// Returns true if the library's version is equal to or greater than the
    /// version requested.
    pub fn hb_version_atleast(major: c_uint, minor: c_uint, micro: c_uint) -> hb_bool_t;
}
