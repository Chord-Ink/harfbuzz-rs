# Utilities

*Adapted from the HarfBuzz 14.3.0 documentation, which is licensed under the terms in the HarfBuzz COPYING file.*

HarfBuzz includes several auxiliary components in addition to the main APIs. These include a set of command-line tools, a set of lower-level APIs for common data types that may be of interest to client programs.

## Command-line tools

HarfBuzz include several command-line tools: `hb-shape`, `hb-view`, `hb-subset`, `hb-info`, `hb-raster`, and `hb-vector`. They can be used to examine HarfBuzz's functionality, debug font binaries, or explore the various shaping models and features from a terminal.

### hb-shape

*`hb-shape`* allows you to run HarfBuzz's `hb_shape()` function on an input string and to examine the outcome, in human-readable form, as terminal output. `hb-shape` does *not* render the results of the shaping call into rendered text (you can use `hb-view`, below, for that). Instead, it prints out the final glyph indices and positions, taking all shaping operations into account, as if the input string were a HarfBuzz input buffer.

You can specify the font to be used for shaping and, with command-line options, you can add various aspects of the internal state to the output that is sent to the terminal. The general format is

```sh
hb-shape [OPTIONS] path/to/font/file.ttf yourinputtext
```

The default output format is plain text (although JSON output can be selected instead by specifying the option `--output-format=json`). The default output syntax reports each glyph name (or glyph index if there is no name) followed by its cluster value, its horizontal and vertical position displacement, and its horizontal and vertical advances.

Output options exist to skip any of these elements in the output, and to include additional data, such as Unicode code-point values, glyph extents, glyph flags, or interim shaping results.

Output can also be redirected to a file, or input read from a file. Additional options enable you to enable or disable specific font features, to set variation-font axis values, to alter the language, script, direction, and clustering settings used, to enable sanity checks, or to change which shaping engine is used.

For a complete explanation of the options available, run

```sh
hb-shape --help
```

### hb-view

*`hb-view`* allows you to see the shaped output of an input string in rendered form. Like `hb-shape`, `hb-view` takes a font file and a text string as its arguments:

```sh
hb-view [OPTIONS] path/to/font/file.ttf yourinputtext
```

By default, `hb-view` renders the shaped text in ASCII block-character images as terminal output. By appending the `--output-file=[filename]` switch, you can write the output to a PNG, SVG, or PDF file (among other formats).

As with `hb-shape`, a lengthy set of options is available, with which you can enable or disable specific font features, set variation-font axis values, alter the language, script, direction, and clustering settings used, enable sanity checks, or change which shaping engine is used.

You can also set the foreground and background colors used for the output, independently control the width of all four margins, alter the line spacing, and annotate the output image with

> **Note from the translation:** the preceding sentence ends mid-clause in the upstream source (`docs/usermanual-utilities.xml`); it is reproduced here verbatim rather than being guessed at or dropped.

In general, `hb-view` is a quick way to verify that the output of HarfBuzz's shaping operation looks correct for a given text-and-font combination, but you may want to use `hb-shape` to figure out exactly why something does not appear as expected.

### hb-raster

*`hb-raster`* takes a font file and a text string, shapes the text, and renders the result as a PPM (Portable Pixel Map) raster image.

```sh
hb-raster [OPTIONS] path/to/font/file.ttf yourinputtext
```

It is similar to `hb-view` but uses HarfBuzz's own rasterizer and currently only outputs PPM images.

### hb-vector

*`hb-vector`* takes a font file and a text string, shapes the text, and renders the result as a SVG vector image.

```sh
hb-vector [OPTIONS] path/to/font/file.ttf yourinputtext
```

It is similar to `hb-view` but uses HarfBuzz's own vector output routines and currently only outputs SVG images.

### hb-subset

*`hb-subset`* allows you to generate a subset of a given font, with a limited set of supported characters, features, and variation settings.

By default, you provide an input font and an input text string as the arguments to `hb-subset`, and it will generate a font that covers the input text exactly like the input font does, but includes no other characters or features.

```sh
hb-subset [OPTIONS] path/to/font/file.ttf yourinputtext
```

For example, to create a subset of Noto Serif that just includes the numerals and the lowercase Latin alphabet, you could run

```sh
hb-subset [OPTIONS] NotoSerif-Regular.ttf 0123456789abcdefghijklmnopqrstuvwxyz
```

There are options available to remove hinting from the subsetted font and to specify a list of variation-axis settings.

### hb-info

*`hb-info`* allows you to query a font file for various information and prints it to the terminal.

If no query options are specified, it will output a general summary about the font.

```sh
hb-info [OPTIONS] path/to/font/file.ttf
```

Command-line options are available to list full tables, names, characters, layout features, font variation, or color palettes, as well as retrieve specific font names, metrics, styles, or baselines.

## Common data types and APIs

HarfBuzz includes several APIs for working with general-purpose data that you may find convenient to leverage in your own software. They include set operations and integer-to-integer mapping operations.

HarfBuzz uses set operations for internal bookkeeping, such as when it collects all of the glyph IDs covered by a particular font feature. You can also use the set API to build sets, add and remove elements, test whether or not sets contain particular elements, or compute the unions, intersections, or differences between sets.

All set elements are integers (specifically, `hb_codepoint_t` 32-bit unsigned ints), and there are functions for fetching the minimum and maximum element from a set. The set API also includes some functions that might not be part of a generic set facility, such as the ability to add a contiguous range of integer elements to a set in bulk, and the ability to fetch the next-smallest or next-largest element.

The HarfBuzz set API includes some conveniences as well. All sets are lifecycle-managed, just like other HarfBuzz objects. You increase the reference count on a set with `hb_set_reference()` and decrease it with `hb_set_destroy()`. You can also attach user data to a set, just like you can to blobs, buffers, faces, fonts, and other objects, and set destroy callbacks.

HarfBuzz also provides an API for keeping track of integer-to-integer mappings. As with the set API, each integer is stored as an unsigned 32-bit `hb_codepoint_t` element. Maps, like other objects, are reference counted with reference and destroy functions, and you can attach user data to them. The mapping operations include adding and deleting integer-to-integer key:value pairs to the map, testing for the presence of a key, fetching the population of the map, and so on.

There are several other internal HarfBuzz facilities that are exposed publicly and which you may want to take advantage of while processing text. HarfBuzz uses a common `hb_tag_t` for a variety of OpenType tag identifiers (for scripts, languages, font features, table names, variation-axis names, and more), and provides functions for converting strings to tags and vice-versa.

Finally, HarfBuzz also includes data type for Booleans, bit masks, and other simple types.
