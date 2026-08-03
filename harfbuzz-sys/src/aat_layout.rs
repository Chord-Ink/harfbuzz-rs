//! Apple Advanced Typography layout — `hb-aat-layout.h`.
//!
//! AAT is Apple's alternative to OpenType Layout. These bindings expose the
//! `feat` table's feature types and selectors, plus predicates for the `morx`,
//! `kerx`, and `trak` tables.

use core::ffi::{c_int, c_uint};

use crate::{hb_bool_t, hb_face_t, hb_ot_name_id_t};

/// The feature types defined for AAT shaping.
///
/// A feature type names a family of typographic behaviours a font can offer —
/// ligatures, number spacing, vertical position, and so on. Each one is
/// identified by a small integer taken from Apple's
/// [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html),
/// and the settings within a type are named by
/// [`hb_aat_layout_feature_selector_t`] values.
///
/// The C enumeration's private sentinel is `HB_TAG_MAX_SIGNED` (`0x7FFFFFFF`),
/// which fits an `int`, so this is transcribed as `c_int`. Values come from
/// font data and are not limited to the constants below.
///
/// Since HarfBuzz 2.2.0.
pub type hb_aat_layout_feature_type_t = c_int;

/// Initial, unset feature type.
pub const HB_AAT_LAYOUT_FEATURE_TYPE_INVALID: hb_aat_layout_feature_type_t = 0xFFFF;

/// All Typographic Features — Apple feature type 0.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type0).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_ALL_TYPOGRAPHIC: hb_aat_layout_feature_type_t = 0;

/// Ligatures — Apple feature type 1.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type1).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES: hb_aat_layout_feature_type_t = 1;

/// Cursive Connection — Apple feature type 2.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type2).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_CURSIVE_CONNECTION: hb_aat_layout_feature_type_t = 2;

/// Letter Case — Apple feature type 3.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type3).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_LETTER_CASE: hb_aat_layout_feature_type_t = 3;

/// Vertical Substitution — Apple feature type 4.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type4).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_SUBSTITUTION: hb_aat_layout_feature_type_t = 4;

/// Linguistic Rearrangement — Apple feature type 5.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type5).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_LINGUISTIC_REARRANGEMENT: hb_aat_layout_feature_type_t = 5;

/// Number Spacing — Apple feature type 6.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type6).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_SPACING: hb_aat_layout_feature_type_t = 6;

/// Smart Swash — Apple feature type 8.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type8).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_SMART_SWASH_TYPE: hb_aat_layout_feature_type_t = 8;

/// Diacritics — Apple feature type 9.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type9).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_DIACRITICS_TYPE: hb_aat_layout_feature_type_t = 9;

/// Vertical Position — Apple feature type 10.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type10).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_POSITION: hb_aat_layout_feature_type_t = 10;

/// Fractions — Apple feature type 11.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type11).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_FRACTIONS: hb_aat_layout_feature_type_t = 11;

/// Overlapping Characters — Apple feature type 13.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type13).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_OVERLAPPING_CHARACTERS_TYPE: hb_aat_layout_feature_type_t = 13;

/// Typographic Extras — Apple feature type 14.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type14).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS: hb_aat_layout_feature_type_t = 14;

/// Mathematical Extras — Apple feature type 15.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type15).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS: hb_aat_layout_feature_type_t = 15;

/// Ornament Sets — Apple feature type 16.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type16).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_ORNAMENT_SETS_TYPE: hb_aat_layout_feature_type_t = 16;

/// Character Alternatives — Apple feature type 17.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type17).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_ALTERNATIVES: hb_aat_layout_feature_type_t = 17;

/// Design Complexity — Apple feature type 18.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type18).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_DESIGN_COMPLEXITY_TYPE: hb_aat_layout_feature_type_t = 18;

/// Style Options — Apple feature type 19.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type19).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_STYLE_OPTIONS: hb_aat_layout_feature_type_t = 19;

/// Character Shape — Apple feature type 20.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type20).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE: hb_aat_layout_feature_type_t = 20;

/// Number Case — Apple feature type 21.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type21).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_CASE: hb_aat_layout_feature_type_t = 21;

/// Text Spacing — Apple feature type 22.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type22).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING: hb_aat_layout_feature_type_t = 22;

/// Transliteration — Apple feature type 23.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type23).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_TRANSLITERATION: hb_aat_layout_feature_type_t = 23;

/// Annotation — Apple feature type 24.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type24).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_ANNOTATION_TYPE: hb_aat_layout_feature_type_t = 24;

/// Kana Spacing — Apple feature type 25.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type25).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_KANA_SPACING_TYPE: hb_aat_layout_feature_type_t = 25;

