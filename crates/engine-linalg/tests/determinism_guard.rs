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

// Los literales se mantienen opacos; no se interpretan escapes en rutas.
fn literal_string(literal: &str) -> Result<&str, ()> {
    let first = literal.find('"').ok_or(())?;
    let last = literal.rfind('"').ok_or(())?;
    let prefix = &literal[..first];
    if first >= last
        || (!prefix.is_empty()
            && !prefix
                .strip_prefix('r')
                .is_some_and(|s| s.bytes().all(|c| c == b'#')))
    {
        return Err(());
    }
    Ok(&literal[first + 1..last])
}

fn literal_rust_path(literal: &str) -> Result<&str, ()> {
    let file = literal_string(literal)?;
    if file.contains('\\') || !file.ends_with(".rs") {
        return Err(());
    }
    Ok(file)
}

// Incluye atributos internos y atributos anidados en macros, pero nunca texto
// de comentarios/cadenas. El índice permite distinguir atributos de raíz.
fn rust_attributes<'a, 's>(tokens: &'a [&'s str]) -> Vec<(usize, &'a [&'s str])> {
    let mut attributes = Vec::new();
    for (i, &token) in tokens.iter().enumerate() {
        if token != "#" {
            continue;
        }
        let open = i + 1 + usize::from(tokens.get(i + 1) == Some(&"!"));
        if tokens.get(open) != Some(&"[") {
            continue;
        }
        let mut depth = 1;
        for (end, &t) in tokens.iter().enumerate().skip(open + 1) {
            match t {
                "[" => depth += 1,
                "]" => depth -= 1,
                _ => {}
            }
            if depth == 0 {
                attributes.push((i, &tokens[open + 1..end]));
                break;
            }
        }
    }
    attributes
}

fn comma_items<'a, 's>(tokens: &'a [&'s str]) -> Vec<&'a [&'s str]> {
    let mut items = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (i, &token) in tokens.iter().enumerate() {
        match token {
            "(" | "[" | "{" => depth += 1,
            ")" | "]" | "}" => depth -= 1,
            "," if depth == 0 => {
                items.push(&tokens[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    items.push(&tokens[start..]);
    items
}

// cfg_attr puede contener varios atributos y otros cfg_attr. Su condición es
// un predicado, no un atributo path/link_name, y no se visita como tal.
fn attribute_meta_items<'a, 's>(body: &'a [&'s str]) -> Vec<&'a [&'s str]> {
    if body.starts_with(&["cfg_attr", "("]) && body.last() == Some(&")") {
        comma_items(&body[2..body.len() - 1])
            .into_iter()
            .skip(1)
            .flat_map(attribute_meta_items)
            .collect()
    } else {
        vec![body]
    }
}

// include! y #[path] introducen código: solo rutas literales .rs pueden
// pertenecer al inventario inspeccionado. include_str!/include_bytes! son datos.
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
        let file = literal_rust_path(literal)?;
        let end = i + 4 + usize::from(tokens.get(i + 4) == Some(&","));
        if tokens.get(end) != Some(&close) {
            return Err(());
        }
        paths.push(file);
    }
    for (_, body) in rust_attributes(tokens) {
        for meta in attribute_meta_items(body) {
            if meta.first() == Some(&"path") {
                let ["path", "=", literal] = meta else {
                    return Err(());
                };
                paths.push(literal_rust_path(literal)?);
            }
        }
    }
    Ok(paths)
}

fn code_paths_violation(input: &str, path: &Path, inspected: &[PathBuf]) -> bool {
    let tokens = rust_tokens(input);
    let Ok(paths) = included_rust_paths(&tokens) else {
        return true;
    };
    paths.into_iter().any(|included| {
        // Una ruta de módulo puede resolverse de forma distinta en un módulo
        // inline. Solo se acepta el destino comprobable relativo a la fuente.
        !path
            .parent()
            .and_then(|p| p.join(included).canonicalize().ok())
            .is_some_and(|p| inspected.contains(&p))
    })
}

fn ffi_violation(input: &str, path: &Path) -> bool {
    if path
        .components()
        .nth(1)
        .is_some_and(|c| c.as_os_str() == "examples")
    {
        return false;
    }
    let tokens = rust_tokens(input);
    let link_name = rust_attributes(&tokens).into_iter().any(|(_, body)| {
        attribute_meta_items(body)
            .iter()
            .any(|meta| meta.first() == Some(&"link_name"))
    });
    link_name
        || tokens.iter().enumerate().any(|(i, &token)| {
            token == "extern"
                && (tokens.get(i + 1) == Some(&"{")
                    || (tokens.get(i + 1).is_some_and(|t| literal_string(t).is_ok())
                        && tokens.get(i + 2) == Some(&"{")))
        })
}

// Sin excepciones en crates/*: las 15 bibliotecas compilan sin unsafe.
// bindings/python también compila con forbid pese a sus macros PyO3, y se
// comprueba aparte. heap_peak.rs es un test de integración, no una biblioteca.
const UNSAFE_LIBRARY_EXCEPTIONS: &[(&str, &str)] = &[];

fn library_unsafe_policy_violation(input: &str, path: &Path) -> bool {
    if UNSAFE_LIBRARY_EXCEPTIONS
        .iter()
        .any(|(exception, reason)| path == Path::new(exception) && !reason.is_empty())
    {
        return false;
    }
    let tokens = rust_tokens(input);
    !rust_attributes(&tokens).into_iter().any(|(i, body)| {
        let mut depth = 0;
        for &token in &tokens[..i] {
            match token {
                "(" | "[" | "{" => depth += 1,
                ")" | "]" | "}" => depth -= 1,
                _ => {}
            }
        }
        depth == 0
            && tokens.get(i + 1) == Some(&"!")
            && body.starts_with(&["forbid", "("])
            && body.last() == Some(&")")
            && comma_items(&body[2..body.len() - 1]).contains(&&["unsafe_code"][..])
    })
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
            "Función escalar, FMA, include! o #[path] fuera de política en {}",
            path.display()
        );
        assert!(
            !code_paths_violation(&source, path, &inspected),
            "include! o #[path] fuera del inventario Rust en {}",
            path.display()
        );
    }
    Ok(())
}

