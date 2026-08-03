//! OpenType mathematical typesetting data — `hb-ot-math.h`.
//!
//! HarfBuzz does not implement a math layout solution of its own. What it
//! offers here is read access to the `MATH` table, so that a client program can
//! do the typesetting itself.

use core::ffi::{c_int, c_uint};

use crate::{
    HB_TAG, hb_bool_t, hb_codepoint_t, hb_direction_t, hb_face_t, hb_font_t, hb_position_t,
    hb_tag_t,
};

/// The tag of the OpenType
/// [Mathematical Typesetting Table](https://docs.microsoft.com/en-us/typography/opentype/spec/math),
/// `MATH`.
///
/// Since HarfBuzz 1.3.3.
pub const HB_OT_TAG_MATH: hb_tag_t = HB_TAG(b'M', b'A', b'T', b'H');

/// The OpenType script tag, `math`, for features specific to math shaping.
///
/// This is not a valid [`hb_script_t`](crate::hb_script_t) and should only be
/// used with functions that accept raw OpenType script tags, such as
/// `hb_ot_layout_collect_features`. In other cases,
/// [`HB_SCRIPT_MATH`](crate::HB_SCRIPT_MATH) should be used instead.
///
/// Since HarfBuzz 3.4.0.
pub const HB_OT_TAG_MATH_SCRIPT: hb_tag_t = HB_TAG(b'm', b'a', b't', b'h');

/// The `MATH` table constants.
///
/// See the
/// [OpenType documentation](https://docs.microsoft.com/en-us/typography/opentype/spec/math#mathconstants-table)
/// for the meaning of each one. Read them with [`hb_ot_math_get_constant`].
///
/// Most constants are lengths and come back as an
/// [`hb_position_t`](crate::hb_position_t) scaled to the font. Three of them —
/// [`HB_OT_MATH_CONSTANT_SCRIPT_PERCENT_SCALE_DOWN`],
/// [`HB_OT_MATH_CONSTANT_SCRIPT_SCRIPT_PERCENT_SCALE_DOWN`], and
/// [`HB_OT_MATH_CONSTANT_RADICAL_DEGREE_BOTTOM_RAISE_PERCENT`] — are instead
/// percentages between 0 and 100.
///
/// The C enumeration has no sentinel and its largest enumerator is 55, so it
/// fits in an `int`.
///
/// Since HarfBuzz 1.3.3.
pub type hb_ot_math_constant_t = c_int;