/// Ideographic Spacing — Apple feature type 26.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type26).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_IDEOGRAPHIC_SPACING_TYPE: hb_aat_layout_feature_type_t = 26;

/// Unicode Decomposition — Apple feature type 27.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type27).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_UNICODE_DECOMPOSITION_TYPE: hb_aat_layout_feature_type_t = 27;

/// Ruby Kana — Apple feature type 28.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type28).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_RUBY_KANA: hb_aat_layout_feature_type_t = 28;

/// CJK Symbol Alternatives — Apple feature type 29.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type29).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_CJK_SYMBOL_ALTERNATIVES_TYPE: hb_aat_layout_feature_type_t = 29;

/// Ideographic Alternatives — Apple feature type 30.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type30).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_IDEOGRAPHIC_ALTERNATIVES_TYPE: hb_aat_layout_feature_type_t = 30;

/// CJK Vertical Roman Placement — Apple feature type 31.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type31).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_CJK_VERTICAL_ROMAN_PLACEMENT_TYPE: hb_aat_layout_feature_type_t = 31;

/// Italic CJK Roman — Apple feature type 32.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type32).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_ITALIC_CJK_ROMAN: hb_aat_layout_feature_type_t = 32;

/// Case Sensitive Layout — Apple feature type 33.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type33).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_CASE_SENSITIVE_LAYOUT: hb_aat_layout_feature_type_t = 33;

/// Alternate Kana — Apple feature type 34.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type34).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_ALTERNATE_KANA: hb_aat_layout_feature_type_t = 34;

/// Stylistic Alternatives — Apple feature type 35.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type35).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES: hb_aat_layout_feature_type_t = 35;

/// Contextual Alternatives — Apple feature type 36.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type36).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_CONTEXTUAL_ALTERNATIVES: hb_aat_layout_feature_type_t = 36;

/// Lower Case — Apple feature type 37.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type37).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_LOWER_CASE: hb_aat_layout_feature_type_t = 37;

/// Upper Case — Apple feature type 38.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type38).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_UPPER_CASE: hb_aat_layout_feature_type_t = 38;

/// Language Tag — Apple feature type 39.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type39).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_LANGUAGE_TAG_TYPE: hb_aat_layout_feature_type_t = 39;

/// CJK Roman Spacing — Apple feature type 103.
///
/// See the [Font Feature Registry](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html#Type103).
pub const HB_AAT_LAYOUT_FEATURE_TYPE_CJK_ROMAN_SPACING_TYPE: hb_aat_layout_feature_type_t = 103;

/// The selectors defined for specifying AAT feature settings.
///
/// A selector picks one setting within an [`hb_aat_layout_feature_type_t`].
/// Selector numbers are only meaningful relative to their feature type: `0`
/// means "required ligatures on" under `HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`
/// and "monospaced numbers" under `HB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_SPACING`.
///
/// The C enumeration's private sentinel is `HB_TAG_MAX_SIGNED` (`0x7FFFFFFF`),
/// which fits an `int`, so this is transcribed as `c_int`. Values come from
/// font data and are not limited to the constants below.
///
/// Since HarfBuzz 2.2.0.
pub type hb_aat_layout_feature_selector_t = c_int;

