//! Unicode character-property callbacks — the data HarfBuzz needs about
//! characters before it can shape them — `hb-unicode.h`.

use core::ffi::{c_int, c_void};

use crate::{
    hb_bool_t, hb_codepoint_t, hb_destroy_func_t, hb_script_t, hb_user_data_key_t,
};

/// Maximum valid Unicode code point.
///
/// Since HarfBuzz 1.9.0.
pub const HB_UNICODE_MAX: hb_codepoint_t = 0x10FFFF;

/// Data type for the General_Category (gc) property from the Unicode Character
/// Database.
///
/// The values are HarfBuzz's own numbering, not the numbering used by any other
/// library, so never transmute a category from ICU or GLib into this type — map
/// it explicitly.
//
// The C enumeration has no sentinel and every enumerator is a small
// non-negative number, so it fits in `int`.
pub type hb_unicode_general_category_t = c_int;

/// Control characters (`Cc`).
pub const HB_UNICODE_GENERAL_CATEGORY_CONTROL: hb_unicode_general_category_t = 0;
/// Format characters (`Cf`).
pub const HB_UNICODE_GENERAL_CATEGORY_FORMAT: hb_unicode_general_category_t = 1;
/// Unassigned code points (`Cn`).
pub const HB_UNICODE_GENERAL_CATEGORY_UNASSIGNED: hb_unicode_general_category_t = 2;
/// Private-use characters (`Co`).
pub const HB_UNICODE_GENERAL_CATEGORY_PRIVATE_USE: hb_unicode_general_category_t = 3;
/// Surrogate code points (`Cs`).
pub const HB_UNICODE_GENERAL_CATEGORY_SURROGATE: hb_unicode_general_category_t = 4;
/// Lowercase letters (`Ll`).
pub const HB_UNICODE_GENERAL_CATEGORY_LOWERCASE_LETTER: hb_unicode_general_category_t = 5;
/// Modifier letters (`Lm`).
pub const HB_UNICODE_GENERAL_CATEGORY_MODIFIER_LETTER: hb_unicode_general_category_t = 6;
/// Other letters (`Lo`).
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER: hb_unicode_general_category_t = 7;
/// Titlecase letters (`Lt`).
pub const HB_UNICODE_GENERAL_CATEGORY_TITLECASE_LETTER: hb_unicode_general_category_t = 8;
/// Uppercase letters (`Lu`).
pub const HB_UNICODE_GENERAL_CATEGORY_UPPERCASE_LETTER: hb_unicode_general_category_t = 9;
/// Spacing combining marks (`Mc`).
pub const HB_UNICODE_GENERAL_CATEGORY_SPACING_MARK: hb_unicode_general_category_t = 10;
/// Enclosing combining marks (`Me`).
pub const HB_UNICODE_GENERAL_CATEGORY_ENCLOSING_MARK: hb_unicode_general_category_t = 11;
/// Non-spacing combining marks (`Mn`).
pub const HB_UNICODE_GENERAL_CATEGORY_NON_SPACING_MARK: hb_unicode_general_category_t = 12;
/// Decimal digits (`Nd`).
pub const HB_UNICODE_GENERAL_CATEGORY_DECIMAL_NUMBER: hb_unicode_general_category_t = 13;
/// Letter-like numbers (`Nl`).
pub const HB_UNICODE_GENERAL_CATEGORY_LETTER_NUMBER: hb_unicode_general_category_t = 14;
/// Other numbers (`No`).
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_NUMBER: hb_unicode_general_category_t = 15;
/// Connector punctuation (`Pc`).
pub const HB_UNICODE_GENERAL_CATEGORY_CONNECT_PUNCTUATION: hb_unicode_general_category_t = 16;
/// Dash punctuation (`Pd`).
pub const HB_UNICODE_GENERAL_CATEGORY_DASH_PUNCTUATION: hb_unicode_general_category_t = 17;
/// Closing punctuation (`Pe`).
pub const HB_UNICODE_GENERAL_CATEGORY_CLOSE_PUNCTUATION: hb_unicode_general_category_t = 18;
/// Final quotation punctuation (`Pf`).
pub const HB_UNICODE_GENERAL_CATEGORY_FINAL_PUNCTUATION: hb_unicode_general_category_t = 19;
/// Initial quotation punctuation (`Pi`).
pub const HB_UNICODE_GENERAL_CATEGORY_INITIAL_PUNCTUATION: hb_unicode_general_category_t = 20;
/// Other punctuation (`Po`).
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_PUNCTUATION: hb_unicode_general_category_t = 21;
/// Opening punctuation (`Ps`).
pub const HB_UNICODE_GENERAL_CATEGORY_OPEN_PUNCTUATION: hb_unicode_general_category_t = 22;
/// Currency symbols (`Sc`).
pub const HB_UNICODE_GENERAL_CATEGORY_CURRENCY_SYMBOL: hb_unicode_general_category_t = 23;
/// Modifier symbols (`Sk`).
pub const HB_UNICODE_GENERAL_CATEGORY_MODIFIER_SYMBOL: hb_unicode_general_category_t = 24;
/// Math symbols (`Sm`).
pub const HB_UNICODE_GENERAL_CATEGORY_MATH_SYMBOL: hb_unicode_general_category_t = 25;
/// Other symbols (`So`).
pub const HB_UNICODE_GENERAL_CATEGORY_OTHER_SYMBOL: hb_unicode_general_category_t = 26;
/// Line separators (`Zl`).
pub const HB_UNICODE_GENERAL_CATEGORY_LINE_SEPARATOR: hb_unicode_general_category_t = 27;
/// Paragraph separators (`Zp`).
pub const HB_UNICODE_GENERAL_CATEGORY_PARAGRAPH_SEPARATOR: hb_unicode_general_category_t = 28;
/// Space separators (`Zs`).
pub const HB_UNICODE_GENERAL_CATEGORY_SPACE_SEPARATOR: hb_unicode_general_category_t = 29;

