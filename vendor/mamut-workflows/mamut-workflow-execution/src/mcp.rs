// SPDX-License-Identifier: Apache-2.0

//! MCP server over stdio.
//!
//! The server speaks newline-delimited JSON-RPC 2.0 over stdio and
//! `serde_json`. It keeps `list_methods` and `compile_workflow`, and publishes
//! one tool per registered method. A method tool's input schema is that
//! method's consumed contract plus its numeric parameters. Calling it runs
//! that one method through [`crate::run_one_method`]. It does not open a socket.

use std::io::{self, BufRead, Read, Write};

use mamut_workflow::{MethodCatalog, MethodDescriptor, NumericParameters, RegisteredCatalog};
use serde_json::{json, Map, Value};

use crate::{
    cli_run_failure, cli_run_success, registered::run_one_method, CliIdentity, ExitCodes,
    MethodCallError, MAX_DOCUMENT_BYTES,
};

const PROTOCOL_VERSION: &str = "2024-11-05";
const SUPPORTED_VERSIONS: &[&str] = &["2024-11-05", "2025-03-26", "2025-06-18"];
const MAX_FRAME_BYTES: usize = MAX_DOCUMENT_BYTES;

/// One loaded catalog serving MCP tools.
pub struct McpServer {
    catalog: RegisteredCatalog,
    initialized: bool,
    exit_codes: ExitCodes,
}

impl McpServer {
    /// Serve `catalog` until the client initializes the session.
    pub fn new(catalog: RegisteredCatalog) -> Self {
        Self {
            catalog,
            initialized: false,
            exit_codes: ExitCodes::default(),
        }
    }

    /// Publish the host's exit tables in MCP initialize result metadata.
    ///
    /// This only advertises codes. Use [`Self::handle_with`] to supply a method
    /// execution hook that emits envelopes with the same configured codes.
    #[must_use]
    pub fn with_exit_codes(mut self, exit_codes: ExitCodes) -> Self {
        self.exit_codes = exit_codes;
        self
    }

    /// Handle one JSON-RPC message.
    ///
    /// Returns the response body. Notifications produce no response.
    #[must_use]
    pub fn handle(&mut self, message: &str) -> Option<String> {
        self.handle_with(message, &CliIdentity::default(), reference_method_result)
    }

    /// Handle a message with an engine identity and method execution hook.
    ///
    /// The hook receives a resolved descriptor and parsed numeric parameters (or
    /// their parse error). It returns a cli-run envelope. This server preserves
    /// framing, tool discovery, initialization, and protocol validation. The
    /// hook controls the execution policy and sanitization of its envelopes.
    #[must_use]
    pub fn handle_with<F>(
        &mut self,
        message: &str,
        identity: &CliIdentity,
        mut execute: F,
    ) -> Option<String>
    where
        F: FnMut(&RegisteredCatalog, &MethodDescriptor, Result<NumericParameters, String>) -> Value,
    {
        let value: Value = match serde_json::from_str(message) {
            Ok(value) => value,
            Err(_) => return Some(error_response(Value::Null, -32700, "parse error")),
        };
        let Some(object) = value.as_object() else {
            return Some(error_response(Value::Null, -32600, "invalid request"));
        };
        if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            let id = object.get("id").cloned().unwrap_or(Value::Null);
            return Some(error_response(id, -32600, "invalid request"));
        }
        let Some(method) = object.get("method").and_then(Value::as_str) else {
            let id = object.get("id").cloned().unwrap_or(Value::Null);
            return Some(error_response(id, -32600, "invalid request"));
        };
        let id = object.get("id").cloned()?;
        if id.is_null() {
            return Some(error_response(Value::Null, -32600, "invalid request"));
        }
        if method != "initialize" && !self.initialized {
            return Some(error_response(
                id,
                -32600,
                "initialize must be called first",
            ));
        }
        let params = object.get("params").cloned().unwrap_or(Value::Null);
        let result = match method {
            "initialize" => initialize(&params, identity, &self.exit_codes),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(tools_list(&self.catalog)),
            "tools/call" => self.tools_call(&params, &mut execute),
            "resources/list" => Ok(json!({"resources": []})),
            "resources/templates/list" => Ok(json!({"resourceTemplates": []})),
            _ => Err((-32601, "method not found".to_owned())),
        };
        match result {
            Ok(result) => {
                if method == "initialize" {
                    self.initialized = true;
                }
                Some(success_response(id, result))
            }
            Err((code, message)) => Some(error_response(id, code, &message)),
        }
    }

    fn tools_call<F>(&self, params: &Value, execute: &mut F) -> Result<Value, (i32, String)>
    where
        F: FnMut(&RegisteredCatalog, &MethodDescriptor, Result<NumericParameters, String>) -> Value,
    {
        let name = params
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| (-32602, "invalid params".to_owned()))?;
        let arguments = match params.get("arguments") {
            None | Some(Value::Null) => json!({}),
            Some(arguments) => arguments.clone(),
        };
        match name {
            "list_methods" => list_methods(&self.catalog, &arguments),
            "compile_workflow" => compile_workflow(&self.catalog, &arguments),
            _ => call_method(&self.catalog, name, &arguments, execute),
        }
    }
}

