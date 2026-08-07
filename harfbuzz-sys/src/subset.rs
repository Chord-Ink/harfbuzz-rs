//! Font subsetting and instancing — `hb-subset.h`, `hb-subset-serialize.h`, `hb-subset-depend.h`.
//!
//! Subsetting reduces the codepoint coverage of a font file and removes all
//! data that is no longer needed. You describe the subset you want in an
//! [`hb_subset_input_t`], hand it to [`hb_subset_or_fail`] together with a
//! source [`hb_face_t`], and get back a new face whose blob is the subsetted
//! font file.
//!
//! Most outline and bitmap tables are supported: `glyf`, `CFF`, `CFF2`, `sbix`,
//! `COLR`, and `CBDT`/`CBLC` — including variable outlines through OpenType
//! variations. `EBDT`/`EBLC` and `SVG` are not. Layout subsetting covers the
//! OpenType Layout tables (`GSUB`, `GPOS`, `GDEF`) only; Graphite and AAT
//! tables are not subsetted. A font carrying Graphite or AAT tables can still
//! be run through the subsetter, but you will normally want to drop those
//! tables, since they would otherwise refer to glyphs that no longer exist.
//!
//! The same machinery performs *instancing*: pinning one or more variation axes
//! to fixed values with [`hb_subset_input_pin_axis_location`], or narrowing an
//! axis with [`hb_subset_input_set_axis_range`], produces a font with a smaller
//! — possibly empty — variation space.
//!
//! Three levels of API are exposed here, from convenient to fine-grained:
//!
//! * [`hb_subset_or_fail`] — one call: source face in, subset face out.
//! * [`hb_subset_plan_create_or_fail`] plus [`hb_subset_plan_execute_or_fail`]
//!   — build the plan separately, so that the old-to-new glyph mapping can be
//!   inspected both before and after the subset font is produced.
//! * [`hb_subset_serialize_or_fail`] — the object-graph repacker on its own,
//!   for callers that build an OpenType table themselves and need HarfBuzz to
//!   resolve offset overflows.
//!
//! [`hb_subset_depend_from_face_or_fail`] and its companions expose the glyph
//! dependency graph the subsetter derives from a face: which glyphs pull in
//! which other glyphs through `GSUB`, composite `glyf` outlines, `CFF` seacs,
//! `COLR` layers, and `MATH` variants.
//!
//! This module needs the crate's `subset` feature, which compiles the HarfBuzz
//! subsetting sources into the archive. Without it the module does not exist.
//! Individual items marked below additionally need the `experimental` feature,
//! which defines `HB_EXPERIMENTAL_API` upstream; the symbols behind it carry no
//! compatibility promise.

use core::ffi::{c_char, c_float, c_int, c_uint, c_void};

use crate::{hb_blob_t, hb_face_t, hb_map_t, hb_set_t};
use crate::{hb_bool_t, hb_codepoint_t, hb_destroy_func_t, hb_tag_t, hb_user_data_key_t};

#[cfg(feature = "experimental")]
use crate::hb_ot_name_id_t;

// ---------------------------------------------------------------------------
// hb-subset.h — objects
// ---------------------------------------------------------------------------

crate::opaque_handle! {
    /// Things that change based on the input. Characters to keep, etc.
    ///
    /// A reference-counted description of the subset you want: the Unicode
    /// codepoints and glyph IDs to retain, the tables to drop or pass through,
    /// the `name` records to keep, the variation axes to pin, and the boolean
    /// [`hb_subset_flags_t`] settings. Create one with
    /// [`hb_subset_input_create_or_fail`] and release it with
    /// [`hb_subset_input_destroy`].
    hb_subset_input_t
}

crate::opaque_handle! {
    /// Contains information about how the subset operation will be executed,
    /// such as mappings from the old glyph IDs to the new ones in the subset.
    ///
    /// A plan is an [`hb_subset_input_t`] resolved against a particular
    /// [`hb_face_t`]: which tables and glyphs survive, and how old glyph IDs map
    /// onto new ones. Create one with [`hb_subset_plan_create_or_fail`], run it
    /// with [`hb_subset_plan_execute_or_fail`], and release it with
    /// [`hb_subset_plan_destroy`].
    hb_subset_plan_t
}

// ---------------------------------------------------------------------------
// hb-subset.h — enumerations
// ---------------------------------------------------------------------------

/// List of boolean properties that can be configured on the subset input.
///
/// These are bit flags, combined with bitwise OR and installed with
/// [`hb_subset_input_set_flags`]. The C enumeration has no sentinel and no
/// value exceeds `0x7FFFFFFF`, so the underlying type is `int`.
///
/// Since HarfBuzz 2.9.0.
pub type hb_subset_flags_t = c_int;

