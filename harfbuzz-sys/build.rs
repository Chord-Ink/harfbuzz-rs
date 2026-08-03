//! Build script for `harfbuzz-sys`: compiles the vendored HarfBuzz sources into
//! a static archive and tells Cargo how to link it.
//!
//! # What gets compiled
//!
//! HarfBuzz is a C++ project — upstream's `meson.build` declares
//! `project('harfbuzz', ['c', 'cpp'])` with `cpp_std=c++11`, and `src/` holds
//! 137 `.cc` files against 50 `.h`. It ships an *amalgamation*,
//! `src/harfbuzz-world.cc`, which `#include`s every other translation unit
//! behind `HB_HAS_*` guards. Compiling that one file builds the whole library,
//! and it makes the Cargo features here map one-for-one onto upstream's build
//! switches.
//!
//! # Compiler selection
//!
//! Everything goes through the `cc` crate, so the usual conventions apply and
//! nothing is hard-coded. Because the build is C++, the compiler comes from
//! **`CXX`** (or `CXX_<target>`), not `CC`:
//!
//! ```sh
//! CXX=/path/to/llvm/bin/clang++ cargo build
//! ```
//!
//! # Cross-language LTO
//!
//! LTO is not something this crate can decide on its own. Cross-language LTO
//! only happens when the *consumer* passes `-Clinker-plugin-lto`, which makes
//! `rustc` emit LLVM bitcode instead of machine code. This script looks for
//! that flag in `CARGO_ENCODED_RUSTFLAGS` and, when it is present, compiles
//! HarfBuzz to bitcode too by adding `-flto`, so the linker can optimize across
//! the language boundary.
//!
//! Two conditions gate it:
//!
//! * **The compiler must be genuine LLVM Clang.** GCC's LTO objects use a
//!   different format entirely, and Apple Clang's bitcode is tied to the LLVM
//!   inside Xcode rather than the one `rustc` embeds. In either case this
//!   script warns and compiles normally.
//! * **Without `-Clinker-plugin-lto` there is no `-flto`.** Emitting bitcode
//!   that the consumer's linker was never told to expect turns a working build
//!   into a link failure, and a build script cannot fix a downstream link:
//!   Cargo does not propagate `rustc-link-arg` to dependents.
//!
//! # Environment variables
//!
//! | Variable                     | Effect                                        |
//! | ---------------------------- | --------------------------------------------- |
//! | `CXX`, `CXXFLAGS`            | Compiler and extra flags (read by `cc`)        |
//! | `AR`                         | Archiver (read by `cc`)                        |
//! | `CARGO_ENCODED_RUSTFLAGS`    | Inspected for `-Clinker-plugin-lto`            |
//! | `MACOSX_DEPLOYMENT_TARGET`   | Minimum macOS version (read by `cc`)           |
//! | `PKG_CONFIG_PATH`            | Where to find optional system libraries        |

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let hb_root = manifest_dir.join("harfbuzz");
    let hb_src = hb_root.join("src");

    rerun_directives(&manifest_dir, &hb_src);

    if !hb_src.join("harfbuzz-world.cc").is_file() {
        panic!(
            "vendored HarfBuzz sources are missing at {}\n\
             This crate builds HarfBuzz from a git submodule. Fetch it with:\n\
             \n    git submodule update --init --recursive\n",
            hb_src.display()
        );
    }

    let features = Features::from_env();
    features.warn_about_size_profiles();

    let mut build = base_build(&hb_src, &out_dir, &features);

    // `get_compiler` resolves CXX, the target triple, and the sysroot, so it has
    // to happen after the target-affecting settings are in place.
    let compiler = build.get_compiler();
    let kind = CompilerKind::detect(compiler.path());

    if let Some(flag) = lto_flag(kind) {
        build.flag(flag);
        // With `-flto` the object file is LLVM bitcode, which has no native
        // symbol table. Only an LLVM-aware archiver can index it; a plain `ar`
        // yields an archive the linker reads as empty.
        if let Some(llvm_ar) = find_llvm_ar(compiler.path()) {
            build.archiver(llvm_ar);
        } else {
            println!(
                "cargo::warning=harfbuzz-sys: emitting LLVM bitcode but could not find `llvm-ar`. \
                 If the link reports undefined HarfBuzz symbols, set AR to your toolchain's \
                 llvm-ar."
            );
        }
    }

    // Probe the target the way upstream's meson build does, rather than
    // hard-coding a table of what each platform is assumed to provide.
    let probes = Probe::run_all(&compiler, &out_dir);
    fs::write(out_dir.join("config.h"), render_config_h(&probes, &features))
        .expect("write config.h");

    // `compile` also emits the link-lib and link-search directives for us.
    build.compile("harfbuzz");

    emit_extra_link_directives(&features);
    emit_test_env(&out_dir, compiler.path());
    emit_metadata(&hb_src, kind);
}

