//! GLib integration — `hb-glib.h`.
//!
//! GLib carries its own copy of the Unicode Character Database, and this module
//! lets HarfBuzz use it instead of the compact one HarfBuzz ships. That is
//! worth doing when the host application already links GLib: it drops
//! HarfBuzz's own tables from the binary and keeps every consumer agreeing on
//! one Unicode version.
//!
//! It also converts between GLib's script enumeration and HarfBuzz's, and wraps
//! a `GBytes` as a blob so font data can cross between the two libraries
//! without being copied.
//!
//! Requires the `glib` feature, which makes the build script locate GLib with
//! `pkg-config` and compile `hb-glib.cc` into the library.

use crate::{hb_blob_t, hb_script_t, hb_unicode_funcs_t};

/// GLib's `GUnicodeScript` enumeration.
///
/// Declared here as a plain integer rather than pulled in from a GLib binding,
/// so this crate does not acquire a dependency on one. The values are GLib's;
/// pass them through from whatever `glib` crate you use.
pub type GUnicodeScript = core::ffi::c_int;

/// GLib's reference-counted immutable byte buffer, `GBytes`.
///
/// Opaque here for the same reason as [`GUnicodeScript`]: this crate names the
/// type so the signatures are honest, without binding GLib itself.
crate::opaque_handle! {
    GBytes
}

unsafe extern "C" {
    /// Converts a `GUnicodeScript` to the corresponding [`hb_script_t`].
    ///
    /// Since HarfBuzz 0.9.38.
    pub fn hb_glib_script_to_script(script: GUnicodeScript) -> hb_script_t;

    /// Converts an [`hb_script_t`] to the corresponding `GUnicodeScript`.
    ///
    /// Since HarfBuzz 0.9.38.
    pub fn hb_glib_script_from_script(script: hb_script_t) -> GUnicodeScript;

    /// Fetches a Unicode-functions structure backed by GLib.
    ///
    /// The returned object is a singleton owned by HarfBuzz; it does not need
    /// to be destroyed.
    ///
    /// Since HarfBuzz 0.9.38.
    pub fn hb_glib_get_unicode_funcs() -> *mut hb_unicode_funcs_t;

    /// Creates a blob wrapping a `GBytes`, without copying the data.
    ///
    /// The blob takes a reference on `gbytes` and releases it when the blob is
    /// destroyed.
    ///
    /// Upstream guards this behind `GLIB_CHECK_VERSION(2, 31, 10)`. Every GLib
    /// new enough to satisfy HarfBuzz's own minimum of 2.30 in practice ships
    /// `GBytes`, so it is declared unconditionally here; calling it against an
    /// older GLib is a link error rather than a compile error.
    ///
    /// Since HarfBuzz 0.9.38.
    pub fn hb_glib_blob_create(gbytes: *mut GBytes) -> *mut hb_blob_t;
}
