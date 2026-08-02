//! Build script for `harfbuzz-sys`: compiles the vendored HarfBuzz sources into
//! a static archive and tells Cargo how to link it.
//!
//! # Why this does not use the `cc` crate
//!
//! This project takes no dependencies it does not need, and the amount of work
//! here is small: HarfBuzz ships an *amalgamation* (`src/harfbuzz-world.cc`)
//! that `#include`s every other translation unit, so the whole library is a
//! single compiler invocation. Driving `clang++` directly also lets us pin an
//! exact toolchain, which matters for the LTO story below.
//!
//! # Toolchain pinning and cross-language LTO
//!
//! When HarfBuzz is compiled with `-flto`, the object file contains LLVM
//! bitcode rather than machine code. Bitcode is only forward-compatible: a
//! linker whose LLVM is *older* than the producer cannot read it. Apple's
//! `ld64` ships its own libLTO (LLVM 21 as of Xcode 26), so bitcode from a
//! newer upstream LLVM fails to link with the error
//!
//! ```text
//! ld: could not parse bitcode object file ...: 'Unknown attribute kind ...'
//! ```
//!
//! To make cross-language LTO work, the C++ side must be compiled by the *same*
//! LLVM that `rustc` embeds. This script therefore
//!
//! 1. locates a pinned LLVM toolchain (see [`Toolchain::discover`]),
//! 2. compares its major version against the LLVM `rustc` reports, and
//! 3. only emits bitcode when the two agree — otherwise it falls back to plain
//!    machine-code objects so the build still succeeds, just without LTO.
//!
//! Consumers additionally need `-Clinker-plugin-lto` and a matching linker; see
//! `.cargo/config.toml` in the repository root for a working configuration.
//!
//! # Environment variables
//!
//! | Variable                        | Effect                                          |
//! | ------------------------------- | ----------------------------------------------- |
//! | `HARFBUZZ_SYS_LLVM_TOOLCHAIN`   | Path to an LLVM install (the dir holding `bin/`) |
//! | `HARFBUZZ_SYS_LTO`              | `thin` (default), `full`, or `off`               |
//! | `HARFBUZZ_SYS_NO_DEBUG_INFO`    | Set to any value to drop `-g`                    |
//! | `MACOSX_DEPLOYMENT_TARGET`      | Minimum macOS version to target                  |

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;

/// Where the pinned toolchain lives when the user has not said otherwise.
/// `rustc` 1.96 embeds LLVM 22.1.2, so this is the version that lets bitcode
/// flow from C++ into a Rust link.
const CANONICAL_TOOLCHAIN: &str = "Developer/SDK/llvm/toolchains/22.1.2/darwin-aarch64";

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

    let target = Target::from_env();
    let features = Features::from_env(&target);
    let toolchain = Toolchain::discover(&target);

    features.warn_about_size_profiles();

    // Probe the host/target the way upstream's meson build does, rather than
    // hard-coding a table of what each platform is assumed to provide.
    let probes = Probe::run_all(&toolchain, &target);
    let config_h = out_dir.join("config.h");
    fs::write(&config_h, render_config_h(&probes, &features)).expect("write config.h");

    let lto = LtoMode::resolve(&toolchain);
    let object = compile(&toolchain, &target, &features, &lto, &hb_src, &out_dir);
    let archive = archive(&toolchain, &object, &out_dir);

    emit_link_directives(&target, &features, &archive, &out_dir);
    emit_lto_linker_directives(&toolchain, &lto);
    emit_metadata(&hb_root, &lto, &toolchain);
}

// ---------------------------------------------------------------------------
// Cargo re-run rules
// ---------------------------------------------------------------------------

