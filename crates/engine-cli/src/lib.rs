// SPDX-License-Identifier: AGPL-3.0-only

//! Adaptador nativo de engine-api sobre el parser y despacho de mamut-workflow-cli.
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::{
    cell::RefCell,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

use engine_api::{ApiError, CancelToken, Diagnostic, Progress, Status};
use mamut_workflow::{CompiledStep, MethodDescriptor, RegisteredMethod};
use mamut_workflow_cli::{
    dispatch, parse_args, CancelFlag, Cancellation, CliConfig, CliIdentity, Command, ExitCodes,
    FailureKind, Method, MethodProvider,
};
use serde_json::{json, Value};

mod error;
#[cfg(unix)]
mod signals;
mod timeout;
use error::report_failure;

/// Catálogo propio; también disponible como archivo `catalog.json` del crate.
pub const CATALOG: &str = include_str!("../catalog.json");

/// Workflow predeterminado de ejecución de engine-input/1.
pub const RUN_WORKFLOW: &str = include_str!("../run.workflow.json");

/// Workflow predeterminado de validación de engine-input/1.
pub const VALIDATE_WORKFLOW: &str = include_str!("../validate.workflow.json");

/// Cancelación compartida del workflow y del motor, sin señales ni hilos implícitos.
#[derive(Clone, Debug, Default)]
pub struct Control {
    flag: CancelFlag,
    token: Arc<CancelToken>,
    cancel_file: Option<PathBuf>,
}

impl Control {
    /// Observe la presencia de una ruta local como solicitud de cancelación.
    /// Se consulta entre pasos y tras cada callback del motor, sin hilos ni señales.
    #[must_use]
    pub fn with_cancel_file(mut self, path: PathBuf) -> Self {
        self.cancel_file = Some(path);
        self
    }

    /// Solicita cancelación en ambas fronteras.
    pub fn cancel(&self) {
        self.flag.cancel();
        self.token.cancel();
    }

    /// Flag de la base; se sincroniza con el token en cada callback del motor.
    pub fn flag(&self) -> CancelFlag {
        self.flag.clone()
    }

    /// Token del motor; puede compartirse con un hilo del anfitrión.
    pub fn token(&self) -> Arc<CancelToken> {
        Arc::clone(&self.token)
    }

    fn sync(&self) -> bool {
        if self.flag.is_cancelled()
            || self.token.is_cancelled()
            || self.cancel_file.as_ref().is_some_and(|path| path.exists())
        {
            self.cancel();
            true
        } else {
            false
        }
    }
}

impl Cancellation for Control {
    fn is_cancelled(&self) -> bool {
        self.sync()
    }
}

mod progress;

type EngineProgress<'a> = Rc<RefCell<Box<dyn FnMut(Progress) + 'a>>>;

/// Configuración del anfitrión, transporte de entrada y timeout sobre la base.
/// `--input` tiene prioridad sobre la ruta de compatibilidad del constructor.
/// Una invocación reutiliza un único documento engine-input/1 para todos sus pasos.
pub struct EngineCli {
    input: Option<PathBuf>,
    control: Control,
    progress: EngineProgress<'static>,
    #[cfg(unix)]
    signals: bool,
}

impl EngineCli {
    /// Configura la ruta de entrada del motor; `None` falla solo al ejecutar métodos.
    pub fn new(input: Option<PathBuf>) -> Self {
        Self {
            input,
            control: Control::default(),
            progress: Rc::new(RefCell::new(Box::new(|_| {}))),
            #[cfg(unix)]
            signals: false,
        }
    }

    /// Conecta cancelación cooperativa, incluida cancelación durante engine-api.
    #[must_use]
    pub fn with_control(mut self, control: Control) -> Self {
        self.control = control;
        self
    }

    /// Enable process-wide SIGINT/SIGTERM handlers during `run` on Unix.
    /// The binary opts in; the library installs no handlers by default.
    /// Removing registrations does not restore previous OS dispositions.
    #[cfg(unix)]
    #[must_use]
    pub fn with_signals(mut self) -> Self {
        self.signals = true;
        self
    }

    /// Observa etapas de engine-api. Puede cancelar mediante un `Control` clonado.
    /// El NDJSON incluye estas etapas entre started/finished del workflow.
    #[must_use]
    pub fn with_engine_progress(mut self, progress: impl FnMut(Progress) + 'static) -> Self {
        self.progress = Rc::new(RefCell::new(Box::new(progress)));
        self
    }

    /// Extrae timeout, luego `--input`, y configura documentos embebidos para la base.
    /// Acepta segundos finitos en (0, 31536000]; fuera de rango es uso inválido (2).
    /// Añade resultados completos y semántica Engine al sobre terminal de `run`.
    /// stdout contiene resultados; stderr contiene NDJSON y errores sanitizados.
    pub fn run<R: Read, O: Write, E: Write>(
        &self,
        args: &[String],
        stdin: &mut R,
        stdout: &mut O,
        stderr: &mut E,
    ) -> u8 {
        let operation = error::operation(args);
        let (args, timeout) = match timeout::parse_timeout_args(args) {
            Ok(parsed) => parsed,
            Err(kind) => return report_failure(kind, operation, None, stderr),
        };
        let prepared = match PreparedArgs::new(&args) {
            Ok(prepared) => prepared,
            Err(kind) => return report_failure(kind, operation, None, stderr),
        };
        if prepared.direct_validation {
            // Validate the timeout option without starting a timer on this short path.
            let input = match prepared.input.as_deref() {
                Some("-") => read_engine_input(stdin),
                Some(path) => read_engine_path(Some(Path::new(path))),
                None => Err(FailureKind::Usage),
            };
            let input = match input {
                Ok(input) => input,
                Err(kind) => return report_failure(kind, operation, None, stderr),
            };
            return self.validate_input(&input, stdout, stderr);
        }
        let command = match parse_args(&prepared.args) {
            Ok(command) => command,
            Err(error) => {
                return report_failure(error.kind, operation, None, stderr);
            }
        };
        let command = match command {
            // La forma explícita sigue validando/compilando solo el workflow.
            Command::Validate { catalog, workflow } if prepared.embedded.is_some() => {
                Command::Run {
                    catalog,
                    workflow,
                    progress_ndjson: false,
                }
            }
            command => command,
        };
        let is_run = matches!(command, Command::Run { .. });
        #[cfg(unix)]
        let _signals = if self.signals {
            match signals::Signals::start(self.control.token()) {
                Ok(signals) => Some(signals),
                Err(_) => return report_failure(FailureKind::Internal, operation, None, stderr),
            }
        } else {
            None
        };
        let timer = match timeout
            .map(|duration| timeout::Timer::start(duration, self.control.clone()))
            .transpose()
        {
            Ok(timer) => timer,
            Err(kind) => return report_failure(kind, operation, None, stderr),
        };
        let input = match prepared.input.as_deref() {
            Some(source) => {
                let document = if source == "-" {
                    read_engine_input(&mut *stdin)
                } else {
                    read_engine_path(Some(Path::new(source)))
                };
                match document {
                    Ok(document) => EngineInput::Document(Ok(Rc::from(document))),
                    Err(kind) => return report_failure(kind, operation, None, stderr),
                }
            }
            // Conservar la clasificación y el momento del fallo de la vía antigua:
            // solo se informa al ejecutar el método, después de la cancelación.
            None if is_run => {
                EngineInput::Document(read_engine_path(self.input.as_deref()).map(Rc::from))
            }
            None => EngineInput::Path(self.input.clone()),
        };
        let state = Rc::new(RefCell::new(State {
            capture_results: is_run,
            ..State::default()
        }));
        let enabled = matches!(
            command,
            Command::Run {
                progress_ndjson: true,
                ..
            }
        );
        let mut stderr = progress::Output::new(stderr, enabled);
        let config = CliConfig::new()
            .with_identity(identity())
            .with_exit_codes(ExitCodes::engine_v1())
            .with_cancellation(self.control.clone())
            .with_progress(stderr.clone())
            .with_progress_writer(stderr.clone())
            .with_provider(EngineProvider {
                input,
                control: self.control.clone(),
                progress: stderr.callback(Rc::clone(&self.progress)),
                state: Rc::clone(&state),
            });
        let mut buffered = Vec::new();
        // Solo se retiene el sobre terminal. MCP y NDJSON conservan streaming.
        let mut output: &mut dyn Write = if is_run { &mut buffered } else { stdout };
        let result = dispatch(command, stdin, &mut output, &mut stderr, &config);
        if stderr.failed() {
            return report_failure(FailureKind::Internal, operation, None, &mut stderr);
        }
        let state = state.borrow();
        let (result, document) = match result {
            Ok(()) => (Ok(()), None),
            Err(error) => {
                let (kind, document) = if error.kind == FailureKind::Method {
                    match &state.failure {
                        Some((kind, document)) => (*kind, document.as_ref()),
                        None => (error.kind, None),
                    }
                } else {
                    (error.kind, None)
                };
                (Err(kind), document)
            }
        };
        if let Err(kind) =
            timeout::resolve(result, timer.as_ref().is_some_and(timeout::Timer::expired))
        {
            let document = if kind == FailureKind::Timeout {
                None
            } else {
                document
            };
            return report_failure(kind, operation, document, &mut stderr);
        }
        if !is_run {
            return config.exit_codes.ok;
        }
        let mut envelope: Value = match serde_json::from_slice(&buffered) {
            Ok(envelope) => envelope,
            Err(_) => return report_failure(FailureKind::Internal, operation, None, &mut stderr),
        };
        let code = if state.warning {
            envelope["outcome"] = json!("warning");
            config.exit_codes.warning
        } else {
            config.exit_codes.ok
        };
        envelope["exitCode"] = json!(code);
        // No se altera el manifiesto sellado ni su recordFingerprint.
        envelope["engineResults"] = json!(state.results);
        if writeln!(stdout, "{envelope}").is_err() {
            return report_failure(FailureKind::Internal, operation, None, &mut stderr);
        }
        code
    }

    fn validate_input(&self, input: &str, stdout: &mut impl Write, stderr: &mut impl Write) -> u8 {
        if self.control.sync() {
            return report_failure(FailureKind::Cancelled, "validate", None, stderr);
        }
        let diagnostics = match engine_api::validate_with(input, &self.control.token, |event| {
            (self.progress.borrow_mut())(event);
            self.control.sync();
        }) {
            Ok(diagnostics) => diagnostics,
            Err(error @ (ApiError::InvalidInput(_) | ApiError::UnsupportedSchema(_))) => {
                vec![Diagnostic {
                    // JSON, DTO y límites son rechazos del contrato de entrada.
                    code: "ENG-E-0009",
                    severity: "error",
                    entities: Vec::new(),
                    cause: error.to_string(),
                    hint: None,
                }]
            }
            Err(error) => return report_failure(api_failure(error), "validate", None, stderr),
        };
        let (code, report) = validation_report(&diagnostics);
        if writeln!(stdout, "{report}").is_err() {
            return report_failure(FailureKind::Internal, "validate", None, stderr);
        }
        code
    }
}

fn validation_report(diagnostics: &[Diagnostic]) -> (u8, Value) {
    let (code, status) = if diagnostics.iter().any(|d| d.severity == "error") {
        (2, "invalid")
    } else if diagnostics.iter().any(|d| d.severity == "warning") {
        (1, "warning")
    } else {
        (0, "ok")
    };
    (
        code,
        json!({"schema":"validate-report/1", "operation":"validate",
        "status":status, "diagnostics":diagnostics}),
    )
}

// El SDK emite cada evento mediante writeln!. Materializar todo el formato
// antes de escribir evita exponer fragmentos JSON durante la serialización.
struct EventWriter<'a, W>(&'a mut W);

