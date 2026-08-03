//! Fundamental types shared by the whole API — `hb-common.h`.

use core::ffi::{c_char, c_float, c_int, c_uint, c_void};

/// Data type for booleans. Zero is false; any other value is true.
pub type hb_bool_t = c_int;

/// Data type for holding Unicode codepoints. Also used to hold glyph IDs.
pub type hb_codepoint_t = u32;

/// An unused [`hb_codepoint_t`] value.
pub const HB_CODEPOINT_INVALID: hb_codepoint_t = u32::MAX;

/// Data type for holding a single coordinate value.
///
/// Contour points and other multi-dimensional data are stored as tuples of
/// these.
pub type hb_position_t = i32;

/// Data type for bitmasks.
pub type hb_mask_t = u32;

/// Data type for tag identifiers.
///
/// Tags are four-byte integers, each byte representing a character. They
/// identify tables, design-variation axes, scripts, languages, font features,
/// and baselines with human-readable names.
pub type hb_tag_t = u32;

/// Constructs an [`hb_tag_t`] from four bytes.
///
/// The Rust equivalent of the C `HB_TAG` macro, usable in a `const` context:
///
/// ```
/// # use harfbuzz_sys::HB_TAG;
/// const KERN: harfbuzz_sys::hb_tag_t = HB_TAG(b'k', b'e', b'r', b'n');
/// ```
#[inline]
pub const fn HB_TAG(c1: u8, c2: u8, c3: u8, c4: u8) -> hb_tag_t {
    ((c1 as u32) << 24) | ((c2 as u32) << 16) | ((c3 as u32) << 8) | (c4 as u32)
}

/// Extracts the four bytes of an [`hb_tag_t`], most significant first.
#[inline]
pub const fn HB_UNTAG(tag: hb_tag_t) -> [u8; 4] {
    tag.to_be_bytes()
}

/// An unset [`hb_tag_t`].
pub const HB_TAG_NONE: hb_tag_t = HB_TAG(0, 0, 0, 0);

/// The maximum possible unsigned [`hb_tag_t`].
pub const HB_TAG_MAX: hb_tag_t = HB_TAG(0xff, 0xff, 0xff, 0xff);

/// The maximum possible signed [`hb_tag_t`].
pub const HB_TAG_MAX_SIGNED: hb_tag_t = HB_TAG(0x7f, 0xff, 0xff, 0xff);

/// The direction of a text segment or buffer.
///
/// Valid directions are [`HB_DIRECTION_LTR`] through [`HB_DIRECTION_BTT`]; the
/// `HB_DIRECTION_IS_*` helpers below classify them.
pub type hb_direction_t = c_int;

/// Initial, unset direction.
pub const HB_DIRECTION_INVALID: hb_direction_t = 0;
/// Text is set horizontally from left to right.
pub const HB_DIRECTION_LTR: hb_direction_t = 4;
/// Text is set horizontally from right to left.
pub const HB_DIRECTION_RTL: hb_direction_t = 5;
/// Text is set vertically from top to bottom.
pub const HB_DIRECTION_TTB: hb_direction_t = 6;
/// Text is set vertically from bottom to top.
pub const HB_DIRECTION_BTT: hb_direction_t = 7;

/// Tests whether a text direction is valid.
#[inline]
pub const fn HB_DIRECTION_IS_VALID(dir: hb_direction_t) -> bool {
    (dir as u32) & !3u32 == 4
}

/// Tests whether a text direction is horizontal. Requires a valid direction.
#[inline]
pub const fn HB_DIRECTION_IS_HORIZONTAL(dir: hb_direction_t) -> bool {
    (dir as u32) & !1u32 == 4
}

/// Tests whether a text direction is vertical. Requires a valid direction.
#[inline]
pub const fn HB_DIRECTION_IS_VERTICAL(dir: hb_direction_t) -> bool {
    (dir as u32) & !1u32 == 6
}

/// Tests whether a text direction moves forward — left to right, or top to
/// bottom. Requires a valid direction.
#[inline]
pub const fn HB_DIRECTION_IS_FORWARD(dir: hb_direction_t) -> bool {
    (dir as u32) & !2u32 == 4
}

/// Tests whether a text direction moves backward — right to left, or bottom to
/// top. Requires a valid direction.
#[inline]
pub const fn HB_DIRECTION_IS_BACKWARD(dir: hb_direction_t) -> bool {
    (dir as u32) & !2u32 == 5
}

