// SPDX-License-Identifier: AGPL-3.0-only

use std::{
    io::{self, Write},
    process::Child,
};

pub fn write_child_stdin(child: &mut Child, input: &str) {
    let mut stdin = child.stdin.take().unwrap();
    // The child may exit without reading stdin; the assertions check its output.
    if let Err(error) = stdin.write_all(input.as_bytes()) {
        assert_eq!(
            error.kind(),
            io::ErrorKind::BrokenPipe,
            "Error writing child stdin: {error}"
        );
    }
    // On exit, stdin is closed before waiting for the child.
}
