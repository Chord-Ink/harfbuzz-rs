//! Fonts: a face at a particular size and variation setting.

use core::ffi::c_int;

use harfbuzz_sys as sys;

use crate::face::Face;
use crate::feature::Variation;
use crate::object::{HarfBuzzObject, IntoShared, Shared, ThreadSafeWhenImmutable, harfbuzz_object};

harfbuzz_object! {
    /// A [`Face`] with a size, and optionally a set of variation-axis values.
    ///
    /// Where a face describes what a font *contains*, a font describes how it
    /// is being *used*: the scale to report metrics in, the pixels-per-em for
    /// hinting, and where each variable axis is pinned. Shaping needs a font
    /// rather than a face for exactly this reason — advances and offsets are
    /// meaningless without a scale.
    ///
    /// Fonts are cheap next to faces. Building several fonts at different sizes
    /// from one shared face is the intended pattern.
    ///
    /// # Scale and units
    ///
    /// A font's scale converts from the face's design units to the units
    /// shaping reports. Set the scale equal to
    /// [`Face::upem`](crate::Face::upem) — which is what a new font does — and
    /// positions come back in font units. Set it to your point size times 64
    /// and they come back in 26.6 fixed point, the convention FreeType uses.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use harfbuzz_rs::{Face, Font, IntoShared};
    ///
    /// let face = Face::from_file("font.ttf", 0)?.into_shared();
    ///
    /// let mut font = Font::new(face);
    /// font.set_scale(16 * 64, 16 * 64);
    /// # Ok::<(), harfbuzz_rs::Error>(())
    /// ```
    Font,
    raw: sys::hb_font_t,
    reference: sys::hb_font_reference,
    destroy: sys::hb_font_destroy,
}

// SAFETY: a frozen font is safe to shape with from several threads. HarfBuzz's
// own font caches are behind atomics, and upstream's create/configure/freeze
// pattern exists precisely so the result can be shared.
unsafe impl ThreadSafeWhenImmutable for Font {}

impl Font {
    /// Builds a font from a shared face, scaled to the face's design units.
    ///
    /// Taking a [`Shared<Face>`] rather than a `&Face` is deliberate: the font
    /// keeps the face alive for as long as it lives, and requiring a frozen
    /// face makes that sharing sound.
    pub fn new(face: Shared<Face>) -> Self {
        // SAFETY: `face` owns a live, immutable face. `hb_font_create` takes
        // its own reference on it rather than consuming ours, so dropping
        // `face` at the end of this function is correct.
        let raw = unsafe { sys::hb_font_create(face.as_raw()) };

        // SAFETY: `hb_font_create` never returns null — on failure it yields
        // the empty-font singleton — and the reference is ours to own.
        unsafe { Self::from_raw(raw) }
    }

    /// The face this font was built from.
    pub fn face(&self) -> Shared<Face> {
        // SAFETY: `self` owns a live font, so its face is live too.
        let raw = unsafe { sys::hb_font_get_face(self.as_raw()) };

        // The getter returns a borrowed pointer, so take our own reference
        // before wrapping it.
        //
        // SAFETY: `raw` points at the live face owned by this font.
        let raw = unsafe { sys::hb_face_reference(raw) };

        // SAFETY: `raw` now carries a reference this wrapper owns, and a font's
        // face is always immutable — `hb_font_create` freezes it.
        unsafe { Shared::from_immutable(Face::from_raw(raw)) }
    }

    /// The horizontal and vertical scale.
    pub fn scale(&self) -> (i32, i32) {
        let (mut x, mut y) = (0, 0);

        // SAFETY: `self` owns a live font; both out-parameters are live locals.
        unsafe { sys::hb_font_get_scale(self.as_raw(), &mut x, &mut y) };

        (x, y)
    }

    /// Sets the horizontal and vertical scale.
    ///
    /// Shaped positions are reported in these units. See the note on units in
    /// the type's documentation.
    pub fn set_scale(&mut self, x: i32, y: i32) {
        // SAFETY: `self` owns a live font, and `&mut self` proves no other
        // handle can observe the change.
        unsafe { sys::hb_font_set_scale(self.as_raw(), x, y) };
    }

    /// The horizontal and vertical pixels-per-em.
    pub fn ppem(&self) -> (u32, u32) {
        let (mut x, mut y) = (0, 0);

        // SAFETY: `self` owns a live font; both out-parameters are live locals.
        unsafe { sys::hb_font_get_ppem(self.as_raw(), &mut x, &mut y) };

        (x, y)
    }

