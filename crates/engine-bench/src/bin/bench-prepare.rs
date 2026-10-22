//! Escribe los fixtures oficiales o traduce un `bench-model/1`.
#![forbid(unsafe_code)]

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if let Err(error) = engine_bench::cli(&args) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
