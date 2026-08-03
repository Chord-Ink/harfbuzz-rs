//! Text buffers — the input characters and output glyphs of shaping —
//! `hb-buffer.h`.

use core::ffi::{c_char, c_int, c_uint, c_void};

use crate::{
    HB_DIRECTION_INVALID, HB_LANGUAGE_INVALID, HB_SCRIPT_INVALID, HB_TAG, HB_TAG_NONE, hb_bool_t,
    hb_codepoint_t, hb_destroy_func_t, hb_direction_t, hb_font_t, hb_language_t, hb_mask_t,
    hb_position_t, hb_script_t, hb_unicode_funcs_t, hb_user_data_key_t, hb_var_int_t,
    opaque_handle,
};

/// Information about a single glyph and its relation to the input text.
///
/// Before shaping, `codepoint` holds a Unicode code point; afterwards it holds
/// a glyph index in the font that was shaped with.
///
/// The `mask`, `var1`, and `var2` fields are private to HarfBuzz. They are
/// declared here only so that the layout matches C — do not read or write them,
/// except through [`hb_glyph_info_get_glyph_flags`].
#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_glyph_info_t {
    /// Either a Unicode code point (before shaping) or a glyph index (after
    /// shaping).
    pub codepoint: hb_codepoint_t,

    /// Private to HarfBuzz. The low bits carry [`hb_glyph_flags_t`]; read them
    /// with [`hb_glyph_info_get_glyph_flags`] rather than by hand.
    pub mask: hb_mask_t,

    /// The index of the character in the original text that this glyph came
    /// from, or whatever value the client passed to [`hb_buffer_add`].
    ///
    /// Several glyphs can share one cluster value when they came from the same
    /// character (one-to-many substitution), and when several characters merge
    /// into one glyph (many-to-one substitution) the resulting glyph carries
    /// the smallest of their cluster values. By default some characters are
    /// merged into the same cluster — combining marks share the cluster of
    /// their base, for instance — even when they remain separate glyphs;
    /// [`hb_buffer_set_cluster_level`] selects more fine-grained handling.
    pub cluster: u32,

    /// Private to HarfBuzz. Scratch space used during shaping.
    pub var1: hb_var_int_t,
    /// Private to HarfBuzz. Scratch space used during shaping.
    pub var2: hb_var_int_t,
}

// `Debug` cannot be derived because `hb_var_int_t` is a union. Only the two
// public fields are printed; the private ones are meaningless to a client.
impl core::fmt::Debug for hb_glyph_info_t {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("hb_glyph_info_t")
            .field("codepoint", &self.codepoint)
            .field("cluster", &self.cluster)
            .finish_non_exhaustive()
    }
}

/// Flags describing how a glyph relates to the text around it.
///
/// These are bit flags. C gives the enumeration no sentinel and no value
/// exceeds `0x7FFFFFFF`, so the underlying type is `int`.
///
/// Since HarfBuzz 1.5.0.
pub type hb_glyph_flags_t = c_int;

/// Breaking the input text at the beginning of the cluster this glyph belongs
/// to requires re-shaping both sides, because the result might differ.
///
/// Conversely, when the flag is absent it is safe to break the glyph run at
/// the beginning of this cluster: the two halves will be identical to what you
/// would get by breaking the input text there and shaping the halves
/// separately. Paragraph layout can use this to avoid re-shaping every line
/// after line-breaking.
pub const HB_GLYPH_FLAG_UNSAFE_TO_BREAK: hb_glyph_flags_t = 0x00000001;

/// Changing the input text on one side of the beginning of this glyph's
/// cluster may change the shaping result on the other side.
///
/// The absence of this flag does *not* by itself mean concatenation is safe —
/// only two pieces of text that are both clear of it can be concatenated
/// safely.
///
/// It lets paragraph layout limit re-shaping to a small window around the break
/// position, even when that position is
/// [`HB_GLYPH_FLAG_UNSAFE_TO_BREAK`], and even when hyphenation or another text
/// transformation happens at the break. For the end of a line: iterate back
/// from the break until the first cluster start that is not unsafe-to-concat,
/// shape from there to the end of the line, and check that the resulting glyph
/// run is also clear of unsafe-to-concat at its start-of-text position; if it
/// is, splice it into place, otherwise move further back and retry. The start
/// of the next line works symmetrically, iterating forward. One complication:
/// the buffer API can report flags for the start-of-text position but has no
/// end-of-text position, which can be worked around by shaping more text than
/// needed and looking for the flag inside the text clusters.
///
/// [`HB_GLYPH_FLAG_UNSAFE_TO_BREAK`] always implies this flag. To use it you
/// must enable [`HB_BUFFER_FLAG_PRODUCE_UNSAFE_TO_CONCAT`] on the buffer during
/// shaping, otherwise it is not reliably produced.
///
/// Since HarfBuzz 4.0.0.
pub const HB_GLYPH_FLAG_UNSAFE_TO_CONCAT: hb_glyph_flags_t = 0x00000002;

/// In scripts that use elongation — Arabic, Mongolian, Syriac and others — it
/// is safe to insert a U+0640 TATWEEL before this cluster.
///
/// The flag does not identify the script-specific places where elongation
/// belongs; it only says that elongating here will not disrupt shaping.
///
/// Since HarfBuzz 5.1.0.
pub const HB_GLYPH_FLAG_SAFE_TO_INSERT_TATWEEL: hb_glyph_flags_t = 0x00000004;

/// The bitwise OR of every currently defined glyph flag.
pub const HB_GLYPH_FLAG_DEFINED: hb_glyph_flags_t = 0x00000007;

