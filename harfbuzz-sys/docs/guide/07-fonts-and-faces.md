# Fonts, faces, and output

*Adapted from the HarfBuzz 14.3.0 documentation, which is licensed under the terms in the HarfBuzz COPYING file.*

In the previous chapter, we saw how to set up a buffer and fill it with text as Unicode code points. In order to shape this buffer text with HarfBuzz, you will need also need a font object.

HarfBuzz provides abstractions to help you cache and reuse the heavier parts of working with binary fonts, so we will look at how to do that. We will also look at how to work with the FreeType font-rendering library and at how you can customize HarfBuzz to work with other libraries.

Finally, we will look at how to work with OpenType variable fonts, the latest update to the OpenType font format, and at some other recent additions to OpenType.

## Font and face objects

The outcome of shaping a run of text depends on the contents of a specific font file (such as the substitutions and positioning moves in the 'GSUB' and 'GPOS' tables), so HarfBuzz makes accessing those internals fast.

An `hb_face_t` represents a *face* in HarfBuzz. This data type is a wrapper around an `hb_blob_t` blob that holds the contents of a binary font file. Since HarfBuzz supports TrueType Collections and OpenType Collections (each of which can include multiple typefaces), a HarfBuzz face also requires an index number specifying which typeface in the file you want to use. Most of the font files you will encounter in the wild include just a single face, however, so most of the time you would pass in `0` as the index when you create a face:

```c
hb_blob_t* blob = hb_blob_create_from_file(file);
...
hb_face_t* face = hb_face_create(blob, 0);
```

On its own, a face object is not quite ready to use for shaping. The typeface must be set to a specific point size in order for some details (such as hinting) to work. In addition, if the font file in question is an OpenType Variable Font, then you may need to specify one or more variation-axis settings (or a named instance) in order to get the output you need.

In HarfBuzz, you do this by creating a *font* object from your face.

Font objects also have the advantage of being considerably lighter-weight than face objects (remember that a face contains the contents of a binary font file mapped into memory). As a result, you can cache and reuse a font object, but you could also create a new one for each additional size you needed. Creating new fonts incurs some additional overhead, of course, but whether or not it is excessive is your call in the end. In contrast, face objects are substantially larger, and you really should cache them and reuse them whenever possible.

You can create a font object from a face object:

```c
hb_font_t* hb_font = hb_font_create(hb_face);
```

After creating a font, there are a few properties you should set. Many fonts enable and disable hints based on the size it is used at, so setting this is important for font objects. `hb_font_set_ppem(font, x_ppem, y_ppem)` sets the pixels-per-EM value of the font. You can also set the point size of the font with `hb_font_set_ptem(font, ptem)`. HarfBuzz uses the industry standard 72 points per inch.

HarfBuzz lets you specify the degree subpixel precision you want through a scaling factor. You can set horizontal and vertical scaling factors on the font by calling `hb_font_set_scale(font, x_scale, y_scale)`.

There may be times when you are handed a font object and need to access the face object that it comes from. For that, you can call

```c
hb_face = hb_font_get_face(hb_font);
```

You can also create a font object from an existing font object using the `hb_font_create_sub_font()` function. This creates a child font object that is initiated with the same attributes as its parent; it can be used to quickly set up a new font for the purpose of overriding a specific font-functions method.

All face objects and font objects are lifecycle-managed by HarfBuzz. After creating a face, you increase its reference count with `hb_face_reference(face)` and decrease it with `hb_face_destroy(face)`. Likewise, you increase the reference count on a font with `hb_font_reference(font)` and decrease it with `hb_font_destroy(font)`.

You can also attach user data to face objects and font objects.

## Customizing font functions

During shaping, HarfBuzz frequently needs to query font objects to get at the contents and parameters of the glyphs in a font file. It includes a built-in set of functions that is tailored to working with OpenType fonts. However, as was the case with Unicode functions in the Buffers, language, script and direction guide, HarfBuzz also wants to make it easy for you to assign a substitute set of font functions if you are developing a program to work with a library or platform that provides its own font functions.

Therefore, the HarfBuzz API defines a set of virtual methods for accessing font-object properties, and you can replace the defaults with your own selections without interfering with the shaping process. Each font object in HarfBuzz includes a structure called `font_funcs` that serves as a vtable for the font object. The virtual methods in `font_funcs` are:

