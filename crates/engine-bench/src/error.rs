use std::fmt;

/// Error de generación, traducción o índice. No incluye rutas de otros motores.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchError {
    message: String,
}

impl BenchError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for BenchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for BenchError {}