/// Positioning information for a single glyph.
///
/// All values are relative to the current point. The `var` field is private to
/// HarfBuzz and is declared here only so that the layout matches C.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct hb_glyph_position_t {
    /// How much the line advances after drawing this glyph when setting text
    /// horizontally.
    pub x_advance: hb_position_t,
    /// How much the line advances after drawing this glyph when setting text
    /// vertically.
    pub y_advance: hb_position_t,
    /// How much the glyph moves along the x-axis before being drawn. This does
    /// not affect how much the line advances.
    pub x_offset: hb_position_t,
    /// How much the glyph moves along the y-axis before being drawn. This does
    /// not affect how much the line advances.
    pub y_offset: hb_position_t,

    /// Private to HarfBuzz. Scratch space used during shaping.
    pub var: hb_var_int_t,
}

// `Debug` cannot be derived because `hb_var_int_t` is a union.
impl core::fmt::Debug for hb_glyph_position_t {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("hb_glyph_position_t")
            .field("x_advance", &self.x_advance)
            .field("y_advance", &self.y_advance)
            .field("x_offset", &self.x_offset)
            .field("y_offset", &self.y_offset)
            .finish_non_exhaustive()
    }
}

/// The text properties of an [`hb_buffer_t`]: direction, script, and language.
///
/// Set and retrieved as a unit with [`hb_buffer_set_segment_properties`] and
/// [`hb_buffer_get_segment_properties`]. The two reserved fields are private to
/// HarfBuzz and must be left as null.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct hb_segment_properties_t {
    /// The direction of the segment. See [`hb_buffer_set_direction`].
    pub direction: hb_direction_t,
    /// The script of the segment. See [`hb_buffer_set_script`].
    pub script: hb_script_t,
    /// The language of the segment. See [`hb_buffer_set_language`].
    pub language: hb_language_t,

    /// Private to HarfBuzz. Always null.
    pub reserved1: *mut c_void,
    /// Private to HarfBuzz. Always null.
    pub reserved2: *mut c_void,
}

/// The segment properties of a freshly created [`hb_buffer_t`]: everything
/// unset.
pub const HB_SEGMENT_PROPERTIES_DEFAULT: hb_segment_properties_t = hb_segment_properties_t {
    direction: HB_DIRECTION_INVALID,
    script: HB_SCRIPT_INVALID,
    language: HB_LANGUAGE_INVALID,
    reserved1: core::ptr::null_mut(),
    reserved2: core::ptr::null_mut(),
};

opaque_handle! {
    /// The main structure holding the input text and its properties before
    /// shaping, and the output glyphs and their information afterwards.
    hb_buffer_t
}

/// The kind of content an [`hb_buffer_t`] currently holds.
///
/// C gives the enumeration no sentinel and no value exceeds `0x7FFFFFFF`, so
/// the underlying type is `int`.
pub type hb_buffer_content_type_t = c_int;

/// The initial value for a new buffer.
pub const HB_BUFFER_CONTENT_TYPE_INVALID: hb_buffer_content_type_t = 0;
/// The buffer holds input characters, before shaping.
pub const HB_BUFFER_CONTENT_TYPE_UNICODE: hb_buffer_content_type_t = 1;
/// The buffer holds output glyphs, after shaping.
pub const HB_BUFFER_CONTENT_TYPE_GLYPHS: hb_buffer_content_type_t = 2;

/// Flags that change how an [`hb_buffer_t`] is shaped.
///
/// These are bit flags. C gives the enumeration no sentinel and no value
/// exceeds `0x7FFFFFFF`, so the underlying type is `int`.
///
/// Since HarfBuzz 0.9.20.
pub type hb_buffer_flags_t = c_int;

/// No flags set.
pub const HB_BUFFER_FLAG_DEFAULT: hb_buffer_flags_t = 0x00000000;

/// The buffer starts a text paragraph, so beginning-of-text handling may be
/// applied. Usually you want this, unless you are passing only part of the text
/// without its full context.
pub const HB_BUFFER_FLAG_BOT: hb_buffer_flags_t = 0x00000001;

/// The buffer ends a text paragraph, so end-of-text handling may be applied.
/// The counterpart of [`HB_BUFFER_FLAG_BOT`].
pub const HB_BUFFER_FLAG_EOT: hb_buffer_flags_t = 0x00000002;

/// Characters with the Unicode Default_Ignorable property should use the
/// corresponding glyph from the font rather than being hidden — hiding is done
/// by replacing them with the space glyph and zeroing the advance width.
///
/// Takes precedence over [`HB_BUFFER_FLAG_REMOVE_DEFAULT_IGNORABLES`].
pub const HB_BUFFER_FLAG_PRESERVE_DEFAULT_IGNORABLES: hb_buffer_flags_t = 0x00000004;

/// Characters with the Unicode Default_Ignorable property should be removed
/// from the glyph string rather than being hidden.
///
/// [`HB_BUFFER_FLAG_PRESERVE_DEFAULT_IGNORABLES`] takes precedence over this.
///
/// Since HarfBuzz 1.8.0.
pub const HB_BUFFER_FLAG_REMOVE_DEFAULT_IGNORABLES: hb_buffer_flags_t = 0x00000008;

/// Do not insert a dotted circle when rendering an incorrect character
/// sequence, such as `<0905 093E>`.
///
/// Since HarfBuzz 2.4.0.
pub const HB_BUFFER_FLAG_DO_NOT_INSERT_DOTTED_CIRCLE: hb_buffer_flags_t = 0x00000010;

/// `hb_shape` and its variants should run verification passes over the shaping
/// results.
///
/// On failure either a buffer message is sent, if a message handler is
/// installed on the buffer, or a message is written to standard error. Either
/// way the shaping result may be modified to show the failed output.
///
/// Since HarfBuzz 3.4.0.
pub const HB_BUFFER_FLAG_VERIFY: hb_buffer_flags_t = 0x00000020;

