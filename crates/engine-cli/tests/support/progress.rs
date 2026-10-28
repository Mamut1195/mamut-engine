// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

use super::error_schema::schema_validator as schema;

fn progress_schema() -> Value {
    value(include_bytes!("../../schemas/progress-1.json"))
}

pub(super) fn conforms(event: &Value) -> bool {
    schema::conforms(event, &progress_schema())
}

#[test]
fn input_transports_and_generous_timeout_preserve_progress_bytes() {
    let files = Files::new();
    let expected = files.run(RUN, true);
    assert_code(&expected, 0);
    for timed in [false, true] {
        let mut args = vec!["run", "--input", "-", "--progress", "ndjson"];
        let mut command = files.command();
        command
            .args(["run", "--input"])
            .arg(files.path("input.json"))
            .args(["--progress", "ndjson"]);
        if timed {
            args.extend(["--timeout", "60"]);
            command.args(["--timeout", "60"]);
        }
        let stdin = with_input_stdin(&files, &args, INPUT.as_bytes());
        let path = command.output().unwrap();
        for output in [stdin, path] {
            assert_code(&output, 0);
            assert_eq!(output.stdout, expected.stdout);
            assert_eq!(output.stderr, expected.stderr);
        }
    }
}

#[test]
fn binary_stage_progress_is_canonical_ordered_and_repeatable_for_all_fixtures() {
    let files = Files::new();
    let mut catalog = value(CATALOG.as_bytes());
    let slot = catalog["profile"]["slots"][0].clone();
    catalog["profile"]["slots"] = json!([slot, slot, slot]);
    files.write("catalog.json", &catalog.to_string());
    let mut workflow = value(RUN.as_bytes());
    workflow["steps"] = json!([
        value(VALIDATE.as_bytes())["steps"][0],
        value(RUN.as_bytes())["steps"][0],
        value(RUN.as_bytes())["steps"][0]
    ]);
    let workflow = workflow.to_string();
    let inputs = [
        ("linear", INPUT),
        (
            "pDelta",
            include_str!("../../../engine-api/tests/fixtures/determinism/pdelta.json"),
        ),
        (
            "modal",
            include_str!("../../../engine-api/tests/fixtures/determinism/modal-1dof.json"),
        ),
    ];
    let root = progress_schema();
    for &(name, input) in inputs.iter().chain(sfc010::FIXTURES) {
        files.write("input.json", input);
        let plain = files.run(&workflow, false);
        assert!(matches!(plain.status.code(), Some(0 | 1)), "{name}");
        assert!(plain.stderr.is_empty(), "{name}");
        let first = files.run(&workflow, true);
        let second = files.run(&workflow, true);
        assert_eq!(first.status, plain.status, "{name}");
        assert_eq!(second.status, plain.status, "{name}");
        assert_eq!(first.stdout, plain.stdout, "{name}");
        assert_eq!(second.stdout, plain.stdout, "{name}");
        assert_eq!(first.stderr, second.stderr, "{name}");
        assert_eq!(first.stderr.last(), Some(&b'\n'));
        let lines: Vec<_> = first
            .stderr
            .strip_suffix(b"\n")
            .unwrap()
            .split(|&b| b == b'\n')
            .collect();
        let events: Vec<Value> = lines.iter().map(|line| value(line)).collect();
        let mut position = 0;
        for index in 0..3 {
            let method = if index == 0 {
                "engine.validate"
            } else {
                "engine.run"
            };
            let sdk = |phase| {
                json!({"phase":phase, "index":index,
                "capability_slot":"engine", "method_id":method, "method_version":1})
            };
            assert_eq!(events[position], sdk("started"), "{name}");
            assert_eq!(lines[position], sdk("started").to_string().as_bytes());
            position += 1;
            let mut expected = Vec::new();
            if index == 0 {
                engine_api::validate_with(input, &engine_api::CancelToken::new(), |p| {
                    expected.push(p)
                })
                .unwrap();
            } else {
                engine_api::run(input, &engine_api::CancelToken::new(), |p| expected.push(p));
            }
            assert!(!expected.is_empty(), "{name}");
            for p in expected {
                let event = &events[position];
                assert!(schema::conforms(event, &root), "{name}: {event}");
                assert_eq!(
                    *event,
                    json!({"schema":"progress/1", "index":index,
                    "operation":p.operation, "stage":p.stage,
                    "completed":p.completed, "total":p.total})
                );
                assert_eq!(
                    lines[position],
                    engine_api::canonical_json(&event.to_string())
                        .unwrap()
                        .as_bytes()
                );
                position += 1;
            }
            assert_eq!(events[position], sdk("finished"), "{name}");
            assert_eq!(lines[position], sdk("finished").to_string().as_bytes());
            position += 1;
        }
        assert_eq!(position, events.len(), "{name}");
        let stages: Vec<_> = events.iter().filter_map(|e| e["stage"].as_str()).collect();
        match name {
            "pDelta" => assert!(stages.contains(&"pdeltaIteration")),
            "modal" => assert!(stages.contains(&"modal")),
            "cubic" | "multilinear" | "steel" => assert!(stages.contains(&"newtonStep")),
            "consolidation" => assert!(stages.contains(&"geotechnical")),
            _ => {}
        }
    }
}

