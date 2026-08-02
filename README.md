# harfbuzz-rs

Safe, idiomatic Rust bindings for [HarfBuzz](https://github.com/harfbuzz/harfbuzz), the
text shaping engine.

The workspace has two crates:

| Crate          | Purpose                                                             |
| -------------- | ------------------------------------------------------------------- |
| `harfbuzz-sys` | Raw FFI declarations plus a vendored HarfBuzz build (`build.rs`).     |
| `harfbuzz-rs`  | The safe wrapper: ownership, lifetimes, and error handling.           |

HarfBuzz itself is vendored as a git submodule pinned to a tagged release, so a clone
needs `--recursive`:

```sh
git clone --recursive https://github.com/Chord-Ink/harfbuzz-rs
```

