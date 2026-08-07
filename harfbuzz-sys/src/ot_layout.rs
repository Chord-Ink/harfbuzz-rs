//! Querying OpenType Layout tables — `GDEF`, `GSUB`, `GPOS`, `BASE` —
//! `hb-ot-layout.h`.
//!
//! These functions inspect what a face's layout tables *contain*; shaping
//! itself is driven from `hb-shape.h`.

use core::ffi::{c_int, c_uint};

use crate::{
    HB_TAG, hb_bool_t, hb_codepoint_t, hb_direction_t, hb_face_t, hb_font_extents_t, hb_font_t,
    hb_language_t, hb_map_t, hb_ot_name_id_t, hb_position_t, hb_script_t, hb_set_t, hb_tag_t,
};

/// OpenType [Baseline Table](https://docs.microsoft.com/en-us/typography/opentype/spec/base)
/// tag, `BASE`.
pub const HB_OT_TAG_BASE: hb_tag_t = HB_TAG(b'B', b'A', b'S', b'E');

/// OpenType [Glyph Definition Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gdef)
/// tag, `GDEF`.
pub const HB_OT_TAG_GDEF: hb_tag_t = HB_TAG(b'G', b'D', b'E', b'F');

/// OpenType [Glyph Substitution Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gsub)
/// tag, `GSUB`.
pub const HB_OT_TAG_GSUB: hb_tag_t = HB_TAG(b'G', b'S', b'U', b'B');

/// OpenType [Glyph Positioning Table](https://docs.microsoft.com/en-us/typography/opentype/spec/gpos)
/// tag, `GPOS`.
pub const HB_OT_TAG_GPOS: hb_tag_t = HB_TAG(b'G', b'P', b'O', b'S');

/// OpenType [Justification Table](https://docs.microsoft.com/en-us/typography/opentype/spec/jstf)
/// tag, `JSTF`.
pub const HB_OT_TAG_JSTF: hb_tag_t = HB_TAG(b'J', b'S', b'T', b'F');

/// OpenType script tag, `DFLT`, for features that are not script-specific.
pub const HB_OT_TAG_DEFAULT_SCRIPT: hb_tag_t = HB_TAG(b'D', b'F', b'L', b'T');

/// OpenType language tag, `dflt`.
///
/// Not a valid language tag, but some fonts mistakenly use it.
pub const HB_OT_TAG_DEFAULT_LANGUAGE: hb_tag_t = HB_TAG(b'd', b'f', b'l', b't');

/// Maximum number of OpenType tags that can correspond to a given
/// [`hb_script_t`].
///
/// Since HarfBuzz 2.0.0.
pub const HB_OT_MAX_TAGS_PER_SCRIPT: c_uint = 3;

/// Maximum number of OpenType tags that can correspond to a given
/// [`hb_language_t`].
///
/// Since HarfBuzz 2.0.0.
pub const HB_OT_MAX_TAGS_PER_LANGUAGE: c_uint = 3;

/// Special value for a script index indicating an unsupported script.
pub const HB_OT_LAYOUT_NO_SCRIPT_INDEX: c_uint = 0xFFFF;

/// Special value for a feature index indicating an unsupported feature.
pub const HB_OT_LAYOUT_NO_FEATURE_INDEX: c_uint = 0xFFFF;

/// Special value for a language index indicating the default or an unsupported
/// language.
pub const HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX: c_uint = 0xFFFF;

/// Special value for a variations index indicating an unsupported variation.
pub const HB_OT_LAYOUT_NO_VARIATIONS_INDEX: c_uint = 0xFFFF_FFFF;

/// The glyph classes defined by the `GDEF` table.
///
/// The C enumeration has no explicit sentinel and its largest enumerator is 4,
/// so it fits in an `int`.
pub type hb_ot_layout_glyph_class_t = c_int;

/// Glyphs not matching the other classifications.
pub const HB_OT_LAYOUT_GLYPH_CLASS_UNCLASSIFIED: hb_ot_layout_glyph_class_t = 0;

/// Spacing, single characters, capable of accepting marks.
pub const HB_OT_LAYOUT_GLYPH_CLASS_BASE_GLYPH: hb_ot_layout_glyph_class_t = 1;

/// Glyphs that represent the ligation of multiple characters.
pub const HB_OT_LAYOUT_GLYPH_CLASS_LIGATURE: hb_ot_layout_glyph_class_t = 2;

