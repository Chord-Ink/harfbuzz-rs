# Integration with other libraries

*Adapted from the HarfBuzz 14.3.0 documentation ("Platform Integration Guide"), which is licensed under the terms in the HarfBuzz `COPYING` file.*

HarfBuzz was first developed for use with the GNOME and GTK software stack commonly found in desktop Linux distributions. Nevertheless, it can be used on other operating systems and platforms, from iOS and macOS to Windows. It can also be used with other application frameworks and components, such as Android, Qt, or application-specific widget libraries.

This chapter will look at how HarfBuzz fits into a typical text-rendering pipeline, and will discuss the APIs available to integrate HarfBuzz with contemporary Linux, Mac, and Windows software. It will also show how HarfBuzz integrates with popular external libraries like FreeType and International Components for Unicode (ICU) and describe the HarfBuzz language bindings for Python.

On a GNOME system, HarfBuzz is designed to tie in with several other common system libraries. The most common architecture uses Pango at the layer directly "above" HarfBuzz; Pango is responsible for text segmentation and for ensuring that each input `hb_buffer_t` passed to HarfBuzz for shaping contains Unicode code points that share the same segment properties (namely, direction, language, and script, but also higher-level properties like the active font, font style, and so on).

The layer directly "below" HarfBuzz is typically FreeType, which is used to rasterize glyph outlines at the necessary optical size, hinting settings, and pixel resolution. FreeType provides APIs for accessing font and face information, so HarfBuzz includes functions to create `hb_face_t` and `hb_font_t` objects directly from FreeType objects. HarfBuzz can use FreeType's built-in functions for `font_funcs` vtable in an `hb_font_t`.

FreeType's output is bitmaps of the rasterized glyphs; on a typical Linux system these will then be drawn by a graphics library like Cairo, but those details are beyond HarfBuzz's control. On the other hand, at the top end of the stack, Pango is part of the larger GNOME framework, and HarfBuzz does include APIs for working with key components of GNOME's higher-level libraries — most notably GLib.

For other operating systems or application frameworks, the critical integration points are where HarfBuzz gets font and face information about the font used for shaping and where HarfBuzz gets Unicode data about the input-buffer code points.

The font and face information is necessary for text shaping because HarfBuzz needs to retrieve the glyph indices for particular code points, and to know the extents and advances of glyphs. Note that, in an OpenType variable font, both of those types of information can change with different variation-axis settings.

The Unicode information is necessary for shaping because the properties of a code point (such as its General Category (gc), Canonical Combining Class (ccc), and decomposition) can directly impact the shaping moves that HarfBuzz performs.

## GNOME integration, GLib, and GObject

*API reference: the `hb-glib` section of the HarfBuzz API reference.*

As mentioned in the preceding section, HarfBuzz offers integration APIs to help client programs using the GNOME and GTK framework commonly found in desktop Linux distributions.

GLib is the main utility library for GNOME applications. It provides basic data types and conversions, file abstractions, string manipulation, and macros, as well as facilities like memory allocation and the main event loop.

Where text shaping is concerned, GLib provides several utilities that HarfBuzz can take advantage of, including a set of Unicode-data functions and a data type for script information. Both are useful when working with HarfBuzz buffers. To make use of them, you will need to include the `hb-glib.h` header file.

GLib's [Unicode manipulation API](https://developer.gnome.org/glib/stable/glib-Unicode-Manipulation.html) includes all the functionality necessary to retrieve Unicode data for the `unicode_funcs` structure of a HarfBuzz `hb_buffer_t`.

The function `hb_glib_get_unicode_funcs()` sets up a `hb_unicode_funcs_t` structure configured with the GLib Unicode functions and returns a pointer to it.

You can attach this Unicode-functions structure to your buffer, and it will be ready for use with GLib:

```c
#include <hb-glib.h>
...
hb_unicode_funcs_t *glibufunctions;
glibufunctions = hb_glib_get_unicode_funcs();
hb_buffer_set_unicode_funcs(buf, glibufunctions);
```

