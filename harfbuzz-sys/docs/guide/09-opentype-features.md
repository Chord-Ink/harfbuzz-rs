# OpenType features

*Adapted from the HarfBuzz 14.3.0 documentation ("Shaping and shape plans"), which is licensed under the terms in the HarfBuzz COPYING file.*

Once you have your face and font objects configured as desired and your input buffer is filled with the characters you need to shape, all you need to do is call `hb_shape()`.

HarfBuzz will return the shaped version of the text in the same buffer that you provided, but it will be in output mode. At that point, you can iterate through the glyphs in the buffer, drawing each one at the specified position or handing them off to the appropriate graphics library.

For the most part, HarfBuzz's shaping step is straightforward from the outside. But that doesn't mean there will never be cases where you want to look under the hood and see what is happening on the inside. HarfBuzz provides facilities for doing that, too.

## Shaping and buffer output

The `hb_shape()` function call takes four arguments: the font object to use, the buffer of characters to shape, an array of user-specified features to apply, and the length of that feature array. The feature array can be NULL, so for the sake of simplicity we will start with that case.

Internally, HarfBuzz looks at the tables of the font file to determine where glyph classes, substitutions, and positioning are defined, using that information to decide which *shaper* to use (`ot` for OpenType fonts, `aat` for Apple Advanced Typography fonts, and so on). It also looks at the direction, script, and language properties of the segment to figure out which script-specific shaping model is needed (at least, in shapers that support multiple options).

If a font has a GDEF table, then that is used for glyph classes; if not, HarfBuzz will fall back to Unicode categorization by code point. If a font has an AAT `morx` table, then it is used for substitutions; if not, but there is a GSUB table, then the GSUB table is used. If the font has an AAT `kerx` table, then it is used for positioning; if not, but there is a GPOS table, then the GPOS table is used. If neither table is found, but there is a `kern` table, then HarfBuzz will use the `kern` table. If there is no `kerx`, no GPOS, and no `kern`, HarfBuzz will fall back to positioning marks itself.

With a well-behaved OpenType font, you expect GDEF, GSUB, and GPOS tables to all be applied. HarfBuzz implements the script-specific shaping models in internal functions, rather than in the public API.

The algorithms used for shaping can be quite involved; HarfBuzz tries to be compatible with the OpenType Layout specification and, wherever there is any ambiguity, HarfBuzz attempts to replicate the output of Microsoft's Uniscribe engine, to the extent that is feasible and desirable. See the [Microsoft Typography pages](https://docs.microsoft.com/en-us/typography/script-development/standard) for more detail.

In general, though, all that you need to know is that `hb_shape()` returns the results of shaping in the same buffer that you provided. The buffer's content type will now be set to `HB_BUFFER_CONTENT_TYPE_GLYPHS`, indicating that it contains shaped output, rather than input text. You can now extract the glyph information and positioning arrays:

```c
hb_glyph_info_t *glyph_info    = hb_buffer_get_glyph_infos(buf, &glyph_count);
hb_glyph_position_t *glyph_pos = hb_buffer_get_glyph_positions(buf, &glyph_count);
```

The glyph information array holds a `hb_glyph_info_t` for each output glyph, which has two fields: `codepoint` and `cluster`. Whereas, in the input buffer, the `codepoint` field contained the Unicode code point, it now contains the glyph ID of the corresponding glyph in the font. The `cluster` field is an integer that you can use to help identify when shaping has reordered, split, or combined code points; we will say more about that in the Clusters guide.

The glyph positions array holds a corresponding `hb_glyph_position_t` for each output glyph, containing four fields: `x_advance`, `y_advance`, `x_offset`, and `y_offset`. The advances tell you how far you need to move the drawing point after drawing this glyph, depending on whether you are setting horizontal text (in which case you will have x advances) or vertical text (for which you will have y advances). The x and y offsets tell you where to move to start drawing the glyph; usually you will have both and x and a y offset, regardless of the text direction.

Most of the time, you will rely on a font-rendering library or other graphics library to do the actual drawing of glyphs, so you will need to iterate through the glyphs in the buffer and pass the corresponding values off.

## OpenType features

OpenType features enable fonts to include smart behavior, implemented as "lookup" rules stored in the GSUB and GPOS tables. The OpenType specification defines a long list of standard features that fonts can use for these behaviors; each feature has a four-character reserved name and a well-defined semantic meaning.

Some OpenType features are defined for the purpose of supporting script-specific shaping, and are automatically activated, but only when a buffer's script property is set to a script that the feature supports.

Other features are more generic and can apply to several (or any) script, and shaping engines are expected to implement them. By default, HarfBuzz activates several of these features on every text run. They include `abvm`, `blwm`, `ccmp`, `locl`, `mark`, `mkmk`, and `rlig`.

In addition, if the text direction is horizontal, HarfBuzz also applies the `calt`, `clig`, `curs`, `dist`, `kern`, `liga`, and `rclt` features.

