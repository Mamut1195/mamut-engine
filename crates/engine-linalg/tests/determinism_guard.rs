//! Guardias conservadoras de determinismo y política escalar para crates/*.
use std::{
    fs,
    path::{Path, PathBuf},
};

fn forbidden(name: &str) -> bool {
    matches!(name, "HashMap" | "HashSet" | "thread" | "rayon") || name.starts_with("rayon_")
}
// Inspección conservadora de tokens de Rust. También detecta macros y `extern
// crate`; elimina comentarios y cadenas para evitar falsos positivos
// de documentación. No pretende ser un parser completo del lenguaje.
fn rust_tokens(input: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
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
        } else if b[i] == b'\'' && (b.get(i + 2) == Some(&b'\'') || b.get(i + 1) == Some(&b'\\')) {
            tokens.push("<literal>");
            i += 1;
            while i < b.len() {
                if b[i] == b'\\' {
                    i += 2;
                } else if b[i] == b'\'' {
                    i += 1;
                    break;
                } else {
                    i += 1;
                }
            }
        } else if b[i] == b'r' && (b.get(i + 1) == Some(&b'#') || b.get(i + 1) == Some(&b'"')) {
            tokens.push("<literal>");
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
            tokens.push("<literal>");
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
            tokens.push(&input[start..i]);
        } else {
            if b[i..].starts_with(b"::") {
                tokens.push("::");
                i += 2;
            } else {
                if b[i].is_ascii() && !b[i].is_ascii_whitespace() {
                    tokens.push(&input[i..i + 1]);
                }
                i += 1;
            }
        }
    }
    tokens
}
fn rust_violation(input: &str) -> bool {
    rust_tokens(input).into_iter().any(forbidden)
}

fn scalar_violation(input: &str) -> bool {
    let tokens = rust_tokens(input);
    for (i, &token) in tokens.iter().enumerate() {
        // La dependencia tampoco puede invocarse al margen del módulo central.
        if token == "libm" {
            return true;
        }
        if matches!(
            token,
            "sin"
                | "cos"
                | "sin_cos"
                | "tan"
                | "asin"
                | "acos"
                | "atan"
                | "atan2"
                | "sinh"
                | "cosh"
                | "tanh"
                | "asinh"
                | "acosh"
                | "atanh"
                | "sqrt"
                | "exp"
                | "exp2"
                | "exp_m1"
                | "ln"
                | "ln_1p"
                | "log"
                | "log2"
                | "log10"
                | "powf"
                | "powi"
                | "hypot"
                | "cbrt"
        ) && i > 0
            && (tokens[i - 1] == "."
                || (tokens[i - 1] == "::" && (i < 2 || tokens[i - 2] != "math")))
        {
            return true;
        }
    }
    false
}
fn files(dir: &Path, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            files(&path, out)?;
        } else if path.extension().is_some_and(|s| s == "rs") {
            out.push(path);
        }
    }
    Ok(())
}

#[test]
fn numerical_libraries_have_no_hash_collections_or_threads() -> std::io::Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut paths = Vec::new();
    for entry in fs::read_dir(root.join("crates"))? {
        let src = entry?.path().join("src");
        if src.is_dir() {
            files(&src, &mut paths)?;
        }
    }
    paths.sort();
    assert!(!paths.is_empty(), "La guardia no encontró fuentes");
    // Incluye engine-api: hoy no necesita una excepción de serialización.
    for path in paths {
        assert!(
            !rust_violation(&fs::read_to_string(&path)?),
            "Colección hash o concurrencia prohibida en {}",
            path.display()
        );
    }
    Ok(())
}

#[test]
fn guard_detects_aliases_spacing_and_macros_without_doc_false_positives() {
    for source in [
        "use std::collections::HashMap as Lookup;",
        "type Set = ::std::collections::HashSet<u64>;",
        "use std :: thread as worker;",
        "use std::{thread};",
        "macro_rules! m { () => { rayon::join(f, g) }; }",
        "extern crate rayon_core as workers;",
    ] {
        assert!(rust_violation(source), "{source}");
    }
    for source in [
        "// HashMap, std::thread, rayon\nuse std::collections::BTreeMap;",
        "/* HashSet /* rayon */ thread */",
        "let text = \"HashMap std::thread rayon\";",
        "let text = r###\"HashSet rayon\"###;",
        "let text = \"escaped \\\" rayon\";",
    ] {
        assert!(!rust_violation(source), "{source}");
    }
}

#[test]
fn engine_crates_use_only_the_central_scalar_module() -> std::io::Result<()> {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut paths = Vec::new();
    files(&crates, &mut paths)?;
    paths.sort();
    assert!(!paths.is_empty(), "La guardia no encontró fuentes");
    for path in paths {
        if path == crates.join("engine-core/src/math.rs") {
            continue;
        }
        assert!(
            !scalar_violation(&fs::read_to_string(&path)?),
            "Función escalar fuera de engine_core::math en {}",
            path.display()
        );
    }
    Ok(())
}

#[test]
fn scalar_guard_detects_methods_ufcs_aliases_and_libm_bypasses() {
    for name in [
        "sin", "cos", "sin_cos", "tan", "asin", "acos", "atan", "atan2", "sinh", "cosh", "tanh",
        "asinh", "acosh", "atanh", "sqrt", "exp", "exp2", "exp_m1", "ln", "ln_1p", "log", "log2",
        "log10", "powf", "powi", "hypot", "cbrt",
    ] {
        for source in [
            format!("x . /* espacio */ {name}(y)"),
            format!("f64 :: {name}(x)"),
            format!("use std::primitive::f64 as Real; let f = Real::{name};"),
            format!("macro_rules! m {{ () => {{ x.{name}() }}; }}"),
        ] {
            assert!(scalar_violation(&source), "{source}");
        }
        assert!(!scalar_violation(&format!("engine_core::math::{name}(x)")));
    }
    for source in ["use libm as alternate;", "libm::hypot(x, y)"] {
        assert!(scalar_violation(source), "{source}");
    }
    for source in [
        "// x.sin()\nmath::cos(x)",
        "/* f64::powi */ math::powi(x, 2)",
        "let text = r###\"x.ln() libm\"###;",
        "let text = \"x.exp()\";",
        "x.abs() + x.max(y)",
        "use engine_core::math::{sin, cos}; sin(x) + cos(x)",
    ] {
        assert!(!scalar_violation(source), "{source}");
    }
}
