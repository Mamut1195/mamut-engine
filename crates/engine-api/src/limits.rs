//! Inspección léxica acotada anterior al parseo serde y al trabajo del motor.
use crate::ApiError;

/// Máximo de bytes UTF-8 de entrada (8 MiB), incluidas cadenas y espacios.
pub const MAX_INPUT_BYTES: usize = 8 * 1024 * 1024;
/// Máximo de nodos en `model.nodes`; acota el backend denso de referencia.
pub const MAX_NODES: usize = 256;
/// Máximo de miembros en `model.members`.
pub const MAX_MEMBERS: usize = 1024;
/// Máximo de casos en `cases`.
pub const MAX_CASES: usize = 64;
/// Máximo total de registros de `loads` y `member_loads` de todos los casos.
pub const MAX_LOADS: usize = 16_384;
/// Máximo de combinaciones en `combinations`.
pub const MAX_COMBINATIONS: usize = 128;
/// Máximo de objetos/arrays abiertos simultáneamente; la raíz cuenta como uno.
pub const MAX_JSON_DEPTH: usize = 64;
/// Máximo de entradas de cualquier array, incluidas colecciones auxiliares.
pub const MAX_JSON_ARRAY_ITEMS: usize = 16_384;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Role {
    Root,
    Model,
    Cases,
    Case,
    Nodes,
    Members,
    Combinations,
    Loads,
    Other,
}

struct Frame {
    role: Role,
    array: bool,
    // Array: próximo token inicia una entrada. Objeto: se espera una clave.
    next: bool,
    key: String,
    items: usize,
}

fn exceeded(name: &str, limit: usize) -> ApiError {
    ApiError::InvalidInput(format!("Limit {name} exceeded (maximum {limit})"))
}

fn child_role(parent: Option<&Frame>, array: bool) -> Role {
    match parent {
        None if !array => Role::Root,
        Some(p) if p.array && p.role == Role::Cases && !array => Role::Case,
        Some(p) if !p.array => match (p.role, p.key.as_str(), array) {
            (Role::Root, "model", false) => Role::Model,
            (Role::Root, "cases", true) => Role::Cases,
            (Role::Root, "combinations", true) => Role::Combinations,
            (Role::Model, "nodes", true) => Role::Nodes,
            (Role::Model, "members", true) => Role::Members,
            (Role::Case, "loads" | "member_loads", true) => Role::Loads,
            _ => Role::Other,
        },
        _ => Role::Other,
    }
}

/// No construye Values/DTOs ni recurre: solo conserva hasta MAX_JSON_DEPTH marcos.
/// Serde sigue siendo responsable de la sintaxis y de las claves duplicadas.
pub(crate) fn preflight(input: &str) -> Result<(), ApiError> {
    if input.len() > MAX_INPUT_BYTES {
        return Err(exceeded("MAX_INPUT_BYTES", MAX_INPUT_BYTES));
    }
    let bytes = input.as_bytes();
    let mut frames: Vec<Frame> = Vec::new();
    let mut counts = [0; 5];
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_whitespace() {
            index += 1;
            continue;
        }
        if !matches!(byte, b',' | b':' | b']' | b'}') {
            if let Some(frame) = frames.last_mut().filter(|f| f.array && f.next) {
                frame.next = false;
                frame.items += 1;
                let counter = match frame.role {
                    Role::Nodes => Some((0, "MAX_NODES", MAX_NODES)),
                    Role::Members => Some((1, "MAX_MEMBERS", MAX_MEMBERS)),
                    Role::Cases => Some((2, "MAX_CASES", MAX_CASES)),
                    Role::Combinations => Some((3, "MAX_COMBINATIONS", MAX_COMBINATIONS)),
                    Role::Loads => Some((4, "MAX_LOADS", MAX_LOADS)),
                    _ => None,
                };
                if let Some((slot, name, limit)) = counter {
                    counts[slot] += 1;
                    if counts[slot] > limit {
                        return Err(exceeded(name, limit));
                    }
                }
                if frame.items > MAX_JSON_ARRAY_ITEMS {
                    return Err(exceeded("MAX_JSON_ARRAY_ITEMS", MAX_JSON_ARRAY_ITEMS));
                }
            }
        }
        match byte {
            b'{' | b'[' => {
                if frames.len() == MAX_JSON_DEPTH {
                    return Err(exceeded("MAX_JSON_DEPTH", MAX_JSON_DEPTH));
                }
                let array = byte == b'[';
                frames.push(Frame {
                    role: child_role(frames.last(), array),
                    array,
                    next: true,
                    key: String::new(),
                    items: 0,
                });
                index += 1;
            }
            b'}' | b']' => {
                let frame = frames.pop().ok_or_else(|| {
                    ApiError::InvalidInput(
                        "JSON closing delimiter without opening delimiter".into(),
                    )
                })?;
                if frame.array != (byte == b']') {
                    return Err(ApiError::InvalidInput(
                        "Mismatched JSON closing delimiter".into(),
                    ));
                }
                index += 1;
            }
            b',' | b':' => {
                if let Some(frame) = frames.last_mut() {
                    frame.next = byte == b',';
                    if frame.next {
                        frame.key.clear();
                    }
                }
                index += 1;
            }
            b'"' => {
                let start = index;
                index += 1;
                loop {
                    match bytes.get(index) {
                        Some(b'"') => {
                            index += 1;
                            break;
                        }
                        Some(b'\\') => index += 2,
                        Some(_) => index += 1,
                        None => {
                            return Err(ApiError::InvalidInput("Unterminated JSON string".into()));
                        }
                    }
                }
                if let Some(frame) = frames.last_mut().filter(|f| !f.array && f.next) {
                    // Decodifica también claves escapadas; nunca reserva arrays/modelos.
                    frame.key = serde_json::from_str(&input[start..index])
                        .map_err(|e| ApiError::InvalidInput(e.to_string()))?;
                }
            }
            _ => {
                let start = index;
                while index < bytes.len()
                    && !bytes[index].is_ascii_whitespace()
                    && !matches!(bytes[index], b',' | b':' | b'[' | b']' | b'{' | b'}' | b'"')
                {
                    index += 1;
                }
                if byte == b'-' || byte.is_ascii_digit() {
                    check_number(&input[start..index])?;
                }
            }
        }
    }
    Ok(())
}

fn check_number(token: &str) -> Result<(), ApiError> {
    if token.contains(['.', 'e', 'E']) {
        if token.parse::<f64>().is_ok_and(f64::is_finite) {
            return Ok(());
        }
        return Err(ApiError::InvalidInput(
            "JSON number is outside the finite f64 range".into(),
        ));
    }
    // JCS convierte a binary64: impedir pérdida de identidad antes de serde,
    // incluso para enteros enormes que serde podría convertir a float.
    const MAX_SAFE_INTEGER: i64 = (1_i64 << 53) - 1;
    let valid = token
        .parse::<i64>()
        .is_ok_and(|n| (-MAX_SAFE_INTEGER..=MAX_SAFE_INTEGER).contains(&n));
    if valid {
        Ok(())
    } else {
        Err(ApiError::InvalidInput(
            "JSON integer is outside the JCS safe range: |n| <= 9007199254740991 (2^53-1) is required"
                .into(),
        ))
    }
}
