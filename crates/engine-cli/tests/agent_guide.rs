// SPDX-License-Identifier: AGPL-3.0-only

use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    io::Read,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
    sync::{Arc, Mutex, OnceLock},
    thread,
    time::{Duration, Instant},
};

const GUIDE: &str = include_str!("../../../docs/GUIA_CLI_AGENTES.md");
const ERROR_CODES: &str = include_str!("../../../docs/contracts/error-codes-1.json");

#[cfg(unix)]
struct Fence<'a> {
    info: &'a str,
    body: String,
}

#[cfg(unix)]
fn fences(source: &str) -> Vec<Fence<'_>> {
    let mut blocks = Vec::new();
    let mut lines = source.lines();
    while let Some(line) = lines.next() {
        if let Some(info) = line.trim_start().strip_prefix("```") {
            let mut body = String::new();
            let mut closed = false;
            for line in lines.by_ref() {
                if line.trim() == "```" {
                    closed = true;
                    break;
                }
                body.push_str(line);
                body.push('\n');
            }
            assert!(closed, "Unclosed guide fence: {info}");
            blocks.push(Fence { info, body });
        }
    }
    blocks
}

fn prose_lines(source: &str) -> Vec<&str> {
    let mut inside = false;
    source
        .lines()
        .filter(|line| {
            if line.trim_start().starts_with("```") {
                inside = !inside;
                false
            } else {
                !inside
            }
        })
        .collect()
}

fn section(heading: &str) -> Vec<&str> {
    let lines = prose_lines(GUIDE);
    let start = lines
        .iter()
        .position(|line| *line == heading)
        .unwrap_or_else(|| panic!("Missing guide section: {heading}"));
    lines[start + 1..]
        .iter()
        .copied()
        .take_while(|line| !line.starts_with("## "))
        .collect()
}

fn table_rows(heading: &str) -> Vec<Vec<&str>> {
    section(heading)
        .into_iter()
        .filter(|line| line.starts_with('|'))
        .map(|line| line.trim_matches('|').split('|').map(str::trim).collect())
        .collect()
}

fn code_cell(cell: &str) -> Option<&str> {
    cell.strip_prefix('`')?.strip_suffix('`')
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn clean_environment(command: &mut Command) -> &mut Command {
    command
        .env_remove("MAMUT_ENGINE_INPUT")
        .env_remove("MAMUT_ENGINE_TIMEOUT")
        .env_remove("MAMUT_ENGINE_CANCEL_FILE")
        .current_dir(repo_root())
        .stdin(Stdio::null())
}

type Captured = Arc<Mutex<Vec<u8>>>;

fn capture(mut pipe: impl Read + Send + 'static) -> (Captured, thread::JoinHandle<()>) {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let captured = Arc::clone(&bytes);
    let reader = thread::spawn(move || {
        let mut buffer = [0; 8192];
        loop {
            let count = pipe
                .read(&mut buffer)
                .expect("Read captured process stream");
            if count == 0 {
                break;
            }
            captured
                .lock()
                .expect("Lock captured process stream")
                .extend_from_slice(&buffer[..count]);
        }
    });
    (bytes, reader)
}

fn run_with_deadline(command: &mut Command, label: &str) -> (Output, bool) {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| panic!("Cannot start {label}: {error}"));
    // Drain both streams while polling, so a full pipe cannot stall the process.
    let (stdout, stdout_reader) = capture(child.stdout.take().expect("Piped stdout"));
    let (stderr, stderr_reader) = capture(child.stderr.take().expect("Piped stderr"));
    let start = Instant::now();
    let timed_out = loop {
        let finished = child
            .try_wait()
            .unwrap_or_else(|error| panic!("Cannot poll {label}: {error}"))
            .is_some();
        if finished && stdout_reader.is_finished() && stderr_reader.is_finished() {
            break false;
        }
        if start.elapsed() >= Duration::from_secs(120) {
            let _ = child.kill();
            break true;
        }
        thread::sleep(Duration::from_millis(10));
    };
    let status = child
        .wait()
        .unwrap_or_else(|error| panic!("Cannot reap {label}: {error}"));
    if !timed_out {
        stdout_reader.join().expect("Stdout reader failed");
        stderr_reader.join().expect("Stderr reader failed");
    }
    // On timeout descendants may still hold pipes; do not block joining readers.
    let stdout = stdout.lock().expect("Read captured stdout").clone();
    let stderr = stderr.lock().expect("Read captured stderr").clone();
    (
        Output {
            status,
            stdout,
            stderr,
        },
        timed_out,
    )
}

