//! ABI de prueba sin wasm-bindgen, WASI, imports ni estado mutable compartido.
#[path = "../tests/support/determinism.rs"]
mod fixtures;
use sha2::{Digest, Sha256};

#[no_mangle]
pub extern "C" fn fixture_count() -> u32 {
    fixtures::case_count() as u32
}

fn hash_word(hash: &[u8], word: u32) -> u64 {
    if word >= 4 {
        return 0;
    }
    hash.iter()
        .skip(word as usize * 8)
        .take(8)
        .fold(0, |value, &byte| (value << 8) | u64::from(byte))
}

// Verifica que ambos binarios embeben los mismos casos en el mismo orden.
#[no_mangle]
pub extern "C" fn fixture_name_hash(index: u32, word: u32) -> u64 {
    let Some(name) = fixtures::case_name(index as usize) else {
        return 0;
    };
    hash_word(&Sha256::digest(name.as_bytes()), word)
}

// Cada palabra se obtiene de una ejecución nueva; los cuatro u64 contienen
// los 32 bytes del SHA-256 en orden big-endian. u64 llega a JS como BigInt.
#[no_mangle]
pub extern "C" fn run_hash(index: u32, word: u32) -> u64 {
    if word >= 4 {
        return 0;
    }
    match fixtures::case_hash(index as usize) {
        Ok(hash) => hash_word(&hash, word),
        Err(_) => 0,
    }
}

#[no_mangle]
pub extern "C" fn run_status(index: u32) -> i32 {
    if fixtures::case_name(index as usize).is_none() {
        return -1;
    }
    if fixtures::case_hash(index as usize).is_ok() {
        0
    } else {
        1
    }
}