fn initialize(
    params: &Value,
    identity: &CliIdentity,
    exit_codes: &ExitCodes,
) -> Result<Value, (i32, String)> {
    if !params.is_object() {
        return Err((-32602, "invalid params".to_owned()));
    }
    let requested = params
        .get("protocolVersion")
        .and_then(Value::as_str)
        .unwrap_or("");
    let version = if SUPPORTED_VERSIONS.contains(&requested) {
        requested
    } else {
        PROTOCOL_VERSION
    };
    Ok(json!({
        "protocolVersion": version,
        // Keep the legacy capabilities object intact; result metadata is additive.
        "capabilities": {"tools": {"listChanged": false}},
        "_meta": {"mamut/cli-run": exit_codes.capabilities()},
        "serverInfo": {
            "name": identity.engine_id,
            "version": identity.engine_version
        }
    }))
}

fn tools_list(catalog: &RegisteredCatalog) -> Value {
    let mut tools = vec![
        json!({
            "name": "list_methods",
            "description": "List the methods registered in the catalog. Does not run a method.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
        json!({
            "name": "compile_workflow",
            "description": "Compile a workflow whose methods are already registered. Does not run a simulation.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "workflow": {
                        "type": "object",
                        "description": "WorkflowDefinitionV1 with profile, version, and steps."
                    }
                },
                "required": ["workflow"],
                "additionalProperties": false
            }
        }),
    ];
    for descriptor in catalog.descriptors() {
        tools.push(method_tool(descriptor));
    }
    json!({ "tools": tools })
}

fn method_tool(descriptor: &MethodDescriptor) -> Value {
    json!({
        "name": method_tool_name(descriptor),
        "description": format!(
            "Run `{}` version {} on slot `{}`. One method, one line.",
            descriptor.method_id, descriptor.method_version, descriptor.capability_slot
        ),
        "inputSchema": input_schema(descriptor)
    })
}

fn method_tool_name(descriptor: &MethodDescriptor) -> String {
    format!(
        "{}.{}.{}",
        descriptor.capability_slot, descriptor.method_id, descriptor.method_version
    )
}

fn input_schema(descriptor: &MethodDescriptor) -> Value {
    let mut properties = Map::new();
    let mut required = Vec::new();
    for parameter in &descriptor.parameters {
        properties.insert(
            parameter.name.clone(),
            json!({
                "type": "number",
                "description": format!(
                    "dimension {}, unit {}",
                    parameter.dimension, parameter.unit
                ),
            }),
        );
        if parameter.required {
            required.push(Value::String(parameter.name.clone()));
        }
    }
    let consumes = descriptor
        .consumes
        .iter()
        .map(|contract| contract.urn())
        .collect::<Vec<_>>()
        .join(", ");
    let produces = descriptor
        .produces
        .iter()
        .map(|contract| contract.urn())
        .collect::<Vec<_>>()
        .join(", ");
    let mut schema = json!({
        "type": "object",
        "description": format!("Consumes {consumes}. Produces {produces}."),
        "properties": properties,
        "required": required,
        "additionalProperties": false
    });
    if let [contract] = descriptor.consumes.as_slice() {
        schema["$id"] = json!(contract.urn());
    }
    schema
}