/// Non-spacing, combining glyphs that represent marks.
pub const HB_OT_LAYOUT_GLYPH_CLASS_MARK: hb_ot_layout_glyph_class_t = 3;

/// Spacing glyphs that represent part of a single character.
pub const HB_OT_LAYOUT_GLYPH_CLASS_COMPONENT: hb_ot_layout_glyph_class_t = 4;

/// Baseline tags from the
/// [Baseline Tags](https://docs.microsoft.com/en-us/typography/opentype/spec/baselinetags)
/// registry.
///
/// Every value is a four-byte OpenType tag. The C enumeration ends with a
/// `_HB_OT_LAYOUT_BASELINE_TAG_MAX_VALUE = HB_TAG_MAX_SIGNED` sentinel, which
/// fits in an `int`.
///
/// Since HarfBuzz 2.6.0.
pub type hb_ot_layout_baseline_tag_t = c_int;

/// The baseline used by alphabetic scripts such as Latin, Cyrillic and Greek —
/// `romn`.
///
/// In vertical writing mode this is the alphabetic baseline for characters
/// rotated 90 degrees clockwise. (It does not apply to alphabetic characters
/// that remain upright in vertical writing mode, since those are not rotated.)
pub const HB_OT_LAYOUT_BASELINE_TAG_ROMAN: hb_ot_layout_baseline_tag_t =
    HB_TAG(b'r', b'o', b'm', b'n') as hb_ot_layout_baseline_tag_t;

/// The hanging baseline — `hang`.
///
/// In the horizontal direction this is the line from which syllables seem to
/// hang in Tibetan and other similar scripts. In vertical writing mode it
/// applies to such characters rotated 90 degrees clockwise.
pub const HB_OT_LAYOUT_BASELINE_TAG_HANGING: hb_ot_layout_baseline_tag_t =
    HB_TAG(b'h', b'a', b'n', b'g') as hb_ot_layout_baseline_tag_t;

/// Ideographic character face bottom or left edge — `icfb` — depending on
/// whether the direction is horizontal or vertical, respectively.
pub const HB_OT_LAYOUT_BASELINE_TAG_IDEO_FACE_BOTTOM_OR_LEFT: hb_ot_layout_baseline_tag_t =
    HB_TAG(b'i', b'c', b'f', b'b') as hb_ot_layout_baseline_tag_t;

/// Ideographic character face top or right edge — `icft` — depending on whether
/// the direction is horizontal or vertical, respectively.
pub const HB_OT_LAYOUT_BASELINE_TAG_IDEO_FACE_TOP_OR_RIGHT: hb_ot_layout_baseline_tag_t =
    HB_TAG(b'i', b'c', b'f', b't') as hb_ot_layout_baseline_tag_t;

/// The centre of the ideographic character face — `Icfc`.
///
/// Since HarfBuzz 4.0.0.
pub const HB_OT_LAYOUT_BASELINE_TAG_IDEO_FACE_CENTRAL: hb_ot_layout_baseline_tag_t =
    HB_TAG(b'I', b'c', b'f', b'c') as hb_ot_layout_baseline_tag_t;

/// Ideographic em-box bottom or left edge — `ideo` — depending on whether the
/// direction is horizontal or vertical, respectively.
pub const HB_OT_LAYOUT_BASELINE_TAG_IDEO_EMBOX_BOTTOM_OR_LEFT: hb_ot_layout_baseline_tag_t =
    HB_TAG(b'i', b'd', b'e', b'o') as hb_ot_layout_baseline_tag_t;

/// Ideographic em-box top or right edge — `idtp` — depending on whether the
/// direction is horizontal or vertical, respectively.
pub const HB_OT_LAYOUT_BASELINE_TAG_IDEO_EMBOX_TOP_OR_RIGHT: hb_ot_layout_baseline_tag_t =
    HB_TAG(b'i', b'd', b't', b'p') as hb_ot_layout_baseline_tag_t;

/// The centre of the ideographic em-box — `Idce`.
///
/// Since HarfBuzz 4.0.0.
pub const HB_OT_LAYOUT_BASELINE_TAG_IDEO_EMBOX_CENTRAL: hb_ot_layout_baseline_tag_t =
    HB_TAG(b'I', b'd', b'c', b'e') as hb_ot_layout_baseline_tag_t;