    /// Sets the horizontal and vertical pixels-per-em.
    ///
    /// This only affects font backends that hint, such as FreeType. Zero, the
    /// default, means unhinted.
    pub fn set_ppem(&mut self, x: u32, y: u32) {
        // SAFETY: as above.
        unsafe { sys::hb_font_set_ppem(self.as_raw(), x, y) };
    }

    /// Pins the font's variation axes.
    ///
    /// Any axis not named keeps its default. Values outside an axis's declared
    /// range are clamped by the font.
    pub fn set_variations(&mut self, variations: &[Variation]) {
        // SAFETY: `self` owns a live font. `Variation` is `#[repr(transparent)]`
        // over `hb_variation_t`, so the slice has the same layout as the array
        // of C structs HarfBuzz expects, and we pass its true length. HarfBuzz
        // copies the values rather than retaining the pointer.
        unsafe {
            sys::hb_font_set_variations(
                self.as_raw(),
                variations.as_ptr().cast(),
                variations.len() as core::ffi::c_uint,
            )
        };
    }

    /// The glyph a code point maps to through the font's character map.
    ///
    /// Returns `None` when the font has no glyph for it. This is the plain
    /// `cmap` lookup — it is *not* shaping, and it will not find glyphs that
    /// only appear through substitution.
    pub fn nominal_glyph(&self, codepoint: char) -> Option<u32> {
        let mut glyph = 0;

        // SAFETY: `self` owns a live font; `glyph` is a live local for
        // HarfBuzz to write through.
        let found =
            unsafe { sys::hb_font_get_nominal_glyph(self.as_raw(), codepoint as u32, &mut glyph) };

        (found != 0).then_some(glyph)
    }

    /// The name the font gives a glyph, if it has one.
    pub fn glyph_name(&self, glyph: u32) -> Option<String> {
        let mut buffer = [0u8; 128];

        // SAFETY: `self` owns a live font; `buffer` is a live local and we
        // pass its true length, which bounds what HarfBuzz writes.
        let found = unsafe {
            sys::hb_font_get_glyph_name(
                self.as_raw(),
                glyph,
                buffer.as_mut_ptr().cast(),
                buffer.len() as core::ffi::c_uint,
            )
        };

        if found == 0 {
            return None;
        }

        let end = buffer.iter().position(|&b| b == 0).unwrap_or(buffer.len());

        String::from_utf8(buffer[..end].to_vec()).ok()
    }

    /// The font's horizontal extents: ascender, descender, and line gap.
    pub fn extents(&self) -> FontExtents {
        let mut extents = sys::hb_font_extents_t {
            ascender: 0,
            descender: 0,
            line_gap: 0,
            reserved9: 0,
            reserved8: 0,
            reserved7: 0,
            reserved6: 0,
            reserved5: 0,
            reserved4: 0,
            reserved3: 0,
            reserved2: 0,
            reserved1: 0,
        };

        // SAFETY: `self` owns a live font; `extents` is a live local. The
        // function leaves it zeroed if the font has no metrics, which is a
        // sensible answer either way.
        unsafe { sys::hb_font_get_h_extents(self.as_raw(), &mut extents) };

        FontExtents {
            ascender: extents.ascender,
            descender: extents.descender,
            line_gap: extents.line_gap,
        }
    }

    /// The bounding box of a glyph, in the font's scaled units.
    ///
    /// Returns `None` when the glyph does not exist or has no outline.
    pub fn glyph_extents(&self, glyph: u32) -> Option<GlyphExtents> {
        let mut extents = sys::hb_glyph_extents_t {
            x_bearing: 0,
            y_bearing: 0,
            width: 0,
            height: 0,
        };

        // SAFETY: `self` owns a live font; `extents` is a live local.
        let found =
            unsafe { sys::hb_font_get_glyph_extents(self.as_raw(), glyph, &mut extents) };

        (found != 0).then_some(GlyphExtents {
            x_bearing: extents.x_bearing,
            y_bearing: extents.y_bearing,
            width: extents.width,
            height: extents.height,
        })
    }

    /// The horizontal advance of a glyph, in the font's scaled units.
    pub fn glyph_h_advance(&self, glyph: u32) -> i32 {
        // SAFETY: `self` owns a live font. An unknown glyph yields a
        // fallback advance rather than an error.
        unsafe { sys::hb_font_get_glyph_h_advance(self.as_raw(), glyph) }
    }

