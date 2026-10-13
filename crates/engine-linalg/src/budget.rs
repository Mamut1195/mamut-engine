//! Presupuesto acumulado de memoria (ENG-LA-008).
//!
//! `alloc_vec` limita cada reserva individual; muchas reservas permitidas pueden
//! sumar más de lo disponible (p. ej. wasm32, 4 GiB de espacio de direcciones).
//! `MemoryBudget` lleva la cuenta de los bytes cargados por un llamador y rechaza
//! con ENG-E-0013 (capacidad no soportada) toda carga que supere el límite.
//! Es contabilidad explícita y determinista: no consulta el sistema operativo,
//! no usa hilos ni estado global; cada llamador posee su presupuesto.
use crate::*;

/// Presupuesto de memoria acumulado, en bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemoryBudget {
    limit: usize,
    used: usize,
    peak: usize,
}

const F64: usize = std::mem::size_of::<Real>();

/// Cota superior del almacenamiento de heap que RETIENE una factorización del
/// backend de referencia de orden `n`: L densa, D, diagonal secundaria, bloques
/// (capacidad reservada n), permutación, escalas y cabecera del factor en su Box.
/// No incluye la entrada ni metadatos internos del allocator. `None` si desborda.
pub fn factor_bytes(n: usize) -> Option<usize> {
    let vectors =
        n.checked_mul(3 * std::mem::size_of::<f64>() + 2 * std::mem::size_of::<usize>())?;
    dense_bytes(n, n)?
        .checked_add(vectors)?
        .checked_add(crate::reference::FACTOR_HEADER_BYTES)
}

/// Cota del trabajo TRANSITORIO: balanced y la copia normalizada a siguen vivas
/// junto con L en LDLᵀ. Las filas temporales del equilibrado (n valores) caben
/// en estas dos matrices para n>0; para n=0 no reservan. `None` si desborda.
fn factor_working_bytes(n: usize) -> Option<usize> {
    dense_bytes(n, n)?.checked_mul(2)
}

/// Cota superior del trabajo TRANSITORIO de `generalized_eigen` de orden `n`
/// en el backend de referencia: nueve matrices densas n×n (Cholesky, inversa,
/// productos, Jacobi y capacidad sobrante al recoger los modos), más vectores
/// auxiliares y cabecera LDLᵀ para el diagnóstico de masa rechazada por Cholesky.
/// Excluye las entradas y el resultado (n² de modos + n autovalores), que
/// `MemoryBudget::generalized_eigen` carga aparte. No incluye metadatos internos
/// del allocator. `None` si desborda.
pub fn eigen_working_bytes(n: usize) -> Option<usize> {
    // Equilibrado, filas temporales, D y diagonal secundaria: cuatro vectores
    // Real; bloques/permutación o índices/espacio del ordenamiento: dos usize.
    let auxiliary = n.checked_mul(4 * F64 + 2 * std::mem::size_of::<usize>())?;
    dense_bytes(n, n)?
        .checked_mul(9)?
        .checked_add(auxiliary)?
        .checked_add(crate::reference::FACTOR_HEADER_BYTES)
}

fn eigen_result_bytes(n: usize) -> Option<usize> {
    dense_bytes(n, n)?.checked_add(n.checked_mul(std::mem::size_of::<f64>())?)
}

/// Bytes de una matriz densa `rows × cols` de `Real`; `None` si desborda.
pub fn dense_bytes(rows: usize, cols: usize) -> Option<usize> {
    rows.checked_mul(cols)?.checked_mul(F64)
}

/// Bytes aproximados de una CSR con `rows` filas y `nnz` entradas
/// (offsets + índices de columna + valores); `None` si desborda.
pub fn sparse_bytes(rows: usize, nnz: usize) -> Option<usize> {
    let offsets = rows
        .checked_add(1)?
        .checked_mul(std::mem::size_of::<usize>())?;
    let entries = nnz.checked_mul(std::mem::size_of::<usize>() + F64)?;
    offsets.checked_add(entries)
}

