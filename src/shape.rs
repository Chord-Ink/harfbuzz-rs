//! The shaping call itself.

use harfbuzz_sys as sys;

use crate::buffer::{Buffer, GlyphBuffer};
use crate::feature::Feature;
use crate::font::Font;
use crate::object::HarfBuzzObject;

/// Shapes text: turns the code points in `buffer` into positioned glyphs.
///
/// This is the whole point of the library. The buffer is consumed and handed
/// back as a [`GlyphBuffer`], because its contents are replaced in place and
/// the two states support different operations.
///
/// `features` overrides the font's default feature settings. Pass `&[]` to take
/// the defaults, which is almost always what you want.
///
/// The buffer's direction, script, and language must be set before calling
/// this. If you do not know them, call
/// [`Buffer::guess_segment_properties`] first.
///
/// # Examples
///
/// ```no_run
/// use harfbuzz_rs::{Face, Font, IntoShared, buffer_from, shape};
///
/// let font = Font::new(Face::from_file("font.ttf", 0)?.into_shared());
/// let output = shape(&font, buffer_from("hello")?, &[]);
///
/// let width: i32 = output.positions().iter().map(|p| p.x_advance()).sum();
/// # Ok::<(), harfbuzz_rs::Error>(())
/// ```
pub fn shape(font: &Font, buffer: Buffer, features: &[Feature]) -> GlyphBuffer {
    // SAFETY: `font` and `buffer` both own live objects for the duration of
    // the call. `Feature` is `#[repr(transparent)]` over `hb_feature_t`, so
    // the slice matches the array of C structs HarfBuzz expects, and its true
    // length is passed. HarfBuzz reads the features and does not retain the
    // pointer. Taking `buffer` by value means nothing else can observe it
    // being rewritten.
    unsafe {
        sys::hb_shape(
            font.as_raw(),
            buffer.as_raw(),
            features.as_ptr().cast(),
            features.len() as core::ffi::c_uint,
        )
    };

    buffer.into_glyph_buffer()
}

/// The shapers this build of HarfBuzz can use, in the order it prefers them.
///
/// The list is fixed at compile time. `"ot"` is HarfBuzz's own OpenType
/// implementation and is always present; `"coretext"` and `"graphite2"` appear
/// only when the matching feature was enabled.
pub fn shapers() -> Vec<&'static str> {
    // SAFETY: takes no arguments and returns a static, null-terminated array of
    // static C strings, owned by HarfBuzz for the lifetime of the process.
    let mut list = unsafe { sys::hb_shape_list_shapers() };

    let mut shapers = Vec::new();

    // SAFETY: the array is terminated by a null pointer, so this stops at the
    // end. Each element is a valid NUL-terminated static string.
    unsafe {
        while !(*list).is_null() {
            if let Ok(name) = core::ffi::CStr::from_ptr(*list).to_str() {
                shapers.push(name);
            }
            list = list.add(1);
        }
    }

    shapers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::buffer_from;
    use crate::testing;
    use crate::{Direction, IntoShared, Tag};

    #[test]
    fn turns_text_into_positioned_glyphs() {
        let font = testing::font();
        let output = shape(&font, buffer_from("ABC").unwrap(), &[]);

        assert_eq!(output.len(), 3);
        assert_eq!(output.infos().len(), output.positions().len());

        for (info, position) in output.iter() {
            assert!(info.glyph() > 0, "every character resolved to a glyph");
            assert!(position.x_advance() > 0, "and has a width");
        }
    }

    #[test]
    fn clusters_point_back_at_the_input() {
        let font = testing::font();
        let output = shape(&font, buffer_from("ABC").unwrap(), &[]);

        let clusters: Vec<u32> = output.infos().iter().map(|i| i.cluster()).collect();
        assert_eq!(clusters, vec![0, 1, 2]);
    }

    #[test]
    fn an_empty_buffer_shapes_to_nothing() {
        let font = testing::font();
        let output = shape(&font, buffer_from("").unwrap(), &[]);

        assert!(output.is_empty());
        assert!(output.infos().is_empty());
        assert!(output.positions().is_empty());
    }

    #[test]
    fn features_are_accepted_and_change_nothing_illegal() {
        let font = testing::font();
        let features = [Feature::new(Tag::new(b"kern"), 0, ..)];

        let output = shape(&font, buffer_from("ABC").unwrap(), &features);
        assert_eq!(output.len(), 3);
    }

    #[test]
    fn a_buffer_can_be_reused_after_shaping() {
        let font = testing::font();

        let first = shape(&font, buffer_from("ABC").unwrap(), &[]);
        assert_eq!(first.len(), 3);

        let mut buffer = first.clear();
        assert!(buffer.is_empty());

        buffer.push_str("AB");
        buffer.guess_segment_properties();

        let second = shape(&font, buffer, &[]);
        assert_eq!(second.len(), 2);
    }

    #[test]
    fn right_to_left_output_comes_back_in_visual_order() {
        let font = testing::font();

        let mut buffer = Buffer::new();
        buffer.push_str("ABC");
        buffer.set_direction(Direction::RightToLeft);
        buffer.guess_segment_properties();

        let output = shape(&font, buffer, &[]);

        // HarfBuzz emits RTL runs in visual order, so the clusters descend.
        let clusters: Vec<u32> = output.infos().iter().map(|i| i.cluster()).collect();
        assert_eq!(clusters, vec![2, 1, 0]);
    }

    #[test]
    fn the_opentype_shaper_is_always_available() {
        assert!(shapers().contains(&"ot"));
    }

    #[test]
    fn scale_changes_the_advances_proportionally() {
        let face = testing::face().into_shared();
        let upem = face.glyph_count(); // just to touch the face
        let _ = upem;

        let mut small = crate::Font::new(face.clone());
        small.set_scale(1000, 1000);

        let mut large = crate::Font::new(face);
        large.set_scale(2000, 2000);

        let small_width: i32 = shape(&small, buffer_from("ABC").unwrap(), &[])
            .positions()
            .iter()
            .map(|p| p.x_advance())
            .sum();
        let large_width: i32 = shape(&large, buffer_from("ABC").unwrap(), &[])
            .positions()
            .iter()
            .map(|p| p.x_advance())
            .sum();

        assert!(small_width > 0);

        // Each glyph's advance is rounded to an integer independently, so
        // doubling the scale doubles the total only to within the accumulated
        // rounding — one unit per glyph at worst.
        let drift = (large_width - small_width * 2).abs();
        assert!(
            drift <= 3,
            "expected about {}, got {large_width}",
            small_width * 2
        );
    }
}