/// Initial, unset feature selector.
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_INVALID: hb_aat_layout_feature_selector_t = 0xFFFF;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_ALL_TYPOGRAPHIC.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ALL_TYPOGRAPHIC`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ALL_TYPE_FEATURES_ON: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ALL_TYPOGRAPHIC`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ALL_TYPE_FEATURES_OFF: hb_aat_layout_feature_selector_t = 1;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_REQUIRED_LIGATURES_ON: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_REQUIRED_LIGATURES_OFF: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_COMMON_LIGATURES_ON: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_COMMON_LIGATURES_OFF: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_RARE_LIGATURES_ON: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_RARE_LIGATURES_OFF: hb_aat_layout_feature_selector_t = 5;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_LOGOS_ON: hb_aat_layout_feature_selector_t = 6;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_LOGOS_OFF: hb_aat_layout_feature_selector_t = 7;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_REBUS_PICTURES_ON: hb_aat_layout_feature_selector_t = 8;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_REBUS_PICTURES_OFF: hb_aat_layout_feature_selector_t = 9;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DIPHTHONG_LIGATURES_ON: hb_aat_layout_feature_selector_t = 10;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DIPHTHONG_LIGATURES_OFF: hb_aat_layout_feature_selector_t = 11;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SQUARED_LIGATURES_ON: hb_aat_layout_feature_selector_t = 12;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SQUARED_LIGATURES_OFF: hb_aat_layout_feature_selector_t = 13;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ABBREV_SQUARED_LIGATURES_ON: hb_aat_layout_feature_selector_t = 14;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ABBREV_SQUARED_LIGATURES_OFF: hb_aat_layout_feature_selector_t = 15;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SYMBOL_LIGATURES_ON: hb_aat_layout_feature_selector_t = 16;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SYMBOL_LIGATURES_OFF: hb_aat_layout_feature_selector_t = 17;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CONTEXTUAL_LIGATURES_ON: hb_aat_layout_feature_selector_t = 18;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CONTEXTUAL_LIGATURES_OFF: hb_aat_layout_feature_selector_t = 19;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HISTORICAL_LIGATURES_ON: hb_aat_layout_feature_selector_t = 20;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HISTORICAL_LIGATURES_OFF: hb_aat_layout_feature_selector_t = 21;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_CURSIVE_CONNECTION. The C header
// mislabels this group as HB_AAT_LAYOUT_FEATURE_TYPE_LIGATURES.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CURSIVE_CONNECTION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_UNCONNECTED: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CURSIVE_CONNECTION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_PARTIALLY_CONNECTED: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CURSIVE_CONNECTION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CURSIVE: hb_aat_layout_feature_selector_t = 2;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_LETTER_CASE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LETTER_CASE`].
///
/// Deprecated.
#[deprecated(note = "deprecated in the C header")]
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_UPPER_AND_LOWER_CASE: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LETTER_CASE`].
///
/// Deprecated.
#[deprecated(note = "deprecated in the C header")]
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ALL_CAPS: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LETTER_CASE`].
///
/// Deprecated.
#[deprecated(note = "deprecated in the C header")]
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ALL_LOWER_CASE: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LETTER_CASE`].
///
/// Deprecated.
#[deprecated(note = "deprecated in the C header")]
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SMALL_CAPS: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LETTER_CASE`].
///
/// Deprecated.
#[deprecated(note = "deprecated in the C header")]
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_INITIAL_CAPS: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LETTER_CASE`].
///
/// Deprecated.
#[deprecated(note = "deprecated in the C header")]
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_INITIAL_CAPS_AND_SMALL_CAPS: hb_aat_layout_feature_selector_t = 5;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_SUBSTITUTION.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_SUBSTITUTION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SUBSTITUTE_VERTICAL_FORMS_ON: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_SUBSTITUTION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SUBSTITUTE_VERTICAL_FORMS_OFF: hb_aat_layout_feature_selector_t = 1;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_LINGUISTIC_REARRANGEMENT.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LINGUISTIC_REARRANGEMENT`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_LINGUISTIC_REARRANGEMENT_ON: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LINGUISTIC_REARRANGEMENT`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_LINGUISTIC_REARRANGEMENT_OFF: hb_aat_layout_feature_selector_t = 1;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_SPACING.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_SPACING`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_MONOSPACED_NUMBERS: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_SPACING`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_PROPORTIONAL_NUMBERS: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_SPACING`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_THIRD_WIDTH_NUMBERS: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_SPACING`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_QUARTER_WIDTH_NUMBERS: hb_aat_layout_feature_selector_t = 3;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_SMART_SWASH_TYPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_SMART_SWASH_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_WORD_INITIAL_SWASHES_ON: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_SMART_SWASH_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_WORD_INITIAL_SWASHES_OFF: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_SMART_SWASH_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_WORD_FINAL_SWASHES_ON: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_SMART_SWASH_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_WORD_FINAL_SWASHES_OFF: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_SMART_SWASH_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_LINE_INITIAL_SWASHES_ON: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_SMART_SWASH_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_LINE_INITIAL_SWASHES_OFF: hb_aat_layout_feature_selector_t = 5;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_SMART_SWASH_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_LINE_FINAL_SWASHES_ON: hb_aat_layout_feature_selector_t = 6;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_SMART_SWASH_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_LINE_FINAL_SWASHES_OFF: hb_aat_layout_feature_selector_t = 7;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_SMART_SWASH_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NON_FINAL_SWASHES_ON: hb_aat_layout_feature_selector_t = 8;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_SMART_SWASH_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NON_FINAL_SWASHES_OFF: hb_aat_layout_feature_selector_t = 9;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_DIACRITICS_TYPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_DIACRITICS_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SHOW_DIACRITICS: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_DIACRITICS_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HIDE_DIACRITICS: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_DIACRITICS_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DECOMPOSE_DIACRITICS: hb_aat_layout_feature_selector_t = 2;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_POSITION.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_POSITION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NORMAL_POSITION: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_POSITION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SUPERIORS: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_POSITION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_INFERIORS: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_POSITION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ORDINALS: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_VERTICAL_POSITION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SCIENTIFIC_INFERIORS: hb_aat_layout_feature_selector_t = 4;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_FRACTIONS.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_FRACTIONS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NO_FRACTIONS: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_FRACTIONS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_VERTICAL_FRACTIONS: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_FRACTIONS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DIAGONAL_FRACTIONS: hb_aat_layout_feature_selector_t = 2;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_OVERLAPPING_CHARACTERS_TYPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_OVERLAPPING_CHARACTERS_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_PREVENT_OVERLAP_ON: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_OVERLAPPING_CHARACTERS_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_PREVENT_OVERLAP_OFF: hb_aat_layout_feature_selector_t = 1;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HYPHENS_TO_EM_DASH_ON: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HYPHENS_TO_EM_DASH_OFF: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HYPHEN_TO_EN_DASH_ON: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HYPHEN_TO_EN_DASH_OFF: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SLASHED_ZERO_ON: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SLASHED_ZERO_OFF: hb_aat_layout_feature_selector_t = 5;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_FORM_INTERROBANG_ON: hb_aat_layout_feature_selector_t = 6;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_FORM_INTERROBANG_OFF: hb_aat_layout_feature_selector_t = 7;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SMART_QUOTES_ON: hb_aat_layout_feature_selector_t = 8;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SMART_QUOTES_OFF: hb_aat_layout_feature_selector_t = 9;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_PERIODS_TO_ELLIPSIS_ON: hb_aat_layout_feature_selector_t = 10;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TYPOGRAPHIC_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_PERIODS_TO_ELLIPSIS_OFF: hb_aat_layout_feature_selector_t = 11;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HYPHEN_TO_MINUS_ON: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HYPHEN_TO_MINUS_OFF: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ASTERISK_TO_MULTIPLY_ON: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ASTERISK_TO_MULTIPLY_OFF: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SLASH_TO_DIVIDE_ON: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SLASH_TO_DIVIDE_OFF: hb_aat_layout_feature_selector_t = 5;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_INEQUALITY_LIGATURES_ON: hb_aat_layout_feature_selector_t = 6;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_INEQUALITY_LIGATURES_OFF: hb_aat_layout_feature_selector_t = 7;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_EXPONENTS_ON: hb_aat_layout_feature_selector_t = 8;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_EXPONENTS_OFF: hb_aat_layout_feature_selector_t = 9;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_MATHEMATICAL_GREEK_ON: hb_aat_layout_feature_selector_t = 10;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_MATHEMATICAL_EXTRAS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_MATHEMATICAL_GREEK_OFF: hb_aat_layout_feature_selector_t = 11;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_ORNAMENT_SETS_TYPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ORNAMENT_SETS_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NO_ORNAMENTS: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ORNAMENT_SETS_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DINGBATS: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ORNAMENT_SETS_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_PI_CHARACTERS: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ORNAMENT_SETS_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_FLEURONS: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ORNAMENT_SETS_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DECORATIVE_BORDERS: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ORNAMENT_SETS_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_INTERNATIONAL_SYMBOLS: hb_aat_layout_feature_selector_t = 5;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ORNAMENT_SETS_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_MATH_SYMBOLS: hb_aat_layout_feature_selector_t = 6;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_ALTERNATIVES.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NO_ALTERNATES: hb_aat_layout_feature_selector_t = 0;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_DESIGN_COMPLEXITY_TYPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_DESIGN_COMPLEXITY_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DESIGN_LEVEL1: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_DESIGN_COMPLEXITY_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DESIGN_LEVEL2: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_DESIGN_COMPLEXITY_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DESIGN_LEVEL3: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_DESIGN_COMPLEXITY_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DESIGN_LEVEL4: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_DESIGN_COMPLEXITY_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DESIGN_LEVEL5: hb_aat_layout_feature_selector_t = 4;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_STYLE_OPTIONS.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLE_OPTIONS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NO_STYLE_OPTIONS: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLE_OPTIONS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DISPLAY_TEXT: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLE_OPTIONS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ENGRAVED_TEXT: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLE_OPTIONS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ILLUMINATED_CAPS: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLE_OPTIONS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_TITLING_CAPS: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLE_OPTIONS`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_TALL_CAPS: hb_aat_layout_feature_selector_t = 5;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_TRADITIONAL_CHARACTERS: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SIMPLIFIED_CHARACTERS: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_JIS1978_CHARACTERS: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_JIS1983_CHARACTERS: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_JIS1990_CHARACTERS: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_TRADITIONAL_ALT_ONE: hb_aat_layout_feature_selector_t = 5;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_TRADITIONAL_ALT_TWO: hb_aat_layout_feature_selector_t = 6;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_TRADITIONAL_ALT_THREE: hb_aat_layout_feature_selector_t = 7;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_TRADITIONAL_ALT_FOUR: hb_aat_layout_feature_selector_t = 8;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_TRADITIONAL_ALT_FIVE: hb_aat_layout_feature_selector_t = 9;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_EXPERT_CHARACTERS: hb_aat_layout_feature_selector_t = 10;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_JIS2004_CHARACTERS: hb_aat_layout_feature_selector_t = 11;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HOJO_CHARACTERS: hb_aat_layout_feature_selector_t = 12;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NLCCHARACTERS: hb_aat_layout_feature_selector_t = 13;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CHARACTER_SHAPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_TRADITIONAL_NAMES_CHARACTERS: hb_aat_layout_feature_selector_t = 14;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_CASE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_CASE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_LOWER_CASE_NUMBERS: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_NUMBER_CASE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_UPPER_CASE_NUMBERS: hb_aat_layout_feature_selector_t = 1;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_PROPORTIONAL_TEXT: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_MONOSPACED_TEXT: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HALF_WIDTH_TEXT: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_THIRD_WIDTH_TEXT: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_QUARTER_WIDTH_TEXT: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ALT_PROPORTIONAL_TEXT: hb_aat_layout_feature_selector_t = 5;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TEXT_SPACING`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ALT_HALF_WIDTH_TEXT: hb_aat_layout_feature_selector_t = 6;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_TRANSLITERATION.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TRANSLITERATION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NO_TRANSLITERATION: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TRANSLITERATION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HANJA_TO_HANGUL: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TRANSLITERATION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HIRAGANA_TO_KATAKANA: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TRANSLITERATION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_KATAKANA_TO_HIRAGANA: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TRANSLITERATION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_KANA_TO_ROMANIZATION: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TRANSLITERATION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ROMANIZATION_TO_HIRAGANA: hb_aat_layout_feature_selector_t = 5;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TRANSLITERATION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ROMANIZATION_TO_KATAKANA: hb_aat_layout_feature_selector_t = 6;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TRANSLITERATION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HANJA_TO_HANGUL_ALT_ONE: hb_aat_layout_feature_selector_t = 7;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TRANSLITERATION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HANJA_TO_HANGUL_ALT_TWO: hb_aat_layout_feature_selector_t = 8;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_TRANSLITERATION`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HANJA_TO_HANGUL_ALT_THREE: hb_aat_layout_feature_selector_t = 9;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_ANNOTATION_TYPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ANNOTATION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NO_ANNOTATION: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ANNOTATION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_BOX_ANNOTATION: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ANNOTATION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ROUNDED_BOX_ANNOTATION: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ANNOTATION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CIRCLE_ANNOTATION: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ANNOTATION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_INVERTED_CIRCLE_ANNOTATION: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ANNOTATION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_PARENTHESIS_ANNOTATION: hb_aat_layout_feature_selector_t = 5;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ANNOTATION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_PERIOD_ANNOTATION: hb_aat_layout_feature_selector_t = 6;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ANNOTATION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ROMAN_NUMERAL_ANNOTATION: hb_aat_layout_feature_selector_t = 7;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ANNOTATION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DIAMOND_ANNOTATION: hb_aat_layout_feature_selector_t = 8;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ANNOTATION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_INVERTED_BOX_ANNOTATION: hb_aat_layout_feature_selector_t = 9;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ANNOTATION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_INVERTED_ROUNDED_BOX_ANNOTATION: hb_aat_layout_feature_selector_t = 10;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_KANA_SPACING_TYPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_KANA_SPACING_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_FULL_WIDTH_KANA: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_KANA_SPACING_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_PROPORTIONAL_KANA: hb_aat_layout_feature_selector_t = 1;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_IDEOGRAPHIC_SPACING_TYPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_IDEOGRAPHIC_SPACING_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_FULL_WIDTH_IDEOGRAPHS: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_IDEOGRAPHIC_SPACING_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_PROPORTIONAL_IDEOGRAPHS: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_IDEOGRAPHIC_SPACING_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HALF_WIDTH_IDEOGRAPHS: hb_aat_layout_feature_selector_t = 2;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_UNICODE_DECOMPOSITION_TYPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_UNICODE_DECOMPOSITION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CANONICAL_COMPOSITION_ON: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_UNICODE_DECOMPOSITION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CANONICAL_COMPOSITION_OFF: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_UNICODE_DECOMPOSITION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_COMPATIBILITY_COMPOSITION_ON: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_UNICODE_DECOMPOSITION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_COMPATIBILITY_COMPOSITION_OFF: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_UNICODE_DECOMPOSITION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_TRANSCODING_COMPOSITION_ON: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_UNICODE_DECOMPOSITION_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_TRANSCODING_COMPOSITION_OFF: hb_aat_layout_feature_selector_t = 5;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_RUBY_KANA.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_RUBY_KANA`].
///
/// Deprecated; use [`HB_AAT_LAYOUT_FEATURE_SELECTOR_RUBY_KANA_OFF`] instead.
#[deprecated(note = "use `HB_AAT_LAYOUT_FEATURE_SELECTOR_RUBY_KANA_OFF` instead")]
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NO_RUBY_KANA: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_RUBY_KANA`].
///
/// Deprecated; use [`HB_AAT_LAYOUT_FEATURE_SELECTOR_RUBY_KANA_ON`] instead.
#[deprecated(note = "use `HB_AAT_LAYOUT_FEATURE_SELECTOR_RUBY_KANA_ON` instead")]
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_RUBY_KANA: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_RUBY_KANA`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_RUBY_KANA_ON: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_RUBY_KANA`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_RUBY_KANA_OFF: hb_aat_layout_feature_selector_t = 3;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_CJK_SYMBOL_ALTERNATIVES_TYPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CJK_SYMBOL_ALTERNATIVES_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NO_CJK_SYMBOL_ALTERNATIVES: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CJK_SYMBOL_ALTERNATIVES_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_SYMBOL_ALT_ONE: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CJK_SYMBOL_ALTERNATIVES_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_SYMBOL_ALT_TWO: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CJK_SYMBOL_ALTERNATIVES_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_SYMBOL_ALT_THREE: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CJK_SYMBOL_ALTERNATIVES_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_SYMBOL_ALT_FOUR: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CJK_SYMBOL_ALTERNATIVES_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_SYMBOL_ALT_FIVE: hb_aat_layout_feature_selector_t = 5;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_IDEOGRAPHIC_ALTERNATIVES_TYPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_IDEOGRAPHIC_ALTERNATIVES_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NO_IDEOGRAPHIC_ALTERNATIVES: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_IDEOGRAPHIC_ALTERNATIVES_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_IDEOGRAPHIC_ALT_ONE: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_IDEOGRAPHIC_ALTERNATIVES_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_IDEOGRAPHIC_ALT_TWO: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_IDEOGRAPHIC_ALTERNATIVES_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_IDEOGRAPHIC_ALT_THREE: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_IDEOGRAPHIC_ALTERNATIVES_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_IDEOGRAPHIC_ALT_FOUR: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_IDEOGRAPHIC_ALTERNATIVES_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_IDEOGRAPHIC_ALT_FIVE: hb_aat_layout_feature_selector_t = 5;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_CJK_VERTICAL_ROMAN_PLACEMENT_TYPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CJK_VERTICAL_ROMAN_PLACEMENT_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_VERTICAL_ROMAN_CENTERED: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CJK_VERTICAL_ROMAN_PLACEMENT_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_VERTICAL_ROMAN_HBASELINE: hb_aat_layout_feature_selector_t = 1;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_ITALIC_CJK_ROMAN.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ITALIC_CJK_ROMAN`].
///
/// Deprecated; use [`HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_ITALIC_ROMAN_OFF`] instead.
#[deprecated(note = "use `HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_ITALIC_ROMAN_OFF` instead")]
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NO_CJK_ITALIC_ROMAN: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ITALIC_CJK_ROMAN`].
///
/// Deprecated; use [`HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_ITALIC_ROMAN_ON`] instead.
#[deprecated(note = "use `HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_ITALIC_ROMAN_ON` instead")]
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_ITALIC_ROMAN: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ITALIC_CJK_ROMAN`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_ITALIC_ROMAN_ON: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ITALIC_CJK_ROMAN`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CJK_ITALIC_ROMAN_OFF: hb_aat_layout_feature_selector_t = 3;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_CASE_SENSITIVE_LAYOUT.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CASE_SENSITIVE_LAYOUT`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CASE_SENSITIVE_LAYOUT_ON: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CASE_SENSITIVE_LAYOUT`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CASE_SENSITIVE_LAYOUT_OFF: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CASE_SENSITIVE_LAYOUT`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CASE_SENSITIVE_SPACING_ON: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CASE_SENSITIVE_LAYOUT`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CASE_SENSITIVE_SPACING_OFF: hb_aat_layout_feature_selector_t = 3;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_ALTERNATE_KANA.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ALTERNATE_KANA`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ALTERNATE_HORIZ_KANA_ON: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ALTERNATE_KANA`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ALTERNATE_HORIZ_KANA_OFF: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ALTERNATE_KANA`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ALTERNATE_VERT_KANA_ON: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_ALTERNATE_KANA`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_ALTERNATE_VERT_KANA_OFF: hb_aat_layout_feature_selector_t = 3;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_NO_STYLISTIC_ALTERNATES: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_ONE_ON: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_ONE_OFF: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TWO_ON: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TWO_OFF: hb_aat_layout_feature_selector_t = 5;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_THREE_ON: hb_aat_layout_feature_selector_t = 6;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_THREE_OFF: hb_aat_layout_feature_selector_t = 7;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FOUR_ON: hb_aat_layout_feature_selector_t = 8;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FOUR_OFF: hb_aat_layout_feature_selector_t = 9;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FIVE_ON: hb_aat_layout_feature_selector_t = 10;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FIVE_OFF: hb_aat_layout_feature_selector_t = 11;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SIX_ON: hb_aat_layout_feature_selector_t = 12;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SIX_OFF: hb_aat_layout_feature_selector_t = 13;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SEVEN_ON: hb_aat_layout_feature_selector_t = 14;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SEVEN_OFF: hb_aat_layout_feature_selector_t = 15;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_EIGHT_ON: hb_aat_layout_feature_selector_t = 16;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_EIGHT_OFF: hb_aat_layout_feature_selector_t = 17;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_NINE_ON: hb_aat_layout_feature_selector_t = 18;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_NINE_OFF: hb_aat_layout_feature_selector_t = 19;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TEN_ON: hb_aat_layout_feature_selector_t = 20;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TEN_OFF: hb_aat_layout_feature_selector_t = 21;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_ELEVEN_ON: hb_aat_layout_feature_selector_t = 22;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_ELEVEN_OFF: hb_aat_layout_feature_selector_t = 23;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TWELVE_ON: hb_aat_layout_feature_selector_t = 24;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TWELVE_OFF: hb_aat_layout_feature_selector_t = 25;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_THIRTEEN_ON: hb_aat_layout_feature_selector_t = 26;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_THIRTEEN_OFF: hb_aat_layout_feature_selector_t = 27;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FOURTEEN_ON: hb_aat_layout_feature_selector_t = 28;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FOURTEEN_OFF: hb_aat_layout_feature_selector_t = 29;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FIFTEEN_ON: hb_aat_layout_feature_selector_t = 30;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_FIFTEEN_OFF: hb_aat_layout_feature_selector_t = 31;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SIXTEEN_ON: hb_aat_layout_feature_selector_t = 32;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SIXTEEN_OFF: hb_aat_layout_feature_selector_t = 33;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SEVENTEEN_ON: hb_aat_layout_feature_selector_t = 34;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_SEVENTEEN_OFF: hb_aat_layout_feature_selector_t = 35;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_EIGHTEEN_ON: hb_aat_layout_feature_selector_t = 36;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_EIGHTEEN_OFF: hb_aat_layout_feature_selector_t = 37;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_NINETEEN_ON: hb_aat_layout_feature_selector_t = 38;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_NINETEEN_OFF: hb_aat_layout_feature_selector_t = 39;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TWENTY_ON: hb_aat_layout_feature_selector_t = 40;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_STYLISTIC_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_STYLISTIC_ALT_TWENTY_OFF: hb_aat_layout_feature_selector_t = 41;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_CONTEXTUAL_ALTERNATIVES.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CONTEXTUAL_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CONTEXTUAL_ALTERNATES_ON: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CONTEXTUAL_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CONTEXTUAL_ALTERNATES_OFF: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CONTEXTUAL_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SWASH_ALTERNATES_ON: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CONTEXTUAL_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_SWASH_ALTERNATES_OFF: hb_aat_layout_feature_selector_t = 3;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CONTEXTUAL_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CONTEXTUAL_SWASH_ALTERNATES_ON: hb_aat_layout_feature_selector_t = 4;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CONTEXTUAL_ALTERNATIVES`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_CONTEXTUAL_SWASH_ALTERNATES_OFF: hb_aat_layout_feature_selector_t = 5;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_LOWER_CASE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LOWER_CASE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DEFAULT_LOWER_CASE: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LOWER_CASE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_LOWER_CASE_SMALL_CAPS: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_LOWER_CASE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_LOWER_CASE_PETITE_CAPS: hb_aat_layout_feature_selector_t = 2;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_UPPER_CASE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_UPPER_CASE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DEFAULT_UPPER_CASE: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_UPPER_CASE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_UPPER_CASE_SMALL_CAPS: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_UPPER_CASE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_UPPER_CASE_PETITE_CAPS: hb_aat_layout_feature_selector_t = 2;

// Selectors for HB_AAT_LAYOUT_FEATURE_TYPE_CJK_ROMAN_SPACING_TYPE.
/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CJK_ROMAN_SPACING_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_HALF_WIDTH_CJK_ROMAN: hb_aat_layout_feature_selector_t = 0;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CJK_ROMAN_SPACING_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_PROPORTIONAL_CJK_ROMAN: hb_aat_layout_feature_selector_t = 1;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CJK_ROMAN_SPACING_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_DEFAULT_CJK_ROMAN: hb_aat_layout_feature_selector_t = 2;

/// Selector for [`HB_AAT_LAYOUT_FEATURE_TYPE_CJK_ROMAN_SPACING_TYPE`].
pub const HB_AAT_LAYOUT_FEATURE_SELECTOR_FULL_WIDTH_CJK_ROMAN: hb_aat_layout_feature_selector_t = 3;

/// A setting for an [`hb_aat_layout_feature_type_t`].
///
/// Filled in by [`hb_aat_layout_feature_type_get_selector_infos`]. Each record
/// describes one selectable setting of a feature type: the `name` table entry
/// that labels it, the selector value that turns it on, and the selector value
/// that turns it off again.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_aat_layout_feature_selector_info_t {
    /// The selector's name identifier, for lookup in the face's `name` table.
    pub name_id: hb_ot_name_id_t,

    /// The value to turn the selector on.
    pub enable: hb_aat_layout_feature_selector_t,

    /// The value to turn the selector off.
    ///
    /// For a non-exclusive feature type this is `enable + 1`, following AAT's
    /// convention that settings come in on/off pairs. For an exclusive feature
    /// type there is no "off" — turning one setting off means selecting
    /// another — so HarfBuzz reports the feature's default selector here.
    pub disable: hb_aat_layout_feature_selector_t,

    /// Private padding. Set to zero by HarfBuzz; do not read or write it.
    pub reserved: c_uint,
}

/// No selector index corresponds to the selector of interest.
///
/// Used when getting or setting AAT feature selectors.
/// [`hb_aat_layout_feature_type_get_selector_infos`] reports this as the
/// default index of a non-exclusive feature type, which has no single default.
pub const HB_AAT_LAYOUT_NO_SELECTOR_INDEX: c_uint = 0xFFFF;

unsafe extern "C" {
    /// Fetches a list of the AAT feature types included in `face`.
    ///
    /// `start_offset` is the index of the first feature type to retrieve. On
    /// entry `feature_count` holds the capacity of `features`; on return it
    /// holds how many entries were actually written, which may be zero. Both
    /// `feature_count` and `features` may be null, in which case nothing is
    /// written and only the total is reported.
    ///
    /// Returns the number of feature types available in total, independent of
    /// `start_offset` and `feature_count`.
    ///
    /// Since HarfBuzz 2.2.0.
    pub fn hb_aat_layout_get_feature_types(
        face: *mut hb_face_t,
        start_offset: c_uint,
        feature_count: *mut c_uint,
        features: *mut hb_aat_layout_feature_type_t,
    ) -> c_uint;

    /// Fetches the name identifier of `feature_type` in the face's `name`
    /// table.
    ///
    /// Since HarfBuzz 2.2.0.
    pub fn hb_aat_layout_feature_type_get_name_id(
        face: *mut hb_face_t,
        feature_type: hb_aat_layout_feature_type_t,
    ) -> hb_ot_name_id_t;

    /// Fetches a list of the selectors available for `feature_type` in `face`.
    ///
    /// `start_offset` is the index of the first selector to retrieve. On entry
    /// `selector_count` holds the capacity of `selectors`; on return it holds
    /// how many entries were actually written, which may be zero. Both
    /// `selector_count` and `selectors` may be null, in which case nothing is
    /// written and only the total is reported.
    ///
    /// When `default_index` is non-null it receives the index of the feature's
    /// default selector, or [`HB_AAT_LAYOUT_NO_SELECTOR_INDEX`] if the feature
    /// type is non-exclusive.
    ///
    /// Returns the number of selectors available in total, independent of
    /// `start_offset` and `selector_count`.
    ///
    /// Since HarfBuzz 2.2.0.
    pub fn hb_aat_layout_feature_type_get_selector_infos(
        face: *mut hb_face_t,
        feature_type: hb_aat_layout_feature_type_t,
        start_offset: c_uint,
        selector_count: *mut c_uint,
        selectors: *mut hb_aat_layout_feature_selector_info_t,
        default_index: *mut c_uint,
    ) -> c_uint;

    /// Tests whether `face` includes any substitutions in the `morx` or `mort`
    /// tables.
    ///
    /// Does not examine the `GSUB` table.
    ///
    /// Since HarfBuzz 2.3.0.
    pub fn hb_aat_layout_has_substitution(face: *mut hb_face_t) -> hb_bool_t;

    /// Tests whether `face` includes any positioning information in the `kerx`
    /// table.
    ///
    /// Does not examine the `GPOS` table.
    ///
    /// Since HarfBuzz 2.3.0.
    pub fn hb_aat_layout_has_positioning(face: *mut hb_face_t) -> hb_bool_t;

    /// Tests whether `face` includes any tracking information in the `trak`
    /// table.
    ///
    /// Since HarfBuzz 2.3.0.
    pub fn hb_aat_layout_has_tracking(face: *mut hb_face_t) -> hb_bool_t;
}