/// Data type for the Canonical_Combining_Class (ccc) property from the Unicode
/// Character Database.
///
/// The constants below name the classes HarfBuzz's shapers care about, but they
/// are not an exhaustive list: newer versions of Unicode may add new values, and
/// client programs should be ready to handle **any** value in the range 0..=254
/// coming back from [`hb_unicode_combining_class`]. That is precisely why this
/// is an integer alias rather than a Rust `enum`.
//
// The C enumeration has no sentinel and its largest enumerator is 255, so it
// fits in `int`.
pub type hb_unicode_combining_class_t = c_int;

/// Spacing and enclosing marks; also many vowel and consonant signs, even if
/// non-spacing.
pub const HB_UNICODE_COMBINING_CLASS_NOT_REORDERED: hb_unicode_combining_class_t = 0;
/// Marks which overlay a base letter or symbol.
pub const HB_UNICODE_COMBINING_CLASS_OVERLAY: hb_unicode_combining_class_t = 1;
/// Diacritic nukta marks in Brahmi-derived scripts.
pub const HB_UNICODE_COMBINING_CLASS_NUKTA: hb_unicode_combining_class_t = 7;
/// Hiragana/Katakana voicing marks.
pub const HB_UNICODE_COMBINING_CLASS_KANA_VOICING: hb_unicode_combining_class_t = 8;
/// Viramas.
pub const HB_UNICODE_COMBINING_CLASS_VIRAMA: hb_unicode_combining_class_t = 9;

/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC10: hb_unicode_combining_class_t = 10;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC11: hb_unicode_combining_class_t = 11;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC12: hb_unicode_combining_class_t = 12;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC13: hb_unicode_combining_class_t = 13;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC14: hb_unicode_combining_class_t = 14;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC15: hb_unicode_combining_class_t = 15;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC16: hb_unicode_combining_class_t = 16;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC17: hb_unicode_combining_class_t = 17;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC18: hb_unicode_combining_class_t = 18;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC19: hb_unicode_combining_class_t = 19;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC20: hb_unicode_combining_class_t = 20;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC21: hb_unicode_combining_class_t = 21;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC22: hb_unicode_combining_class_t = 22;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC23: hb_unicode_combining_class_t = 23;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC24: hb_unicode_combining_class_t = 24;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC25: hb_unicode_combining_class_t = 25;
/// Hebrew.
pub const HB_UNICODE_COMBINING_CLASS_CCC26: hb_unicode_combining_class_t = 26;

