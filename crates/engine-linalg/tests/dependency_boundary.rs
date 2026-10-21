//! Architectural boundary without code analysis dependencies.
use std::{
    fs,
    path::{Path, PathBuf},
};

fn provider(name: &str) -> bool {
    ["faer", "nalgebra", "sprs"].iter().any(|root| {
        name == *root
            || name.starts_with(&format!("{root}-"))
            || name.starts_with(&format!("{root}_"))
    })
}
fn words(input: &str) -> impl Iterator<Item = &str> {
    input
        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_' && c != '-')
        .filter(|s| !s.is_empty())
}
fn manifest_violation(input: &str) -> bool {
    let mut dependencies = false;
    for line in input.lines() {
        let line = line.trim();
        if line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            dependencies = words(line).any(|w| {
                matches!(
                    w,
                    "dependencies" | "dev-dependencies" | "build-dependencies"
                )
            });
            if dependencies && words(line).any(provider) {
                return true;
            }
        } else if dependencies && words(line.split('#').next().unwrap()).any(provider) {
            // Includes alias `numeric={package="faer"}` and table dependencies.
            return true;
        }
    }
    false
}
// Conservative inspection of Rust tokens. Also detects macros and `extern
// crate`; removes comments, strings, and characters to avoid documentation
// false positives. Does not attempt to be a complete language parser.
fn rust_violation(input: &str) -> bool {
    let b = input.as_bytes();
    let mut i = 0;
    while i < b.len() {
        if b[i..].starts_with(b"//") {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
        } else if b[i..].starts_with(b"/*") {
            i += 2;
            let mut depth = 1;
            while i < b.len() && depth > 0 {
                if b[i..].starts_with(b"/*") {
                    depth += 1;
                    i += 2;
                } else if b[i..].starts_with(b"*/") {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
        } else if b[i] == b'r' && (b.get(i + 1) == Some(&b'#') || b.get(i + 1) == Some(&b'"')) {
            let start = i;
            i += 1;
            while b.get(i) == Some(&b'#') {
                i += 1;
            }
            let hashes = i - start - 1;
            if b.get(i) == Some(&b'"') {
                i += 1;
                while i < b.len() {
                    if b[i] == b'"'
                        && b.get(i + 1..i + 1 + hashes)
                            .is_some_and(|s| s.iter().all(|&v| v == b'#'))
                    {
                        i += 1 + hashes;
                        break;
                    }
                    i += 1;
                }
            }
        } else if b[i] == b'"' {
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    i += 2;
                } else if b[i] == b'"' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
        } else if b[i].is_ascii_alphabetic() || b[i] == b'_' {
            let start = i;
            i += 1;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            if provider(&input[start..i]) {
                return true;
            }
        } else {
            i += 1;
        }
    }
    false
}
fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(&path, out);
        } else if path.file_name().is_some_and(|s| s == "Cargo.toml")
            || path.extension().is_some_and(|s| s == "rs")
        {
            out.push(path);
        }
    }
}
#[test]
fn only_linalg_can_import_numeric_providers() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let mut paths = vec![root.join("Cargo.toml")];
    for entry in fs::read_dir(root.join("crates")).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() && path.file_name().unwrap() != "engine-linalg" {
            files(&path, &mut paths);
        }
    }
    // All crates under crates/ are traversed, including new members not yet
    // in the manifest; the workspace layout resides in that directory.
    for path in paths {
        let content = fs::read_to_string(&path).unwrap();
        let violation = if path.extension().is_some_and(|s| s == "rs") {
            rust_violation(&content)
        } else {
            manifest_violation(&content)
        };
        assert!(
            !violation,
            "Proveedor numérico fuera de engine-linalg: {}",
            path.display()
        );
    }
}
#[test]
fn scanner_detects_aliases_and_imports() {
    for manifest in [
        "[dependencies]\nfaer = '0.24.4'",
        "[dependencies]\nnumeric = { package = 'nalgebra', version = '*' }",
        "[target.'cfg(windows)'.dependencies.sprs]\nversion='*'",
        "[workspace.dependencies]\nfaer = '*'",
        "[dependencies.numeric]\npackage = 'faer'",
    ] {
        assert!(manifest_violation(manifest));
    }
    for rust in [
        "use faer::Mat;",
        "extern crate nalgebra as numeric;",
        "::sprs::CsMat;",
        "use faer_traits::ComplexField;",
        "macro_rules! m { () => { faer::Mat }; }",
    ] {
        assert!(rust_violation(rust));
    }
    for rust in [
        "// use faer::Mat;\nfn f() {}",
        "/* /* nalgebra */ sprs */",
        "let text = \"faer::Mat\";",
        "let text = r###\"nalgebra::DMatrix\"###;",
        "let text = \"escaped \\\" faer\";",
    ] {
        assert!(!rust_violation(rust));
    }
}
