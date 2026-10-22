// SPDX-License-Identifier: AGPL-3.0-only

//! Puente Unix de flags de señales al token cooperativo existente.
use std::{
    io,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, RecvTimeoutError, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use engine_api::CancelToken;
use signal_hook::{
    consts::signal::{SIGINT, SIGTERM},
    flag, low_level, SigId,
};

pub(crate) struct Signals {
    registrations: Vec<SigId>,
    stop: Sender<()>,
    thread: Option<JoinHandle<()>>,
}

impl Signals {
    pub(crate) fn start(token: Arc<CancelToken>) -> io::Result<Self> {
        let interrupted = Arc::new(AtomicBool::new(false));
        let (stop, receiver) = mpsc::channel();
        let mut signals = Self {
            registrations: Vec::new(),
            stop,
            thread: None,
        };
        // El manejador solo escribe un atómico. Ni E/S ni cancelación del SDK
        // (cuyo flag usa Rc<Cell<bool>>) se ejecutan en contexto de señal.
        for signal in [SIGINT, SIGTERM] {
            signals
                .registrations
                .push(flag::register(signal, Arc::clone(&interrupted))?);
        }
        signals.thread = Some(
            thread::Builder::new()
                .name("mamut-engine-signals".into())
                .spawn(move || loop {
                    if interrupted.load(Ordering::Relaxed) {
                        // Como timeout, cancela el token incluso dentro de un solver;
                        // Control sincroniza el flag SDK en su siguiente consulta.
                        token.cancel();
                        return;
                    }
                    if !matches!(
                        receiver.recv_timeout(Duration::from_millis(5)),
                        Err(RecvTimeoutError::Timeout)
                    ) {
                        return;
                    }
                })?,
        );
        Ok(signals)
    }
}

impl Drop for Signals {
    fn drop(&mut self) {
        for registration in self.registrations.drain(..) {
            low_level::unregister(registration);
        }
        // También recoge el hilo sin esperar al próximo sondeo al salir o fallar.
        let _ = self.stop.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