fn call_method<F>(
    catalog: &RegisteredCatalog,
    name: &str,
    arguments: &Value,
    execute: &mut F,
) -> Result<Value, (i32, String)>
where
    F: FnMut(&RegisteredCatalog, &MethodDescriptor, Result<NumericParameters, String>) -> Value,
{
    let descriptor = catalog
        .descriptors()
        .into_iter()
        .find(|descriptor| method_tool_name(descriptor) == name)
        .cloned();
    let Some(descriptor) = descriptor else {
        return Err((-32602, format!("unknown tool `{name}`")));
    };
    let envelope = execute(catalog, &descriptor, numeric_parameters(arguments));
    let is_error = envelope.get("error").is_some();
    Ok(run_tool_result(envelope, is_error))
}

fn reference_method_result(
    catalog: &RegisteredCatalog,
    descriptor: &MethodDescriptor,
    parameters: Result<NumericParameters, String>,
) -> Value {
    let parameters = match parameters {
        Ok(parameters) => parameters,
        Err(_) => return cli_run_failure(3),
    };
    match run_one_method(catalog, descriptor, parameters) {
        Ok(manifest) => cli_run_success(&manifest),
        Err(error) => {
            let code = match error {
                MethodCallError::InvalidInput(_) => 4,
                MethodCallError::MethodFailed(_) => 5,
                MethodCallError::Internal(_) => 10,
            };
            cli_run_failure(code)
        }
    }
}

fn numeric_parameters(arguments: &Value) -> Result<NumericParameters, String> {
    let Some(object) = arguments.as_object() else {
        return Err("arguments must be an object".to_owned());
    };
    let mut parameters = NumericParameters::new();
    for (name, value) in object {
        let Some(number) = value.as_f64() else {
            return Err(format!("parameter `{name}` is not a number"));
        };
        if !number.is_finite() {
            return Err(format!("parameter `{name}` is not a finite number"));
        }
        parameters.insert(name.clone(), number);
    }
    Ok(parameters)
}

fn list_methods(catalog: &RegisteredCatalog, arguments: &Value) -> Result<Value, (i32, String)> {
    let Some(object) = arguments.as_object() else {
        return Err((-32602, "invalid params".to_owned()));
    };
    if !object.is_empty() {
        return Err((-32602, "invalid params".to_owned()));
    }
    let text = catalog
        .methods_json()
        .map_err(|_| (-32603, "Execution could not be completed".to_owned()))?;
    Ok(tool_result(text, false))
}

fn compile_workflow(
    catalog: &RegisteredCatalog,
    arguments: &Value,
) -> Result<Value, (i32, String)> {
    let Some(object) = arguments.as_object() else {
        return Err((-32602, "invalid params".to_owned()));
    };
    let Some(workflow) = object.get("workflow") else {
        return Err((-32602, "invalid params".to_owned()));
    };
    if object.len() != 1 {
        return Err((-32602, "invalid params".to_owned()));
    }
    let text =
        serde_json::to_string(workflow).map_err(|_| (-32602, "invalid params".to_owned()))?;
    match catalog.compile_json(&text) {
        Ok(compiled) => {
            let text = serde_json::to_string(&compiled)
                .map_err(|_| (-32603, "Execution could not be completed".to_owned()))?;
            Ok(tool_result(text, false))
        }
        Err(_) => Ok(tool_result(
            "Workflow does not match the catalog".to_owned(),
            true,
        )),
    }
}

fn tool_result(text: String, is_error: bool) -> Value {
    json!({
        "content": [{"type": "text", "text": text}],
        "isError": is_error
    })
}

fn run_tool_result(result: Value, is_error: bool) -> Value {
    json!({
        "content": [{"type": "text", "text": result.to_string()}],
        "structuredContent": result,
        "isError": is_error
    })
}

/// Stdio encoding. New clients use JSON lines; old callers can retain their
/// Content-Length encoding without changing the data returned by the tools.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StdioFraming {
    /// Standard MCP stdio: one UTF-8 JSON message followed by LF.
    JsonLines,
    /// Legacy framing used by earlier versions of this crate.
    ContentLength,
}