/// All flags at their default value of false.
pub const HB_SUBSET_FLAGS_DEFAULT: hb_subset_flags_t = 0x0000_0000;

/// If set, hinting instructions will be dropped in the produced subset.
/// Otherwise hinting instructions will be retained.
pub const HB_SUBSET_FLAGS_NO_HINTING: hb_subset_flags_t = 0x0000_0001;

/// If set, glyph indices will not be modified in the produced subset. If glyphs
/// are dropped their indices will be retained as an empty glyph.
pub const HB_SUBSET_FLAGS_RETAIN_GIDS: hb_subset_flags_t = 0x0000_0002;

/// If set and subsetting a CFF font, the subsetter will attempt to remove
/// subroutines from the CFF glyphs.
pub const HB_SUBSET_FLAGS_DESUBROUTINIZE: hb_subset_flags_t = 0x0000_0004;

/// If set, non-Unicode `name` records will be retained in the subset.
pub const HB_SUBSET_FLAGS_NAME_LEGACY: hb_subset_flags_t = 0x0000_0008;

/// If set, the subsetter will set the `OVERLAP_SIMPLE` flag on each simple
/// glyph.
pub const HB_SUBSET_FLAGS_SET_OVERLAPS_FLAG: hb_subset_flags_t = 0x0000_0010;

/// If set, the subsetter will not drop unrecognized tables and instead pass
/// them through untouched.
pub const HB_SUBSET_FLAGS_PASSTHROUGH_UNRECOGNIZED: hb_subset_flags_t = 0x0000_0020;

/// If set, the `.notdef` glyph outline will be retained in the final subset.
pub const HB_SUBSET_FLAGS_NOTDEF_OUTLINE: hb_subset_flags_t = 0x0000_0040;

/// If set, the PostScript glyph names will be retained in the final subset.
pub const HB_SUBSET_FLAGS_GLYPH_NAMES: hb_subset_flags_t = 0x0000_0080;

/// If set, the Unicode ranges in `OS/2` will not be recalculated.
pub const HB_SUBSET_FLAGS_NO_PRUNE_UNICODE_RANGES: hb_subset_flags_t = 0x0000_0100;

/// If set, do not perform glyph closure on layout substitution rules (`GSUB`).
///
/// Since HarfBuzz 7.2.0.
pub const HB_SUBSET_FLAGS_NO_LAYOUT_CLOSURE: hb_subset_flags_t = 0x0000_0200;

/// If set, perform IUP delta optimization on the remaining `gvar` table's
/// deltas.
///
/// Since HarfBuzz 8.5.0.
pub const HB_SUBSET_FLAGS_OPTIMIZE_IUP_DELTAS: hb_subset_flags_t = 0x0000_0400;

/// If set, do not pull mirrored versions of input codepoints into the subset.
///
/// Since HarfBuzz 11.1.0.
pub const HB_SUBSET_FLAGS_NO_BIDI_CLOSURE: hb_subset_flags_t = 0x0000_0800;

/// If set, enforce requirements on the output subset to allow it to be used
/// with incremental font transfer IFTB patches. Primarily, this forces all
/// outline data to use long (32-bit) offsets.
///
/// Needs the `experimental` feature; upstream marks it `Since: EXPERIMENTAL`.
#[cfg(feature = "experimental")]
pub const HB_SUBSET_FLAGS_IFTB_REQUIREMENTS: hb_subset_flags_t = 0x0000_1000;

/// If this flag is set alongside [`HB_SUBSET_FLAGS_RETAIN_GIDS`], the number of
/// glyphs in the font will not be reduced as a result of subsetting. If
/// necessary, empty glyphs will be included at the end of the font to keep the
/// glyph count unchanged.
///
/// Needs the `experimental` feature; upstream marks it `Since: EXPERIMENTAL`.
#[cfg(feature = "experimental")]
pub const HB_SUBSET_FLAGS_RETAIN_NUM_GLYPHS: hb_subset_flags_t = 0x0000_2000;

/// If set and instantiating a variable font with all axes pinned, convert the
/// output `CFF2` table to `CFF`. This enables compatibility with older
/// renderers that do not support `CFF2`.
///
/// Since HarfBuzz 13.0.0.
pub const HB_SUBSET_FLAGS_DOWNGRADE_CFF2: hb_subset_flags_t = 0x0000_4000;

/// If set and subsetting a CID-keyed CFF font, the output CFF charset will use
/// sequential identity CIDs (CID = new GID) rather than preserving the original
/// CIDs.
///
/// Since HarfBuzz 14.3.0.
pub const HB_SUBSET_FLAGS_CFF_IDENTITY_CHARSET: hb_subset_flags_t = 0x0000_8000;

