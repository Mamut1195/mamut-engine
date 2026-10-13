//! Backend contador compartido por los ensayos de ensamblaje y análisis.
use engine_core::EResult;
use engine_linalg::{
    DMat, FactorInfo, GeneralizedEigen, LinAlgBackend, ReferenceBackend, SymFactorization,
    SymMatrix,
};
use std::{cell::Cell, rc::Rc};

pub struct CountingBackend {
    pub factorizations: Rc<Cell<usize>>,
    pub solves: Rc<Cell<usize>>,
    pub solution_scale: f64,
}

impl Default for CountingBackend {
    fn default() -> Self {
        Self {
            factorizations: Rc::new(Cell::new(0)),
            solves: Rc::new(Cell::new(0)),
            solution_scale: 1.0,
        }
    }
}

struct CountingFactor {
    inner: Box<dyn SymFactorization>,
    solves: Rc<Cell<usize>>,
    scale: f64,
}
impl SymFactorization for CountingFactor {
    fn info(&self) -> &FactorInfo {
        self.inner.info()
    }
    fn solve_in_place(&self, b: &mut DMat) -> EResult<()> {
        self.solves.set(self.solves.get() + 1);
        self.inner.solve_in_place(b)?;
        // 1 por defecto. Otro valor simula un solver inexacto para comprobar
        // que los reportes detectan desequilibrio y residual libre.
        *b = DMat::new(
            b.rows(),
            b.cols(),
            b.values().iter().map(|v| v * self.scale).collect(),
        )?;
        Ok(())
    }
}
impl LinAlgBackend for CountingBackend {
    fn name(&self) -> &'static str {
        "test.counting.reference"
    }
    fn matmul(&mut self, a: &DMat, b: &DMat) -> EResult<DMat> {
        ReferenceBackend.matmul(a, b)
    }
    fn transpose(&mut self, a: &DMat) -> EResult<DMat> {
        ReferenceBackend.transpose(a)
    }
    fn solve_dense(&mut self, a: &DMat, b: &DMat) -> EResult<DMat> {
        ReferenceBackend.solve_dense(a, b)
    }
    fn factorize(&mut self, a: &SymMatrix) -> EResult<Box<dyn SymFactorization>> {
        self.factorizations.set(self.factorizations.get() + 1);
        Ok(Box::new(CountingFactor {
            inner: ReferenceBackend.factorize(a)?,
            solves: self.solves.clone(),
            scale: self.solution_scale,
        }))
    }
    fn generalized_eigen(&mut self, k: &SymMatrix, m: &SymMatrix) -> EResult<GeneralizedEigen> {
        ReferenceBackend.generalized_eigen(k, m)
    }
}
