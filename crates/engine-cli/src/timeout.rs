//! Extensión aislada del parser SDK y temporizador cooperativo nativo.
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use mamut_workflow_cli::FailureKind;

use crate::Control;

pub(crate) fn parse_timeout_args(
    args: &[String],
) -> Result<(Vec<String>, Option<f64>), FailureKind> {
    let mut forwarded = Vec::with_capacity(args.len());
    let mut timeout = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if arg == "--timeout" {
            if timeout.is_some() {
                return Err(FailureKind::Usage);
            }
            timeout = Some(parse_seconds(args.next().ok_or(FailureKind::Usage)?)?);
        } else {
            forwarded.push(arg.clone());
            // La base consume el siguiente token de estos flags como valor,
            // incluso si luego lo rechaza. No reinterpretarlo como timeout.
            if matches!(
                arg.as_str(),
                "--catalog" | "--workflow" | "--progress" | "--input"
            ) {
                if let Some(value) = args.next() {
                    forwarded.push(value.clone());
                }
            }
        }
    }
    // El flag prevalece incluso sobre una variable inválida o no Unicode.
    if timeout.is_none() {
        if let Some(value) = std::env::var_os("MAMUT_ENGINE_TIMEOUT") {
            timeout = Some(parse_seconds(value.to_str().ok_or(FailureKind::Usage)?)?);
        }
    }
    Ok((forwarded, timeout))
}

fn parse_seconds(value: &str) -> Result<f64, FailureKind> {
    let seconds: f64 = value.parse().map_err(|_| FailureKind::Usage)?;
    if !seconds.is_finite() || seconds <= 0.0 || seconds > 31_536_000.0 {
        return Err(FailureKind::Usage);
    }
    Ok(seconds)
}

// El deadline solo clasifica una cancelación observada. Un resultado terminal
// ya obtenido conserva su semántica aunque el reloj venza antes de comprobarlo.
pub(crate) fn resolve<T>(result: Result<T, FailureKind>, expired: bool) -> Result<T, FailureKind> {
    result.map_err(|kind| {
        if expired && kind == FailureKind::Cancelled {
            FailureKind::Timeout
        } else {
            kind
        }
    })
}

pub(crate) struct Timer {
    stop: Sender<()>,
    thread: Option<JoinHandle<()>>,
    expired: Arc<AtomicBool>,
}

impl Timer {
    pub(crate) fn start(seconds: f64, control: Control) -> Result<Self, FailureKind> {
        let started = Instant::now();
        let deadline = Duration::try_from_secs_f64(seconds)
            .ok()
            .and_then(|duration| started.checked_add(duration));
        let (stop, receiver) = mpsc::channel();
        control.sync();
        let token = control.token();
        let cancel_file = control.cancel_file;
        let expired = Arc::new(AtomicBool::new(false));
        let thread_expired = Arc::clone(&expired);
        let thread = thread::Builder::new()
            .name("mamut-engine-timeout".into())
            .spawn(move || {
                loop {
                    // Si la plataforma no representa el deadline en Instant,
                    // esperar por tramos dentro del plazo validado.
                    let wait = match deadline {
                        Some(deadline) => deadline.saturating_duration_since(Instant::now()),
                        None => match Duration::try_from_secs_f64(
                            (seconds - started.elapsed().as_secs_f64()).clamp(0.0, 86400.0),
                        ) {
                            Ok(wait) => wait,
                            Err(_) => return,
                        },
                    };
                    if !matches!(receiver.recv_timeout(wait), Err(RecvTimeoutError::Timeout)) {
                        return;
                    }
                    if deadline.is_none() && started.elapsed().as_secs_f64() < seconds {
                        continue;
                    }
                    if !token.is_cancelled()
                        && !cancel_file.as_ref().is_some_and(|path| path.exists())
                    {
                        thread_expired.store(true, Ordering::Release);
                        token.cancel();
                    }
                    return;
                }
            })
            .map_err(|_| FailureKind::Internal)?;
        Ok(Self {
            stop,
            thread: Some(thread),
            expired,
        })
    }

    pub(crate) fn expired(&self) -> bool {
        self.expired.load(Ordering::Acquire)
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        // Despertar y recoger el hilo también al terminar mucho antes del plazo.
        let _ = self.stop.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeout_token_in_base_flag_value_is_forwarded() {
        for flag in ["--catalog", "--workflow", "--progress", "--input"] {
            let args = ["run", flag, "--timeout", "5", "--timeout", "60"].map(String::from);
            let (forwarded, timeout) = parse_timeout_args(&args).unwrap();
            assert_eq!(forwarded, ["run", flag, "--timeout", "5"]);
            assert_eq!(timeout, Some(60.0));
        }
    }

    #[test]
    fn terminal_result_survives_deadline_before_post_dispatch_check() {
        // Vencimiento ya visible al comprobar un dispatch terminado, sin sleeps.
        assert_eq!(resolve(Ok(42), true), Ok(42));
        for kind in [
            FailureKind::Usage,
            FailureKind::Document,
            FailureKind::Validation,
            FailureKind::Unsupported,
            FailureKind::NotConverged,
            FailureKind::Method,
            FailureKind::Internal,
        ] {
            assert_eq!(resolve::<()>(Err(kind), true), Err(kind));
        }
        assert_eq!(
            resolve::<()>(Err(FailureKind::Cancelled), true),
            Err(FailureKind::Timeout)
        );
        assert_eq!(
            resolve::<()>(Err(FailureKind::Cancelled), false),
            Err(FailureKind::Cancelled)
        );
    }
}
