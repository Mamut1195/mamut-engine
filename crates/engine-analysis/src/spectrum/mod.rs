//! Espectro de respuesta `spectrum`, exclusivamente tabulado por el usuario.
//!
//! Sa es pseudoaceleración en m/s² para el ζ declarado de cada tabla;
//! q_n,max=Γ_n Sa(T_n,ζ_n)/ω_n² y u_n=φ_n q_n,max. Las fuerzas equivalentes
//! globales son F_n=Mφ_n Γ_n Sa. K, M y participación vienen de [`Modal`].
//! Los modos deben tener ω>0; no se asigna respuesta a modos rígidos.
//!
//! R_n=(K u_n−F_n)_p en todos los apoyos, con perturbaciones prescritas cero.
//! El cortante basal firmado es −Σ_apoyos R_traslacional·d̂. Se incluyen los
//! acoplamientos de masa consistente hacia apoyos en F_p; para masas solo en
//! GDL libres de un edificio de cortante, V_n=M*_n Sa_n. No se suman cargas
//! estáticas, asentamientos ni términos M_fp r_p a la participación modal.
//!
//! SRSS/CQC combinan CADA escalar después de recuperarlo, usando su signo
//! modal. Sus envolventes no constituyen un estado simultáneo ni equilibrado:
//! el cortante basal combinado se calcula a partir de los cortantes por modo,
//! nunca sumando las envolventes de reacciones. Para miembros, extraer u_e de
//! cada `global_displacements`, usar `Element::recover` sin cargas estáticas y
//! combinar cada escalar con [`combine_modal`]. El sistema ensamblado no retiene
//! objetos Element/contextos ni dispone de una fachada results completa.
//!
//! El usuario elige tabla(s), amortiguamiento(s), interpolación, extrapolación,
//! dirección(es), factores y combinación. No hay normas, espectros incorporados,
//! corrección de amortiguamiento, gravedad ni factores predeterminados.

mod combination;
mod table;

pub use combination::{
    combine_directions, combine_modal, cqc_correlation, DirectionalCombination, ModalCombination,
};
pub use table::{Extrapolation, Interpolation, SpectrumUnits, TabulatedSpectrum};

use crate::{
    finite,
    modal::{Modal, ModalResults},
};
use combination::{correlations, quadratic};
use engine_assembly::{LinearSystem, Reaction};
use engine_contracts::MassKind;
use engine_core::{DofIndex, EResult, EngineError, Real};
use engine_linalg::{LinAlgBackend, SparseMatrix};

fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", format!("Espectro: {message}"))
}

fn validate_damping(zeta: Real) -> EResult<()> {
    finite(&[zeta])?;
    if !(0.0..1.0).contains(&zeta) {
        return Err(invalid("El amortiguamiento ζ debe pertenecer a [0,1)."));
    }
    Ok(())
}

/// Excitación declarada: una tabla común o una tabla por modo, en orden modal.
#[derive(Clone, Copy, Debug)]
pub struct SpectrumDirection<'a> {
    /// Dirección global del usuario, finita y no nula; se normaliza.
    pub direction: [Real; 3],
    /// Longitud 1 (ζ común) o `mode_count` (ζ/tablas por modo); nunca vacía.
    pub spectra: &'a [TabulatedSpectrum],
}

/// Opciones explícitas del usuario; no implementa Default.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrumOptions {
    /// Número no nulo de modos solicitado a [`Modal::run`].
    pub mode_count: usize,
    /// Tipo de masa del sistema.
    pub mass_kind: MassKind,
    /// Combinación de respuestas modales firmadas.
    pub modal_combination: ModalCombination,
    /// Combinación de envolventes direccionales.
    pub directional_combination: DirectionalCombination,
    /// Un factor finito no negativo por dirección, en el mismo orden.
    pub directional_factors: Vec<Real>,
}

/// Desplazamientos, reacciones y cortante basal de un modo o una envolvente.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrumResponse {
    /// Desplazamientos libres en orden reducido, m/rad.
    pub free_displacements: Vec<Real>,
    /// Desplazamientos globales, m/rad; perturbaciones cero en apoyos.
    pub global_displacements: Vec<Real>,
    /// Reacciones en orden global, N/N·m; firmadas por modo, magnitudes combinadas.
    pub reactions: Vec<Reaction>,
    /// Cortante basal proyectado en d̂, N; firmado por modo, magnitud combinada.
    pub base_shear: Real,
}

