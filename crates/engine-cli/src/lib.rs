// SPDX-License-Identifier: AGPL-3.0-only

//! Adaptador nativo de engine-api sobre el parser y despacho de mamut-workflow-cli.
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::{
    cell::RefCell,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    rc::Rc,
    sync::Arc,
};

use engine_api::{ApiError, CancelToken, Diagnostic, Progress, Status};
use mamut_workflow::{CompiledStep, MethodDescriptor, RegisteredMethod};
use mamut_workflow_cli::{
    cli_run_failure_with_identity, dispatch, parse_args, CancelFlag, Cancellation, CliConfig,
    CliIdentity, Command, ExitCodes, FailureKind, Method, MethodProvider,
};
use serde_json::{json, Value};

/// Catálogo propio; también disponible como archivo `catalog.json` del crate.
pub const CATALOG: &str = include_str!("../catalog.json");

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

type EngineProgress = Rc<RefCell<Box<dyn FnMut(Progress)>>>;

/// Configuración del anfitrión. Los argumentos siguen siendo los de la base.
/// Una invocación conserva una única ruta engine-input/1 para todos sus pasos.
pub struct EngineCli {
    input: Option<PathBuf>,
    control: Control,
    progress: EngineProgress,
}

impl EngineCli {
    /// Configura la ruta de entrada del motor; `None` falla solo al ejecutar métodos.
    pub fn new(input: Option<PathBuf>) -> Self {
        Self {
            input,
            control: Control::default(),
            progress: Rc::new(RefCell::new(Box::new(|_| {}))),
        }
    }

    /// Conecta cancelación cooperativa, incluida cancelación durante engine-api.
    #[must_use]
    pub fn with_control(mut self, control: Control) -> Self {
        self.control = control;
        self
    }

    /// Observa etapas de engine-api. Puede cancelar mediante un `Control` clonado.
    /// El NDJSON estándar conserva los eventos started/finished del workflow.
    #[must_use]
    pub fn with_engine_progress(mut self, progress: impl FnMut(Progress) + 'static) -> Self {
        self.progress = Rc::new(RefCell::new(Box::new(progress)));
        self
    }

    /// Delega íntegramente el parser y despacho a la base.
    /// Añade resultados completos y semántica Engine al sobre terminal de `run`.
    /// stdout contiene resultados; stderr contiene NDJSON y errores sanitizados.
    pub fn run<R: Read, O: Write, E: Write>(
        &self,
        args: &[String],
        stdin: &mut R,
        stdout: &mut O,
        stderr: &mut E,
    ) -> u8 {
        let command = match parse_args(args) {
            Ok(command) => command,
            Err(error) => {
                return report_failure(
                    error.kind,
                    args.first().is_some_and(|arg| arg == "run"),
                    None,
                    stderr,
                );
            }
        };
        let is_run = matches!(command, Command::Run { .. });
        let state = Rc::new(RefCell::new(State {
            capture_results: is_run,
            ..State::default()
        }));
        let config = CliConfig::new()
            .with_identity(identity())
            .with_exit_codes(ExitCodes::engine_v1())
            .with_cancellation(self.control.clone())
            .with_provider(EngineProvider {
                input: self.input.clone(),
                control: self.control.clone(),
                progress: Rc::clone(&self.progress),
                state: Rc::clone(&state),
            });
        let mut buffered = Vec::new();
        // Solo se retiene el sobre terminal. MCP y NDJSON conservan streaming.
        let mut output: &mut dyn Write = if is_run { &mut buffered } else { stdout };
        let result = dispatch(command, stdin, &mut output, stderr, &config);
        let state = state.borrow();
        if let Err(error) = result {
            let (kind, document) = if error.kind == FailureKind::Method {
                match &state.failure {
                    Some((kind, document)) => (*kind, document.as_ref()),
                    None => (error.kind, None),
                }
            } else {
                (error.kind, None)
            };
            return report_failure(kind, is_run, document, stderr);
        }
        if !is_run {
            return config.exit_codes.ok;
        }
        let mut envelope: Value = match serde_json::from_slice(&buffered) {
            Ok(envelope) => envelope,
            Err(_) => return report_failure(FailureKind::Internal, true, None, stderr),
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
            return report_failure(FailureKind::Internal, true, None, stderr);
        }
        code
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

fn report_failure(
    kind: FailureKind,
    is_run: bool,
    document: Option<&Value>,
    stderr: &mut impl Write,
) -> u8 {
    let code = ExitCodes::engine_v1().code(kind);
    let (outcome, message) = match kind {
        FailureKind::Usage => ("usage", "Invalid command or arguments"),
        FailureKind::Document | FailureKind::Validation => ("invalid_input", "Invalid input"),
        FailureKind::Unsupported => ("unsupported", "Operation is not supported"),
        FailureKind::NotConverged => ("not_converged", "Execution did not converge or is unstable"),
        FailureKind::Cancelled => ("cancelled", "Execution was cancelled"),
        FailureKind::Timeout => ("timeout", "Execution timed out"),
        FailureKind::Method | FailureKind::Internal => {
            ("internal", "Execution could not be completed")
        }
    };
    let mut envelope = if is_run {
        cli_run_failure_with_identity(&identity(), code, outcome, message)
    } else {
        json!({"exit_code":code, "message":message})
    };
    if let Some(document) = document {
        envelope["engineResults"] = json!([document]);
    }
    if writeln!(stderr, "{envelope}").is_err() {
        ExitCodes::engine_v1().internal
    } else {
        code
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

struct EngineProvider {
    input: Option<PathBuf>,
    control: Control,
    progress: EngineProgress,
    state: Rc<RefCell<State>>,
}

impl MethodProvider for EngineProvider {
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
        Ok(Box::new(EngineMethod {
            input: self.input.clone(),
            validation,
            control: self.control.clone(),
            progress: Rc::clone(&self.progress),
            state: Rc::clone(&self.state),
        }))
    }
}

impl EngineProvider {
    fn fail<T>(&self, kind: FailureKind) -> Result<T, String> {
        self.state.borrow_mut().failure = Some((kind, None));
        Err("Engine method rejected".into())
    }
}

struct EngineMethod {
    input: Option<PathBuf>,
    validation: bool,
    control: Control,
    progress: EngineProgress,
    state: Rc<RefCell<State>>,
}

impl EngineMethod {
    fn execute(&self) -> Result<(Status, Value), FailureKind> {
        if self.control.sync() {
            return Err(FailureKind::Cancelled);
        }
        let path = self.input.as_ref().ok_or(FailureKind::Document)?;
        let mut input = String::new();
        File::open(path)
            .map_err(|_| FailureKind::Document)?
            .take((engine_api::MAX_INPUT_BYTES + 1) as u64)
            .read_to_string(&mut input)
            .map_err(|_| FailureKind::Document)?;
        if input.len() > engine_api::MAX_INPUT_BYTES {
            return Err(FailureKind::Document);
        }
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

impl Method for EngineMethod {
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
            "ENG-E-0006" => Status::Unstable,
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
