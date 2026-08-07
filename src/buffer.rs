//! Buffers: the text going in, and the glyphs coming out.

use harfbuzz_sys as sys;

use crate::direction::Direction;
use crate::error::{Error, Result};
use crate::language::Language;
use crate::object::{HarfBuzzObject, harfbuzz_object};
use crate::script::Script;

harfbuzz_object! {
    /// A run of text before shaping, and the glyphs it became after.
    ///
    /// A buffer starts empty, is filled with Unicode text, and is handed to
    /// [`shape`](crate::shape) along with a [`Font`](crate::Font). Shaping
    /// replaces its contents in place: the code points become glyph IDs and
    /// gain positions. The same buffer holds both, which is why it has one
    /// content type that flips from Unicode to glyphs.
    ///
    /// Buffers are the one HarfBuzz object that is inherently mutable, so they
    /// are never shared. They are also the one worth reusing: clearing and
    /// refilling a buffer avoids reallocating for every run of text.
    ///
    /// # Segment properties
    ///
    /// Shaping needs to know the direction, script, and language of the text.
    /// Set them if you know them; call
    /// [`guess_segment_properties`](Buffer::guess_segment_properties) if you do
    /// not, and HarfBuzz will infer them from the code points.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use harfbuzz_rs::{Buffer, Face, Font, IntoShared, shape};
    ///
    /// let font = Font::new(Face::from_file("font.ttf", 0)?.into_shared());
    ///
    /// let mut buffer = Buffer::new();
    /// buffer.push_str("hello");
    /// buffer.guess_segment_properties();
    ///
    /// let output = shape(&font, buffer, &[]);
    /// for (info, position) in output.iter() {
    ///     println!("glyph {} advance {}", info.glyph(), position.x_advance());
    /// }
    /// # Ok::<(), harfbuzz_rs::Error>(())
    /// ```
    Buffer,
    raw: sys::hb_buffer_t,
    reference: sys::hb_buffer_reference,
    destroy: sys::hb_buffer_destroy,
}

impl Buffer {
    /// Creates an empty buffer.
    pub fn new() -> Self {
        // SAFETY: takes no arguments. Never returns null — on failure it yields
        // the inert empty-buffer singleton — and the reference is ours.
        unsafe { Self::from_raw(sys::hb_buffer_create()) }
    }

    /// Appends UTF-8 text.
    ///
    /// Cluster values start at the byte offset within the *whole* text added to
    /// this buffer, so appending twice keeps offsets continuous.
    pub fn push_str(&mut self, text: &str) {
        let offset = self.len();

        // SAFETY: `self` owns a live buffer and `&mut self` proves exclusivity.
        // `text` is valid UTF-8 whose byte length we pass explicitly, so
        // HarfBuzz reads exactly those bytes. Passing the full range as the
        // "item" means every code point is added, not just a sub-range.
        unsafe {
            sys::hb_buffer_add_utf8(
                self.as_raw(),
                text.as_ptr().cast(),
                text.len() as core::ffi::c_int,
                0,
                text.len() as core::ffi::c_int,
            )
        };

        let _ = offset;
    }

    /// Appends UTF-8 text along with the surrounding context.
    ///
    /// Shaping a fragment in isolation gets Arabic joining and similar
    /// context-sensitive behaviour wrong. The context is not shaped and does
    /// not appear in the output; it only informs the shaper about what sits on
    /// either side.
    pub fn push_str_with_context(&mut self, before: &str, text: &str, after: &str) {
        // Concatenating lets HarfBuzz see one contiguous run, with the item
        // range naming the part we actually want shaped.
        let combined = format!("{before}{text}{after}");
        let start = before.len();
        let end = start + text.len();

        // SAFETY: `combined` is valid UTF-8 that outlives the call; the byte
        // length is passed explicitly, and `start`/`end` are byte offsets
        // within it by construction, so HarfBuzz stays in bounds.
        unsafe {
            sys::hb_buffer_add_utf8(
                self.as_raw(),
                combined.as_ptr().cast(),
                combined.len() as core::ffi::c_int,
                start as core::ffi::c_uint,
                (end - start) as core::ffi::c_int,
            )
        };
    }

    /// How many items the buffer holds — code points before shaping, glyphs
    /// after.
    pub fn len(&self) -> usize {
        // SAFETY: `self` owns a live buffer.
        unsafe { sys::hb_buffer_get_length(self.as_raw()) as usize }
    }

    /// Whether the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Empties the buffer, keeping its allocation.
    ///
    /// This is the cheap way to shape run after run without reallocating.
    /// Note that it also clears the segment properties — upstream describes it
    /// as [`reset`](Buffer::reset) minus the Unicode functions and the
    /// replacement code point — so direction, script, and language have to be
    /// set again for the next run.
    pub fn clear_contents(&mut self) {
        // SAFETY: `self` owns a live buffer; `&mut self` proves exclusivity.
        unsafe { sys::hb_buffer_clear_contents(self.as_raw()) };
    }

    /// Empties the buffer and resets every property to its default.
    pub fn reset(&mut self) {
        // SAFETY: as above.
        unsafe { sys::hb_buffer_reset(self.as_raw()) };
    }