/// The shaper should produce the [`HB_GLYPH_FLAG_UNSAFE_TO_CONCAT`] glyph flag.
/// It is off by default because computing it costs time.
///
/// Since HarfBuzz 4.0.0.
pub const HB_BUFFER_FLAG_PRODUCE_UNSAFE_TO_CONCAT: hb_buffer_flags_t = 0x00000040;

/// The shaper should produce the [`HB_GLYPH_FLAG_SAFE_TO_INSERT_TATWEEL`] glyph
/// flag. It is off by default.
///
/// Since HarfBuzz 5.1.0.
pub const HB_BUFFER_FLAG_PRODUCE_SAFE_TO_INSERT_TATWEEL: hb_buffer_flags_t = 0x00000080;

/// The bitwise OR of every currently defined buffer flag.
///
/// Since HarfBuzz 4.4.0.
pub const HB_BUFFER_FLAG_DEFINED: hb_buffer_flags_t = 0x000000FF;

/// How HarfBuzz groups cluster values, which is one aspect of how it treats
/// non-base characters during shaping.
///
/// C gives the enumeration no sentinel and no value exceeds `0x7FFFFFFF`, so
/// the underlying type is `int`.
///
/// Since HarfBuzz 0.9.42.
pub type hb_buffer_cluster_level_t = c_int;

/// Return cluster values grouped by graphemes into monotone order.
///
/// Non-base characters are merged into the cluster of the base character that
/// precedes them, and clusters are merged again whenever they would otherwise
/// become non-monotone.
pub const HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES: hb_buffer_cluster_level_t = 0;

/// Return cluster values grouped into monotone order.
///
/// Non-base characters are initially given their own cluster values, which are
/// not merged into preceding base clusters. That lets HarfBuzz perform extra
/// operations such as reordering runs of adjacent marks. The output is still
/// monotone, but the cluster values are more granular.
pub const HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS: hb_buffer_cluster_level_t = 1;

/// Do not group cluster values.
///
/// Non-base characters get their own cluster values, which are neither merged
/// into preceding base clusters nor forced into monotone order. This is the
/// most granular level and tells you the exact cluster of every character, but
/// it is harder to consume because clusters may appear in any order.
pub const HB_BUFFER_CLUSTER_LEVEL_CHARACTERS: hb_buffer_cluster_level_t = 2;

/// Group clusters by grapheme, but do not enforce monotone order.
///
/// Non-base characters are merged into the cluster of the base character that
/// precedes them. This resembles the Unicode Grapheme Cluster algorithm without
/// being exactly the same, and makes HarfBuzz usable as a cheap implementation
/// of it.
pub const HB_BUFFER_CLUSTER_LEVEL_GRAPHEMES: hb_buffer_cluster_level_t = 3;

/// The default cluster level, equal to
/// [`HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES`].
///
/// It is the default because it keeps backward compatibility with older
/// versions of HarfBuzz. New programs that do not need that compatibility are
/// recommended to use [`HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS`] instead.
pub const HB_BUFFER_CLUSTER_LEVEL_DEFAULT: hb_buffer_cluster_level_t =
    HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES;

/// Tests whether a cluster level groups cluster values into monotone order.
///
/// Requires that `level` be a valid [`hb_buffer_cluster_level_t`].
///
/// Since HarfBuzz 11.0.0.
#[inline]
pub const fn HB_BUFFER_CLUSTER_LEVEL_IS_MONOTONE(level: hb_buffer_cluster_level_t) -> bool {
    ((1u32 << (level as u32))
        & ((1u32 << HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES)
            | (1u32 << HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS)))
        != 0
}

/// Tests whether a cluster level groups cluster values by graphemes.
///
/// Requires that `level` be a valid [`hb_buffer_cluster_level_t`].
///
/// Since HarfBuzz 11.0.0.
#[inline]
pub const fn HB_BUFFER_CLUSTER_LEVEL_IS_GRAPHEMES(level: hb_buffer_cluster_level_t) -> bool {
    ((1u32 << (level as u32))
        & ((1u32 << HB_BUFFER_CLUSTER_LEVEL_MONOTONE_GRAPHEMES)
            | (1u32 << HB_BUFFER_CLUSTER_LEVEL_GRAPHEMES)))
        != 0
}

/// Tests whether a cluster level does *not* group cluster values by graphemes.
///
/// Requires that `level` be a valid [`hb_buffer_cluster_level_t`].
///
/// Since HarfBuzz 11.0.0.
#[inline]
pub const fn HB_BUFFER_CLUSTER_LEVEL_IS_CHARACTERS(level: hb_buffer_cluster_level_t) -> bool {
    ((1u32 << (level as u32))
        & ((1u32 << HB_BUFFER_CLUSTER_LEVEL_MONOTONE_CHARACTERS)
            | (1u32 << HB_BUFFER_CLUSTER_LEVEL_CHARACTERS)))
        != 0
}

/// The default code point used to replace invalid characters in a given
/// encoding: U+FFFD REPLACEMENT CHARACTER.
///
/// Since HarfBuzz 0.9.31.
pub const HB_BUFFER_REPLACEMENT_CODEPOINT_DEFAULT: hb_codepoint_t = 0xFFFD;

/// Which pieces of glyph information [`hb_buffer_serialize_glyphs`] writes out.
///
/// These are bit flags. C gives the enumeration no sentinel and no value
/// exceeds `0x7FFFFFFF`, so the underlying type is `int`.
///
/// Since HarfBuzz 0.9.20.
pub type hb_buffer_serialize_flags_t = c_int;