/// Respuesta modal máxima firmada en una dirección; no representa una historia.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrumModeResponse {
    /// Amortiguamiento ζ declarado en la tabla usada.
    pub damping: Real,
    /// Sa(T_n,ζ_n) tabulada/interpolada, m/s².
    pub spectral_acceleration: Real,
    /// Coordenada modal máxima Γ_n Sa/ω_n²; su signo depende del signo de φ.
    /// Las unidades de q dependen de la normalización a masa de φ; φq es m/rad.
    pub modal_displacement: Real,
    /// Fuerzas equivalentes globales Mφ_n Γ_n Sa, N/N·m, incluidos apoyos.
    pub equivalent_forces: Vec<Real>,
    /// Respuesta física firmada recuperada antes de combinar.
    pub response: SpectrumResponse,
}

/// Respuesta por dirección con trazabilidad de las tablas y combinaciones.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrumDirectionResult {
    /// Dirección unitaria global utilizada.
    pub unit_direction: [Real; 3],
    /// Tablas de entrada, incluida su elección de interpolación/extrapolación.
    pub spectra: Vec<TabulatedSpectrum>,
    /// Respuestas por modo, en el mismo orden que `modal.modes`.
    pub modes: Vec<SpectrumModeResponse>,
    /// Envolvente SRSS o CQC por componente.
    pub combined: SpectrumResponse,
}

/// Resultado completo; los estados aceptados y de prueba no se modifican.
#[derive(Clone, Debug, PartialEq)]
pub struct SpectrumResults {
    /// Modos y participaciones de ENG-ANL-002 para estas mismas direcciones.
    pub modal: ModalResults,
    /// Opciones declaradas y factores aplicados, sin interpretación normativa.
    pub options: SpectrumOptions,
    /// Resultados direccionales, en orden de entrada.
    pub directions: Vec<SpectrumDirectionResult>,
    /// Envolvente direccional; el basal combina las proyecciones de cada dirección.
    /// No es una proyección en una nueva dirección ni un equilibrio simultáneo.
    pub combined: SpectrumResponse,
}

/// Método espectral lineal sin estado propio ni espectro predeterminado.
pub struct Spectrum;

impl Spectrum {
    /// Identificador público del método.
    pub const KIND: &'static str = "spectrum";

    /// Calcula respuestas máximas modales, envolventes y combinación direccional.
    /// ENG-E-0010: modos/direcciones/tablas vacíos, ω=0, dirección nula,
    /// factores inválidos o periodo fuera de tabla con Extrapolation::Error.
    /// ENG-E-0008: no finitos/desbordamientos. Propaga los errores del módulo
    /// modal y del ensamblador. No modifica ni acepta estados del sistema.
    pub fn run<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        directions: &[SpectrumDirection<'_>],
        options: &SpectrumOptions,
    ) -> EResult<SpectrumResults> {
        Self::run_with(system, directions, options, |_, _| Ok(()))
    }

