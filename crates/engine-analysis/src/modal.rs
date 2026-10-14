//! Análisis `modal`: K constante, masas SI y vibración libre no amortiguada.
//!
//! Usa K_ff y M_ff de [`LinearSystem`] y `lowest_modes`, sin factores normativos.
//! Los modos globales tienen cero en apoyos, incluso con asentamientos: son
//! perturbaciones alrededor del equilibrio, no desplazamientos estáticos.
//! Γ=φᵀM_ff r_f/(φᵀM_ffφ), M*= (φᵀM_ff r_f)² para φ normalizado a masa.
//! La dirección del usuario se normaliza con `rigid_translation`.
//!
//! Con TODOS los modos, ΣM*=r_fᵀM_ff r_f. La masa global usada como denominador
//! de las fracciones incluye apoyos y puede ser mayor que esta masa libre.
//! No se usa M_fp r_p para excitar los GDL libres. Para una matriz consistente
//! con apoyos, la identidad de completitud es la de M_ff, no la masa global.
//!
//! No condensa ni descarta GDL sin masa: esa reducción queda en ENG-LA-009/ASM.
//! M_ff singular propaga ENG-E-0006 y su nodo/GDL cuando el solver aporta índice.
//! También propaga el corte de multiplicidad y los límites densos de
//! [`engine_linalg::lowest_modes`]. No distingue mecanismos de modos rígidos.

use engine_assembly::LinearSystem;
use engine_contracts::MassKind;
use engine_core::{EResult, EngineError, Real};
use engine_linalg::{LinAlgBackend, SparseMatrix};
use engine_loads::mass::{rigid_translation, total_mass};

/// Método de vibración libre lineal sin estado propio.
pub struct Modal;

/// Espectro seleccionado y sumas direccionales, en el orden pedido.
#[derive(Clone, Debug, PartialEq)]
pub struct ModalResults {
    /// Tipo de masa explícitamente elegido por el usuario.
    pub mass_kind: MassKind,
    /// Modos en orden ascendente de frecuencia.
    pub modes: Vec<ModalMode>,
    /// Totales de cada dirección, en el orden de entrada.
    pub directions: Vec<ModalDirection>,
    /// Modos devueltos con rigidez numéricamente nula: rígidos o mecanismos.
    pub rigid_mode_count: usize,
}

/// Un modo normalizado a masa, sin amortiguamiento.
#[derive(Clone, Debug, PartialEq)]
pub struct ModalMode {
    /// λ=ω², en s⁻².
    pub eigenvalue: Real,
    /// Frecuencia angular ω, en rad/s.
    pub angular_frequency: Real,
    /// Frecuencia f=ω/(2π), en Hz.
    pub frequency_hz: Real,
    /// Periodo T=2π/ω, en s; None significa periodo infinito (λ=0).
    pub period: Option<Real>,
    /// φ en orden reducido, φᵀM_ffφ=1.
    pub free_shape: Vec<Real>,
    /// φ en orden global, con ceros en los GDL prescritos.
    pub global_shape: Vec<Real>,
    /// Participación por dirección, en el orden de entrada.
    pub participation: Vec<ModalParticipation>,
}

/// Participación y masa efectiva de un modo para una dirección unitaria.
#[derive(Clone, Debug, PartialEq)]
pub struct ModalParticipation {
    /// Γ=φᵀM_ff r_f/(φᵀM_ffφ), con signo de la forma modal.
    pub factor: Real,
    /// M*=(φᵀM_ff r_f)², en kg, con φ normalizado a masa.
    pub effective_mass: Real,
    /// Suma de M* hasta este modo inclusive, en kg.
    pub cumulative_effective_mass: Real,
    /// M*/masa global direccional; None si la masa total es cero.
    pub mass_fraction: Option<Real>,
    /// Masa acumulada/masa global direccional; None si la masa total es cero.
    pub cumulative_mass_fraction: Option<Real>,
}

/// Totales direccionales para el conjunto de modos solicitado.
#[derive(Clone, Debug, PartialEq)]
pub struct ModalDirection {
    /// Dirección original del usuario; la evaluación usa su versión unitaria.
    pub direction: [Real; 3],
    /// Masa global rᵀMr en kg, incluidos apoyos (`engine_loads::mass::total_mass`).
    pub total_mass: Real,
    /// Masa r_fᵀM_ff r_f en kg: límite de la suma con todos los modos libres.
    pub free_mass: Real,
    /// Suma de masas efectivas de los modos solicitados, en kg.
    pub cumulative_effective_mass: Real,
    /// Suma anterior/masa global; None si la masa total es cero.
    pub cumulative_mass_fraction: Option<Real>,
}

impl Modal {
    /// Identificador público del método.
    pub const KIND: &'static str = "modal";