impl<W: Write> Write for EventWriter<'_, W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.write(bytes)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }

    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        let mut line = Vec::new();
        line.write_fmt(args)?;
        self.0.write_all(&line)
    }
}

struct PreparedArgs {
    args: Vec<String>,
    input: Option<String>,
    direct_validation: bool,
    // La base solo admite archivos o stdin. Mantener los documentos hasta
    // terminar dispatch; sus rutas nunca forman parte del manifiesto.
    embedded: Option<EmbeddedDocuments>,
}

impl PreparedArgs {
    fn new(args: &[String]) -> Result<Self, FailureKind> {
        if !args
            .first()
            .is_some_and(|arg| arg == "run" || arg == "validate")
        {
            return Ok(Self {
                args: args.to_vec(),
                input: None,
                direct_validation: false,
                embedded: None,
            });
        }
        let mut forwarded = vec![args[0].clone()];
        let mut input = None;
        let mut catalog = None;
        let mut workflow = None;
        let mut rest = args[1..].iter();
        while let Some(arg) = rest.next() {
            // Igual que la base: no hay sintaxis --flag=valor ni separador --.
            if arg == "--" || arg.starts_with("--input=") {
                return Err(FailureKind::Usage);
            }
            if arg == "--input" {
                if input.is_some() {
                    return Err(FailureKind::Usage);
                }
                match rest.next() {
                    Some(value)
                        if !value.is_empty() && (value == "-" || !value.starts_with('-')) =>
                    {
                        input = Some(value.clone());
                    }
                    _ => return Err(FailureKind::Usage),
                }
            } else {
                forwarded.push(arg.clone());
                // Respetar los valores de flags de la base: no reinterpretarlos.
                if matches!(arg.as_str(), "--catalog" | "--workflow" | "--progress") {
                    if let Some(value) = rest.next() {
                        if arg == "--catalog" {
                            catalog = Some(value.as_str());
                        } else if arg == "--workflow" {
                            workflow = Some(value.as_str());
                        }
                        forwarded.push(value.clone());
                    }
                }
            }
        }
        if input.as_deref() == Some("-") && (catalog == Some("-") || workflow == Some("-")) {
            return Err(FailureKind::Usage);
        }
        let direct_validation =
            args[0] == "validate" && input.is_some() && catalog.is_none() && workflow.is_none();
        if direct_validation {
            // Esta forma no admite flags del workflow ni necesita temporales.
            if forwarded.len() != 1 {
                return Err(FailureKind::Usage);
            }
            return Ok(Self {
                args: forwarded,
                input,
                direct_validation,
                embedded: None,
            });
        }
        let embedded = if catalog.is_none() && workflow.is_none() {
            let workflow = if args[0] == "validate" {
                VALIDATE_WORKFLOW
            } else {
                RUN_WORKFLOW
            };
            let embedded = EmbeddedDocuments::new(workflow)?;
            forwarded.extend([
                "--catalog".into(),
                embedded.path("catalog.json")?,
                "--workflow".into(),
                embedded.path("workflow.json")?,
            ]);
            Some(embedded)
        } else {
            None
        };
        Ok(Self {
            args: forwarded,
            input,
            direct_validation,
            embedded,
        })
    }
}