// ---------------------------------------------------------------------------
// Cargo re-run rules
// ---------------------------------------------------------------------------

/// Tell Cargo the narrow set of inputs that can change what we produce.
///
/// Pointing `rerun-if-changed` at the source *directory* is deliberate: Cargo
/// walks it recursively, so moving the submodule to a different tag invalidates
/// the build.
fn rerun_directives(manifest_dir: &Path, hb_src: &Path) {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={}", hb_src.display());
    println!(
        "cargo::rerun-if-changed={}",
        manifest_dir.join("Cargo.toml").display()
    );

    // `cc` already watches CXX and friends, but the LTO decision is ours and it
    // reads the encoded rustflags.
    println!("cargo::rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    println!("cargo::rerun-if-env-changed=RUSTFLAGS");
    println!("cargo::rerun-if-env-changed=PKG_CONFIG_PATH");
}

// ---------------------------------------------------------------------------
// The compile
// ---------------------------------------------------------------------------

/// Configure everything about the build that does not depend on knowing which
/// compiler `cc` resolved.
fn base_build(hb_src: &Path, out_dir: &Path, features: &Features) -> cc::Build {
    let mut build = cc::Build::new();

    build
        .cpp(true)
        .file(hb_src.join("harfbuzz-world.cc"))
        .include(hb_src)
        // Where `config.h` lands.
        .include(out_dir)
        .define("HAVE_CONFIG_H", None);

    if let Some(sdk) = apple_sdk_path() {
        build.flag("-isysroot").flag(&sdk);
    }

    // ICU 75 and later need C++17 in their public headers; upstream's meson
    // build special-cases this the same way.
    let needs_cxx17 = features
        .icu
        .as_ref()
        .and_then(PkgConfig::major)
        .is_some_and(|v| v >= 75);
    build.std(if needs_cxx17 { "c++17" } else { "c++11" });

    // Matched to upstream's meson defaults. RTTI stays on when ICU is linked,
    // because ICU's headers use it — upstream's meson.build carries the same
    // caveat.
    build.flag_if_supported("-fno-exceptions");
    if features.icu.is_none() {
        build.flag_if_supported("-fno-rtti");
    }
    build.flag_if_supported("-fno-threadsafe-statics");
    build.flag_if_supported("-fvisibility-inlines-hidden");

    // Size first, as requested. `-ffunction-sections`/`-fdata-sections` give
    // the linker symbol-granularity dead stripping; rustc already passes
    // `-Wl,-dead_strip` (Mach-O) and `--gc-sections` (ELF) by default, so there
    // is nothing to add on the link side.
    build.opt_level_str("z");
    build.flag_if_supported("-ffunction-sections");
    build.flag_if_supported("-fdata-sections");

    // The `debug` feature owns this, rather than Cargo's profile, so that a
    // release build can still carry a debuggable HarfBuzz.
    build.debug(features.debug);
    if features.debug {
        // Frame pointers keep stacks walkable for profilers that do not parse
        // DWARF. Mandatory on arm64 anyway; stating it keeps other targets
        // consistent.
        build.flag_if_supported("-fno-omit-frame-pointer");
    }

    // HarfBuzz builds clean upstream but warns freely under our flags, and
    // those warnings are not actionable from here.
    build.warnings(false);
    build.extra_warnings(false);

    // The `HB_HAS_*` switches the amalgamation dispatches on. These must be
    // defined on the command line: `harfbuzz-world.cc` tests them before it
    // includes anything, so `config.h` would be too late.
    for (enabled, macro_name) in [
        (features.subset, "HB_HAS_SUBSET"),
        (features.raster, "HB_HAS_RASTER"),
        (features.vector, "HB_HAS_VECTOR"),
        (features.gpu, "HB_HAS_GPU"),
        (features.coretext, "HB_HAS_CORETEXT"),
        (features.freetype.is_some(), "HB_HAS_FREETYPE"),
        (features.graphite2.is_some(), "HB_HAS_GRAPHITE"),
        (features.icu.is_some(), "HB_HAS_ICU"),
        (features.glib.is_some(), "HB_HAS_GLIB"),
    ] {
        if enabled {
            build.define(macro_name, "1");
        }
    }

    // Size profiles from upstream's CONFIG.md.
    for (enabled, macro_name) in [
        (features.tiny, "HB_TINY"),
        (features.lean, "HB_LEAN"),
        (features.mini, "HB_MINI"),
    ] {
        if enabled {
            build.define(macro_name, "1");
        }
    }

    for pkg in features.pkg_configs() {
        for flag in &pkg.cflags {
            build.flag(flag);
        }
    }

    build
}

/// Locate the Apple SDK, for compilers that do not know where it is.
///
/// Apple's own clang has the SDK path baked in, and `cc` relies on that, so it
/// passes no `-isysroot` for a macOS host build. A standalone LLVM clang has no
/// such default and fails on the very first system header. Since pointing an
/// LLVM toolchain at this crate is the supported way to get cross-language LTO,
/// the sysroot has to be supplied explicitly.
///
/// Returns `None` off Apple platforms, and when `SDKROOT` is already set —
/// clang honours that itself, and overriding it would ignore the user.
fn apple_sdk_path() -> Option<String> {
    if env::var("CARGO_CFG_TARGET_VENDOR").as_deref() != Ok("apple") {
        return None;
    }

    if env::var_os("SDKROOT").is_some() {
        return None;
    }

    let sdk = match env::var("CARGO_CFG_TARGET_OS").as_deref() {
        Ok("ios") => "iphoneos",
        Ok("tvos") => "appletvos",
        Ok("watchos") => "watchos",
        Ok("visionos") => "xros",
        _ => "macosx",
    };

    let output = Command::new("xcrun")
        .args(["--sdk", sdk, "--show-sdk-path"])
        .output()
        .ok()?;

    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .filter(|path| !path.is_empty())
}

// ---------------------------------------------------------------------------
// LTO
// ---------------------------------------------------------------------------

/// Which compiler `cc` resolved, to the precision the LTO decision needs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum CompilerKind {
    /// Upstream LLVM Clang.
    Clang,
    /// Apple's fork, shipped with Xcode. Its bitcode belongs to the LLVM inside
    /// Xcode, which is not the one `rustc` embeds.
    AppleClang,
    /// GCC, or anything claiming to be it.
    Gnu,
    /// Anything unrecognised.
    Other,
}