    /// The text direction.
    pub fn direction(&self) -> Direction {
        // SAFETY: `self` owns a live buffer.
        Direction::from_raw(unsafe { sys::hb_buffer_get_direction(self.as_raw()) })
    }

    /// Sets the text direction.
    pub fn set_direction(&mut self, direction: Direction) {
        // SAFETY: `self` owns a live buffer; `&mut self` proves exclusivity.
        unsafe { sys::hb_buffer_set_direction(self.as_raw(), direction.to_raw()) };
    }

    /// The script.
    pub fn script(&self) -> Script {
        // SAFETY: `self` owns a live buffer.
        Script::from_raw(unsafe { sys::hb_buffer_get_script(self.as_raw()) })
    }

    /// Sets the script.
    pub fn set_script(&mut self, script: Script) {
        // SAFETY: as above.
        unsafe { sys::hb_buffer_set_script(self.as_raw(), script.to_raw()) };
    }

    /// The language.
    pub fn language(&self) -> Language {
        // SAFETY: `self` owns a live buffer. The returned handle is interned
        // and outlives the buffer.
        Language(unsafe { sys::hb_buffer_get_language(self.as_raw()) })
    }

    /// Sets the language.
    pub fn set_language(&mut self, language: Language) {
        // SAFETY: `self` owns a live buffer; the language handle is interned
        // and stays valid.
        unsafe { sys::hb_buffer_set_language(self.as_raw(), language.0) };
    }

    /// Fills in whichever of direction, script, and language are still unset,
    /// by inspecting the code points already in the buffer.
    ///
    /// Call this after adding text and before shaping. It only fills gaps, so
    /// anything you set explicitly is preserved.
    pub fn guess_segment_properties(&mut self) {
        // SAFETY: `self` owns a live buffer; `&mut self` proves exclusivity.
        unsafe { sys::hb_buffer_guess_segment_properties(self.as_raw()) };
    }

    /// Reverses the buffer's contents.
    pub fn reverse(&mut self) {
        // SAFETY: as above.
        unsafe { sys::hb_buffer_reverse(self.as_raw()) };
    }

    /// Whether every allocation the buffer attempted succeeded.
    ///
    /// Buffers accumulate failure rather than reporting it per call, so this is
    /// the one place to check after a series of pushes.
    pub fn allocation_successful(&self) -> bool {
        // SAFETY: `self` owns a live buffer.
        unsafe { sys::hb_buffer_allocation_successful(self.as_raw()) != 0 }
    }

    /// Turns a filled buffer into a [`GlyphBuffer`] once shaping is done.
    pub(crate) fn into_glyph_buffer(self) -> GlyphBuffer {
        GlyphBuffer(self)
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for Buffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Buffer")
            .field("len", &self.len())
            .field("direction", &self.direction())
            .field("script", &self.script())
            .finish()
    }
}

/// A buffer that has been shaped, holding glyphs rather than code points.
///
/// Keeping this distinct from [`Buffer`] means the accessors that only make
/// sense after shaping cannot be called before it, and the text-adding methods
/// cannot be called after. To shape another run, get the buffer back with
/// [`GlyphBuffer::clear`].
pub struct GlyphBuffer(Buffer);

impl GlyphBuffer {
    /// How many glyphs the shaper produced.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether shaping produced no glyphs.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The glyph identities and their cluster values.
    pub fn infos(&self) -> &[GlyphInfo] {
        let mut length = 0;

        // SAFETY: `self.0` owns a live, shaped buffer, and `length` is a live
        // local for HarfBuzz to write through.
        let data = unsafe { sys::hb_buffer_get_glyph_infos(self.0.as_raw(), &mut length) };

        if data.is_null() || length == 0 {
            return &[];
        }

        // SAFETY: HarfBuzz reported `length` elements at `data`, owned by the
        // buffer. `GlyphInfo` is `#[repr(transparent)]` over
        // `hb_glyph_info_t`, so the layouts match exactly. Tying the slice to
        // `&self` keeps the buffer alive for the borrow, and `&self` rules out
        // a concurrent `&mut`.
        unsafe { std::slice::from_raw_parts(data.cast(), length as usize) }
    }

    /// The advances and offsets shaping produced.
    ///
    /// This is the same length as [`infos`](GlyphBuffer::infos) and in the same
    /// order.
    pub fn positions(&self) -> &[GlyphPosition] {
        let mut length = 0;

        // SAFETY: as in `infos`.
        let data = unsafe { sys::hb_buffer_get_glyph_positions(self.0.as_raw(), &mut length) };

        if data.is_null() || length == 0 {
            return &[];
        }

        // SAFETY: as in `infos`; `GlyphPosition` is `#[repr(transparent)]`
        // over `hb_glyph_position_t`.
        unsafe { std::slice::from_raw_parts(data.cast(), length as usize) }
    }

    /// Walks the glyphs and their positions together.
    pub fn iter(&self) -> impl Iterator<Item = (&GlyphInfo, &GlyphPosition)> {
        self.infos().iter().zip(self.positions())
    }

