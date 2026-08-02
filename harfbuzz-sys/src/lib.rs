//! Raw FFI bindings to HarfBuzz.
//!
//! This is a placeholder that exists to validate the build; the generated
//! surface replaces it.

use core::ffi::c_char;

unsafe extern "C" {
    pub fn hb_version_string() -> *const c_char;
    pub fn hb_version(major: *mut u32, minor: *mut u32, micro: *mut u32);
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ffi::CStr;

    #[test]
    fn links_against_the_vendored_version() {
        // SAFETY: `hb_version_string` takes no arguments and returns a pointer
        // to a static, NUL-terminated string owned by the library, so it is
        // valid for the lifetime of the process.
        let version = unsafe { CStr::from_ptr(hb_version_string()) };
        assert_eq!(version.to_str().unwrap(), "14.3.0");
    }
}