impl CompilerKind {
    /// Read the compiler's own version banner.
    ///
    /// Apple Clang identifies itself as `Apple clang version 17.0.0 (...)`,
    /// upstream as `clang version 22.1.2 (https://github.com/llvm/...)`. There
    /// is no flag that distinguishes them, so the banner is what we have.
    fn detect(path: &Path) -> Self {
        let Ok(output) = Command::new(path).arg("--version").output() else {
            return Self::Other;
        };

        let banner = String::from_utf8_lossy(&output.stdout).to_lowercase();

        if banner.contains("apple clang") || banner.contains("apple llvm") {
            Self::AppleClang
        } else if banner.contains("clang version") {
            Self::Clang
        } else if banner.contains("free software foundation") || banner.contains("gcc") {
            Self::Gnu
        } else {
            Self::Other
        }
    }

    fn describe(self) -> &'static str {
        match self {
            Self::Clang => "LLVM Clang",
            Self::AppleClang => "Apple Clang",
            Self::Gnu => "GCC",
            Self::Other => "an unrecognised compiler",
        }
    }
}

/// Decide whether to compile HarfBuzz to LLVM bitcode.
///
/// Cross-language LTO is driven from the consumer's side: `-Clinker-plugin-lto`
/// is what makes `rustc` emit bitcode rather than machine code, and only then
/// does it help for the C++ half to be bitcode too. Without that flag, emitting
/// bitcode here would hand the consumer's linker something it was not set up to
/// read.
fn lto_flag(kind: CompilerKind) -> Option<&'static str> {
    if !consumer_wants_cross_language_lto() {
        return None;
    }

    if kind == CompilerKind::Clang {
        return Some("-flto");
    }

    println!(
        "cargo::warning=harfbuzz-sys: -Clinker-plugin-lto is set, but the C++ compiler is {}. \
         Cross-language LTO needs the same LLVM that rustc embeds, so HarfBuzz is being compiled \
         without -flto. Set CXX to an LLVM clang++ whose version matches `rustc -vV`.",
        kind.describe()
    );

    None
}

