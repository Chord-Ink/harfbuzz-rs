use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;

use pkg_config::Library;

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let hb_src = manifest_dir.join("harfbuzz").join("src");

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

    let build = configure(&hb_src, &out_dir, &features);

    // Probe the target the way upstream's meson build does, rather than
    // hard-coding a table of what each platform is assumed to provide.
    let probes = Probe::run_all(&build.get_compiler(), &out_dir);
    fs::write(out_dir.join("config.h"), render_config_h(&probes, &features))
        .expect("write config.h");

    // `compile` also emits the link-lib and link-search directives for us;
    // `pkg-config` emitted its own during `probe`.
    build.compile("harfbuzz");

    emit_framework_directives(&features);
    emit_test_env(&out_dir);
    println!("cargo::metadata=include={}", hb_src.display());
}

/// Tell Cargo the narrow set of inputs that can change what we produce.
///
/// Pointing `rerun-if-changed` at the source *directory* is deliberate: Cargo
/// walks it recursively, so moving the submodule to a different tag invalidates
/// the build. `cc` and `pkg-config` register their own environment variables.
fn rerun_directives(manifest_dir: &Path, hb_src: &Path) {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed={}", hb_src.display());
    println!(
        "cargo::rerun-if-changed={}",
        manifest_dir.join("Cargo.toml").display()
    );
}

// ---------------------------------------------------------------------------
// The compile
// ---------------------------------------------------------------------------

fn configure(hb_src: &Path, out_dir: &Path, features: &Features) -> cc::Build {
    let mut build = cc::Build::new();

    build
        .cpp(true)
        .file(hb_src.join("harfbuzz-world.cc"))
        .include(hb_src)
        // Where `config.h` lands.
        .include(out_dir)
        .define("HAVE_CONFIG_H", None);

    // ICU 75 and later need C++17 in their public headers; upstream's meson
    // build special-cases this the same way.
    let needs_cxx17 = features.icu.as_ref().is_some_and(|lib| major(lib) >= 75);
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

    // Size first. `-ffunction-sections`/`-fdata-sections` give the linker
    // symbol-granularity dead stripping; rustc already passes `-Wl,-dead_strip`
    // (Mach-O) and `--gc-sections` (ELF), so there is nothing to add on the
    // link side.
    build.opt_level_str("z");
    build.flag_if_supported("-ffunction-sections");
    build.flag_if_supported("-fdata-sections");

    // The `debug` feature owns this, rather than Cargo's profile, so a release
    // build can still carry a debuggable HarfBuzz.
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

    for lib in features.libraries() {
        build.includes(&lib.include_paths);
    }

    build
}

// ---------------------------------------------------------------------------
// Features
// ---------------------------------------------------------------------------

/// The enabled Cargo features, with every optional system library already
/// located.
struct Features {
    subset: bool,
    raster: bool,
    vector: bool,
    gpu: bool,
    coretext: bool,
    freetype: Option<Library>,
    graphite2: Option<Library>,
    icu: Option<Library>,
    glib: Option<Library>,
    png: Option<Library>,
    zlib: Option<Library>,
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
            freetype: probe(on("FREETYPE"), "freetype2"),
            graphite2: probe(on("GRAPHITE2"), "graphite2"),
            icu: probe(on("ICU"), "icu-uc"),
            glib: probe(on("GLIB"), "glib-2.0"),
            png: probe(on("PNG"), "libpng"),
            zlib: probe(on("ZLIB"), "zlib"),
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

    fn libraries(&self) -> impl Iterator<Item = &Library> {
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

/// Locate `name` when `enabled`.
///
/// `probe` emits the link directives itself, so the caller only needs the
/// include paths. Failure is fatal on purpose: a feature the user explicitly
/// asked for should never be silently dropped.
fn probe(enabled: bool, name: &str) -> Option<Library> {
    if !enabled {
        return None;
    }

    match pkg_config::Config::new().probe(name) {
        Ok(library) => Some(library),
        Err(err) => panic!(
            "the requested feature needs the `{name}` library, which pkg-config could not \
             find.\nInstall it, or point PKG_CONFIG_PATH at it.\n\n{err}"
        ),
    }
}

/// The major version of a located library, or zero when it is unparsable.
fn major(library: &Library) -> u32 {
    library
        .version
        .split('.')
        .next()
        .and_then(|major| major.parse().ok())
        .unwrap_or(0)
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
                .map(|handle| handle.join().expect("probe thread panicked"))
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

    cmd.output().map(|out| out.status.success()).unwrap_or(false)
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

/// Apple system frameworks, which have no `pkg-config` entry.
fn emit_framework_directives(features: &Features) {
    if !features.coretext {
        return;
    }

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

/// Hand `tests/symbols.rs` what it needs to check every declared binding
/// against the symbols the archive actually defines.
fn emit_test_env(out_dir: &Path) {
    println!(
        "cargo::rustc-env=HARFBUZZ_SYS_ARCHIVE={}",
        out_dir.join("libharfbuzz.a").display()
    );
    println!(
        "cargo::rustc-env=HARFBUZZ_SYS_NM={}",
        env::var("NM").unwrap_or_else(|_| "nm".to_string())
    );
}