/// `scriptPercentScaleDown`: percentage scale down applied to script-level
/// text.
pub const HB_OT_MATH_CONSTANT_SCRIPT_PERCENT_SCALE_DOWN: hb_ot_math_constant_t = 0;
/// `scriptScriptPercentScaleDown`: percentage scale down applied to
/// script-script-level text.
pub const HB_OT_MATH_CONSTANT_SCRIPT_SCRIPT_PERCENT_SCALE_DOWN: hb_ot_math_constant_t = 1;
/// `delimitedSubFormulaMinHeight`: minimum height of a sub-formula inside
/// delimiters before the delimiters are stretched.
pub const HB_OT_MATH_CONSTANT_DELIMITED_SUB_FORMULA_MIN_HEIGHT: hb_ot_math_constant_t = 2;
/// `displayOperatorMinHeight`: minimum height of n-ary operators such as the
/// summation sign in display style.
pub const HB_OT_MATH_CONSTANT_DISPLAY_OPERATOR_MIN_HEIGHT: hb_ot_math_constant_t = 3;
/// `mathLeading`: white space to be left between math formulae to ensure
/// proper line spacing.
pub const HB_OT_MATH_CONSTANT_MATH_LEADING: hb_ot_math_constant_t = 4;
/// `axisHeight`: height of the math axis above the baseline, where fraction
/// bars and binary operators are centred.
pub const HB_OT_MATH_CONSTANT_AXIS_HEIGHT: hb_ot_math_constant_t = 5;
/// `accentBaseHeight`: maximum height of a base above which an accent is
/// raised no further.
pub const HB_OT_MATH_CONSTANT_ACCENT_BASE_HEIGHT: hb_ot_math_constant_t = 6;
/// `flattenedAccentBaseHeight`: base height above which a flattened accent
/// form is used.
pub const HB_OT_MATH_CONSTANT_FLATTENED_ACCENT_BASE_HEIGHT: hb_ot_math_constant_t = 7;
/// `subscriptShiftDown`: standard shift down applied to subscript elements.
pub const HB_OT_MATH_CONSTANT_SUBSCRIPT_SHIFT_DOWN: hb_ot_math_constant_t = 8;
/// `subscriptTopMax`: maximum allowed height of the top of a subscript.
pub const HB_OT_MATH_CONSTANT_SUBSCRIPT_TOP_MAX: hb_ot_math_constant_t = 9;
/// `subscriptBaselineDropMin`: minimum drop of the subscript baseline below
/// the base's bottom.
pub const HB_OT_MATH_CONSTANT_SUBSCRIPT_BASELINE_DROP_MIN: hb_ot_math_constant_t = 10;
/// `superscriptShiftUp`: standard shift up applied to superscript elements.
pub const HB_OT_MATH_CONSTANT_SUPERSCRIPT_SHIFT_UP: hb_ot_math_constant_t = 11;
/// `superscriptShiftUpCramped`: standard shift up applied to superscripts in
/// cramped style.
pub const HB_OT_MATH_CONSTANT_SUPERSCRIPT_SHIFT_UP_CRAMPED: hb_ot_math_constant_t = 12;
/// `superscriptBottomMin`: minimum allowed height of the bottom of a
/// superscript.
pub const HB_OT_MATH_CONSTANT_SUPERSCRIPT_BOTTOM_MIN: hb_ot_math_constant_t = 13;
/// `superscriptBaselineDropMax`: maximum drop of the superscript baseline
/// below the base's top.
pub const HB_OT_MATH_CONSTANT_SUPERSCRIPT_BASELINE_DROP_MAX: hb_ot_math_constant_t = 14;
/// `subSuperscriptGapMin`: minimum gap between a superscript's bottom and a
/// subscript's top.
pub const HB_OT_MATH_CONSTANT_SUB_SUPERSCRIPT_GAP_MIN: hb_ot_math_constant_t = 15;
/// `superscriptBottomMaxWithSubscript`: maximum superscript bottom when a
/// subscript is also present.
pub const HB_OT_MATH_CONSTANT_SUPERSCRIPT_BOTTOM_MAX_WITH_SUBSCRIPT: hb_ot_math_constant_t = 16;
/// `spaceAfterScript`: extra white space added after each sub- or superscript.
pub const HB_OT_MATH_CONSTANT_SPACE_AFTER_SCRIPT: hb_ot_math_constant_t = 17;
/// `upperLimitGapMin`: minimum gap between an upper limit and the operator it
/// sits above.
pub const HB_OT_MATH_CONSTANT_UPPER_LIMIT_GAP_MIN: hb_ot_math_constant_t = 18;
/// `upperLimitBaselineRiseMin`: minimum rise of an upper limit's baseline
/// above the operator's top.
pub const HB_OT_MATH_CONSTANT_UPPER_LIMIT_BASELINE_RISE_MIN: hb_ot_math_constant_t = 19;
/// `lowerLimitGapMin`: minimum gap between a lower limit and the operator it
/// sits below.
pub const HB_OT_MATH_CONSTANT_LOWER_LIMIT_GAP_MIN: hb_ot_math_constant_t = 20;
/// `lowerLimitBaselineDropMin`: minimum drop of a lower limit's baseline below
/// the operator's bottom.
pub const HB_OT_MATH_CONSTANT_LOWER_LIMIT_BASELINE_DROP_MIN: hb_ot_math_constant_t = 21;
/// `stackTopShiftUp`: standard shift up applied to the top element of a stack.
pub const HB_OT_MATH_CONSTANT_STACK_TOP_SHIFT_UP: hb_ot_math_constant_t = 22;
/// `stackTopDisplayStyleShiftUp`: standard shift up applied to the top element
/// of a stack in display style.
pub const HB_OT_MATH_CONSTANT_STACK_TOP_DISPLAY_STYLE_SHIFT_UP: hb_ot_math_constant_t = 23;
/// `stackBottomShiftDown`: standard shift down applied to the bottom element
/// of a stack.
pub const HB_OT_MATH_CONSTANT_STACK_BOTTOM_SHIFT_DOWN: hb_ot_math_constant_t = 24;
/// `stackBottomDisplayStyleShiftDown`: standard shift down applied to the
/// bottom element of a stack in display style.
pub const HB_OT_MATH_CONSTANT_STACK_BOTTOM_DISPLAY_STYLE_SHIFT_DOWN: hb_ot_math_constant_t = 25;
/// `stackGapMin`: minimum gap between the elements of a stack.
pub const HB_OT_MATH_CONSTANT_STACK_GAP_MIN: hb_ot_math_constant_t = 26;
/// `stackDisplayStyleGapMin`: minimum gap between the elements of a stack in
/// display style.
pub const HB_OT_MATH_CONSTANT_STACK_DISPLAY_STYLE_GAP_MIN: hb_ot_math_constant_t = 27;
/// `stretchStackTopShiftUp`: standard shift up applied to the top element of a
/// stretch stack.
pub const HB_OT_MATH_CONSTANT_STRETCH_STACK_TOP_SHIFT_UP: hb_ot_math_constant_t = 28;
/// `stretchStackBottomShiftDown`: standard shift down applied to the bottom
/// element of a stretch stack.
pub const HB_OT_MATH_CONSTANT_STRETCH_STACK_BOTTOM_SHIFT_DOWN: hb_ot_math_constant_t = 29;
/// `stretchStackGapAboveMin`: minimum gap between the stretched element and
/// the element above it.
pub const HB_OT_MATH_CONSTANT_STRETCH_STACK_GAP_ABOVE_MIN: hb_ot_math_constant_t = 30;
/// `stretchStackGapBelowMin`: minimum gap between the stretched element and
/// the element below it.
pub const HB_OT_MATH_CONSTANT_STRETCH_STACK_GAP_BELOW_MIN: hb_ot_math_constant_t = 31;
/// `fractionNumeratorShiftUp`: standard shift up applied to a fraction's
/// numerator.
pub const HB_OT_MATH_CONSTANT_FRACTION_NUMERATOR_SHIFT_UP: hb_ot_math_constant_t = 32;
/// `fractionNumeratorDisplayStyleShiftUp`: standard shift up applied to a
/// fraction's numerator in display style.
pub const HB_OT_MATH_CONSTANT_FRACTION_NUMERATOR_DISPLAY_STYLE_SHIFT_UP: hb_ot_math_constant_t = 33;
/// `fractionDenominatorShiftDown`: standard shift down applied to a fraction's
/// denominator.
pub const HB_OT_MATH_CONSTANT_FRACTION_DENOMINATOR_SHIFT_DOWN: hb_ot_math_constant_t = 34;
/// `fractionDenominatorDisplayStyleShiftDown`: standard shift down applied to
/// a fraction's denominator in display style.
pub const HB_OT_MATH_CONSTANT_FRACTION_DENOMINATOR_DISPLAY_STYLE_SHIFT_DOWN: hb_ot_math_constant_t =
    35;