/// Tell Cargo the narrow set of inputs that can change what we produce.
///
/// Pointing `rerun-if-changed` at the source *directory* is deliberate: Cargo
/// walks it recursively, so adding or editing any vendored file (for instance
/// by moving the submodule to a different tag) invalidates the build.
fn rerun_directives(manifest_dir: &Path, hb_src: &Path) {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={}", hb_src.display());
    println!(
        "cargo::rerun-if-changed={}",
        manifest_dir.join("Cargo.toml").display()
    );

    for var in [
        "HARFBUZZ_SYS_LLVM_TOOLCHAIN",
        "HARFBUZZ_SYS_LTO",
        "HARFBUZZ_SYS_NO_DEBUG_INFO",
        "MACOSX_DEPLOYMENT_TARGET",
        "CXX",
        "AR",
    ] {
        println!("cargo::rerun-if-env-changed={var}");
    }
}

// ---------------------------------------------------------------------------
// Target
// ---------------------------------------------------------------------------

/// The platform we are compiling *for*, translated from Cargo's vocabulary into
/// clang's.
struct Target {
    /// The Rust triple, e.g. `aarch64-apple-darwin`.
    triple: String,
    /// `CARGO_CFG_TARGET_OS`, e.g. `macos`.
    os: String,
    /// `CARGO_CFG_TARGET_ARCH`, e.g. `aarch64`.
    arch: String,
    /// True for macOS/iOS/tvOS/watchOS/visionOS, which share Mach-O, the
    /// Apple SDK layout, and `-dead_strip`.
    is_apple: bool,
}

impl Target {
    fn from_env() -> Self {
        let os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS");
        let is_apple = env::var("CARGO_CFG_TARGET_VENDOR").as_deref() == Ok("apple");

        Self {
            triple: env::var("TARGET").expect("TARGET"),
            arch: env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH"),
            os,
            is_apple,
        }
    }

    /// The triple to hand clang.
    ///
    /// Rust spells the Apple triples `aarch64-apple-darwin`, which clang
    /// accepts but reads as "whatever Darwin the host is". Naming the platform
    /// and its minimum version explicitly is what makes the build reproducible
    /// across machines.
    fn clang_triple(&self) -> String {
        let arch = match self.arch.as_str() {
            "aarch64" => "arm64",
            other => other,
        };

        if self.os == "macos" {
            let min = env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| {
                // Apple silicon has never shipped anything older than Big Sur.
                match arch {
                    "arm64" => "11.0".to_string(),
                    _ => "10.12".to_string(),
                }
            });
            format!("{arch}-apple-macosx{min}")
        } else {
            self.triple.clone()
        }
    }

    /// Apple's clang needs to be told where the SDK is when it is not the
    /// Xcode-bundled one. `xcrun` is the supported way to ask.
    fn sdk_path(&self) -> Option<String> {
        if !self.is_apple {
            return None;
        }

        let sdk = match self.os.as_str() {
            "ios" => "iphoneos",
            "tvos" => "appletvos",
            "watchos" => "watchos",
            "visionos" => "xros",
            _ => "macosx",
        };

        let out = Command::new("xcrun")
            .args(["--sdk", sdk, "--show-sdk-path"])
            .output()
            .ok()?;

        out.status
            .success()
            .then(|| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .filter(|p| !p.is_empty())
    }
}

// ---------------------------------------------------------------------------
// Toolchain
// ---------------------------------------------------------------------------

/// The C++ compiler and archiver used for this build, plus the LLVM version
/// behind them.
struct Toolchain {
    cxx: PathBuf,
    ar: PathBuf,
    /// LLVM major version of `cxx`, if it identified itself as clang.
    clang_llvm_major: Option<u32>,
    /// LLVM major version `rustc` reports for itself.
    rustc_llvm_major: Option<u32>,
}