/// Read a standard MCP message, or a legacy Content-Length frame.
/// Both encodings enforce the same 16 MiB message limit.
pub fn read_stdio_message(
    reader: &mut impl BufRead,
) -> io::Result<Option<(Vec<u8>, StdioFraming)>> {
    let prefix = reader.fill_buf()?;
    if prefix.is_empty() {
        return Ok(None);
    }
    if matches!(prefix[0], b'C' | b'c') {
        return read_frame(reader).map(|body| body.map(|body| (body, StdioFraming::ContentLength)));
    }
    let mut body = Vec::new();
    reader
        .take((MAX_FRAME_BYTES + 2) as u64)
        .read_until(b'\n', &mut body)?;
    if body.last() != Some(&b'\n') {
        return Err(invalid("stdio message exceeds limit or has no newline"));
    }
    body.pop();
    if body.last() == Some(&b'\r') {
        body.pop();
    }
    if body.len() > MAX_FRAME_BYTES {
        return Err(invalid("stdio message exceeds 16 MiB"));
    }
    Ok(Some((body, StdioFraming::JsonLines)))
}

/// Write a response using the request's encoding and flush stdout.
pub fn write_stdio_message(
    writer: &mut impl Write,
    body: &[u8],
    framing: StdioFraming,
) -> io::Result<()> {
    if framing == StdioFraming::ContentLength {
        return write_frame(writer, body);
    }
    if body.len() > MAX_FRAME_BYTES || body.contains(&b'\n') || body.contains(&b'\r') {
        return Err(invalid("invalid JSON-line message"));
    }
    writer.write_all(body)?;
    writer.write_all(b"\n")?;
    writer.flush()
}

fn success_response(id: Value, result: Value) -> String {
    serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    }))
    .expect("response json is finite")
}

fn error_response(id: Value, code: i32, message: &str) -> String {
    serde_json::to_string(&json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {"code": code, "message": message}
    }))
    .expect("error json is finite")
}

/// Read one `Content-Length` frame.
///
/// `Ok(None)` is end of input before a frame starts.
pub fn read_frame(reader: &mut impl BufRead) -> io::Result<Option<Vec<u8>>> {
    let mut content_length = None;
    loop {
        let mut line = String::new();
        let read = reader.read_line(&mut line)?;
        if read == 0 {
            if content_length.is_none() && line.is_empty() {
                return Ok(None);
            }
            return Err(invalid("frame ended before the header terminator"));
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            break;
        }
        let Some((name, value)) = trimmed.split_once(':') else {
            return Err(invalid("frame header has no colon"));
        };
        if name.eq_ignore_ascii_case("Content-Length") {
            let length: usize = value
                .trim()
                .parse()
                .map_err(|_| invalid("Content-Length is not a number"))?;
            if length > MAX_FRAME_BYTES {
                return Err(invalid("Content-Length exceeds 16 MiB"));
            }
            content_length = Some(length);
        }
    }
    let Some(length) = content_length else {
        return Err(invalid("frame has no Content-Length"));
    };
    let mut body = vec![0; length];
    reader.read_exact(&mut body)?;
    Ok(Some(body))
}

