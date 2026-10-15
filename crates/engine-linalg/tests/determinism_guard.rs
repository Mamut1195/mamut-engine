//! Guardias conservadoras de determinismo y política escalar para crates/*.
use std::{
    fs,
    path::{Path, PathBuf},
};

fn forbidden(name: &str) -> bool {
    matches!(name, "HashMap" | "HashSet" | "thread" | "rayon") || name.starts_with("rayon_")
}
// Inspección conservadora de tokens de Rust. También detecta macros y `extern
// crate`; elimina comentarios y conserva cada literal como un token opaco
// para evitar falsos positivos de documentación. No es un parser completo.
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
            let start = i;
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
            tokens.push(&input[start..i.min(b.len())]);
        } else if b[i..].starts_with(b"r#")
            && b.get(i + 2)
                .is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_')
        {
            // r#sin es un identificador, no el comienzo de una cadena raw.
            i += 2;
            let start = i;
            while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'_') {
                i += 1;
            }
            tokens.push(&input[start..i]);
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
            tokens.push(&input[start..i.min(b.len())]);
        } else if b[i] == b'"' {
            let start = i;
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
            tokens.push(&input[start..i.min(b.len())]);
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

fn scalar_name(name: &str) -> bool {
    matches!(
        name,
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
    )
}

// Solo la importación sin alias del módulo central autoriza math::función.
// El nombre queda reservado: no se admite otro import ni una redefinición.
fn central_math_import(tokens: &[&str], path: &Path) -> Result<bool, ()> {
    let mut imported = false;
    for (i, &token) in tokens.iter().enumerate() {
        if token == "use" {
            let end = tokens[i..].iter().position(|&t| t == ";").ok_or(())? + i;
            let import = &tokens[i + 1..end];
            let import = import.strip_prefix(&["::"]).unwrap_or(import);
            if import.contains(&"math") {
                if import != ["engine_core", "::", "math"] {
                    return Err(());
                }
                imported = true;
            }
            if import.contains(&"as") && (import.contains(&"f64") || import.contains(&"f32")) {
                return Err(());
            }
            // Las funciones se usan con la ruta del módulo, no imports sueltos.
            if import.iter().any(|name| scalar_name(name)) {
                return Err(());
            }
        }
        if token == "math"
            && i > 0
            && matches!(
                tokens[i - 1],
                "as" | "type" | "mod" | "struct" | "enum" | "trait" | "union"
            )
            && !(tokens[i - 1] == "mod" && path == Path::new("engine-core/src/lib.rs"))
        {
            return Err(());
        }
    }
    Ok(imported)
}

fn central_scalar_path(tokens: &[&str], i: usize, imported: bool) -> bool {
    let mut start = i;
    while start >= 2
        && tokens[start - 1] == "::"
        && tokens[start - 2]
            .bytes()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
    {
        start -= 2;
    }
    let prefix = &tokens[start..i];
    prefix == ["engine_core", "::", "math", "::"] || (imported && prefix == ["math", "::"])
}

// include! introduce código: solo una ruta literal .rs puede pertenecer al
// inventario inspeccionado. include_str!/include_bytes! solo introducen datos.
fn included_rust_paths<'a>(tokens: &[&'a str]) -> Result<Vec<&'a str>, ()> {
    let mut paths = Vec::new();
    for (i, &token) in tokens.iter().enumerate() {
        if token != "include" || tokens.get(i + 1) != Some(&"!") {
            continue;
        }
        let close = match tokens.get(i + 2) {
            Some(&"(") => ")",
            Some(&"[") => "]",
            Some(&"{") => "}",
            _ => return Err(()),
        };
        let literal = tokens.get(i + 3).ok_or(())?;
        let first_quote = literal.find('"').ok_or(())?;
        let last_quote = literal.rfind('"').ok_or(())?;
        if first_quote >= last_quote {
            return Err(());
        }
        let prefix = &literal[..first_quote];
        if !prefix.is_empty()
            && !prefix
                .strip_prefix('r')
                .is_some_and(|s| s.bytes().all(|c| c == b'#'))
        {
            return Err(());
        }
        let file = &literal[first_quote + 1..last_quote];
        let end = i + 4 + usize::from(tokens.get(i + 4) == Some(&","));
        if tokens.get(end) != Some(&close) || file.contains('\\') || !file.ends_with(".rs") {
            return Err(());
        }
        paths.push(file);
    }
    Ok(paths)
}