impl MemoryBudget {
    /// Presupuesto con `limit` bytes.
    pub fn new(limit: usize) -> Self {
        Self {
            limit,
            used: 0,
            peak: 0,
        }
    }
    /// Presupuesto igual al límite por reserva del destino (`MAX_ALLOC_BYTES`:
    /// 8 GiB en 64 bits, 1 GiB en 32 bits).
    pub fn for_target() -> Self {
        Self::new(MAX_ALLOC_BYTES)
    }
    /// Límite total en bytes.
    pub fn limit(&self) -> usize {
        self.limit
    }
    /// Bytes cargados actualmente.
    pub fn used(&self) -> usize {
        self.used
    }
    /// Máximo de bytes cargados simultáneamente desde la creación.
    pub fn peak(&self) -> usize {
        self.peak
    }
    /// Bytes aún disponibles.
    pub fn remaining(&self) -> usize {
        self.limit - self.used
    }
    /// Carga `bytes`. ENG-E-0013 si excede el límite (o desborda); ante error
    /// `used` queda intacto (`peak` no cambia porque la carga fue rechazada).
    pub fn charge(&mut self, bytes: usize) -> EResult<()> {
        match self.used.checked_add(bytes) {
            Some(total) if total <= self.limit => {
                self.used = total;
                self.peak = self.peak.max(total);
                Ok(())
            }
            _ => Err(unsupported(&format!(
                "Presupuesto de memoria excedido: {} usados + {bytes} solicitados > límite {} bytes.",
                self.used, self.limit
            ))),
        }
    }
    /// Libera `bytes` (saturando en 0: liberar de más nunca da un uso negativo).
    /// No hay identidad de cargas: una doble liberación del llamador resta de otras
    /// cargas vivas (subcontabilidad); liberar solo lo cargado por el propio llamador.
    pub fn release(&mut self, bytes: usize) {
        self.used = self.used.saturating_sub(bytes);
    }
    /// Libera el almacenamiento de una matriz densa cargada con este presupuesto.
    pub fn release_dense(&mut self, m: &DMat) {
        if let Some(b) = dense_bytes(m.rows(), m.cols()) {
            self.release(b);
        }
    }
    /// Factoriza `a` cargando antes el presupuesto (retenido + trabajo). Éxito: queda
    /// cargada solo la factorización retenida (`factor_bytes(n)`; libérala con
    /// `release` al soltarla); error o pánico recuperado: `used` intacto.
    /// Orden de errores: forma/desborde de tamaño → presupuesto (ENG-E-0013) → backend.
    /// La cota cubre el almacenamiento numérico del backend de referencia;
    /// otros backends y la construcción de mensajes de error no están acotados.
    pub fn factorize<B: LinAlgBackend>(
        &mut self,
        backend: &mut B,
        a: &SymMatrix,
    ) -> EResult<Box<dyn SymFactorization>> {
        let overflow = || invalid("Tamaño matricial desbordado.");
        let n = a.order();
        let retained = factor_bytes(n).ok_or_else(overflow)?;
        let total = retained
            .checked_add(factor_working_bytes(n).ok_or_else(overflow)?)
            .ok_or_else(overflow)?;
        self.charge(total)?;
        let mut guard = ReleaseOnDrop {
            budget: self,
            bytes: total,
        };
        let result = backend.factorize(a);
        if result.is_ok() {
            guard.bytes = total - retained;
        }
        drop(guard);
        result
    }
    /// Autovalores generalizados con presupuesto: carga el conjunto de trabajo
    /// (`eigen_working_bytes(n)`) más el resultado; éxito: quedan cargados solo los
    /// bytes del resultado (n² + n valores `f64`); error o pánico: `used` intacto.
    /// La cota cubre el almacenamiento numérico del backend de referencia;
    /// otros backends y la construcción de mensajes de error no están acotados.
    pub fn generalized_eigen<B: LinAlgBackend>(
        &mut self,
        backend: &mut B,
        k: &SymMatrix,
        m: &SymMatrix,
    ) -> EResult<GeneralizedEigen> {
        if k.order() != m.order() {
            return backend.generalized_eigen(k, m);
        }
        let overflow = || invalid("Tamaño matricial desbordado.");
        let n = k.order();
        let work = eigen_working_bytes(n).ok_or_else(overflow)?;
        let result_bytes = eigen_result_bytes(n).ok_or_else(overflow)?;
        let total = work.checked_add(result_bytes).ok_or_else(overflow)?;
        self.charge(total)?;
        let mut guard = ReleaseOnDrop {
            budget: self,
            bytes: total,
        };
        let result = backend.generalized_eigen(k, m);
        if result.is_ok() {
            guard.bytes = work;
        }
        drop(guard);
        result
    }
    /// Crea una matriz nula cargándola; el llamador la libera con `release_dense`.
    /// ENG-E-0013 si no cabe en el presupuesto; ENG-E-0010 por desborde/reserva.
    pub fn zeros(&mut self, rows: usize, cols: usize) -> EResult<DMat> {
        let bytes =
            dense_bytes(rows, cols).ok_or_else(|| invalid("Tamaño matricial desbordado."))?;
        self.charge(bytes)?;
        DMat::zeros(rows, cols).inspect_err(|_| self.release(bytes))
    }
    /// Resuelve `A X = B` con un backend cargando el conjunto de trabajo
    /// (copia de A, B y X: `n² + 2·n·m` elementos). El trabajo transitorio se
    /// libera al terminar; la solución X queda cargada hasta `release_dense`.
    /// ENG-E-0013 si el conjunto de trabajo no cabe; errores del backend se
    /// propagan y dejan `used` como estaba (`peak` conserva el máximo alcanzado
    /// por el conjunto de trabajo, que sí llegó a cargarse); un pánico recuperado
    /// tampoco deja `used` cargado.
    pub fn solve_dense<B: LinAlgBackend>(
        &mut self,
        backend: &mut B,
        a: &DMat,
        b: &DMat,
    ) -> EResult<DMat> {
        // Orden de errores independiente del presupuesto: primero la forma de los
        // operandos (la valida el backend sin reservar nada), después desborde de
        // tamaños y por último el presupuesto (ENG-E-0013).
        if a.rows() != a.cols() || b.rows() != a.rows() {
            return backend.solve_dense(a, b);
        }
        let overflow = || invalid("Tamaño matricial desbordado.");
        let a_bytes = dense_bytes(a.rows(), a.cols()).ok_or_else(overflow)?;
        let b_bytes = dense_bytes(b.rows(), b.cols()).ok_or_else(overflow)?;
        let transient = a_bytes.checked_add(b_bytes).ok_or_else(overflow)?;
        let total = transient.checked_add(b_bytes).ok_or_else(overflow)?;
        self.charge(total)?;
        // Guarda RAII: si el backend entra en pánico (y el llamador lo recupera
        // con `catch_unwind`), la carga se libera al desenrollar; `used` no queda
        // cargado.
        let mut guard = ReleaseOnDrop {
            budget: self,
            bytes: total,
        };
        let result = backend.solve_dense(a, b);
        if result.is_ok() {
            // Éxito: solo el trabajo transitorio se libera; X queda cargada.
            guard.bytes = transient;
        }
        drop(guard);
        result
    }
}

/// Libera `bytes` del presupuesto al destruirse (también al desenrollar un pánico).
struct ReleaseOnDrop<'a> {
    budget: &'a mut MemoryBudget,
    bytes: usize,
}
impl Drop for ReleaseOnDrop<'_> {
    fn drop(&mut self) {
        self.budget.release(self.bytes);
    }
}