/// Write one `Content-Length` frame and flush it.
pub fn write_frame(writer: &mut impl Write, body: &[u8]) -> io::Result<()> {
    if body.len() > MAX_FRAME_BYTES {
        return Err(invalid("frame exceeds 16 MiB"));
    }
    write!(writer, "Content-Length: {}\r\n\r\n", body.len())?;
    writer.write_all(body)?;
    writer.flush()
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use serde_json::json;

    use super::{
        read_frame, read_stdio_message, write_frame, write_stdio_message, McpServer, StdioFraming,
        MAX_FRAME_BYTES,
    };
    use crate::ExitCodes;
    use mamut_workflow::RegisteredCatalog;

    const CATALOG: &str = r#"{
      "profile": {
        "profile": "sequence.v1",
        "version": 1,
        "slots": [{
          "capability_slot": "only",
          "dimensions": {
            "inputs": [{"name": "in", "dimension": "L", "unit": "m"}],
            "outputs": [{"name": "out", "dimension": "L", "unit": "m"}]
          },
          "consumes": ["urn:mamut:sequence:stage:1.0.0"],
          "produces": ["urn:mamut:sequence:stage:1.0.0"]
        }]
      },
      "methods": [{
        "method_id": "method-a",
        "method_version": 1,
        "capability_slot": "only",
        "dimensions": {
          "inputs": [{"name": "in", "dimension": "L", "unit": "m"}],
          "outputs": [{"name": "out", "dimension": "L", "unit": "m"}]
        },
        "consumes": ["urn:mamut:sequence:stage:1.0.0"],
        "produces": ["urn:mamut:sequence:stage:1.0.0"],
        "parameters": []
      }]
    }"#;

    fn server() -> McpServer {
        McpServer::new(RegisteredCatalog::from_json(CATALOG).unwrap())
    }

    fn request(id: u32, method: &str, params: serde_json::Value) -> String {
        json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string()
    }

    #[test]
    fn direct_server_publishes_default_and_configured_exit_tables() {
        for codes in [ExitCodes::default(), ExitCodes::engine_v1()] {
            let mut server = if codes == ExitCodes::default() {
                server()
            } else {
                server().with_exit_codes(codes.clone())
            };
            let response = server.handle(&request(1, "initialize", json!({}))).unwrap();
            let response: serde_json::Value = serde_json::from_str(&response).unwrap();
            let tables = &response["result"]["_meta"]["mamut/cli-run"];
            assert_eq!(*tables, codes.capabilities());
            assert_eq!(tables["exit_code_categories"]["cancelled"], codes.cancelled);
            assert_eq!(
                tables["reserved_exit_code_categories"]["timeout"],
                codes.timeout
            );
            assert!(tables["exit_code_categories"].get("timeout").is_none());
        }
    }

    #[test]
    fn frame_round_trip_counts_bytes() {
        let body = "{\"name\":\"métier\"}".as_bytes();
        let mut encoded = Vec::new();
        write_frame(&mut encoded, body).unwrap();
        let mut cursor = Cursor::new(encoded);
        let decoded = read_frame(&mut cursor).unwrap().unwrap();
        assert_eq!(decoded, body);
        assert!(read_frame(&mut cursor).unwrap().is_none());
    }

    #[test]
    fn json_lines_preserve_utf8_and_separate_messages() {
        let body = "{\"name\":\"métier\"}".as_bytes();
        let mut encoded = Vec::new();
        write_stdio_message(&mut encoded, body, StdioFraming::JsonLines).unwrap();
        write_stdio_message(&mut encoded, b"{}", StdioFraming::JsonLines).unwrap();
        let mut cursor = Cursor::new(encoded);
        assert_eq!(
            read_stdio_message(&mut cursor).unwrap(),
            Some((body.to_vec(), StdioFraming::JsonLines))
        );
        assert_eq!(
            read_stdio_message(&mut cursor).unwrap(),
            Some((b"{}".to_vec(), StdioFraming::JsonLines))
        );
        assert!(read_stdio_message(&mut cursor).unwrap().is_none());
    }

    #[test]
    fn stdio_rejects_truncated_or_oversized_messages() {
        assert!(read_stdio_message(&mut Cursor::new(b"{}")).is_err());
        assert!(read_stdio_message(&mut Cursor::new(vec![b' '; MAX_FRAME_BYTES + 2])).is_err());
        assert!(write_stdio_message(&mut Vec::new(), b"{\n}", StdioFraming::JsonLines).is_err());
        assert!(read_stdio_message(&mut Cursor::new(b"Content-Length: 16777217\r\n\r\n")).is_err());
    }

    #[test]
    fn lists_and_compiles_after_initialize() {
        let mut server = server();
        assert!(server
            .handle(&request(1, "tools/list", json!({})))
            .unwrap()
            .contains("initialize must be called first"));

        let initialized = server
            .handle(&request(
                2,
                "initialize",
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "test", "version": "0"}
                }),
            ))
            .unwrap();
        let initialized: serde_json::Value = serde_json::from_str(&initialized).unwrap();
        assert_eq!(initialized["result"]["protocolVersion"], "2024-11-05");
        assert_eq!(
            initialized["result"]["serverInfo"]["name"],
            "mamut-workflow"
        );

        assert!(server
            .handle(r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#)
            .is_none());

        let tools = server.handle(&request(3, "tools/list", json!({}))).unwrap();
        let tools: serde_json::Value = serde_json::from_str(&tools).unwrap();
        let names: Vec<&str> = tools["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .map(|tool| tool["name"].as_str().unwrap())
            .collect();
        assert_eq!(
            names,
            ["list_methods", "compile_workflow", "only.method-a.1"]
        );
        let method_tool = &tools["result"]["tools"][2];
        assert_eq!(
            method_tool["inputSchema"]["$id"],
            "urn:mamut:sequence:stage:1.0.0"
        );
        assert_eq!(
            method_tool["inputSchema"]["description"],
            "Consumes urn:mamut:sequence:stage:1.0.0. Produces urn:mamut:sequence:stage:1.0.0."
        );

        let listed = server
            .handle(&request(
                4,
                "tools/call",
                json!({"name": "list_methods", "arguments": {}}),
            ))
            .unwrap();
        let listed: serde_json::Value = serde_json::from_str(&listed).unwrap();
        assert_eq!(listed["result"]["isError"], false);
        let text = listed["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("method-a"));

        let compiled = server
            .handle(&request(
                5,
                "tools/call",
                json!({
                    "name": "compile_workflow",
                    "arguments": {
                        "workflow": {
                            "profile": "sequence.v1",
                            "version": 1,
                            "steps": [{
                                "capability_slot": "only",
                                "method_id": "method-a",
                                "method_version": 1,
                                "parameters": {}
                            }]
                        }
                    }
                }),
            ))
            .unwrap();
        let compiled: serde_json::Value = serde_json::from_str(&compiled).unwrap();
        assert_eq!(compiled["result"]["isError"], false);
        let text = compiled["result"]["content"][0]["text"].as_str().unwrap();
        let workflow: serde_json::Value = serde_json::from_str(text).unwrap();
        assert_eq!(workflow["steps"][0]["method_id"], "method-a");
        assert_eq!(workflow["steps"][0]["instance"]["capability_slot"], "only");

        let rejected = server
            .handle(&request(
                6,
                "tools/call",
                json!({
                    "name": "compile_workflow",
                    "arguments": {
                        "workflow": {
                            "profile": "sequence.v1",
                            "version": 1,
                            "steps": [{
                                "capability_slot": "only",
                                "method_id": "missing",
                                "method_version": 1,
                                "parameters": {}
                            }]
                        }
                    }
                }),
            ))
            .unwrap();
        let rejected: serde_json::Value = serde_json::from_str(&rejected).unwrap();
        assert_eq!(rejected["result"]["isError"], true);
        assert_eq!(
            rejected["result"]["content"][0]["text"],
            "Workflow does not match the catalog"
        );

        let called = server
            .handle(&request(
                7,
                "tools/call",
                json!({"name": "only.method-a.1", "arguments": {}}),
            ))
            .unwrap();
        let called: serde_json::Value = serde_json::from_str(&called).unwrap();
        assert_eq!(called["result"]["isError"], false);
        let manifest: serde_json::Value =
            serde_json::from_str(called["result"]["content"][0]["text"].as_str().unwrap()).unwrap();
        assert_eq!(
            manifest["data"]["id"],
            "urn:mamut:workflow:execution-manifest:1.0.0"
        );
        assert_eq!(manifest["data"]["steps"][0]["method"]["id"], "method-a");
        assert_eq!(manifest["data"]["recordFingerprint"]["algorithm"], "sha256");

        let bad = server
            .handle(&request(
                8,
                "tools/call",
                json!({"name": "only.method-a.1", "arguments": {"extra": 1}}),
            ))
            .unwrap();
        let bad: serde_json::Value = serde_json::from_str(&bad).unwrap();
        assert_eq!(bad["result"]["isError"], true);
        let text = bad["result"]["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("invalid_input"));
        assert!(!text.contains("execution-manifest"));
        assert!(!text.contains("fingerprint"));
    }
}