/// List of sets that can be configured on the subset input.
///
/// Each value selects one of the [`hb_set_t`] collections an
/// [`hb_subset_input_t`] carries; retrieve the set with [`hb_subset_input_set`]
/// and edit it in place. The C enumeration has no sentinel and its largest
/// enumerator is 7, so it fits in an `int`.
///
/// Since HarfBuzz 2.9.1.
pub type hb_subset_sets_t = c_int;

/// The set of glyph indexes to retain in the subset.
pub const HB_SUBSET_SETS_GLYPH_INDEX: hb_subset_sets_t = 0;

/// The set of Unicode codepoints to retain in the subset.
pub const HB_SUBSET_SETS_UNICODE: hb_subset_sets_t = 1;

/// The set of table tags which specifies tables that should not be subsetted.
pub const HB_SUBSET_SETS_NO_SUBSET_TABLE_TAG: hb_subset_sets_t = 2;

/// The set of table tags which specifies tables which will be dropped in the
/// subset.
pub const HB_SUBSET_SETS_DROP_TABLE_TAG: hb_subset_sets_t = 3;

/// The set of `name` IDs that will be retained.
pub const HB_SUBSET_SETS_NAME_ID: hb_subset_sets_t = 4;

/// The set of `name` language IDs that will be retained.
pub const HB_SUBSET_SETS_NAME_LANG_ID: hb_subset_sets_t = 5;

/// The set of layout feature tags that will be retained in the subset.
pub const HB_SUBSET_SETS_LAYOUT_FEATURE_TAG: hb_subset_sets_t = 6;

/// The set of layout script tags that will be retained in the subset. Defaults
/// to all tags.
///
/// Since HarfBuzz 5.0.0.
pub const HB_SUBSET_SETS_LAYOUT_SCRIPT_TAG: hb_subset_sets_t = 7;

// ---------------------------------------------------------------------------
// hb-subset-serialize.h — object graph
// ---------------------------------------------------------------------------

/// Represents a link between two objects in the object graph to be serialized.
///
/// A link records where, inside one object's bytes, an offset to another object
/// lives.
///
/// Since HarfBuzz 10.2.0.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_subset_serialize_link_t {
    /// `offsetSize` in bytes.
    pub width: c_uint,
    /// Position of the offset field in bytes from the beginning of the
    /// subtable.
    pub position: c_uint,
    /// Index of the subtable this link points at, into the object array passed
    /// to [`hb_subset_serialize_or_fail`].
    pub objidx: c_uint,
}

/// Represents an object in the object graph to be serialized.
///
/// The object's own bytes are the half-open range `head .. tail`. Real links
/// are offset fields that live inside those bytes; virtual links write nothing
/// and only constrain packing order.
///
/// Since HarfBuzz 10.2.0.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct hb_subset_serialize_object_t {
    /// Start of object data.
    pub head: *mut c_char,
    /// End of object data.
    pub tail: *mut c_char,
    /// Number of offset fields in the object.
    pub num_real_links: c_uint,
    /// Array of offset info.
    pub real_links: *mut hb_subset_serialize_link_t,
    /// Number of objects that must be packed after the current object in the
    /// final serialized order.
    pub num_virtual_links: c_uint,
    /// Array of virtual link info.
    pub virtual_links: *mut hb_subset_serialize_link_t,
}

// ---------------------------------------------------------------------------
// hb-subset-depend.h — glyph dependency graph
// ---------------------------------------------------------------------------

/// Flags on dependency edges returned by [`hb_subset_depend_lookup_glyph`].
///
/// They mark edges which may produce expected over-approximation when computing
/// closure via the depend graph, relative to
/// [`hb_ot_layout_lookups_substitute_closure`](crate::hb_ot_layout_lookups_substitute_closure).
/// These flags help distinguish known limitations of static dependency analysis
/// (expected over-approximation) from bugs (unexpected over-approximation).
///
/// These are bit flags. The C enumeration has no sentinel and its largest
/// enumerator is 2, so it fits in an `int`.
///
/// Since HarfBuzz 14.3.0.
pub type hb_subset_depend_edge_flags_t = c_int;

/// No flags set.
pub const HB_SUBSET_DEPEND_EDGE_FLAG_NONE: hb_subset_depend_edge_flags_t = 0x00;

/// Edge from a multi-position contextual rule (`Context` or `ChainContext` with
/// `inputCount > 1`).
///
/// Depend extraction records edges based on what glyphs could statically be at
/// each position according to input coverage or class. However, at runtime the
/// lookups within the rule are applied sequentially: a lookup at an earlier
/// position may transform the glyph at a later position, and two lookups at the
/// same position may interact such that one produces a glyph that another
/// immediately consumes as an "intermediate". A glyph that matches the static
/// coverage may therefore not persist at that position when the rule actually
/// fires, so this edge may not trigger during closure.
pub const HB_SUBSET_DEPEND_EDGE_FLAG_FROM_CONTEXT_POSITION: hb_subset_depend_edge_flags_t = 0x01;

