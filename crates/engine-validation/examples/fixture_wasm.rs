//! Safe wasm transport from engine-input/1 → results/1, without imports or bindgen.
//! Node transfers bytes through scalar functions; it never shares pointers.
use engine_api::{run, CancelToken, MAX_INPUT_BYTES};
use std::sync::Mutex;

struct Bridge {
    input: Vec<u8>,
    output: Vec<u8>,
}

static BRIDGE: Mutex<Bridge> = Mutex::new(Bridge {
    input: Vec::new(),
    output: Vec::new(),
});

/// Start new input. Zero indicates success; one indicates a transport error.
#[no_mangle]
pub extern "C" fn input_clear() -> u32 {
    let Ok(mut bridge) = BRIDGE.lock() else {
        return 1;
    };
    bridge.input.clear();
    bridge.output.clear();
    0
}

/// Append a byte within the size limit; zero indicates success.
#[no_mangle]
pub extern "C" fn input_push(byte: u32) -> u32 {
    let Ok(mut bridge) = BRIDGE.lock() else {
        return 1;
    };
    if byte > 255 || bridge.input.len() >= MAX_INPUT_BYTES {
        return 1;
    }
    bridge.input.push(byte as u8);
    0
}

/// Run the input and return the results/1 size; zero indicates an ABI failure.
#[no_mangle]
pub extern "C" fn execute() -> u32 {
    let Ok(mut bridge) = BRIDGE.lock() else {
        return 0;
    };
    bridge.output.clear();
    let Ok(input) = std::str::from_utf8(&bridge.input) else {
        return 0;
    };
    let outcome = run(input, &CancelToken::new(), |_| {});
    bridge.output = outcome.document.into_bytes();
    u32::try_from(bridge.output.len()).unwrap_or(0)
}

/// Read a results/1 byte; 256 indicates an out-of-range index or ABI failure.
#[no_mangle]
pub extern "C" fn output_byte(index: u32) -> u32 {
    let Ok(bridge) = BRIDGE.lock() else {
        return 256;
    };
    bridge
        .output
        .get(index as usize)
        .map_or(256, |byte| u32::from(*byte))
}