/// `fractionNumeratorGapMin`: minimum gap between a fraction bar and the
/// numerator above it.
pub const HB_OT_MATH_CONSTANT_FRACTION_NUMERATOR_GAP_MIN: hb_ot_math_constant_t = 36;
/// `fractionNumDisplayStyleGapMin`: minimum gap between a fraction bar and the
/// numerator above it, in display style.
pub const HB_OT_MATH_CONSTANT_FRACTION_NUM_DISPLAY_STYLE_GAP_MIN: hb_ot_math_constant_t = 37;
/// `fractionRuleThickness`: thickness of the fraction bar.
pub const HB_OT_MATH_CONSTANT_FRACTION_RULE_THICKNESS: hb_ot_math_constant_t = 38;
/// `fractionDenominatorGapMin`: minimum gap between a fraction bar and the
/// denominator below it.
pub const HB_OT_MATH_CONSTANT_FRACTION_DENOMINATOR_GAP_MIN: hb_ot_math_constant_t = 39;
/// `fractionDenomDisplayStyleGapMin`: minimum gap between a fraction bar and
/// the denominator below it, in display style.
pub const HB_OT_MATH_CONSTANT_FRACTION_DENOM_DISPLAY_STYLE_GAP_MIN: hb_ot_math_constant_t = 40;
/// `skewedFractionHorizontalGap`: horizontal distance between the numerator
/// and denominator of a skewed fraction.
pub const HB_OT_MATH_CONSTANT_SKEWED_FRACTION_HORIZONTAL_GAP: hb_ot_math_constant_t = 41;
/// `skewedFractionVerticalGap`: vertical distance between the ink of the
/// numerator and denominator of a skewed fraction.
pub const HB_OT_MATH_CONSTANT_SKEWED_FRACTION_VERTICAL_GAP: hb_ot_math_constant_t = 42;
/// `overbarVerticalGap`: distance between the overbar and the ink top of the
/// base.
pub const HB_OT_MATH_CONSTANT_OVERBAR_VERTICAL_GAP: hb_ot_math_constant_t = 43;
/// `overbarRuleThickness`: thickness of the overbar.
pub const HB_OT_MATH_CONSTANT_OVERBAR_RULE_THICKNESS: hb_ot_math_constant_t = 44;
/// `overbarExtraAscender`: extra white space reserved above the overbar.
pub const HB_OT_MATH_CONSTANT_OVERBAR_EXTRA_ASCENDER: hb_ot_math_constant_t = 45;
/// `underbarVerticalGap`: distance between the underbar and the ink bottom of
/// the base.
pub const HB_OT_MATH_CONSTANT_UNDERBAR_VERTICAL_GAP: hb_ot_math_constant_t = 46;
/// `underbarRuleThickness`: thickness of the underbar.
pub const HB_OT_MATH_CONSTANT_UNDERBAR_RULE_THICKNESS: hb_ot_math_constant_t = 47;
/// `underbarExtraDescender`: extra white space reserved below the underbar.
pub const HB_OT_MATH_CONSTANT_UNDERBAR_EXTRA_DESCENDER: hb_ot_math_constant_t = 48;
/// `radicalVerticalGap`: space between the ink top of the radicand and the
/// radical rule.
pub const HB_OT_MATH_CONSTANT_RADICAL_VERTICAL_GAP: hb_ot_math_constant_t = 49;
/// `radicalDisplayStyleVerticalGap`: space between the ink top of the radicand
/// and the radical rule, in display style.
pub const HB_OT_MATH_CONSTANT_RADICAL_DISPLAY_STYLE_VERTICAL_GAP: hb_ot_math_constant_t = 50;
/// `radicalRuleThickness`: thickness of the radical rule.
pub const HB_OT_MATH_CONSTANT_RADICAL_RULE_THICKNESS: hb_ot_math_constant_t = 51;
/// `radicalExtraAscender`: extra white space reserved above the radical rule.
pub const HB_OT_MATH_CONSTANT_RADICAL_EXTRA_ASCENDER: hb_ot_math_constant_t = 52;
/// `radicalKernBeforeDegree`: extra horizontal kern before the degree of a
/// radical, if such a kern is needed.
pub const HB_OT_MATH_CONSTANT_RADICAL_KERN_BEFORE_DEGREE: hb_ot_math_constant_t = 53;
/// `radicalKernAfterDegree`: negative kern after the degree of a radical, if
/// such a kern is needed.
pub const HB_OT_MATH_CONSTANT_RADICAL_KERN_AFTER_DEGREE: hb_ot_math_constant_t = 54;
/// `radicalDegreeBottomRaisePercent`: height of the bottom of the radical
/// degree, as a percentage of the radical sign's ascender.
pub const HB_OT_MATH_CONSTANT_RADICAL_DEGREE_BOTTOM_RAISE_PERCENT: hb_ot_math_constant_t = 55;