For script information, GLib uses the `GUnicodeScript` type. Like HarfBuzz's own `hb_script_t`, this data type is an enumeration of Unicode scripts, but text segments passed in from GLib code will be tagged with a `GUnicodeScript`. Therefore, when setting the script property on a `hb_buffer_t`, you will need to convert between the `GUnicodeScript` of the input provided by GLib and HarfBuzz's `hb_script_t` type.

The `hb_glib_script_to_script()` function takes an `GUnicodeScript` script identifier as its sole argument and returns the corresponding `hb_script_t`. The `hb_glib_script_from_script()` does the reverse, taking an `hb_script_t` and returning the `GUnicodeScript` identifier for GLib.

Finally, GLib also provides a reference-counted object type called [`GBytes`](https://developer.gnome.org/glib/stable/glib-Byte-Arrays.html#GBytes) that is used for accessing raw memory segments with the benefits of GLib's lifecycle management. HarfBuzz provides a `hb_glib_blob_create()` function that lets you create an `hb_blob_t` directly from a `GBytes` object. This function takes only the `GBytes` object as its input; HarfBuzz registers the GLib `destroy` callback automatically.

The GNOME platform also features an object system called GObject. For HarfBuzz, the main advantage of GObject is a feature called [GObject Introspection](https://gi.readthedocs.io/en/latest/). This is a middleware facility that can be used to generate language bindings for C libraries. HarfBuzz uses it to build its Python bindings, which we will look at in a separate section.

## FreeType integration

*API reference: the `hb-ft` section of the HarfBuzz API reference.*

FreeType is the free-software font-rendering engine included in desktop Linux distributions, Android, ChromeOS, iOS, and multiple Unix operating systems, and used by cross-platform programs like Chrome, Java, and GhostScript. Used together, HarfBuzz can perform shaping on Unicode text segments, outputting the glyph IDs that FreeType should rasterize from the active font as well as the positions at which those glyphs should be drawn.

HarfBuzz provides integration points with FreeType at the face-object and font-object level and for the font-functions virtual-method structure of a font object. These functions make it easy for clients that use FreeType for rasterization or font-loading, to use HarfBuzz for shaping. To use the FreeType-integration API, include the `hb-ft.h` header.

In a typical client program, you will create your `hb_face_t` face object and `hb_font_t` font object from a FreeType `FT_Face`. HarfBuzz provides a suite of functions for doing this.

In the most common case, you will want to use `hb_ft_font_create_referenced()`, which creates both an `hb_face_t` face object and `hb_font_t` font object (linked to that face object), and provides lifecycle management.

It is important to note, though, that while HarfBuzz makes a distinction between its face and font objects, FreeType's `FT_Face` does not. After you create your `FT_Face`, you must set its size parameter using `FT_Set_Char_Size()`, because an `hb_font_t` is defined as an instance of an `hb_face_t` with size specified.

```c
#include <hb-ft.h>
...
FT_New_Face(ft_library, font_path, index, &face);
FT_Set_Char_Size(face, 0, 1000, 0, 0);
hb_font_t *font = hb_ft_font_create(face);
```

`hb_ft_font_create_referenced()` is the recommended function for creating an `hb_face_t` face object. This function calls `FT_Reference_Face()` before using the `FT_Face` and calls `FT_Done_Face()` when it is finished using the `FT_Face`. Consequently, your client program does not need to worry about destroying the `FT_Face` while HarfBuzz is still using it.

Although `hb_ft_font_create_referenced()` is the recommended function, there is another variant for client code where special circumstances make it necessary. The simpler version of the function is `hb_ft_font_create()`, which takes an `FT_Face` and an optional destroy callback as its arguments. Because `hb_ft_font_create()` does not offer lifecycle management, however, your client code will be responsible for tracking references to the `FT_Face` objects and destroying them when they are no longer needed. If you do not have a valid reason for doing this, use `hb_ft_font_create_referenced()`.

After you have created your font object from your `FT_Face`, you can set or retrieve the `load_flags` of the `FT_Face` through the `hb_font_t` object. HarfBuzz provides `hb_ft_font_set_load_flags()` and `hb_ft_font_get_load_flags()` for this purpose. The ability to set the `load_flags` through the font object could be useful for enabling or disabling hinting, for example, or to activate vertical layout.

HarfBuzz also provides a utility function called `hb_ft_font_changed()` that you should call whenever you have altered the properties of your underlying `FT_Face`, as well as a `hb_ft_get_face()` that you can call on an `hb_font_t` font object to fetch its underlying `FT_Face`.

With an `hb_face_t` and `hb_font_t` both linked to your `FT_Face`, you will typically also want to use FreeType for the `font_funcs` vtable of your `hb_font_t`. As a reminder, this font-functions structure is the set of methods that HarfBuzz will use to fetch important information from the font, such as the advances and extents of individual glyphs.

All you need to do is call

```c
hb_ft_font_set_funcs(font);
```

and HarfBuzz will use FreeType for the font-functions in `font`.

As we noted above, an `hb_font_t` is derived from an `hb_face_t` with size (and, perhaps, other parameters, such as variation-axis coordinates) specified. Consequently, you can reuse an `hb_face_t` with several `hb_font_t` objects, and HarfBuzz provides functions to simplify this.

The `hb_ft_face_create_referenced()` function creates just an `hb_face_t` from a FreeType `FT_Face` and, as with `hb_ft_font_create_referenced()` above, provides lifecycle management for the `FT_Face`.

Similarly, there is an `hb_ft_face_create()` function variant that does not provide the lifecycle-management feature. As with the font-object case, if you use this version of the function, it will be your client code's respsonsibility to track usage of the `FT_Face` objects.

A third variant of this function is `hb_ft_face_create_cached()`, which is the same as `hb_ft_face_create()` except that it also uses the `generic` field of the `FT_Face` structure to save a pointer to the newly created `hb_face_t`. Subsequently, function calls that pass the same `FT_Face` will get the same `hb_face_t` returned — and the `hb_face_t` will be correctly reference counted. Still, as with `hb_ft_face_create()`, your client code must track references to the `FT_Face` itself, and destroy it when it is unneeded.

## Cairo integration

*API reference: the `hb-cairo` section of the HarfBuzz API reference.*

Cairo is a 2D graphics library that is frequently used together with GTK and Pango. Cairo supports rendering text using FreeType, or by using callback-based 'user fonts'.

HarfBuzz provides integration points with cairo for fonts as well as for buffers. To use the Cairo-integration API, link against libharfbuzz-cairo, and include the `hb-cairo.h` header. For easy buildsystem integration, HarfBuzz comes with a `harfbuzz-cairo.pc` pkg-config file.

To create a `cairo_scaled_font_t` font from a HarfBuzz `hb_font_t`, you can use `hb_cairo_font_face_create_for_font()` or `hb_cairo_font_face_create_for_face()`. The former API applies variations and synthetic slant from the `hb_font_t` when rendering, the latter takes them from the `cairo_font_options_t` that were passed when creating the `cairo_scaled_font_t`.

The Cairo fonts created in this way make use of Cairo's user-font facilities. They can be used to render on any Cairo context, and provide full support for font rendering features, including color. One current limitation of the implementation is that it does not support hinting for glyph outlines.

When using color fonts with this API, the color palette index is taken from the `cairo_font_options_t` (with new enough Cairo), and the foreground color is extracted from the source of the Cairo context.

To render the results of shaping a piece of text, use `hb_cairo_glyphs_from_buffer()` to obtain the glyphs in a form that can be passed to `cairo_show_text_glyphs()` or `cairo_show_glyphs()`.

## Uniscribe integration

*API reference: the `hb-uniscribe` section of the HarfBuzz API reference.*

If your client program is running on Windows, HarfBuzz offers an additional API that can help integrate with Microsoft's Uniscribe engine and the Windows GDI.

Overall, the Uniscribe API covers a broader set of typographic layout functions than HarfBuzz implements, but HarfBuzz's shaping API can serve as a drop-in replacement for Uniscribe's shaping functionality. In fact, one of HarfBuzz's design goals is to accurately reproduce the same output for shaping a given text segment that Uniscribe produces — even to the point of duplicating known shaping bugs or deviations from the specification — so you can be confident that your users' documents with their existing fonts will not be affected adversely by switching to HarfBuzz.

At a basic level, HarfBuzz's `hb_shape()` function replaces both the `ScriptShape()` and [`ScriptPlace()`](https://docs.microsoft.com/en-us/windows/desktop/api/Usp10/nf-usp10-scriptplace) functions from Uniscribe.

However, whereas `ScriptShape()` returns the glyphs and clusters for a shaped sequence and `ScriptPlace()` returns the advances and offsets for those glyphs, `hb_shape()` handles both. After `hb_shape()` shapes a buffer, the output glyph IDs and cluster IDs are returned as an array of `hb_glyph_info_t` structures, and the glyph advances and offsets are returned as an array of `hb_glyph_position_t` structures.

Your client program only needs to ensure that it converts correctly between HarfBuzz's low-level data types (such as `hb_position_t`) and Windows's corresponding types (such as `GOFFSET` and `ABC`). Be sure you read the Buffers, language, script and direction guide for a full explanation of how HarfBuzz input buffers are used, and see the "Shaping and buffer output" section of the OpenType features guide for the details of what `hb_shape()` returns in the output buffer when shaping is complete.

Although `hb_shape()` itself is functionally equivalent to Uniscribe's shaping routines, there are two additional HarfBuzz functions you may want to use to integrate the libraries in your code. Both are used to link HarfBuzz font objects to the equivalent Windows structures.

The `hb_uniscribe_font_get_logfontw()` function takes a `hb_font_t` font object and returns a pointer to the [`LOGFONTW`](https://docs.microsoft.com/en-us/windows/desktop/api/wingdi/ns-wingdi-logfontw) "logical font" that corresponds to it. A `LOGFONTW` structure holds font-wide attributes, including metrics, size, and style information.

The `hb_uniscribe_font_get_hfont()` function also takes a `hb_font_t` font object, but it returns an `HFONT` — a handle to the underlying logical font — instead.

`LOGFONTW`s and `HFONT`s are both needed by other Uniscribe functions.

As a final note, you may notice a reference to an optional `uniscribe` shaper back-end in the "Configuration options" section of the Installing HarfBuzz guide. This option is not a Uniscribe-integration facility.

Instead, it is a internal code path used in the `hb-shape` command-line utility, which hands shaping functionality over to Uniscribe entirely, when run on a Windows system. That allows testing HarfBuzz's native output against the Uniscribe engine, for tracking compatibility and debugging.

Because this back-end is only used when testing HarfBuzz functionality, it is disabled by default when building the HarfBuzz binaries.

## Core Text integration

*API reference: the `hb-coretext` section of the HarfBuzz API reference.*

If your client program is running on macOS or iOS, HarfBuzz offers an additional API that can help integrate with Apple's Core Text engine and the underlying Core Graphics framework. HarfBuzz does not attempt to offer the same drop-in-replacement functionality for Core Text that it strives for with Uniscribe on Windows, but you can still use HarfBuzz to perform text shaping in native macOS and iOS applications.

Note, though, that if your interest is just in using fonts that contain Apple Advanced Typography (AAT) features, then you do not need to add Core Text integration. HarfBuzz natively supports AAT features and will shape AAT fonts (on any platform) automatically, without requiring additional work on your part. This includes support for AAT-specific TrueType tables such as `mort`, `morx`, and `kerx`, which AAT fonts use instead of `GSUB` and `GPOS`.

On a macOS or iOS system, the primary integration points offered by HarfBuzz are for face objects and font objects.

The Apple APIs offer a pair of data structures that map well to HarfBuzz's face and font objects. The Core Graphics API, which is slightly lower-level than Core Text, provides [`CGFontRef`](https://developer.apple.com/documentation/coregraphics/cgfontref), which enables access to typeface properties, but does not include size information. Core Text's [`CTFontRef`](https://developer.apple.com/documentation/coretext/ctfont-q6r) is analogous to a HarfBuzz font object, with all of the properties required to render text at a specific size and configuration. Consequently, a HarfBuzz `hb_font_t` font object can be hooked up to a Core Text `CTFontRef`, and a HarfBuzz `hb_face_t` face object can be hooked up to a `CGFontRef`.

You can create a `hb_face_t` from a `CGFontRef` by using the `hb_coretext_face_create()`. Subsequently, you can retrieve the `CGFontRef` from a `hb_face_t` with `hb_coretext_face_get_cg_font()`.

Likewise, you create a `hb_font_t` from a `CTFontRef` by calling `hb_coretext_font_create()`, and you can fetch the associated `CTFontRef` from a `hb_font_t` font object with `hb_coretext_face_get_ct_font()`.

HarfBuzz also offers a `hb_font_set_ptem()` that you an use to set the nominal point size on any `hb_font_t` font object. Core Text uses this value to implement optical scaling.

As a final note, you may notice a reference to an optional `coretext` shaper back-end in the "Configuration options" section of the Installing HarfBuzz guide. This option is not a Core Text-integration facility.

Instead, it is a internal code path used in the `hb-shape` command-line utility, which hands shaping functionality over to Core Text entirely, when run on a macOS system. That allows testing HarfBuzz's native output against the Core Text engine, for tracking compatibility and debugging.

Because this back-end is only used when testing HarfBuzz functionality, it is disabled by default when building the HarfBuzz binaries.

## ICU integration

*API reference: the `hb-icu` section of the HarfBuzz API reference.*

Although HarfBuzz includes its own Unicode-data functions, it also provides integration APIs for using the International Components for Unicode (ICU) library as a source of Unicode data on any supported platform.

The principal integration point with ICU is the `hb_unicode_funcs_t` Unicode-functions structure attached to a buffer. This structure holds the virtual methods used for retrieving Unicode character properties, such as General Category, Script, Combining Class, decomposition mappings, and mirroring information.

To use ICU in your client program, you need to call `hb_icu_get_unicode_funcs()`, which creates a Unicode-functions structure populated with the ICU function for each included method. Subsequently, you can attach the Unicode-functions structure to your buffer:

```c
hb_unicode_funcs_t *icufunctions;
icufunctions = hb_icu_get_unicode_funcs();
hb_buffer_set_unicode_funcs(buf, icufunctions);
```

and ICU will be used for Unicode-data access.

HarfBuzz also supplies a pair of functions (`hb_icu_script_from_script()` and `hb_icu_script_to_script()`) for converting between ICU's and HarfBuzz's internal enumerations of Unicode scripts. The `hb_icu_script_from_script()` function converts from a HarfBuzz `hb_script_t` to an ICU `UScriptCode`. The `hb_icu_script_to_script()` function does the reverse: converting from a `UScriptCode` identifier to a `hb_script_t`.

By default, HarfBuzz's ICU support is built as a separate shared library (`libharfbuzz-icu.so`) when compiling HarfBuzz from source. This allows client programs that do not need ICU to link against HarfBuzz without unnecessarily adding ICU as a dependency. You can also build HarfBuzz with ICU support built directly into the main HarfBuzz shared library (`libharfbuzz.so`), by specifying the `--with-icu=builtin` compile-time option.

## Python bindings

As noted in the "GNOME integration, GLib, and GObject" section above, HarfBuzz uses a feature called [GObject Introspection](https://wiki.gnome.org/Projects/GObjectIntrospection) (GI) to provide bindings for Python.

At compile time, the GI scanner analyzes the HarfBuzz C source and builds metadata objects connecting the language bindings to the C library. Your Python code can then use the HarfBuzz binary through its Python interface.

HarfBuzz's Python bindings support Python 2 and Python 3. To use them, you will need to have the `pygobject` package installed. Then you should import `HarfBuzz` from `gi.repository`:

```python
from gi.repository import HarfBuzz
```

and you can call HarfBuzz functions from Python. Sample code can be found in the `sample.py` script in the HarfBuzz `src` directory.

Do note, however, that the Python API is subject to change without advance notice. GI allows the bindings to be automatically updated, which is one of its advantages, but you may need to update your Python code.
