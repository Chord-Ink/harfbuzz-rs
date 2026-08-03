# Shaping concepts

*Adapted from the HarfBuzz 14.3.0 documentation, which is licensed under the terms in the HarfBuzz COPYING file.*

## Text shaping

Text shaping is the process of transforming a sequence of Unicode codepoints that represent individual characters (letters, diacritics, tone marks, numbers, symbols, etc.) into the orthographically and linguistically correct two-dimensional layout of glyph shapes taken from a specified font.

For some writing systems (or *scripts*) and languages, the process is simple, requiring the shaper to do little more than advance the horizontal position forward by the correct amount for each successive glyph.

But, for other scripts (often unceremoniously called *complex scripts*), any combination of several shaping operations may be required, and the rules for how and when they are applied vary from script to script. HarfBuzz and other shaping engines implement these rules.

The exact rules and necessary operations for a particular script constitute a shaping *model*. OpenType specifies a set of shaping models that covers all of Unicode. Other shaping models are available, however, including Graphite and Apple Advanced Typography (AAT).

## Script-specific shaping

In many scripts, transforming the input sequence into the final layout often requires some combination of operations — such as context-dependent substitutions, context-dependent mark positioning, glyph-to-glyph joining, glyph reordering, or glyph stacking.

In some scripts, the shaping rules require that a text run be divided into syllables before the operations can be applied. Other scripts may apply shaping operations over entire words or over the entire text run, with no subdivision required.

Other scripts, do not require these operations. However, correctly shaping a text run in any script may still involve Unicode normalization, ligature substitutions, mark positioning, kerning, and applying other font features.

## Shaping operations

Shaping a text run involves transforming the input sequence of Unicode codepoints with some combination of operations that is specified in the shaping model for the script.

The specific conditions that trigger a given operation for a text run varies from script to script, as do the order that the operations are performed in and which codepoints are affected. However, the same general set of shaping operations is common to all of the script shaping models.

- A *reordering* operation moves a glyph from its original ("logical") position in the sequence to some other ("visual") position.

  The shaping model for a given script might involve more than one reordering step.

- A *joining* operation replaces a glyph with an alternate form that is designed to connect with one or more of the adjacent glyphs in the sequence.

- A contextual *substitution* operation replaces either a single glyph or a subsequence of several glyphs with an alternate glyph. This substitution is performed when the original glyph or subsequence of glyphs occurs in a specified position with respect to the surrounding sequence. For example, one substitution might be performed only when the target glyph is the first glyph in the sequence, while another substitution is performed only when a different target glyph occurs immediately after a particular string pattern.

  The shaping model for a given script might involve multiple contextual-substitution operations, each applying to different target glyphs and patterns, and which are performed in separate steps.

- A contextual *positioning* operation moves the horizontal and/or vertical position of a glyph. This positioning move is performed when the glyph occurs in a specified position with respect to the surrounding sequence.

  Many contextual positioning operations are used to place *mark* glyphs (such as diacritics, vowel signs, and tone markers) with respect to *base* glyphs. However, some scripts may use contextual positioning operations to correctly place base glyphs as well, such as when the script uses *stacking* characters.

## Unicode character categories

Shaping models are typically specified with respect to how scripts are defined in the Unicode standard.

Every codepoint in the Unicode Character Database (UCD) is assigned a *Unicode General Category* (UGC), which provides the most fundamental information about the codepoint: whether the codepoint represents a *Letter*, a *Mark*, a *Number*, *Punctuation*, a *Symbol*, a *Separator*, or something else (*Other*).

These UGC properties are "Major" categories. Each codepoint is further assigned to a "minor" category within its Major category, such as "Letter, uppercase" (`Lu`) or "Letter, modifier" (`Lm`).

Shaping models are concerned primarily with Letter and Mark codepoints. The minor categories of Mark codepoints are particularly important for shaping. Marks can be nonspacing (`Mn`), spacing combining (`Mc`), or enclosing (`Me`).

