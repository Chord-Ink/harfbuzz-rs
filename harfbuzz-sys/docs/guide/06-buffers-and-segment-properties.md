# Buffers, language, script and direction

*Adapted from the HarfBuzz 14.3.0 documentation, which is licensed under the terms in the HarfBuzz COPYING file.*

The input to the HarfBuzz shaper is a series of Unicode characters, stored in a buffer. In this chapter, we'll look at how to set up a buffer with the text that we want and how to customize the properties of the buffer. We'll also look at a piece of lower-level machinery that you will need to understand before proceeding: the functions that HarfBuzz uses to retrieve Unicode information.

After shaping is complete, HarfBuzz puts its output back into the buffer. But getting that output requires setting up a face and a font first, so we will look at that in the next chapter instead of here.

## Creating and destroying buffers

As we saw in our *Getting Started* example, a buffer is created and initialized with `hb_buffer_create()`. This produces a new, empty buffer object, instantiated with some default values and ready to accept your Unicode strings.

HarfBuzz manages the memory of objects (such as buffers) that it creates, so you don't have to. When you have finished working on a buffer, you can call `hb_buffer_destroy()`:

```c
hb_buffer_t *buf = hb_buffer_create();
...
hb_buffer_destroy(buf);
```

This will destroy the object and free its associated memory - unless some other part of the program holds a reference to this buffer. If you acquire a HarfBuzz buffer from another subsystem and want to ensure that it is not garbage collected by someone else destroying it, you should increase its reference count:

```c
void somefunc(hb_buffer_t *buf) {
  buf = hb_buffer_reference(buf);
  ...
```

And then decrease it once you're done with it:

```c
  hb_buffer_destroy(buf);
}
```

While we are on the subject of reference-counting buffers, it is worth noting that an individual buffer can only meaningfully be used by one thread at a time.

To throw away all the data in your buffer and start from scratch, call `hb_buffer_reset(buf)`. If you want to throw away the string in the buffer but keep the options, you can instead call `hb_buffer_clear_contents(buf)`.

## Adding text to the buffer

Now we have a brand new HarfBuzz buffer. Let's start filling it with text! From HarfBuzz's perspective, a buffer is just a stream of Unicode code points, but your input string is probably in one of the standard Unicode character encodings (UTF-8, UTF-16, or UTF-32). HarfBuzz provides convenience functions that accept each of these encodings: `hb_buffer_add_utf8()`, `hb_buffer_add_utf16()`, and `hb_buffer_add_utf32()`. Other than the character encoding they accept, they function identically.

You can add UTF-8 text to a buffer by passing in the text array, the array's length, an offset into the array for the first character to add, and the length of the segment to add:

```c
hb_buffer_add_utf8 (hb_buffer_t *buf,
                const char *text,
                int text_length,
                unsigned int item_offset,
                int item_length)
```

So, in practice, you can say:

```c
hb_buffer_add_utf8(buf, text, strlen(text), 0, strlen(text));
```

This will append your new characters to `buf`, not replace its existing contents. Also, note that you can use `-1` in place of the first instance of `strlen(text)` if your text array is NULL-terminated. Similarly, you can also use `-1` as the final argument want to add its full contents.

Whatever start `item_offset` and `item_length` you provide, HarfBuzz will also attempt to grab the five characters *before* the offset point and the five characters *after* the designated end. These are the before and after "context" segments, which are used internally for HarfBuzz to make shaping decisions. They will not be part of the final output, but they ensure that HarfBuzz's script-specific shaping operations are correct. If there are fewer than five characters available for the before or after contexts, HarfBuzz will just grab what is there.

For longer text runs, such as full paragraphs, it might be tempting to only add smaller sub-segments to a buffer and shape them in piecemeal fashion. Generally, this is not a good idea, however, because a lot of shaping decisions are dependent on this context information. For example, in Arabic and other connected scripts, HarfBuzz needs to know the code points before and after each character in order to correctly determine which glyph to return.

The safest approach is to add all of the text available (even if your text contains a mix of scripts, directions, languages and fonts), then use `item_offset` and `item_length` to indicate which characters you want shaped (which must all have the same script, direction, language and font), so that HarfBuzz has access to any context.

You can also add Unicode code points directly with `hb_buffer_add_codepoints()`. The arguments to this function are the same as those for the UTF encodings. But it is particularly important to note that HarfBuzz does not do validity checking on the text that is added to a buffer. Invalid code points will be replaced, but it is up to you to do any deep-sanity checking necessary.

## Setting buffer properties

Buffers containing input characters still need several properties set before HarfBuzz can shape their text correctly.

Initially, all buffers are set to the `HB_BUFFER_CONTENT_TYPE_INVALID` content type. After adding text, the buffer should be set to `HB_BUFFER_CONTENT_TYPE_UNICODE` instead, which indicates that it contains un-shaped input characters. After shaping, the buffer will have the `HB_BUFFER_CONTENT_TYPE_GLYPHS` content type.

`hb_buffer_add_utf8()` and the other UTF functions set the content type of their buffer automatically. But if you are reusing a buffer you may want to check its state with `hb_buffer_get_content_type(buffer)`. If necessary you can set the content type with

```c
hb_buffer_set_content_type(buf, HB_BUFFER_CONTENT_TYPE_UNICODE);
```

to prepare for shaping.

Buffers also need to carry information about the script, language, and text direction of their contents. You can set these properties individually:

```c
hb_buffer_set_direction(buf, HB_DIRECTION_LTR);
hb_buffer_set_script(buf, HB_SCRIPT_LATIN);
hb_buffer_set_language(buf, hb_language_from_string("en", -1));
```

However, since these properties are often repeated for multiple text runs, you can also save them in a `hb_segment_properties_t` for reuse:

```c
hb_segment_properties_t *savedprops;
hb_buffer_get_segment_properties (buf, savedprops);
...
hb_buffer_set_segment_properties (buf2, savedprops);
```

HarfBuzz also provides getter functions to retrieve a buffer's direction, script, and language properties individually.

HarfBuzz recognizes four text directions in `hb_direction_t`: left-to-right (`HB_DIRECTION_LTR`), right-to-left (`HB_DIRECTION_RTL`), top-to-bottom (`HB_DIRECTION_TTB`), and bottom-to-top (`HB_DIRECTION_BTT`). For the script property, HarfBuzz uses identifiers based on the [ISO 15924 standard](https://unicode.org/iso15924/). For languages, HarfBuzz uses tags based on the [IETF BCP 47](https://tools.ietf.org/html/bcp47) standard.

Helper functions are provided to convert character strings into the necessary script and language tag types.

Two additional buffer properties to be aware of are the "invisible glyph" and the replacement code point. The replacement code point is inserted into buffer output in place of any invalid code points encountered in the input. By default, it is the Unicode `REPLACEMENT CHARACTER` code point, `U+FFFD` "�". You can change this with

```c
hb_buffer_set_replacement_codepoint(buf, replacement);
```

passing in the replacement Unicode code point as the `replacement` parameter.

The invisible glyph is used to replace all output glyphs that are invisible. By default, the standard space character `U+0020` is used; you can replace this (for example, when using a font that provides script-specific spaces) with

```c
hb_buffer_set_invisible_glyph(buf, replacement_glyph);
```

Do note that in the `replacement_glyph` parameter, you must provide the glyph ID of the replacement you wish to use, not the Unicode code point.

HarfBuzz supports a few additional flags you might want to set on your buffer under certain circumstances. The `HB_BUFFER_FLAG_BOT` and `HB_BUFFER_FLAG_EOT` flags tell HarfBuzz that the buffer represents the beginning or end (respectively) of a text element (such as a paragraph or other block). Knowing this allows HarfBuzz to apply certain contextual font features when shaping, such as initial or final variants in connected scripts.

`HB_BUFFER_FLAG_PRESERVE_DEFAULT_IGNORABLES` tells HarfBuzz not to hide glyphs with the `Default_Ignorable` property in Unicode. This property designates control characters and other non-printing code points, such as joiners and variation selectors. Normally HarfBuzz replaces them in the output buffer with zero-width space glyphs (using the "invisible glyph" property discussed above); setting this flag causes them to be printed, which can be helpful for troubleshooting.

Conversely, setting the `HB_BUFFER_FLAG_REMOVE_DEFAULT_IGNORABLES` flag tells HarfBuzz to remove `Default_Ignorable` glyphs from the output buffer entirely. Finally, setting the `HB_BUFFER_FLAG_DO_NOT_INSERT_DOTTED_CIRCLE` flag tells HarfBuzz not to insert the dotted-circle glyph (`U+25CC`, "◌"), which is normally inserted into buffer output when broken character sequences are encountered (such as combining marks that are not attached to a base character).

## Customizing Unicode functions

HarfBuzz requires some simple functions for accessing information from the Unicode Character Database (such as the `General_Category` (gc) and `Script` (sc) properties) that is useful for shaping, as well as some useful operations like composing and decomposing code points.

HarfBuzz includes its own internal, lightweight set of Unicode functions. At build time, it is also possible to compile support for some other options, such as the Unicode functions provided by GLib or the International Components for Unicode (ICU) library. Generally, this option is only of interest for client programs that have specific integration requirements or that do a significant amount of customization.

If your program has access to other Unicode functions, however, such as through a system library or application framework, you might prefer to use those instead of the built-in options. HarfBuzz supports this by implementing its Unicode functions as a set of virtual methods that you can replace — without otherwise affecting HarfBuzz's functionality.

The Unicode functions are specified in a structure called `unicode_funcs` which is attached to each buffer. But even though `unicode_funcs` is associated with a `hb_buffer_t`, the functions themselves are called by other HarfBuzz APIs that access buffers, so it would be unwise for you to hook different functions into different buffers.

In addition, you can mark your `unicode_funcs` as immutable by calling `hb_unicode_funcs_make_immutable (ufuncs)`. This is especially useful if your code is a library or framework that will have its own client programs. By marking your Unicode function choices as immutable, you prevent your own client programs from changing the `unicode_funcs` configuration and introducing inconsistencies and errors downstream.

You can retrieve the Unicode-functions configuration for your buffer by calling `hb_buffer_get_unicode_funcs()`:

```c
hb_unicode_funcs_t *ufunctions;
ufunctions = hb_buffer_get_unicode_funcs(buf);
```

The current version of `unicode_funcs` uses six functions:

- `hb_unicode_combining_class_func_t`: returns the Canonical Combining Class of a code point.
- `hb_unicode_general_category_func_t`: returns the General Category (gc) of a code point.
- `hb_unicode_mirroring_func_t`: returns the Mirroring Glyph code point (for bi-directional replacement) of a code point.
- `hb_unicode_script_func_t`: returns the Script (sc) property of a code point.
- `hb_unicode_compose_func_t`: returns the canonical composition of a sequence of two code points.
- `hb_unicode_decompose_func_t`: returns the canonical decomposition of a code point.

Note, however, that future HarfBuzz releases may alter this set.

Each Unicode function has a corresponding setter, with which you can assign a callback to your replacement function. For example, to replace `hb_unicode_general_category_func_t`, you can call

```c
hb_unicode_funcs_set_general_category_func (*ufuncs, func, *user_data, destroy)
```

Virtualizing this set of Unicode functions is primarily intended to improve portability. There is no need for every client program to make the effort to replace the default options, so if you are unsure, do not feel any pressure to customize `unicode_funcs`.
