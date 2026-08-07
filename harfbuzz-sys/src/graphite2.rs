//! Graphite2 integration — `hb-graphite2.h`.
//!
//! [Graphite](http://graphite.sil.org/) is SIL International's "smart font"
//! technology. Where OpenType describes shaping as a fixed pipeline of
//! substitution and positioning lookups, Graphite compiles a small rule
//! *program* into the font — written in GDL, the Graphite Description
//! Language — and a Graphite engine executes that program over the text. This
//! makes it possible to support writing systems whose behaviour OpenType has no
//! model for, which is why Graphite fonts are common for minority and
//! lesser-documented scripts.
//!
//! A Graphite font carries extra tables alongside its ordinary OpenType ones:
//! `Silf` (the compiled rules), `Glat` and `Gloc` (glyph attributes and their
//! index), `Feat` (the feature declarations), and optionally `Sill` (language
//! defaults). HarfBuzz does not implement Graphite itself — it delegates to
//! SIL's `libgraphite2`, which is why this is an optional back end rather than
//! part of the core.
//!
//! # Shaping happens automatically
//!
//! There is nothing in this header you have to call to get Graphite shaping.
//! When HarfBuzz is built with Graphite support, `graphite2` is registered
//! *ahead of* the native `ot` shaper in the default shaper list, and
//! [`hb_shape`](crate::hb_shape) picks it for any face whose
//! [`HB_GRAPHITE2_TAG_SILF`] table is present and non-empty. A face without a
//! `Silf` table silently falls through to the OpenType shaper, which is the
//! desired behaviour: enabling this back end never changes how a non-Graphite
//! font shapes.
//!
//! To go the other way and *refuse* Graphite for a particular call, pass an
//! explicit shaper list to [`hb_shape_full`](crate::hb_shape_full) — `["ot",
//! NULL]` — or reorder the defaults process-wide with the `HB_SHAPER_LIST`
//! environment variable.
//!
//! # What this header is for
//!
//! Given that, the header's job is narrow: it hands you the underlying
//! `libgraphite2` object so you can ask Graphite questions HarfBuzz does not
//! expose — enumerating a font's Graphite features with `gr_face_n_fref` and
//! `gr_face_fref`, listing its languages, or building your own `gr_segment`.
//! That is what [`hb_graphite2_face_get_gr_face`] is for.
//!
//! One caveat about features: HarfBuzz maps each [`hb_feature_t`] you pass to
//! shaping onto a Graphite feature reference by tag, and ignores tags the font
//! does not declare. Graphite features are named by the font designer and are
//! not drawn from the OpenType registry, so the tags that work here are
//! whatever the font declares in its `Feat` table — discovering them is the
//! main reason to reach for the `gr_face`.
//!
//! # Building
//!
//! The sources behind these declarations are compiled only when this crate's
//! `graphite2` feature is enabled, which also requires the system `graphite2`
//! library to be discoverable through `pkg-config`. That same feature gates
//! this module, so a declaration here can never refer to a symbol that was left
//! out of the archive. Because of the gating the module is reached as
//! `harfbuzz_sys::graphite2` rather than being glob re-exported at the crate
//! root.
//!
//! # The `gr_face` and `gr_font` types
//!
//! `hb-graphite2.h` includes `<graphite2/Font.h>` to get `gr_face` and
//! `gr_font`. Those declarations belong to `libgraphite2`, not to HarfBuzz, and
//! this crate has no `graphite2` dependency to import them from — so both are
//! declared here as what they are in C: opaque structs used only behind
//! pointers. They are ABI-identical to `libgraphite2`'s own types, and a handle
//! from a Graphite binding crate converts with a pointer cast:
//!
//! ```ignore
//! let gr = unsafe { harfbuzz_sys::graphite2::hb_graphite2_face_get_gr_face(face) };
//! let theirs = gr.cast::<their_crate::gr_face>();
//! ```
//!
//! [`hb_feature_t`]: crate::hb_feature_t

use crate::{HB_TAG, hb_face_t, hb_font_t, hb_tag_t};

// ---------------------------------------------------------------------------
// Foreign types
//
// `gr_face` and `gr_font` belong to SIL's `libgraphite2`, not to HarfBuzz, and
// this crate has no dependency that supplies them. They are declared here so
// that the signatures below can be written honestly. Both are opaque structs in
// C too, so these are layout- and ABI-compatible with the real definitions in
// `<graphite2/Font.h>`.
// ---------------------------------------------------------------------------