/// The baseline about which mathematical characters are centred — `math`.
///
/// In vertical writing mode this is the baseline about which mathematical
/// characters rotated 90 degrees clockwise are centred.
pub const HB_OT_LAYOUT_BASELINE_TAG_MATH: hb_ot_layout_baseline_tag_t =
    HB_TAG(b'm', b'a', b't', b'h') as hb_ot_layout_baseline_tag_t;

unsafe extern "C" {
    /// Converts a script and a language into the OpenType script and language
    /// tags a font would use for them.
    ///
    /// `script_count` and `language_count` are in/out: on entry each holds the
    /// capacity of the corresponding array, on return the number of tags
    /// written. Either count may be null, in which case the matching array is
    /// not filled. At most [`HB_OT_MAX_TAGS_PER_SCRIPT`] script tags and
    /// [`HB_OT_MAX_TAGS_PER_LANGUAGE`] language tags are ever produced.
    ///
    /// Since HarfBuzz 2.0.0.
    pub fn hb_ot_tags_from_script_and_language(
        script: hb_script_t,
        language: hb_language_t,
        script_count: *mut c_uint,
        script_tags: *mut hb_tag_t,
        language_count: *mut c_uint,
        language_tags: *mut hb_tag_t,
    );

    /// Converts an OpenType script tag into an [`hb_script_t`].
    pub fn hb_ot_tag_to_script(tag: hb_tag_t) -> hb_script_t;

    /// Converts an OpenType language tag into an [`hb_language_t`].
    ///
    /// The returned language is interned and must not be freed. It may be
    /// [`HB_LANGUAGE_INVALID`](crate::HB_LANGUAGE_INVALID).
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_ot_tag_to_language(tag: hb_tag_t) -> hb_language_t;

    /// Converts an OpenType script tag and language tag back into an
    /// [`hb_script_t`] and an [`hb_language_t`].
    ///
    /// Either output pointer may be null.
    ///
    /// Since HarfBuzz 2.0.0.
    pub fn hb_ot_tags_to_script_and_language(
        script_tag: hb_tag_t,
        language_tag: hb_tag_t,
        script: *mut hb_script_t,
        language: *mut hb_language_t,
    );

    /// Tests whether a face has any glyph classes defined in its `GDEF` table.
    pub fn hb_ot_layout_has_glyph_classes(face: *mut hb_face_t) -> hb_bool_t;

    /// Fetches the `GDEF` class of `glyph` in `face`.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_ot_layout_get_glyph_class(
        face: *mut hb_face_t,
        glyph: hb_codepoint_t,
    ) -> hb_ot_layout_glyph_class_t;

    /// Collects into `glyphs` every glyph of the face's `GDEF` table that
    /// belongs to `klass`.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_ot_layout_get_glyphs_in_class(
        face: *mut hb_face_t,
        klass: hb_ot_layout_glyph_class_t,
        glyphs: *mut hb_set_t,
    );

    /// Fetches the attachment points defined for `glyph` in the face's `GDEF`
    /// table, starting at `start_offset`.
    ///
    /// `point_count` is in/out — capacity in, count written out — and both it
    /// and `point_array` may be null. Returns the total number of attachment
    /// points for `glyph`, which may exceed the number written.
    ///
    /// Useful if the client program wishes to cache the list.
    pub fn hb_ot_layout_get_attach_points(
        face: *mut hb_face_t,
        glyph: hb_codepoint_t,
        start_offset: c_uint,
        point_count: *mut c_uint,
        point_array: *mut c_uint,
    ) -> c_uint;

    /// Fetches the caret positions defined for a ligature glyph in the font's
    /// `GDEF` table, starting at `start_offset`.
    ///
    /// A ligature formed from *n* characters has *n* − 1 caret positions: the
    /// first character is not represented, since its caret position is the
    /// glyph position. The positions returned are "unshaped" and must be fixed
    /// up for any kerning applied to the ligature glyph.
    ///
    /// `caret_count` is in/out — capacity in, count written out — and both it
    /// and `caret_array` may be null. Returns the total number of caret
    /// positions for `glyph`.
    pub fn hb_ot_layout_get_ligature_carets(
        font: *mut hb_font_t,
        direction: hb_direction_t,
        glyph: hb_codepoint_t,
        start_offset: c_uint,
        caret_count: *mut c_uint,
        caret_array: *mut hb_position_t,
    ) -> c_uint;

    /// Fetches the script tags enumerated in the face's `GSUB` or `GPOS` table,
    /// starting at `start_offset`.
    ///
    /// `table_tag` must be [`HB_OT_TAG_GSUB`] or [`HB_OT_TAG_GPOS`].
    /// `script_count` is in/out — capacity in, count written out — and both it
    /// and `script_tags` may be null. Returns the total number of script tags.
    pub fn hb_ot_layout_table_get_script_tags(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        start_offset: c_uint,
        script_count: *mut c_uint,
        script_tags: *mut hb_tag_t,
    ) -> c_uint;

    /// Fetches the index of `script_tag` in the face's `GSUB` or `GPOS` table.
    ///
    /// Returns true if the script itself was found. If it was not, the function
    /// returns false but still falls back to `DFLT`, then `dflt`, then `latn`,
    /// writing whichever it finds into `script_index`; if none is present,
    /// `script_index` is set to [`HB_OT_LAYOUT_NO_SCRIPT_INDEX`].
    ///
    /// Upstream's reference manual indexes this function under its deprecated
    /// section and points at [`hb_ot_layout_table_select_script`] instead,
    /// although the header does not formally deprecate it.
    pub fn hb_ot_layout_table_find_script(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        script_tag: hb_tag_t,
        script_index: *mut c_uint,
    ) -> hb_bool_t;

    /// Selects an OpenType script for `table_tag` from `script_tags`, in order
    /// of preference.
    ///
    /// If the table has none of the requested scripts, `DFLT`, `dflt` and
    /// `latn` are tried in that order. If it has none of those either,
    /// `script_index` is set to [`HB_OT_LAYOUT_NO_SCRIPT_INDEX`] and
    /// `chosen_script` to [`HB_TAG_NONE`](crate::HB_TAG_NONE).
    ///
    /// Both output pointers may be null. Returns true only when one of the
    /// *requested* scripts was selected — false when a fallback was used or
    /// nothing was found.
    ///
    /// Since HarfBuzz 2.0.0.
    pub fn hb_ot_layout_table_select_script(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        script_count: c_uint,
        script_tags: *const hb_tag_t,
        script_index: *mut c_uint,
        chosen_script: *mut hb_tag_t,
    ) -> hb_bool_t;

    /// Fetches the feature tags in the face's `GSUB` or `GPOS` table, starting
    /// at `start_offset`.
    ///
    /// There may be duplicate tags, belonging to different script/language
    /// system pairs of the table. `feature_count` is in/out and both it and
    /// `feature_tags` may be null. Returns the total number of feature tags.
    ///
    /// Since HarfBuzz 0.6.0.
    pub fn hb_ot_layout_table_get_feature_tags(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        start_offset: c_uint,
        feature_count: *mut c_uint,
        feature_tags: *mut hb_tag_t,
    ) -> c_uint;

    /// Fetches the language tags under `script_index` in the face's `GSUB` or
    /// `GPOS` table, starting at `start_offset`.
    ///
    /// `language_count` is in/out and both it and `language_tags` may be null.
    /// Returns the total number of language tags.
    ///
    /// Since HarfBuzz 0.6.0.
    pub fn hb_ot_layout_script_get_language_tags(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        script_index: c_uint,
        start_offset: c_uint,
        language_count: *mut c_uint,
        language_tags: *mut hb_tag_t,
    ) -> c_uint;

    /// Fetches the index of the first tag from `language_tags` present under
    /// `script_index` in the face's `GSUB` or `GPOS` table.
    ///
    /// If none is found, returns false and sets `language_index` to the default
    /// language index.
    ///
    /// Since HarfBuzz 2.0.0.
    pub fn hb_ot_layout_script_select_language(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        script_index: c_uint,
        language_count: c_uint,
        language_tags: *const hb_tag_t,
        language_index: *mut c_uint,
    ) -> hb_bool_t;

    /// As [`hb_ot_layout_script_select_language`], but also reports which tag
    /// was chosen.
    ///
    /// If none of the given tags is found, returns false, sets `language_index`
    /// to [`HB_OT_LAYOUT_DEFAULT_LANGUAGE_INDEX`] and `chosen_language` to
    /// [`HB_TAG_NONE`](crate::HB_TAG_NONE).
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_ot_layout_script_select_language2(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        script_index: c_uint,
        language_count: c_uint,
        language_tags: *const hb_tag_t,
        language_index: *mut c_uint,
        chosen_language: *mut hb_tag_t,
    ) -> hb_bool_t;

    /// Fetches the index of the required feature for the given script and
    /// language system in the face's `GSUB` or `GPOS` table.
    ///
    /// Returns true if a required feature is declared, false otherwise.
    ///
    /// Since HarfBuzz 0.6.0.
    pub fn hb_ot_layout_language_get_required_feature_index(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        script_index: c_uint,
        language_index: c_uint,
        feature_index: *mut c_uint,
    ) -> hb_bool_t;

    /// As [`hb_ot_layout_language_get_required_feature_index`], but also
    /// reports the required feature's tag.
    ///
    /// Since HarfBuzz 0.9.30.
    pub fn hb_ot_layout_language_get_required_feature(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        script_index: c_uint,
        language_index: c_uint,
        feature_index: *mut c_uint,
        feature_tag: *mut hb_tag_t,
    ) -> hb_bool_t;

    /// Fetches the feature indices under the given script and language system
    /// in the face's `GSUB` or `GPOS` table, starting at `start_offset`.
    ///
    /// `feature_count` is in/out and both it and `feature_indexes` may be null.
    /// Returns the total number of features.
    ///
    /// Since HarfBuzz 0.6.0.
    pub fn hb_ot_layout_language_get_feature_indexes(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        script_index: c_uint,
        language_index: c_uint,
        start_offset: c_uint,
        feature_count: *mut c_uint,
        feature_indexes: *mut c_uint,
    ) -> c_uint;

    /// As [`hb_ot_layout_language_get_feature_indexes`], but returns the
    /// features' tags rather than their indices.
    ///
    /// Since HarfBuzz 0.6.0.
    pub fn hb_ot_layout_language_get_feature_tags(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        script_index: c_uint,
        language_index: c_uint,
        start_offset: c_uint,
        feature_count: *mut c_uint,
        feature_tags: *mut hb_tag_t,
    ) -> c_uint;

    /// Fetches the index of `feature_tag` under the given script and language
    /// system in the face's `GSUB` or `GPOS` table.
    ///
    /// Returns true if the feature is found, false otherwise.
    ///
    /// Since HarfBuzz 0.6.0.
    pub fn hb_ot_layout_language_find_feature(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        script_index: c_uint,
        language_index: c_uint,
        feature_tag: hb_tag_t,
        feature_index: *mut c_uint,
    ) -> hb_bool_t;

    /// Fetches the lookup indices enumerated for `feature_index` in the face's
    /// `GSUB` or `GPOS` table, starting at `start_offset`.
    ///
    /// `lookup_count` is in/out and both it and `lookup_indexes` may be null.
    /// Returns the total number of lookups.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_ot_layout_feature_get_lookups(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        feature_index: c_uint,
        start_offset: c_uint,
        lookup_count: *mut c_uint,
        lookup_indexes: *mut c_uint,
    ) -> c_uint;

    /// Fetches the total number of lookups enumerated in the face's `GSUB` or
    /// `GPOS` table.
    ///
    /// Valid lookup indices for that table run from zero to one less than this.
    ///
    /// Since HarfBuzz 0.9.22.
    pub fn hb_ot_layout_table_get_lookup_count(face: *mut hb_face_t, table_tag: hb_tag_t)
    -> c_uint;

    /// Collects into `feature_indexes` the feature indices of the face's `GSUB`
    /// or `GPOS` table under the given scripts, languages and features.
    ///
    /// `scripts`, `languages` and `features` are each either null — meaning
    /// "all of them" — or an array terminated by
    /// [`HB_TAG_NONE`](crate::HB_TAG_NONE).
    ///
    /// Since HarfBuzz 1.8.5.
    pub fn hb_ot_layout_collect_features(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        scripts: *const hb_tag_t,
        languages: *const hb_tag_t,
        features: *const hb_tag_t,
        feature_indexes: *mut hb_set_t,
    );

    /// Fetches the mapping from feature tag to feature index for the given
    /// script and language system.
    ///
    /// Since HarfBuzz 8.1.0.
    pub fn hb_ot_layout_collect_features_map(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        script_index: c_uint,
        language_index: c_uint,
        feature_map: *mut hb_map_t,
    );

    /// Collects into `lookup_indexes` the lookup indices reachable from the
    /// face's `GSUB` or `GPOS` table under the given scripts, languages and
    /// features.
    ///
    /// `scripts`, `languages` and `features` are each either null — meaning
    /// "all of them" — or an array terminated by
    /// [`HB_TAG_NONE`](crate::HB_TAG_NONE).
    ///
    /// Since HarfBuzz 0.9.8.
    pub fn hb_ot_layout_collect_lookups(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        scripts: *const hb_tag_t,
        languages: *const hb_tag_t,
        features: *const hb_tag_t,
        lookup_indexes: *mut hb_set_t,
    );

    /// Collects the glyphs affected by `lookup_index` in the face's `GSUB` or
    /// `GPOS` table.
    ///
    /// The four output sets receive, respectively: glyphs preceding the
    /// substitution range, the input glyphs the lookup would act on, glyphs
    /// following the range, and the glyphs the lookup would produce. Each may
    /// be null.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_ot_layout_lookup_collect_glyphs(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        lookup_index: c_uint,
        glyphs_before: *mut hb_set_t,
        glyphs_input: *mut hb_set_t,
        glyphs_after: *mut hb_set_t,
        glyphs_output: *mut hb_set_t,
    );

    /// Finds the feature-variations record of the face's `GSUB` or `GPOS` table
    /// that applies at the given normalized variation coordinates.
    ///
    /// Returns true if a record was found. `variations_index` receives the
    /// record's index, or [`HB_OT_LAYOUT_NO_VARIATIONS_INDEX`] when none
    /// applies.
    ///
    /// Since HarfBuzz 1.4.0.
    pub fn hb_ot_layout_table_find_feature_variations(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        coords: *const c_int,
        num_coords: c_uint,
        variations_index: *mut c_uint,
    ) -> hb_bool_t;

    /// As [`hb_ot_layout_feature_get_lookups`], but reports the lookups the
    /// feature enables at the given feature-variations index.
    ///
    /// Since HarfBuzz 1.4.0.
    pub fn hb_ot_layout_feature_with_variations_get_lookups(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        feature_index: c_uint,
        variations_index: c_uint,
        start_offset: c_uint,
        lookup_count: *mut c_uint,
        lookup_indexes: *mut c_uint,
    ) -> c_uint;

    /// Tests whether the face includes any `GSUB` substitutions.
    ///
    /// Since HarfBuzz 0.6.0.
    pub fn hb_ot_layout_has_substitution(face: *mut hb_face_t) -> hb_bool_t;

    /// Fetches the alternates of `glyph` from the `GSUB` lookup at
    /// `lookup_index`, starting at `start_offset`.
    ///
    /// For one-to-one substitutions this yields the substituted glyph.
    /// `alternate_count` is in/out and both it and `alternate_glyphs` may be
    /// null. Returns the total number of alternates found.
    ///
    /// Since HarfBuzz 2.6.8.
    pub fn hb_ot_layout_lookup_get_glyph_alternates(
        face: *mut hb_face_t,
        lookup_index: c_uint,
        glyph: hb_codepoint_t,
        start_offset: c_uint,
        alternate_count: *mut c_uint,
        alternate_glyphs: *mut hb_codepoint_t,
    ) -> c_uint;

    /// Collects the alternates of many glyphs from the `GSUB` lookup at
    /// `lookup_index` in one pass.
    ///
    /// Both maps are in/out. On entry `alternate_count` holds the glyph IDs of
    /// interest as keys, with the number of alternates already known for each
    /// as values; on return it holds the updated counts. `alternate_glyphs`
    /// stores alternate *i* of glyph *G* under the key `G + (i << 24)`.
    ///
    /// Returns true if alternates were collected; false for lookup types this
    /// does not handle, in which case nothing is written.
    ///
    /// Since HarfBuzz 12.1.0.
    pub fn hb_ot_layout_lookup_collect_glyph_alternates(
        face: *mut hb_face_t,
        lookup_index: c_uint,
        alternate_count: *mut hb_map_t,
        alternate_glyphs: *mut hb_map_t,
    ) -> hb_bool_t;

    /// Tests whether the lookup at `lookup_index` would substitute the given
    /// glyph sequence.
    ///
    /// `zero_context` indicates whether pre- and post-context are disallowed in
    /// the substitution.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_ot_layout_lookup_would_substitute(
        face: *mut hb_face_t,
        lookup_index: c_uint,
        glyphs: *const hb_codepoint_t,
        glyphs_length: c_uint,
        zero_context: hb_bool_t,
    ) -> hb_bool_t;

    /// Adds to `glyphs` the transitive closure of the glyphs the lookup at
    /// `lookup_index` can produce from the glyphs already in the set.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_ot_layout_lookup_substitute_closure(
        face: *mut hb_face_t,
        lookup_index: c_uint,
        glyphs: *mut hb_set_t,
    );

    /// As [`hb_ot_layout_lookup_substitute_closure`], but computes the closure
    /// over a whole set of lookups at once.
    ///
    /// Since HarfBuzz 1.8.1.
    pub fn hb_ot_layout_lookups_substitute_closure(
        face: *mut hb_face_t,
        lookups: *const hb_set_t,
        glyphs: *mut hb_set_t,
    );

    /// Tests whether the face includes any `GPOS` positioning.
    pub fn hb_ot_layout_has_positioning(face: *mut hb_face_t) -> hb_bool_t;

    /// Fetches the optical-size feature data — the `size` feature from `GPOS`.
    ///
    /// Every output pointer may be null. The `subfamily_id` and the subfamily
    /// name reached through `subfamily_name_id` pertain only to fonts within a
    /// family that differ specifically in their size ranges; other ways of
    /// differentiating fonts within a subfamily are outside the `size`
    /// feature's scope. See the
    /// [`size` feature documentation](https://docs.microsoft.com/en-us/typography/opentype/spec/features_pt#tag-size).
    ///
    /// Returns true if the data was found, false otherwise.
    ///
    /// Since HarfBuzz 0.9.10.
    pub fn hb_ot_layout_get_size_params(
        face: *mut hb_face_t,
        design_size: *mut c_uint,
        subfamily_id: *mut c_uint,
        subfamily_name_id: *mut hb_ot_name_id_t,
        range_start: *mut c_uint,
        range_end: *mut c_uint,
    ) -> hb_bool_t;

    /// Fetches the optical bound of a glyph positioned at the margin of text.
    ///
    /// `direction` identifies which edge of the glyph to query. Returns the
    /// adjustment value; negative values mean the glyph will stick out of the
    /// margin.
    ///
    /// Since HarfBuzz 5.3.0.
    pub fn hb_ot_layout_lookup_get_optical_bound(
        font: *mut hb_font_t,
        lookup_index: c_uint,
        direction: hb_direction_t,
        glyph: hb_codepoint_t,
    ) -> hb_position_t;

    /// Fetches the `name`-table name IDs recorded in the feature parameters of
    /// a Stylistic Set (`ssXX`) or Character Variant (`cvXX`) feature.
    ///
    /// Every output pointer may be null. `label_id` names the feature for a
    /// user interface, `tooltip_id` supplies tooltip text, `sample_id` supplies
    /// illustrative sample text, `num_named_parameters` counts the named
    /// parameters, and `first_param_id` is the first name ID used to label
    /// them — necessarily zero when the count is zero.
    ///
    /// Returns true if the data was found, false otherwise.
    ///
    /// Since HarfBuzz 2.0.0.
    pub fn hb_ot_layout_feature_get_name_ids(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        feature_index: c_uint,
        label_id: *mut hb_ot_name_id_t,
        tooltip_id: *mut hb_ot_name_id_t,
        sample_id: *mut hb_ot_name_id_t,
        num_named_parameters: *mut c_uint,
        first_param_id: *mut hb_ot_name_id_t,
    ) -> hb_bool_t;

    /// Fetches the Unicode characters declared as having a variant under a
    /// Character Variant (`cvXX`) feature, starting at `start_offset`.
    ///
    /// `char_count` is in/out and both it and `characters` may be null. Returns
    /// the total number of sample characters in the feature.
    ///
    /// Since HarfBuzz 2.0.0.
    pub fn hb_ot_layout_feature_get_characters(
        face: *mut hb_face_t,
        table_tag: hb_tag_t,
        feature_index: c_uint,
        start_offset: c_uint,
        char_count: *mut c_uint,
        characters: *mut hb_codepoint_t,
    ) -> c_uint;

    /// Fetches script- and language-specific font extents from the `BASE`
    /// table's `MinMax` records.
    ///
    /// If no such extents are found, the font's default extents are fetched
    /// instead, so the return value can mostly be ignored. Per-script and
    /// per-language extents carry no line-gap value, and the line gap is set to
    /// zero in that case. `extents` may be null.
    ///
    /// Since HarfBuzz 8.0.0.
    pub fn hb_ot_layout_get_font_extents(
        font: *mut hb_font_t,
        direction: hb_direction_t,
        script_tag: hb_tag_t,
        language_tag: hb_tag_t,
        extents: *mut hb_font_extents_t,
    ) -> hb_bool_t;

    /// As [`hb_ot_layout_get_font_extents`], but takes an [`hb_script_t`] and
    /// an [`hb_language_t`] instead of OpenType tags.
    ///
    /// `language` may be [`HB_LANGUAGE_INVALID`](crate::HB_LANGUAGE_INVALID).
    ///
    /// Since HarfBuzz 8.0.0.
    pub fn hb_ot_layout_get_font_extents2(
        font: *mut hb_font_t,
        direction: hb_direction_t,
        script: hb_script_t,
        language: hb_language_t,
        extents: *mut hb_font_extents_t,
    ) -> hb_bool_t;

    /// Fetches the dominant horizontal baseline tag used by `script`.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_ot_layout_get_horizontal_baseline_tag_for_script(
        script: hb_script_t,
    ) -> hb_ot_layout_baseline_tag_t;

    /// Fetches a baseline value from the font's `BASE` table.
    ///
    /// Returns true if the value was found, false otherwise, in which case
    /// `coord` is left alone. `coord` may be null. `language_tag` is currently
    /// unused.
    ///
    /// Since HarfBuzz 2.6.0.
    pub fn hb_ot_layout_get_baseline(
        font: *mut hb_font_t,
        baseline_tag: hb_ot_layout_baseline_tag_t,
        direction: hb_direction_t,
        script_tag: hb_tag_t,
        language_tag: hb_tag_t,
        coord: *mut hb_position_t,
    ) -> hb_bool_t;

    /// As [`hb_ot_layout_get_baseline`], but takes an [`hb_script_t`] and an
    /// [`hb_language_t`] instead of OpenType tags.
    ///
    /// `language` may be [`HB_LANGUAGE_INVALID`](crate::HB_LANGUAGE_INVALID)
    /// and is currently unused.
    ///
    /// Since HarfBuzz 8.0.0.
    pub fn hb_ot_layout_get_baseline2(
        font: *mut hb_font_t,
        baseline_tag: hb_ot_layout_baseline_tag_t,
        direction: hb_direction_t,
        script: hb_script_t,
        language: hb_language_t,
        coord: *mut hb_position_t,
    ) -> hb_bool_t;

    /// Fetches a baseline value from the font, synthesizing it when the font
    /// does not supply one.
    ///
    /// Unlike [`hb_ot_layout_get_baseline`] this always writes a value to
    /// `coord`, so there is nothing to report and no return value.
    /// `language_tag` is currently unused.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_ot_layout_get_baseline_with_fallback(
        font: *mut hb_font_t,
        baseline_tag: hb_ot_layout_baseline_tag_t,
        direction: hb_direction_t,
        script_tag: hb_tag_t,
        language_tag: hb_tag_t,
        coord: *mut hb_position_t,
    );

    /// As [`hb_ot_layout_get_baseline_with_fallback`], but takes an
    /// [`hb_script_t`] and an [`hb_language_t`] instead of OpenType tags.
    ///
    /// `language` may be [`HB_LANGUAGE_INVALID`](crate::HB_LANGUAGE_INVALID)
    /// and is currently unused.
    ///
    /// Since HarfBuzz 8.0.0.
    pub fn hb_ot_layout_get_baseline_with_fallback2(
        font: *mut hb_font_t,
        baseline_tag: hb_ot_layout_baseline_tag_t,
        direction: hb_direction_t,
        script: hb_script_t,
        language: hb_language_t,
        coord: *mut hb_position_t,
    );
}