fn scalar_source_violation(input: &str, path: &Path) -> bool {
    let tokens = rust_tokens(input);
    let Ok(imported) = central_math_import(&tokens, path) else {
        return true;
    };
    if included_rust_paths(&tokens).is_err() {
        return true;
    }
    let mut delimiters = Vec::new();
    for (i, &token) in tokens.iter().enumerate() {
        let in_macro = delimiters.last().copied().unwrap_or(false);
        if matches!(token, "(" | "[" | "{") {
            let begins_macro = i > 0 && tokens[i - 1] == "!"
                || (i >= 3 && tokens[i - 3..i - 1] == ["macro_rules", "!"]);
            delimiters.push(in_macro || begins_macro);
        } else if matches!(token, ")" | "]" | "}") {
            delimiters.pop();
        }
        if token == "mul_add" && path != Path::new("engine-linalg/src/modes.rs") {
            return true;
        }
        if path == Path::new("engine-core/src/math.rs") {
            continue;
        }
        if token == "libm" {
            return true;
        }
        if scalar_name(token)
            && ((in_macro && tokens.get(i + 1) != Some(&"."))
                || (i > 0 && matches!(tokens[i - 1], "." | "::"))
                || tokens.get(i + 1) == Some(&"("))
            && !central_scalar_path(&tokens, i, imported)
        {
            return true;
        }
    }
    false
}

fn scalar_violation(input: &str) -> bool {
    scalar_source_violation(input, Path::new("engine-example/src/lib.rs"))
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
    // Un .rs incluido debe estar también dentro del inventario, incluso si
    // usa ../ o un enlace simbólico. No se excluye math.rs de estas comprobaciones.
    let inspected = paths
        .iter()
        .map(|p| p.canonicalize())
        .collect::<std::io::Result<Vec<_>>>()?;
    for path in &paths {
        let source = fs::read_to_string(path)?;
        let relative = path.strip_prefix(&crates).map_err(std::io::Error::other)?;
        assert!(
            !scalar_source_violation(&source, relative),
            "Función escalar, FMA o include! fuera de política en {}",
            path.display()
        );
        for included in included_rust_paths(&rust_tokens(&source))
            .map_err(|()| std::io::Error::other("include! no inspeccionable"))?
        {
            let parent = path
                .parent()
                .ok_or_else(|| std::io::Error::other("Fuente sin directorio"))?;
            assert!(
                inspected.contains(&parent.join(included).canonicalize()?),
                "include! fuera del inventario Rust en {}: {included}",
                path.display()
            );
        }
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
            format!("x.r#{name}(y)"),
            format!("f64::r#{name}(x)"),
            format!("call!(x, {name})"),
            format!("call![x, r#{name}]"),
            format!("call! {{ x, {name} }}"),
            format!("f64 :: {name}(x)"),
            format!("use std::primitive::f64 as Real; let f = Real::{name};"),
            format!("macro_rules! m {{ () => {{ x.{name}() }}; }}"),
        ] {
            assert!(scalar_violation(&source), "{source}");
        }
        for source in [
            format!("engine_core::math::{name}(x)"),
            format!("::engine_core::math::r#{name}(x)"),
            format!("use engine_core::math; math::{name}(x)"),
            format!("call!(x, engine_core::math::{name})"),
            format!("use engine_core::math; call!(x, math::{name})"),
        ] {
            assert!(!scalar_violation(&source), "{source}");
        }
    }
    for source in ["use libm as alternate;", "libm::hypot(x, y)"] {
        assert!(scalar_violation(source), "{source}");
    }
    for source in [
        "// x.sin()\nengine_core::math::cos(x)",
        "/* f64::powi */ engine_core::math::powi(x, 2)",
        "let text = r###\"x.ln() libm\"###;",
        "let text = \"x.exp()\";",
        "x.abs() + x.max(y)",
        "use engine_core::math; math::sin(x) + math::cos(x)",
    ] {
        assert!(!scalar_violation(source), "{source}");
    }
}

#[test]
fn scalar_guard_rejects_raw_identifiers() {
    assert!(scalar_violation(include_str!(
        "fixtures/determinism_guard/raw-identifiers-negative.txt"
    )));
    assert!(!scalar_violation(include_str!(
        "fixtures/determinism_guard/raw-identifiers-positive.txt"
    )));
    // El identificador crudo no debe consumir ni ocultar los tokens siguientes.
    assert!(scalar_violation("let r#value = 1; x.sin()"));
    assert!(rust_violation("use std::collections::r#HashMap;"));
}