struct EmbeddedDocuments(PathBuf);

impl EmbeddedDocuments {
    fn new(workflow: &str) -> Result<Self, FailureKind> {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        Self::new_in(&std::env::temp_dir(), &NEXT, workflow)
    }

    fn new_in(root: &Path, next: &AtomicUsize, workflow: &str) -> Result<Self, FailureKind> {
        // create es exclusivo; el contador nunca se reinicia entre llamadas.
        // PID separa procesos, y los reintentos toleran residuos de un PID reutilizado.
        for _ in 0..128 {
            let path = root.join(format!(
                "mamut-engine-{}-{}",
                std::process::id(),
                next.fetch_add(1, Ordering::Relaxed)
            ));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            match builder.create(&path) {
                Ok(()) => {
                    let documents = Self(path);
                    fs::write(documents.0.join("catalog.json"), CATALOG)
                        .map_err(|_| FailureKind::Internal)?;
                    fs::write(documents.0.join("workflow.json"), workflow)
                        .map_err(|_| FailureKind::Internal)?;
                    return Ok(documents);
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(_) => return Err(FailureKind::Internal),
            }
        }
        Err(FailureKind::Internal)
    }

    fn path(&self, name: &str) -> Result<String, FailureKind> {
        self.0
            .join(name)
            .into_os_string()
            .into_string()
            .map_err(|_| FailureKind::Internal)
    }
}

impl Drop for EmbeddedDocuments {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn identity() -> CliIdentity {
    CliIdentity {
        engine_id: "mamut-engine".into(),
        engine_version: match engine_api::version()["engine_version"].as_str() {
            Some(version) => version.into(),
            None => String::new(),
        },
        operation: "run".into(),
    }
}

#[derive(Default)]
struct State {
    // MCP entrega sus propios frames: no retener documentos entre llamadas.
    capture_results: bool,
    results: Vec<Value>,
    warning: bool,
    failure: Option<(FailureKind, Option<Value>)>,
}

struct EngineProvider<'a> {
    input: EngineInput,
    control: Control,
    progress: EngineProgress<'a>,
    state: Rc<RefCell<State>>,
}

impl MethodProvider for EngineProvider<'_> {
    fn create(
        &self,
        descriptor: &MethodDescriptor,
        step: &CompiledStep<RegisteredMethod>,
    ) -> Result<Box<dyn Method>, String> {
        let validation = match (descriptor.method_id.as_str(), descriptor.method_version) {
            ("engine.run", 1) => false,
            ("engine.validate", 1) => true,
            _ => return self.fail(FailureKind::Unsupported),
        };
        if !step.parameters.is_empty() {
            return self.fail(FailureKind::Validation);
        }
        let mut method = EngineMethod {
            input: self.input.clone(),
            validation,
            control: self.control.clone(),
            progress: Rc::clone(&self.progress),
            state: Rc::clone(&self.state),
        };
        Ok(progress::completed(method.run()))
    }
}

