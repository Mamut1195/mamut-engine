// SPDX-License-Identifier: AGPL-3.0-only

//! Bridge Unix signal flags to the existing cooperative cancellation token.
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
        // The handler only writes an atomic. I/O and SDK cancellation
        // (whose flag uses Rc<Cell<bool>>) never run in signal context.
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
                        // Like timeout, cancel the token even inside a solver;
                        // Control synchronizes the SDK flag on its next check.
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
        // Join promptly on completion or failure without waiting for the next poll.
        let _ = self.stop.send(());
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