    /// Variante cooperativa: etapas modal, spectrumMode y spectrumDirection.
    /// Callback después de cada modo/dirección; el eigensolver es indivisible.
    /// Un error detiene el análisis sin aceptar estado.
    pub fn run_with<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        directions: &[SpectrumDirection<'_>],
        options: &SpectrumOptions,
        mut checkpoint: impl FnMut(&'static str, usize) -> EResult<()>,
    ) -> EResult<SpectrumResults> {
        if options.mode_count == 0 || directions.is_empty() {
            return Err(invalid("Se requieren modos y direcciones no vacíos."));
        }
        // Valida factores incluso si todas las respuestas son cero.
        combine_directions(
            &vec![0.0; directions.len()],
            &options.directional_factors,
            options.directional_combination,
        )?;
        for input in directions {
            if input.spectra.len() != 1 && input.spectra.len() != options.mode_count {
                return Err(invalid("Declarar una tabla común o una tabla por modo."));
            }
        }
        let input_directions: Vec<_> = directions.iter().map(|d| d.direction).collect();
        let modal = Modal::run_with(
            system,
            options.mode_count,
            options.mass_kind,
            &input_directions,
            |n| checkpoint("modal", n),
        )?;
        let frequencies: Vec<_> = modal.modes.iter().map(|m| m.angular_frequency).collect();
        let mut results = Vec::new();
        for (d, input) in directions.iter().enumerate() {
            let unit_direction = unit(input.direction)?;
            let dampings: Vec<_> = (0..options.mode_count)
                .map(|n| input.spectra[if input.spectra.len() == 1 { 0 } else { n }].damping())
                .collect();
            let rho = correlations(&frequencies, &dampings, options.modal_combination)?;
            let mut modes = Vec::new();
            for (n, mode) in modal.modes.iter().enumerate() {
                let table = &input.spectra[if input.spectra.len() == 1 { 0 } else { n }];
                let period = mode
                    .period
                    .ok_or_else(|| invalid("Modo rígido sin periodo finito."))?;
                let sa = table.acceleration(period)?;
                let acceleration = mode.participation[d].factor * sa;
                let q = acceleration / mode.eigenvalue;
                finite(&[acceleration, q])?;
                let global: Vec<_> = mode.global_shape.iter().map(|v| v * q).collect();
                finite(&global)?;
                let mut force =
                    multiply(system.global_mass(options.mass_kind), &mode.global_shape)?;
                for value in &mut force {
                    *value *= acceleration;
                }
                finite(&force)?;
                let internal = multiply(system.stiffness(), &global)?;
                let mut reactions = Vec::new();
                let mut base_shear = 0.0;
                for (i, prescribed) in system.constraints().prescribed_values().iter().enumerate() {
                    if prescribed.is_some() {
                        let (node, dof) = system.dof_map().label(DofIndex(i as u32))?;
                        let value = internal[i] - force[i];
                        finite(&[value])?;
                        if dof.index() < 3 {
                            base_shear -= value * unit_direction[dof.index()];
                            finite(&[base_shear])?;
                        }
                        reactions.push(Reaction { node, dof, value });
                    }
                }
                let free_displacements = system
                    .constraints()
                    .free_dofs()
                    .iter()
                    .map(|i| global[i.0 as usize])
                    .collect();
                modes.push(SpectrumModeResponse {
                    damping: table.damping(),
                    spectral_acceleration: sa,
                    modal_displacement: q,
                    equivalent_forces: force,
                    response: SpectrumResponse {
                        free_displacements,
                        global_displacements: global,
                        reactions,
                        base_shear,
                    },
                });
                checkpoint("spectrumMode", d * options.mode_count + n + 1)?;
            }
            let responses: Vec<_> = modes.iter().map(|m| &m.response).collect();
            let combined = combine_responses(&responses, |v| quadratic(v, &rho))?;
            results.push(SpectrumDirectionResult {
                unit_direction,
                spectra: input.spectra.to_vec(),
                modes,
                combined,
            });
            checkpoint("spectrumDirection", d + 1)?;
        }
        let responses: Vec<_> = results.iter().map(|d| &d.combined).collect();
        let combined = combine_responses(&responses, |v| {
            combine_directions(
                v,
                &options.directional_factors,
                options.directional_combination,
            )
        })?;
        Ok(SpectrumResults {
            modal,
            options: options.clone(),
            directions: results,
            combined,
        })
    }
}

fn unit(direction: [Real; 3]) -> EResult<[Real; 3]> {
    finite(&direction)?;
    let scale = direction.iter().fold(0.0_f64, |s, v| s.max(v.abs()));
    if scale == 0.0 {
        return Err(invalid("Dirección de excitación nula."));
    }
    let d = direction.map(|v| v / scale);
    let norm = d[0].hypot(d[1]).hypot(d[2]);
    Ok(d.map(|v| v / norm))
}

fn multiply(matrix: &SparseMatrix, vector: &[Real]) -> EResult<Vec<Real>> {
    if matrix.cols() != vector.len() {
        return Err(EngineError::new(
            "ENG-E-0012",
            "Producto matricial incompatible.",
        ));
    }
    finite(vector)?;
    let mut result = vec![0.0; matrix.rows()];
    for (i, out) in result.iter_mut().enumerate() {
        for p in matrix.row_offsets()[i]..matrix.row_offsets()[i + 1] {
            let term = matrix.values()[p] * vector[matrix.column_indices()[p]];
            finite(&[term])?;
            *out += term;
            finite(&[*out])?;
        }
    }
    Ok(result)
}

fn combine_responses(
    responses: &[&SpectrumResponse],
    mut combine: impl FnMut(&[Real]) -> EResult<Real>,
) -> EResult<SpectrumResponse> {
    let first = responses
        .first()
        .ok_or_else(|| invalid("Respuestas vacías."))?;
    let mut global = Vec::new();
    for i in 0..first.global_displacements.len() {
        global.push(combine(
            &responses
                .iter()
                .map(|r| r.global_displacements[i])
                .collect::<Vec<_>>(),
        )?);
    }
    let mut free = Vec::new();
    for i in 0..first.free_displacements.len() {
        free.push(combine(
            &responses
                .iter()
                .map(|r| r.free_displacements[i])
                .collect::<Vec<_>>(),
        )?);
    }
    let mut reactions = Vec::new();
    for (i, reaction) in first.reactions.iter().enumerate() {
        let value = combine(
            &responses
                .iter()
                .map(|r| r.reactions[i].value)
                .collect::<Vec<_>>(),
        )?;
        reactions.push(Reaction { value, ..*reaction });
    }
    let base_shear = combine(&responses.iter().map(|r| r.base_shear).collect::<Vec<_>>())?;
    Ok(SpectrumResponse {
        free_displacements: free,
        global_displacements: global,
        reactions,
        base_shear,
    })
}