impl EngineProvider<'_> {
    fn fail<T>(&self, kind: FailureKind) -> Result<T, String> {
        self.state.borrow_mut().failure = Some((kind, None));
        Err("Engine method rejected".into())
    }
}

struct EngineMethod<'a> {
    input: EngineInput,
    validation: bool,
    control: Control,
    progress: EngineProgress<'a>,
    state: Rc<RefCell<State>>,
}

impl EngineMethod<'_> {
    fn execute(&self) -> Result<(Status, Value), FailureKind> {
        if self.control.sync() {
            return Err(FailureKind::Cancelled);
        }
        let input = match &self.input {
            EngineInput::Document(document) => document.clone()?,
            EngineInput::Path(path) => Rc::from(read_engine_path(path.as_deref())?),
        };
        let progress = |event| {
            (self.progress.borrow_mut())(event);
            self.control.sync();
        };
        if self.validation {
            let diagnostics = engine_api::validate_with(&input, &self.control.token, progress)
                .map_err(api_failure)?;
            let status = diagnostic_status(&diagnostics);
            Ok((
                status,
                json!({"operation":"validate", "status":status, "diagnostics":diagnostics}),
            ))
        } else {
            let outcome = engine_api::run(&input, &self.control.token, progress);
            let document =
                serde_json::from_str(&outcome.document).map_err(|_| FailureKind::Internal)?;
            Ok((outcome.status, document))
        }
    }
}