impl Toolchain {
    /// Resolve the toolchain, preferring the most specific instruction available.
    ///
    /// Search order:
    /// 1. `HARFBUZZ_SYS_LLVM_TOOLCHAIN` — an explicit LLVM install root.
    /// 2. `$HOME/Developer/SDK/llvm/toolchains/22.1.2/darwin-aarch64` — the
    ///    canonical pinned toolchain for this project.
    /// 3. `CXX` / `AR` — the conventional cross-compilation escape hatch.
    /// 4. `clang++` and `llvm-ar` from `PATH`, then `c++` and `ar`.
    ///
    /// Only case 1 and 2 reliably give bitcode a linker can read, so the
    /// version check in [`LtoMode::resolve`] does the actual gating.
    fn discover(target: &Target) -> Self {
        let from_root = |root: &Path| -> Option<(PathBuf, PathBuf)> {
            let bin = root.join("bin");
            let cxx = bin.join("clang++");
            let ar = bin.join("llvm-ar");
            (cxx.is_file() && ar.is_file()).then_some((cxx, ar))
        };

        let explicit = env::var_os("HARFBUZZ_SYS_LLVM_TOOLCHAIN").map(PathBuf::from);
        if let Some(root) = &explicit {
            let found = from_root(root).unwrap_or_else(|| {
                panic!(
                    "HARFBUZZ_SYS_LLVM_TOOLCHAIN={} does not contain bin/clang++ and bin/llvm-ar",
                    root.display()
                )
            });
            return Self::probe_versions(found.0, found.1);
        }

        let canonical = env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(CANONICAL_TOOLCHAIN))
            .and_then(|root| from_root(&root));
        if let Some((cxx, ar)) = canonical {
            return Self::probe_versions(cxx, ar);
        }

        // Fall back to whatever the environment offers. `_ = target` keeps the
        // signature uniform; the fallback path is platform-agnostic.
        let _ = target;
        let cxx = env::var_os("CXX")
            .map(PathBuf::from)
            .or_else(|| which("clang++"))
            .or_else(|| which("c++"))
            .expect("no C++ compiler found: set CXX or HARFBUZZ_SYS_LLVM_TOOLCHAIN");
        let ar = env::var_os("AR")
            .map(PathBuf::from)
            .or_else(|| which("llvm-ar"))
            .or_else(|| which("ar"))
            .expect("no archiver found: set AR or HARFBUZZ_SYS_LLVM_TOOLCHAIN");

        println!(
            "cargo::warning=harfbuzz-sys: pinned LLVM toolchain not found, falling back to {}. \
             Cross-language LTO will be disabled unless its LLVM matches rustc's.",
            cxx.display()
        );

        Self::probe_versions(cxx, ar)
    }

    fn probe_versions(cxx: PathBuf, ar: PathBuf) -> Self {
        Self {
            clang_llvm_major: clang_major(&cxx),
            rustc_llvm_major: rustc_llvm_major(),
            cxx,
            ar,
        }
    }
}

/// Parse the major version out of `clang --version`, whose first line reads
/// e.g. `clang version 22.1.2 (https://github.com/llvm/llvm-project ...)`.
fn clang_major(cxx: &Path) -> Option<u32> {
    let out = Command::new(cxx).arg("--version").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let after = text.split("clang version ").nth(1)?;
    after.split(['.', '-', ' ']).next()?.parse().ok()
}

/// Ask the `rustc` Cargo is driving which LLVM it embeds. `rustc -vV` prints a
/// `LLVM version: 22.1.2` line.
fn rustc_llvm_major() -> Option<u32> {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let out = Command::new(rustc).arg("-vV").output().ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().find(|l| l.starts_with("LLVM version:"))?;
    line.trim_start_matches("LLVM version:")
        .trim()
        .split('.')
        .next()?
        .parse()
        .ok()
}

/// Minimal `which`, so the build script keeps its zero-dependency promise.
fn which(program: &str) -> Option<PathBuf> {
    env::var_os("PATH").and_then(|paths| {
        env::split_paths(&paths)
            .map(|dir| dir.join(program))
            .find(|candidate| candidate.is_file())
    })
}

// ---------------------------------------------------------------------------
// LTO
// ---------------------------------------------------------------------------

enum LtoMode {
    Thin,
    Full,
    Off,
}

