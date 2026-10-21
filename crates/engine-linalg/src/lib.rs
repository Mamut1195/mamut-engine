#![forbid(unsafe_code)]
//! Frontera numérica interna con tipos propios y errores del núcleo.
//!
//! `ReferenceBackend` es una implementación sencilla para pruebas y problemas
//! pequeños: trabajo cúbico y memoria cuadrática, sin promesas de rendimiento.
//! La feature `faer` reserva un stub: integración PENDIENTE de verificar los
//! avisos de licencia del paquete 0.24.4; véase README.md de este crate.
//! Las magnitudes conservan las unidades SI de sus filas y columnas; este crate
//! no convierte unidades ni conoce el mapa entre índices globales y nodos.
//! V1 usa la referencia secuencial, sin SIMD explícito ni FMA implícito.
//! El orden de reducción y la excepción FMA explícita de Sturm se documentan
//! en `docs/DETERMINISMO.md` (ENG-CORE-010). Las funciones escalares pasan por
//! `engine_core::math`: libm 0.2.16 (MIT) salvo sqrt, correctamente redondeada
//! IEEE-754 en std. El cruce nativo/wasm verifica resultados y bits escalares.
//!
//! ```
//! use engine_linalg::{DMat, LinAlgBackend, ReferenceBackend};
//! let a = DMat::new(2, 2, vec![2.0, 0.0, 0.0, 4.0])?;
//! let b = DMat::new(2, 1, vec![6.0, 8.0])?;
//! let x = ReferenceBackend.solve_dense(&a, &b)?;
//! assert_eq!(x.values(), &[3.0, 2.0]);
//! # Ok::<(), engine_core::EngineError>(())
//! ```
#![deny(missing_docs)]

use engine_core::{DofIndex, EResult, EngineError, EntityRef, Real};
mod budget;
mod guyan;
mod matrix;
mod modes;
mod reference;
mod sparse;
pub use budget::{dense_bytes, eigen_working_bytes, factor_bytes, sparse_bytes, MemoryBudget};
pub use guyan::{guyan_condense, GuyanReduction};
pub use matrix::{DMat, SparsityPattern, SymMatrix};
pub use modes::{lowest_modes, sturm_count, LowestModes};
pub use reference::ReferenceBackend;
pub use sparse::{SparseMatrix, Triplet};
#[cfg(feature = "faer")]
mod faer_backend;
#[cfg(feature = "faer")]
pub use faer_backend::FaerBackend;

/// Información numérica de una factorización simétrica sin regularización.
#[derive(Clone, Debug, PartialEq)]
pub struct FactorInfo {
    /// Conteos (positivos, negativos, nulos), si el backend los ofrece.
    /// El control de Sturm usa el número de negativos de esta inercia.
    pub inertia: Option<(usize, usize, usize)>,
    /// Mínimo valor absoluto de pivote dividido por el máximo, si se conoce.
    /// En bloques 2×2 se usan sus dos autovalores, no solo la diagonal.
    /// Es un indicador adimensional, no una estimación del número de condición.
    pub min_pivot_ratio: Option<Real>,
    /// Índices globales de pivotes singulares detectados, contados desde cero.
    /// Una factorización singular devuelve ENG-E-0006 con estas entidades;
    /// el ensamblador futuro deberá traducirlas a nodo y componente física.
    pub singular_dofs: Vec<DofIndex>,
}

/// Factor reutilizable; resolver no refactoriza ni modifica el factor.
pub trait SymFactorization {
    /// Metadatos de la factorización numérica.
    fn info(&self) -> &FactorInfo;
    /// Si la factorización usó bloques 2×2 para evaluar su inercia.
    /// None indica que el backend no publica este dato.
    fn uses_2x2_pivots(&self) -> Option<bool> {
        None
    }
    /// Resuelve todas las columnas de `b` y las reemplaza por `x` en `Ax=b`.
    ///
    /// `b.rows()` debe coincidir con el orden del factor. Cero columnas es válido.
    /// ENG-E-0010 indica tamaño incompatible, ENG-E-0008 datos o resultado no
    /// finitos. Ante un error `b` se conserva completo, sin escritura parcial.
    fn solve_in_place(&self, b: &mut DMat) -> EResult<()>;
    /// Devuelve una solución nueva, conservando la matriz de cargas original.
    fn solve(&self, b: &DMat) -> EResult<DMat> {
        let mut x = b.clone();
        self.solve_in_place(&mut x)?;
        Ok(x)
    }
}