impl Method for EngineMethod<'_> {
    fn run(&mut self) -> Result<Value, String> {
        let result = self.execute();
        let mut state = self.state.borrow_mut();
        match result {
            Ok((status, document)) => {
                if let Some(kind) = failure_kind(status) {
                    if state.capture_results {
                        state.failure = Some((kind, Some(document)));
                    }
                    Err("Engine operation failed".into())
                } else {
                    if state.capture_results {
                        state.warning |= status == Status::Warning;
                        state.results.push(document.clone());
                    }
                    Ok(document)
                }
            }
            Err(kind) => {
                state.failure = Some((kind, None));
                Err("Engine input could not be processed".into())
            }
        }
    }
}

#[derive(Clone)]
enum EngineInput {
    Document(Result<Rc<str>, FailureKind>),
    // MCP conserva la lectura por llamada de su transporte de compatibilidad.
    Path(Option<PathBuf>),
}

fn read_engine_path(path: Option<&Path>) -> Result<String, FailureKind> {
    let path = path.ok_or(FailureKind::Document)?;
    read_engine_input(File::open(path).map_err(|_| FailureKind::Document)?)
}

fn read_engine_input(reader: impl Read) -> Result<String, FailureKind> {
    let mut input = String::new();
    reader
        .take((engine_api::MAX_INPUT_BYTES + 1) as u64)
        .read_to_string(&mut input)
        .map_err(|_| FailureKind::Document)?;
    if input.len() > engine_api::MAX_INPUT_BYTES {
        return Err(FailureKind::Document);
    }
    Ok(input)
}

