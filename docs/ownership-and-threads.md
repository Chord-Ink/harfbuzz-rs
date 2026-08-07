# Ownership, sharing, and threads

This is the design idea at the centre of the crate. HarfBuzz has a rule that it
enforces at runtime by *silently ignoring you*; this crate turns that rule into
a compile error. Understanding the trade makes the rest of the API obvious.

- [HarfBuzz's lifecycle](#harfbuzzs-lifecycle)
- [The problem: setters that fail silently](#the-problem-setters-that-fail-silently)
- [The fix: two types, one door between them](#the-fix-two-types-one-door-between-them)
- [`Shared<T>`](#sharedt)
- [Send and Sync](#send-and-sync)
- [`ThreadSafeWhenImmutable`](#threadsafewhenimmutable)
- [Shaping on several threads](#shaping-on-several-threads)
- [The unsafe traits](#the-unsafe-traits)
- [Gotchas](#gotchas)

---

## HarfBuzz's lifecycle

Every HarfBuzz object — blob, face, font, buffer — is reference counted.
`hb_x_create()` returns it with a count of one, `hb_x_reference()` raises the
count, `hb_x_destroy()` lowers it and frees at zero. The counting is atomic, so
handles can cross threads even when the object's *contents* cannot.

Most object types also have a second phase to their life. Upstream describes the
pattern as: **create the object, make a few `set_*` calls, then use it without
further modification.** An object can be marked immutable with
`hb_x_make_immutable()`, after which it is safe to use from several threads at
once. There is no inverse operation: immutability is permanent.

```text
   create  ──►  configure (set_*)  ──►  make_immutable  ──►  use, share, read
                     ▲                        │
                     └──── no way back ───────┘
```

## The problem: setters that fail silently

After `hb_face_make_immutable`, `hb_face_set_upem` does nothing. It does not
return an error. It does not warn. Your call is discarded and the program runs
on with the value you thought you had changed.

Worse, freezing happens implicitly: `hb_font_create(face)` makes the face
immutable as a side effect, because the new font now depends on it. So this C
code is a bug that produces no diagnostic at all:

```c
hb_font_t *font = hb_font_create(face);
hb_face_set_upem(face, 1000);    /* ignored — creating the font froze the face */
```

## The fix: two types, one door between them

The crate splits an object's two phases across two Rust types.

| | Owned (`Face`, `Font`, `Blob`) | Frozen (`Shared<Face>`, …) |
| --- | --- | --- |
| How many handles | Exactly one | Any number |
| Setters | `&mut self` — reachable | Unreachable: only `&T` is exposed |
| `Clone` | No | Yes, one atomic increment |
| `Send` | Yes | Yes |
| `Sync` | **No** | Yes |
| How to get one | A constructor | `owned.into_shared()` |

An owned wrapper holds the *only* handle to its object, so its setters can take
`&mut self` and the borrow checker guarantees nothing else observes a
half-configured object. `IntoShared::into_shared` is the one-way door: it calls
`hb_x_make_immutable` and hands back a `Shared<T>`.

```rust
use harfbuzz_rs::{Face, IntoShared};

fn main() -> harfbuzz_rs::Result<()> {
    let mut face = Face::from_file("font.ttf", 0)?;
    face.set_upem(1000);            // fine: we hold the only handle

    let face = face.into_shared();  // frozen from here on
    let clone = face.clone();       // cheap: bumps HarfBuzz's own refcount

    // clone.set_upem(2048);
    // ^ error[E0596]: cannot borrow data in dereference of `Shared<Face>` as mutable
    //   help: trait `DerefMut` is required to modify through a dereference,
    //         but it is not implemented for `Shared<Face>`

    assert_eq!(clone.upem(), 1000);
    Ok(())
}
```

The C bug above cannot be written: `Font::new` demands a `Shared<Face>`, so the
face is already frozen — visibly, in the type — before a font exists.

**To keep editing, do not freeze.** There is no `Shared::into_owned`, exactly as
there is no `hb_face_make_mutable`. If you need a differently configured face,
build a second one from the same bytes:

```rust
use harfbuzz_rs::{Blob, Face, IntoShared};

fn main() -> harfbuzz_rs::Result<()> {
    let blob = Blob::from_file("font.ttf")?.into_shared();

    let standard = Face::new(&blob, 0)?.into_shared();

    let mut rescaled = Face::new(&blob, 0)?;   // a separate face, same bytes
    rescaled.set_upem(1000);
    let rescaled = rescaled.into_shared();

    println!("{} vs {}", standard.upem(), rescaled.upem());
    Ok(())
}
```

Building the second face is not free, but it is honest: HarfBuzz would have made
you duplicate the object too.

## `Shared<T>`

`Shared<T>` is a counted reference to a frozen object.

| Item | Signature | Notes |
| --- | --- | --- |
| `Deref` | `Target = T` | Reaches every `&self` accessor, no `&mut self` setter. `&Shared<Font>` coerces to `&Font` at call sites. |
| `Clone` | `(&self) -> Shared<T>` | One atomic increment. No data is copied. |
| `as_raw` | `(&self) -> *mut T::Raw` | Borrows the pointer. The wrapper still owns it — do not destroy it. |
| `into_raw` | `(self) -> *mut T::Raw` | Gives up ownership; the caller must eventually `hb_x_destroy`. |
| `Shared::from_immutable` | `unsafe (T) -> Shared<T>` | Escape hatch for an object you froze yourself. Prefer `into_shared`. |
| `Debug` | | Prints as `Shared(Face { .. })` when `T: Debug`. |

Which types can be shared:

| Type | `IntoShared` | `Shared<T>` is `Send + Sync` |
| --- | --- | --- |
| `Blob` | yes | yes |
| `Face` | yes | yes |
| `Font` | yes | yes |
| `Buffer` | **no** | n/a |

`Buffer` is deliberately excluded. A buffer is mutated throughout shaping —
that is its entire purpose — so there is no immutable phase to share. One buffer
per thread, always.

## Send and Sync

```text
             Send    Sync
Blob          ✔       ✘
Face          ✔       ✘
Font          ✔       ✘
Buffer        ✔       ✘
GlyphBuffer   ✔       ✘
Shared<Blob>  ✔       ✔
Shared<Face>  ✔       ✔
Shared<Font>  ✔       ✔
Language      ✔       ✔      (interned, never freed)
Tag, Direction, Script, Feature, Variation, Error   ✔  ✔   (plain values)
```

**Owned wrappers are `Send`**: moving the unique handle to another thread cannot
race with anything, because there is nothing else to race with.

**Owned wrappers are not `Sync`**, and this is the subtle part. `&Font` looks
harmless, but HarfBuzz populates internal caches during `&self` operations —
glyph metrics, shape plans — and only a *frozen* object is documented to
tolerate that concurrently. Sharing a `&Font` across threads would be a data
race inside C code that Rust cannot see. So the only way to read one font from
several threads is `Shared<Font>`.

`GlyphBuffer` follows `Buffer`: `Send`, not `Sync`, not shareable.

## `ThreadSafeWhenImmutable`

```rust,ignore
pub unsafe trait ThreadSafeWhenImmutable: HarfBuzzObject {}
```

An empty marker trait, and the sole reason `Shared<T>` is `Send + Sync` for some
`T` and not others:

```rust,ignore
unsafe impl<T: HarfBuzzObject + ThreadSafeWhenImmutable> Send for Shared<T> {}
unsafe impl<T: HarfBuzzObject + ThreadSafeWhenImmutable> Sync for Shared<T> {}
```

Implementing it is a promise that **after `hb_x_make_immutable()`, concurrent
calls to this type's `&self` methods on the same object are free of data
races** — either because HarfBuzz does not mutate the object on those paths, or
because it does so through atomics. `Blob`, `Face`, and `Font` implement it,
resting on upstream's own guarantee. `Buffer` does not, and could not.

You only need to implement it yourself if you are wrapping a HarfBuzz type the
crate does not cover; see [the unsafe traits](#the-unsafe-traits).

## Shaping on several threads

The pattern: **share the font, never the buffer.**

```rust
use std::thread;

use harfbuzz_rs::{Face, Font, IntoShared, buffer_from, points_to_scale, shape};

fn main() -> harfbuzz_rs::Result<()> {
    let face = Face::from_file("font.ttf", 0)?.into_shared();

    let mut font = Font::new(face);
    font.set_scale(points_to_scale(16), points_to_scale(16));
    let font = font.into_shared();          // frozen: now Send + Sync

    let paragraphs = ["the first line", "the second line", "the third"];

    let widths: Vec<i32> = thread::scope(|scope| {
        let handles: Vec<_> = paragraphs
            .iter()
            .map(|text| {
                let font = font.clone();     // one atomic increment per thread
                scope.spawn(move || {
                    // Each thread builds its own buffer. Buffers are never shared.
                    let output = shape(&font, buffer_from(text).unwrap(), &[]);
                    output.positions().iter().map(|p| p.x_advance()).sum::<i32>()
                })
            })
            .collect();

        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });

    println!("{widths:?}");
    Ok(())
}
```

`thread::scope` is not required — `Shared<Font>` is `'static` as long as the
data behind it is, so `thread::spawn` and long-lived worker pools work the same
way. What matters is the shape of the ownership:

| Object | Where it lives |
| --- | --- |
| `Shared<Face>` | One, in the cache, cloned as needed |
| `Shared<Font>` | One per size/axis combination, cloned into each thread |
| `Buffer` | One **per thread**, reused across that thread's runs |
| `GlyphBuffer` | Never leaves the thread that shaped it |

A worker that shapes many runs should hold its buffer across them:

```rust
use harfbuzz_rs::{Buffer, Font, Shared, shape};

fn worker(font: Shared<Font>, jobs: &[&str]) -> Vec<usize> {
    let mut buffer = Buffer::new();          // thread-local, reused below
    let mut counts = Vec::new();

    for text in jobs {
        buffer.push_str(text);
        buffer.guess_segment_properties();

        let output = shape(&font, buffer, &[]);
        counts.push(output.len());

        buffer = output.clear();
    }

    counts
}
```

## The unsafe traits

Two `unsafe` traits sit under the design. You do not need them to use the crate;
you need them to extend it.

### `HarfBuzzObject`

```rust,ignore
pub unsafe trait HarfBuzzObject: Sized {
    type Raw;
    fn as_raw(&self) -> *mut Self::Raw;
    unsafe fn from_raw(raw: *mut Self::Raw) -> Self;
    unsafe fn reference_raw(raw: *mut Self::Raw) -> *mut Self::Raw;
    unsafe fn destroy_raw(raw: *mut Self::Raw);
}
```

Implemented by `Blob`, `Face`, `Font`, and `Buffer`. Its practical use is
`as_raw()`, which hands the underlying C pointer to `harfbuzz_sys` calls the
safe API does not wrap:

```rust
use harfbuzz_rs::{Font, HarfBuzzObject, sys};

/// The vertical advance of a glyph — not wrapped by the safe API.
fn glyph_v_advance(font: &Font, glyph: u32) -> i32 {
    // SAFETY: `font` owns a live `hb_font_t` for the duration of the call, the
    // glyph id is a plain integer, and the function only reads from the font.
    unsafe { sys::hb_font_get_glyph_v_advance(font.as_raw(), glyph) }
}
```

The borrow is exactly that — a borrow. The wrapper still owns its reference, so
never call `hb_*_destroy` on a pointer from `as_raw()`, and never keep it past
the wrapper's lifetime. `Shared::into_raw()` is the deliberate opposite: it
gives up ownership, and the caller becomes responsible for the reference.

Implementing `HarfBuzzObject` yourself is a promise that the wrapper is exactly
one non-null pointer, owns exactly one reference for its whole life, releases it
exactly once on drop, and that `reference_raw`/`destroy_raw` call the matching
HarfBuzz entry points and nothing else.

### `ThreadSafeWhenImmutable`

Described [above](#threadsafewhenimmutable). Implement it only after checking
that upstream documents the type as safe to share once frozen.

## Gotchas

**`Font::new` consumes the `Shared<Face>`.** Clone first if you want more fonts:
`Font::new(face.clone())`.

**Freezing is not free to undo — it is impossible.** Decide what a face's upem
should be before `into_shared()`.

**`Shared<T>` does not deref-mut.** Every setter needs the owned type. If you
find yourself wanting `set_variations` on a `Shared<Font>`, you want a second
font instead.

**`Language::default_from_locale()` is not thread-safe on its first call.** It
inspects the environment and caches the result inside HarfBuzz. Call it once
during start-up if several threads might reach it.

**Reference counts are atomic; contents are not.** Cloning and dropping a
`Shared<T>` from many threads is fine. That is a completely separate question
from whether the object's *contents* may be read concurrently — which is what
`ThreadSafeWhenImmutable` answers.

---

Next: [text-and-buffers.md](text-and-buffers.md) for the one object that is
never shared, or [object-model.md](object-model.md) for how the types stack.

The C-level rules this design encodes are in
[`../harfbuzz-sys/docs/guide/05-object-model.md`](../harfbuzz-sys/docs/guide/05-object-model.md).