/// The math kerning-table types defined for the four corners of a glyph.
///
/// The C enumeration has no sentinel and its largest enumerator is 3, so it
/// fits in an `int`.
///
/// Since HarfBuzz 1.3.3.
pub type hb_ot_math_kern_t = c_int;

/// The top right corner of the glyph.
pub const HB_OT_MATH_KERN_TOP_RIGHT: hb_ot_math_kern_t = 0;
/// The top left corner of the glyph.
pub const HB_OT_MATH_KERN_TOP_LEFT: hb_ot_math_kern_t = 1;
/// The bottom right corner of the glyph.
pub const HB_OT_MATH_KERN_BOTTOM_RIGHT: hb_ot_math_kern_t = 2;
/// The bottom left corner of the glyph.
pub const HB_OT_MATH_KERN_BOTTOM_LEFT: hb_ot_math_kern_t = 3;

/// Data type to hold math kerning (cut-in) information for a glyph.
///
/// Filled in by [`hb_ot_math_get_glyph_kernings`].
///
/// Since HarfBuzz 3.4.0.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_ot_math_kern_entry_t {
    /// The maximum height at which this entry should be used.
    pub max_correction_height: hb_position_t,
    /// The kern value of the entry.
    pub kern_value: hb_position_t,
}

/// Data type to hold math-variant information for a glyph.
///
/// Filled in by [`hb_ot_math_get_glyph_variants`].
///
/// Since HarfBuzz 1.3.3.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_ot_math_glyph_variant_t {
    /// The glyph index of the variant.
    pub glyph: hb_codepoint_t,
    /// The advance width of the variant.
    pub advance: hb_position_t,
}

