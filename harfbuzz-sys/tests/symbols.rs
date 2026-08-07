//! Checks that every function this crate declares actually exists in the
//! HarfBuzz archive it links against.
//!
//! An `extern "C"` declaration is an unchecked promise. Rust only notices a
//! wrong or missing name when something calls it, so a typo in a binding can
//! sit undetected until a downstream user hits a link error — with no clue that
//! the binding, rather than their code, is at fault. This test closes that gap
//! by comparing the crate's declarations against the archive's symbol table.
//!
//! It cannot check *signatures*; nothing can, from outside the compiler. What
//! it does catch is misspelled names, functions that upstream renamed or
//! removed, and bindings enabled by a feature that does not actually compile
//! the code behind them.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Modules compiled unconditionally.
const CORE_MODULES: &[&str] = &[
    "common.rs",
    "script.rs",
    "version.rs",
    "blob.rs",
    "buffer.rs",
    "draw.rs",
    "face.rs",
    "font.rs",
    "map.rs",
    "paint.rs",
    "set.rs",
    "shape.rs",
    "shape_plan.rs",
    "style.rs",
    "unicode.rs",
    "aat_layout.rs",
    "ot_color.rs",
    "ot_deprecated.rs",
    "ot_fetch.rs",
    "ot_font.rs",
    "ot_layout.rs",
    "ot_math.rs",
    "ot_meta.rs",
    "ot_metrics.rs",
    "ot_name.rs",
    "ot_shape.rs",
    "ot_var.rs",
    "deprecated.rs",
];

/// Modules compiled only when their feature is on. Reading this list through
/// `cfg!` keeps it honest: the test checks exactly the surface that was built.
fn feature_modules() -> Vec<&'static str> {
    let mut modules = Vec::new();

    for (enabled, file) in [
        (cfg!(feature = "subset"), "subset.rs"),
        (cfg!(feature = "raster"), "raster.rs"),
        (cfg!(feature = "vector"), "vector.rs"),
        (cfg!(feature = "gpu"), "gpu.rs"),
        (cfg!(feature = "coretext"), "coretext.rs"),
        (cfg!(feature = "freetype"), "ft.rs"),
        (cfg!(feature = "graphite2"), "graphite2.rs"),
        (cfg!(feature = "icu"), "icu.rs"),
        (cfg!(feature = "glib"), "glib.rs"),
    ] {
        if enabled {
            modules.push(file);
        }
    }

    modules
}

/// A function this crate declares, and where it was declared.
#[derive(Debug)]
struct Declaration {
    name: String,
    module: String,
    /// True when the declaration carries a `#[cfg(...)]`. Whether such a
    /// function is compiled depends on flags this test cannot resolve from
    /// source text, so the forward check skips them — but the coverage report
    /// still counts them as declared, or every experimental binding would look
    /// like a gap.
    cfg_gated: bool,
}

/// Scan one module for the names inside its `unsafe extern "C"` blocks.
///
/// Declarations carrying a `#[cfg(...)]` attribute are skipped: whether they
/// are compiled depends on flags this test cannot resolve from source text, so
/// checking them would produce false failures.
fn declarations_in(path: &Path) -> Vec<Declaration> {
    let module = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("?")
        .to_string();
    let source = fs::read_to_string(path).unwrap_or_default();

    let mut declarations = Vec::new();
    let mut in_extern_block = false;
    let mut depth = 0i32;
    let mut cfg_gated = false;

    for line in source.lines() {
        let trimmed = line.trim();

        if !in_extern_block {
            if trimmed.starts_with("unsafe extern \"C\"") {
                in_extern_block = true;
                depth = 0;
            }
            continue;
        }

        depth += trimmed.matches('{').count() as i32;
        depth -= trimmed.matches('}').count() as i32;
        if depth <= 0 && trimmed.contains('}') && !trimmed.contains("pub fn") {
            in_extern_block = false;
            continue;
        }

        if trimmed.starts_with("#[cfg(") {
            cfg_gated = true;
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("pub fn ") {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                declarations.push(Declaration {
                    name,
                    module: module.clone(),
                    cfg_gated,
                });
            }
            cfg_gated = false;
        } else if !trimmed.is_empty() && !trimmed.starts_with("///") && !trimmed.starts_with("//") {
            // Any other code line ends the reach of a pending attribute.
            if !trimmed.starts_with('#') {
                cfg_gated = false;
            }
        }
    }

    declarations
}

/// The externally visible symbols defined by the archive.
///
/// Mach-O prefixes C symbols with an underscore, so `hb_shape` appears as
/// `_hb_shape`; both spellings go into the set so the comparison works on
/// either platform.
fn defined_symbols(nm: &str, archive: &str) -> BTreeSet<String> {
    let output = Command::new(nm)
        .args(["--defined-only", "--extern-only", archive])
        .output()
        .unwrap_or_else(|e| panic!("could not run {nm} on {archive}: {e}"));

    assert!(
        output.status.success(),
        "{nm} failed on {archive}: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| line.split_whitespace().last())
        .flat_map(|symbol| {
            let stripped = symbol.strip_prefix('_').unwrap_or(symbol).to_string();
            [symbol.to_string(), stripped]
        })
        .collect()
}

#[test]
fn every_declared_function_exists_in_the_archive() {
    let archive = env!("HARFBUZZ_SYS_ARCHIVE");
    let nm = env!("HARFBUZZ_SYS_NM");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");

    let defined = defined_symbols(nm, archive);
    assert!(
        !defined.is_empty(),
        "{nm} reported no symbols in {archive}; the archive is probably empty or unreadable"
    );

    let declarations: Vec<Declaration> = CORE_MODULES
        .iter()
        .chain(feature_modules().iter())
        .map(|file| src.join(file))
        .filter(|path| path.is_file())
        .flat_map(|path| declarations_in(&path))
        .collect();

    assert!(
        declarations.len() > 500,
        "only found {} declarations, which suggests the scanner is broken rather than the \
         bindings being that small",
        declarations.len()
    );

    let missing: Vec<&Declaration> = declarations
        .iter()
        .filter(|d| !d.cfg_gated)
        .filter(|d| !defined.contains(&d.name))
        .collect();

    assert!(
        missing.is_empty(),
        "{} of {} declared functions are absent from {archive}:\n{}",
        missing.len(),
        declarations.len(),
        missing
            .iter()
            .map(|d| format!("  {} (declared in src/{})", d.name, d.module))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The mirror of the test above: report public HarfBuzz functions the archive
/// defines that this crate never declared.
///
/// This is a coverage report rather than a hard failure. Some exported symbols
/// belong to sub-libraries whose feature is off, and some are internal helpers
/// that upstream exports without documenting in a public header.
#[test]
fn report_undeclared_archive_symbols() {
    let archive = env!("HARFBUZZ_SYS_ARCHIVE");
    let nm = env!("HARFBUZZ_SYS_NM");
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");

    let declared: BTreeSet<String> = CORE_MODULES
        .iter()
        .chain(feature_modules().iter())
        .map(|file| src.join(file))
        .filter(|path| path.is_file())
        .flat_map(|path| declarations_in(&path))
        .map(|d| d.name)
        .collect();

    let undeclared: Vec<String> = defined_symbols(nm, archive)
        .into_iter()
        .filter(|s| s.starts_with("hb_"))
        .filter(|s| !declared.contains(s))
        .collect();

    println!(
        "{} exported hb_* symbols are not declared by this crate:",
        undeclared.len()
    );
    for symbol in &undeclared {
        println!("  {symbol}");
    }
}
