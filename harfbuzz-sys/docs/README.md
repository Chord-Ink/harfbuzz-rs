# HarfBuzz 14.3.0 — C API Reference

The complete reference for the HarfBuzz C API, as exposed by the `harfbuzz-sys`
crate. Every page documents one public header: what each type is for, what each
function does, what it returns on failure, who owns what afterwards, and the
traps the header itself does not mention.

Every page is checked against its section in upstream's
`docs/harfbuzz-sections.txt`, so nothing documented upstream is missing here.

If you are writing Rust, you probably want the safe wrapper's documentation in
[`../../docs/`](../../docs/) instead. Come here when you need the underlying
detail, or when the safe API does not yet cover what you need.

## Start here

Never done text shaping before? Read these four, in order — about an hour:

1. [What is HarfBuzz?](guide/01-what-is-harfbuzz.md) — what shaping is and is not
2. [Shaping concepts](guide/04-shaping-concepts.md) — the vocabulary
3. [The object model](guide/05-object-model.md) — blob, face, font, buffer
4. [Getting started](guide/03-getting-started.md) — a first working program

Then read [Buffers](buffer.md) and [Clusters](guide/08-clusters.md). Between
them they cover most of what goes wrong in practice.

## The core path

These five headers are what a normal shaping program touches, in the order the
data flows through them:

| Page | What it covers |
| ---- | -------------- |
| [Blobs](blob.md) | Wrapping font bytes so HarfBuzz can track their lifetime |
| [Faces](face.md) | One font's tables, glyph inventory, and design units |
| [Fonts](font.md) | A face at a size, with variation axes pinned |
| [Buffers](buffer.md) | Text in, positioned glyphs out |
| [Shaping](shape.md) | The `hb_shape` call itself |

Supporting types: [Common types](common.md) (tags, directions, languages,
features, variations, colours), [Scripts](script.md),
[Unicode functions](unicode.md), [Shape plans](shape_plan.md),
[Sets](set.md), [Maps](map.md), [Version](version.md), [Style](style.md).

## OpenType

| Page | What it covers |
| ---- | -------------- |
| [OpenType layout](ot_layout.md) | `GSUB`/`GPOS`: querying scripts, languages, features, lookups |
| [OpenType variations](ot_var.md) | Variable-font axes, named instances, normalisation |
| [OpenType colour fonts](ot_color.md) | `COLR`/`CPAL`, `SVG`, and bitmap colour glyphs |
| [OpenType math](ot_math.md) | `MATH` table constants, italic correction, glyph assembly |
| [OpenType metrics](ot_metrics.md) | Ascender, descender, caret slope, and friends |
| [OpenType name table](ot_name.md) | Family, style, and designer strings |
| [OpenType metadata](ot_meta.md) | The `meta` table |
| [OpenType shaping](ot_shape.md) | Shaper-specific entry points |
| [OpenType font funcs](ot_font.md) | HarfBuzz's own font implementation |
| [OpenType table fetching](ot_fetch.md) | Reading raw table data |
| [AAT layout](aat_layout.md) | Apple Advanced Typography, the `morx`/`kerx` path |

## Glyph output

| Page | What it covers |
| ---- | -------------- |
| [Drawing glyph outlines](draw.md) | Callback-based outline extraction |
| [Painting colour glyphs](paint.md) | The COLRv1 paint model: transforms, gradients, compositing |
| [Rasterization](raster.md) | CPU rasterizing to a bitmap — needs the `raster` feature |
| [Vector output](vector.md) | SVG and PDF — needs the `vector` feature |
| [GPU outlines](gpu.md) | GPU-oriented outline data — needs the `gpu` feature |

## Subsetting

[Subsetting](subset.md) — cutting a font down to the glyphs you use, and
instancing variable fonts. Needs the `subset` feature. See also
[Subset preprocessing](guide/13-subset-preprocessing.md),
[the repacker](guide/14-repacker.md), and
[the serializer](guide/15-serializer.md).

## Integrations

Each needs the matching Cargo feature and a system library.

