//! Raw FFI bindings to [HarfBuzz](https://harfbuzz.github.io/) 14.3.0.
//!
//! This crate is a direct, unopinionated transcription of HarfBuzz's public C
//! headers. It applies no safety, no ownership, and no error handling of its
//! own — use [`harfbuzz-rs`](https://docs.rs/harfbuzz-rs) for that. What you get
//! here is the C API, spelled in Rust, with the same names and the same
//! semantics.
//!
//! The HarfBuzz sources are vendored as a git submodule and compiled by
//! `build.rs`; see the crate's `docs/` directory for the API reference.
//!
//! # Transcription conventions
//!
//! Knowing these makes the rest of the crate predictable:
//!
//! * **Opaque objects** — `hb_blob_t`, `hb_font_t` and friends are declared with
//!   [`opaque_handle!`], which produces a zero-sized `#[repr(C)]` type that
//!   cannot be constructed, moved out of, or shared across threads by accident.
//!   You only ever handle them through pointers, exactly as in C.
//!
//! * **Enumerations** — C enumerations become an integer type alias plus a set
//!   of `const` values, rather than a Rust `enum`. This is deliberate: HarfBuzz
//!   returns enumeration values that come from font data, and a Rust `enum`
//!   holding a value outside its variant list is undefined behaviour. An alias
//!   can hold anything the C side produces, and the safe wrapper is where those
//!   values get validated.
//!
//! * **Callbacks** — function-pointer typedefs are wrapped in [`Option`], so
//!   that `None` is the null pointer. HarfBuzz accepts null for most callbacks
//!   and for every `destroy` argument, and this makes that representable.
//!
//! * **Booleans** — [`hb_bool_t`] is a C `int`. Zero is false; anything else is
//!   true.
//!
//! * **Nullability** — the C headers do not distinguish nullable from non-null
//!   pointers, so neither does this crate. Each function's documentation in
//!   `docs/` records what HarfBuzz actually accepts.
//!
//! # Features
//!
//! Features mirror upstream's optional sub-libraries and back ends one-for-one.
//! Enabling one compiles the corresponding HarfBuzz sources *and* exposes the
//! matching module here; the two cannot drift apart.
//!
//! | Feature      | Module        | Upstream header      |
//! | ------------ | ------------- | -------------------- |
//! | `subset`     | [`subset`]    | `hb-subset.h`        |
//! | `raster`     | [`raster`]    | `hb-raster.h`        |
//! | `vector`     | [`vector`]    | `hb-vector.h`        |
//! | `gpu`        | [`gpu`]       | `hb-gpu.h`           |
//! | `coretext`   | [`coretext`]  | `hb-coretext.h`      |
//! | `freetype`   | [`ft`]        | `hb-ft.h`            |
//! | `graphite2`  | [`graphite2`] | `hb-graphite2.h`     |
//! | `icu`        | [`icu`]       | `hb-icu.h`           |
//! | `glib`       | [`glib`]      | `hb-glib.h`          |

#![no_std]
// A -sys crate's job is to reproduce C names exactly, so that anyone reading
// HarfBuzz's own documentation can find them here unchanged.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

/// Declare an opaque C type that is only ever used behind a pointer.
///
/// The generated type is zero-sized and has a private field, so it cannot be
/// constructed or copied. The [`PhantomData`](core::marker::PhantomData) also
/// makes it `!Send`, `!Sync`, and `!Unpin`, which is the honest default: the C
/// object it stands for may have interior mutability and may not be safe to
/// move or share. The safe wrapper opts back into those traits where HarfBuzz
/// documents that it is sound.
macro_rules! opaque_handle {
    ($(#[$meta:meta])* $name:ident) => {
        $(#[$meta])*
        #[repr(C)]
        pub struct $name {
            _data: [u8; 0],
            _marker: core::marker::PhantomData<(*mut u8, core::marker::PhantomPinned)>,
        }
    };
}

pub(crate) use opaque_handle;

mod common;
mod script;
mod version;

mod blob;
mod buffer;
mod draw;
mod face;
mod font;
mod map;
mod paint;
mod set;
mod shape;
mod shape_plan;
mod style;
mod unicode;

mod aat_layout;
mod ot_color;
mod ot_deprecated;
mod ot_fetch;
mod ot_font;
mod ot_layout;
mod ot_math;
mod ot_meta;
mod ot_metrics;
mod ot_name;
mod ot_shape;
mod ot_var;

mod deprecated;

pub use common::*;
pub use script::*;
pub use version::*;

pub use blob::*;
pub use buffer::*;
pub use draw::*;
pub use face::*;
pub use font::*;
pub use map::*;
pub use paint::*;
pub use set::*;
pub use shape::*;
pub use shape_plan::*;
pub use style::*;
pub use unicode::*;

pub use aat_layout::*;
pub use ot_color::*;
pub use ot_deprecated::*;
pub use ot_fetch::*;
pub use ot_font::*;
pub use ot_layout::*;
pub use ot_math::*;
pub use ot_meta::*;
pub use ot_metrics::*;
pub use ot_name::*;
pub use ot_shape::*;
pub use ot_var::*;

pub use deprecated::*;

// Optional sub-libraries. Each is gated on the same feature that decides
// whether `build.rs` compiles the sources behind it, so a declaration here can
// never refer to a symbol that was left out of the archive.

#[cfg(feature = "subset")]
pub mod subset;

#[cfg(feature = "raster")]
pub mod raster;

#[cfg(feature = "vector")]
pub mod vector;

#[cfg(feature = "gpu")]
pub mod gpu;

#[cfg(feature = "coretext")]
pub mod coretext;

#[cfg(feature = "freetype")]
pub mod ft;

#[cfg(feature = "graphite2")]
pub mod graphite2;

#[cfg(feature = "icu")]
pub mod icu;

#[cfg(feature = "glib")]
pub mod glib;