/// Look for `-Clinker-plugin-lto` in the flags Cargo is passing to `rustc`.
///
/// `CARGO_ENCODED_RUSTFLAGS` is the authoritative form — it is `\x1f`-separated
/// so that flags containing spaces survive intact. `RUSTFLAGS` is checked as a
/// fallback for the rare setup that sets it without Cargo re-encoding it.
fn consumer_wants_cross_language_lto() -> bool {
    let encoded = env::var("CARGO_ENCODED_RUSTFLAGS")
        .map(|flags| flags.split('\u{1f}').any(mentions_linker_plugin_lto))
        .unwrap_or(false);

    encoded
        || env::var("RUSTFLAGS")
            .map(|flags| flags.split_whitespace().any(mentions_linker_plugin_lto))
            .unwrap_or(false)
}

/// Match every spelling `rustc` accepts: `-Clinker-plugin-lto`,
/// `-C linker-plugin-lto` (which arrives as two separate encoded flags),
/// `-Clinker-plugin-lto=yes`, and `--codegen linker-plugin-lto`.
///
/// A bare `linker-plugin-lto` is matched too, because that is what the value
/// half of a split `-C linker-plugin-lto` looks like.
fn mentions_linker_plugin_lto(flag: &str) -> bool {
    flag.trim_start_matches(['-', 'C'])
        .trim_start_matches("-codegen")
        .trim_start()
        .starts_with("linker-plugin-lto")
}

/// Find the LLVM archiver that belongs to the compiler we are using.
///
/// Respecting `AR` first matters for cross-compilation setups that already
/// point at the right tool; `cc` reads it too, so returning `None` there leaves
/// `cc` in charge.
fn find_llvm_ar(compiler: &Path) -> Option<PathBuf> {
    if env::var_os("AR").is_some() {
        return None;
    }

    compiler
        .parent()
        .map(|bin| bin.join("llvm-ar"))
        .filter(|ar| ar.is_file())
        .or_else(|| which("llvm-ar"))
}

/// Minimal `which`, so the build script keeps its dependency list to `cc`.
fn which(program: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|dir| dir.join(program))
            .find(|candidate| candidate.is_file())
    })
}

// ---------------------------------------------------------------------------
// Features
// ---------------------------------------------------------------------------

/// The enabled Cargo features, resolved against what the target can provide.
struct Features {
    subset: bool,
    raster: bool,
    vector: bool,
    gpu: bool,
    coretext: bool,
    freetype: Option<PkgConfig>,
    graphite2: Option<PkgConfig>,
    icu: Option<PkgConfig>,
    glib: Option<PkgConfig>,
    png: Option<PkgConfig>,
    zlib: Option<PkgConfig>,
    experimental: bool,
    debug: bool,
    mini: bool,
    lean: bool,
    tiny: bool,
}