In addition to the UGC property, codepoints in the Indic and Southeast Asian scripts are also assigned *Unicode Indic Syllabic Category* (UISC) and *Unicode Indic Positional Category* (UIPC) properties that provide more detailed information needed for shaping.

The UISC property sub-categorizes Letters and Marks according to common script-shaping behaviors. For example, UISC distinguishes between consonant letters, vowel letters, and vowel marks. The UIPC property sub-categorizes Mark codepoints by the relative visual position that they occupy (above, below, right, left, or in multiple positions).

Some scripts require that the text run be split into syllables. What constitutes a valid syllable in these scripts is specified in regular expressions, formed from the Letter and Mark codepoints, that take the UISC and UIPC properties into account.

## Text runs

Real-world text usually contains codepoints from a mixture of different Unicode scripts (including punctuation, numbers, symbols, white-space characters, and other codepoints that do not belong to any script). Real-world text may also be marked up with formatting that changes font properties (including the font, font style, and font size).

For shaping purposes, all real-world text streams must be first segmented into runs that have a uniform set of properties.

In particular, shaping models always assume that every codepoint in a text run has the same *direction*, *script* tag, and *language* tag.

## OpenType shaping models

OpenType provides shaping models for the following scripts:

- The *default* shaping model handles all scripts with no script-specific shaping model, and may also be used as a fallback for handling unrecognized scripts.

- The *Indic* shaping model handles the Indic scripts Bengali, Devanagari, Gujarati, Gurmukhi, Kannada, Malayalam, Oriya, Tamil, and Telugu.

  The Indic shaping model was revised significantly in 2005. To denote the change, a new set of *script tags* was assigned for Bengali, Devanagari, Gujarati, Gurmukhi, Kannada, Malayalam, Oriya, Tamil, and Telugu. For the sake of clarity, the term "Indic2" is sometimes used to refer to the current, revised shaping model.

- The *Arabic* shaping model supports Arabic, Mongolian, N'Ko, Syriac, and several other connected or cursive scripts.

- The *Thai/Lao* shaping model supports the Thai and Lao scripts.

- The *Khmer* shaping model supports the Khmer script.

- The *Myanmar* shaping model supports the Myanmar (or Burmese) script.

- The *Tibetan* shaping model supports the Tibetan script.

- The *Hangul* shaping model supports the Hangul script.

- The *Hebrew* shaping model supports the Hebrew script.

- The *Universal Shaping Engine* (USE) shaping model supports scripts not covered by one of the above, script-specific shaping models, including Javanese, Balinese, Buginese, Batak, Chakma, Lepcha, Modi, Phags-pa, Tagalog, Siddham, Sundanese, Tai Le, Tai Tham, Tai Viet, and many others.

- Text runs that do not fall under one of the above shaping models may still require processing by a shaping engine. Of particular note is *Emoji* shaping, which may involve variation-selector sequences and glyph substitution. Emoji shaping is handled by the default shaping model.

## Graphite shaping

In contrast to OpenType shaping, Graphite shaping does not specify a predefined set of shaping models or a set of supported scripts.

Instead, each Graphite font contains a complete set of rules that implement the required shaping model for the intended script. These rules include finite-state machines to match sequences of codepoints to the shaping operations to perform.

Graphite shaping can perform the same shaping operations used in OpenType shaping, as well as other functions that have not been defined for OpenType shaping.

## AAT shaping

In contrast to OpenType shaping, AAT shaping does not specify a predefined set of shaping models or a set of supported scripts.

Instead, each AAT font includes a complete set of rules that implement the desired shaping model for the intended script. These rules include finite-state machines to match glyph sequences and the shaping operations to perform.

Notably, AAT shaping rules are expressed for glyphs in the font, not for Unicode codepoints. AAT shaping can perform the same shaping operations used in OpenType shaping, as well as other functions that have not been defined for OpenType shaping.
