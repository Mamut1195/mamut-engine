//! Transporte local de los bytes de schema() al generador, sin E/S de red.
use std::collections::BTreeMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut documents = BTreeMap::new();
    for kind in engine_api::schema_kinds() {
        documents.insert(kind, engine_api::schema(kind)?);
    }
    println!("{}", serde_json::to_string(&documents)?);
    Ok(())
}
