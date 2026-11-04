//! Times repeated in-process work on one linearStatic model.
//!
//! Usage: `displacement_loop <model.json> <repeats> [prepare|run]`
//! `prepare` (default) times only `displacements` after one preparation.
//! `run` times full `engine_api::run` calls. Prints `RUST <repeats> <nanoseconds>`.
fn main() {
    let mut args = std::env::args().skip(1);
    let path = args.next().expect("model path");
    let repeats: u32 = args
        .next()
        .expect("repeat count")
        .parse()
        .expect("repeat count is an integer");
    let mode = args.next().unwrap_or_else(|| "prepare".to_owned());
    let json = std::fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("read {path}: {error}");
    });
    let cancel = engine_api::CancelToken::new();
    match mode.as_str() {
        "prepare" => {
            let prepared = engine_api::PreparedLinear::prepare(&json, &cancel)
                .unwrap_or_else(|error| panic!("{error}"));
            for _ in 0..3 {
                prepared
                    .displacements()
                    .unwrap_or_else(|error| panic!("{error}"));
            }
            let start = std::time::Instant::now();
            for _ in 0..repeats {
                prepared
                    .displacements()
                    .unwrap_or_else(|error| panic!("{error}"));
            }
            println!("RUST {repeats} {}", start.elapsed().as_nanos());
        }
        "run" => {
            for _ in 0..3 {
                engine_api::run(&json, &cancel, |_| {});
            }
            let start = std::time::Instant::now();
            for _ in 0..repeats {
                engine_api::run(&json, &cancel, |_| {});
            }
            println!("RUST {repeats} {}", start.elapsed().as_nanos());
        }
        other => panic!("unknown mode {other}"),
    }
}
