// SPDX-License-Identifier: AGPL-3.0-only

//! Local transport for complete events, shared with the SDK observer.

use std::{cell::RefCell, io, io::Write, rc::Rc};

use engine_api::Progress;
use mamut_workflow_cli::{Method, ProgressEvent, ProgressListener, ProgressPhase};
use serde_json::{json, Value};

use crate::{EngineProgress, EventWriter};

pub(crate) struct Output<'a, E> {
    inner: Rc<RefCell<Sink<'a, E>>>,
}

struct Sink<'a, E> {
    writer: &'a mut E,
    enabled: bool,
    index: Option<usize>,
    failed: bool,
}

impl<E> Clone for Output<'_, E> {
    fn clone(&self) -> Self {
        Self {
            inner: Rc::clone(&self.inner),
        }
    }
}

impl<'a, E: Write> Output<'a, E> {
    pub(crate) fn new(writer: &'a mut E, enabled: bool) -> Self {
        Self {
            inner: Rc::new(RefCell::new(Sink {
                writer,
                enabled,
                index: None,
                failed: false,
            })),
        }
    }

    pub(crate) fn failed(&self) -> bool {
        self.inner.borrow().failed
    }

    pub(crate) fn callback(&self, host: EngineProgress<'static>) -> EngineProgress<'a> {
        let output = self.clone();
        Rc::new(RefCell::new(Box::new(move |event| {
            output.emit(event);
            (host.borrow_mut())(event);
        })))
    }

    fn emit(&self, event: Progress) {
        let mut sink = self.inner.borrow_mut();
        if !sink.enabled || sink.failed {
            return;
        }
        let Some(index) = sink.index else {
            return;
        };
        let value = json!({"schema":"progress/1", "index":index,
            "operation":event.operation, "stage":event.stage,
            "completed":event.completed, "total":event.total});
        // Same canonicalizer and numerical format as results/1.
        let line = engine_api::canonical_json(&value.to_string());
        match line {
            Ok(line) => {
                sink.failed = writeln!(EventWriter(&mut *sink.writer), "{line}")
                    .and_then(|()| sink.writer.flush())
                    .is_err();
            }
            Err(_) => sink.failed = true,
        }
    }
}

impl<E: Write> ProgressListener for Output<'_, E> {
    fn on_progress(&mut self, event: ProgressEvent) {
        self.inner.borrow_mut().index = match event.phase {
            ProgressPhase::Started => Some(event.index),
            ProgressPhase::Finished => None,
        };
    }
}

impl<E: Write> Write for Output<'_, E> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let mut sink = self.inner.borrow_mut();
        let result = sink.writer.write(bytes);
        sink.failed |= result.is_err();
        result
    }

    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> io::Result<()> {
        let mut sink = self.inner.borrow_mut();
        // Reuse the main writer for SDK lines as well.
        let result = EventWriter(&mut *sink.writer).write_fmt(args);
        sink.failed |= result.is_err();
        result
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut sink = self.inner.borrow_mut();
        let result = sink.writer.flush();
        sink.failed |= result.is_err();
        result
    }
}

// The SDK requests Method without borrows. The factory is invoked within started /
// finished: executing there allows using borrowed stderr during each callback,
// without threads, global state, or retaining stages until the end of the step.
pub(crate) fn completed(result: Result<Value, String>) -> Box<dyn Method> {
    Box::new(Completed(Some(result)))
}

struct Completed(Option<Result<Value, String>>);

impl Method for Completed {
    fn run(&mut self) -> Result<Value, String> {
        self.0
            .take()
            .ok_or_else(|| String::from("Engine result already consumed"))?
    }
}
