//! Only runs[].engine_input is deserialized; other fields are ignored.
use engine_api::{validate, CancelToken, Status};
use serde::Deserialize;
use serde_json::Value;
use std::{hint::black_box, time::Instant};

#[derive(Deserialize)]
struct Fixture {
    runs: Vec<InputRun>,
}

#[derive(Deserialize)]
struct InputRun {
    engine_input: Value,
}

#[test]
fn validate_is_at_least_ten_times_faster_than_run_on_e1() {
    let fixture: Fixture = serde_json::from_str(include_str!(
        "../../../docs/validation-report/fixtures/val032/E1-v1.1.2.json"
    ))
    .unwrap();
    let input = fixture.runs[0].engine_input.to_string();
    let cancel = CancelToken::new();
    let check_validate = || {
        let diagnostics = validate(black_box(&input)).unwrap();
        assert!(diagnostics.is_empty());
        black_box(diagnostics);
    };
    let check_run = || {
        let outcome = engine_api::run(black_box(&input), &cancel, |_| {});
        assert!(matches!(outcome.status, Status::Ok | Status::Warning));
        black_box(outcome);
    };
    // Warms up both paths and alternates the order to reduce bias.
    check_validate();
    check_run();
    let mut validation = Vec::new();
    let mut execution = Vec::new();
    let measure = |operation: &dyn Fn()| {
        let start = Instant::now();
        operation();
        start.elapsed()
    };
    for index in 0..11 {
        if index % 2 == 0 {
            validation.push(measure(&check_validate));
            execution.push(measure(&check_run));
        } else {
            execution.push(measure(&check_run));
            validation.push(measure(&check_validate));
        }
    }
    validation.sort_unstable();
    execution.sort_unstable();
    let validate_median = validation[validation.len() / 2];
    let run_median = execution[execution.len() / 2];
    let ratio = run_median.as_secs_f64() / validate_median.as_secs_f64();
    eprintln!(
        "E1, mediana N=11: validate={validate_median:?}, run={run_median:?}, ratio={ratio:.2}x"
    );
    assert!(ratio >= 10.0, "validate/run: {ratio:.2}x < 10x");
}