/// Reverses a text direction. Requires a valid direction.
#[inline]
pub const fn HB_DIRECTION_REVERSE(dir: hb_direction_t) -> hb_direction_t {
    ((dir as u32) ^ 1) as hb_direction_t
}

crate::opaque_handle! {
    /// The object [`hb_language_t`] points at. Never dereferenced directly.
    hb_language_impl_t
}

/// Data type for languages. Each one corresponds to a BCP 47 language tag.
///
/// Languages are interned by HarfBuzz and live for the lifetime of the process,
/// so an `hb_language_t` never needs to be freed and can be compared by pointer.
pub type hb_language_t = *const hb_language_impl_t;

/// An unset [`hb_language_t`].
pub const HB_LANGUAGE_INVALID: hb_language_t = core::ptr::null();

/// Data structure for holding user-data keys.
///
/// The contents are private; HarfBuzz uses the *address* of an
/// `hb_user_data_key_t` as the key, never its value.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct hb_user_data_key_t {
    /// Private padding. Never read by HarfBuzz.
    pub unused: c_char,
}

/// A virtual method for destroy user-data callbacks.
pub type hb_destroy_func_t = Option<unsafe extern "C" fn(user_data: *mut c_void)>;

/// Special setting for [`hb_feature_t::start`] to apply a feature from the
/// start of the buffer.
pub const HB_FEATURE_GLOBAL_START: c_uint = 0;

/// Special setting for [`hb_feature_t::end`] to apply a feature to the end of
/// the buffer.
pub const HB_FEATURE_GLOBAL_END: c_uint = c_uint::MAX;

/// Information about a requested feature application.
///
/// The feature is applied with `value` to every glyph in a cluster between
/// `start` (inclusive) and `end` (exclusive). Using
/// [`HB_FEATURE_GLOBAL_START`] and [`HB_FEATURE_GLOBAL_END`] applies it to the
/// whole buffer.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_feature_t {
    /// The tag of the feature.
    pub tag: hb_tag_t,
    /// The value of the feature. Zero disables it; non-zero (usually one)
    /// enables it. For features implemented as lookup type 3 — `salt`, for
    /// instance — this is a one-based index into the alternates.
    pub value: u32,
    /// The cluster to start applying this feature setting (inclusive).
    pub start: c_uint,
    /// The cluster to stop applying this feature setting (exclusive).
    pub end: c_uint,
}

/// Data type for holding variation data.
///
/// Registered OpenType variation-axis tags are listed in the
/// [OpenType Axis Tag Registry](https://docs.microsoft.com/en-us/typography/opentype/spec/dvaraxisreg).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct hb_variation_t {
    /// The tag of the variation-axis name.
    pub tag: hb_tag_t,
    /// The value of the variation axis.
    pub value: c_float,
}

/// Data type for holding color values: eight bits per channel, RGB plus alpha.
pub type hb_color_t = u32;

/// Constructs an [`hb_color_t`] from four channel values.
///
/// Note the argument order, which follows the C macro: blue, green, red, alpha.
#[inline]
pub const fn HB_COLOR(b: u8, g: u8, r: u8, a: u8) -> hb_color_t {
    HB_TAG(b, g, r, a)
}

/// Glyph extent values, measured in font units.
///
/// Note that `height` is negative in coordinate systems that grow up.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_glyph_extents_t {
    /// Distance from the x-origin to the left extremum of the glyph.
    pub x_bearing: hb_position_t,
    /// Distance from the top extremum of the glyph to the y-origin.
    pub y_bearing: hb_position_t,
    /// Distance from the left extremum of the glyph to the right extremum.
    pub width: hb_position_t,
    /// Distance from the top extremum of the glyph to the bottom extremum.
    pub height: hb_position_t,
}

/// A union of the integer widths HarfBuzz stores in a single 32-bit slot.
#[repr(C)]
#[derive(Clone, Copy)]
pub union hb_var_int_t {
    pub u32_: u32,
    pub i32_: i32,
    pub u16_: [u16; 2],
    pub i16_: [i16; 2],
    pub u8_: [u8; 4],
    pub i8_: [i8; 4],
}

