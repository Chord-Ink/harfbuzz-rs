# Getting started with HarfBuzz

*Adapted from the HarfBuzz 14.3.0 documentation, which is licensed under the terms in the HarfBuzz COPYING file.*

## An overview of the HarfBuzz shaping API

The core of the HarfBuzz shaping API is the function `hb_shape()`. This function takes a font, a buffer containing a string of Unicode codepoints and (optionally) a list of font features as its input. It replaces the codepoints in the buffer with the corresponding glyphs from the font, correctly ordered and positioned, and with any of the optional font features applied.

In addition to holding the pre-shaping input (the Unicode codepoints that comprise the input string) and the post-shaping output (the glyphs and positions), a HarfBuzz buffer has several properties that affect shaping. The most important are the text-flow direction (e.g., left-to-right, right-to-left, top-to-bottom, or bottom-to-top), the script tag, and the language tag.

For input string buffers, flags are available to denote when the buffer represents the beginning or end of a paragraph, to indicate whether or not to visibly render Unicode `Default Ignorable` codepoints, and to modify the cluster-merging behavior for the buffer. For shaped output buffers, the individual X and Y offsets and `advances` (the logical dimensions) of each glyph are accessible. HarfBuzz also flags glyphs as `UNSAFE_TO_BREAK` if breaking the string at that glyph (e.g., in a line-breaking or hyphenation process) would require re-shaping the text.

HarfBuzz also provides methods to compare the contents of buffers, join buffers, normalize buffer contents, and handle invalid codepoints, as well as to determine the state of a buffer (e.g., input codepoints or output glyphs). Buffer lifecycles are managed and all buffers are reference-counted.

Although the default `hb_shape()` function is sufficient for most use cases, a variant is also provided that lets you specify which of HarfBuzz's shapers to use on a buffer.

HarfBuzz can read TrueType fonts, TrueType collections, OpenType fonts, and OpenType collections. Functions are provided to query font objects about metrics, Unicode coverage, available tables and features, and variation selectors. Individual glyphs can also be queried for metrics, variations, and glyph names. OpenType variable fonts are supported, and HarfBuzz allows you to set variation-axis coordinates on font objects.

HarfBuzz provides glue code to integrate with various other libraries, including FreeType, GObject, and CoreText. Support for integrating with Uniscribe and DirectWrite is experimental at present.

## Terminology

### script

In text shaping, a *script* is a writing system: a set of symbols, rules, and conventions that is used to represent a language or multiple languages.

In general computing lingo, the word "script" can also be used to mean an executable program (usually one written in a human-readable programming language). For the sake of clarity, HarfBuzz documents will always use more specific terminology when referring to this meaning, such as "Python script" or "shell script." In all other instances, "script" refers to a writing system.

For developers using HarfBuzz, it is important to note the distinction between a script and a language. Most scripts are used to write a variety of different languages, and many languages may be written in more than one script.

### shaper

In HarfBuzz, a *shaper* is a handler for a specific script-shaping model. HarfBuzz implements separate shapers for Indic, Arabic, Thai and Lao, Khmer, Myanmar, Tibetan, Hangul, Hebrew, the Universal Shaping Engine (USE), and a default shaper for scripts with no script-specific shaping model.

### cluster

In text shaping, a *cluster* is a sequence of codepoints that must be treated as an indivisible unit. Clusters can include code-point sequences that form a ligature or base-and-mark sequences. Tracking and preserving clusters is important when shaping operations might separate or reorder code points.

HarfBuzz provides three cluster *levels* that implement different approaches to the problem of preserving clusters during shaping operations.

### grapheme

In linguistics, a *grapheme* is one of the indivisible units that make up a writing system or script. Often, graphemes are individual symbols (letters, numbers, punctuation marks, logograms, etc.) but, depending on the writing system, a particular grapheme might correspond to a sequence of several Unicode code points.

In practice, HarfBuzz and other text-shaping engines are not generally concerned with graphemes. However, it is important for developers using HarfBuzz to recognize that there is a difference between graphemes and shaping clusters (see above). The two concepts may overlap frequently, but there is no guarantee that they will be identical.

### syllable

In linguistics, a *syllable* is a sequence of sounds that makes up a building block of a particular language. Every language has its own set of rules describing what constitutes a valid syllable.

For text-shaping purposes, the various definitions of "syllable" are important because script-specific shaping operations may be applied at the syllable level. For example, a reordering rule might specify that a vowel mark be reordered to the beginning of the syllable.

Syllables will consist of one or more Unicode code points. The definition of a syllable for a particular writing system might correspond to how HarfBuzz identifies clusters (see above) for the same writing system. However, it is important for developers using HarfBuzz to recognize that there is a difference between syllables and shaping clusters. The two concepts may overlap frequently, but there is no guarantee that they will be identical.

## A simple shaping example

Below is the simplest HarfBuzz shaping example possible.

1. Create a buffer and put your text in it.

   ```c
   #include <hb.h>

   hb_buffer_t *buf;
   buf = hb_buffer_create();
   hb_buffer_add_utf8(buf, text, -1, 0, -1);
   ```

2. Set the script, language and direction of the buffer.

   ```c
   // If you know the direction, script, and language
   hb_buffer_set_direction(buf, HB_DIRECTION_LTR);
   hb_buffer_set_script(buf, HB_SCRIPT_LATIN);
   hb_buffer_set_language(buf, hb_language_from_string("en", -1));

   // If you don't know the direction, script, and language
   hb_buffer_guess_segment_properties(buf);
   ```

3. Create a face and a font from a font file.

   ```c
   hb_blob_t *blob = hb_blob_create_from_file(filename); /* or hb_blob_create_from_file_or_fail() */
   hb_face_t *face = hb_face_create(blob, 0);
   hb_font_t *font = hb_font_create(face);
   ```

4. Shape!

   ```c
   hb_shape(font, buf, NULL, 0);
   ```

5. Get the glyph and position information.

   ```c
   unsigned int glyph_count;
   hb_glyph_info_t *glyph_info    = hb_buffer_get_glyph_infos(buf, &glyph_count);
   hb_glyph_position_t *glyph_pos = hb_buffer_get_glyph_positions(buf, &glyph_count);
   ```

6. Iterate over each glyph.

   ```c
   hb_position_t cursor_x = 0;
   hb_position_t cursor_y = 0;
   for (unsigned int i = 0; i < glyph_count; i++) {
       hb_codepoint_t glyphid  = glyph_info[i].codepoint;
       hb_position_t x_offset  = glyph_pos[i].x_offset;
       hb_position_t y_offset  = glyph_pos[i].y_offset;
       hb_position_t x_advance = glyph_pos[i].x_advance;
       hb_position_t y_advance = glyph_pos[i].y_advance;
     /* draw_glyph(glyphid, cursor_x + x_offset, cursor_y + y_offset); */
       cursor_x += x_advance;
       cursor_y += y_advance;
   }
   ```

7. Tidy up.

   ```c
   hb_buffer_destroy(buf);
   hb_font_destroy(font);
   hb_face_destroy(face);
   hb_blob_destroy(blob);
   ```

This example shows enough to get us started using HarfBuzz. In the sections that follow, we will use the remainder of HarfBuzz's API to refine and extend the example and improve its text-shaping capabilities.