/// Edge from a lookup invoked within another contextual lookup.
///
/// The outer context's requirements are not propagated to this edge, so the
/// edge may fire even when those requirements are not met.
pub const HB_SUBSET_DEPEND_EDGE_FLAG_FROM_NESTED_CONTEXT: hb_subset_depend_edge_flags_t = 0x02;

crate::opaque_handle! {
    /// Data type for holding glyph dependency graphs.
    ///
    /// Built from a face with [`hb_subset_depend_from_face_or_fail`] and
    /// released with [`hb_subset_depend_destroy`].
    ///
    /// **Highly experimental API. Subject to change.**
    ///
    /// Since HarfBuzz 14.3.0.
    hb_subset_depend_t
}

/// A single dependency edge returned by [`hb_subset_depend_lookup_glyph`].
///
/// Since HarfBuzz 14.3.0.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct hb_subset_depend_entry_t {
    /// Source table — `GSUB`, `glyf`, `CFF`, `COLR`, or `MATH`.
    pub table_tag: hb_tag_t,
    /// Target glyph ID.
    pub dependent: hb_codepoint_t,
    /// Feature tag for `GSUB` edges; zero otherwise.
    pub layout_tag: hb_tag_t,
    /// Index into the sets array for ligature component glyphs, or
    /// [`HB_CODEPOINT_INVALID`](crate::HB_CODEPOINT_INVALID) if this is not a
    /// ligature edge.
    pub ligature_set_index: hb_codepoint_t,
    /// Index into the sets array for context requirement glyphs, or
    /// [`HB_CODEPOINT_INVALID`](crate::HB_CODEPOINT_INVALID) if none. Use
    /// [`hb_subset_depend_lookup_set`] to retrieve the set.
    pub context_set_index: hb_codepoint_t,
    /// Edge flags — see [`hb_subset_depend_edge_flags_t`].
    pub flags: hb_subset_depend_edge_flags_t,
}

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