- `hb_font_get_font_h_extents_func_t`: returns the extents of the font for horizontal text.
- `hb_font_get_font_v_extents_func_t`: returns the extents of the font for vertical text.
- `hb_font_get_nominal_glyph_func_t`: returns the font's nominal glyph for a given code point.
- `hb_font_get_variation_glyph_func_t`: returns the font's glyph for a given code point when it is followed by a given Variation Selector.
- `hb_font_get_nominal_glyphs_func_t`: returns the font's nominal glyphs for a series of code points.
- `hb_font_get_glyph_advance_func_t`: returns the advance for a glyph.
- `hb_font_get_glyph_h_advance_func_t`: returns the advance for a glyph for horizontal text.
- `hb_font_get_glyph_v_advance_func_t`: returns the advance for a glyph for vertical text.
- `hb_font_get_glyph_advances_func_t`: returns the advances for a series of glyphs.
- `hb_font_get_glyph_h_advances_func_t`: returns the advances for a series of glyphs for horizontal text.
- `hb_font_get_glyph_v_advances_func_t`: returns the advances for a series of glyphs for vertical text.
- `hb_font_get_glyph_origin_func_t`: returns the origin coordinates of a glyph.
- `hb_font_get_glyph_h_origin_func_t`: returns the origin coordinates of a glyph for horizontal text.
- `hb_font_get_glyph_v_origin_func_t`: returns the origin coordinates of a glyph for vertical text.
- `hb_font_get_glyph_extents_func_t`: returns the extents for a glyph.
- `hb_font_get_glyph_contour_point_func_t`: returns the coordinates of a specific contour point from a glyph.
- `hb_font_get_glyph_name_func_t`: returns the name of a glyph (from its glyph index).
- `hb_font_get_glyph_from_name_func_t`: returns the glyph index that corresponds to a given glyph name.
- `hb_font_draw_glyph_or_fail_func_t`: gets the outlines of a glyph (by calling `hb_draw_funcs_t` callbacks).
- `hb_font_paint_glyph_or_fail_func_t`: paints a glyph (by calling `hb_paint_funcs_t` callbacks).

You can create new font-functions by calling `hb_font_funcs_create()`:

```c
hb_font_funcs_t *ffunctions = hb_font_funcs_create ();
hb_font_set_funcs (font, ffunctions, font_data, destroy);
```

The individual methods can each be set with their own setter function, such as `hb_font_funcs_set_nominal_glyph_func(ffunctions, func, user_data, destroy)`.

Font-functions structures can be reused for multiple font objects, and can be reference counted with `hb_font_funcs_reference()` and `hb_font_funcs_destroy()`. Just like other objects in HarfBuzz, you can set user-data for each font-functions structure and assign a destroy callback for it.

You can also mark a font-functions structure as immutable, with `hb_font_funcs_make_immutable()`. This is especially useful if your code is a library or framework that will have its own client programs. By marking your font-functions structures as immutable, you prevent your client programs from changing the configuration and introducing inconsistencies and errors downstream.

To override only some functions while using the default implementation for the others, you will need to create a sub-font. By default, the sub-font uses the font functions of its parent except for the functions that were explicitly set. The following code will override only the `hb_font_get_nominal_glyph_func_t` for the sub-font:

```c
hb_font_t *subfont = hb_font_create_sub_font (font)
hb_font_funcs_t *ffunctions = hb_font_funcs_create ();
hb_font_funcs_set_nominal_glyph_func (ffunctions, func, user_data, destroy);
hb_font_set_funcs (subfont, ffunctions, font_data, destroy);
hb_font_funcs_destroy (ffunctions);
```

## Font objects and HarfBuzz's native OpenType implementation

By default, whenever HarfBuzz creates a font object, it will configure the font to use a built-in set of font functions that supports contemporary OpenType font internals. If you want to work with OpenType or TrueType fonts, you should be able to use these functions without difficulty.

