//! P-Delta elástico con axiales del caso de gravedad lineal, sin factores normativos.
//!
//! Se congela Kg en u_g: los productores recuperan el axial de cada miembro con
//! su geometría original. Se resuelve el caso lateral mediante iteración de
//! punto fijo `K_ff u_{i+1} = F_f − K_fp u_p − (Kg u_i)_f`, equivalente a
//! `(K + Kg)u = F`. No se actualizan ejes, materiales ni axiales por la carga
//! lateral; esta es una aproximación de pretensión inicial, no gran deformación.
//! La inercia LDLᵀ de `(K + Kg)_ff` se comprueba antes de iterar; un mecanismo
//! sigue siendo ENG-E-0006. Una matriz estable puede no converger por punto fijo
//! (p. ej. tracción muy grande): se reporta sin cambiar estrategia ni reintentar.
//! Los casos son independientes: u lateral no incluye u_g; los prescritos no se
//! escalan. No se acepta ni modifica el estado del sistema. Véase la derivación
//! y las tolerancias analíticas en `provenance/ENG-ANL-003.md`.

use crate::{finite, StaticCaseResult, StaticLinear};
use engine_assembly::{GeometricProducer, LinearSystem, Reaction};
use engine_contracts::{CaseRef, Element, ElementCtx, SystemView};
use engine_core::{DofIndex, EResult, EngineError, Real};
use engine_linalg::{DMat, LinAlgBackend, SparseMatrix, SymMatrix, Triplet};
use std::{collections::BTreeSet, sync::Arc};

/// Ajustes explícitos del punto fijo, sin valores normativos ni reintentos.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PDeltaOptions {
    /// Tolerancia relativa positiva y menor que uno, para cambio y residual.
    pub tolerance: Real,
    /// Máximo de pasos laterales; positivo. La gravedad no cuenta como paso.
    pub max_iterations: usize,
    /// Escala positiva en m para normalizar traslaciones (no modifica ecuaciones).
    pub displacement_scale: Real,
    /// Escala positiva en rad para normalizar giros.
    pub rotation_scale: Real,
}

/// Terminación física/numerérica del método, distinta de errores de entrada.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PDeltaOutcome {
    /// Cambio y residual relativos cumplen la tolerancia del usuario.
    Completed,
    /// Se agotó exactamente max_iterations; no se aceptó ningún estado.
    NotConverged,
    /// Pérdida de estabilidad por inercia LDLᵀ de la rigidez reducida total.
    Unstable {
        /// Número de autovalores negativos (incluye signos de bloques 2×2).
        negative_eigenvalues: usize,
    },
}

/// Reporte de cada paso lateral; no hay iteraciones ocultas.
#[derive(Clone, Debug, PartialEq)]
pub struct PDeltaIteration {
    /// Número de paso, comenzando en uno.
    pub iteration: usize,
    /// Desplazamiento libre lateral de esta prueba, m/rad.
    pub free_displacements: Vec<Real>,
    /// ‖Δu/s‖∞ / max(1, ‖u_nuevo/s‖∞, ‖u_previo/s‖∞).
    /// s es displacement_scale o rotation_scale según el GDL.
    pub relative_displacement_change: Real,
    /// ‖((K+Kg)u−F)_f‖∞ en componentes SI N/N·m.
    pub free_residual_norm: Real,
    /// Residual / max(1, ‖((K+Kg)u)_f‖∞, ‖F_f‖∞).
    /// Indicador SI mixto; no es una norma física invariante de unidades.
    pub relative_free_residual: Real,
}

