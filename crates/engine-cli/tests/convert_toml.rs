// SPDX-License-Identifier: AGPL-3.0-only

//! `mamut-engine convert` for the mamut-toml/1 structural core.

#[path = "support/schema.rs"]
mod schema;

use std::io::{self, Cursor, Write};

use engine_cli::EngineCli;
use serde_json::Value;

const TOML: &str = "\
format = \"mamut-toml/1\"
units = \"SI\"

[[nodes]]
id = 1
x = 0
y = 0
z = 0
dim = \"D2\"
dofs = [\"Ux\"]

[analysis]
method = \"linearStatic\"
";

fn invoke(args: &[&str], stdin: &[u8]) -> (u8, Vec<u8>, Vec<u8>) {
    let args: Vec<String> = args.iter().copied().map(str::to_string).collect();
    let mut stdin = Cursor::new(stdin);
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let code = EngineCli::new(None).run(&args, &mut stdin, &mut stdout, &mut stderr);
    (code, stdout, stderr)
}

fn error_of(args: &[&str], stdin: &[u8]) -> (u8, Value, String, Vec<u8>) {
    let (code, stdout, stderr) = invoke(args, stdin);
    assert!(stdout.is_empty(), "stdout was not empty: {stdout:?}");
    let envelope = schema::assert_error_stream(&stderr, i32::from(code));
    let text = String::from_utf8(stderr.clone()).unwrap();
    (code, envelope, text, stderr)
}

#[test]
fn convert_round_trip_and_direction_flags() {
    let (code, json, stderr) = invoke(
        &["--timeout", "30", "convert", "--input", "-"],
        TOML.as_bytes(),
    );
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
    assert_eq!(json.last(), Some(&b'\n'));
    let value: Value = serde_json::from_slice(&json[..json.len() - 1]).unwrap();
    assert_eq!(value["schema"], "engine-input/1");
    assert_eq!(value["model"]["nodes"][0]["id"], 1);
    assert!(value["model"]["masses"].as_array().unwrap().is_empty());
    assert!(value.get("prescribed").is_none());

    let (forced, forced_json, forced_err) = invoke(
        &["convert", "--to", "json", "--input", "-"],
        TOML.as_bytes(),
    );
    assert_eq!(forced, 0);
    assert!(forced_err.is_empty());
    assert_eq!(forced_json, json);

    let (toml_code, toml, toml_err) = invoke(&["convert", "--input", "-"], &json);
    assert_eq!(toml_code, 0);
    assert!(toml_err.is_empty());
    assert!(std::str::from_utf8(&toml)
        .unwrap()
        .starts_with("format = \"mamut-toml/1\"\n"));

    let (explicit, explicit_toml, explicit_err) =
        invoke(&["convert", "--to", "toml", "--input", "-"], &json);
    assert_eq!(explicit, 0);
    assert!(explicit_err.is_empty());
    assert_eq!(explicit_toml, toml);

    let (back, back_json, back_err) = invoke(&["convert", "--to", "json", "--input", "-"], &toml);
    assert_eq!(back, 0);
    assert!(back_err.is_empty());
    assert_eq!(
        engine_api::input_sha256(std::str::from_utf8(&json).unwrap().trim()).unwrap(),
        engine_api::input_sha256(std::str::from_utf8(&back_json).unwrap().trim()).unwrap()
    );
}

#[test]
fn convert_accepts_mass_load_spring_and_prescribed() {
    let source = format!(
        "{TOML}[[masses]]\nid = \"m\"\nnode = 1\nUx = 2\n[[springs]]\nid = 4\nnodes = [1]\nkind = \"spring.linear\"\ndof = \"Ux\"\nk = 5\n[[prescribed]]\nnode = 1\ndof = \"Ux\"\nvalue = 0\n[[diaphragms]]\nmaster = 1\nslaves = [1]\n[[cases]]\nname = \"D\"\n[[cases.loads]]\nnode = 1\nFx = 3\n[combinations.D]\nD = 1\n"
    );
    let (code, json, stderr) = invoke(&["convert", "--input", "-"], source.as_bytes());
    assert_eq!(code, 0, "{}", String::from_utf8_lossy(&stderr));
    assert!(stderr.is_empty());
    let value: Value = serde_json::from_slice(&json[..json.len() - 1]).unwrap();
    assert_eq!(value["model"]["masses"][0]["values"][0]["dof"], "Ux");
    assert_eq!(value["model"]["springs"][0]["kind"], "spring.linear");
    assert_eq!(value["model"]["prescribed"][0]["value"], 0);
    assert_eq!(value["model"]["diaphragms"][0]["master"], 1);
    assert_eq!(value["cases"][0]["loads"][0]["values"][0]["value"], 3);
    assert_eq!(value["combinations"][0]["id"], "D");

    let (toml_code, toml, toml_err) = invoke(&["convert", "--to", "toml", "--input", "-"], &json);
    assert_eq!(toml_code, 0, "{}", String::from_utf8_lossy(&toml_err));
    let (back, back_json, back_err) = invoke(&["convert", "--to", "json", "--input", "-"], &toml);
    assert_eq!(back, 0, "{}", String::from_utf8_lossy(&back_err));
    assert_eq!(
        engine_api::input_sha256(std::str::from_utf8(&json).unwrap().trim()).unwrap(),
        engine_api::input_sha256(std::str::from_utf8(&back_json).unwrap().trim()).unwrap()
    );
}

#[test]
fn usage_document_and_unsupported_keep_the_registered_contract() {
    let (code, envelope, _, _) = error_of(&["convert"], b"");
    assert_eq!(code, 2);
    assert_eq!(envelope["code"], "CLI-E-USAGE");
    assert_eq!(envelope["operation"], "cli");
    assert!(envelope.get("engineResults").is_none());

    for args in [
        ["convert", "--input=x"].as_slice(),
        ["convert", "--to", "yaml", "--input", "-"].as_slice(),
        ["convert", "--", "--input", "-"].as_slice(),
        ["convert", "--input", "-file"].as_slice(),
        ["convert", "--input", "-", "--input", "-"].as_slice(),
    ] {
        let (code, envelope, _, _) = error_of(args, b"");
        assert_eq!(code, 2);
        assert_eq!(envelope["code"], "CLI-E-USAGE");
        assert_eq!(envelope["operation"], "cli");
    }

    let missing = std::env::temp_dir().join("SENTINEL_MISSING_9f3a.toml");
    let _ = std::fs::remove_file(&missing);
    let (code, envelope, text, _) = error_of(
        &[
            "convert",
            "--input",
            missing.to_str().expect("temp path is unicode"),
        ],
        b"",
    );
    assert_eq!(code, 2);
    assert_eq!(envelope["code"], "CLI-E-DOCUMENT");
    assert_eq!(envelope["operation"], "cli");
    assert!(!text.contains("SENTINEL_MISSING_9f3a"));
    assert!(envelope.get("engineResults").is_none());

    let (code, envelope, text, _) = error_of(&["convert", "--input", "-"], &[0xff]);
    assert_eq!(code, 2);
    assert_eq!(envelope["code"], "CLI-E-DOCUMENT");
    assert_eq!(
        envelope["diagnostics"][0]["cause"],
        "The document is not valid UTF-8."
    );
    assert!(!text.contains("\\xff") && !text.contains("0xff"));

    let bad = "format = \"mamut-toml/2\"\nunits = \"SI\"\n[analysis]\nmethod = \"linearStatic\"\n";
    let (code, envelope, _, _) = error_of(&["convert", "--input", "-"], bad.as_bytes());
    assert_eq!(code, 3);
    assert_eq!(envelope["code"], "CLI-E-UNSUPPORTED");
    assert_eq!(envelope["outcome"], "unsupported");
    assert_eq!(envelope["operation"], "cli");
    assert!(envelope.get("engineResults").is_none());
}

#[test]
fn stderr_does_not_repeat_sentinels() {
    let bodies = [
        "SENTINEL_KEY_9f3a = 1\n",
        "[engine]\nraw = \"SENTINEL_ENGINE_9f3a\"\n",
        "[[nodes]]\nid = \"SENTINEL_TYPE_9f3a\"\nx = 0\ny = 0\ndim = \"D2\"\ndofs = [\"Ux\"]\n",
        "[[nodes]]\nid = 1\nx = 0\ny = 0\nz = 0\ndim = \"D2\"\ndofs = [\"Ux\"]\n[[supports]]\nnode = 1\ntype = \"SENTINEL_PIN_9f3a\"\n",
        "SENTINEL_DUPKEY_9f3a = 1\nSENTINEL_DUPKEY_9f3a = 2\n",
    ];
    for body in bodies {
        let source = format!(
            "format = \"mamut-toml/1\"\nunits = \"SI\"\n{body}\n[analysis]\nmethod = \"linearStatic\"\n"
        );
        let (code, envelope, text, _) = error_of(&["convert", "--input", "-"], source.as_bytes());
        assert_eq!(code, 2, "{text}");
        assert_eq!(envelope["operation"], "cli");
        assert!(envelope.get("engineResults").is_none());
        for sentinel in [
            "SENTINEL_KEY_9f3a",
            "SENTINEL_ENGINE_9f3a",
            "SENTINEL_TYPE_9f3a",
            "SENTINEL_PIN_9f3a",
            "SENTINEL_DUPKEY_9f3a",
        ] {
            if body.contains(sentinel) {
                assert!(!text.contains(sentinel), "{text}");
            }
        }
    }
}

#[test]
fn write_failure_is_internal_and_other_commands_stay_put() {
    struct BrokenWriter;
    impl Write for BrokenWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::ErrorKind::BrokenPipe.into())
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut stderr = Vec::new();
    let code = EngineCli::new(None).run(
        &["convert".into(), "--input".into(), "-".into()],
        &mut Cursor::new(TOML.as_bytes()),
        &mut BrokenWriter,
        &mut stderr,
    );
    assert_eq!(code, 70);
    let envelope = schema::assert_error_stream(&stderr, 70);
    assert_eq!(envelope["code"], "CLI-E-INTERNAL");
    assert_eq!(envelope["operation"], "cli");
    assert_eq!(
        envelope["diagnostics"][0]["cause"],
        "The output could not be written."
    );

    let (code, stdout, stderr) = invoke(&["describe", "--json"], b"");
    assert_eq!(code, 0);
    assert!(stderr.is_empty());
    assert!(stdout.starts_with(b"{"));
    let description: Value = serde_json::from_slice(&stdout[..stdout.len() - 1]).unwrap();
    assert_eq!(description["engineId"], "mamut-engine");

    let (code, stdout, _) = invoke(&["validate", "--input", "-"], TOML.as_bytes());
    assert_ne!(code, 0);
    assert!(!String::from_utf8_lossy(&stdout).contains("\"schema\":\"engine-input/1\""));
}
