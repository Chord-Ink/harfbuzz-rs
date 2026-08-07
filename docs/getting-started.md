# Getting started

From zero to positioned glyphs. By the end of this page you will have loaded a
font file, sized it, shaped a string, and turned the result into coordinates you
could hand to a renderer.

- [The concepts, briefly](#the-concepts-briefly)
- [Step 1 — load a face](#step-1--load-a-face)
- [Step 2 — freeze it](#step-2--freeze-it)
- [Step 3 — make a font at a size](#step-3--make-a-font-at-a-size)
- [Step 4 — shape](#step-4--shape)
- [Step 5 — read the output](#step-5--read-the-output)
- [Step 6 — pen positions](#step-6--pen-positions)
- [Measuring without drawing](#measuring-without-drawing)
- [Shaping many runs](#shaping-many-runs)
- [The complete program](#the-complete-program)
- [Where to go next](#where-to-go-next)

---

## The concepts, briefly

If you have never done text shaping, five terms carry most of the weight.

**Character vs. glyph.** A character is a Unicode code point — `'A'`, `U+0041`.
A glyph is a drawing in a font, identified by a number that means nothing
outside that font. The mapping between them is not one-to-one: `f` + `i` may
become a single `fi` ligature glyph, `é` may become two glyphs (an `e` and an
acute accent), and Arabic letters pick one of four shapes depending on their
neighbours. Shaping is the process that decides.

**Advance.** How far the pen moves after drawing a glyph. Add advances together
and you have the width of a run of text.

**Offset.** How far a glyph is displaced from the pen *for drawing only* —
used to position accents over letters. Offsets do not accumulate; advances do.

**Cluster.** The link back from output to input. Every glyph carries the byte
offset of the input text it came from. Several glyphs can share one cluster
value (a decomposed accent), and one glyph can cover several input characters (a
ligature). Clusters are how you place a cursor or hit-test a click. See
[text-and-buffers.md](text-and-buffers.md#clusters).

**Units per em (upem).** Fonts draw their outlines on an abstract grid — 1000
units per em for CFF fonts, 1024 or 2048 for TrueType. Shaped positions come
back scaled from that grid to whatever unit you asked for. See
[fonts-and-sizing.md](fonts-and-sizing.md#units-per-em).

---

## Step 1 — load a face

A **face** is one typeface out of a font file: its tables, its glyph inventory,
its design grid. No size is attached yet.

```rust
use harfbuzz_rs::Face;

fn main() -> harfbuzz_rs::Result<()> {
    // The second argument selects a face inside a `.ttc` collection.
    // Pass 0 for an ordinary single-face `.ttf` or `.otf`.
    let face = Face::from_file("font.ttf", 0)?;

    println!("{} glyphs, {} units per em", face.glyph_count(), face.upem());
    Ok(())
}
```

Three constructors, depending on where the bytes are:

| Constructor | Use when |
| --- | --- |
| `Face::from_file(path, index)` | You have a path. HarfBuzz memory-maps the file where it can, so this is cheaper than reading it yourself. |
| `Face::from_bytes(bytes, index)` | The font is embedded (`include_bytes!`), downloaded, or already in memory. Takes anything convertible to `Arc<[u8]>` — `Vec<u8>`, `Box<[u8]>`, `Arc<[u8]>` — and moves it into HarfBuzz without copying. |
| `Face::new(&blob, index)` | You want several faces out of one collection file, and would rather not re-read the bytes for each. See [`Blob`](object-model.md#blob). |

All three return [`Result`](errors.md): the file may be missing, the bytes may
not be a font, or the index may be past the end of a collection.

## Step 2 — freeze it

```rust
use harfbuzz_rs::{Face, IntoShared};

fn main() -> harfbuzz_rs::Result<()> {
    let face = Face::from_file("font.ttf", 0)?.into_shared();
    Ok(())
}
```

`into_shared()` marks the face immutable and hands back a `Shared<Face>`, which
is `Clone`, `Send`, and `Sync`. Cloning it costs one atomic increment, not a
copy of the font.

This is required, not optional: `Font::new` only accepts a `Shared<Face>`. The
reason is that HarfBuzz's setters *silently do nothing* once an object has been
frozen, and freezing is exactly what building a font from a face does. Making
the frozen state a distinct Rust type turns "your `set_upem` call was ignored"
into a compile error. [ownership-and-threads.md](ownership-and-threads.md)
explains the whole design.

If you do want to change something on the face — `set_upem`, `set_glyph_count`
— do it *before* `into_shared()`, while you still hold the unique handle:

```rust
use harfbuzz_rs::{Face, IntoShared};

fn main() -> harfbuzz_rs::Result<()> {
    let mut face = Face::from_file("font.ttf", 0)?;
    face.set_upem(1000);

    let face = face.into_shared();
    // face.set_upem(2048);   <- does not compile
    Ok(())
}
```

## Step 3 — make a font at a size

A **font** is a face plus a size, plus a position on each variable axis. Shaping
needs a font rather than a face because advances are meaningless without a
scale.

```rust
use harfbuzz_rs::{Face, Font, IntoShared, points_to_scale};

fn main() -> harfbuzz_rs::Result<()> {
    let face = Face::from_file("font.ttf", 0)?.into_shared();

    let mut font = Font::new(face);
    font.set_scale(points_to_scale(16), points_to_scale(16));

    Ok(())
}
```

`set_scale` says *how many output units there are per em*. Two conventions
cover almost everything:

| You want positions in | Set the scale to | Notes |
| --- | --- | --- |
| Font design units | `face.upem()` | This is the default for a new font — call nothing. Positions are resolution-independent integers. |
| 26.6 fixed-point pixels | `pixels_per_em * 64` | Divide the results by `64.0` to get pixels. `points_to_scale(n)` is just `n * 64`. |

`points_to_scale(16)` gives `1024`, which is 16 pixels per em in 26.6 — correct
when one point is one pixel (72 dpi). On a 2× display, use
`points_to_scale(32)`, or write `px_per_em * 64` directly. Full detail in
[fonts-and-sizing.md](fonts-and-sizing.md#scale-and-the-266-convention).

Fonts are cheap; faces are not. Build one shared face and as many fonts from it
as you have sizes.

## Step 4 — shape

```rust
use harfbuzz_rs::{Face, Font, GlyphBuffer, IntoShared, buffer_from, shape};

fn main() -> harfbuzz_rs::Result<()> {
    let font = Font::new(Face::from_file("font.ttf", 0)?.into_shared());

    let output: GlyphBuffer = shape(&font, buffer_from("Hello")?, &[]);

    println!("{} glyphs", output.len());
    Ok(())
}
```

`shape` takes the font, a `Buffer` **by value**, and a slice of
[`Feature`](values.md#feature) overrides. It hands back a `GlyphBuffer`: the
same underlying object, rewritten in place, with a type that only exposes the
post-shaping accessors. Pass `&[]` for features unless you have a reason not to
— the font's own defaults are almost always what you want.

`buffer_from(text)` is shorthand for the three lines you would otherwise write:

```rust
use harfbuzz_rs::Buffer;

fn build() -> Buffer {
    let mut buffer = Buffer::new();
    buffer.push_str("Hello");
    buffer.guess_segment_properties();
    buffer
}
```

`guess_segment_properties()` infers the direction, script, and language from
the code points already in the buffer. Shaping without them produces nonsense
for anything that is not left-to-right Latin — see
[text-and-buffers.md](text-and-buffers.md#segment-properties).

## Step 5 — read the output

A `GlyphBuffer` holds two parallel arrays of the same length: what to draw, and
where.

```rust
use harfbuzz_rs::GlyphBuffer;

fn dump(output: &GlyphBuffer) {
    for (info, position) in output.iter() {
        println!(
            "glyph {:>5}  cluster {:>3}  advance ({:>5}, {:>5})  offset ({:>4}, {:>4})",
            info.glyph(),
            info.cluster(),
            position.x_advance(),
            position.y_advance(),
            position.x_offset(),
            position.y_offset(),
        );
    }
}
```

| Accessor | Type | Meaning |
| --- | --- | --- |
| `info.glyph()` | `u32` | Glyph index in *this* font. `0` is `.notdef`, the "missing glyph" box. |
| `info.cluster()` | `u32` | Byte offset in the input text this glyph came from. |
| `info.is_unsafe_to_break()` | `bool` | `true` if breaking the line before this glyph would change how it or its neighbour shapes. |
| `position.x_advance()` | `i32` | Pen movement after this glyph, horizontally. |
| `position.y_advance()` | `i32` | Pen movement vertically — zero for horizontal text. |
| `position.x_offset()` | `i32` | Draw-time displacement; does **not** move the pen. |
| `position.y_offset()` | `i32` | As above, vertically. |

`output.infos()` and `output.positions()` give the two slices directly if you
prefer them to `iter()`; they are always the same length and in the same order.

## Step 6 — pen positions

Turning advances and offsets into coordinates is the same four lines in every
text renderer: keep a pen, draw at pen-plus-offset, then move the pen by the
advance.

```rust
use harfbuzz_rs::GlyphBuffer;

/// Where each glyph should be drawn, in 26.6 units relative to the run's origin.
fn place(output: &GlyphBuffer) -> Vec<(u32, i32, i32)> {
    let mut placed = Vec::with_capacity(output.len());
    let (mut pen_x, mut pen_y) = (0, 0);

    for (info, position) in output.iter() {
        placed.push((
            info.glyph(),
            pen_x + position.x_offset(),
            pen_y + position.y_offset(),
        ));

        pen_x += position.x_advance();
        pen_y += position.y_advance();
    }

    placed
}
```

Two things to keep straight:

- **The pen is not the glyph.** The offset shifts the drawing without shifting
  the pen. That is how a combining accent lands over the letter before it: its
  advance is zero, its offset is not.
- **The origin is the baseline.** Y grows *upward* in HarfBuzz's coordinate
  system. If your renderer's Y grows downward, negate — and remember that a
  glyph's `y_bearing` is then the distance *above* the baseline. See
  [fonts-and-sizing.md](fonts-and-sizing.md#metrics).

To place a whole line on a page, ask the font for its vertical metrics:

```rust
use harfbuzz_rs::Font;

/// Baseline of the first line, and the step down to each following one,
/// in the font's scaled units.
fn line_layout(font: &Font) -> (i32, i32) {
    let extents = font.extents();
    (extents.ascender, extents.line_height())
}
```

## Measuring without drawing

The width of a run is the sum of its horizontal advances.

```rust
use harfbuzz_rs::{Font, buffer_from, shape};

fn width(font: &Font, text: &str) -> harfbuzz_rs::Result<i32> {
    let output = shape(font, buffer_from(text)?, &[]);
    Ok(output.positions().iter().map(|p| p.x_advance()).sum())
}
```

This is the correct way to measure text, and summing per-character advances from
`Font::glyph_h_advance` is not: it misses kerning, ligatures, and every other
contextual adjustment shaping performs.

## Shaping many runs

Shaping consumes the buffer and gives it back inside the `GlyphBuffer`.
`clear()` empties it and returns a plain `Buffer` ready for the next run, with
its allocation intact — that is the cheap way to shape a paragraph.

```rust
use harfbuzz_rs::{Buffer, Font, shape};

fn widths(font: &Font, words: &[&str]) -> Vec<i32> {
    let mut buffer = Buffer::new();
    let mut widths = Vec::new();

    for word in words {
        buffer.push_str(word);
        buffer.guess_segment_properties();

        let output = shape(font, buffer, &[]);
        widths.push(output.positions().iter().map(|p| p.x_advance()).sum());

        // Hands the buffer back, emptied but still allocated.
        buffer = output.clear();
    }

    widths
}
```

Note that `guess_segment_properties()` is called again on each pass:
`clear()` wipes the segment properties along with the text. See
[text-and-buffers.md](text-and-buffers.md#reuse).

## The complete program

Everything above, in one piece.

```rust
use harfbuzz_rs::{Face, Font, IntoShared, buffer_from, points_to_scale, shape};

fn main() -> harfbuzz_rs::Result<()> {
    // Expensive, cache it: parse the font file once.
    let face = Face::from_file("font.ttf", 0)?.into_shared();

    // Cheap: one font per size you render at.
    let mut font = Font::new(face.clone());
    font.set_scale(points_to_scale(16), points_to_scale(16));

    let text = "Hello, world!";
    let output = shape(&font, buffer_from(text)?, &[]);

    let metrics = font.extents();
    println!("line height: {:.2}px", metrics.line_height() as f32 / 64.0);

    let mut pen_x = 0;
    for (info, position) in output.iter() {
        let x = (pen_x + position.x_offset()) as f32 / 64.0;
        let y = position.y_offset() as f32 / 64.0;

        println!(
            "glyph {:>5} from byte {:>2} at ({x:>7.2}, {y:>5.2})",
            info.glyph(),
            info.cluster(),
        );

        pen_x += position.x_advance();
    }

    println!("total width: {:.2}px", pen_x as f32 / 64.0);
    Ok(())
}
```

## Where to go next

- Your text is not plain Latin, or you need a cursor:
  [text-and-buffers.md](text-and-buffers.md)
- You need a different size, a bold weight from a variable font, or glyph
  bounding boxes: [fonts-and-sizing.md](fonts-and-sizing.md)
- You want to shape on several threads:
  [ownership-and-threads.md](ownership-and-threads.md)
- You want to turn ligatures off, or select a stylistic set:
  [values.md](values.md#feature)
- Something returned an error, or worse, silently did nothing:
  [errors.md](errors.md)
- You need part of HarfBuzz this crate does not wrap: the raw C API is
  documented under [`../harfbuzz-sys/docs/`](../harfbuzz-sys/docs/), starting
  with [`shape.md`](../harfbuzz-sys/docs/shape.md).