/// Flags for math glyph parts.
///
/// The C enumeration is a flags enumeration with no sentinel, and its only
/// enumerator is 1, so it fits in an `int`.
///
/// Since HarfBuzz 1.3.3.
pub type hb_ot_math_glyph_part_flags_t = c_int;

/// This is an extender glyph part that can be repeated to reach the desired
/// length.
pub const HB_OT_MATH_GLYPH_PART_FLAG_EXTENDER: hb_ot_math_glyph_part_flags_t = 0x00000001;

/// Data type to hold information for a "part" component of a math-variant
/// glyph.
///
/// Large variants for stretchable math glyphs — such as parentheses — can be
/// constructed on the fly from parts. Filled in by
/// [`hb_ot_math_get_glyph_assembly`].
///
/// Since HarfBuzz 1.3.3.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_ot_math_glyph_part_t {
    /// The glyph index of the variant part.
    pub glyph: hb_codepoint_t,
    /// The length of the connector on the starting side of the variant part.
    pub start_connector_length: hb_position_t,
    /// The length of the connector on the ending side of the variant part.
    pub end_connector_length: hb_position_t,
    /// The total advance of the part.
    pub full_advance: hb_position_t,
    /// [`hb_ot_math_glyph_part_flags_t`] flags for the part.
    pub flags: hb_ot_math_glyph_part_flags_t,
}