#[test]
fn engine_crates_reject_ffi_and_require_safe_libraries() -> std::io::Result<()> {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut paths = Vec::new();
    files(&crates, &mut paths)?;
    paths.sort();
    let mut libraries = 0;
    for path in paths {
        let relative = path.strip_prefix(&crates).map_err(std::io::Error::other)?;
        let source = fs::read_to_string(&path)?;
        assert!(
            !ffi_violation(&source, relative),
            "#[link_name] o bloque extern prohibido en {}",
            path.display()
        );
        if relative.components().count() == 3 && relative.ends_with("src/lib.rs") {
            libraries += 1;
            assert!(
                !library_unsafe_policy_violation(&source, relative),
                "Biblioteca sin #![forbid(unsafe_code)] en {}",
                path.display()
            );
        }
    }
    assert!(libraries > 0, "La guardia no encontró bibliotecas");
    let python = crates.join("../bindings/python/src/lib.rs");
    assert!(
        !library_unsafe_policy_violation(
            &fs::read_to_string(python)?,
            Path::new("bindings/python/src/lib.rs")
        ),
        "Binding Python sin #![forbid(unsafe_code)]"
    );
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

#[test]
fn guard_rejects_module_paths_not_ending_in_rs() {
    for source in [
        "#[path = \"hidden.txt\"] mod m;",
        "# [ path /* espacio */ = r###\"hidden.txt\"### ] mod m;",
        "#[cfg_attr(any(), path = \"hidden.txt\")] mod m;",
        "#[cfg_attr(all(), allow(dead_code), cfg_attr(unix, path = r#\"hidden.txt\"#))] mod m;",
        "#[path = concat!(\"hidden\", \".rs\")] mod m;",
        "#[path = file] mod m;",
        "#[path = b\"hidden.rs\"] mod m;",
        "#[path = \"hidden\\\\.rs\"] mod m;",
    ] {
        assert!(scalar_violation(source), "{source}");
        assert!(
            scalar_source_violation(source, Path::new("engine-core/src/math.rs")),
            "{source}"
        );
    }
}

#[test]
fn guard_rejects_module_paths_outside_the_inventory() -> std::io::Result<()> {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut paths = Vec::new();
    files(&crates, &mut paths)?;
    let inspected = paths
        .iter()
        .map(|p| p.canonicalize())
        .collect::<std::io::Result<Vec<_>>>()?;
    let path = crates.join("engine-linalg/src/lib.rs");
    // Archivo real .rs fuera de crates/**; no depende de un destino inexistente.
    for source in [
        "#[path = \"../../../bindings/python/src/lib.rs\"] mod m;",
        "#[cfg_attr(unix, path = r###\"../../../bindings/python/src/lib.rs\"###)] mod m;",
        "include!(\"../../../bindings/python/src/lib.rs\");",
        "#[path = \"missing.rs\"] mod m;",
    ] {
        assert!(code_paths_violation(source, &path, &inspected), "{source}");
    }
    for source in [
        "#[path = \"reference.rs\"] mod m;",
        "# [ path = r###\"reference.rs\"### ] mod m;",
        "#[cfg_attr(any(), path = \"reference.rs\")] mod m;",
        "#[cfg_attr(all(), allow(dead_code), cfg_attr(unix, path = \"reference.rs\"))] mod m;",
        "#[path = \"../../engine-core/src/math.rs\"] mod m;",
        "include!(\"reference.rs\");",
    ] {
        assert!(!code_paths_violation(source, &path, &inspected), "{source}");
    }
    Ok(())
}

#[test]
fn guard_rejects_link_name_attributes() {
    for source in [
        "#[link_name = \"sin\"] fn imported();",
        "# [ r#link_name /* espacio */ = r###\"native\"### ] static VALUE: u8;",
        "#[cfg_attr(unix, link_name = \"native\")] fn imported();",
        "#[cfg_attr(all(), allow(dead_code), cfg_attr(unix, link_name = \"native\"))] fn imported();",
    ] {
        assert!(ffi_violation(source, Path::new("engine-core/src/math.rs")), "{source}");
        assert!(ffi_violation(source, Path::new("engine-linalg/tests/ffi.rs")), "{source}");
        assert!(!ffi_violation(source, Path::new("engine-api/examples/ffi.rs")), "{source}");
    }
}

#[test]
fn guard_rejects_extern_blocks() {
    for source in [
        "extern \"C\" { fn imported(); }",
        "unsafe extern /* espacio */ \"system\" { static VALUE: u8; }",
        "extern { }",
        "extern \"C-unwind\" {}",
        "extern r###\"C\"### {}",
        "#[cfg(any())] extern \"C\" {}",
        "macro_rules! ffi { () => { extern \"C\" {} }; }",
    ] {
        assert!(
            ffi_violation(source, Path::new("engine-core/src/math.rs")),
            "{source}"
        );
        assert!(
            ffi_violation(source, Path::new("engine-linalg/tests/ffi.rs")),
            "{source}"
        );
        assert!(
            !ffi_violation(source, Path::new("engine-api/examples/ffi.rs")),
            "{source}"
        );
    }
}

#[test]
fn guard_rejects_libraries_without_forbid_unsafe_code() {
    let path = Path::new("engine-new/src/lib.rs");
    for source in [
        "pub fn safe() {}",
        "#![deny(unsafe_code)]",
        "#[forbid(unsafe_code)] mod child {}",
        "#[cfg_attr(unix, forbid(unsafe_code))] mod child {}",
        "#![cfg_attr(unix, forbid(unsafe_code))]",
        "mod child { #![forbid(unsafe_code)] }",
        "macro_rules! m { () => { #![forbid(unsafe_code)] }; }",
        "// #![forbid(unsafe_code)]\nconst TEXT: &str = \"#![forbid(unsafe_code)]\";",
        "/* #![forbid(unsafe_code)] */ const TEXT: &str = r###\"#![forbid(unsafe_code)]\"###;",
    ] {
        assert!(library_unsafe_policy_violation(source, path), "{source}");
    }
    for source in [
        "#![forbid(unsafe_code)]",
        "# ! [ forbid /* espacio */ ( r#unsafe_code, ) ]",
        "#![forbid(missing_docs, unsafe_code, unused)]",
    ] {
        assert!(!library_unsafe_policy_violation(source, path), "{source}");
    }
}

#[test]
fn guard_allows_comments_strings_and_non_ffi_extern_items() {
    for source in [
        "// #[link_name = \"sin\"] extern \"C\" {} #[path = \"hidden.txt\"]",
        "/* extern {} /* #[path = \"hidden.txt\"] */ #[link_name = \"sin\"] */",
        "let text = \"extern {} #[link_name = \\\"sin\\\"] #[path = \\\"hidden.txt\\\"]\";",
        "let text = r###\"extern \"C\" {} #[link_name = \"sin\"] #[path = \"hidden.txt\"]\"###;",
        "extern crate engine_core as core;",
        "pub extern \"C\" fn exported() {}",
        "type Callback = extern \"C\" fn();",
        "fn link_name() {} let path = \"hidden.txt\";",
        "#[cfg(path = \"linux\")] mod m;",
        "#[cfg_attr(path = \"linux\", allow(dead_code))] mod m;",
    ] {
        assert!(
            !ffi_violation(source, Path::new("engine-linalg/src/lib.rs")),
            "{source}"
        );
        assert!(
            included_rust_paths(&rust_tokens(source)).is_ok(),
            "{source}"
        );
    }
    // examples/ solo exceptúa FFI; no relaja la política de código incluido.
    assert!(scalar_source_violation(
        "#[path = \"hidden.txt\"] mod m;",
        Path::new("engine-api/examples/ffi.rs")
    ));
    assert!(rust_violation("extern crate rayon;"));
    // Un módulo de biblioteca llamado examples no es el directorio de ejemplos.
    assert!(ffi_violation(
        "extern {}",
        Path::new("engine-core/src/examples/ffi.rs")
    ));
}