/// Resultado lateral y gravedad usada como pretensión, sin aceptación implícita.
#[derive(Clone, Debug, PartialEq)]
pub struct PDeltaResult {
    /// Terminación del análisis lateral.
    pub outcome: PDeltaOutcome,
    /// Ajustes efectivamente utilizados, incluidas escalas y tolerancia.
    pub options: PDeltaOptions,
    /// Caso de gravedad lineal previo; axiales evaluados en sus desplazamientos.
    pub gravity: StaticCaseResult,
    /// Caso lateral independiente.
    pub lateral_case: CaseRef,
    /// Factor de cargas lateral, sin escalar prescritos.
    pub lateral_factor: Real,
    /// Inercia (positivos, negativos, nulos) de `(K+Kg)_ff`, dada por LDLᵀ.
    pub inertia: (usize, usize, usize),
    /// Bloques 2×2 usados en la inercia; None si el backend no lo informa.
    pub inertia_uses_2x2: Option<bool>,
    /// Reportes ordenados; vacío para Unstable (no se resuelve carga lateral).
    pub iterations: Vec<PDeltaIteration>,
    /// Última prueba lateral global, o None para Unstable.
    pub global_displacements: Option<Vec<Real>>,
    /// Reacciones `(K+Kg)u−F` en GDL prescritos de la última prueba.
    /// Vacío para Unstable. No son reacciones de la combinación gravedad+lateral.
    pub reactions: Vec<Reaction>,
}

/// Análisis P-Delta con pretensión inicial y punto fijo explícito.
pub struct PDelta;

impl PDelta {
    /// Identificador público del método.
    pub const KIND: &'static str = "pdelta";

    /// Primero StaticLinear(gravedad), luego Kg(u_g) y el caso lateral iterado.
    /// Deben registrarse productores para evaluar Kg fuera de su referencia
    /// estática (o declararse explícitamente la referencia correcta).
    /// ENG-E-0008: no finitos; ENG-E-0010: ajustes inválidos; ENG-E-0003: caso
    /// ausente; ENG-E-0006: mecanismo con nodo/GDL; ENG-E-0013: backend sin
    /// inercia; ENG-E-0012: diagnósticos incoherentes. Sin cambio de estados.
    pub fn run<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        gravity: &(CaseRef, Real),
        lateral: &(CaseRef, Real),
        options: PDeltaOptions,
    ) -> EResult<PDeltaResult> {
        Self::run_with(system, gravity, lateral, options, |_| Ok(()))
    }

    /// Variante cooperativa: callback tras gravedad (0) y cada iteración lateral.
    /// Un error del callback detiene el análisis sin aceptar estado.
    pub fn run_with<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        gravity: &(CaseRef, Real),
        lateral: &(CaseRef, Real),
        options: PDeltaOptions,
        mut checkpoint: impl FnMut(usize) -> EResult<()>,
    ) -> EResult<PDeltaResult> {
        finite(&[
            gravity.1,
            lateral.1,
            options.tolerance,
            options.displacement_scale,
            options.rotation_scale,
        ])?;
        if options.tolerance <= 0.0
            || options.tolerance >= 1.0
            || options.max_iterations == 0
            || options.displacement_scale <= 0.0
            || options.rotation_scale <= 0.0
        {
            return Err(EngineError::new(
                "ENG-E-0010",
                "P-Delta: ajustes inválidos.",
            ));
        }
        // Validar también el caso lateral antes de cualquier factorización.
        let force = system.global_force(&lateral.0, lateral.1)?;
        let elastic_rhs = system.reduced_rhs(&lateral.0, lateral.1)?;
        let gravity = StaticLinear::run(system, std::slice::from_ref(gravity))?
            .cases
            .into_iter()
            .next()
            .ok_or_else(|| {
                EngineError::new("ENG-E-0012", "P-Delta: falta solución de gravedad.")
            })?;
        checkpoint(0)?;
        let kg = system.geometric_at(&gravity.free_displacements)?;
        let total = sum(system.stiffness(), &kg)?;
        let total_free = reduce(system, &total)?;
        let total_factor = system.factorize_reduced(&total_free)?;
        let inertia = total_factor.info().inertia.ok_or_else(|| {
            EngineError::new("ENG-E-0013", "P-Delta: el backend no ofrece inercia LDLᵀ.")
        })?;
        if inertia
            .0
            .checked_add(inertia.1)
            .and_then(|s| s.checked_add(inertia.2))
            != Some(total_free.order())
        {
            return Err(EngineError::new(
                "ENG-E-0012",
                "P-Delta: inercia incoherente.",
            ));
        }
        if inertia.2 != 0 {
            return Err(EngineError::new(
                "ENG-E-0006",
                "P-Delta: rigidez total singular.",
            ));
        }
        let mut result = PDeltaResult {
            outcome: if inertia.1 > 0 {
                PDeltaOutcome::Unstable {
                    negative_eigenvalues: inertia.1,
                }
            } else {
                PDeltaOutcome::NotConverged
            },
            options,
            gravity,
            lateral_case: lateral.0.clone(),
            lateral_factor: lateral.1,
            inertia,
            inertia_uses_2x2: total_factor.uses_2x2_pivots(),
            iterations: Vec::new(),
            global_displacements: None,
            reactions: Vec::new(),
        };
        if inertia.1 > 0 {
            return Ok(result);
        }
        // Este factor solo diagnosticó estabilidad, nunca se usa un eigensolver.
        drop(total_factor);
        let elastic = system.reduced_stiffness().clone();
        let factor = system.factorize_reduced(&elastic)?;
        let n = elastic.order();
        let mut previous = vec![0.0; n];
        let scales = (0..n)
            .map(|i| {
                let (_, dof) = system.try_dof_label(DofIndex(i as u32))?;
                Ok(if dof.index() < 3 {
                    options.displacement_scale
                } else {
                    options.rotation_scale
                })
            })
            .collect::<EResult<Vec<_>>>()?;
        for iteration in 1..=options.max_iterations {
            let old_global = system.constraints().expand(&previous)?;
            let geometric_force = multiply(&kg, &old_global)?;
            let rhs = system
                .constraints()
                .free_dofs()
                .iter()
                .zip(&elastic_rhs)
                .map(|(index, f)| f - geometric_force[index.0 as usize])
                .collect();
            let rhs = DMat::new(n, 1, rhs)?;
            let solution = factor.solve(&rhs)?;
            if solution.rows() != n || solution.cols() != 1 {
                return Err(EngineError::new(
                    "ENG-E-0012",
                    "P-Delta: tamaño de solución inválido.",
                ));
            }
            finite(solution.values())?;
            let current = solution.values().to_vec();
            let global = system.constraints().expand(&current)?;
            let internal = multiply(&total, &global)?;
            let (norm, relative) = residual(system, &internal, &force)?;
            let change = displacement_change(&current, &previous, &scales)?;
            result.iterations.push(PDeltaIteration {
                iteration,
                free_displacements: current.clone(),
                relative_displacement_change: change,
                free_residual_norm: norm,
                relative_free_residual: relative,
            });
            result.reactions = reactions(system, &internal, &force)?;
            result.global_displacements = Some(global);
            checkpoint(iteration)?;
            if change <= options.tolerance && relative <= options.tolerance {
                result.outcome = PDeltaOutcome::Completed;
                break;
            }
            previous = current;
        }
        Ok(result)
    }
}