impl LtoMode {
    /// Decide whether it is *safe* to emit bitcode.
    ///
    /// Bitcode is only readable by an LLVM at least as new as the one that
    /// produced it, and the linker — not this build script — is what has to
    /// read it. Matching clang's major version to `rustc`'s is the closest
    /// proxy we have: when they agree, the toolchain that can link the result
    /// is the one already in use.
    fn resolve(toolchain: &Toolchain) -> Self {
        let requested = env::var("HARFBUZZ_SYS_LTO").unwrap_or_else(|_| "thin".to_string());

        let requested = match requested.as_str() {
            "off" | "no" | "false" => return Self::Off,
            "full" | "fat" => Self::Full,
            "thin" | "" => Self::Thin,
            other => panic!("HARFBUZZ_SYS_LTO must be thin, full, or off (got {other:?})"),
        };

        match (toolchain.clang_llvm_major, toolchain.rustc_llvm_major) {
            (Some(clang), Some(rustc)) if clang == rustc => requested,
            (Some(clang), Some(rustc)) => {
                println!(
                    "cargo::warning=harfbuzz-sys: disabling LTO because clang's LLVM ({clang}) \
                     differs from rustc's ({rustc}); bitcode produced by one would not be \
                     readable by the other's linker."
                );
                Self::Off
            }
            _ => {
                println!(
                    "cargo::warning=harfbuzz-sys: disabling LTO because the compiler's LLVM \
                     version could not be determined."
                );
                Self::Off
            }
        }
    }

    fn flag(&self) -> Option<&'static str> {
        match self {
            Self::Thin => Some("-flto=thin"),
            Self::Full => Some("-flto"),
            Self::Off => None,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Self::Thin => "thin",
            Self::Full => "full",
            Self::Off => "off",
        }
    }
}

// ---------------------------------------------------------------------------
// Features
// ---------------------------------------------------------------------------

/// The enabled Cargo features, resolved against what the target can actually
/// provide.
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
    mini: bool,
    lean: bool,
    tiny: bool,
}

