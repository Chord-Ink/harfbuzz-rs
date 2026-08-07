//! Font faces: the tables of a font file, before any sizing.

use std::path::Path;

use harfbuzz_sys as sys;

use crate::blob::Blob;
use crate::error::{Error, Result};
use crate::object::{HarfBuzzObject, IntoShared, Shared, ThreadSafeWhenImmutable, harfbuzz_object};
use crate::tag::Tag;

harfbuzz_object! {
    /// One face from a font file: its tables, glyph inventory, and design
    /// units, with no size attached.
    ///
    /// A face is the parsed structure of a font. It knows how many glyphs there
    /// are and what the units-per-em is, but nothing about the size you intend
    /// to render at — that belongs to [`Font`](crate::Font), which is built
    /// from a face.
    ///
    /// Faces are the expensive object to build and the natural one to cache. A
    /// single face can back many fonts at different sizes and variation
    /// settings, so build it once, freeze it with
    /// [`into_shared`](IntoShared::into_shared), and clone the handle.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use harfbuzz_rs::{Face, IntoShared};
    ///
    /// let face = Face::from_file("font.ttf", 0)?.into_shared();
    /// println!("{} glyphs, {} upem", face.glyph_count(), face.upem());
    /// # Ok::<(), harfbuzz_rs::Error>(())
    /// ```
    Face,
    raw: sys::hb_face_t,
    reference: sys::hb_face_reference,
    destroy: sys::hb_face_destroy,
}

// SAFETY: once frozen, a face's tables are read-only and HarfBuzz's internal
// caches on it are guarded by atomics. Upstream documents the create/configure/
// freeze pattern precisely so a frozen face can be shaped with from several
// threads at once.
unsafe impl ThreadSafeWhenImmutable for Face {}

impl Face {
    /// Builds a face from font data already in memory.
    ///
    /// `index` selects a face inside a collection; pass `0` for an ordinary
    /// single-face file. The blob is referenced, not copied, so the two share
    /// the underlying bytes.
    pub fn new(blob: &Blob, index: u32) -> Result<Self> {
        // SAFETY: `blob` owns a live blob for the duration of the call.
        // `create_or_fail` takes its own reference on success rather than
        // consuming ours, and reports failure as null instead of substituting
        // the empty face.
        let raw = unsafe { sys::hb_face_create_or_fail(blob.as_raw(), index) };

        if raw.is_null() {
            // HarfBuzz does not distinguish "index out of range" from "this is
            // not a font", but the face count is cheap to check and makes the
            // common mistake legible. A count of zero means the data is not a
            // font at all, so reporting a missing face there would be
            // misleading.
            let available = blob.face_count();
            if available > 0 && index >= available {
                return Err(Error::NoSuchFace {
                    requested: index,
                    available,
                });
            }

            return Err(Error::FontLoadFailed);
        }

        // SAFETY: `raw` is non-null and carries the reference the constructor
        // created, which this wrapper now owns.
        Ok(unsafe { Self::from_raw(raw) })
    }

    /// Reads a font file and builds one of its faces.
    ///
    /// `index` selects a face inside a collection; pass `0` for an ordinary
    /// single-face file.
    pub fn from_file(path: impl AsRef<Path>, index: u32) -> Result<Self> {
        Self::new(&Blob::from_file(path)?, index)
    }

    /// Builds a face from bytes Rust owns, handing them to HarfBuzz.
    pub fn from_bytes(bytes: impl Into<std::sync::Arc<[u8]>>, index: u32) -> Result<Self> {
        Self::new(&Blob::from_bytes(bytes)?, index)
    }

    /// The number of glyphs in the face.
    pub fn glyph_count(&self) -> u32 {
        // SAFETY: `self` owns a live face.
        unsafe { sys::hb_face_get_glyph_count(self.as_raw()) }
    }

    /// The design units per em.
    ///
    /// This is the coordinate system the font's outlines are drawn in —
    /// typically 1000 for CFF fonts and 1024 or 2048 for TrueType. It is what
    /// [`Font::set_scale`](crate::Font::set_scale) scales *from*.
    pub fn upem(&self) -> u32 {
        // SAFETY: `self` owns a live face.
        unsafe { sys::hb_face_get_upem(self.as_raw()) }
    }

