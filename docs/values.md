# Value types

Six small `Copy` types describe *what* to shape and *how*: `Tag`, `Direction`,
`Script`, `Language`, `Feature`, and `Variation`. None of them owns a HarfBuzz
object, all of them are cheap to pass around, and all of them are `Send + Sync`.

- [`Tag`](#tag)
- [`Direction`](#direction)
- [`Script`](#script)
- [`Language`](#language)
- [`Feature`](#feature)
- [`Variation`](#variation)
- [Parsing at a glance](#parsing-at-a-glance)

---

## `Tag`

A four-character OpenType tag. Tags name almost everything in a font: tables
(`glyf`, `GSUB`), features (`kern`, `liga`), scripts (`latn`), languages
(`TRK `), and variation axes (`wght`).

```rust
use harfbuzz_rs::Tag;

fn main() -> harfbuzz_rs::Result<()> {
    const WEIGHT_AXIS: Tag = Tag::new(b"wght");     // `new` is const

    let kern = Tag::new(b"kern");
    assert_eq!(kern.to_string(), "kern");

    // Shorter names are padded on the right with spaces, as OpenType expects.
    let turkish: Tag = "TRK".parse()?;
    assert_eq!(turkish, Tag::new(b"TRK "));
    assert_eq!(turkish.to_string(), "TRK ");

    assert_eq!(WEIGHT_AXIS.to_raw(), 0x77676874);   // 'w' is the high byte
    Ok(())
}
```

| Item | Signature | Notes |
| --- | --- | --- |
| `Tag::NONE` | `Tag` | All four bytes zero. Displays as `????`. |
| `Tag::new` | `const (&[u8; 4]) -> Tag` | Exactly four bytes — the array length is checked at compile time. |
| `Tag::from_raw` | `const (u32) -> Tag` | No validation. |
| `to_raw` | `const (self) -> u32` | First character is the most significant byte. |
| `to_bytes` | `const (self) -> [u8; 4]` | Big-endian order, i.e. reading order. |
| `FromStr` | `(&str) -> Result<Tag>` | One to four characters, padded with spaces. |
| `Display` | | Four characters. Non-printable bytes render as `?`. |
| conversions | `From<u32>`, `From<&[u8; 4]>`, `From<Tag> for u32` | |

Derives `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`, `PartialOrd`, `Ord`, `Hash`
— so tags work as `HashMap` keys and sort predictably.

### Gotchas

**Padding is on the right, with spaces.** `"TRK"` is `"TRK "`, not `"TRK\0"` and
not `" TRK"`. Compare `Tag`s, never their string forms, if you built them
different ways.

**Parsing is stricter than HarfBuzz's.** `hb_tag_from_string` silently truncates
anything longer than four characters; this crate returns
[`Error::InvalidTag`](errors.md) instead, because truncation almost always means
the caller made a mistake. Empty strings and non-printable-ASCII bytes are
rejected too.

```rust
use harfbuzz_rs::{Error, Tag};

fn main() {
    assert_eq!("toolong".parse::<Tag>(), Err(Error::InvalidTag));
    assert_eq!("".parse::<Tag>(), Err(Error::InvalidTag));
    assert_eq!("wghé".parse::<Tag>(), Err(Error::InvalidTag));
}
```

**`Tag::new` and `Tag::from_raw` do not validate.** They accept any bytes, and
`Display` falls back to `?` per byte rather than emitting invalid UTF-8.

## `Direction`

```rust
use harfbuzz_rs::Direction;

fn main() {
    assert!(Direction::LeftToRight.is_horizontal());
    assert!(Direction::LeftToRight.is_forward());
    assert!(Direction::BottomToTop.is_vertical());
    assert!(Direction::BottomToTop.is_backward());

    assert_eq!(Direction::LeftToRight.reverse(), Direction::RightToLeft);
    assert_eq!(Direction::default(), Direction::Invalid);
}
```

| Variant | Axis | Polarity | `Display` |
| --- | --- | --- | --- |
| `Invalid` | — | — | `invalid` |
| `LeftToRight` | horizontal | forward | `ltr` |
| `RightToLeft` | horizontal | backward | `rtl` |
| `TopToBottom` | vertical | forward | `ttb` |
| `BottomToTop` | vertical | backward | `btt` |

| Method | Returns |
| --- | --- |
| `is_valid()` | `false` only for `Invalid` |
| `is_horizontal()` / `is_vertical()` | which axis |
| `is_forward()` / `is_backward()` | which way along that axis |
| `reverse()` | The opposite along the same axis; `Invalid` reverses to itself |

`Direction::default()` is `Invalid`, which is also what a fresh `Buffer` has and
what `guess_segment_properties` replaces.

### Gotcha: parsing looks at one character

Matching is case-insensitive and, exactly as in HarfBuzz, **only the first
character is significant**:

```rust
use harfbuzz_rs::Direction;

fn main() -> harfbuzz_rs::Result<()> {
    assert_eq!("rtl".parse::<Direction>()?, Direction::RightToLeft);
    assert_eq!("RTL".parse::<Direction>()?, Direction::RightToLeft);
    assert_eq!("b".parse::<Direction>()?, Direction::BottomToTop);

    // Surprising, but faithful to the C API:
    assert_eq!("leftwards".parse::<Direction>()?, Direction::LeftToRight);
    assert!("sideways".parse::<Direction>().is_err());
    Ok(())
}
```

## `Script`

A Unicode script, identified by its ISO 15924 code. The script of a run decides
which shaper HarfBuzz uses — Arabic joining, Indic reordering, Hangul
composition are all selected this way — so it matters more than any other buffer
property.

```rust
use harfbuzz_rs::{Direction, Script, Tag};

fn main() -> harfbuzz_rs::Result<()> {
    let arabic: Script = "Arab".parse()?;
    assert_eq!(arabic, Script::ARABIC);
    assert_eq!(arabic.horizontal_direction(), Direction::RightToLeft);

    // A script *is* its tag.
    assert_eq!(Script::LATIN.to_iso15924_tag(), Tag::new(b"Latn"));
    assert_eq!(Script::LATIN.to_string(), "Latn");
    Ok(())
}
```

| Item | Signature | Notes |
| --- | --- | --- |
| `from_iso15924_tag` | `(Tag) -> Script` | Accepts any four-character tag, assigned or not. |
| `to_iso15924_tag` | `(self) -> Tag` | |
| `horizontal_direction` | `(self) -> Direction` | The direction the script is normally set in. |
| `FromStr` | `(&str) -> Result<Script>` | See the gotcha below. |
| `Display` | | The ISO 15924 tag. |
| conversions | `From<Tag> for Script`, `From<Script> for Tag` | |

### Constants

Four special values, and 28 well-known scripts:

| Constant | Tag | Meaning |
| --- | --- | --- |
| `Script::INVALID` | `????` | Unset. What a fresh buffer has. |
| `Script::COMMON` | `Zyyy` | Used across scripts: spaces, most punctuation, digits. |
| `Script::INHERITED` | `Zinh` | Takes the script of the preceding character: combining marks. |
| `Script::UNKNOWN` | `Zzzz` | Unassigned code points. |

`ARABIC`, `ARMENIAN`, `BENGALI`, `CYRILLIC`, `DEVANAGARI`, `GEORGIAN`, `GREEK`,
`GUJARATI`, `GURMUKHI`, `HAN`, `HANGUL`, `HEBREW`, `HIRAGANA`, `KANNADA`,
`KATAKANA`, `KHMER`, `LAO`, `LATIN`, `MALAYALAM`, `MYANMAR`, `ORIYA`, `SINHALA`,
`SYRIAC`, `TAMIL`, `TELUGU`, `THAANA`, `THAI`, `TIBETAN`.

All 177 of HarfBuzz's scripts exist as `HB_SCRIPT_*` constants in
[`harfbuzz_rs::sys`](../harfbuzz-sys/docs/script.md); anything not named above
can be built from its tag:

```rust
use harfbuzz_rs::{Script, Tag};

fn main() {
    let coptic = Script::from_iso15924_tag(Tag::new(b"Copt"));
    assert_eq!(coptic.to_string(), "Copt");
}
```

### Gotchas

**Parsing takes tags, not names.** `FromStr` forwards to
`hb_script_from_string`, which is tag-based and truncates to four characters —
so `"Latin"` parses to the tag `Lati`, which is *not* `Latn`:

```rust
use harfbuzz_rs::Script;

fn main() -> harfbuzz_rs::Result<()> {
    assert_eq!("Latn".parse::<Script>()?, Script::LATIN);

    let wrong: Script = "Latin".parse()?;
    assert_ne!(wrong, Script::LATIN);       // it became the tag `Lati`
    Ok(())
}
```

**Parsing almost never fails.** Any non-empty string maps to *some* script —
unrecognised tags become `Script::UNKNOWN`. Only the empty string returns
`Error::InvalidScript`. Do not use `parse()` as a validity check; compare
against the constant you expect.

**`horizontal_direction()` returns `LeftToRight` for scripts HarfBuzz does not
know**, and `Direction::Invalid` for scripts that are not written horizontally
at all.

## `Language`

A BCP 47 language tag such as `en-GB`. HarfBuzz **interns** languages: every
distinct tag is canonicalised and stored once for the lifetime of the process,
so a `Language` is a plain copyable handle that never needs freeing, and two
languages are equal exactly when their interned pointers match.

```rust
use harfbuzz_rs::Language;

fn main() -> harfbuzz_rs::Result<()> {
    let english: Language = "en-GB".parse()?;

    // Canonicalisation lowercases, so the round trip is not byte-identical.
    assert_eq!(english.to_string(), "en-gb");
    assert_eq!("EN-gb".parse::<Language>()?, english);

    // `matches` asks "is `self` a more general tagging than the argument?"
    let general: Language = "en".parse()?;
    assert!(general.matches(english));
    assert!(!english.matches(general));
    Ok(())
}
```

| Item | Signature | Notes |
| --- | --- | --- |
| `default_from_locale` | `() -> Language` | Derived from the process locale. **The first call is not thread-safe** — make it during start-up. |
| `is_invalid` | `(self) -> bool` | True for the unset language, which is what a fresh buffer has. |
| `matches` | `(self, Language) -> bool` | `en` matches `en-GB`; `en-GB` does not match `en`. |
| `FromStr` | `(&str) -> Result<Language>` | Rejects only the empty string. |
| `Display` | | The canonical, lowercased spelling; `invalid` for the unset language. |

`Language` is `Copy`, `Send`, and `Sync`. It has no `Default`; use
`default_from_locale()` or leave the buffer's language unset and let
`guess_segment_properties` fill it in.

Language affects shaping less than script does — it selects language-specific
feature variants, such as Turkish dotless-i handling or Serbian Cyrillic
italics. Setting it wrong degrades quality; leaving it unset falls back to the
default variants.

## `Feature`

A request to turn an OpenType feature on or off over a range of the buffer.
Features are the switches for optional typographic behaviour: `liga` for
standard ligatures, `kern` for kerning, `smcp` for small capitals, `ss01` for
the first stylistic set, `salt` for stylistic alternates.

```rust
use harfbuzz_rs::{Feature, Tag};

fn main() -> harfbuzz_rs::Result<()> {
    // Turn ligatures off everywhere.
    let no_ligatures = Feature::new(Tag::new(b"liga"), 0, ..);

    // The same thing in the syntax the `hb-shape` tool uses.
    let also: Feature = "-liga".parse()?;
    assert_eq!(no_ligatures, also);

    // Select the third alternate of `salt`, over clusters 2..5 only.
    let alternate = Feature::new(Tag::new(b"salt"), 3, 2..5);
    assert_eq!(alternate.to_string(), "salt[2:5]=3");
    Ok(())
}
```

Pass them to [`shape`](getting-started.md#step-4--shape):

```rust
use harfbuzz_rs::{Feature, Font, Tag, buffer_from, shape};

fn shape_without_ligatures(font: &Font, text: &str) -> harfbuzz_rs::Result<usize> {
    let features = [Feature::new(Tag::new(b"liga"), 0, ..)];
    Ok(shape(font, buffer_from(text)?, &features).len())
}
```

| Item | Signature | Notes |
| --- | --- | --- |
| `Feature::new` | `(Tag, u32, impl RangeBounds<u32>) -> Feature` | `..` for the whole buffer. |
| `tag` | `(self) -> Tag` | |
| `value` | `(self) -> u32` | `0` disables, non-zero enables; for alternate features, a one-based index. |
| `start` / `end` | `(self) -> u32` | Half-open range in cluster values. |
| `is_global` | `(self) -> bool` | True only for the full `0..=u32::MAX` range. |
| `FromStr` | `(&str) -> Result<Feature>` | `kern`, `+kern`, `-liga`, `aalt=2`, `salt[3:5]=2`. |
| `Display` | | The same syntax, canonicalised. |

`Feature` is `Copy`, `PartialEq`, `Eq`, `Hash`.

### Gotchas

> **Ranges are cluster values, not byte offsets and not glyph indices.** A
> cluster value happens to *be* a byte offset for text pushed with a single
> `push_str` — which makes the distinction easy to miss until a ligature or a
> multi-byte character shifts things. Take the range from
> `GlyphInfo::cluster()` values, never from `str` indices you computed
> independently.

Rust range syntax converts as you would expect, with the end bound exclusive
internally:

```rust
use harfbuzz_rs::{Feature, Tag};

fn main() {
    let half_open = Feature::new(Tag::new(b"salt"), 2, 2..5);
    assert_eq!((half_open.start(), half_open.end()), (2, 5));

    let inclusive = Feature::new(Tag::new(b"salt"), 2, 2..=5);
    assert_eq!((inclusive.start(), inclusive.end()), (2, 6));

    let global = Feature::new(Tag::new(b"kern"), 1, ..);
    assert!(global.is_global());
    assert_eq!(global.end(), u32::MAX);

    // `..5` is *not* global: it starts at 0 but stops early.
    assert!(!Feature::new(Tag::new(b"salt"), 2, ..5).is_global());
}
```

**Most features are already on.** `liga`, `kern`, `calt`, and the script-specific
features a shaper needs are enabled by default. Passing `&[]` is right unless you
are deliberately overriding one; passing `+kern` changes nothing.

**A feature the font does not have is silently ignored.** There is no error and
no way to tell from the output.

## `Variation`

One axis of a variable font pinned to one value. See
[fonts-and-sizing.md](fonts-and-sizing.md#variable-fonts) for how to apply them.

```rust
use harfbuzz_rs::{Tag, Variation};

fn main() -> harfbuzz_rs::Result<()> {
    let bold = Variation::new(Tag::new(b"wght"), 700.0);

    assert_eq!(bold, "wght=700".parse()?);
    assert_eq!(bold.to_string(), "wght=700");
    assert_eq!(bold.tag(), Tag::new(b"wght"));
    assert_eq!(bold.value(), 700.0);
    Ok(())
}
```

| Item | Signature | Notes |
| --- | --- | --- |
| `Variation::new` | `(Tag, f32) -> Variation` | No range checking — the font clamps. |
| `tag` / `value` | `(self) -> Tag` / `f32` | |
| `FromStr` | `(&str) -> Result<Variation>` | `axis=value`, e.g. `wght=700`, `wdth=87.5`. |
| `Display` | | `axis=value`. |

`Variation` is `Copy` and `PartialEq` — but **not** `Eq` or `Hash`, because it
holds an `f32`. To key a cache by axis settings, quantise the values to integers
first.

## Parsing at a glance

Every value type implements `FromStr`, so `"…".parse::<T>()` works throughout.
They differ sharply in how strict they are:

| Type | Accepts | Rejects | Silent surprise |
| --- | --- | --- | --- |
| `Tag` | 1–4 printable ASCII chars | empty, >4 chars, non-ASCII | none — this one is strict |
| `Direction` | `ltr` `rtl` `ttb` `btt`, any case | empty, or a first letter that is not `l`/`r`/`t`/`b` | only the first character is read |
| `Script` | any non-empty string | empty only | names are truncated to 4 chars; unknown tags become `Script::UNKNOWN` |
| `Language` | any non-empty string | empty only | canonicalised and lowercased |
| `Feature` | `hb-shape` syntax | malformed strings | none |
| `Variation` | `axis=value` | malformed strings | value is not range-checked |

All of them return [`Error`](errors.md) variants named after the type:
`InvalidTag`, `InvalidDirection`, `InvalidScript`, `InvalidLanguage`,
`InvalidFeature`, `InvalidVariation`.

---

The underlying C definitions are documented in
[`../harfbuzz-sys/docs/script.md`](../harfbuzz-sys/docs/script.md) (tags,
directions, scripts, and languages all live in `hb-common.h`) and
[`../harfbuzz-sys/docs/shape.md`](../harfbuzz-sys/docs/shape.md) (features).
Upstream's guide to what features actually do is
[`../harfbuzz-sys/docs/guide/09-opentype-features.md`](../harfbuzz-sys/docs/guide/09-opentype-features.md).
