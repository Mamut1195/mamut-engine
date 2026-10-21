use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

#[test]
fn published_schemas_match_the_complete_api() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut documents = BTreeMap::new();
    for kind in engine_api::schema_kinds() {
        assert!(documents.insert(kind, engine_api::schema(kind)?).is_none());
    }
    let capabilities = engine_api::capabilities();
    let advertised = capabilities["schemas"]
        .as_object()
        .ok_or("schemas ausente")?;
    for kind in advertised.keys() {
        assert!(documents.contains_key(kind.as_str()), "{kind} sin publicar");
    }
    // Detecta también un archivo fuente añadido sin entrada en el registro API.
    let mut sources = BTreeSet::new();
    for entry in std::fs::read_dir(root.join("crates/engine-api/schemas"))? {
        let path = entry?.path();
        assert_eq!(path.extension().and_then(|s| s.to_str()), Some("json"));
        assert!(
            sources.insert(std::fs::read_to_string(path)?),
            "fuente duplicada"
        );
    }
    assert_eq!(
        sources,
        documents.values().map(|s| (*s).to_owned()).collect()
    );

    let mut child = Command::new("python3")
        .args(["scripts/generate-schemas.py", "--check", "--api-stdin"])
        .current_dir(&root)
        .stdin(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("stdin ausente")?
        .write_all(&serde_json::to_vec(&documents)?)?;
    assert!(child.wait()?.success(), "árbol de esquemas desactualizado");
    Ok(())
}

#[test]
fn schema_tools_and_release_archive_are_reproducible() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(Command::new("python3")
        .args(["-B", "scripts/tests/test_schema_publication.py"])
        .current_dir(root)
        .status()?
        .success());
    Ok(())
}