impl Features {
    fn from_env(target: &Target) -> Self {
        let on = |name: &str| env::var_os(format!("CARGO_FEATURE_{name}")).is_some();

        // CoreText only exists on Apple platforms. Silently ignoring the
        // feature elsewhere is friendlier than failing, because Cargo features
        // are additive: an unrelated crate in the graph may have turned it on.
        let coretext = on("CORETEXT") && target.is_apple;
        if on("CORETEXT") && !coretext {
            println!(
                "cargo::warning=harfbuzz-sys: ignoring the `coretext` feature on a non-Apple \
                 target ({})",
                target.triple
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
    /// Run every probe, in parallel. Each one is a sub-second compile, but
    /// there are fifteen of them and they are completely independent.
    fn run_all(toolchain: &Toolchain, target: &Target) -> Vec<Self> {
        let scratch = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR")).join("probes");
        fs::create_dir_all(&scratch).expect("create probe directory");

        let jobs: Vec<(&'static str, String)> = HEADER_PROBES
            .iter()
            .map(|(macro_name, header)| (*macro_name, format!("#include <{header}>\nint main(void){{return 0;}}\n")))
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
                        available: compiles(toolchain, target, scratch, macro_name, source),
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
fn compiles(
    toolchain: &Toolchain,
    target: &Target,
    scratch: &Path,
    name: &str,
    source: &str,
) -> bool {
    let file = scratch.join(format!("{name}.c"));
    let object = scratch.join(format!("{name}.o"));
    if fs::write(&file, source).is_err() {
        return false;
    }

    let mut cmd = Command::new(&toolchain.cxx);
    cmd.arg("-x").arg("c").arg("-c").arg("-w");
    cmd.arg(format!("--target={}", target.clang_triple()));
    if let Some(sdk) = target.sdk_path() {
        cmd.arg("-isysroot").arg(sdk);
    }
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
    if cfg!(not(windows)) {
        out.push_str("\n#define HAVE_PTHREAD 1\n");
    }

    out.push('\n');

    // Libraries whose presence the amalgamation does *not* infer from
    // `HB_HAS_*`. Note `HAVE_GRAPHITE2`: `harfbuzz-world.cc` translates
    // `HB_HAS_GRAPHITE` into `HAVE_GRAPHITE`, but every consumer in the
    // sources tests `HAVE_GRAPHITE2`, so setting it here is what actually
    // enables the shaper.
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
// Compile and archive
// ---------------------------------------------------------------------------

/// Compile the amalgamation into a single object file.
fn compile(
    toolchain: &Toolchain,
    target: &Target,
    features: &Features,
    lto: &LtoMode,
    hb_src: &Path,
    out_dir: &Path,
) -> PathBuf {
    let object = out_dir.join("harfbuzz.o");
    let mut cmd = Command::new(&toolchain.cxx);

    cmd.arg("-c");
    cmd.arg(format!("--target={}", target.clang_triple()));
    if let Some(sdk) = target.sdk_path() {
        cmd.arg("-isysroot").arg(sdk);
    }

    // Language settings, matched to upstream's meson defaults. ICU 75 and
    // later require C++17 in its public headers, which meson also special-cases.
    let needs_cxx17 = features.icu.as_ref().and_then(PkgConfig::major).is_some_and(|v| v >= 75);
    cmd.arg(if needs_cxx17 { "-std=c++17" } else { "-std=c++11" });
    cmd.args(["-fno-exceptions", "-fno-threadsafe-statics"]);
    // Upstream builds without RTTI, but ICU's headers use it. Keeping RTTI on
    // only when ICU is linked mirrors the note in upstream's meson.build.
    if features.icu.is_none() {
        cmd.arg("-fno-rtti");
    }
    cmd.arg("-fvisibility-inlines-hidden");

    // Size first, as requested. `-ffunction-sections`/`-fdata-sections` put
    // every function and object in its own section so the linker can drop the
    // ones nothing references — see `emit_link_directives` for the other half.
    cmd.args(["-Oz", "-ffunction-sections", "-fdata-sections"]);

    // Debug info is kept on purpose: it is what makes the C++ side legible to
    // lldb and to sampling profilers. It costs archive size, not runtime.
    if env::var_os("HARFBUZZ_SYS_NO_DEBUG_INFO").is_none() {
        cmd.arg("-g");
        // Frame pointers make stacks walkable by instrumentation that does not
        // parse DWARF. On arm64 the ABI mandates them anyway; stating it keeps
        // other targets consistent.
        cmd.arg("-fno-omit-frame-pointer");
    }

    if let Some(flag) = lto.flag() {
        cmd.arg(flag);
    }

    cmd.arg("-DHAVE_CONFIG_H");
    cmd.arg(format!("-I{}", out_dir.display()));
    cmd.arg(format!("-I{}", hb_src.display()));

    // The `HB_HAS_*` switches the amalgamation dispatches on. These have to be
    // on the command line: `harfbuzz-world.cc` tests them before it includes
    // anything, so `config.h` would be too late.
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
            cmd.arg(format!("-D{macro_name}=1"));
        }
    }

    // Size profiles from upstream's CONFIG.md.
    for (enabled, macro_name) in [
        (features.tiny, "HB_TINY"),
        (features.lean, "HB_LEAN"),
        (features.mini, "HB_MINI"),
    ] {
        if enabled {
            cmd.arg(format!("-D{macro_name}=1"));
        }
    }

    for pkg in features.pkg_configs() {
        cmd.args(&pkg.cflags);
    }

    cmd.arg(hb_src.join("harfbuzz-world.cc"));
    cmd.arg("-o").arg(&object);

    run(cmd, "compiling HarfBuzz");
    object
}

/// Wrap the object in a static archive.
///
/// `llvm-ar` rather than the system `ar` is not optional when LTO is on: a
/// bitcode member has no Mach-O symbol table, and only an LLVM-aware archiver
/// can index it. A plain `ar` produces an archive the linker treats as empty.
fn archive(toolchain: &Toolchain, object: &Path, out_dir: &Path) -> PathBuf {
    let archive = out_dir.join("libharfbuzz.a");
    // `ar` appends to an existing archive, which would accumulate stale members
    // across rebuilds.
    let _ = fs::remove_file(&archive);

    let mut cmd = Command::new(&toolchain.ar);
    cmd.arg("crs").arg(&archive).arg(object);
    run(cmd, "archiving HarfBuzz");

    archive
}

fn run(mut cmd: Command, what: &str) {
    let rendered = format!("{cmd:?}");
    match cmd.status() {
        Ok(status) if status.success() => {}
        Ok(status) => panic!("{what} failed ({status})\ncommand: {rendered}"),
        Err(err) => panic!("{what} could not start: {err}\ncommand: {rendered}"),
    }
}

// ---------------------------------------------------------------------------
// Cargo output
// ---------------------------------------------------------------------------

fn emit_link_directives(target: &Target, features: &Features, archive: &Path, out_dir: &Path) {
    let _ = archive;
    println!("cargo::rustc-link-search=native={}", out_dir.display());
    println!("cargo::rustc-link-lib=static=harfbuzz");

    // HarfBuzz is C++, so the standard library has to come along. libc++ is the
    // only option on Apple platforms; elsewhere clang defaults to libstdc++ on
    // GNU hosts.
    if target.is_apple {
        println!("cargo::rustc-link-lib=dylib=c++");
    } else if target.os == "linux" || target.os == "android" {
        println!("cargo::rustc-link-lib=dylib=stdc++");
    }

    if features.coretext {
        // ApplicationServices umbrella-links CoreText, CoreGraphics, and
        // CoreFoundation on macOS. iOS-family targets expose them separately.
        if target.os == "macos" {
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
                "cargo::warning=harfbuzz-sys: `pkg-config --libs {}` returned nothing; the \
                 link step will probably report undefined symbols.",
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

    // Complete the size story started by `-ffunction-sections`: tell the linker
    // to drop sections nothing reaches. Mach-O spells this `-dead_strip`; ELF
    // linkers spell it `--gc-sections`.
    //
    // These apply to this crate's own binaries and tests. A downstream binary
    // needs the same flag in its own configuration to benefit.
    if target.is_apple {
        println!("cargo::rustc-link-arg=-Wl,-dead_strip");
    } else {
        println!("cargo::rustc-link-arg=-Wl,--gc-sections");
    }
}

/// Point this crate's own binaries and tests at a linker that can read the
/// bitcode we just produced.
///
/// With LTO on, `libharfbuzz.a` holds LLVM bitcode. `rustc` links through the
/// platform C compiler, which on macOS drives Apple's `ld64` and its bundled,
/// older libLTO — that combination cannot parse the archive at all. Selecting
/// the `lld` that shipped alongside our `clang++` keeps the producer and the
/// consumer of the bitcode on the same LLVM.
///
/// Only this package's own artifacts are covered. Cargo does not propagate link
/// arguments to dependents, so a downstream binary has to make the same choice
/// in its own `.cargo/config.toml`; the repository root has a worked example.
fn emit_lto_linker_directives(toolchain: &Toolchain, lto: &LtoMode) {
    if matches!(lto, LtoMode::Off) {
        return;
    }

    let Some(root) = toolchain.cxx.parent().and_then(Path::parent) else {
        return;
    };

    // Apple's linker loads LTO support from a dylib, and `-lto_library` is the
    // supported way to choose which one. Swapping in the matching libLTO is
    // less invasive than replacing `ld64` outright, and keeps every Apple
    // linker feature working.
    let lto_library = root.join("lib").join("libLTO.dylib");
    if !lto_library.is_file() {
        println!(
            "cargo::warning=harfbuzz-sys: LTO is on but {} is missing. Linking may fail with \
             \"could not parse bitcode object file\"; set HARFBUZZ_SYS_LTO=off to opt out.",
            lto_library.display()
        );
        return;
    }

    println!(
        "cargo::rustc-link-arg=-Wl,-lto_library,{}",
        lto_library.display()
    );
}

/// Publish what we did to dependent build scripts.
///
/// Because this package declares `links = "harfbuzz"`, Cargo forwards each of
/// these to dependents as `DEP_HARFBUZZ_<KEY>`.
fn emit_metadata(hb_root: &Path, lto: &LtoMode, toolchain: &Toolchain) {
    println!("cargo::metadata=include={}", hb_root.join("src").display());
    println!("cargo::metadata=lto={}", lto.as_str());
    println!("cargo::metadata=cxx={}", toolchain.cxx.display());
    if let Some(major) = toolchain.clang_llvm_major {
        println!("cargo::metadata=llvm_major={major}");
    }
}