Many of the methods in the font-functions structure deal with the fundamental properties of glyphs that are required for shaping text: extents (the maximums and minimums on each axis), origins (the `(0,0)` coordinate point which glyphs are drawn in reference to), and advances (the amount that the cursor needs to be moved after drawing each glyph, including any empty space for the glyph's side bearings).

As you can see in the list of functions, there are separate "horizontal" and "vertical" variants depending on whether the text is set in the horizontal or vertical direction. For some scripts, fonts that are designed to support text set horizontally or vertically (for example, in Japanese) may include metrics for both text directions. When fonts don't include this information, HarfBuzz does its best to transform what the font provides.

In addition to the direction-specific functions, HarfBuzz provides some higher-level functions for fetching information like extents and advances for a glyph. If you call

```c
hb_font_get_glyph_advance_for_direction(font, direction, extents);
```

then you can provide any `hb_direction_t` as the `direction` parameter, and HarfBuzz will use the correct function variant for the text direction. There are similar higher-level versions of the functions for fetching extents, origin coordinates, and contour-point coordinates. There are also addition and subtraction functions for moving points with respect to the origin.

There are also methods for fetching the glyph ID that corresponds to a Unicode code point (possibly when followed by a variation-selector code point), fetching the glyph name from the font, and fetching the glyph ID that corresponds to a glyph name you already have.

HarfBuzz also provides functions for converting between glyph names and string variables. `hb_font_glyph_to_string(font, glyph, s, size)` retrieves the name for the glyph ID `glyph` from the font object. It generates a generic name of the form `gidDDD` (where DDD is the glyph index) if there is no name for the glyph in the font. The `hb_font_glyph_from_string(font, s, len, glyph)` takes an input string `s` and looks for a glyph with that name in the font, returning its glyph ID in the `glyph` output parameter. It automatically parses `gidDDD` and `uniUUUU` strings.

## Working with OpenType Variable Fonts

If you are working with OpenType Variable Fonts, there are a few additional functions you should use to specify the variation-axis settings of your font object. Without doing so, your variable font's font object can still be used, but only at the default setting for every axis (which, of course, is sometimes what you want, but does not cover general usage).

HarfBuzz manages variation settings in the `hb_variation_t` data type, which holds a `tag` for the variation-axis identifier tag and a `value` for its setting. You can retrieve the list of variation axes in a font binary from the face object (not from a font object, notably) by calling `hb_ot_var_get_axis_count(face)` to find the number of axes, then using `hb_ot_var_get_axis_infos()` to collect the axis structures:

```c
axes = hb_ot_var_get_axis_count(face);
...
hb_ot_var_get_axis_infos(face, 0, axes, axes_array);
```

For each axis returned in the array, you can can access the identifier in its `tag`. HarfBuzz also has tag definitions predefined for the five standard axes specified in OpenType (`ital` for italic, `opsz` for optical size, `slnt` for slant, `wdth` for width, and `wght` for weight). Each axis also has a `min_value`, a `default_value`, and a `max_value`.

To set your font object's variation settings, you call the `hb_font_set_variations()` function with an array of `hb_variation_t` variation settings. Let's say our font has weight and width axes. We need to specify each of the axes by tag and assign a value on the axis:

```c
unsigned int variation_count = 2;
hb_variation_t variation_data[variation_count];
variation_data[0].tag = HB_OT_TAG_VAR_AXIS_WIDTH;
variation_data[1].tag = HB_OT_TAG_VAR_AXIS_WEIGHT;
variation_data[0].value = 80;
variation_data[1].value = 750;
...
hb_font_set_variations(font, variation_data, variation_count);
```

That should give us a slightly condensed font ("normal" on the `wdth` axis is 100) at a noticeably bolder weight ("regular" is 400 on the `wght` axis).

In practice, though, you should always check that the value you want to set on the axis is within the [`min_value`,`max_value`] range actually implemented in the font's variation axis. After all, a font might only provide lighter-than-regular weights, and setting a heavier value on the `wght` axis will not change that.

Once your variation settings are specified on your font object, however, shaping with a variable font is just like shaping a static font.

In addition to providing the variation axes themselves, fonts may also pre-define certain variation coordinates as named instances. HarfBuzz makes these coordinates (and their associated names) available via `hb_ot_var_named_instance_get_design_coords()` and `hb_ot_var_named_instance_get_subfamily_name_id()`.

Applications should treat named instances like multiple independent, static fonts.

## Glyphs and rendering

The main purpose of HarfBuzz is shaping, which creates a list of positioned glyphs as output. The remaining task for text layout is to convert this list into rendered output. While HarfBuzz does not handle rasterization of glyphs per se, it does have APIs that provide access to the font data that is needed to perform this task.

Traditionally, the shapes of glyphs in scalable fonts are provided as quadratic or cubic Beziér curves defining outlines to be filled. To obtain the outlines for a glyph, call `hb_font_draw_glyph()` and pass a `hb_draw_funcs_t` struct. The callbacks in that struct will be called for each segment of the outline. Note that this API provides access to outlines as they are defined in the font, without applying hinting to fit the curves to the pixel grid.

Fonts may provide pre-rendered images for glyphs instead of or in addition to outlines. This is most common for fonts that contain colored glyphs, such as Emoji. To access these images, use `hb_ot_color_reference_png()` or `hb_ot_color_reference_svg()`.

Another way in which fonts provide colored glyphs is via paint graphs that combine glyph outlines with gradients and allow for transformations and compositing. In its simplest form, this can be presented as a series of layers that are rendered on top of each other, each with its own color. HarfBuzz has the `hb_ot_color_glyph_get_layers()` to access glyph data in this form.

In the general case, you have to use `hb_font_paint_or_fail_glyph()` and pass a `hb_paint_funcs_t` struct with callbacks to obtain paint color glyphs, either graphs or image glyphs. The `hb_font_paint_glyph()` API is higher level and can handle outline glyphs as well, so it provides a unified API for access to glyph rendering information.