#[test]
fn scalar_guard_rejects_spoofed_math_and_equivalent_aliases() {
    assert!(scalar_violation(include_str!(
        "fixtures/determinism_guard/aliases-negative.txt"
    )));
    assert!(!scalar_violation(include_str!(
        "fixtures/determinism_guard/aliases-positive.txt"
    )));
    for source in [
        "use core::primitive::f64 as r#math; math::cos(x)",
        "use core::primitive::{f64 as math}; math::cos(x)",
        "use core::primitive::f64 as Real; Real::cos(x)",
        "use core::primitive::f64 as numeric; numeric::cos(x)",
        "type math = f64; math::cos(x)",
        "math::cos(x)",
        "other::math::cos(x)",
        "other::engine_core::math::cos(x)",
        "use engine_core::math; other::math::cos(x)",
        "use engine_core::math; mod math { } math::cos(x)",
        "use engine_core::math; fn f() { type math = f64; math::cos(x) }",
        "use engine_core::math; use other::math; math::cos(x)",
        "use engine_core::math as numeric; numeric::cos(x)",
        "use engine_core::math::{sin, cos}; sin(x) + cos(x)",
    ] {
        assert!(scalar_violation(source), "{source}");
    }
}

#[test]
fn scalar_guard_rejects_bare_macro_identifiers() {
    assert!(scalar_violation(include_str!(
        "fixtures/determinism_guard/macros-negative.txt"
    )));
    assert!(!scalar_violation(include_str!(
        "fixtures/determinism_guard/macros-positive.txt"
    )));
    for source in [
        "outer! { nested!(x, exp) }",
        "macro_rules! call { ($x:expr) => { f64::exp($x) }; }",
        "macro_rules! call { ($x:expr) => { invoke!($x, exp) }; }",
    ] {
        assert!(scalar_violation(source), "{source}");
    }
    assert!(!scalar_violation(
        "macro_rules! call { ($x:expr) => { engine_core::math::exp($x) }; }"
    ));
}

#[test]
fn scalar_guard_allows_fma_only_in_the_existing_sturm_source() {
    let negative = include_str!("fixtures/determinism_guard/mul-add-negative.txt");
    let positive = include_str!("fixtures/determinism_guard/mul-add-positive.txt");
    for path in [
        "engine-core/src/math.rs",
        "engine-linalg/src/reference.rs",
        "engine-linalg/tests/modes.rs",
        "engine-other/src/modes.rs",
    ] {
        assert!(scalar_source_violation(negative, Path::new(path)), "{path}");
    }
    assert!(!scalar_source_violation(
        positive,
        Path::new("engine-linalg/src/modes.rs")
    ));
    for source in [
        "x.r#mul_add(y, z)",
        "f64::mul_add(x, y, z)",
        "call!(x, mul_add)",
    ] {
        assert!(scalar_violation(source), "{source}");
    }
    // La excepción de FMA no permite trascendentes std en modes.rs.
    assert!(scalar_source_violation(
        "x.sin()",
        Path::new("engine-linalg/src/modes.rs")
    ));
}

#[test]
fn scalar_guard_rejects_includes_of_uninspected_code() {
    assert!(scalar_violation(include_str!(
        "fixtures/determinism_guard/include-negative.txt"
    )));
    assert!(!scalar_violation(include_str!(
        "fixtures/determinism_guard/include-positive.txt"
    )));
    for source in [
        "include!(r###\"hidden.inc\"###)",
        "r#include! { \"hidden.txt\" }",
        "include![\"hidden\"]",
        "include!(concat!(env!(\"OUT_DIR\"), \"/generated.rs\"))",
        "include!(path)",
        "include!(\"hidden.rs\", another)",
    ] {
        assert!(scalar_violation(source), "{source}");
    }
    let tokens = rust_tokens("include!(r###\"relative/source.rs\"###,)");
    assert_eq!(included_rust_paths(&tokens), Ok(vec!["relative/source.rs"]));
    // La excepción central tampoco admite include! de código no inspeccionado.
    assert!(scalar_source_violation(
        "include!(\"hidden.inc\")",
        Path::new("engine-core/src/math.rs")
    ));
}