    /// The face's index within its collection.
    pub fn index(&self) -> u32 {
        // SAFETY: `self` owns a live face.
        unsafe { sys::hb_face_get_index(self.as_raw()) }
    }

    /// The face's underlying data, as a blob.
    ///
    /// For a face built from a blob this is the original data. For a face built
    /// table by table, HarfBuzz serialises one on demand.
    pub fn blob(&self) -> Blob {
        // SAFETY: `self` owns a live face. `reference_blob` returns a *new*
        // reference, which the wrapper takes ownership of.
        let raw = unsafe { sys::hb_face_reference_blob(self.as_raw()) };

        // SAFETY: `reference_blob` never returns null — on failure it yields
        // the empty-blob singleton — and the reference is ours.
        unsafe { Blob::from_raw(raw) }
    }

    /// One table of the font, by tag.
    ///
    /// Returns an empty blob when the face has no such table.
    pub fn table(&self, tag: Tag) -> Blob {
        // SAFETY: `self` owns a live face; the tag is a plain integer. The
        // returned blob carries a new reference that the wrapper owns.
        let raw = unsafe { sys::hb_face_reference_table(self.as_raw(), tag.to_raw()) };

        // SAFETY: never null — the empty-blob singleton stands in for a missing
        // table — and the reference is ours.
        unsafe { Blob::from_raw(raw) }
    }

    /// Sets the number of glyphs.
    ///
    /// Only meaningful for a face being built table by table; for a face parsed
    /// from font data HarfBuzz already knows.
    pub fn set_glyph_count(&mut self, count: u32) {
        // SAFETY: `self` owns a live face, and `&mut self` proves no other
        // handle can observe the change.
        unsafe { sys::hb_face_set_glyph_count(self.as_raw(), count) };
    }

    /// Sets the design units per em.
    pub fn set_upem(&mut self, upem: u32) {
        // SAFETY: as above.
        unsafe { sys::hb_face_set_upem(self.as_raw(), upem) };
    }
}

impl IntoShared for Face {
    fn into_shared(self) -> Shared<Self> {
        // SAFETY: `self` owns a live face.
        unsafe { sys::hb_face_make_immutable(self.as_raw()) };

        // SAFETY: the face was just frozen, which is what `from_immutable`
        // requires.
        unsafe { Shared::from_immutable(self) }
    }
}

impl std::fmt::Debug for Face {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Face")
            .field("index", &self.index())
            .field("glyph_count", &self.glyph_count())
            .field("upem", &self.upem())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing;

    #[test]
    fn reads_the_basics_of_a_real_font() {
        let face = testing::face();

        assert!(face.glyph_count() > 0);
        assert!(face.upem() > 0);
        assert_eq!(face.index(), 0);
    }

    #[test]
    fn rejects_data_that_is_not_a_font() {
        let blob = Blob::from_bytes(vec![0u8; 128]).unwrap();

        assert!(matches!(Face::new(&blob, 0), Err(Error::FontLoadFailed)));
    }

    #[test]
    fn reports_an_out_of_range_face_index_precisely() {
        let blob = testing::font_data();
        let available = blob.face_count();

        match Face::new(&blob, available + 5) {
            Err(Error::NoSuchFace {
                requested,
                available: reported,
            }) => {
                assert_eq!(requested, available + 5);
                assert_eq!(reported, available);
            }
            other => panic!("expected NoSuchFace, got {other:?}"),
        }
    }

    #[test]
    fn hands_back_the_tables_it_was_built_from() {
        let face = testing::face();

        // Every TrueType or CFF font has a head table.
        assert!(!face.table(Tag::new(b"head")).is_empty());
        // And none has this one.
        assert!(face.table(Tag::new(b"zzzz")).is_empty());
    }

    #[test]
    fn a_shared_face_can_cross_threads() {
        let face = testing::face().into_shared();
        let count = face.glyph_count();

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let face = face.clone();
                std::thread::spawn(move || face.glyph_count())
            })
            .collect();

        for handle in handles {
            assert_eq!(handle.join().unwrap(), count);
        }
    }
}
