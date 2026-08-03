//! Integer-to-integer hash maps — `hb-map.h`.

use core::ffi::{c_int, c_uint, c_void};

use crate::{
    HB_CODEPOINT_INVALID, hb_bool_t, hb_codepoint_t, hb_destroy_func_t, hb_set_t,
    hb_user_data_key_t, opaque_handle,
};

/// An unset [`hb_map_t`] value.
///
/// This is the same sentinel as [`HB_CODEPOINT_INVALID`], and it doubles as the
/// "not found" answer: [`hb_map_get`] returns it for a key that is not present.
/// Storing it as a value is therefore indistinguishable from storing nothing —
/// use [`hb_map_has`] when you need to tell the two apart.
///
/// Since HarfBuzz 1.7.7.
pub const HB_MAP_VALUE_INVALID: hb_codepoint_t = HB_CODEPOINT_INVALID;

opaque_handle! {
    /// Data type for holding integer-to-integer hash maps.
    ///
    /// Both keys and values are [`hb_codepoint_t`], so a map is a natural fit
    /// for glyph-to-glyph and codepoint-to-glyph tables. HarfBuzz's own public
    /// API does not currently consume maps; they are exposed for client code
    /// that wants the same container the library uses internally.
    ///
    /// Maps are reference counted. [`hb_map_create`] hands back a reference,
    /// [`hb_map_reference`] takes another, and [`hb_map_destroy`] gives one
    /// back; the object is freed when the last one is released.
    hb_map_t
}

unsafe extern "C" {
    /// Creates a new, initially empty map.
    ///
    /// The caller owns the returned reference and must release it with
    /// [`hb_map_destroy`]. On allocation failure this returns the singleton
    /// empty map rather than null, so the result is always usable — call
    /// [`hb_map_allocation_successful`] if you need to know which you got.
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_create() -> *mut hb_map_t;

    /// Fetches the singleton empty map.
    ///
    /// The returned map is immutable in practice: writes to it are discarded.
    /// It still participates in reference counting, so the caller owns the
    /// returned reference and should release it with [`hb_map_destroy`].
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_get_empty() -> *mut hb_map_t;

    /// Increases the reference count on a map and returns it.
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_reference(map: *mut hb_map_t) -> *mut hb_map_t;

    /// Decreases the reference count on a map.
    ///
    /// When the count reaches zero the map is destroyed and all of its memory
    /// is freed.
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_destroy(map: *mut hb_map_t);

    /// Attaches a user-data key/data pair to a map.
    ///
    /// `destroy` may be `None`; when it is not, it is called with `data` once
    /// the map is destroyed or the entry is replaced. `replace` decides whether
    /// an existing entry under the same key is overwritten.
    ///
    /// Returns true on success, false otherwise.
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_set_user_data(
        map: *mut hb_map_t,
        key: *mut hb_user_data_key_t,
        data: *mut c_void,
        destroy: hb_destroy_func_t,
        replace: hb_bool_t,
    ) -> hb_bool_t;

    /// Fetches the user data attached to a map under the given key.
    ///
    /// The map retains ownership of the returned pointer; do not free it.
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_get_user_data(map: *const hb_map_t, key: *mut hb_user_data_key_t) -> *mut c_void;

    /// Tests whether every memory allocation this map has attempted succeeded.
    ///
    /// Once an allocation fails the map latches into an error state and stops
    /// accepting new entries, so this is the way to distinguish "the key was
    /// never inserted" from "the insertion ran out of memory".
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_allocation_successful(map: *const hb_map_t) -> hb_bool_t;

    /// Allocates a copy of a map.
    ///
    /// The caller owns the returned reference and must release it with
    /// [`hb_map_destroy`]. On allocation failure this returns the singleton
    /// empty map rather than null.
    ///
    /// Since HarfBuzz 4.4.0.
    pub fn hb_map_copy(map: *const hb_map_t) -> *mut hb_map_t;

    /// Clears out the contents of a map, leaving it empty.
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_clear(map: *mut hb_map_t);

    /// Tests whether a map contains no key/value pairs.
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_is_empty(map: *const hb_map_t) -> hb_bool_t;

    /// Returns the number of key/value pairs in a map.
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_get_population(map: *const hb_map_t) -> c_uint;

    /// Tests whether two maps hold the same key/value pairs.
    ///
    /// Since HarfBuzz 4.3.0.
    pub fn hb_map_is_equal(map: *const hb_map_t, other: *const hb_map_t) -> hb_bool_t;

    /// Creates a hash representing a map.
    ///
    /// Equal maps — as judged by [`hb_map_is_equal`] — hash equally, so this
    /// pairs with that function for use as a hash-table key.
    ///
    /// Since HarfBuzz 4.4.0.
    pub fn hb_map_hash(map: *const hb_map_t) -> c_uint;

    /// Stores `value` under `key`, replacing any previous value.
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_set(map: *mut hb_map_t, key: hb_codepoint_t, value: hb_codepoint_t);

    /// Fetches the value stored under `key`.
    ///
    /// Returns [`HB_MAP_VALUE_INVALID`] when the key is not in the map.
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_get(map: *const hb_map_t, key: hb_codepoint_t) -> hb_codepoint_t;

    /// Removes `key` and its stored value from the map.
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_del(map: *mut hb_map_t, key: hb_codepoint_t);

    /// Tests whether `key` is present in the map.
    ///
    /// Since HarfBuzz 1.7.7.
    pub fn hb_map_has(map: *const hb_map_t, key: hb_codepoint_t) -> hb_bool_t;

    /// Adds the contents of `other` to `map`.
    ///
    /// Keys that exist in both take their value from `other`. `other` is only
    /// read, never modified.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_map_update(map: *mut hb_map_t, other: *const hb_map_t);

    /// Fetches the next key/value pair in a map.
    ///
    /// `idx` is the iterator's internal state: set it to `-1` to start, then
    /// pass the same variable back on each call. `key` and `value` are written
    /// only when the call returns true.
    ///
    /// The iteration order is undefined, and modifying the map part-way through
    /// an iteration is undefined behaviour.
    ///
    /// Returns true when a pair was produced, false once the map is exhausted.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_map_next(
        map: *const hb_map_t,
        idx: *mut c_int,
        key: *mut hb_codepoint_t,
        value: *mut hb_codepoint_t,
    ) -> hb_bool_t;

    /// Adds the keys of `map` to the set `keys`.
    ///
    /// The set is added to, not replaced; clear it first if you want only this
    /// map's keys.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_map_keys(map: *const hb_map_t, keys: *mut hb_set_t);

    /// Adds the values of `map` to the set `values`.
    ///
    /// Because a set holds each element once, duplicate values collapse and the
    /// resulting population may be smaller than the map's.
    ///
    /// Since HarfBuzz 7.0.0.
    pub fn hb_map_values(map: *const hb_map_t, values: *mut hb_set_t);
}
