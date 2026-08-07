# Text and buffers

A buffer is the object you shape *with*. It plays two roles: before shaping it
holds the input characters and the properties that decide how they are shaped;
after shaping the very same object holds the output glyphs. This page covers
getting text in, getting glyphs out, and the one concept — clusters — that
connects them.

- [The lifecycle](#the-lifecycle)
- [Adding text](#adding-text)
- [Segment properties](#segment-properties)
- [Clusters](#clusters)
- [Cursor placement and hit-testing](#cursor-placement-and-hit-testing)
- [Shaping a fragment: context](#shaping-a-fragment-context)
- [Reading the output](#reading-the-output)
- [Right-to-left output](#right-to-left-output)
- [Line breaking and `is_unsafe_to_break`](#line-breaking-and-is_unsafe_to_break)
- [Reuse](#reuse)
- [Method reference](#method-reference)

---

## The lifecycle

```text
Buffer::new()
    │
    ├── push_str(...)                  code points in
    ├── set_direction / set_script / set_language     ┐  or
    ├── guess_segment_properties()                    ┘
    │
    ▼
shape(&font, buffer, &features)        consumes the Buffer
    │
    ▼
GlyphBuffer                            infos(), positions(), iter()
    │
    └── clear()  ──►  Buffer           emptied, allocation kept, properties gone
```

The shortest correct version of the first half is `buffer_from`:

```rust
use harfbuzz_rs::{Buffer, buffer_from};

fn main() -> harfbuzz_rs::Result<()> {
    let buffer: Buffer = buffer_from("Hello")?;
    assert_eq!(buffer.len(), 5);
    Ok(())
}
```

`buffer_from` creates the buffer, pushes the text, checks
`allocation_successful()` — returning `Error::AllocationFailed` if it is not —
and calls `guess_segment_properties()`. Use it whenever you do not need to set
properties by hand.

## Adding text

| Method | Signature | Notes |
| --- | --- | --- |
| `Buffer::new` | `() -> Buffer` | Empty, with no segment properties. `Default` does the same. |
| `push_str` | `(&mut self, &str)` | Appends UTF-8. Cluster values are byte offsets **within this call's string** — see the warning below. |
| `push_str_with_context` | `(&mut self, before: &str, text: &str, after: &str)` | Shapes `text` while letting the shaper see its neighbours. Only `text` ends up in the buffer. |
| `buffer_from` | `(&str) -> Result<Buffer>` | Free function: `new` + `push_str` + `guess_segment_properties`. |
| `len` / `is_empty` | `(&self) -> usize` / `bool` | Code points before shaping, glyphs after. |

`len()` counts **code points**, not bytes and not characters-as-a-human-sees-them:

```rust
use harfbuzz_rs::Buffer;

fn main() {
    let mut buffer = Buffer::new();
    buffer.push_str("café");

    assert_eq!(buffer.len(), 4);          // four code points
    assert_eq!("café".len(), 5);          // five UTF-8 bytes
}
```

> **Two `push_str` calls do not produce continuous clusters.** Each call
> restarts cluster numbering at zero, so `push_str("AB")` followed by
> `push_str("CD")` yields clusters `[0, 1, 0, 1]` rather than `[0, 1, 2, 3]`.
> Push the whole run in one call — build the `String` first if you have to —
> or use `push_str_with_context`, which is designed for the split case and
> numbers clusters relative to the combined text.

## Segment properties

Three properties tell the shaper how to treat the text. Together they select
which of HarfBuzz's script-specific shapers runs, and in which order glyphs come
out.

| Property | Type | Default | What it decides |
| --- | --- | --- | --- |
| Direction | [`Direction`](values.md#direction) | `Invalid` | Which way the pen moves, and the order of the output |
| Script | [`Script`](values.md#script) | `Script::INVALID` | Which shaper runs: Arabic joining, Indic reordering, Hangul composition… |
| Language | [`Language`](values.md#language) | invalid | Language-specific feature variants, e.g. Turkish `i` handling |

**Shaping with them unset produces wrong output, not an error.** Either set them
or guess them:

```rust
use harfbuzz_rs::{Buffer, Direction, Script};

fn main() -> harfbuzz_rs::Result<()> {
    // If you know them — from your BiDi/itemisation pass — set them.
    let mut explicit = Buffer::new();
    explicit.push_str("مرحبا");
    explicit.set_direction(Direction::RightToLeft);
    explicit.set_script(Script::ARABIC);
    explicit.set_language("ar".parse()?);

    // If you do not, let HarfBuzz infer them from the code points.
    let mut guessed = Buffer::new();
    guessed.push_str("مرحبا");
    guessed.guess_segment_properties();

    assert_eq!(guessed.direction(), Direction::RightToLeft);
    assert_eq!(guessed.script(), Script::ARABIC);
    Ok(())
}
```

`guess_segment_properties()` only fills in what is still unset, so anything you
set explicitly survives:

```rust
use harfbuzz_rs::{Buffer, Direction};

fn main() {
    let mut buffer = Buffer::new();
    buffer.set_direction(Direction::TopToBottom);
    buffer.push_str("hello");
    buffer.guess_segment_properties();

    // Latin would have been guessed left-to-right; ours survived.
    assert_eq!(buffer.direction(), Direction::TopToBottom);
}
```

What it infers, and how:

- **Script** from the code points, ignoring `Common` and `Inherited` ones
  (spaces, punctuation, combining marks).
- **Direction** from the script — `Script::horizontal_direction()`.
- **Language** from the process locale, the same value as
  `Language::default_from_locale()`.

Guessing is a convenience, not a substitute for proper itemisation. Real
multi-script, bidirectional text should be segmented into runs by a Unicode BiDi
implementation first, and each run shaped with its own explicit properties.

## Clusters

A cluster value is the link from a shaped glyph back to the text it came from.
It is **the byte offset** at which that piece of input started.

```rust
use harfbuzz_rs::{Font, buffer_from, shape};

fn clusters(font: &Font, text: &str) -> harfbuzz_rs::Result<Vec<u32>> {
    let output = shape(font, buffer_from(text)?, &[]);
    Ok(output.infos().iter().map(|i| i.cluster()).collect())
}
```

For plain ASCII shaped one-to-one, clusters are just `[0, 1, 2, …]`. Everything
interesting is a departure from that:

| Input | Typical output | Clusters | Why |
| --- | --- | --- | --- |
| `"ABC"` | 3 glyphs | `[0, 1, 2]` | One glyph per byte |
| `"AéB"` (4 UTF-8 bytes) | 3 glyphs | `[0, 1, 3]` | `é` is two UTF-8 bytes, so the next cluster starts at 3 |
| `"fi"` with `liga` on | **1** glyph | `[0]` | A ligature: one glyph covering two characters |
| `"e"` + combining acute | 2 glyphs | `[0, 0]` | Both glyphs belong to one cluster |
| Indic syllable | reordered glyphs | all equal | Glyphs may appear in a different order than the input |

Two rules follow, and they are the ones that matter for building a text editor:

1. **A cluster value is a byte offset into the input string**, so it can index
   directly into your `&str` — but only at a character boundary, which cluster
   values always are.
2. **Cluster values are monotonic** along the buffer: non-decreasing for
   left-to-right text, non-increasing for right-to-left. They are *not*
   necessarily contiguous, and several glyphs may share one value.

Never assume glyph *n* corresponds to character *n*.

## Cursor placement and hit-testing

Clusters make both directions of the mapping possible: from a byte offset in the
text to an x coordinate, and back.

```rust
use harfbuzz_rs::{Font, buffer_from, shape};

/// The x coordinate of the leading edge of every cluster in `text`, plus a final
/// entry for the end of the run. Indexed by byte offset; left-to-right text.
fn caret_positions(font: &Font, text: &str) -> harfbuzz_rs::Result<Vec<i32>> {
    let output = shape(font, buffer_from(text)?, &[]);

    let mut carets = vec![0; text.len() + 1];
    let mut pen = 0;
    let mut current = None;

    for (info, position) in output.iter() {
        // Several glyphs can share a cluster; only the first of them starts it.
        if current != Some(info.cluster()) {
            carets[info.cluster() as usize] = pen;
            current = Some(info.cluster());
        }
        pen += position.x_advance();
    }

    carets[text.len()] = pen;
    Ok(carets)
}

/// The byte offset a click at `x` should place the caret at.
fn hit_test(font: &Font, text: &str, x: i32) -> harfbuzz_rs::Result<usize> {
    let output = shape(font, buffer_from(text)?, &[]);

    let mut pen = 0;
    for (info, position) in output.iter() {
        let advance = position.x_advance();

        // Past the midpoint of a glyph, the caret belongs after it.
        if x < pen + advance / 2 {
            return Ok(info.cluster() as usize);
        }
        pen += advance;
    }

    Ok(text.len())
}
```

For a cluster that produced several glyphs — a ligature — this places the caret
at the start of the whole cluster. Splitting a ligature to put a caret inside it
requires dividing the cluster's total advance by the number of characters it
covers; HarfBuzz cannot do it for you, because a ligature glyph has no internal
structure.

Note that `caret_positions` only fills the entries for byte offsets that
actually begin a cluster; offsets inside a ligature or inside a multi-byte
character are left at zero. Those are not valid caret positions in the first
place, so an editor should snap to the nearest cluster boundary before querying.

## Shaping a fragment: context

Shaping half a word in isolation gets it wrong. An Arabic letter takes one of
four forms depending on its neighbours, and a shaper that cannot see the
neighbours picks the isolated form. The same applies to any contextual
substitution, including Latin ligatures across a highlight boundary.

`push_str_with_context` gives the shaper the surrounding text without adding it
to the output:

```rust
use harfbuzz_rs::Buffer;

fn main() {
    let mut buffer = Buffer::new();

    // Shape only "def", but let the shaper see "abc" before and "ghi" after.
    buffer.push_str_with_context("abc", "def", "ghi");

    assert_eq!(buffer.len(), 3, "only the middle run is in the buffer");
}
```

Use it whenever you shape a slice of a longer paragraph — one style run, one
highlighted selection, one line of a wrapped paragraph.

> **Cluster values shift by the length of the context.** With
> `push_str_with_context("abc", "def", "ghi")`, the three shaped glyphs carry
> clusters `[3, 4, 5]`, not `[0, 1, 2]`: they are byte offsets into the
> *combined* string. Subtract `before.len()` to get offsets into the middle run,
> or keep the offsets absolute and index into the full paragraph — which is
> usually what you want anyway.

```rust
use harfbuzz_rs::{Buffer, Font, shape};

/// Shape `paragraph[range]`, returning clusters relative to the fragment.
fn shape_fragment(font: &Font, paragraph: &str, start: usize, end: usize) -> Vec<u32> {
    let mut buffer = Buffer::new();
    buffer.push_str_with_context(&paragraph[..start], &paragraph[start..end], &paragraph[end..]);
    buffer.guess_segment_properties();

    let output = shape(font, buffer, &[]);
    output
        .infos()
        .iter()
        .map(|info| info.cluster() - start as u32)
        .collect()
}
```

## Reading the output

A `GlyphBuffer` exposes two parallel slices, always the same length and in the
same order.

```rust
use harfbuzz_rs::{GlyphInfo, GlyphPosition, GlyphBuffer};

fn split(output: &GlyphBuffer) -> (&[GlyphInfo], &[GlyphPosition]) {
    (output.infos(), output.positions())
}

fn zipped(output: &GlyphBuffer) {
    for (info, position) in output.iter() {
        let _ = (info.glyph(), position.x_advance());
    }
}
```

| `GlyphInfo` | Meaning |
| --- | --- |
| `glyph() -> u32` | Glyph index in this font. `0` is `.notdef` — the font had no glyph for that character. |
| `cluster() -> u32` | Byte offset of the input this glyph came from. |
| `is_unsafe_to_break() -> bool` | Breaking the line before this glyph would change the shaping. |

| `GlyphPosition` | Meaning |
| --- | --- |
| `x_advance() -> i32` | Pen movement after this glyph. |
| `y_advance() -> i32` | Vertical pen movement; zero in horizontal text. |
| `x_offset() -> i32` | Draw-time displacement. Does not move the pen. |
| `y_offset() -> i32` | As above, vertically. Positive is up. |

All four are in the font's scaled units — see
[fonts-and-sizing.md](fonts-and-sizing.md#scale-and-the-266-convention).

A missing character is not an error. It shapes to glyph `0`, which most fonts
draw as an empty or crossed box:

```rust
use harfbuzz_rs::{Font, buffer_from, shape};

fn missing(font: &Font, text: &str) -> harfbuzz_rs::Result<bool> {
    let output = shape(font, buffer_from(text)?, &[]);
    Ok(output.infos().iter().any(|info| info.glyph() == 0))
}
```

Checking for `.notdef` is how you decide to fall back to another font.

## Right-to-left output

HarfBuzz emits glyphs in **visual order**, left to right on the page, whatever
the text direction. For a right-to-left run that means the first glyph in the
output is the *last* character of the input, and cluster values descend:

```rust
use harfbuzz_rs::{Buffer, Direction, Font, shape};

fn rtl_clusters(font: &Font, text: &str) -> Vec<u32> {
    let mut buffer = Buffer::new();
    buffer.push_str(text);
    buffer.set_direction(Direction::RightToLeft);
    buffer.guess_segment_properties();

    let output = shape(font, buffer, &[]);
    output.infos().iter().map(|i| i.cluster()).collect()
    // For "ABC": [2, 1, 0]
}
```

The pen loop from [getting-started.md](getting-started.md#step-6--pen-positions)
needs no change — advances are still applied left to right. What changes is that
the *origin* of the run is its right edge if you are laying out right-aligned
text, and that cluster-to-x mapping runs backwards.

`GlyphBuffer::direction()` reports the direction the run was shaped in, which is
what you need to decide that.

## Line breaking and `is_unsafe_to_break`

Line breaking happens *after* shaping, and it can invalidate the shaping it acts
on: splitting between two characters that formed a ligature, or between an
Arabic letter and the one it joins to, changes how both should look. HarfBuzz
flags exactly those positions.

```rust
use harfbuzz_rs::{Font, buffer_from, shape};

/// Byte offsets where the run may be split without reshaping either side.
fn safe_break_offsets(font: &Font, text: &str) -> harfbuzz_rs::Result<Vec<u32>> {
    let output = shape(font, buffer_from(text)?, &[]);

    Ok(output
        .infos()
        .iter()
        .filter(|info| !info.is_unsafe_to_break())
        .map(|info| info.cluster())
        .collect())
}
```

`is_unsafe_to_break()` is true when breaking the line **before** this glyph
would change how it or its predecessor shapes. The flag is advisory in one
direction only: a `false` means the break is safe and you can keep the shaped
glyphs on both sides; a `true` means you must reshape each side separately after
splitting there.

The cheap policy is to break only at safe positions. The correct policy is to
break wherever your line-breaking algorithm says — usually a UAX #14 pass over
the original text — and reshape the two halves when the flag says you must. In
either case the flag is a shaping fact, not a typographic one: it does not know
about words, spaces, or hyphenation.

## Reuse

Buffers are the one HarfBuzz object worth reusing rather than caching. Clearing
one keeps its allocated arrays, so a paragraph loop performs a handful of
allocations in total rather than one pair per run.

```rust
use harfbuzz_rs::{Buffer, Font, shape};

fn shape_all(font: &Font, runs: &[&str]) -> Vec<usize> {
    let mut buffer = Buffer::new();
    let mut counts = Vec::new();

    for run in runs {
        buffer.push_str(run);
        buffer.guess_segment_properties();     // required again — see below

        let output = shape(font, buffer, &[]);
        counts.push(output.len());

        buffer = output.clear();
    }

    counts
}
```

> **`clear()` and `clear_contents()` wipe the segment properties too.** Upstream
> defines `hb_buffer_clear_contents` as a full reset minus the Unicode functions
> and the replacement code point, so direction, script, and language all go back
> to their invalid defaults. Set or guess them again on every pass — a loop that
> sets them once outside the loop silently shapes everything after the first run
> with no properties at all.

```rust
use harfbuzz_rs::{Buffer, Direction};

fn main() {
    let mut buffer = Buffer::new();
    buffer.set_direction(Direction::RightToLeft);
    buffer.push_str("hello");

    buffer.clear_contents();

    assert!(buffer.is_empty());
    assert_eq!(buffer.direction(), Direction::Invalid);   // the property went too
}
```

| Method | What survives |
| --- | --- |
| `GlyphBuffer::clear()` | The allocation. Returns a `Buffer`. |
| `Buffer::clear_contents()` | The allocation, the Unicode functions, and the replacement code point. |
| `Buffer::reset()` | The allocation only — every property returns to its default. |

## Method reference

### `Buffer`

| Method | Signature |
| --- | --- |
| `new` / `default` | `() -> Buffer` |
| `push_str` | `(&mut self, &str)` |
| `push_str_with_context` | `(&mut self, &str, &str, &str)` |
| `len` / `is_empty` | `(&self) -> usize` / `bool` |
| `direction` / `set_direction` | `(&self) -> Direction` / `(&mut self, Direction)` |
| `script` / `set_script` | `(&self) -> Script` / `(&mut self, Script)` |
| `language` / `set_language` | `(&self) -> Language` / `(&mut self, Language)` |
| `guess_segment_properties` | `(&mut self)` |
| `reverse` | `(&mut self)` — reverses the contents in place |
| `allocation_successful` | `(&self) -> bool` — see [errors.md](errors.md#3-accumulating-failure-flags) |
| `clear_contents` | `(&mut self)` |
| `reset` | `(&mut self)` |

`Buffer` is `Send` but not `Sync`, is not `Clone`, and has no `into_shared` —
one buffer per thread. See
[ownership-and-threads.md](ownership-and-threads.md#send-and-sync).

### `GlyphBuffer`

| Method | Signature |
| --- | --- |
| `len` / `is_empty` | `(&self) -> usize` / `bool` |
| `infos` | `(&self) -> &[GlyphInfo]` |
| `positions` | `(&self) -> &[GlyphPosition]` |
| `iter` | `(&self) -> impl Iterator<Item = (&GlyphInfo, &GlyphPosition)>` |
| `direction` | `(&self) -> Direction` |
| `clear` | `(self) -> Buffer` |

---

The C buffer API has more knobs than this crate wraps — cluster levels, buffer
flags, invisible-glyph substitution, and a debugging serializer among them. They
are documented in
[`../harfbuzz-sys/docs/buffer.md`](../harfbuzz-sys/docs/buffer.md), and reachable
through `HarfBuzzObject::as_raw()`; see
[ownership-and-threads.md](ownership-and-threads.md#the-unsafe-traits). Upstream's
conceptual treatment of clusters is
[`../harfbuzz-sys/docs/guide/08-clusters.md`](../harfbuzz-sys/docs/guide/08-clusters.md).
