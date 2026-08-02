//! Safe, idiomatic Rust bindings for HarfBuzz.
//!
//! This is a placeholder that exists to validate the build.

/// The version of the vendored HarfBuzz this crate was built against.
pub fn version() -> (u32, u32, u32) {
    let (mut major, mut minor, mut micro) = (0, 0, 0);

    // SAFETY: `hb_version` writes one `unsigned` through each pointer. All
    // three point to live, correctly aligned `u32` locals.
    unsafe { harfbuzz_sys::hb_version(&mut major, &mut minor, &mut micro) };

    (major, minor, micro)
}

#[cfg(test)]
mod tests {
    #[test]
    fn reports_the_vendored_version() {
        assert_eq!(super::version(), (14, 3, 0));
    }
}