/// Serialize glyph names, clusters, and positions.
pub const HB_BUFFER_SERIALIZE_FLAG_DEFAULT: hb_buffer_serialize_flags_t = 0x00000000;
/// Do not serialize glyph clusters.
pub const HB_BUFFER_SERIALIZE_FLAG_NO_CLUSTERS: hb_buffer_serialize_flags_t = 0x00000001;
/// Do not serialize glyph position information.
pub const HB_BUFFER_SERIALIZE_FLAG_NO_POSITIONS: hb_buffer_serialize_flags_t = 0x00000002;
/// Do not serialize glyph names.
pub const HB_BUFFER_SERIALIZE_FLAG_NO_GLYPH_NAMES: hb_buffer_serialize_flags_t = 0x00000004;
/// Serialize glyph extents.
pub const HB_BUFFER_SERIALIZE_FLAG_GLYPH_EXTENTS: hb_buffer_serialize_flags_t = 0x00000008;

/// Serialize glyph flags.
///
/// Since HarfBuzz 1.5.0.
pub const HB_BUFFER_SERIALIZE_FLAG_GLYPH_FLAGS: hb_buffer_serialize_flags_t = 0x00000010;

/// Do not serialize glyph advances; glyph offsets then reflect absolute glyph
/// positions.
///
/// When this flag is used on a partial range of the buffer — that is, `start`
/// is not zero — computing the absolute positions costs time proportional to
/// `start`. Serializing in many small chunks therefore becomes quadratic; use a
/// larger `buf_size` to keep the cost down.
///
/// Since HarfBuzz 1.8.0.
pub const HB_BUFFER_SERIALIZE_FLAG_NO_ADVANCES: hb_buffer_serialize_flags_t = 0x00000020;

/// The bitwise OR of every currently defined serialization flag.
///
/// Since HarfBuzz 4.4.0.
pub const HB_BUFFER_SERIALIZE_FLAG_DEFINED: hb_buffer_serialize_flags_t = 0x0000003F;

/// The format used by [`hb_buffer_serialize_glyphs`] and
/// [`hb_buffer_deserialize_glyphs`].
///
/// The values are four-byte tags. C gives the enumeration no sentinel and no
/// value exceeds `0x7FFFFFFF`, so the underlying type is `int`.
///
/// Since HarfBuzz 0.9.2.
pub type hb_buffer_serialize_format_t = c_int;

/// A human-readable, plain-text format.
pub const HB_BUFFER_SERIALIZE_FORMAT_TEXT: hb_buffer_serialize_format_t =
    HB_TAG(b'T', b'E', b'X', b'T') as hb_buffer_serialize_format_t;

/// A machine-readable JSON format.
pub const HB_BUFFER_SERIALIZE_FORMAT_JSON: hb_buffer_serialize_format_t =
    HB_TAG(b'J', b'S', b'O', b'N') as hb_buffer_serialize_format_t;

/// An invalid format.
pub const HB_BUFFER_SERIALIZE_FORMAT_INVALID: hb_buffer_serialize_format_t =
    HB_TAG_NONE as hb_buffer_serialize_format_t;

/// The kinds of difference [`hb_buffer_diff`] can report between two buffers.
///
/// These are bit flags. C gives the enumeration no sentinel and no value
/// exceeds `0x7FFFFFFF`, so the underlying type is `int`.
///
/// Buffers with different [`hb_buffer_content_type_t`] cannot be compared in
/// any further detail. For buffers of differing length the per-glyph comparison
/// is skipped, although the reference buffer is still scanned for dotted-circle
/// and `.notdef` glyphs. When the lengths match, the buffers are compared
/// glyph by glyph and each differing aspect is reported.
///
/// Since HarfBuzz 1.5.0.
pub type hb_buffer_diff_flags_t = c_int;

/// The buffers are equal.
pub const HB_BUFFER_DIFF_FLAG_EQUAL: hb_buffer_diff_flags_t = 0x0000;

/// The buffers have different [`hb_buffer_content_type_t`].
pub const HB_BUFFER_DIFF_FLAG_CONTENT_TYPE_MISMATCH: hb_buffer_diff_flags_t = 0x0001;
/// The buffers have differing lengths.
pub const HB_BUFFER_DIFF_FLAG_LENGTH_MISMATCH: hb_buffer_diff_flags_t = 0x0002;

/// The `.notdef` glyph is present in the reference buffer.
pub const HB_BUFFER_DIFF_FLAG_NOTDEF_PRESENT: hb_buffer_diff_flags_t = 0x0004;
/// The dotted-circle glyph is present in the reference buffer.
pub const HB_BUFFER_DIFF_FLAG_DOTTED_CIRCLE_PRESENT: hb_buffer_diff_flags_t = 0x0008;

/// The buffers differ in [`hb_glyph_info_t::codepoint`].
pub const HB_BUFFER_DIFF_FLAG_CODEPOINT_MISMATCH: hb_buffer_diff_flags_t = 0x0010;
/// The buffers differ in [`hb_glyph_info_t::cluster`].
pub const HB_BUFFER_DIFF_FLAG_CLUSTER_MISMATCH: hb_buffer_diff_flags_t = 0x0020;
/// The buffers differ in [`hb_glyph_flags_t`].
pub const HB_BUFFER_DIFF_FLAG_GLYPH_FLAGS_MISMATCH: hb_buffer_diff_flags_t = 0x0040;
/// The buffers differ in [`hb_glyph_position_t`].
pub const HB_BUFFER_DIFF_FLAG_POSITION_MISMATCH: hb_buffer_diff_flags_t = 0x0080;

/// A tracing callback invoked at each step of the shaping process.
///
/// It is called with the buffer it was set on, the font the buffer is being
/// shaped with, and a NUL-terminated message describing the step that is about
/// to be performed. Return `true` to perform the step, `false` to skip it and
/// move on to the next one.
///
/// Since HarfBuzz 1.1.3.
pub type hb_buffer_message_func_t = Option<
    unsafe extern "C" fn(
        buffer: *mut hb_buffer_t,
        font: *mut hb_font_t,
        message: *const c_char,
        user_data: *mut c_void,
    ) -> hb_bool_t,
>;