unsafe extern "C" {
    /// Tests whether a face has a `MATH` table.
    ///
    /// Returns true if the table is found, false otherwise.
    ///
    /// Since HarfBuzz 1.3.3.
    pub fn hb_ot_math_has_data(face: *mut hb_face_t) -> hb_bool_t;

    /// Fetches the specified math constant.
    ///
    /// For most constants the value returned is an
    /// [`hb_position_t`](crate::hb_position_t). However, if the requested
    /// constant is [`HB_OT_MATH_CONSTANT_SCRIPT_PERCENT_SCALE_DOWN`],
    /// [`HB_OT_MATH_CONSTANT_SCRIPT_SCRIPT_PERCENT_SCALE_DOWN`], or
    /// [`HB_OT_MATH_CONSTANT_RADICAL_DEGREE_BOTTOM_RAISE_PERCENT`], the return
    /// value is an integer between 0 and 100 representing that percentage.
    ///
    /// Returns the requested constant, or zero.
    ///
    /// Since HarfBuzz 1.3.3.
    pub fn hb_ot_math_get_constant(
        font: *mut hb_font_t,
        constant: hb_ot_math_constant_t,
    ) -> hb_position_t;

    /// Fetches an italics-correction value, if one exists, for the specified
    /// glyph index.
    ///
    /// Returns the italics correction of the glyph, or zero.
    ///
    /// Since HarfBuzz 1.3.3.
    pub fn hb_ot_math_get_glyph_italics_correction(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
    ) -> hb_position_t;

    /// Fetches a top-accent-attachment value, if one exists, for the specified
    /// glyph index.
    ///
    /// For any glyph that does not have a top-accent-attachment value — that
    /// is, a glyph not covered by the `MathTopAccentAttachment` table, or any
    /// glyph at all when the font has no `MathTopAccentAttachment` table or no
    /// `MATH` table — the function synthesizes a value, returning the position
    /// at one-half the glyph's advance width.
    ///
    /// Returns the top accent attachment of the glyph, or half the advance
    /// width of `glyph`.
    ///
    /// Since HarfBuzz 1.3.3.
    pub fn hb_ot_math_get_glyph_top_accent_attachment(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
    ) -> hb_position_t;

    /// Tests whether the given glyph index is an extended shape in the face.
    ///
    /// Returns true if the glyph is an extended shape, false otherwise.
    ///
    /// Since HarfBuzz 1.3.3.
    pub fn hb_ot_math_is_glyph_extended_shape(
        face: *mut hb_face_t,
        glyph: hb_codepoint_t,
    ) -> hb_bool_t;

    /// Fetches the math kerning (cut-in) value for the specified font, glyph
    /// index, and corner.
    ///
    /// If the `MathKern` table is found, the function examines it to find a
    /// height value that is greater than or equal to `correction_height`. If
    /// such a height value is found, the corresponding kerning value from the
    /// table is returned. If no such height value is found, the last kerning
    /// value is returned.
    ///
    /// Returns the requested kerning value, or zero.
    ///
    /// Since HarfBuzz 1.3.3.
    pub fn hb_ot_math_get_glyph_kerning(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        kern: hb_ot_math_kern_t,
        correction_height: hb_position_t,
    ) -> hb_position_t;

    /// Fetches the raw `MathKern` (cut-in) data for the specified font, glyph
    /// index, and corner.
    ///
    /// The corresponding list of kern values and correction heights is returned
    /// as a list of [`hb_ot_math_kern_entry_t`] structs. `entries_count` is an
    /// in/out parameter: on input the maximum number of entries to return, on
    /// output the number actually written into `kern_entries`. Both may be
    /// null, in which case nothing is written and only the total is reported.
    ///
    /// See also [`hb_ot_math_get_glyph_kerning`], which handles selecting the
    /// appropriate kern value for a given correction height.
    ///
    /// For a glyph with *n* defined kern values (where *n* > 0), there are only
    /// *n* − 1 defined correction heights, as each correction height defines a
    /// boundary past which the next kern value should be selected. Therefore
    /// only the [`kern_value`](hb_ot_math_kern_entry_t::kern_value) of the
    /// uppermost entry actually comes from the font; its corresponding
    /// [`max_correction_height`](hb_ot_math_kern_entry_t::max_correction_height)
    /// is always set to [`i32::MAX`].
    ///
    /// Returns the total number of kern values available, or zero.
    ///
    /// Since HarfBuzz 3.4.0.
    pub fn hb_ot_math_get_glyph_kernings(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        kern: hb_ot_math_kern_t,
        start_offset: c_uint,
        entries_count: *mut c_uint,
        kern_entries: *mut hb_ot_math_kern_entry_t,
    ) -> c_uint;

    /// Fetches the `MathGlyphConstruction` for the specified font, glyph index,
    /// and direction.
    ///
    /// The corresponding list of size variants is returned as a list of
    /// [`hb_ot_math_glyph_variant_t`] structs. `variants_count` is an in/out
    /// parameter: on input the maximum number of variants to return, on output
    /// the number actually written into `variants`.
    ///
    /// The `direction` parameter is only used to select between horizontal and
    /// vertical directions for the construction. Even though all
    /// [`hb_direction_t`](crate::hb_direction_t) values are accepted, only the
    /// result of
    /// [`HB_DIRECTION_IS_HORIZONTAL`](crate::HB_DIRECTION_IS_HORIZONTAL) is
    /// considered.
    ///
    /// Returns the total number of size variants available, or zero.
    ///
    /// Since HarfBuzz 1.3.3.
    pub fn hb_ot_math_get_glyph_variants(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        direction: hb_direction_t,
        start_offset: c_uint,
        variants_count: *mut c_uint,
        variants: *mut hb_ot_math_glyph_variant_t,
    ) -> c_uint;

    /// Fetches the `MathVariants` table for the specified font and returns the
    /// minimum overlap of connecting glyphs required to draw a glyph assembly
    /// in the specified direction.
    ///
    /// The `direction` parameter is only used to select between horizontal and
    /// vertical directions for the construction. Even though all
    /// [`hb_direction_t`](crate::hb_direction_t) values are accepted, only the
    /// result of
    /// [`HB_DIRECTION_IS_HORIZONTAL`](crate::HB_DIRECTION_IS_HORIZONTAL) is
    /// considered.
    ///
    /// Returns the requested minimum connector overlap, or zero.
    ///
    /// Since HarfBuzz 1.3.3.
    pub fn hb_ot_math_get_min_connector_overlap(
        font: *mut hb_font_t,
        direction: hb_direction_t,
    ) -> hb_position_t;

    /// Fetches the `GlyphAssembly` for the specified font, glyph index, and
    /// direction.
    ///
    /// Returned are a list of [`hb_ot_math_glyph_part_t`] glyph parts that can
    /// be used to draw the glyph and — through `italics_correction` — an
    /// italics-correction value, if one is defined in the font. `parts_count`
    /// is an in/out parameter: on input the maximum number of parts to return,
    /// on output the number actually written into `parts`.
    ///
    /// The `direction` parameter is only used to select between horizontal and
    /// vertical directions for the construction. Even though all
    /// [`hb_direction_t`](crate::hb_direction_t) values are accepted, only the
    /// result of
    /// [`HB_DIRECTION_IS_HORIZONTAL`](crate::HB_DIRECTION_IS_HORIZONTAL) is
    /// considered.
    ///
    /// Returns the total number of parts in the glyph assembly.
    ///
    /// Since HarfBuzz 1.3.3.
    pub fn hb_ot_math_get_glyph_assembly(
        font: *mut hb_font_t,
        glyph: hb_codepoint_t,
        direction: hb_direction_t,
        start_offset: c_uint,
        parts_count: *mut c_uint,
        parts: *mut hb_ot_math_glyph_part_t,
        italics_correction: *mut hb_position_t,
    ) -> c_uint;
}