/// Todos los autovalores densos ordenados y sus autovectores como columnas.
#[derive(Clone, Debug, PartialEq)]
pub struct GeneralizedEigen {
    /// Valores ascendentes de λ en `Kφ=λMφ`; unidades de K divididas por M.
    pub eigenvalues: Vec<Real>,
    /// Columnas normalizadas: `φᵢᵀMφⱼ=δᵢⱼ` dentro de precisión numérica.
    /// La componente de mayor módulo es positiva; multiplicidades pueden
    /// producir bases distintas del mismo subespacio según el backend.
    pub vectors: DMat,
}

/// Contrato independiente del proveedor de álgebra lineal.
///
/// Cada método conserva sus entradas y valida finitud/dimensiones. No modifica
/// rigidez ni masa mediante desplazamientos o regularización ocultos. Los
/// índices matriciales son la numeración de GDL que aporta el llamador.
/// Factorizar/solve usa ENG-E-0006 para singularidad con entidad GDL;
/// autovalores usa ENG-E-0007 si no converge. La masa debe ser definida positiva:
/// una masa singular se rechaza con ENG-E-0006 sin producir NaN; una masa
/// indefinida con ENG-E-0010. La reducción explícita de GDL sin masa está en [`guyan_condense`].
/// En factorización simétrica y masa (Cholesky) la matriz se equilibra antes con
/// `S A S` (equilibrado iterativo tipo Ruiz) para tolerar GDL de escalas muy
/// distintas. Esto MITIGA, no garantiza, la invariancia al reescalado diagonal
/// `D A D`: es exacta para matrices diagonales y fiable en acopladas de escalado
/// moderado, pero no se promete para rangos extremos. Un pivote es
/// numéricamente singular si su módulo es
/// ≤ `64 ε max(n,1)` tras dividir la matriz (equilibrada) por su entrada de mayor módulo.
/// LÍMITE de `solve_in_place`: cada columna de `b` se normaliza por su entrada de
/// mayor módulo; si las entradas de una misma columna abarcan más de ~1e300 de
/// rango (cociente < ~1e-300) el componente pequeño subdesborda y se pierde
/// (aporta 0 a la solución). Es un límite de f64 y no se señala como error.
/// Por ello no se garantiza resolver matrices muy mal condicionadas. La
/// política es numérica, adimensional y explícita; no es una tolerancia física.
pub trait LinAlgBackend {
    /// Nombre descriptivo propio del backend, sin dependencia del formato externo.
    fn name(&self) -> &'static str;
    /// Calcula `a b`; ENG-E-0010 si `a.cols()!=b.rows()`.
    fn matmul(&mut self, a: &DMat, b: &DMat) -> EResult<DMat>;
    /// Transpone una matriz rectangular, conservando sus magnitudes.
    fn transpose(&mut self, a: &DMat) -> EResult<DMat>;
    /// Resuelve un sistema denso cuadrado, posiblemente no simétrico, con varias RHS.
    /// ENG-E-0010 por cuadratura/tamaños y ENG-E-0006 por singularidad.
    fn solve_dense(&mut self, a: &DMat, b: &DMat) -> EResult<DMat>;
    /// Ensambla tripletas generales en CSR, sumando duplicados en orden de entrada.
    ///
    /// Cada tripleta representa una sola entrada, sin reflejo automático ni
    /// división por dos. Para simetría se aportan ambas mitades. Los ceros
    /// explícitos se retienen en el patrón. ENG-E-0010 por índices fuera de rango
    /// y ENG-E-0008 por valores o sumas no finitos.
    fn assemble(&mut self, rows: usize, cols: usize, entries: &[Triplet]) -> EResult<SparseMatrix> {
        SparseMatrix::from_triplets(rows, cols, entries)
    }
    /// Factoriza una matriz simétrica densa una sola vez, sin regularización.
    /// El factor posee sus datos y puede sobrevivir al backend y a la matriz.
    fn factorize(&mut self, a: &SymMatrix) -> EResult<Box<dyn SymFactorization>>;
    /// Factoriza consultando un control que puede interrumpir con su propio error.
    /// Por defecto consulta antes de factorizar; ReferenceBackend consulta además
    /// entre bloques de hasta 16 columnas, sin alterar las operaciones numéricas.
    fn factorize_with_check(
        &mut self,
        a: &SymMatrix,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<Box<dyn SymFactorization>> {
        check()?;
        self.factorize(a)
    }
    /// Inercia (positivos, negativos, nulos) por congruencia LDLᵀ.
    ///
    /// Por defecto consulta el factor; ENG-E-0013 si no ofrece inercia.
    /// El backend de referencia admite matrices singulares para este conteo,
    /// con la misma tolerancia numérica de pivotes que `factorize`, pero sin
    /// entregar un factor singular para resolver. Otros backends pueden
    /// devolver ENG-E-0006 si el desplazamiento coincide con un autovalor.
    fn symmetric_inertia(&mut self, a: &SymMatrix) -> EResult<(usize, usize, usize)> {
        self.factorize(a)?
            .info()
            .inertia
            .ok_or_else(|| unsupported("El backend no ofrece inercia para el control de Sturm."))
    }
    /// Resuelve el problema generalizado denso con masa definida positiva.
    /// ENG-E-0010 si los órdenes difieren; véase la política de masa del trait.
    fn generalized_eigen(&mut self, k: &SymMatrix, m: &SymMatrix) -> EResult<GeneralizedEigen>;
    /// Modos más bajos normalizados a masa con control de completitud de Sturm.
    /// Véanse [`lowest_modes`] y [`LowestModes`] para tolerancias y diagnósticos.
    fn lowest_modes(&mut self, k: &SymMatrix, m: &SymMatrix, count: usize) -> EResult<LowestModes> {
        lowest_modes(self, k, m, count)
    }
}

pub(crate) fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}
/// Límite de una reserva individual de almacenamiento numérico, por destino.
///
/// Evita pánicos de capacidad (`capacity overflow`) y abortos por falta de
/// memoria ante dimensiones extremas; más allá de este tamaño el motor denso no
/// es el camino correcto (solver disperso, ENG-LA-008).
/// En destinos de 64 bits el tope es 8 GiB; en destinos de 32 bits (p. ej.
/// wasm32) es 1 GiB, explícito, porque `isize::MAX` (≈2 GiB) no limitaría en la práctica.
#[cfg(target_pointer_width = "64")]
pub(crate) const MAX_ALLOC_BYTES: usize = 1 << 33;