#[test]
fn progress_schema_rejects_missing_wrong_extra_and_nonfinite_fields() {
    let root = progress_schema();
    let event = json!({"schema":"progress/1", "index":0, "operation":"run",
        "stage":"parse", "completed":1, "total":8});
    assert!(schema::conforms(&event, &root));
    for key in event.as_object().unwrap().keys() {
        let mut changed = event.clone();
        changed.as_object_mut().unwrap().remove(key);
        assert!(!schema::conforms(&changed, &root), "{key}");
    }
    for (key, bad) in [
        ("schema", json!("progress/2")),
        ("index", json!(-1)),
        ("completed", json!(1.5)),
        ("total", Value::Null),
        ("operation", json!("other")),
        ("stage", json!(3)),
        ("duration", json!(0)),
        ("residual", json!(f64::INFINITY)),
    ] {
        let mut changed = event.clone();
        changed[key] = bad;
        assert!(!schema::conforms(&changed, &root), "{key}");
    }
}

struct LineWriter {
    lines: Rc<RefCell<Vec<Value>>>,
    fail_progress: bool,
    fail_flush: bool,
}

impl Write for LineWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        // A writer call always receives a complete JSON line.
        assert_eq!(bytes.last(), Some(&b'\n'));
        let event = value(bytes);
        if self.fail_progress && event["schema"] == "progress/1" {
            return Err(io::Error::other("Stage output unavailable"));
        }
        self.lines.borrow_mut().push(event);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.fail_flush
            && self
                .lines
                .borrow()
                .last()
                .is_some_and(|e| e["schema"] == "progress/1")
        {
            Err(io::Error::other("Stage flush unavailable"))
        } else {
            Ok(())
        }
    }
}

#[test]
fn stage_lines_stream_before_host_callback_and_io_errors_return_seventy() {
    let files = Files::new();
    let mut args = files.args();
    args.extend(["--progress".into(), "ndjson".into()]);
    for (fail_progress, fail_flush) in [(false, false), (true, false), (false, true)] {
        let lines = Rc::new(RefCell::new(Vec::<Value>::new()));
        let observed = Rc::clone(&lines);
        let calls = Rc::new(RefCell::new(0));
        let called = Rc::clone(&calls);
        let cli = EngineCli::new(Some(files.path("input.json"))).with_engine_progress(move |p| {
            *called.borrow_mut() += 1;
            if !fail_progress && !fail_flush {
                let lines = observed.borrow();
                let event = lines.last().unwrap();
                assert_eq!(event["stage"], p.stage);
                assert_eq!(event["completed"], p.completed);
            }
        });
        let mut writer = LineWriter {
            lines,
            fail_progress,
            fail_flush,
        };
        let mut stdout = Vec::new();
        let code = cli.run(&args, &mut RUN.as_bytes(), &mut stdout, &mut writer);
        assert!(*calls.borrow() > 0);
        assert_eq!(code, if fail_progress || fail_flush { 70 } else { 0 });
        assert_eq!(stdout.is_empty(), code == 70);
        if code == 70 {
            let stream: String = writer
                .lines
                .borrow()
                .iter()
                .map(|event| format!("{event}\n"))
                .collect();
            let envelope = error_schema::assert_error_stream(stream.as_bytes(), 70);
            assert_eq!(envelope["code"], "CLI-E-INTERNAL");
            assert_eq!(envelope["operation"], "run");
        }
    }
}

#[test]
fn failed_step_keeps_stages_before_error_without_finished() {
    let files = Files::new();
    let mut input = value(include_bytes!(
        "../../../engine-api/tests/fixtures/determinism/pdelta.json"
    ));
    input["analyses"][0]["options"]["max_iterations"] = json!(1);
    files.write("input.json", &input.to_string());
    let plain = files.run(RUN, false);
    assert_code(&plain, 4);
    let first = files.run(RUN, true);
    let second = files.run(RUN, true);
    assert_code(&first, 4);
    assert_code(&second, 4);
    assert_eq!(first.stdout, plain.stdout);
    assert_eq!(first.stderr, second.stderr);
    let events: Vec<Value> = first
        .stderr
        .strip_suffix(b"\n")
        .unwrap()
        .split(|&b| b == b'\n')
        .map(value)
        .collect();
    assert_eq!(events[0]["phase"], "started");
    assert!(events[1..events.len() - 1]
        .iter()
        .all(|e| schema::conforms(e, &progress_schema())));
    assert!(events.len() > 2);
    assert_eq!(events.last().unwrap(), &value(&plain.stderr));
}