/// Arabic.
pub const HB_UNICODE_COMBINING_CLASS_CCC27: hb_unicode_combining_class_t = 27;
/// Arabic.
pub const HB_UNICODE_COMBINING_CLASS_CCC28: hb_unicode_combining_class_t = 28;
/// Arabic.
pub const HB_UNICODE_COMBINING_CLASS_CCC29: hb_unicode_combining_class_t = 29;
/// Arabic.
pub const HB_UNICODE_COMBINING_CLASS_CCC30: hb_unicode_combining_class_t = 30;
/// Arabic.
pub const HB_UNICODE_COMBINING_CLASS_CCC31: hb_unicode_combining_class_t = 31;
/// Arabic.
pub const HB_UNICODE_COMBINING_CLASS_CCC32: hb_unicode_combining_class_t = 32;
/// Arabic.
pub const HB_UNICODE_COMBINING_CLASS_CCC33: hb_unicode_combining_class_t = 33;
/// Arabic.
pub const HB_UNICODE_COMBINING_CLASS_CCC34: hb_unicode_combining_class_t = 34;
/// Arabic.
pub const HB_UNICODE_COMBINING_CLASS_CCC35: hb_unicode_combining_class_t = 35;

/// Syriac.
pub const HB_UNICODE_COMBINING_CLASS_CCC36: hb_unicode_combining_class_t = 36;

/// Telugu.
pub const HB_UNICODE_COMBINING_CLASS_CCC84: hb_unicode_combining_class_t = 84;
/// Telugu.
pub const HB_UNICODE_COMBINING_CLASS_CCC91: hb_unicode_combining_class_t = 91;

/// Thai.
pub const HB_UNICODE_COMBINING_CLASS_CCC103: hb_unicode_combining_class_t = 103;
/// Thai.
pub const HB_UNICODE_COMBINING_CLASS_CCC107: hb_unicode_combining_class_t = 107;

/// Lao.
pub const HB_UNICODE_COMBINING_CLASS_CCC118: hb_unicode_combining_class_t = 118;
/// Lao.
pub const HB_UNICODE_COMBINING_CLASS_CCC122: hb_unicode_combining_class_t = 122;

/// Tibetan.
pub const HB_UNICODE_COMBINING_CLASS_CCC129: hb_unicode_combining_class_t = 129;
/// Tibetan.
pub const HB_UNICODE_COMBINING_CLASS_CCC130: hb_unicode_combining_class_t = 130;
/// Tibetan.
///
/// Since HarfBuzz 7.2.0.
pub const HB_UNICODE_COMBINING_CLASS_CCC132: hb_unicode_combining_class_t = 132;