impl Features {
    fn from_env() -> Self {
        let on = |name: &str| env::var_os(format!("CARGO_FEATURE_{name}")).is_some();
        let is_apple = env::var("CARGO_CFG_TARGET_VENDOR").as_deref() == Ok("apple");

        // CoreText only exists on Apple platforms. Ignoring the feature
        // elsewhere is friendlier than failing, because Cargo features are
        // additive: an unrelated crate in the graph may have turned it on.
        let coretext = on("CORETEXT") && is_apple;
        if on("CORETEXT") && !coretext {
            println!(
                "cargo::warning=harfbuzz-sys: ignoring the `coretext` feature on a non-Apple \
                 target"
            );
        }

        Self {
            subset: on("SUBSET"),
            raster: on("RASTER"),
            vector: on("VECTOR"),
            gpu: on("GPU"),
            coretext,
            freetype: PkgConfig::require(on("FREETYPE"), "freetype2"),
            graphite2: PkgConfig::require(on("GRAPHITE2"), "graphite2"),
            icu: PkgConfig::require(on("ICU"), "icu-uc"),
            glib: PkgConfig::require(on("GLIB"), "glib-2.0"),
            png: PkgConfig::require(on("PNG"), "libpng"),
            zlib: PkgConfig::require(on("ZLIB"), "zlib"),
            experimental: on("EXPERIMENTAL"),
            debug: on("DEBUG"),
            mini: on("MINI"),
            lean: on("LEAN"),
            tiny: on("TINY"),
        }
    }

    /// The size profiles work by deleting code, including public entry points.
    /// The FFI declarations stay in `lib.rs` either way, so a call to something
    /// that was compiled out surfaces as a link error rather than a compile
    /// error. Say so up front.
    fn warn_about_size_profiles(&self) {
        for (enabled, name) in [(self.tiny, "tiny"), (self.lean, "lean"), (self.mini, "mini")] {
            if enabled {
                println!(
                    "cargo::warning=harfbuzz-sys: the `{name}` feature removes public HarfBuzz \
                     API to save space. Bindings for removed functions still compile but will \
                     fail to link. See harfbuzz/CONFIG.md."
                );
            }
        }
    }

    /// Every external library we located, in link order.
    fn pkg_configs(&self) -> impl Iterator<Item = &PkgConfig> {
        [
            self.freetype.as_ref(),
            self.graphite2.as_ref(),
            self.icu.as_ref(),
            self.glib.as_ref(),
            self.png.as_ref(),
            self.zlib.as_ref(),
        ]
        .into_iter()
        .flatten()
    }
}

// ---------------------------------------------------------------------------
// pkg-config
// ---------------------------------------------------------------------------

/// The compile and link flags for one system library, as reported by
/// `pkg-config`.
struct PkgConfig {
    name: String,
    cflags: Vec<String>,
    libs: Vec<String>,
    modversion: Option<String>,
}

impl PkgConfig {
    /// Look up `name` when `enabled`, panicking with an actionable message if
    /// the library is missing. A feature the user explicitly asked for should
    /// never be silently dropped.
    fn require(enabled: bool, name: &str) -> Option<Self> {
        if !enabled {
            return None;
        }

        let query = |arg: &str| -> Option<Vec<String>> {
            let out = Command::new("pkg-config").args([arg, name]).output().ok()?;
            out.status.success().then(|| {
                String::from_utf8_lossy(&out.stdout)
                    .split_whitespace()
                    .map(str::to_string)
                    .collect()
            })
        };

        let cflags = query("--cflags").unwrap_or_else(|| {
            panic!(
                "the requested feature needs the `{name}` library, but `pkg-config --cflags \
                 {name}` failed.\nInstall it, or point PKG_CONFIG_PATH at it."
            )
        });

        let modversion = Command::new("pkg-config")
            .args(["--modversion", name])
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());

        Some(Self {
            name: name.to_string(),
            cflags,
            libs: query("--libs").unwrap_or_default(),
            modversion,
        })
    }

    /// The major version, used to apply version-dependent build rules.
    fn major(&self) -> Option<u32> {
        self.modversion.as_ref()?.split('.').next()?.parse().ok()
    }
}

// ---------------------------------------------------------------------------
// Configuration probes
// ---------------------------------------------------------------------------

/// One `HAVE_*` fact about the target, established by compiling a snippet
/// rather than assumed from the platform name.
struct Probe {
    /// The macro to define in `config.h`, e.g. `HAVE_MMAP`.
    macro_name: &'static str,
    available: bool,
}

/// The headers upstream's meson build checks for.
const HEADER_PROBES: &[(&str, &str)] = &[
    ("HAVE_UNISTD_H", "unistd.h"),
    ("HAVE_SYS_MMAN_H", "sys/mman.h"),
    ("HAVE_STDBOOL_H", "stdbool.h"),
    ("HAVE_XLOCALE_H", "xlocale.h"),
];