/// As [`hb_var_int_t`], but also able to hold a `float`.
#[repr(C)]
#[derive(Clone, Copy)]
pub union hb_var_num_t {
    pub f: c_float,
    pub u32_: u32,
    pub i32_: i32,
    pub u16_: [u16; 2],
    pub i16_: [i16; 2],
    pub u8_: [u8; 4],
    pub i8_: [i8; 4],
}

unsafe extern "C" {
    /// Converts a string into an [`hb_tag_t`].
    ///
    /// Pass `len` as `-1` when `str_` is NUL-terminated.
    pub fn hb_tag_from_string(str_: *const c_char, len: c_int) -> hb_tag_t;

    /// Converts an [`hb_tag_t`] into a string. `buf` must have room for four
    /// bytes, and the result is *not* NUL-terminated.
    pub fn hb_tag_to_string(tag: hb_tag_t, buf: *mut c_char);

    /// Converts a string to an [`hb_direction_t`].
    ///
    /// Matching is loose and case-insensitive: only the first character
    /// matters. Pass `len` as `-1` when `str_` is NUL-terminated.
    pub fn hb_direction_from_string(str_: *const c_char, len: c_int) -> hb_direction_t;

    /// Converts an [`hb_direction_t`] to a string.
    pub fn hb_direction_to_string(direction: hb_direction_t) -> *const c_char;

    /// Converts a string to an [`hb_language_t`].
    ///
    /// Pass `len` as `-1` when `str_` is NUL-terminated.
    pub fn hb_language_from_string(str_: *const c_char, len: c_int) -> hb_language_t;

    /// Converts an [`hb_language_t`] to a string.
    pub fn hb_language_to_string(language: hb_language_t) -> *const c_char;

    /// Fetches the default language from the current locale.
    ///
    /// This function is *not* thread-safe the first time it is called, because
    /// it caches the result of inspecting the locale.
    pub fn hb_language_get_default() -> hb_language_t;

    /// Checks whether `specific` is a more specific tagging of `language`.
    pub fn hb_language_matches(language: hb_language_t, specific: hb_language_t) -> hb_bool_t;

    /// Parses a string into an [`hb_feature_t`].
    ///
    /// The format is the one accepted by the `hb-shape` utility, for example
    /// `"kern"`, `"-liga"`, or `"aalt[3:5]=2"`.
    pub fn hb_feature_from_string(
        str_: *const c_char,
        len: c_int,
        feature: *mut hb_feature_t,
    ) -> hb_bool_t;

    /// Converts an [`hb_feature_t`] into a NUL-terminated string, writing at
    /// most `size` bytes into `buf`.
    pub fn hb_feature_to_string(feature: *mut hb_feature_t, buf: *mut c_char, size: c_uint);

    /// Parses a string into an [`hb_variation_t`], for example `"wght=700"`.
    pub fn hb_variation_from_string(
        str_: *const c_char,
        len: c_int,
        variation: *mut hb_variation_t,
    ) -> hb_bool_t;

    /// Converts an [`hb_variation_t`] into a NUL-terminated string, writing at
    /// most `size` bytes into `buf`.
    pub fn hb_variation_to_string(variation: *mut hb_variation_t, buf: *mut c_char, size: c_uint);

    /// Fetches the alpha channel of a colour.
    pub fn hb_color_get_alpha(color: hb_color_t) -> u8;
    /// Fetches the red channel of a colour.
    pub fn hb_color_get_red(color: hb_color_t) -> u8;
    /// Fetches the green channel of a colour.
    pub fn hb_color_get_green(color: hb_color_t) -> u8;
    /// Fetches the blue channel of a colour.
    pub fn hb_color_get_blue(color: hb_color_t) -> u8;

    /// Allocates memory with HarfBuzz's allocator.
    ///
    /// Only needed when handing ownership of a buffer to a HarfBuzz function
    /// that will free it later — see [`hb_free`].
    pub fn hb_malloc(size: usize) -> *mut c_void;
    /// Allocates zero-initialized memory with HarfBuzz's allocator.
    pub fn hb_calloc(nmemb: usize, size: usize) -> *mut c_void;
    /// Resizes a block obtained from [`hb_malloc`] or [`hb_calloc`].
    pub fn hb_realloc(ptr: *mut c_void, size: usize) -> *mut c_void;
    /// Frees a block obtained from [`hb_malloc`], [`hb_calloc`], or
    /// [`hb_realloc`].
    pub fn hb_free(ptr: *mut c_void);
}