| Page | Feature |
| ---- | ------- |
| [CoreText](coretext.md) | `coretext` — Apple platforms |
| [FreeType](ft.md) | `freetype` |
| [Graphite2](graphite2.md) | `graphite2` |
| [ICU](icu.md) | `icu` |
| [GLib](glib.md) | `glib` |

## Deprecated

[Deprecated API](deprecated.md) and
[Deprecated OpenType API](ot_deprecated.md). Still exported, still linkable,
but each entry names its replacement.

## Guides

Conceptual chapters, adapted from HarfBuzz's own user manual.

| Guide | What it covers |
| ----- | -------------- |
| [01 What is HarfBuzz?](guide/01-what-is-harfbuzz.md) | Scope, history, what shaping does not do |
| [02 Installing](guide/02-installing.md) | Building upstream (this crate vendors it) |
| [03 Getting started](guide/03-getting-started.md) | A first shaping program |
| [04 Shaping concepts](guide/04-shaping-concepts.md) | Scripts, complex shaping, positioning |
| [05 The object model](guide/05-object-model.md) | Lifecycle, refcounting, immutability, user data |
| [06 Buffers, language, script, direction](guide/06-buffers-and-segment-properties.md) | Segment properties in depth |
| [07 Fonts and faces](guide/07-fonts-and-faces.md) | Faces, fonts, funcs, output |
| [08 Clusters](guide/08-clusters.md) | Cluster levels, and mapping glyphs back to text |
| [09 OpenType features](guide/09-opentype-features.md) | What features are and how to apply them |
| [10 Glyph information](guide/10-glyph-information.md) | Reading the shaped output |
| [11 Integration](guide/11-integration.md) | FreeType, CoreText, Uniscribe, ICU |
| [12 Utilities](guide/12-utilities.md) | `hb-shape`, `hb-view`, and friends |
| [13 Subset preprocessing](guide/13-subset-preprocessing.md) | Speeding up repeated subsetting |
| [14 The repacker](guide/14-repacker.md) | Fitting rewritten layout tables into offsets |
| [15 The serializer](guide/15-serializer.md) | How subset output is assembled |
| [16 The dependency API](guide/16-depend-api.md) | Glyph dependency queries |
| [17 Dependency internals](guide/17-depend-implementation.md) | How they are computed |
| [18 Dependencies for closure](guide/18-depend-for-closure.md) | Using them for glyph closure |
| [19 The WebAssembly shaper](guide/19-wasm-shaper.md) | Shaping logic shipped inside a font |

## Orientation: the object model in one page

Four objects, and the data flows through them in one direction:

```
bytes ──► hb_blob_t ──► hb_face_t ──► hb_font_t ──┐
                                                  ├──► hb_shape() ──► glyphs
text ─────────────────► hb_buffer_t ──────────────┘
```

* A **blob** is bytes plus a destroy callback. It exists so HarfBuzz can hold
  font data without guessing who frees it.
* A **face** is one font parsed out of a blob: tables, glyph count, units-per-em.
  A `.ttc` collection holds several, selected by index.
* A **font** is a face *at a size*, with variation axes pinned. Shaping needs
  one because advances are meaningless without a scale.
* A **buffer** holds Unicode code points before shaping and glyphs after.
  Shaping rewrites it in place.

Every one of these is reference counted, created with a count of one, and
released with `hb_*_destroy()`. Counting is atomic, so lifetimes are thread-safe
even where the object's contents are not.

Constructors **never return null**. On allocation failure they return an inert
"empty" singleton instead, so calling code does not have to null-check every
step. When you would rather know, use the `_or_fail` variant where one exists.

Most objects can be marked immutable with `hb_*_make_immutable()`. After that,
setters **fail silently**, and there is no way back — you duplicate the object
instead. This is the pattern to follow: create, configure, freeze, then share.

## Licence

The prose here is adapted from HarfBuzz's own documentation and from the
gtk-doc comments in its headers, under
[HarfBuzz's licence](../harfbuzz/COPYING).