/// Marks attached at the bottom left.
pub const HB_UNICODE_COMBINING_CLASS_ATTACHED_BELOW_LEFT: hb_unicode_combining_class_t = 200;
/// Marks attached directly below.
pub const HB_UNICODE_COMBINING_CLASS_ATTACHED_BELOW: hb_unicode_combining_class_t = 202;
/// Marks attached directly above.
pub const HB_UNICODE_COMBINING_CLASS_ATTACHED_ABOVE: hb_unicode_combining_class_t = 214;
/// Marks attached at the top right.
pub const HB_UNICODE_COMBINING_CLASS_ATTACHED_ABOVE_RIGHT: hb_unicode_combining_class_t = 216;
/// Distinct marks at the bottom left.
pub const HB_UNICODE_COMBINING_CLASS_BELOW_LEFT: hb_unicode_combining_class_t = 218;
/// Distinct marks directly below.
pub const HB_UNICODE_COMBINING_CLASS_BELOW: hb_unicode_combining_class_t = 220;
/// Distinct marks at the bottom right.
pub const HB_UNICODE_COMBINING_CLASS_BELOW_RIGHT: hb_unicode_combining_class_t = 222;
/// Distinct marks to the left.
pub const HB_UNICODE_COMBINING_CLASS_LEFT: hb_unicode_combining_class_t = 224;
/// Distinct marks to the right.
pub const HB_UNICODE_COMBINING_CLASS_RIGHT: hb_unicode_combining_class_t = 226;
/// Distinct marks at the top left.
pub const HB_UNICODE_COMBINING_CLASS_ABOVE_LEFT: hb_unicode_combining_class_t = 228;
/// Distinct marks directly above.
pub const HB_UNICODE_COMBINING_CLASS_ABOVE: hb_unicode_combining_class_t = 230;
/// Distinct marks at the top right.
pub const HB_UNICODE_COMBINING_CLASS_ABOVE_RIGHT: hb_unicode_combining_class_t = 232;
/// Distinct marks subtending two bases.
pub const HB_UNICODE_COMBINING_CLASS_DOUBLE_BELOW: hb_unicode_combining_class_t = 233;
/// Distinct marks extending above two bases.
pub const HB_UNICODE_COMBINING_CLASS_DOUBLE_ABOVE: hb_unicode_combining_class_t = 234;

/// Greek iota subscript only.
pub const HB_UNICODE_COMBINING_CLASS_IOTA_SUBSCRIPT: hb_unicode_combining_class_t = 240;

/// Invalid combining class.
///
/// This is a HarfBuzz sentinel, not a Unicode value; the UCD only assigns
/// classes in the range 0..=254.
pub const HB_UNICODE_COMBINING_CLASS_INVALID: hb_unicode_combining_class_t = 255;

opaque_handle! {
    /// A set of virtual methods used for accessing various Unicode character
    /// properties.
    ///
    /// HarfBuzz provides a default implementation of every method — see
    /// [`hb_unicode_funcs_get_default`]. Client programs can implement their own
    /// replacements for the individual Unicode functions, as needed, and install
    /// them by calling the setter for a method.
    ///
    /// A Unicode-functions structure is reference counted and may have a
    /// *parent*: any method left unset falls through to the parent's
    /// implementation, which is what makes it cheap to override just one
    /// property and inherit the rest. Creating a child makes the parent
    /// immutable.
    hb_unicode_funcs_t
}

/// A virtual method for [`hb_unicode_funcs_t`] that retrieves the Canonical
/// Combining Class (ccc) property of a code point.
///
/// Returns the [`hb_unicode_combining_class_t`] of `unicode`.
pub type hb_unicode_combining_class_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_unicode_combining_class_t,
>;

/// A virtual method for [`hb_unicode_funcs_t`] that retrieves the General
/// Category property of a code point.
///
/// Returns the [`hb_unicode_general_category_t`] of `unicode`.
pub type hb_unicode_general_category_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_unicode_general_category_t,
>;

/// A virtual method for [`hb_unicode_funcs_t`] that retrieves the
/// bi-directional mirroring glyph code point of a code point.
///
/// If a code point has no mirroring glyph defined, the method should return the
/// original code point.
pub type hb_unicode_mirroring_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_codepoint_t,
>;

/// A virtual method for [`hb_unicode_funcs_t`] that retrieves the Script
/// property of a code point.
///
/// Returns the [`hb_script_t`] of `unicode`.
pub type hb_unicode_script_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_script_t,
>;

/// A virtual method for [`hb_unicode_funcs_t`] that composes two code points by
/// canonical equivalence.
///
/// On success the composed code point is written through `ab` and the method
/// returns true; otherwise it returns false and `ab` is left alone.
pub type hb_unicode_compose_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        a: hb_codepoint_t,
        b: hb_codepoint_t,
        ab: *mut hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