fn diagnostics(label: &str, output: &Output, expected: &str) -> String {
    format!(
        "{label}: obtained exit {:?}\nstdout:\n{}\nstderr:\n{}\nexpected:\n{expected}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    )
}

fn binary_json(args: &[&str], input: Option<&Path>) -> Value {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mamut-engine"));
    clean_environment(&mut command).args(args);
    if let Some(input) = input {
        command.arg("--input").arg(input);
    }
    let label = format!("mamut-engine {}", args.join(" "));
    let (output, timed_out) = run_with_deadline(&mut command, &label);
    let details = diagnostics(&label, &output, "exit 0 and one JSON document");
    assert!(!timed_out, "120 s deadline exceeded; {details}");
    assert_eq!(output.status.code(), Some(0), "{details}");
    serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("Invalid JSON: {error}; {details}"))
}

fn describe() -> &'static Value {
    static DESCRIPTION: OnceLock<Value> = OnceLock::new();
    DESCRIPTION.get_or_init(|| binary_json(&["describe", "--json"], None))
}

#[cfg(unix)]
mod examples {
    use super::*;
    use std::fs;

    struct Example<'a> {
        id: &'a str,
        exit: i32,
        script: &'a str,
        expected: &'a str,
    }

    fn attribute<'a>(word: &'a str, prefix: &str, info: &str) -> &'a str {
        word.strip_prefix(prefix)
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| panic!("Missing {prefix} in guide fence: {info}"))
    }

    fn parse_examples<'a>(blocks: &'a [Fence<'a>]) -> Vec<Example<'a>> {
        let mut ids = BTreeSet::new();
        let mut outputs = BTreeSet::new();
        let mut examples = Vec::new();
        for (index, block) in blocks.iter().enumerate() {
            let words: Vec<_> = block.info.split_whitespace().collect();
            if words.starts_with(&["sh", "agent-example"]) {
                assert_eq!(words.len(), 4, "Invalid example fence: {}", block.info);
                let id = attribute(words[2], "id=", block.info);
                assert!(ids.insert(id), "Duplicate guide example id: {id}");
                let exit = attribute(words[3], "exit=", block.info)
                    .parse()
                    .unwrap_or_else(|error| panic!("Example {id}: invalid exit code: {error}"));
                let output = blocks
                    .get(index + 1)
                    .unwrap_or_else(|| panic!("Example {id}: missing following output fence"));
                assert_eq!(
                    output.info,
                    format!("text agent-output id={id}"),
                    "Example {id}: next fence must be its matching output"
                );
                examples.push(Example {
                    id,
                    exit,
                    script: &block.body,
                    expected: &output.body,
                });
            } else if words.starts_with(&["text", "agent-output"]) {
                assert_eq!(words.len(), 3, "Invalid output fence: {}", block.info);
                let id = attribute(words[2], "id=", block.info);
                assert!(outputs.insert(id), "Duplicate guide output id: {id}");
                assert!(
                    examples.last().is_some_and(|example| example.id == id)
                        && index > 0
                        && blocks[index - 1].info.starts_with("sh agent-example "),
                    "Orphan guide output: {id}"
                );
            }
        }
        assert_eq!(ids, outputs, "Guide example/output ids differ");
        assert!(
            examples.len() >= 15,
            "Guide must retain at least 15 examples"
        );
        examples
    }

    struct Session(PathBuf);

    impl Session {
        fn new() -> Self {
            let path = Path::new(env!("CARGO_TARGET_TMPDIR"))
                .join(format!("agent-guide-{}", std::process::id()));
            match fs::symlink_metadata(&path) {
                Ok(metadata) => {
                    assert!(
                        metadata.is_dir(),
                        "Session path is not a directory: {path:?}"
                    );
                    fs::remove_dir_all(&path).expect("Clear previous agent-guide session");
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => panic!("Cannot inspect session directory {path:?}: {error}"),
            }
            fs::create_dir_all(&path).expect("Create agent-guide session directory");
            Self(path)
        }
    }

    impl Drop for Session {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn guide_examples_run_and_match_output() {
        let blocks = fences(GUIDE);
        let examples = parse_examples(&blocks);
        let version = describe()["engineVersion"]
            .as_str()
            .expect("describe.engineVersion must be a string");
        let session = Session::new();
        let bin_dir = Path::new(env!("CARGO_BIN_EXE_mamut-engine"))
            .parent()
            .expect("Engine binary must have a parent directory");
        let mut paths = vec![bin_dir.to_path_buf()];
        if let Some(inherited) = std::env::var_os("PATH") {
            paths.extend(std::env::split_paths(&inherited));
        }
        let path = std::env::join_paths(paths).expect("Prepend engine binary directory to PATH");
        let mut failures = Vec::new();
        for example in examples {
            let expected = example.expected.replace("<VERSION>", version);
            let mut command = Command::new("sh");
            clean_environment(&mut command)
                .args(["-c", example.script])
                .env("PATH", &path)
                .env("TMPDIR", &session.0);
            let (output, timed_out) = run_with_deadline(&mut command, example.id);
            if timed_out
                || output.status.code() != Some(example.exit)
                || std::str::from_utf8(&output.stdout) != Ok(expected.as_str())
            {
                failures.push(format!(
                    "{}\nexpected exit {}; 120 s deadline exceeded: {timed_out}",
                    diagnostics(example.id, &output, &expected),
                    example.exit
                ));
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n\n"));
    }
}

fn heading_slug(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, ' ' | '-' | '_'))
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

#[test]
fn guide_relative_links_resolve() {
    let lines = prose_lines(GUIDE);
    let slugs: BTreeSet<_> = lines
        .iter()
        .filter_map(|line| {
            let heading = line.trim_start_matches('#');
            (heading.len() != line.len() && heading.starts_with(' '))
                .then(|| heading_slug(heading.trim()))
        })
        .collect();
    for line in lines {
        for after in line.split("](").skip(1) {
            let destination = after
                .split_once(')')
                .unwrap_or_else(|| panic!("Unclosed Markdown link: {line}"))
                .0;
            if destination.starts_with("http") || destination.starts_with("mailto") {
                continue;
            }
            if let Some(anchor) = destination.strip_prefix('#') {
                assert!(
                    slugs.contains(anchor),
                    "Missing guide heading: {destination}"
                );
            } else {
                let relative = destination.split(['#', '?']).next().expect("Link path");
                let path = repo_root().join("docs").join(relative);
                assert!(path.exists(), "Broken guide link {destination}: {path:?}");
            }
        }
    }
}

#[test]
fn guide_exit_code_table_matches_describe() {
    let rows = table_rows("## 5. Exit codes and errors");
    let mut exits = BTreeMap::new();
    let mut errors = BTreeMap::new();
    for row in rows {
        let first = row.first().expect("Exit table row must have a first cell");
        if !first.is_empty() && first.chars().all(|c| c.is_ascii_digit()) {
            let name = row
                .get(1)
                .and_then(|cell| code_cell(cell))
                .unwrap_or_else(|| panic!("Missing backtick name in exit row: {row:?}"));
            assert!(
                exits.insert((*first).to_owned(), name.to_owned()).is_none(),
                "Duplicate exit code: {first}"
            );
        } else if let Some(id) = code_cell(first).filter(|id| id.starts_with("CLI-E-")) {
            let code: i64 = row
                .get(1)
                .expect("CLI error row must have an exit code")
                .parse()
                .unwrap_or_else(|error| panic!("Invalid exit code for {id}: {error}"));
            assert!(
                errors.insert(id.to_owned(), code).is_none(),
                "Duplicate {id}"
            );
        }
    }
    let described: BTreeMap<String, String> = describe()["exitCodes"]
        .as_object()
        .expect("describe.exitCodes must be an object")
        .iter()
        .map(|(code, name)| {
            (
                code.clone(),
                name.as_str()
                    .expect("Exit name must be a string")
                    .to_owned(),
            )
        })
        .collect();
    assert_eq!(exits, described, "Guide exit table differs from describe");
    let registry: Value = serde_json::from_str(ERROR_CODES).expect("Parse error-codes-1.json");
    let mut registered = BTreeMap::new();
    for entry in registry["codes"].as_array().expect("Registry codes array") {
        let id = entry["id"].as_str().expect("Registry code id string");
        if id.starts_with("CLI-E-") {
            let code = entry["exitCode"].as_i64().expect("Registry exit integer");
            assert!(
                registered.insert(id.to_owned(), code).is_none(),
                "Duplicate registry {id}"
            );
        }
    }
    assert_eq!(
        errors, registered,
        "Guide CLI error table differs from registry"
    );
}

fn limit_value(cell: &str, name: &str) -> u64 {
    let number = cell.split('(').next().expect("Limit value cell").trim();
    let digits: String = number
        .chars()
        .filter(|c| !c.is_whitespace() && !matches!(c, ',' | '.' | '_'))
        .collect();
    digits
        .parse()
        .unwrap_or_else(|error| panic!("Invalid guide value for {name}: {cell:?}: {error}"))
}

#[test]
fn guide_limits_table_matches_describe() {
    let limits = describe()["limits"]
        .as_object()
        .expect("describe.limits must be an object");
    let mut listed = BTreeSet::new();
    for row in table_rows("## 6. Limits") {
        let Some(name) = row.first().and_then(|cell| code_cell(cell)) else {
            continue;
        };
        if !name.starts_with("MAX_") || name.contains('*') {
            continue;
        }
        assert!(listed.insert(name), "Duplicate guide limit: {name}");
        let value = limit_value(row.get(1).expect("Limit value column"), name);
        let expected = limits
            .get(name)
            .and_then(Value::as_u64)
            .unwrap_or_else(|| panic!("Guide limit {name} missing or not numeric in describe"));
        assert_eq!(value, expected, "Guide limit differs: {name}");
    }
    for name in limits.keys() {
        assert!(
            name.starts_with("MAX_CONSOLIDATION") || listed.contains(name.as_str()),
            "Limit missing from guide: {name}"
        );
    }
}

#[test]
fn guide_model_exists_and_roof_matches_hand_check() {
    let input = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/examples/agent-cli/two-storey-frame.json");
    assert!(input.is_file(), "Guide model missing: {input:?}");
    let document = binary_json(&["run"], Some(&input));
    let result = &document["engineResults"][0];
    let nodes = result["nodes"].as_array().expect("Result nodes array");
    let elevations: Vec<_> = nodes
        .iter()
        .map(|node| {
            (
                node["id"].as_u64().expect("Model node integer id"),
                node["coordinates"][1]
                    .as_f64()
                    .expect("Model node Y coordinate"),
            )
        })
        .collect();
    let top = elevations
        .iter()
        .map(|(_, y)| *y)
        .reduce(f64::max)
        .expect("Guide model must have nodes");
    let roof: BTreeSet<_> = elevations
        .iter()
        .filter(|(_, y)| *y == top)
        .map(|(id, _)| *id)
        .collect();
    assert!(
        roof.contains(&5),
        "Node 5 must be on the roof at Y={top}: {roof:?}"
    );
    let wind = result["cases"]
        .as_array()
        .expect("Result cases array")
        .iter()
        .find(|case| case["case"] == "wind")
        .expect("Guide result must contain the wind case");
    let node = wind["nodes"]
        .as_array()
        .expect("Wind case nodes array")
        .iter()
        .find(|node| node["id"] == 5)
        .expect("Wind case must contain roof node 5");
    let ux = node["displacements"][0]
        .as_f64()
        .expect("Roof node 5 must have numeric Ux");
    let estimate = 5.5827e-3;
    let relative_error = (ux - estimate).abs() / estimate;
    assert!(
        relative_error < 0.01,
        "Roof node 5, wind: Ux={ux:e} m; hand estimate={estimate:e} m; relative error={relative_error:.6}, must be < 0.01"
    );
}