/// The functions upstream's meson build checks for, with the header that
/// declares each. `sincos`/`sincosf` are GNU extensions and deliberately absent
/// on Apple platforms, which is exactly why they are probed rather than assumed.
const FUNCTION_PROBES: &[(&str, &str, &str)] = &[
    ("HAVE_ATEXIT", "atexit", "#include <stdlib.h>"),
    ("HAVE_MPROTECT", "mprotect", "#include <sys/mman.h>"),
    ("HAVE_SYSCONF", "sysconf", "#include <unistd.h>"),
    ("HAVE_GETPAGESIZE", "getpagesize", "#include <unistd.h>"),
    ("HAVE_MMAP", "mmap", "#include <sys/mman.h>"),
    ("HAVE_ISATTY", "isatty", "#include <unistd.h>"),
    ("HAVE_USELOCALE", "uselocale", "#include <locale.h>"),
    ("HAVE_NEWLOCALE", "newlocale", "#include <locale.h>"),
    ("HAVE_LOCALECONV_L", "localeconv_l", "#include <locale.h>"),
    (
        "HAVE_SINCOS",
        "sincos",
        "#define _GNU_SOURCE\n#include <math.h>",
    ),
    (
        "HAVE_SINCOSF",
        "sincosf",
        "#define _GNU_SOURCE\n#include <math.h>",
    ),
];