/// A virtual method for [`hb_unicode_funcs_t`] that decomposes a code point by
/// canonical equivalence.
///
/// On success the two decomposed code points are written through `a` and `b`
/// and the method returns true; otherwise it returns false.
pub type hb_unicode_decompose_func_t = Option<
    unsafe extern "C" fn(
        ufuncs: *mut hb_unicode_funcs_t,
        ab: hb_codepoint_t,
        a: *mut hb_codepoint_t,
        b: *mut hb_codepoint_t,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

unsafe extern "C" {
    /// Fetches the default Unicode-functions structure, the one used when no
    /// functions are explicitly set on a buffer.
    ///
    /// Which implementation this is depends on how HarfBuzz was built: its own
    /// bundled UCD tables normally, otherwise GLib or ICU. The structure is
    /// owned by HarfBuzz and shared process-wide, so the caller does *not* own
    /// the returned reference and should not destroy it. It is immutable in
    /// practice — use it as the parent of your own structure instead of trying
    /// to modify it.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_get_default() -> *mut hb_unicode_funcs_t;

    /// Creates a new Unicode-functions structure that inherits from `parent`.
    ///
    /// Every method starts out delegating to `parent`, so you only need to
    /// install the ones you want to change. `parent` may be null, in which case
    /// the singleton empty structure is used instead. Creating a child takes a
    /// reference on the parent and makes it immutable.
    ///
    /// The caller owns the returned reference and must release it with
    /// [`hb_unicode_funcs_destroy`]. On allocation failure this returns the
    /// singleton empty structure rather than null.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_create(parent: *mut hb_unicode_funcs_t) -> *mut hb_unicode_funcs_t;

    /// Fetches the singleton empty Unicode-functions structure.
    ///
    /// Its methods are inert stubs: every code point is reported as
    /// [`HB_UNICODE_GENERAL_CATEGORY_OTHER_LETTER`] with combining class
    /// [`HB_UNICODE_COMBINING_CLASS_NOT_REORDERED`] and script
    /// `HB_SCRIPT_UNKNOWN`, mirroring is the identity, and composition and
    /// decomposition always fail. This is what the creation functions fall back
    /// to, so it is never null.
    ///
    /// It participates in reference counting like any other structure and may be
    /// passed to [`hb_unicode_funcs_destroy`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_get_empty() -> *mut hb_unicode_funcs_t;

    /// Increases the reference count on a Unicode-functions structure and
    /// returns it.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_reference(ufuncs: *mut hb_unicode_funcs_t) -> *mut hb_unicode_funcs_t;

    /// Decreases the reference count on a Unicode-functions structure.
    ///
    /// When the count reaches zero the structure is destroyed and all of its
    /// memory freed. Each installed callback's `destroy` notifier is invoked on
    /// its user data first, and the reference held on the parent is released.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_destroy(ufuncs: *mut hb_unicode_funcs_t);

    /// Attaches a user-data key/data pair to a Unicode-functions structure.
    ///
    /// `destroy` may be `None`; when it is not, it is called with `data` once
    /// the structure is destroyed or the entry is replaced. `replace` decides
    /// whether an existing entry under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_set_user_data(
        ufuncs: *mut hb_unicode_funcs_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to a Unicode-functions structure under the
    /// given key.
    ///
    /// The structure retains ownership of the returned pointer; do not free it.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_get_user_data(
        ufuncs: *const hb_unicode_funcs_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Makes a Unicode-functions structure immutable.
    ///
    /// After this call the `hb_unicode_funcs_set_*_func` setters silently do
    /// nothing (they still run the `destroy` notifier on the user data they were
    /// handed). The operation cannot be undone.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_make_immutable(ufuncs: *mut hb_unicode_funcs_t);

    /// Tests whether a Unicode-functions structure is immutable.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_is_immutable(ufuncs: *mut hb_unicode_funcs_t) -> hb_bool_t;

    /// Fetches the parent of a Unicode-functions structure.
    ///
    /// The structure keeps owning the returned reference — this is a borrow, not
    /// a new reference, so do not destroy it unless you first call
    /// [`hb_unicode_funcs_reference`] on it. Structures with no parent report the
    /// singleton empty structure, so the result is never null.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_get_parent(ufuncs: *mut hb_unicode_funcs_t) -> *mut hb_unicode_funcs_t;

    /// Sets the implementation of [`hb_unicode_combining_class_func_t`].
    ///
    /// `user_data` is passed back to `func` on every call, and `destroy` — which
    /// may be `None` — is invoked on it when the structure is destroyed or the
    /// method is replaced. Passing `None` for `func` restores the parent's
    /// implementation. Does nothing if `ufuncs` is immutable, except to run
    /// `destroy` on `user_data`.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_set_combining_class_func(
        ufuncs: *mut hb_unicode_funcs_t,
        func: hb_unicode_combining_class_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_unicode_general_category_func_t`].
    ///
    /// The `user_data`, `destroy`, and immutability rules are the same as for
    /// [`hb_unicode_funcs_set_combining_class_func`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_set_general_category_func(
        ufuncs: *mut hb_unicode_funcs_t,
        func: hb_unicode_general_category_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_unicode_mirroring_func_t`].
    ///
    /// The `user_data`, `destroy`, and immutability rules are the same as for
    /// [`hb_unicode_funcs_set_combining_class_func`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_set_mirroring_func(
        ufuncs: *mut hb_unicode_funcs_t,
        func: hb_unicode_mirroring_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_unicode_script_func_t`].
    ///
    /// The `user_data`, `destroy`, and immutability rules are the same as for
    /// [`hb_unicode_funcs_set_combining_class_func`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_set_script_func(
        ufuncs: *mut hb_unicode_funcs_t,
        func: hb_unicode_script_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_unicode_compose_func_t`].
    ///
    /// The `user_data`, `destroy`, and immutability rules are the same as for
    /// [`hb_unicode_funcs_set_combining_class_func`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_set_compose_func(
        ufuncs: *mut hb_unicode_funcs_t,
        func: hb_unicode_compose_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Sets the implementation of [`hb_unicode_decompose_func_t`].
    ///
    /// The `user_data`, `destroy`, and immutability rules are the same as for
    /// [`hb_unicode_funcs_set_combining_class_func`].
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_funcs_set_decompose_func(
        ufuncs: *mut hb_unicode_funcs_t,
        func: hb_unicode_decompose_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Retrieves the Canonical Combining Class (ccc) property of `unicode`.
    ///
    /// The value may be anything in the range 0..=254, not only the named
    /// [`hb_unicode_combining_class_t`] constants.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_combining_class(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
    ) -> hb_unicode_combining_class_t;

    /// Retrieves the General Category (gc) property of `unicode`.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_general_category(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
    ) -> hb_unicode_general_category_t;

    /// Retrieves the bi-directional mirroring glyph code point defined for
    /// `unicode`.
    ///
    /// Code points with no mirroring glyph come back unchanged.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_mirroring(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
    ) -> hb_codepoint_t;

    /// Retrieves the script to which `unicode` belongs.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_script(
        ufuncs: *mut hb_unicode_funcs_t,
        unicode: hb_codepoint_t,
    ) -> hb_script_t;

    /// Fetches the canonical composition of the code-point pair `a`, `b`,
    /// writing it through `ab`.
    ///
    /// Returns true if the pair composed, false otherwise. `ab` is only
    /// meaningful when the call returns true.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_compose(
        ufuncs: *mut hb_unicode_funcs_t,
        a: hb_codepoint_t,
        b: hb_codepoint_t,
        ab: *mut hb_codepoint_t,
    ) -> hb_bool_t;

    /// Fetches the canonical decomposition of `ab`, writing the two resulting
    /// code points through `a` and `b`.
    ///
    /// Returns true if `ab` decomposed, false otherwise. `a` and `b` are only
    /// meaningful when the call returns true.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_unicode_decompose(
        ufuncs: *mut hb_unicode_funcs_t,
        ab: hb_codepoint_t,
        a: *mut hb_codepoint_t,
        b: *mut hb_codepoint_t,
    ) -> hb_bool_t;
}