// 8 GiB no cabe en usize de 32 bits: tope explícito de 1 GiB.
#[cfg(not(target_pointer_width = "64"))]
pub(crate) const MAX_ALLOC_BYTES: usize = 1 << 30;

/// Crea un `Vec` de `len` copias de `fill` con reserva fallible y acotada.
/// ENG-E-0010 con diagnóstico (elementos, bytes, límite) si excede el límite o
/// la reserva falla; nunca provoca pánico.
pub(crate) fn alloc_vec<T: Clone>(len: usize, fill: T) -> EResult<Vec<T>> {
    let bytes = len
        .checked_mul(std::mem::size_of::<T>())
        .filter(|&b| b <= MAX_ALLOC_BYTES)
        .ok_or_else(|| {
            invalid(&format!(
                "Álgebra lineal: la reserva de {len} elementos excede el límite de {MAX_ALLOC_BYTES} bytes."
            ))
        })?;
    let mut v = Vec::new();
    v.try_reserve_exact(len).map_err(|_| {
        invalid(&format!(
            "Álgebra lineal: no se pudo reservar {bytes} bytes para {len} elementos."
        ))
    })?;
    v.resize(len, fill);
    Ok(v)
}
pub(crate) fn unsupported(message: &str) -> EngineError {
    EngineError::new("ENG-E-0013", message)
}
pub(crate) fn size(actual: usize, expected: usize) -> EResult<()> {
    if actual != expected {
        return Err(invalid(&format!(
            "Álgebra lineal: tamaño {actual}, esperado {expected}."
        )));
    }
    Ok(())
}
pub(crate) fn finite(values: &[Real]) -> EResult<()> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err(EngineError::new(
            "ENG-E-0008",
            "Álgebra lineal: entrada o resultado no finito.",
        ));
    }
    Ok(())
}
pub(crate) fn singular(operation: &str, index: usize) -> EngineError {
    let mut error = EngineError::new(
        "ENG-E-0006",
        format!("{operation}: pivote singular en GDL {index}."),
    );
    if let Ok(index) = u32::try_from(index) {
        error.entities.push(EntityRef::Dof(DofIndex(index)));
    }
    error
}

#[cfg(test)]
mod max_alloc_tests {
    use super::*;

    #[test]
    fn max_alloc_bytes_is_explicit_per_target() {
        #[cfg(target_pointer_width = "64")]
        assert_eq!(MAX_ALLOC_BYTES, 8usize << 30);
        #[cfg(not(target_pointer_width = "64"))]
        {
            assert_eq!(MAX_ALLOC_BYTES, 1usize << 30);
            assert!(MAX_ALLOC_BYTES < isize::MAX as usize);
        }
    }

    #[test]
    fn alloc_vec_rejects_above_limit() {
        let too_many = MAX_ALLOC_BYTES / std::mem::size_of::<f64>() + 1;
        assert!(alloc_vec(too_many, 0.0f64).is_err());
    }
}