    /// The direction the text was shaped in.
    pub fn direction(&self) -> Direction {
        self.0.direction()
    }

    /// Empties the buffer and hands it back for another run, keeping the
    /// allocation.
    pub fn clear(mut self) -> Buffer {
        self.0.clear_contents();
        self.0
    }
}

impl std::fmt::Debug for GlyphBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphBuffer")
            .field("len", &self.len())
            .field("direction", &self.direction())
            .finish()
    }
}

/// What a shaped glyph is, and which part of the input it came from.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct GlyphInfo(sys::hb_glyph_info_t);

impl GlyphInfo {
    /// The glyph index in the font.
    ///
    /// Before shaping this field holds a Unicode code point instead; the
    /// [`GlyphBuffer`] type is what guarantees you only see it after.
    pub fn glyph(&self) -> u32 {
        self.0.codepoint
    }

    /// Which cluster of the original text this glyph belongs to.
    ///
    /// Clusters are how shaped output maps back to input. Several glyphs can
    /// share a cluster — a decomposed accent — and one glyph can cover several
    /// input characters — a ligature. The value is the byte offset the cluster
    /// started at.
    pub fn cluster(&self) -> u32 {
        self.0.cluster
    }

    /// Whether this glyph may be safely broken from the previous one without
    /// changing the shaping of either.
    pub fn is_unsafe_to_break(&self) -> bool {
        self.0.mask & (sys::HB_GLYPH_FLAG_UNSAFE_TO_BREAK as u32) != 0
    }
}

impl std::fmt::Debug for GlyphInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphInfo")
            .field("glyph", &self.glyph())
            .field("cluster", &self.cluster())
            .finish()
    }
}

/// Where a shaped glyph goes, in the font's scaled units.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct GlyphPosition(sys::hb_glyph_position_t);

impl GlyphPosition {
    /// How far to move the pen horizontally after drawing this glyph.
    pub fn x_advance(&self) -> i32 {
        self.0.x_advance
    }

    /// How far to move the pen vertically after drawing this glyph.
    pub fn y_advance(&self) -> i32 {
        self.0.y_advance
    }

    /// Horizontal shift of this glyph from the pen position.
    pub fn x_offset(&self) -> i32 {
        self.0.x_offset
    }

    /// Vertical shift of this glyph from the pen position.
    pub fn y_offset(&self) -> i32 {
        self.0.y_offset
    }
}

impl std::fmt::Debug for GlyphPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlyphPosition")
            .field("x_advance", &self.x_advance())
            .field("y_advance", &self.y_advance())
            .field("x_offset", &self.x_offset())
            .field("y_offset", &self.y_offset())
            .finish()
    }
}

/// Builds a buffer from text, filling in the segment properties.
///
/// A shorthand for the usual three lines.
pub fn buffer_from(text: &str) -> Result<Buffer> {
    let mut buffer = Buffer::new();
    buffer.push_str(text);

    if !buffer.allocation_successful() {
        return Err(Error::AllocationFailed);
    }

    buffer.guess_segment_properties();

    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_code_points_before_shaping() {
        let mut buffer = Buffer::new();
        buffer.push_str("hello");

        assert_eq!(buffer.len(), 5);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn guesses_properties_from_the_text() {
        let mut buffer = Buffer::new();
        buffer.push_str("hello");
        buffer.guess_segment_properties();

        assert_eq!(buffer.direction(), Direction::LeftToRight);
        assert_eq!(buffer.script(), Script::LATIN);
    }

    #[test]
    fn guesses_right_to_left_for_arabic() {
        let mut buffer = Buffer::new();
        buffer.push_str("مرحبا");
        buffer.guess_segment_properties();

        assert_eq!(buffer.direction(), Direction::RightToLeft);
        assert_eq!(buffer.script(), Script::ARABIC);
    }

    #[test]
    fn keeps_properties_that_were_set_explicitly() {
        let mut buffer = Buffer::new();
        buffer.set_direction(Direction::TopToBottom);
        buffer.push_str("hello");
        buffer.guess_segment_properties();

        // Latin would have been guessed left-to-right; ours survived.
        assert_eq!(buffer.direction(), Direction::TopToBottom);
    }

    #[test]
    fn clearing_drops_the_contents_and_the_properties() {
        let mut buffer = Buffer::new();
        buffer.set_direction(Direction::RightToLeft);
        buffer.push_str("hello");

        buffer.clear_contents();

        // Upstream defines this as `reset` minus the Unicode functions, so the
        // segment properties go too.
        assert!(buffer.is_empty());
        assert_eq!(buffer.direction(), Direction::Invalid);
    }

    #[test]
    fn context_does_not_become_part_of_the_buffer() {
        let mut buffer = Buffer::new();
        buffer.push_str_with_context("abc", "def", "ghi");

        assert_eq!(buffer.len(), 3, "only the middle run is shaped");
    }

    #[test]
    fn the_convenience_constructor_fills_properties_in() {
        let buffer = buffer_from("hello").unwrap();

        assert_eq!(buffer.len(), 5);
        assert_eq!(buffer.script(), Script::LATIN);
    }
}