/// Adaptador poseído para frame2d, barras u otros elementos que proporcionen Kg.
/// Captura coordenadas, parámetros e historia aceptada sin aceptar nuevas pruebas.
/// Cada evaluación delega en Element::geometric_stiffness, usando sus axiales
/// propios (tracción positiva). No estima N a partir de la carga total del modelo.
/// ENG-E-0010 por tamaños; ENG-E-0002 por GDL repetidos; ENG-E-0008 por no
/// finitos; ENG-E-0013 si el elemento no soporta Kg. El sistema valida el rango
/// global al registrar mediante set_geometric_producers.
pub fn geometric_producer(
    element: Arc<dyn Element>,
    ctx: &ElementCtx<'_>,
    location: Vec<DofIndex>,
    committed: Vec<Real>,
) -> EResult<GeometricProducer> {
    if element.n_nodes().checked_mul(element.dofs_per_node().len()) != Some(location.len())
        || committed.len() != element.state_len()
    {
        return Err(EngineError::new(
            "ENG-E-0010",
            "P-Delta: tamaños elementales inválidos.",
        ));
    }
    if location.iter().collect::<BTreeSet<_>>().len() != location.len() {
        return Err(EngineError::new(
            "ENG-E-0002",
            "P-Delta: GDL elemental repetido.",
        ));
    }
    finite(&committed)?;
    let evaluate = |ctx: &ElementCtx<'_>, u: &[Real]| {
        element
            .geometric_stiffness(ctx, u, &committed)?
            .ok_or_else(|| {
                EngineError::new("ENG-E-0013", "P-Delta: el elemento no proporciona Kg.")
            })
    };
    evaluate(ctx, &vec![0.0; location.len()])?;
    let dim = ctx.dim;
    let coords = ctx.coords.to_vec();
    let params = ctx.params.clone();
    Ok(GeometricProducer {
        location,
        eval: Box::new(move |u| {
            let ctx = ElementCtx {
                dim,
                coords: &coords,
                params: &params,
            };
            element
                .geometric_stiffness(&ctx, u, &committed)?
                .ok_or_else(|| {
                    EngineError::new("ENG-E-0013", "P-Delta: el elemento no proporciona Kg.")
                })
        }),
    })
}

