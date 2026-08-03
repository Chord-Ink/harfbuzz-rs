//! HarfBuzz's native OpenType font-functions implementation — `hb-ot-font.h`.
//!
//! One function, which installs the built-in `hb-ot-font` callbacks on a font so
//! that glyph lookups, metrics, outlines and colour paints are answered by
//! HarfBuzz's own OpenType table readers. Newly created fonts use these already,
//! so most clients never call it.

use crate::hb_font_t;

unsafe extern "C" {
    /// Sets the font functions used when working with `font` to HarfBuzz's
    /// native OpenType implementation.
    ///
    /// This is the default for newly created fonts, so most client programs
    /// never need to call this directly. It is useful for putting a font back
    /// on the native implementation after installing another one — a FreeType
    /// or CoreText back end, say — or when the default was changed through the
    /// `HB_FONT_FUNCS` environment variable.
    ///
    /// Replaces both the font's [`hb_font_funcs_t`](crate::hb_font_funcs_t) and
    /// its attached font data, exactly as
    /// [`hb_font_set_funcs`](crate::hb_font_set_funcs) does, so any data
    /// previously attached to the font is destroyed. Has no effect on an
    /// immutable font.
    ///
    /// Since HarfBuzz 0.9.28.
    pub fn hb_ot_font_set_funcs(font: *mut hb_font_t);
}