unsafe extern "C" {
    /// Returns the [`hb_glyph_flags_t`] encoded in a glyph's private mask.
    ///
    /// In C this name is also a function-like macro that reads `info->mask`
    /// directly; the exported function is what Rust binds to.
    pub fn hb_glyph_info_get_glyph_flags(info: *const hb_glyph_info_t) -> hb_glyph_flags_t;

    /// Checks whether two [`hb_segment_properties_t`] are equal.
    pub fn hb_segment_properties_equal(
        a: *const hb_segment_properties_t,
        b: *const hb_segment_properties_t,
    ) -> hb_bool_t;

    /// Creates a hash of an [`hb_segment_properties_t`] suitable for use as a
    /// map key.
    pub fn hb_segment_properties_hash(p: *const hb_segment_properties_t) -> c_uint;

    /// Fills in the missing fields of `p` from `src`, in a considered manner.
    ///
    /// First, if `p` has no direction set, the direction is copied from `src`.
    /// Next, if `p` and `src` now have the same direction — which may be unset
    /// — and `p` has no script set, the script is copied. Finally, if `p` and
    /// `src` have the same direction and script and `p` has no language set,
    /// the language is copied.
    ///
    /// Since HarfBuzz 3.3.0.
    pub fn hb_segment_properties_overlay(
        p: *mut hb_segment_properties_t,
        src: *const hb_segment_properties_t,
    );

    /// Creates a new [`hb_buffer_t`] with every property at its default, and a
    /// reference count of one that the caller must release with
    /// [`hb_buffer_destroy`].
    ///
    /// Never returns null: if memory cannot be allocated, a special buffer is
    /// returned for which [`hb_buffer_allocation_successful`] reports false.
    pub fn hb_buffer_create() -> *mut hb_buffer_t;

    /// Creates a new buffer like [`hb_buffer_create`] does, except that it is
    /// configured similarly to `src`. The contents of `src` are not copied.
    ///
    /// Since HarfBuzz 3.3.0.
    pub fn hb_buffer_create_similar(src: *const hb_buffer_t) -> *mut hb_buffer_t;

    /// Resets the buffer to its initial status, as if it had just been created
    /// by [`hb_buffer_create`].
    pub fn hb_buffer_reset(buffer: *mut hb_buffer_t);

    /// Fetches the immortal empty buffer, which ignores every modification.
    ///
    /// The returned reference is owned by the caller, as with
    /// [`hb_buffer_create`].
    pub fn hb_buffer_get_empty() -> *mut hb_buffer_t;

    /// Increases the reference count on a buffer and returns it.
    pub fn hb_buffer_reference(buffer: *mut hb_buffer_t) -> *mut hb_buffer_t;

    /// Decreases the reference count on a buffer, destroying it and its
    /// contents when the count reaches zero.
    pub fn hb_buffer_destroy(buffer: *mut hb_buffer_t);

    /// Attaches a user-data item to a buffer, keyed by the address of `key`.
    pub fn hb_buffer_set_user_data(
        buffer: *mut hb_buffer_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user-data item attached to a buffer under `key`.
    pub fn hb_buffer_get_user_data(
        buffer: *const hb_buffer_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Sets the kind of content the buffer holds.
    ///
    /// You rarely need this: the other functions transition the content type
    /// for you. A new buffer starts out
    /// [`HB_BUFFER_CONTENT_TYPE_INVALID`], and [`hb_buffer_reset`],
    /// [`hb_buffer_clear_contents`], and [`hb_buffer_set_length`] with zero all
    /// return it to that state. The `hb_buffer_add_*` functions require the
    /// buffer to be either empty and invalid or already
    /// [`HB_BUFFER_CONTENT_TYPE_UNICODE`], and set it to Unicode when they add
    /// to an empty buffer. `hb_shape` requires the same and sets the type to
    /// [`HB_BUFFER_CONTENT_TYPE_GLYPHS`] on success. The transitions are
    /// designed so that a "reset, add text, shape" loop never has to touch the
    /// content type by hand.
    pub fn hb_buffer_set_content_type(
        buffer: *mut hb_buffer_t,
        content_type: hb_buffer_content_type_t,
    );

    /// Fetches the kind of content the buffer holds.
    pub fn hb_buffer_get_content_type(buffer: *const hb_buffer_t) -> hb_buffer_content_type_t;

    /// Sets the Unicode-functions structure the buffer will use to look up
    /// character properties.
    pub fn hb_buffer_set_unicode_funcs(
        buffer: *mut hb_buffer_t,
        unicode_funcs: *mut hb_unicode_funcs_t,
    );

    /// Fetches the Unicode-functions structure attached to the buffer.
    pub fn hb_buffer_get_unicode_funcs(buffer: *const hb_buffer_t) -> *mut hb_unicode_funcs_t;

    /// Sets the text flow direction of the buffer.
    pub fn hb_buffer_set_direction(buffer: *mut hb_buffer_t, direction: hb_direction_t);

    /// Fetches the text flow direction of the buffer.
    pub fn hb_buffer_get_direction(buffer: *const hb_buffer_t) -> hb_direction_t;

    /// Sets the script of the buffer.
    pub fn hb_buffer_set_script(buffer: *mut hb_buffer_t, script: hb_script_t);

    /// Fetches the script of the buffer.
    pub fn hb_buffer_get_script(buffer: *const hb_buffer_t) -> hb_script_t;

    /// Sets the language of the buffer.
    pub fn hb_buffer_set_language(buffer: *mut hb_buffer_t, language: hb_language_t);

    /// Fetches the language of the buffer.
    pub fn hb_buffer_get_language(buffer: *const hb_buffer_t) -> hb_language_t;

    /// Sets the buffer's direction, script, and language in one call.
    pub fn hb_buffer_set_segment_properties(
        buffer: *mut hb_buffer_t,
        props: *const hb_segment_properties_t,
    );

    /// Fetches the buffer's direction, script, and language in one call.
    pub fn hb_buffer_get_segment_properties(
        buffer: *const hb_buffer_t,
        props: *mut hb_segment_properties_t,
    );

    /// Sets any unset segment properties from the buffer's Unicode contents.
    /// If the buffer is not empty it must have content type
    /// [`HB_BUFFER_CONTENT_TYPE_UNICODE`].
    ///
    /// An unset script becomes the script of the first character whose script
    /// is not Common, Inherited, or Unknown. An unset direction then becomes
    /// the natural horizontal direction of that script, falling back to
    /// [`HB_DIRECTION_LTR`](crate::HB_DIRECTION_LTR). An unset language finally
    /// becomes the process default from
    /// [`hb_language_get_default`](crate::hb_language_get_default), which is
    /// not thread-safe the first time it is called.
    pub fn hb_buffer_guess_segment_properties(buffer: *mut hb_buffer_t);

    /// Sets the buffer's [`hb_buffer_flags_t`].
    pub fn hb_buffer_set_flags(buffer: *mut hb_buffer_t, flags: hb_buffer_flags_t);

    /// Fetches the buffer's [`hb_buffer_flags_t`].
    pub fn hb_buffer_get_flags(buffer: *const hb_buffer_t) -> hb_buffer_flags_t;

    /// Sets the cluster level, which controls how cluster values are grouped.
    pub fn hb_buffer_set_cluster_level(
        buffer: *mut hb_buffer_t,
        cluster_level: hb_buffer_cluster_level_t,
    );

    /// Fetches the buffer's cluster level.
    pub fn hb_buffer_get_cluster_level(buffer: *const hb_buffer_t) -> hb_buffer_cluster_level_t;

    /// Sets the code point substituted for invalid input, as used by the
    /// `hb_buffer_add_utf*` family.
    pub fn hb_buffer_set_replacement_codepoint(
        buffer: *mut hb_buffer_t,
        replacement: hb_codepoint_t,
    );

    /// Fetches the code point substituted for invalid input.
    pub fn hb_buffer_get_replacement_codepoint(buffer: *const hb_buffer_t) -> hb_codepoint_t;

    /// Sets the glyph that replaces invisible characters in the shaping result.
    ///
    /// Zero, the default, means the glyph for U+0020 SPACE is used; any other
    /// value is used verbatim.
    ///
    /// Since HarfBuzz 2.0.0.
    pub fn hb_buffer_set_invisible_glyph(buffer: *mut hb_buffer_t, invisible: hb_codepoint_t);

    /// Fetches the glyph that replaces invisible characters.
    pub fn hb_buffer_get_invisible_glyph(buffer: *const hb_buffer_t) -> hb_codepoint_t;

    /// Sets the glyph that replaces characters the font has no glyph for.
    ///
    /// It defaults to zero, the `.notdef` glyph; setting it lets you tell a
    /// genuine `.notdef` in the font apart from a lookup failure.
    ///
    /// Since HarfBuzz 3.1.0.
    pub fn hb_buffer_set_not_found_glyph(buffer: *mut hb_buffer_t, not_found: hb_codepoint_t);

    /// Fetches the glyph that replaces characters the font has no glyph for.
    pub fn hb_buffer_get_not_found_glyph(buffer: *const hb_buffer_t) -> hb_codepoint_t;

    /// Sets the glyph that replaces variation-selector characters the font does
    /// not resolve.
    ///
    /// The default is
    /// [`HB_CODEPOINT_INVALID`](crate::HB_CODEPOINT_INVALID), which removes an
    /// unresolved variation selector from the glyph string entirely. Setting a
    /// real glyph retains it instead, so that the client can detect the
    /// situation and react — by trying a different font, for instance.
    ///
    /// Since HarfBuzz 10.0.0.
    pub fn hb_buffer_set_not_found_variation_selector_glyph(
        buffer: *mut hb_buffer_t,
        not_found_variation_selector: hb_codepoint_t,
    );

    /// Fetches the glyph used for an unresolved variation selector.
    pub fn hb_buffer_get_not_found_variation_selector_glyph(
        buffer: *const hb_buffer_t,
    ) -> hb_codepoint_t;

    /// Sets the buffer's random state, which advances every time a glyph uses
    /// randomness — the OpenType `rand` feature, for example.
    ///
    /// Together with [`hb_buffer_get_random_state`] this lets you carry the
    /// state over to the next buffer for a better randomness distribution. It
    /// defaults to one, including when the buffer contents are cleared; a value
    /// of zero disables randomness during shaping.
    ///
    /// Since HarfBuzz 8.4.0.
    pub fn hb_buffer_set_random_state(buffer: *mut hb_buffer_t, state: c_uint);

    /// Fetches the buffer's random state.
    ///
    /// Since HarfBuzz 8.4.0.
    pub fn hb_buffer_get_random_state(buffer: *const hb_buffer_t) -> c_uint;

    /// Like [`hb_buffer_reset`], but keeps the Unicode functions and the
    /// replacement code point.
    pub fn hb_buffer_clear_contents(buffer: *mut hb_buffer_t);

    /// Pre-allocates memory for at least `size` items. Returns true when the
    /// allocation succeeded.
    pub fn hb_buffer_pre_allocate(buffer: *mut hb_buffer_t, size: c_uint) -> hb_bool_t;

    /// Reports whether every memory allocation the buffer has made succeeded.
    pub fn hb_buffer_allocation_successful(buffer: *mut hb_buffer_t) -> hb_bool_t;

    /// Reverses the buffer's contents.
    pub fn hb_buffer_reverse(buffer: *mut hb_buffer_t);

    /// Reverses the buffer's contents in the range `start` (inclusive) to `end`
    /// (exclusive).
    pub fn hb_buffer_reverse_range(buffer: *mut hb_buffer_t, start: c_uint, end: c_uint);

    /// Reverses the buffer's contents, then reverses each cluster — each run of
    /// consecutive items sharing a cluster number — again.
    pub fn hb_buffer_reverse_clusters(buffer: *mut hb_buffer_t);

    /// Appends a single code point to the buffer with the given cluster value.
    pub fn hb_buffer_add(buffer: *mut hb_buffer_t, codepoint: hb_codepoint_t, cluster: c_uint);

    /// Appends UTF-8 text to the buffer, replacing invalid sequences with the
    /// buffer's replacement code point — see
    /// [`hb_buffer_set_replacement_codepoint`].
    ///
    /// Pass `text_length` as `-1` when `text` is NUL-terminated, and
    /// `item_length` as `-1` to take everything from `item_offset` to the end.
    /// Text outside the `item_offset`/`item_length` window is still used as
    /// context for shaping; see [`hb_buffer_add_codepoints`].
    pub fn hb_buffer_add_utf8(
        buffer: *mut hb_buffer_t,
        text: *const c_char,
        text_length: c_int,
        item_offset: c_uint,
        item_length: c_int,
    );

    /// Appends UTF-16 text to the buffer, following the same conventions as
    /// [`hb_buffer_add_utf8`].
    pub fn hb_buffer_add_utf16(
        buffer: *mut hb_buffer_t,
        text: *const u16,
        text_length: c_int,
        item_offset: c_uint,
        item_length: c_int,
    );

    /// Appends UTF-32 text to the buffer, following the same conventions as
    /// [`hb_buffer_add_utf8`].
    pub fn hb_buffer_add_utf32(
        buffer: *mut hb_buffer_t,
        text: *const u32,
        text_length: c_int,
        item_offset: c_uint,
        item_length: c_int,
    );

    /// Appends Latin-1 (ISO-8859-1) text to the buffer, following the same
    /// conventions as [`hb_buffer_add_utf8`].
    pub fn hb_buffer_add_latin1(
        buffer: *mut hb_buffer_t,
        text: *const u8,
        text_length: c_int,
        item_offset: c_uint,
        item_length: c_int,
    );

    /// Appends already-decoded Unicode code points to the buffer.
    ///
    /// `item_offset` is the index of the first code point to append and
    /// `item_length` how many to append, or `-1` for the rest of `text`. When
    /// shaping part of a larger text — a run inside a paragraph, say — it is
    /// better to pass the whole paragraph and delimit the run with
    /// `item_offset` and `item_length` than to pass only the substring: that
    /// gives HarfBuzz the full context it needs for cross-run Arabic shaping
    /// and for combining marks at the start of a run.
    ///
    /// This function does not validate `text`; the caller must ensure it holds
    /// valid Unicode scalar values. [`hb_buffer_add_utf32`] takes the same kind
    /// of input but sanity-checks it.
    ///
    /// Since HarfBuzz 0.9.31.
    pub fn hb_buffer_add_codepoints(
        buffer: *mut hb_buffer_t,
        text: *const hb_codepoint_t,
        text_length: c_int,
        item_offset: c_uint,
        item_length: c_int,
    );

    /// Appends part of the contents of another buffer to this one.
    ///
    /// Use `0` for `start` to copy from the beginning of `source`, and
    /// `c_uint::MAX` for `end` to copy through to its end.
    ///
    /// Since HarfBuzz 1.5.0.
    pub fn hb_buffer_append(
        buffer: *mut hb_buffer_t,
        source: *const hb_buffer_t,
        start: c_uint,
        end: c_uint,
    );

    /// Sets the number of items in the buffer. Like [`hb_buffer_pre_allocate`],
    /// but any new items added at the end are cleared. Returns true when the
    /// allocation succeeded.
    pub fn hb_buffer_set_length(buffer: *mut hb_buffer_t, length: c_uint) -> hb_bool_t;

    /// Fetches the number of items in the buffer.
    pub fn hb_buffer_get_length(buffer: *const hb_buffer_t) -> c_uint;

    /// Returns the buffer's glyph-information array and, through `length`, its
    /// item count.
    ///
    /// The array belongs to the buffer and stays valid until the buffer is
    /// modified or destroyed.
    pub fn hb_buffer_get_glyph_infos(
        buffer: *mut hb_buffer_t,
        length: *mut c_uint,
    ) -> *mut hb_glyph_info_t;

    /// Returns the buffer's glyph-position array and, through `length`, its
    /// item count.
    ///
    /// The array belongs to the buffer and stays valid until the buffer is
    /// modified or destroyed.
    pub fn hb_buffer_get_glyph_positions(
        buffer: *mut hb_buffer_t,
        length: *mut c_uint,
    ) -> *mut hb_glyph_position_t;

    /// Reports whether the buffer holds glyph position data.
    ///
    /// A buffer gains position data when [`hb_buffer_get_glyph_positions`] is
    /// called on it, and loses it again on [`hb_buffer_clear_contents`].
    ///
    /// Since HarfBuzz 2.7.3.
    pub fn hb_buffer_has_positions(buffer: *mut hb_buffer_t) -> hb_bool_t;

    /// Reorders the buffer so that glyph order and positions within each
    /// cluster are canonical. The reordered clusters behave identically to the
    /// originals.
    ///
    /// This has nothing to do with Unicode normalization.
    pub fn hb_buffer_normalize_glyphs(buffer: *mut hb_buffer_t);

    /// Parses a string into an [`hb_buffer_serialize_format_t`], such as
    /// `"text"` or `"json"`.
    ///
    /// Pass `len` as `-1` when `str_` is NUL-terminated. The string is not
    /// checked against the supported formats; use
    /// [`hb_buffer_serialize_list_formats`] for that.
    pub fn hb_buffer_serialize_format_from_string(
        str_: *const c_char,
        len: c_int,
    ) -> hb_buffer_serialize_format_t;

    /// Converts a serialization format to its NUL-terminated name, or null when
    /// `format` is not valid. The string must not be freed.
    pub fn hb_buffer_serialize_format_to_string(
        format: hb_buffer_serialize_format_t,
    ) -> *const c_char;

    /// Returns a NULL-terminated array of the supported serialization format
    /// names. The array must not be freed.
    pub fn hb_buffer_serialize_list_formats() -> *mut *const c_char;

    /// Serializes the buffer's glyph content between `start` and `end` into
    /// `buf` as text, which is useful for inspecting a buffer while debugging.
    ///
    /// Returns the number of items serialized, and writes the number of bytes
    /// put into `buf` through `buf_consumed` unless that is null. `font`
    /// supplies glyph names and extents; passing null uses an empty font.
    pub fn hb_buffer_serialize_glyphs(
        buffer: *mut hb_buffer_t,
        start: c_uint,
        end: c_uint,
        buf: *mut c_char,
        buf_size: c_uint,
        buf_consumed: *mut c_uint,
        font: *mut hb_font_t,
        format: hb_buffer_serialize_format_t,
        flags: hb_buffer_serialize_flags_t,
    ) -> c_uint;

    /// Serializes the buffer's Unicode content — that is, before shaping —
    /// between `start` and `end` into `buf`, in the same manner as
    /// [`hb_buffer_serialize_glyphs`]. Returns the number of items serialized.
    ///
    /// Since HarfBuzz 2.7.3.
    pub fn hb_buffer_serialize_unicode(
        buffer: *mut hb_buffer_t,
        start: c_uint,
        end: c_uint,
        buf: *mut c_char,
        buf_size: c_uint,
        buf_consumed: *mut c_uint,
        format: hb_buffer_serialize_format_t,
        flags: hb_buffer_serialize_flags_t,
    ) -> c_uint;

    /// Serializes the buffer whatever it holds, dispatching to
    /// [`hb_buffer_serialize_glyphs`] or [`hb_buffer_serialize_unicode`]
    /// according to its content type. Returns the number of items serialized.
    ///
    /// Since HarfBuzz 2.7.3.
    pub fn hb_buffer_serialize(
        buffer: *mut hb_buffer_t,
        start: c_uint,
        end: c_uint,
        buf: *mut c_char,
        buf_size: c_uint,
        buf_consumed: *mut c_uint,
        font: *mut hb_font_t,
        format: hb_buffer_serialize_format_t,
        flags: hb_buffer_serialize_flags_t,
    ) -> c_uint;

    /// Parses glyphs into `buffer` from the textual representation produced by
    /// [`hb_buffer_serialize_glyphs`]. Returns true when the whole string was
    /// parsed.
    ///
    /// Pass `buf_len` as `-1` when `buf` is NUL-terminated. Unless `end_ptr` is
    /// null it receives a pointer to the character after the last one consumed.
    /// `font`, used to look up glyph IDs, may be null.
    pub fn hb_buffer_deserialize_glyphs(
        buffer: *mut hb_buffer_t,
        buf: *const c_char,
        buf_len: c_int,
        end_ptr: *mut *const c_char,
        font: *mut hb_font_t,
        format: hb_buffer_serialize_format_t,
    ) -> hb_bool_t;

    /// Parses Unicode text into `buffer` from the textual representation
    /// produced by [`hb_buffer_serialize_unicode`], in the same manner as
    /// [`hb_buffer_deserialize_glyphs`].
    ///
    /// Since HarfBuzz 2.7.3.
    pub fn hb_buffer_deserialize_unicode(
        buffer: *mut hb_buffer_t,
        buf: *const c_char,
        buf_len: c_int,
        end_ptr: *mut *const c_char,
        format: hb_buffer_serialize_format_t,
    ) -> hb_bool_t;

    /// Compares the contents of two buffers and reports the kinds of difference
    /// found.
    ///
    /// `dottedcircle_glyph` is the glyph ID of U+25CC DOTTED CIRCLE, and
    /// `position_fuzz` the allowed absolute difference in position values.
    /// Passing `(hb_codepoint_t) -1` — that is,
    /// [`HB_CODEPOINT_INVALID`](crate::HB_CODEPOINT_INVALID) — for
    /// `dottedcircle_glyph` suppresses
    /// [`HB_BUFFER_DIFF_FLAG_DOTTED_CIRCLE_PRESENT`] and
    /// [`HB_BUFFER_DIFF_FLAG_NOTDEF_PRESENT`] entirely, which is what most
    /// callers who only want to compare two buffers should do.
    ///
    /// Since HarfBuzz 1.5.0.
    pub fn hb_buffer_diff(
        buffer: *mut hb_buffer_t,
        reference: *mut hb_buffer_t,
        dottedcircle_glyph: hb_codepoint_t,
        position_fuzz: c_uint,
    ) -> hb_buffer_diff_flags_t;

    /// Installs the [`hb_buffer_message_func_t`] implementation for this
    /// buffer.
    ///
    /// `user_data` may be null, as may `destroy`; `destroy` is called on
    /// `user_data` once it is no longer needed.
    ///
    /// Since HarfBuzz 1.1.3.
    pub fn hb_buffer_set_message_func(
        buffer: *mut hb_buffer_t,
        func: hb_buffer_message_func_t,
        user_data: *mut c_void,
        destroy: hb_destroy_func_t,
    );

    /// Called by a message callback after it has modified the buffer's glyph
    /// indices, to update HarfBuzz's internal caches.
    ///
    /// Does nothing when called from outside a message callback.
    ///
    /// Since HarfBuzz 13.0.0.
    pub fn hb_buffer_changed(buffer: *mut hb_buffer_t);
}