fn sum(a: &SparseMatrix, b: &SparseMatrix) -> EResult<SparseMatrix> {
    let entries = [a, b]
        .into_iter()
        .flat_map(|m| {
            (0..m.rows()).flat_map(move |row| {
                (m.row_offsets()[row]..m.row_offsets()[row + 1]).map(move |p| Triplet {
                    row,
                    col: m.column_indices()[p],
                    value: m.values()[p],
                })
            })
        })
        .collect::<Vec<_>>();
    SparseMatrix::from_triplets(a.rows(), a.cols(), &entries)
}

fn reduce<B: LinAlgBackend>(system: &LinearSystem<B>, matrix: &SparseMatrix) -> EResult<SymMatrix> {
    let mut entries = Vec::new();
    for (row, index) in system.constraints().free_dofs().iter().enumerate() {
        let global = index.0 as usize;
        for p in matrix.row_offsets()[global]..matrix.row_offsets()[global + 1] {
            if let Some(col) = system
                .constraints()
                .reduced_index(DofIndex(matrix.column_indices()[p] as u32))?
            {
                entries.push(Triplet {
                    row,
                    col: col.0 as usize,
                    value: matrix.values()[p],
                });
            }
        }
    }
    let n = system.n_free_dofs();
    SymMatrix::new(SparseMatrix::from_triplets(n, n, &entries)?.to_dense()?)
}

fn multiply(matrix: &SparseMatrix, u: &[Real]) -> EResult<Vec<Real>> {
    let mut result = vec![0.0; matrix.rows()];
    for (row, value) in result.iter_mut().enumerate() {
        for p in matrix.row_offsets()[row]..matrix.row_offsets()[row + 1] {
            let term = matrix.values()[p] * u[matrix.column_indices()[p]];
            finite(&[term])?;
            *value += term;
            finite(&[*value])?;
        }
    }
    Ok(result)
}

fn displacement_change(current: &[Real], previous: &[Real], scales: &[Real]) -> EResult<Real> {
    let mut change: Real = 0.0;
    let mut denominator: Real = 1.0;
    for ((u, old), scale) in current.iter().zip(previous).zip(scales) {
        let a = u / scale;
        let b = old / scale;
        let delta = a - b;
        finite(&[a, b, delta])?;
        change = change.max(delta.abs());
        denominator = denominator.max(a.abs()).max(b.abs());
    }
    Ok(change / denominator)
}

fn residual<B: LinAlgBackend>(
    system: &LinearSystem<B>,
    internal: &[Real],
    force: &[Real],
) -> EResult<(Real, Real)> {
    let mut norm: Real = 0.0;
    let mut scale: Real = 1.0;
    for index in system.constraints().free_dofs() {
        let i = index.0 as usize;
        let value = internal[i] - force[i];
        finite(&[value])?;
        norm = norm.max(value.abs());
        scale = scale.max(internal[i].abs()).max(force[i].abs());
    }
    Ok((norm, norm / scale))
}

fn reactions<B: LinAlgBackend>(
    system: &LinearSystem<B>,
    internal: &[Real],
    force: &[Real],
) -> EResult<Vec<Reaction>> {
    let mut result = Vec::new();
    for (i, prescribed) in system.constraints().prescribed_values().iter().enumerate() {
        if prescribed.is_some() {
            let (node, dof) = system.dof_map().label(DofIndex(i as u32))?;
            let value = internal[i] - force[i];
            finite(&[value])?;
            result.push(Reaction { node, dof, value });
        }
    }
    Ok(result)
}