crate::opaque_handle! {
    /// Graphite's face object, spelled `typedef struct gr_face gr_face` in
    /// `<graphite2/Font.h>`.
    ///
    /// This is `libgraphite2`'s counterpart of [`hb_face_t`]: a typeface with
    /// its compiled Graphite rule tables loaded, and no size attached. It is
    /// the object every `gr_face_*` call and `gr_make_seg` takes.
    ///
    /// This crate never looks inside it. A `gr_face` obtained from
    /// [`hb_graphite2_face_get_gr_face`] is owned by the [`hb_face_t`] it came
    /// from — do not pass it to `gr_face_destroy`.
    gr_face
}

crate::opaque_handle! {
    /// Graphite's font object, spelled `typedef struct gr_font gr_font` in
    /// `<graphite2/Font.h>`.
    ///
    /// This is `libgraphite2`'s counterpart of [`hb_font_t`]: a [`gr_face`] at
    /// a particular pixels-per-em size, used to scale Graphite's design-unit
    /// output.
    ///
    /// HarfBuzz no longer creates one — it shapes with a null `gr_font` and
    /// applies its own scaling — so the only function here that mentions this
    /// type, [`hb_graphite2_font_get_gr_font`], is deprecated and always
    /// returns null.
    gr_font
}

// ---------------------------------------------------------------------------
// Table tags
// ---------------------------------------------------------------------------

/// The tag for the `Silf` table, which holds a font's compiled Graphite rules.
///
/// `Silf` is the table HarfBuzz tests when deciding whether a face can be
/// shaped by Graphite: a face whose `Silf` table is missing or zero-length gets
/// no Graphite back end at all, and shapes with the OpenType shaper instead.
///
/// Fetch it like any other table, with
/// [`hb_face_reference_table`](crate::hb_face_reference_table).
///
/// For more information, see <http://graphite.sil.org/>.
pub const HB_GRAPHITE2_TAG_SILF: hb_tag_t = HB_TAG(b'S', b'i', b'l', b'f');

// ---------------------------------------------------------------------------
// Functions
// ---------------------------------------------------------------------------

unsafe extern "C" {
    /// Fetches the Graphite `gr_face` corresponding to a face object.
    ///
    /// The `gr_face` is created on first use and cached on the face: HarfBuzz
    /// builds it with `gr_make_face_with_ops`, backed by a table-fetch callback
    /// that reads through [`hb_face_reference_table`] and keeps a reference to
    /// every blob it hands out. Creation fails — and this returns null — when
    /// the face has no [`HB_GRAPHITE2_TAG_SILF`] table, when that table is
    /// empty, when `libgraphite2` rejects the font, or when allocation fails.
    /// A null return is therefore the normal answer for an ordinary OpenType
    /// font, not an error to report.
    ///
    /// The returned `gr_face` is owned by `face` and lives exactly as long as
    /// it does. Do not call `gr_face_destroy` on it, and do not use it after
    /// the last reference to `face` is released. Because the Graphite tables
    /// are borrowed from the face's blobs rather than copied, the face must
    /// outlive any `gr_segment` you build from the result.
    ///
    /// The lazy creation goes through an atomic compare-and-exchange, so two
    /// threads calling this on the same face at the same time is safe: one of
    /// the two candidate `gr_face` objects is cached and returned to both.
    ///
    /// Since HarfBuzz 0.9.10.
    ///
    /// [`hb_face_reference_table`]: crate::hb_face_reference_table
    pub fn hb_graphite2_face_get_gr_face(face: *mut hb_face_t) -> *mut gr_face;

    /// Always returns null. Use [`hb_graphite2_face_get_gr_face`] instead.
    ///
    /// HarfBuzz used to build a per-size `gr_font` for each [`hb_font_t`] and
    /// hand it to `gr_make_seg`. Since HarfBuzz 1.4.2 it shapes with a null
    /// `gr_font` and scales Graphite's design-unit output itself, using the
    /// font's `x_scale`, `y_scale`, and the face's units-per-em — so there is
    /// no `gr_font` left to return. The symbol is retained for binary
    /// compatibility, and returning null is its entire implementation.
    ///
    /// The `font` argument is ignored, including a null one.
    ///
    /// Since HarfBuzz 0.9.10. Deprecated since HarfBuzz 1.4.2.
    #[deprecated(note = "deprecated in HarfBuzz 1.4.2 and always returns null; \
                         use hb_graphite2_face_get_gr_face instead")]
    pub fn hb_graphite2_font_get_gr_font(font: *mut hb_font_t) -> *mut gr_font;
}