    /// Switches the font to HarfBuzz's own OpenType implementation.
    ///
    /// This is already the default for a newly created font; it is here for
    /// undoing a different backend.
    pub fn set_ot_funcs(&mut self) {
        // SAFETY: `self` owns a live font, and `&mut self` proves exclusivity.
        unsafe { sys::hb_ot_font_set_funcs(self.as_raw()) };
    }
}

impl IntoShared for Font {
    fn into_shared(self) -> Shared<Self> {
        // SAFETY: `self` owns a live font.
        unsafe { sys::hb_font_make_immutable(self.as_raw()) };

        // SAFETY: the font was just frozen, which is what `from_immutable`
        // requires.
        unsafe { Shared::from_immutable(self) }
    }
}

impl std::fmt::Debug for Font {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Font")
            .field("scale", &self.scale())
            .field("ppem", &self.ppem())
            .finish()
    }
}

/// A font's vertical metrics, in the font's scaled units.
///
/// The sign convention is HarfBuzz's: `ascender` is positive above the
/// baseline, `descender` is negative below it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FontExtents {
    /// Distance from the baseline to the top of the tallest glyph.
    pub ascender: i32,
    /// Distance from the baseline to the bottom of the lowest glyph; negative.
    pub descender: i32,
    /// Recommended extra space between consecutive lines.
    pub line_gap: i32,
}

impl FontExtents {
    /// The recommended distance between the baselines of consecutive lines.
    pub fn line_height(self) -> i32 {
        self.ascender - self.descender + self.line_gap
    }
}

/// A glyph's bounding box, in the font's scaled units.
///
/// `height` is negative in the coordinate system HarfBuzz reports, which grows
/// upward: the box starts at `y_bearing` and extends downward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GlyphExtents {
    /// Distance from the origin to the left edge.
    pub x_bearing: i32,
    /// Distance from the origin to the top edge.
    pub y_bearing: i32,
    /// Width of the box.
    pub width: i32,
    /// Height of the box; negative when the coordinate system grows upward.
    pub height: i32,
}

/// A scale expressed in the 26.6 fixed-point convention FreeType uses.
///
/// Multiplying a point size by 64 is such a common way to set a font's scale
/// that it is worth naming.
pub const fn points_to_scale(points: i32) -> c_int {
    points * 64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    #[test]
    fn starts_out_scaled_to_the_faces_design_units() {
        let face = testing::face();
        let upem = face.upem() as i32;

        let font = Font::new(face.into_shared());
        assert_eq!(font.scale(), (upem, upem));
    }

    #[test]
    fn remembers_the_scale_it_was_given() {
        let mut font = testing::font();
        font.set_scale(points_to_scale(16), points_to_scale(16));

        assert_eq!(font.scale(), (1024, 1024));
    }

    #[test]
    fn maps_code_points_through_the_character_map() {
        let font = testing::font();

        // The test font is a subset containing a, b, and c.
        assert!(font.nominal_glyph('A').is_some());
        // And nothing anywhere near this.
        assert_eq!(font.nominal_glyph('\u{10FFFF}'), None);
    }

    #[test]
    fn reports_vertical_metrics_that_make_sense() {
        let extents = testing::font().extents();

        assert!(extents.ascender > 0);
        assert!(extents.descender < 0);
        assert!(extents.line_height() > extents.ascender);
    }

    #[test]
    fn measures_a_real_glyph() {
        let font = testing::font();
        let glyph = font.nominal_glyph('A').unwrap();

        let extents = font.glyph_extents(glyph).expect("`A` has an outline");
        assert!(extents.width > 0);
        assert!(font.glyph_h_advance(glyph) > 0);
    }

    #[test]
    fn keeps_the_face_alive_through_the_font() {
        let font = {
            let face = testing::face().into_shared();
            Font::new(face)
        };

        // The face handle above is gone, but the font holds its own reference.
        assert!(font.face().glyph_count() > 0);
    }

    #[test]
    fn a_shared_font_can_cross_threads() {
        let font = testing::font().into_shared();
        let scale = font.scale();

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let font = font.clone();
                std::thread::spawn(move || font.scale())
            })
            .collect();

        for handle in handles {
            assert_eq!(handle.join().unwrap(), scale);
        }
    }
}