unsafe extern "C" {
    /// Creates a new subset input object.
    ///
    /// Returns a new subset input, or null on failure. Destroy it with
    /// [`hb_subset_input_destroy`].
    ///
    /// Since HarfBuzz 1.8.0.
    pub fn hb_subset_input_create_or_fail() -> *mut hb_subset_input_t;

    /// Increases the reference count on `input` and returns it.
    ///
    /// Since HarfBuzz 1.8.0.
    pub fn hb_subset_input_reference(input: *mut hb_subset_input_t) -> *mut hb_subset_input_t;

    /// Decreases the reference count on `input`, destroying it and freeing all
    /// its memory when the count reaches zero.
    ///
    /// Since HarfBuzz 1.8.0.
    pub fn hb_subset_input_destroy(input: *mut hb_subset_input_t);

    /// Attaches a user-data key/data pair to the given subset input object.
    ///
    /// `destroy` — which may be null — is called with `data` when the input is
    /// destroyed or the value is replaced. `replace` decides whether existing
    /// data stored under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 2.9.0.
    pub fn hb_subset_input_set_user_data(
        input: *mut hb_subset_input_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to the subset input under the specified
    /// key.
    ///
    /// Ownership stays with the input; the caller must not free the returned
    /// pointer.
    ///
    /// Since HarfBuzz 2.9.0.
    pub fn hb_subset_input_get_user_data(
        input: *const hb_subset_input_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Configures the input object to keep everything in the font face — all
    /// Unicode codepoints, glyphs, names, layout items, glyph names, and so on.
    ///
    /// The input can be tailored afterwards by the caller. This is the natural
    /// starting point when you want to *remove* a few specific things rather
    /// than *select* a few specific things.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_subset_input_keep_everything(input: *mut hb_subset_input_t);

    /// Gets the set of Unicode codepoints to retain; the caller should modify
    /// the set as needed.
    ///
    /// Equivalent to [`hb_subset_input_set`] with [`HB_SUBSET_SETS_UNICODE`].
    /// The set belongs to the input and must not be destroyed by the caller.
    ///
    /// Since HarfBuzz 1.8.0.
    pub fn hb_subset_input_unicode_set(input: *mut hb_subset_input_t) -> *mut hb_set_t;

    /// Gets the set of glyph IDs to retain; the caller should modify the set as
    /// needed.
    ///
    /// Equivalent to [`hb_subset_input_set`] with
    /// [`HB_SUBSET_SETS_GLYPH_INDEX`]. The set belongs to the input and must
    /// not be destroyed by the caller.
    ///
    /// Since HarfBuzz 1.8.0.
    pub fn hb_subset_input_glyph_set(input: *mut hb_subset_input_t) -> *mut hb_set_t;

    /// Gets the set of the specified type.
    ///
    /// The set belongs to the input and must not be destroyed by the caller;
    /// edit it in place with the `hb_set_*` functions.
    ///
    /// Since HarfBuzz 2.9.1.
    pub fn hb_subset_input_set(
        input: *mut hb_subset_input_t,
        set_type: hb_subset_sets_t,
    ) -> *mut hb_set_t;

    /// Returns a map that can be used to provide an explicit mapping from old
    /// to new glyph IDs in the produced subset. The caller should populate the
    /// map as desired.
    ///
    /// If this map is left empty then glyph IDs are assigned automatically by
    /// the subsetter. If populated, the mapping must be unique — no two
    /// original glyph IDs may map to the same new ID — and
    /// [`HB_SUBSET_FLAGS_RETAIN_GIDS`] cannot also be enabled. Any retained
    /// glyphs not named in the mapping are assigned IDs above the highest ID in
    /// the mapping.
    ///
    /// Note that non-monotonic mappings are accepted and applied, but may
    /// result in unsorted `Coverage` tables, which some consumers (OTS, for
    /// one) reject. Prefer a monotonic mapping where possible.
    ///
    /// The map belongs to the input and must not be destroyed by the caller.
    ///
    /// Since HarfBuzz 7.3.0.
    pub fn hb_subset_input_old_to_new_glyph_mapping(input: *mut hb_subset_input_t)
    -> *mut hb_map_t;

    /// Gets all of the subsetting flags in the input object, as a bit field of
    /// [`hb_subset_flags_t`] values.
    ///
    /// Since HarfBuzz 2.9.0.
    pub fn hb_subset_input_get_flags(input: *mut hb_subset_input_t) -> hb_subset_flags_t;

    /// Sets all of the subsetting flags in the input object at once.
    ///
    /// This replaces the whole bit field rather than merging into it; combine
    /// [`hb_subset_flags_t`] values with bitwise OR. Note that the parameter is
    /// an `unsigned`, while [`hb_subset_input_get_flags`] returns the
    /// enumeration type.
    ///
    /// Since HarfBuzz 2.9.0.
    pub fn hb_subset_input_set_flags(input: *mut hb_subset_input_t, value: c_uint);

    /// Pins all variation axes to their default locations in the given subset
    /// input object.
    ///
    /// The `CFF2` table, if present, will be de-subroutinized.
    ///
    /// Returns true on success, false otherwise — including when `face` has no
    /// variation axes at all.
    ///
    /// Since HarfBuzz 8.3.1.
    pub fn hb_subset_input_pin_all_axes_to_default(
        input: *mut hb_subset_input_t,
        face: *mut hb_face_t,
    ) -> hb_bool_t;

    /// Pins one variation axis to its default location in the given subset
    /// input object.
    ///
    /// The `CFF2` table, if present, will be de-subroutinized.
    ///
    /// Returns true on success, false otherwise — in particular when `face` has
    /// no axis with tag `axis_tag`.
    ///
    /// Since HarfBuzz 6.0.0.
    pub fn hb_subset_input_pin_axis_to_default(
        input: *mut hb_subset_input_t,
        face: *mut hb_face_t,
        axis_tag: hb_tag_t,
    ) -> hb_bool_t;

    /// Pins one variation axis to a fixed location in the given subset input
    /// object.
    ///
    /// `axis_value` is a user-space design coordinate and is clamped to the
    /// axis's `fvar` minimum and maximum. The `CFF2` table, if present, will be
    /// de-subroutinized.
    ///
    /// Returns true on success, false otherwise — in particular when `face` has
    /// no axis with tag `axis_tag`.
    ///
    /// Since HarfBuzz 6.0.0.
    pub fn hb_subset_input_pin_axis_location(
        input: *mut hb_subset_input_t,
        face: *mut hb_face_t,
        axis_tag: hb_tag_t,
        axis_value: c_float,
    ) -> hb_bool_t;

    /// Gets the axis range assigned by previous calls to
    /// [`hb_subset_input_set_axis_range`].
    ///
    /// The three out-parameters receive the configured minimum, maximum, and
    /// default. They are written only when the function returns true.
    ///
    /// Returns true if a range has been set for this axis tag, false otherwise.
    ///
    /// Since HarfBuzz 8.5.0.
    pub fn hb_subset_input_get_axis_range(
        input: *mut hb_subset_input_t,
        axis_tag: hb_tag_t,
        axis_min_value: *mut c_float,
        axis_max_value: *mut c_float,
        axis_def_value: *mut c_float,
    ) -> hb_bool_t;

    /// Restricts the range of variation on an axis in the given subset input
    /// object.
    ///
    /// Passing NaN for any of the three values keeps the corresponding value
    /// from the face's `fvar` axis. New minimum, default, and maximum values
    /// are clamped into the `fvar` axis range. If the `fvar` default value
    /// falls outside the new range, the new default becomes whichever of the
    /// new minimum or maximum is closer to it.
    ///
    /// The input minimum may not be greater than the input maximum; if the
    /// input default falls outside the new minimum/maximum range it is clamped.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 8.5.0.
    pub fn hb_subset_input_set_axis_range(
        input: *mut hb_subset_input_t,
        face: *mut hb_face_t,
        axis_tag: hb_tag_t,
        axis_min_value: c_float,
        axis_max_value: c_float,
        axis_def_value: c_float,
    ) -> hb_bool_t;

    /// Parses a string into a subset axis range — minimum, default, maximum.
    ///
    /// The axis-position string is in the format `min:def:max` or `min:max`; a
    /// bare value sets all three; the literal `drop` sets all three to NaN.
    /// Empty components mean "keep the existing value for that part", so
    /// `:300:500` specifies minimum = existing, default = 300, maximum = 500.
    /// In the output, any value that should take its default is set to NaN.
    ///
    /// Pass `len` as `-1` when `str_` is NUL-terminated.
    ///
    /// Returns true if `str_` is successfully parsed, false otherwise.
    ///
    /// Since HarfBuzz 10.2.0.
    pub fn hb_subset_axis_range_from_string(
        str_: *const c_char,
        len: c_int,
        axis_min_value: *mut c_float,
        axis_max_value: *mut c_float,
        axis_def_value: *mut c_float,
    ) -> hb_bool_t;

    /// Converts an axis range into a NUL-terminated string in the format
    /// understood by [`hb_subset_axis_range_from_string`].
    ///
    /// The caller is responsible for allocating a big enough `buf`; 128 bytes
    /// is more than enough. Nothing is written when `size` is zero or when no
    /// range has been configured for `axis_tag`.
    ///
    /// Since HarfBuzz 10.2.0.
    pub fn hb_subset_axis_range_to_string(
        input: *mut hb_subset_input_t,
        axis_tag: hb_tag_t,
        buf: *mut c_char,
        size: c_uint,
    );

    /// Produces a command-line string representation of the given subset input,
    /// suitable for use with the `hb-subset` command-line tool.
    ///
    /// Returns a new blob containing the command-line string, or null on
    /// failure. Destroy it with [`hb_blob_destroy`](crate::hb_blob_destroy).
    ///
    /// Needs the `experimental` feature; upstream marks it
    /// `XSince: EXPERIMENTAL` and makes no compatibility promise.
    #[cfg(feature = "experimental")]
    pub fn hb_subset_input_to_string_or_fail(input: *mut hb_subset_input_t) -> *mut hb_blob_t;

    /// Overrides the name string of the `name` record identified by `name_id`,
    /// `platform_id`, `encoding_id`, and `language_id`. If a record with that
    /// `name_id` does not exist, it is created and inserted into the `name`
    /// table.
    ///
    /// Pass a null `name_str` to indicate the record should be removed, and
    /// `str_len` as `-1` when `name_str` is NUL-terminated.
    ///
    /// For the Macintosh platform (`platform_id` 1) only all-ASCII `name_str`
    /// values are supported; a string containing non-ASCII characters is
    /// ignored and the call returns false.
    ///
    /// Needs the `experimental` feature; upstream marks it
    /// `XSince: EXPERIMENTAL` and makes no compatibility promise.
    #[cfg(feature = "experimental")]
    pub fn hb_subset_input_override_name_table(
        input: *mut hb_subset_input_t,
        name_id: hb_ot_name_id_t,
        platform_id: c_uint,
        encoding_id: c_uint,
        language_id: c_uint,
        name_str: *const c_char,
        str_len: c_int,
    ) -> hb_bool_t;

    /// Returns the raw outline data from the `CFF` table associated with the
    /// given glyph index.
    ///
    /// Needs the `experimental` feature; upstream marks it
    /// `XSince: EXPERIMENTAL` and makes no compatibility promise.
    #[cfg(feature = "experimental")]
    pub fn hb_subset_cff_get_charstring_data(
        face: *mut hb_face_t,
        glyph: hb_codepoint_t,
    ) -> *mut hb_blob_t;

    /// Returns the raw `CharStrings INDEX` from the `CFF` table.
    ///
    /// Needs the `experimental` feature; upstream marks it
    /// `XSince: EXPERIMENTAL` and makes no compatibility promise.
    #[cfg(feature = "experimental")]
    pub fn hb_subset_cff_get_charstrings_index(face: *mut hb_face_t) -> *mut hb_blob_t;

    /// Returns the raw outline data from the `CFF2` table associated with the
    /// given glyph index.
    ///
    /// Needs the `experimental` feature; upstream marks it
    /// `XSince: EXPERIMENTAL` and makes no compatibility promise.
    #[cfg(feature = "experimental")]
    pub fn hb_subset_cff2_get_charstring_data(
        face: *mut hb_face_t,
        glyph: hb_codepoint_t,
    ) -> *mut hb_blob_t;

    /// Returns the raw `CharStrings INDEX` from the `CFF2` table.
    ///
    /// Needs the `experimental` feature; upstream marks it
    /// `XSince: EXPERIMENTAL` and makes no compatibility promise.
    #[cfg(feature = "experimental")]
    pub fn hb_subset_cff2_get_charstrings_index(face: *mut hb_face_t) -> *mut hb_blob_t;

    /// Preprocesses the face and attaches data that will be needed by the
    /// subsetter, so that future subsetting operations can reuse the
    /// precomputed data and run faster.
    ///
    /// Note that the preprocessed face may contain sub-blobs that reference the
    /// memory backing the source face. If that memory is not owned by the
    /// source face, it must live at least as long as the returned face.
    ///
    /// Returns a new face the caller owns; destroy it with
    /// [`hb_face_destroy`](crate::hb_face_destroy). Never null: on failure it
    /// returns a new reference to `source`.
    ///
    /// Since HarfBuzz 6.0.0.
    pub fn hb_subset_preprocess(source: *mut hb_face_t) -> *mut hb_face_t;

    /// Subsets a font according to the provided input.
    ///
    /// Returns a new face the caller owns — destroy it with
    /// [`hb_face_destroy`](crate::hb_face_destroy) — or null if the subset
    /// operation fails, if either argument is null, or if the source face has
    /// no glyphs. Reach the subsetted font-file bytes with
    /// [`hb_face_reference_blob`](crate::hb_face_reference_blob).
    ///
    /// Since HarfBuzz 2.9.0.
    pub fn hb_subset_or_fail(
        source: *mut hb_face_t,
        input: *const hb_subset_input_t,
    ) -> *mut hb_face_t;

    /// Executes the provided subsetting plan.
    ///
    /// Returns a new face the caller owns — destroy it with
    /// [`hb_face_destroy`](crate::hb_face_destroy) — or null if the subsetting
    /// operation fails.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_subset_plan_execute_or_fail(plan: *mut hb_subset_plan_t) -> *mut hb_face_t;

    /// Computes a plan for subsetting `face` according to `input`. The plan
    /// describes which tables and glyphs should be retained.
    ///
    /// Returns a new subset plan the caller owns; destroy it with
    /// [`hb_subset_plan_destroy`]. Returns null if creating the plan fails.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_subset_plan_create_or_fail(
        face: *mut hb_face_t,
        input: *const hb_subset_input_t,
    ) -> *mut hb_subset_plan_t;

    /// Decreases the reference count on `plan`, destroying it and freeing all
    /// its memory when the count reaches zero.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_subset_plan_destroy(plan: *mut hb_subset_plan_t);

    /// Returns the mapping between glyphs in the original font and glyphs in
    /// the subset that will be produced by `plan`.
    ///
    /// The map belongs to the plan and must not be destroyed by the caller.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_subset_plan_old_to_new_glyph_mapping(plan: *const hb_subset_plan_t) -> *mut hb_map_t;

    /// Returns the mapping between glyphs in the subset that will be produced
    /// by `plan` and the glyphs in the original font.
    ///
    /// The map belongs to the plan and must not be destroyed by the caller.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_subset_plan_new_to_old_glyph_mapping(plan: *const hb_subset_plan_t) -> *mut hb_map_t;

    /// Returns the mapping between codepoints in the original font and the
    /// associated glyph ID in the original font.
    ///
    /// The map belongs to the plan and must not be destroyed by the caller.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_subset_plan_unicode_to_old_glyph_mapping(
        plan: *const hb_subset_plan_t,
    ) -> *mut hb_map_t;

    /// Increases the reference count on `plan` and returns it.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_subset_plan_reference(plan: *mut hb_subset_plan_t) -> *mut hb_subset_plan_t;

    /// Attaches a user-data key/data pair to the given subset plan object.
    ///
    /// `destroy` — which may be null — is called with `data` when the plan is
    /// destroyed or the value is replaced. `replace` decides whether existing
    /// data stored under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_subset_plan_set_user_data(
        plan: *mut hb_subset_plan_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to the subset plan under the specified
    /// key.
    ///
    /// Ownership stays with the plan; the caller must not free the returned
    /// pointer.
    ///
    /// Since HarfBuzz 4.0.0.
    pub fn hb_subset_plan_get_user_data(
        plan: *const hb_subset_plan_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Given the input object-graph info, repacks a table to eliminate offset
    /// overflows and serializes it into a continuous array of bytes.
    ///
    /// Table-specific optimizations — extension promotion in `GSUB`/`GPOS`, for
    /// instance — may be performed. Passing
    /// [`HB_TAG_NONE`](crate::HB_TAG_NONE) as `table_tag` disables
    /// table-specific optimizations.
    ///
    /// Returns a new blob holding the serialized table, or null if the
    /// serializing attempt fails. Destroy it with
    /// [`hb_blob_destroy`](crate::hb_blob_destroy).
    ///
    /// Since HarfBuzz 10.2.0.
    pub fn hb_subset_serialize_or_fail(
        table_tag: hb_tag_t,
        hb_objects: *mut hb_subset_serialize_object_t,
        num_hb_objs: c_uint,
    ) -> *mut hb_blob_t;

    /// Calculates the dependencies between glyphs in the supplied face.
    ///
    /// Dependency information is extracted from the `GSUB`, `glyf`, `CFF`,
    /// `COLR`, and `MATH` tables. UVS (Unicode Variation Sequence) dependencies
    /// are *not* included; handle those with
    /// [`hb_font_get_variation_glyph`](crate::hb_font_get_variation_glyph).
    ///
    /// Returns a new depend object the caller owns, or null if creation failed
    /// (out of memory, or an invalid face). Destroy it with
    /// [`hb_subset_depend_destroy`].
    ///
    /// Needs the `experimental` feature: `hb-config.hh` defines
    /// `HB_NO_SUBSET_DEPEND` unless `HB_EXPERIMENTAL_API` is set, so without it
    /// this symbol is not compiled into the library at all.
    ///
    /// `HB_LEAN` defines it a second time, so combining `experimental` with
    /// `lean` or `tiny` leaves this declared but unlinkable.
    ///
    /// Since HarfBuzz 14.3.0.
    #[cfg(feature = "experimental")]
    pub fn hb_subset_depend_from_face_or_fail(face: *mut hb_face_t) -> *mut hb_subset_depend_t;

    /// Retrieves dependency edges for a glyph.
    ///
    /// This follows the standard HarfBuzz array-getter pattern: it always
    /// returns the *total* number of edges for `gid`, regardless of
    /// `start_offset` or `entry_count`. On input `entry_count` holds the number
    /// of entries to fill; on output it holds the number actually filled. Pass
    /// null for `entry_count` — and then also for `entries` — to query the
    /// total count without filling anything.
    ///
    /// Needs the `experimental` feature; see
    /// [`hb_subset_depend_from_face_or_fail`].
    ///
    /// Since HarfBuzz 14.3.0.
    #[cfg(feature = "experimental")]
    pub fn hb_subset_depend_lookup_glyph(
        depend: *mut hb_subset_depend_t,
        gid: hb_codepoint_t,
        start_offset: c_uint,
        entry_count: *mut c_uint,
        entries: *mut hb_subset_depend_entry_t,
    ) -> c_uint;

    /// Gets all glyphs in the set identified by `index`, copying them into
    /// `out`.
    ///
    /// The set index comes from the `ligature_set_index` or `context_set_index`
    /// field of an [`hb_subset_depend_entry_t`] returned by
    /// [`hb_subset_depend_lookup_glyph`]. `out` is a caller-owned set whose
    /// previous contents are replaced.
    ///
    /// Returns true if there is such a set, false otherwise.
    ///
    /// Needs the `experimental` feature; see
    /// [`hb_subset_depend_from_face_or_fail`].
    ///
    /// Since HarfBuzz 14.3.0.
    #[cfg(feature = "experimental")]
    pub fn hb_subset_depend_lookup_set(
        depend: *mut hb_subset_depend_t,
        index: hb_codepoint_t,
        out: *mut hb_set_t,
    ) -> hb_bool_t;

    /// Decreases the reference count on `depend`, destroying it and freeing all
    /// its memory when the count reaches zero.
    ///
    /// Needs the `experimental` feature; see
    /// [`hb_subset_depend_from_face_or_fail`].
    ///
    /// Since HarfBuzz 14.3.0.
    #[cfg(feature = "experimental")]
    pub fn hb_subset_depend_destroy(depend: *mut hb_subset_depend_t);
}