impl Probe {
    /// Run every probe, in parallel. Each is a sub-second compile, but there
    /// are fifteen of them and they are completely independent.
    fn run_all(compiler: &cc::Tool, out_dir: &Path) -> Vec<Self> {
        let scratch = out_dir.join("probes");
        fs::create_dir_all(&scratch).expect("create probe directory");

        let jobs: Vec<(&'static str, String)> = HEADER_PROBES
            .iter()
            .map(|(macro_name, header)| {
                (
                    *macro_name,
                    format!("#include <{header}>\nint main(void){{return 0;}}\n"),
                )
            })
            .chain(FUNCTION_PROBES.iter().map(|(macro_name, func, prefix)| {
                // Taking the function's address forces the compiler to resolve
                // a real declaration, which a bare call could satisfy with an
                // implicit one.
                (
                    *macro_name,
                    format!("{prefix}\nvoid *probe(void){{return (void *)&{func};}}\n"),
                )
            }))
            .collect();

        thread::scope(|scope| {
            let handles: Vec<_> = jobs
                .iter()
                .map(|(macro_name, source)| {
                    let scratch = &scratch;
                    scope.spawn(move || Self {
                        macro_name: *macro_name,
                        available: compiles(compiler, scratch, macro_name, source),
                    })
                })
                .collect();

            handles
                .into_iter()
                .map(|h| h.join().expect("probe thread panicked"))
                .collect()
        })
    }
}

/// Compile `source` to an object file and report whether it succeeded.
///
/// The command comes from `cc`, so it already carries the target triple, the
/// sysroot, and anything the user put in `CXXFLAGS` — the probe therefore sees
/// the same world the real compile will.
fn compiles(compiler: &cc::Tool, scratch: &Path, name: &str, source: &str) -> bool {
    let file = scratch.join(format!("{name}.c"));
    let object = scratch.join(format!("{name}.o"));
    if fs::write(&file, source).is_err() {
        return false;
    }

    let mut cmd = compiler.to_command();
    cmd.args(["-x", "c", "-c", "-w"]);
    cmd.arg(&file).arg("-o").arg(&object);

    cmd.output().map(|o| o.status.success()).unwrap_or(false)
}

/// Render the `config.h` that HarfBuzz picks up via `-DHAVE_CONFIG_H`.
///
/// This is the same mechanism meson uses, so the vendored sources see exactly
/// the shape of configuration they were written against.
fn render_config_h(probes: &[Probe], features: &Features) -> String {
    let mut out = String::from(
        "/* Generated by harfbuzz-sys/build.rs. Do not edit. */\n\
         #ifndef HARFBUZZ_SYS_CONFIG_H\n\
         #define HARFBUZZ_SYS_CONFIG_H\n\n",
    );

    for probe in probes {
        if probe.available {
            let _ = writeln!(out, "#define {} 1", probe.macro_name);
        } else {
            let _ = writeln!(out, "/* #undef {} */", probe.macro_name);
        }
    }

    // HarfBuzz's atomics and locks compile to no-ops without this. Every
    // target this crate supports has pthreads; Windows uses its own path.
    if env::var("CARGO_CFG_TARGET_FAMILY").as_deref() != Ok("windows") {
        out.push_str("\n#define HAVE_PTHREAD 1\n");
    }

    out.push('\n');

    // Libraries whose presence the amalgamation does *not* infer from
    // `HB_HAS_*`. Two of these paper over upstream bugs in
    // `harfbuzz-world.cc`: it translates `HB_HAS_GRAPHITE` into
    // `HAVE_GRAPHITE`, but every consumer in the sources tests
    // `HAVE_GRAPHITE2`; and it never translates `HB_HAS_ICU` at all. Without
    // these two defines the sources compile to empty translation units that
    // link fine and do nothing.
    for (enabled, macro_name) in [
        (features.icu.is_some(), "HAVE_ICU"),
        (features.png.is_some(), "HAVE_PNG"),
        (features.zlib.is_some(), "HAVE_ZLIB"),
        (features.graphite2.is_some(), "HAVE_GRAPHITE2"),
    ] {
        if enabled {
            let _ = writeln!(out, "#define {macro_name} 1");
        }
    }

    if features.experimental {
        out.push_str("#define HB_EXPERIMENTAL_API 1\n");
    }

    out.push_str("\n#endif /* HARFBUZZ_SYS_CONFIG_H */\n");
    out
}

// ---------------------------------------------------------------------------
// Cargo output
// ---------------------------------------------------------------------------

/// Link directives `cc` does not emit for us: system frameworks and the
/// libraries `pkg-config` reported.
fn emit_extra_link_directives(features: &Features) {
    if features.coretext {
        // ApplicationServices umbrella-links CoreText, CoreGraphics, and
        // CoreFoundation on macOS. iOS-family targets expose them separately.
        if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("macos") {
            println!("cargo::rustc-link-lib=framework=ApplicationServices");
        } else {
            for framework in ["CoreText", "CoreGraphics", "CoreFoundation"] {
                println!("cargo::rustc-link-lib=framework={framework}");
            }
        }
    }

    for pkg in features.pkg_configs() {
        if pkg.libs.is_empty() {
            println!(
                "cargo::warning=harfbuzz-sys: `pkg-config --libs {}` returned nothing; the link \
                 step will probably report undefined symbols.",
                pkg.name
            );
        }

        for flag in &pkg.libs {
            if let Some(lib) = flag.strip_prefix("-l") {
                println!("cargo::rustc-link-lib={lib}");
            } else if let Some(dir) = flag.strip_prefix("-L") {
                println!("cargo::rustc-link-search=native={dir}");
            } else if let Some(framework) = flag.strip_prefix("-framework") {
                let framework = framework.trim();
                if !framework.is_empty() {
                    println!("cargo::rustc-link-lib=framework={framework}");
                }
            }
        }
    }
}

/// Hand `tests/symbols.rs` what it needs to check every declared binding
/// against the symbols the archive actually defines.
fn emit_test_env(out_dir: &Path, compiler: &Path) {
    println!(
        "cargo::rustc-env=HARFBUZZ_SYS_ARCHIVE={}",
        out_dir.join("libharfbuzz.a").display()
    );

    // With LTO on, the archive members are bitcode and only an LLVM-aware
    // reader can list their symbols.
    let nm = compiler
        .parent()
        .map(|bin| bin.join("llvm-nm"))
        .filter(|nm| nm.is_file())
        .or_else(|| which("llvm-nm"))
        .unwrap_or_else(|| PathBuf::from("nm"));
    println!("cargo::rustc-env=HARFBUZZ_SYS_NM={}", nm.display());
}

/// Publish what we did to dependent build scripts.
///
/// Because this package declares `links = "harfbuzz"`, Cargo forwards each of
/// these to dependents as `DEP_HARFBUZZ_<KEY>`.
fn emit_metadata(hb_src: &Path, kind: CompilerKind) {
    println!("cargo::metadata=include={}", hb_src.display());
    println!("cargo::metadata=compiler={:?}", kind);
}
