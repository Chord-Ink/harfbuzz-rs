# What is HarfBuzz?

*Adapted from the HarfBuzz 14.3.0 documentation, which is licensed under the terms in the HarfBuzz COPYING file.*

HarfBuzz is a *text-shaping engine*. If you give HarfBuzz a font and a string containing a sequence of Unicode codepoints, HarfBuzz selects and positions the corresponding glyphs from the font, applying all of the necessary layout rules and font features. HarfBuzz then returns the string to you in the form that is correctly arranged for the language and writing system.

HarfBuzz can properly shape all of the world's major writing systems. It runs on all major operating systems and software platforms and it supports the major font formats in use today.

## What is text shaping?

Text shaping is the process of translating a string of character codes (such as Unicode codepoints) into a properly arranged sequence of glyphs that can be rendered onto a screen or into final output form for inclusion in a document.

The shaping process is dependent on the input string, the active font, the script (or writing system) that the string is in, and the language that the string is in.

Modern software systems generally only deal with strings in the Unicode encoding scheme (although legacy systems and documents may involve other encodings).

There are several font formats that a program might encounter, each of which has a set of standard text-shaping rules.

The dominant format is [OpenType](http://www.microsoft.com/typography/otspec/). The OpenType specification defines a series of [shaping models](https://github.com/n8willis/opentype-shaping-documents) for various scripts from around the world. These shaping models depend on the font incorporating certain features as *lookups* in its `GSUB` and `GPOS` tables.

Alternatively, OpenType fonts can include shaping features for the [Graphite](https://graphite.sil.org/) shaping model.

TrueType fonts can also include OpenType shaping features. Alternatively, TrueType fonts can also include [Apple Advanced Typography](https://developer.apple.com/fonts/TrueType-Reference-Manual/RM09/AppendixF.html) (AAT) tables to implement shaping support. AAT fonts are generally only found on macOS and iOS systems.

Text strings will usually be tagged with a script and language tag that provide the context needed to perform text shaping correctly. The necessary [script](https://docs.microsoft.com/en-us/typography/opentype/spec/scripttags) and [language](https://docs.microsoft.com/en-us/typography/opentype/spec/languagetags) tags are defined by OpenType.

## Why do I need a shaping engine?

Text shaping is an integral part of preparing text for display. Before a Unicode sequence can be rendered, the codepoints in the sequence must be mapped to the corresponding glyphs provided in the font, and those glyphs must be positioned correctly relative to each other. For many of the scripts supported in Unicode, these steps involve script-specific layout rules, including complex joining, reordering, and positioning behavior. Implementing these rules is the job of the shaping engine.

Text shaping is a fairly low-level operation. HarfBuzz is used directly by text-handling libraries like [Pango](https://www.pango.org/), as well as by the layout engines in Firefox, LibreOffice, and Chromium. Unless you are *writing* one of these layout engines yourself, you will probably not need to use HarfBuzz: normally, a layout engine, toolkit, or other library will turn text into glyphs for you.

However, if you *are* writing a layout engine or graphics library yourself, then you will need to perform text shaping, and this is where HarfBuzz can help you.

Here are some specific scenarios where a text-shaping engine like HarfBuzz helps you:

- OpenType fonts contain a set of glyphs (that is, shapes to represent the letters, numbers, punctuation marks, and all other symbols), which are indexed by a `glyph ID`.

  A particular glyph ID within the font does not necessarily correlate to a predictable Unicode codepoint. For instance, some fonts have the letter "a" as glyph ID 1, but many others do not. In order to retrieve the right glyph from the font to display "a", you need to consult the table inside the font (the `cmap` table) that maps Unicode codepoints to glyph IDs. In other words, *text shaping turns codepoints into glyph IDs*.

- Many OpenType fonts contain ligatures: combinations of characters that are rendered as a single unit. For instance, it is common for the "f, i" letter sequence to appear in print as the single ligature glyph "ﬁ".

  Whether you should render an "f, i" sequence as `fi` or as "ﬁ" does not depend on the input text. Instead, it depends on the whether or not the font includes an "ﬁ" glyph and on the level of ligature application you wish to perform. The font and the amount of ligature application used are under your control. In other words, *text shaping involves querying the font's ligature tables and determining what substitutions should be made*.

- While ligatures like "ﬁ" are optional typographic refinements, some languages *require* certain substitutions to be made in order to display text correctly.

  For example, in Tamil, when the letter "TTA" (ட) letter is followed by the vowel sign "U" (ு), the pair must be replaced by the single glyph "டு". The sequence of Unicode characters "ட,ு" needs to be substituted with a single "டு" glyph from the font.

  But "டு" does not have a Unicode codepoint. To find this glyph, you need to consult the table inside the font (the `GSUB` table) that contains substitution information. In other words, *text shaping chooses the correct glyph for a sequence of characters provided*.

- Similarly, each Arabic character has four different variants corresponding to the different positions it might appear in within a sequence. Inside a font, there will be separate glyphs for the initial, medial, final, and isolated forms of each letter, each at a different glyph ID.

  Unicode only assigns one codepoint per character, so a Unicode string will not tell you which glyph variant to use for each character. To decide, you need to analyze the whole string and determine the appropriate glyph for each character based on its position. In other words, *text shaping chooses the correct form of the letter by its position and returns the correct glyph from the font*.

- Other languages involve marks and accents that need to be rendered in specific positions relative a base character. For instance, the Moldovan language includes the Cyrillic letter "zhe" (ж) with a breve accent, like so: "ӂ".

  Some fonts will provide this character as a single zhe-with-breve glyph, but other fonts will not and, instead, will expect the rendering engine to form the character by superimposing the separate "ж" and "˘" glyphs.

  But exactly where you should draw the breve depends on the height and width of the preceding zhe glyph. To find the right position, you need to consult the table inside the font (the `GPOS` table) that contains positioning information. In other words, *text shaping tells you whether you have a precomposed glyph within your font or if you need to compose a glyph yourself out of combining marks—and, if so, where to position those marks.*

If tasks like these are something that you need to do, then you need a text shaping engine. You could use Uniscribe if you are writing Windows software; you could use CoreText on macOS; or you could use HarfBuzz.

> **Note**
>
> In the rest of this manual, the text will assume that the reader is that implementor of a text-layout engine.

## What does HarfBuzz do?

HarfBuzz provides text shaping through a cross-platform C API that accepts sequences of Unicode codepoints as input. Currently, the following OpenType shaping models are supported:

- Indic (covering Devanagari, Bengali, Gujarati, Gurmukhi, Kannada, Malayalam, Oriya, Tamil, and Telugu)
- Arabic (covering Arabic, N'Ko, Syriac, and Mongolian)
- Thai and Lao
- Khmer
- Myanmar
- Tibetan
- Hangul
- Hebrew
- The Universal Shaping Engine or *USE* (covering complex scripts not covered by the above shaping models)
- A default shaping model for non-complex scripts (covering Latin, Cyrillic, Greek, Armenian, Georgian, Tifinagh, and many others)
- Emoji (including emoji modifier sequences, flag sequences, and ZWJ sequences)

In addition to OpenType shaping, HarfBuzz supports the latest version of Graphite shaping (the "Graphite 2" model) and AAT shaping.

HarfBuzz can read and understand TrueType fonts (.ttf), TrueType collections (.ttc), and OpenType fonts (.otf, including those fonts that contain TrueType-style outlines and those that contain PostScript CFF or CFF2 outlines).

HarfBuzz is designed and tested to run on top of the FreeType font renderer. It can run on Linux, Android, Windows, macOS, and iOS systems.

In addition to its core shaping functionality, HarfBuzz provides functions for accessing other font features, including optional GSUB and GPOS OpenType features, as well as all color-font formats (`CBDT`, `sbix`, `COLR/CPAL`, and `SVG-OT`) and OpenType variable fonts. HarfBuzz also includes a font-subsetting feature. HarfBuzz can perform some low-level math-shaping operations, although it does not currently perform full shaping for mathematical typesetting.

A suite of command-line utilities is also provided in the source-code tree, designed to help users test and debug HarfBuzz's features on real-world fonts and input.

## What HarfBuzz doesn't do

HarfBuzz will take a Unicode string, shape it, and give you the information required to lay it out correctly on a single horizontal (or vertical) line using the font provided. That is the extent of HarfBuzz's responsibility.

It is important to note that if you are implementing a complete text-layout engine you may have other responsibilities that HarfBuzz will *not* help you with. For example:

- HarfBuzz won't help you with bidirectionality. If you want to lay out text that includes a mix of Hebrew and English, you will need to ensure that each buffer provided to HarfBuzz has all of its characters in the same order and that the directionality of the buffer is set correctly. This may mean segmenting the text before it is placed into HarfBuzz buffers. In other words, the user will hit the keys in the following sequence:

  ```text
  A B C [space] ג ב א [space] D E F
  ```

  but will expect to see in the output:

  ```text
  ABC אבג DEF
  ```

  This reordering is called *bidi processing* ("bidi" is short for bidirectional), and there's an algorithm as an annex to the Unicode Standard which tells you how to process a string of mixed directionality. Before sending your string to HarfBuzz, you may need to apply the bidi algorithm to it. Libraries such as [ICU](http://icu-project.org/) and [fribidi](http://fribidi.org/) can do this for you.

- HarfBuzz won't help you with text that contains different font properties. For instance, if you have the string "a *huge* breakfast", and you expect "huge" to be italic, then you will need to send three strings to HarfBuzz: `a`, in your Roman font; `huge` using your italic font; and `breakfast` using your Roman font again.

  Similarly, if you change the font, font size, script, language, or direction within your string, then you will need to shape each run independently and output them independently. HarfBuzz expects to shape a run of characters that all share the same properties.

- HarfBuzz won't help you with line breaking, hyphenation, or justification. As mentioned above, HarfBuzz lays out the string along a *single line* of, notionally, infinite length. If you want to find out where the potential word, sentence and line break points are in your text, you could use the ICU library's break iterator functions.

  HarfBuzz can tell you how wide a shaped piece of text is, which is useful input to a justification algorithm, but it knows nothing about paragraphs, lines or line lengths. Nor will it adjust the space between words to fit them proportionally into a line.

As a layout-engine implementor, HarfBuzz will help you with the interface between your text and your font, and that's something that you'll need—what you then do with the glyphs that your font returns is up to you.

## Why is it called HarfBuzz?

HarfBuzz began its life as text-shaping code within the FreeType project (and you will see references to the FreeType authors within the source code copyright declarations), but was then extracted out to its own project. This project is maintained by Behdad Esfahbod, who named it HarfBuzz. Originally, it was a shaping engine for OpenType fonts—"HarfBuzz" is the Persian for "open type".