fn api_failure(error: ApiError) -> FailureKind {
    match error {
        ApiError::Cancelled => FailureKind::Cancelled,
        ApiError::InvalidInput(_) | ApiError::UnsupportedSchema(_) => FailureKind::Validation,
        ApiError::UnknownSchema(_) | ApiError::UnknownCode(_) => FailureKind::Unsupported,
    }
}

fn diagnostic_status(diagnostics: &[Diagnostic]) -> Status {
    if let Some(diagnostic) = diagnostics.iter().find(|d| d.severity == "error") {
        match diagnostic.code {
            "ENG-E-0006" | "ENG-E-0014" => Status::Unstable,
            "ENG-E-0007" => Status::NotConverged,
            "ENG-E-0012" => Status::InternalError,
            "ENG-E-0013" => Status::Unsupported,
            _ => Status::Invalid,
        }
    } else if diagnostics.iter().any(|d| d.severity == "warning") {
        Status::Warning
    } else {
        Status::Ok
    }
}

fn failure_kind(status: Status) -> Option<FailureKind> {
    match status {
        Status::Ok | Status::Warning => None,
        Status::Invalid => Some(FailureKind::Validation),
        Status::Unsupported => Some(FailureKind::Unsupported),
        Status::NotConverged | Status::Unstable => Some(FailureKind::NotConverged),
        Status::Cancelled => Some(FailureKind::Cancelled),
        Status::Timeout => Some(FailureKind::Timeout),
        Status::InternalError => Some(FailureKind::Internal),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn ndjson_event_and_error_are_buffered_as_complete_lines() {
        #[derive(Default)]
        struct CompleteLines(Vec<Value>);
        impl Write for CompleteLines {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                assert_eq!(bytes.last(), Some(&b'\n'));
                self.0.push(serde_json::from_slice(bytes).unwrap());
                Ok(bytes.len())
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut stderr = CompleteLines::default();
        // Mismo despacho dinámico de Write que usa el SDK; Value::fmt produce
        // muchos fragmentos, pero el escritor solo recibe la línea completa.
        let mut writer = EventWriter(&mut stderr);
        let writer: &mut dyn Write = &mut writer;
        writeln!(writer, "{}", json!({"phase":"started", "index":0})).unwrap();
        writer.flush().unwrap();
        assert_eq!(
            report_failure(FailureKind::Timeout, "run", None, &mut stderr),
            6
        );
        assert_eq!(stderr.0.len(), 2);
        assert_eq!(stderr.0[0]["phase"], "started");
        assert_eq!(stderr.0[1]["outcome"], "timeout");
    }

    #[test]
    fn direct_validation_maps_warnings_and_all_model_errors() {
        let mut diagnostics = vec![Diagnostic {
            code: "API-W-TEST",
            severity: "warning",
            entities: Vec::new(),
            cause: "Aviso sintético: validate_with no genera avisos actualmente".into(),
            hint: None,
        }];
        let (code, report) = validation_report(&diagnostics);
        assert_eq!(code, 1);
        assert_eq!(report["status"], "warning");
        for error in ["ENG-E-0003", "ENG-E-0006", "ENG-E-0012", "ENG-E-0013"] {
            diagnostics[0].code = error;
            diagnostics[0].severity = "error";
            let (code, report) = validation_report(&diagnostics);
            assert_eq!(code, 2);
            assert_eq!(report["status"], "invalid");
        }
    }

    #[test]
    fn embedded_documents_retry_collisions_and_keep_names_unique() -> Result<(), FailureKind> {
        let root = EmbeddedDocuments::new(RUN_WORKFLOW)?;
        let prefix = format!("mamut-engine-{}-", std::process::id());
        let next = AtomicUsize::new(0);
        // Contador local: este test no modifica ni reinicia el contador global.
        for index in 0..8 {
            fs::create_dir(root.0.join(format!("{prefix}{index}")))
                .map_err(|_| FailureKind::Internal)?;
        }
        let documents = (0..16)
            .map(|_| EmbeddedDocuments::new_in(&root.0, &next, VALIDATE_WORKFLOW))
            .collect::<Result<Vec<_>, _>>()?;
        let mut paths = std::collections::BTreeSet::new();
        for document in &documents {
            assert!(paths.insert(document.0.clone()));
            assert!(document
                .0
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with(&prefix));
            assert_eq!(
                fs::read_to_string(document.0.join("catalog.json")).unwrap(),
                CATALOG
            );
            assert_eq!(
                fs::read_to_string(document.0.join("workflow.json")).unwrap(),
                VALIDATE_WORKFLOW
            );
        }
        drop(documents);
        assert!(paths.iter().all(|path| !path.exists()));
        for index in 0..8 {
            assert!(root.0.join(format!("{prefix}{index}")).is_dir());
        }
        Ok(())
    }

    #[test]
    fn input_read_accepts_exact_max_with_eof() -> Result<(), FailureKind> {
        let bytes = vec![b' '; engine_api::MAX_INPUT_BYTES];
        let input = read_engine_input(bytes.as_slice())?;
        assert_eq!(input.len(), engine_api::MAX_INPUT_BYTES);
        Ok(())
    }

    #[test]
    fn input_read_rejects_max_plus_one_with_eof_as_document_two() {
        let bytes = vec![b' '; engine_api::MAX_INPUT_BYTES + 1];
        let result = read_engine_input(bytes.as_slice());
        assert!(matches!(result, Err(FailureKind::Document)));
        if let Err(kind) = result {
            let mut stderr = Vec::new();
            assert_eq!(report_failure(kind, "run", None, &mut stderr), 2);
        }
    }

    #[derive(Default)]
    struct GuardedReader {
        consumed: usize,
    }

    impl Read for GuardedReader {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            // Generador sin EOF. El fusible deja observar un byte de exceso y
            // aborta después para que incluso la mutación termine con poca memoria.
            let remaining = engine_api::MAX_INPUT_BYTES + 2 - self.consumed;
            if remaining == 0 {
                return Err(io::Error::other("Lectura más allá del límite de prueba"));
            }
            let count = buffer.len().min(remaining);
            buffer[..count].fill(b' ');
            self.consumed += count;
            Ok(count)
        }
    }

    #[test]
    fn input_read_stops_at_max_plus_one_and_returns_document_two() {
        let mut reader = GuardedReader::default();
        let result = read_engine_input(&mut reader);
        assert_eq!(reader.consumed, engine_api::MAX_INPUT_BYTES + 1);
        assert!(matches!(result, Err(FailureKind::Document)));
        if let Err(kind) = result {
            let mut stderr = Vec::new();
            assert_eq!(report_failure(kind, "run", None, &mut stderr), 2);
            let envelope = serde_json::from_slice::<Value>(&stderr);
            assert!(matches!(envelope, Ok(value) if value["outcome"] == "invalid_input"));
        }
    }

    #[test]
    fn validate_massless_instability_maps_to_exit_four() {
        let diagnostics = [Diagnostic {
            code: "ENG-E-0014",
            severity: "error",
            entities: Vec::new(),
            cause: "Rigidez sin masa no positiva".into(),
            hint: None,
        }];
        let status = diagnostic_status(&diagnostics);
        assert_eq!(status, Status::Unstable);
        assert_eq!(failure_kind(status), Some(FailureKind::NotConverged));
    }

    #[test]
    fn validate_eng_e_0013_maps_to_exit_three() {
        // validate_with hoy solo emite diagnósticos topológicos. Inyectamos el
        // diagnóstico en el clasificador privado que consume su resultado.
        let diagnostics = [Diagnostic {
            code: "ENG-E-0013",
            severity: "error",
            entities: Vec::new(),
            cause: "Capacidad no soportada".into(),
            hint: None,
        }];
        let status = diagnostic_status(&diagnostics);
        let kind = failure_kind(status);
        if let Some(kind) = kind {
            let mut stderr = Vec::new();
            assert_eq!(report_failure(kind, "run", None, &mut stderr), 3);
            let envelope = serde_json::from_slice::<Value>(&stderr);
            assert!(matches!(envelope, Ok(value)
                if value["exitCode"] == 3 && value["outcome"] == "unsupported"));
        }
        assert_eq!(status, Status::Unsupported);
        assert!(matches!(kind, Some(FailureKind::Unsupported)));
    }
}