    /// Calcula exactamente `count` modos y participación en cada dirección.
    /// No modifica los estados aceptado o de prueba. `count=0` devuelve listas
    /// modales vacías; conserva la semántica de validación de `lowest_modes`.
    /// ENG-E-0010: lista de direcciones vacía, dirección nula, count>n;
    /// ENG-E-0008: no finitos/desbordamientos; ENG-E-0011: dirección con GDL
    /// ausentes; ENG-E-0012: resultado del backend incompatible. Propaga los
    /// demás diagnósticos de masa/espectro descritos en el módulo.
    pub fn run<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        count: usize,
        kind: MassKind,
        directions: &[[Real; 3]],
    ) -> EResult<ModalResults> {
        Self::run_with(system, count, kind, directions, |_| Ok(()))
    }

    /// Variante cooperativa: callback tras el eigensolver (0) y cada modo.
    /// La extracción de autovalores es indivisible; no acepta estado.
    pub fn run_with<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        count: usize,
        kind: MassKind,
        directions: &[[Real; 3]],
        mut checkpoint: impl FnMut(usize) -> EResult<()>,
    ) -> EResult<ModalResults> {
        if directions.is_empty() {
            return Err(EngineError::new(
                "ENG-E-0010",
                "Análisis modal: faltan direcciones.",
            ));
        }
        let free = system.constraints().free_dofs();
        let mut translations = Vec::new();
        let mut totals = Vec::new();
        for &direction in directions {
            let mut r = rigid_translation(system.dof_map(), direction)?;
            let total = total_mass(system.dof_map(), system.global_mass(kind), direction)?;
            for (value, prescribed) in r.iter_mut().zip(system.constraints().prescribed_values()) {
                if prescribed.is_some() {
                    *value = 0.0;
                }
            }
            let free_mass = product(system.global_mass(kind), &r, &r)?;
            totals.push(ModalDirection {
                direction,
                total_mass: total,
                free_mass,
                cumulative_effective_mass: 0.0,
                cumulative_mass_fraction: fraction(0.0, total)?,
            });
            translations.push(r);
        }
        let n = free.len();
        let spectrum = system.lowest_modes(kind, count)?;
        if spectrum.eigenvalues.len() != count
            || spectrum.vectors.rows() != n
            || spectrum.vectors.cols() != count
            || spectrum.rigid_mode_count > count
        {
            return Err(EngineError::new(
                "ENG-E-0012",
                "Análisis modal: dimensiones del espectro incompatibles.",
            ));
        }
        super::finite(&spectrum.eigenvalues)?;
        super::finite(spectrum.vectors.values())?;
        if spectrum.eigenvalues.iter().any(|&v| v < 0.0)
            || spectrum.eigenvalues.windows(2).any(|v| v[0] > v[1])
        {
            return Err(EngineError::new(
                "ENG-E-0012",
                "Análisis modal: espectro negativo o sin ordenar.",
            ));
        }
        let free = system.constraints().free_dofs();
        checkpoint(0)?;
        let matrix = system.global_mass(kind);
        let mut modes = Vec::new();
        for (col, &eigenvalue) in spectrum.eigenvalues.iter().enumerate() {
            let mut global = vec![0.0; system.dof_map().len()];
            for (row, index) in free.iter().enumerate() {
                global[index.0 as usize] = spectrum.vectors.values()[row * count + col];
            }
            let norm = product(matrix, &global, &global)?;
            if norm <= 0.0 {
                return Err(EngineError::new(
                    "ENG-E-0012",
                    "Análisis modal: forma modal sin norma de masa positiva.",
                ));
            }
            for value in &mut global {
                *value /= norm.sqrt();
            }
            super::finite(&global)?;
            let denominator = product(matrix, &global, &global)?;
            if denominator <= 0.0 {
                return Err(EngineError::new(
                    "ENG-E-0012",
                    "Análisis modal: norma de masa normalizada no positiva.",
                ));
            }
            let mut participation = Vec::new();
            for (r, total) in translations.iter().zip(&mut totals) {
                let projection = product(matrix, &global, r)?;
                let factor = projection / denominator;
                let effective_mass = projection * projection;
                total.cumulative_effective_mass += effective_mass;
                super::finite(&[factor, effective_mass, total.cumulative_effective_mass])?;
                total.cumulative_mass_fraction =
                    fraction(total.cumulative_effective_mass, total.total_mass)?;
                participation.push(ModalParticipation {
                    factor,
                    effective_mass,
                    cumulative_effective_mass: total.cumulative_effective_mass,
                    mass_fraction: fraction(effective_mass, total.total_mass)?,
                    cumulative_mass_fraction: total.cumulative_mass_fraction,
                });
            }
            let angular_frequency = eigenvalue.sqrt();
            let frequency_hz = angular_frequency / std::f64::consts::TAU;
            let period = if eigenvalue == 0.0 {
                None
            } else {
                Some(std::f64::consts::TAU / angular_frequency)
            };
            super::finite(&[angular_frequency, frequency_hz])?;
            if let Some(t) = period {
                super::finite(&[t])?;
            }
            modes.push(ModalMode {
                eigenvalue,
                angular_frequency,
                frequency_hz,
                period,
                free_shape: free.iter().map(|i| global[i.0 as usize]).collect(),
                global_shape: global,
                participation,
            });
            checkpoint(col + 1)?;
        }
        Ok(ModalResults {
            mass_kind: kind,
            modes,
            directions: totals,
            rigid_mode_count: spectrum.rigid_mode_count,
        })
    }
}

fn fraction(value: Real, total: Real) -> EResult<Option<Real>> {
    if total == 0.0 {
        return Ok(None);
    }
    let fraction = value / total;
    super::finite(&[fraction])?;
    Ok(Some(fraction))
}

fn product(matrix: &SparseMatrix, a: &[Real], b: &[Real]) -> EResult<Real> {
    let mut sum = 0.0;
    for (row, &value) in a.iter().enumerate() {
        for p in matrix.row_offsets()[row]..matrix.row_offsets()[row + 1] {
            let term = (value * matrix.values()[p]) * b[matrix.column_indices()[p]];
            super::finite(&[term])?;
            sum += term;
            super::finite(&[sum])?;
        }
    }
    Ok(sum)
}
