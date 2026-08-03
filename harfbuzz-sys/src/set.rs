//! Objects representing a set of integers — `hb-set.h`.
//!
//! Sets are used across the non-shaping API to query which characters, glyphs,
//! or other discrete values a font or table covers.

use core::ffi::{c_uint, c_void};

use crate::{HB_CODEPOINT_INVALID, hb_bool_t, hb_codepoint_t, hb_destroy_func_t, hb_user_data_key_t};

/// An unset [`hb_set_t`] value.
///
/// Doubles as the "start here" sentinel for the iteration functions
/// ([`hb_set_next`], [`hb_set_previous`], and their range variants).
///
/// Since HarfBuzz 0.9.21.
pub const HB_SET_VALUE_INVALID: hb_codepoint_t = HB_CODEPOINT_INVALID;

crate::opaque_handle! {
    /// Data type for holding a set of integers.
    ///
    /// Sets gather and contain glyph IDs, Unicode code points, and various
    /// other collections of discrete values. They are reference-counted: create
    /// one with [`hb_set_create`], share it with [`hb_set_reference`], and
    /// release your share with [`hb_set_destroy`].
    hb_set_t
}

unsafe extern "C" {
    /// Creates a new, initially empty set.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_create() -> *mut hb_set_t;

    /// Fetches the singleton empty set.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_get_empty() -> *mut hb_set_t;

    /// Increases the reference count on a set and returns it.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_reference(set: *mut hb_set_t) -> *mut hb_set_t;

    /// Decreases the reference count on a set.
    ///
    /// When the reference count reaches zero the set is destroyed, freeing all
    /// its memory.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_destroy(set: *mut hb_set_t);

    /// Attaches a user-data key/data pair to the specified set.
    ///
    /// Returns true on success.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_set_user_data(
        set: *mut hb_set_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to the specified set under the given key.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_get_user_data(
        set: *const hb_set_t,
        key: *mut hb_user_data_key_t,
    ) -> *mut c_void;

    /// Tests whether memory allocation for a set was successful.
    ///
    /// Returns false if any allocation has failed before, in which case the
    /// set's contents are not to be trusted.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_allocation_successful(set: *const hb_set_t) -> hb_bool_t;

    /// Allocates a copy of a set.
    ///
    /// Since HarfBuzz 2.8.2.
    pub fn hb_set_copy(set: *const hb_set_t) -> *mut hb_set_t;

    /// Clears out the contents of a set.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_clear(set: *mut hb_set_t);

    /// Tests whether a set is empty — that is, contains no elements.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_set_is_empty(set: *const hb_set_t) -> hb_bool_t;

    /// Inverts the contents of a set.
    ///
    /// Since HarfBuzz 3.0.0.
    pub fn hb_set_invert(set: *mut hb_set_t);

    /// Returns whether the set is inverted.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_set_is_inverted(set: *const hb_set_t) -> hb_bool_t;

    /// Tests whether `codepoint` belongs to the set.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_has(set: *const hb_set_t, codepoint: hb_codepoint_t) -> hb_bool_t;

    /// Adds `codepoint` to the set.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_add(set: *mut hb_set_t, codepoint: hb_codepoint_t);

    /// Adds all of the elements from `first` to `last`, inclusive, to the set.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_set_add_range(set: *mut hb_set_t, first: hb_codepoint_t, last: hb_codepoint_t);

    /// Adds `num_codepoints` code points to a set at once.
    ///
    /// The array must be in increasing order and have at least
    /// `num_codepoints` elements.
    ///
    /// Since HarfBuzz 4.1.0.
    pub fn hb_set_add_sorted_array(
        set: *mut hb_set_t,
        sorted_codepoints: *const hb_codepoint_t,
        num_codepoints: c_uint,
    );

    /// Removes `codepoint` from the set.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_del(set: *mut hb_set_t, codepoint: hb_codepoint_t);

    /// Removes all of the elements from `first` to `last`, inclusive, from the
    /// set.
    ///
    /// If `last` is [`HB_SET_VALUE_INVALID`], every value greater than or equal
    /// to `first` is removed.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_set_del_range(set: *mut hb_set_t, first: hb_codepoint_t, last: hb_codepoint_t);

    /// Tests whether two sets are equal — that is, contain the same elements.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_set_is_equal(set: *const hb_set_t, other: *const hb_set_t) -> hb_bool_t;

    /// Creates a hash representing the set.
    ///
    /// Since HarfBuzz 4.4.0.
    pub fn hb_set_hash(set: *const hb_set_t) -> c_uint;

    /// Tests whether `set` is a subset of `larger_set`, or equal to it.
    ///
    /// Since HarfBuzz 1.8.1.
    pub fn hb_set_is_subset(set: *const hb_set_t, larger_set: *const hb_set_t) -> hb_bool_t;

    /// Makes the contents of `set` equal to the contents of `other`.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_set(set: *mut hb_set_t, other: *const hb_set_t);

    /// Makes `set` the union of `set` and `other`.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_union(set: *mut hb_set_t, other: *const hb_set_t);

    /// Makes `set` the intersection of `set` and `other`.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_intersect(set: *mut hb_set_t, other: *const hb_set_t);

    /// Subtracts the contents of `other` from `set`.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_subtract(set: *mut hb_set_t, other: *const hb_set_t);

    /// Makes `set` the symmetric difference of `set` and `other`.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_symmetric_difference(set: *mut hb_set_t, other: *const hb_set_t);

    /// Returns the number of elements in the set.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_set_get_population(set: *const hb_set_t) -> c_uint;

    /// Finds the smallest element in the set.
    ///
    /// Returns [`HB_SET_VALUE_INVALID`] if the set is empty.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_set_get_min(set: *const hb_set_t) -> hb_codepoint_t;

    /// Finds the largest element in the set.
    ///
    /// Returns [`HB_SET_VALUE_INVALID`] if the set is empty.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_set_get_max(set: *const hb_set_t) -> hb_codepoint_t;

    /// Fetches the next element in the set that is greater than the current
    /// value of `codepoint`, writing it back through the same pointer.
    ///
    /// Pass [`HB_SET_VALUE_INVALID`] in to get started. Returns true if there
    /// was a next value.
    ///
    /// Since HarfBuzz 0.9.2.
    pub fn hb_set_next(set: *const hb_set_t, codepoint: *mut hb_codepoint_t) -> hb_bool_t;

    /// Fetches the previous element in the set that is lower than the current
    /// value of `codepoint`, writing it back through the same pointer.
    ///
    /// Pass [`HB_SET_VALUE_INVALID`] in to get started. Returns true if there
    /// was a previous value.
    ///
    /// Since HarfBuzz 1.8.0.
    pub fn hb_set_previous(set: *const hb_set_t, codepoint: *mut hb_codepoint_t) -> hb_bool_t;

    /// Fetches the next consecutive range of elements in the set that are
    /// greater than the current value of `last`.
    ///
    /// `first` is written out; `last` is read and written. Pass
    /// [`HB_SET_VALUE_INVALID`] for both to get started. Returns true if there
    /// was a next range.
    ///
    /// Since HarfBuzz 0.9.7.
    pub fn hb_set_next_range(
        set: *const hb_set_t,
        first: *mut hb_codepoint_t,
        last: *mut hb_codepoint_t,
    ) -> hb_bool_t;

    /// Fetches the previous consecutive range of elements in the set that are
    /// lower than the current value of `first`.
    ///
    /// `first` is read and written; `last` is written out. Pass
    /// [`HB_SET_VALUE_INVALID`] for both to get started. Returns true if there
    /// was a previous range.
    ///
    /// Since HarfBuzz 1.8.0.
    pub fn hb_set_previous_range(
        set: *const hb_set_t,
        first: *mut hb_codepoint_t,
        last: *mut hb_codepoint_t,
    ) -> hb_bool_t;

    /// Finds the elements of the set greater than `codepoint` and writes them
    /// into `out`, stopping when the set runs out of elements or `size` values
    /// have been written, whichever comes first.
    ///
    /// Pass [`HB_SET_VALUE_INVALID`] as `codepoint` to get started. Returns the
    /// number of values written.
    ///
    /// Since HarfBuzz 4.2.0.
    pub fn hb_set_next_many(
        set: *const hb_set_t,
        codepoint: hb_codepoint_t,
        out: *mut hb_codepoint_t,
        size: c_uint,
    ) -> c_uint;
}