Additionally, when HarfBuzz encounters a fraction slash (`U+2044`), it looks backward and forward for decimal digits (Unicode General Category = Nd), and enables features `numr` on the sequence before the fraction slash, `dnom` on the sequence after the fraction slash, and `frac` on the whole sequence including the fraction slash.

Some script-specific shaping models (see the OpenType shaping models section of the Shaping concepts guide) disable some of the features listed above:

- Hangul: `calt`
- Indic: `liga`
- Khmer: `liga`

If the text direction is vertical, HarfBuzz applies the `vert` feature by default.

Still other features are designed to be purely optional and left up to the application or the end user to enable or disable as desired.

You can adjust the set of features that HarfBuzz applies to a buffer by supplying an array of `hb_feature_t` features as the third argument to `hb_shape()`. For a simple case, let's just enable the `dlig` feature, which turns on any "discretionary" ligatures in the font:

```c
hb_feature_t userfeatures[1];
userfeatures[0].tag = HB_TAG('d','l','i','g');
userfeatures[0].value = 1;
userfeatures[0].start = HB_FEATURE_GLOBAL_START;
userfeatures[0].end = HB_FEATURE_GLOBAL_END;
```

`HB_FEATURE_GLOBAL_END` and `HB_FEATURE_GLOBAL_END` are macros we can use to indicate that the features will be applied to the entire buffer. We could also have used a literal `0` for the start and a `-1` to indicate the end of the buffer (or have selected other start and end positions, if needed).

When we pass the `userfeatures` array to `hb_shape()`, any discretionary ligature substitutions from our font that match the text in our buffer will get performed:

```c
hb_shape(font, buf, userfeatures, num_features);
```

Just like we enabled the `dlig` feature by setting its `value` to `1`, you would disable a feature by setting its `value` to `0`. Some features can take other `value` settings; be sure you read the full specification of each feature tag to understand what it does and how to control it.

## Shaper selection

The basic version of `hb_shape()` determines its shaping strategy based on examining the capabilities of the font file. OpenType font tables cause HarfBuzz to try the `ot` shaper, while AAT font tables cause HarfBuzz to try the `aat` shaper.

In the real world, however, a font might include some unusual mix of tables, or one of the tables might simply be broken for the script you need to shape. So, sometimes, you might not want to rely on HarfBuzz's process for deciding what to do, and just tell `hb_shape()` what you want it to try.

`hb_shape_full()` is an alternate shaping function that lets you supply a list of shapers for HarfBuzz to try, in order, when shaping your buffer. For example, if you have determined that HarfBuzz's attempts to work around broken tables gives you better results than the AAT shaper itself does, you might move the AAT shaper to the end of your list of preferences and call `hb_shape_full()`

```c
char *shaperprefs[3] = {"ot", "default", "aat"};
...
hb_shape_full(font, buf, userfeatures, num_features, shaperprefs);
```

to get results you are happier with.

You may also want to call `hb_shape_list_shapers()` to get a list of the shapers that were built at compile time in your copy of HarfBuzz.

## Plans and caching

Internally, HarfBuzz uses a structure called a shape plan to track its decisions about how to shape the contents of a buffer. The `hb_shape()` function builds up the shape plan by examining segment properties and by inspecting the contents of the font.

This process can involve some decision-making and trade-offs — for example, HarfBuzz inspects the GSUB and GPOS lookups for the script and language tags set on the segment properties, but it falls back on the lookups under the `DFLT` tag (and sometimes other common tags) if there are actually no lookups for the tag requested.

HarfBuzz also includes some work-arounds for handling well-known older font conventions that do not follow OpenType or Unicode specifications, for buggy system fonts, and for peculiarities of Microsoft Uniscribe. All of that means that a shape plan, while not something that you should edit directly in client code, still might be an object that you want to inspect. Furthermore, if resources are tight, you might want to cache the shape plan that HarfBuzz builds for your buffer and font, so that you do not have to rebuild it for every shaping call.

You can create a cacheable shape plan with `hb_shape_plan_create_cached(face, props, user_features, num_user_features, shaper_list)`, where `face` is a face object (not a font object, notably), `props` is an `hb_segment_properties_t`, `user_features` is an array of `hb_feature_t`s (with length `num_user_features`), and `shaper_list` is a list of shapers to try.

Shape plans are objects in HarfBuzz, so there are reference-counting functions and user-data attachment functions you can use. `hb_shape_plan_reference(shape_plan)` increases the reference count on a shape plan, while `hb_shape_plan_destroy(shape_plan)` decreases the reference count, destroying the shape plan when the last reference is dropped.

You can attach user data to a shaper (with a key) using the `hb_shape_plan_set_user_data(shape_plan,key,data,destroy,replace)` function, optionally supplying a `destroy` callback to use. You can then fetch the user data attached to a shape plan with `hb_shape_plan_get_user_data(shape_plan, key)`.
