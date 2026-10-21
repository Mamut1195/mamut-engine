//! Consolidación Terzaghi 1D multicapa, elementos lineales y masa consistente.
//!
//! Contrato `consolidation/1`: z en m, positiva hacia abajo; t en s; exceso u y
//! Δσ en kPa, compresión positiva. mv en 1/kPa, k en m/s, cv en m²/s y γw en
//! kN/m³. Se exige cv=k/(mv γw), con discrepancia relativa máxima 1e-10.
//! No se deriva ni sustituye un parámetro ausente. Geometría independiente del
//! perfil geostático: se pueden reutilizar sus espesores, sin exigir γ/γsat/zw
//! que no intervienen en este problema incremental saturado.
//!
//! Ecuación conservativa: mv u̇ − ((k/γw)u′)′ = mv Δσ̇. Los nodos compartidos
//! imponen continuidad de u; el flujo se equilibra débilmente en interfaces.
//! Drenes ideales u=0, caras restantes impermeables. Carga uniforme en z,
//! inicial en reposo: Δσ(0) se aplica instantáneamente, u(0)=Δσ(0) en nodos
//! interiores y cero en drenes. Esta interpolación tiene una capa de error
//! inicial de tamaño h; no se afirma s(0)=0 exactamente en una malla finita.
//!
//! La tabla empieza en t=0, interpola linealmente y no extrapola. El integrador
//! divide en todos los quiebres de carga y tiempos de salida. θ=0.5 por defecto;
//! θ<0.5 exige estabilidad modal, verificada con la cota de Gershgorin del par
//! (K,M) documentada en provenance/ENG-GEO-006.md. No se recortan oscilaciones
//! de Crank–Nicolson ni resultados negativos durante descarga.
//!
//! `U=s/[Δσ∫mv dz]` es el grado medio ponderado por compresibilidad. Para Δσ=0
//! se devuelve None (también si existe presión residual). Con descarga U puede
//! salir de `[0,1]`. s=∫mv(Δσ−u)dz se integra exactamente para el campo lineal.
//! El resultado conserva todos los parámetros Assumed, incluidos carga y γw.
//!
//! Se reutiliza la factorización de engine-linalg; su backend de referencia es
//! denso, O(n³) al factorizar y O(n²) por solve. El ensamblaje es CSR. Límites
//! explícitos de esta API de referencia: 2048 elementos, 100000 salidas,
//! 8000000 valores nodales de salida y 1000000 subpasos por intervalo.
//! ENG-E-0010 por rango, malla, estabilidad o
//! cv incompatible; 0003 por dato ausente; 0008 por datos/resultados no finitos.
use crate::{Diagnostic, GeoResult, Parameter, Provenance, Real};
use engine_core::{EngineError, Severity};
use engine_linalg::{
    DMat, LinAlgBackend, ReferenceBackend, SparseMatrix, SymFactorization, SymMatrix, Triplet,
};

/// Capa saturada; todos los datos físicos llevan origen explícito.
#[derive(Clone, Debug)]
pub struct ConsolidationLayer {
    /// Espesor positivo en m, contiguo a la capa anterior desde z=0.
    pub thickness_m: Parameter,
    /// cv positivo en m²/s, requerido y contrastado con k/(mv γw).
    pub cv_m2_s: Option<Parameter>,
    /// mv positivo en 1/kPa, requerido.
    pub mv_per_kpa: Option<Parameter>,
    /// Conductividad hidráulica k positiva en m/s, requerida.
    pub k_m_s: Option<Parameter>,
    /// Número positivo de elementos uniformes de esta capa.
    pub elements: usize,
}

/// Caras permeables; las demás son impermeables, sin flujo prescrito.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drainage {
    /// Dren superior en z=0.
    Top,
    /// Dren inferior en z=H.
    Bottom,
    /// Drenes en z=0 y z=H.
    Double,
}

/// Punto de una historia de incremento de tensión total uniforme en profundidad.
#[derive(Clone, Copy, Debug)]
pub struct LoadPoint {
    /// Tiempo en s, estrictamente creciente; la tabla empieza exactamente en 0.
    pub time_s: Real,
    /// Δσ en kPa y su origen; requerido, puede ser cero o negativo.
    pub delta_stress_kpa: Option<Parameter>,
}

/// Controles numéricos explícitos, sin parámetros físicos supuestos.
#[derive(Clone, Copy, Debug)]
pub struct ConsolidationOptions {
    /// θ en `[0,1]`; 0 explícito, 0.5 Crank–Nicolson, 1 Euler implícito.
    pub theta: Real,
    /// Paso máximo positivo en s; se reduce para alcanzar quiebres y salidas.
    pub max_step_s: Real,
}
impl ConsolidationOptions {
    /// Crank–Nicolson; el paso máximo siempre lo aporta el llamador.
    pub const fn new(max_step_s: Real) -> Self {
        Self {
            theta: 0.5,
            max_step_s,
        }
    }
}

/// Campo en los nodos comunes a todas las salidas y magnitudes integradas.
#[derive(Clone, Debug)]
pub struct ConsolidationState {
    /// Tiempo en s.
    pub time_s: Real,
    /// Δσ interpolada, kPa.
    pub delta_stress_kpa: Real,
    /// u en kPa, alineada con ConsolidationResult::depths_m.
    pub excess_pore_pressure_kpa: Vec<Real>,
    /// Asiento positivo hacia abajo, m; integral exacta del campo FE.
    pub settlement_m: Real,
    /// U=s/(Δσ∫mv dz); indefinido cuando Δσ=0, sin recorte.
    pub mean_degree: Option<Real>,
}

/// Salidas y procedencia completa del cálculo incremental.
#[derive(Clone, Debug)]
pub struct ConsolidationResult {
    /// Profundidades nodales en m, incluidas todas las interfaces.
    pub depths_m: Vec<Real>,
    /// Estados, exactamente en el orden de tiempos solicitado.
    pub states: Vec<ConsolidationState>,
    /// Rutas de todos los parámetros declarados Assumed, sin duplicados.
    pub assumed_parameters: Vec<String>,
}
impl ConsolidationResult {
    /// Interpola u(z,t) linealmente a una profundidad dentro del perfil.
    /// El tiempo se selecciona por índice de salida; no extrapola espacio/tiempo.
    pub fn pressure_at(&self, state_index: usize, depth_m: Real) -> GeoResult<Real> {
        finite(depth_m, "depth_m")?;
        let state = self
            .states
            .get(state_index)
            .ok_or_else(|| invalid("Índice de salida fuera de rango."))?;
        // También valida los campos públicos si el llamador modifica el resultado.
        if self.depths_m.len() < 2 || state.excess_pore_pressure_kpa.len() != self.depths_m.len() {
            return Err(invalid("Dimensiones del resultado modificadas."));
        }
        for (i, pair) in self.depths_m.windows(2).enumerate() {
            if depth_m >= pair[0] && depth_m <= pair[1] && pair[1] > pair[0] {
                let r = (depth_m - pair[0]) / (pair[1] - pair[0]);
                return finite(
                    (1.0 - r) * state.excess_pore_pressure_kpa[i]
                        + r * state.excess_pore_pressure_kpa[i + 1],
                    "u(z,t)",
                );
            }
        }
        Err(invalid("Profundidad fuera del perfil."))
    }
}

#[derive(Clone, Debug)]
struct Element {
    h: Real,
    mv: Real,
    conductivity: Real, // k/γw
}

/// Perfil de consolidación validado e inmutable, sin parámetros por defecto.
#[derive(Clone, Debug)]
pub struct ConsolidationProfile {
    depths: Vec<Real>,
    elements: Vec<Element>,
    drainage: Drainage,
    assumed: Vec<String>,
}
fn diagnostic(code: &'static str, hint: impl Into<String>) -> Diagnostic {
    let mut d = match Diagnostic::from_code(code) {
        Some(d) => d,
        None => Diagnostic::new(
            "ENG-E-0012",
            Severity::Error,
            "Código de diagnóstico no registrado.",
        ),
    };
    d.hint = Some(hint.into());
    d
}
fn algebra(e: EngineError) -> Diagnostic {
    if e.code == "API-I-CANCELLED" {
        return Diagnostic::new(e.code, Severity::Info, e.message);
    }
    let mut d = diagnostic(e.code, e.message);
    d.entities = e.entities;
    d
}
fn invalid(hint: &str) -> Diagnostic {
    diagnostic("ENG-E-0010", hint)
}
fn finite(v: Real, field: &str) -> GeoResult<Real> {
    if v.is_finite() {
        Ok(v)
    } else {
        Err(diagnostic(
            "ENG-E-0008",
            format!("{field}: dato o resultado no finito."),
        ))
    }
}
fn positive(v: Real, field: &str) -> GeoResult<Real> {
    finite(v, field)?;
    if v > 0.0 {
        Ok(v)
    } else {
        Err(invalid(&format!(
            "{field}: debe ser positivo y representable."
        )))
    }
}
fn parameter(p: Option<Parameter>, field: &str, assumed: &mut Vec<String>) -> GeoResult<Real> {
    let p = p.ok_or_else(|| {
        diagnostic(
            "ENG-E-0003",
            format!("Falta {field}; aporte valor y Provenance, sin sustitución implícita."),
        )
    })?;
    finite(p.value, field)?;
    if p.provenance == Provenance::Assumed {
        assumed.push(field.into());
    }
    Ok(p.value)
}
impl ConsolidationProfile {
    /// Valida todas las capas, k/mv/cv/γw, procedencia, malla y aritmética.
    /// No exige datos del perfil de tensiones geostáticas que aquí no intervienen.
    pub fn new(
        layers: Vec<ConsolidationLayer>,
        water_unit_weight_kn_m3: Option<Parameter>,
        drainage: Drainage,
    ) -> GeoResult<Self> {
        let mut assumed = Vec::new();
        let water = positive(
            parameter(
                water_unit_weight_kn_m3,
                "water_unit_weight_kn_m3",
                &mut assumed,
            )?,
            "γw",
        )?;
        let count = layers
            .iter()
            .try_fold(0usize, |n, l| n.checked_add(l.elements))
            .ok_or_else(|| invalid("Número de elementos desbordado."))?;
        if layers.is_empty() || count > 2048 || count == 0 {
            return Err(invalid(
                "Se requieren capas y entre 1 y 2048 elementos en total.",
            ));
        }
        if count == 1 && drainage == Drainage::Double {
            return Err(invalid(
                "Doble drenaje requiere al menos dos elementos (un nodo libre).",
            ));
        }
        let mut depths = Vec::with_capacity(count + 1);
        let mut elements = Vec::with_capacity(count);
        depths.push(0.0);
        let mut top = 0.0;
        for (i, layer) in layers.into_iter().enumerate() {
            let prefix = format!("layers[{i}]");
            let thickness = positive(
                parameter(
                    Some(layer.thickness_m),
                    &format!("{prefix}.thickness_m"),
                    &mut assumed,
                )?,
                "espesor",
            )?;
            let cv = positive(
                parameter(layer.cv_m2_s, &format!("{prefix}.cv_m2_s"), &mut assumed)?,
                "cv",
            )?;
            let mv = positive(
                parameter(
                    layer.mv_per_kpa,
                    &format!("{prefix}.mv_per_kpa"),
                    &mut assumed,
                )?,
                "mv",
            )?;
            let k = positive(
                parameter(layer.k_m_s, &format!("{prefix}.k_m_s"), &mut assumed)?,
                "k",
            )?;
            if layer.elements == 0 {
                return Err(invalid("Cada capa requiere al menos un elemento."));
            }
            let conductivity = positive(k / water, "k/γw")?;
            let derived_cv = positive(conductivity / mv, "k/(γw mv)")?;
            if (finite(cv / derived_cv, "cv / cv derivado")? - 1.0).abs() > 1e-10 {
                return Err(invalid(&format!(
                    "{prefix}: cv incompatible con k/(γw mv); discrepancia relativa máxima 1e-10."
                )));
            }
            let bottom = finite(top + thickness, "profundidad acumulada")?;
            for e in 1..=layer.elements {
                let z = if e == layer.elements {
                    bottom
                } else {
                    finite(
                        top + thickness * (e as Real / layer.elements as Real),
                        "z nodal",
                    )?
                };
                let previous = depths[depths.len() - 1];
                let h = positive(z - previous, "longitud elemental")?;
                positive(mv * h / 6.0, "masa elemental")?;
                positive(conductivity / h, "difusión elemental")?;
                elements.push(Element {
                    h,
                    mv,
                    conductivity,
                });
                depths.push(z);
            }
            top = bottom;
        }
        Ok(Self {
            depths,
            elements,
            drainage,
            assumed,
        })
    }

    fn free_range(&self) -> (usize, usize) {
        let first = usize::from(self.drainage != Drainage::Bottom);
        let end = self.depths.len() - usize::from(self.drainage != Drainage::Top);
        (first, end)
    }

    fn matrices(
        &self,
        cancelled: &mut dyn FnMut() -> bool,
    ) -> GeoResult<(SparseMatrix, SparseMatrix, Vec<Real>)> {
        let (first, end) = self.free_range();
        let n = end - first;
        let mut mass = Vec::with_capacity(4 * self.elements.len());
        let mut diffusion = Vec::with_capacity(4 * self.elements.len());
        let mut source = vec![0.0; n];
        for (e, el) in self.elements.iter().enumerate() {
            check_cancelled(cancelled).map_err(algebra)?;
            let c = el.mv * el.h / 6.0;
            let d = el.conductivity / el.h;
            for a in 0..2 {
                let row = e + a;
                if row < first || row >= end {
                    continue;
                }
                source[row - first] = finite(
                    source[row - first] + el.mv * el.h / 2.0,
                    "carga elemental acumulada",
                )?;
                for b in 0..2 {
                    let col = e + b;
                    if col < first || col >= end {
                        continue;
                    }
                    mass.push(Triplet {
                        row: row - first,
                        col: col - first,
                        value: c * if a == b { 2.0 } else { 1.0 },
                    });
                    diffusion.push(Triplet {
                        row: row - first,
                        col: col - first,
                        value: d * if a == b { 1.0 } else { -1.0 },
                    });
                }
            }
        }
        Ok((
            SparseMatrix::from_triplets(n, n, &mass).map_err(algebra)?,
            SparseMatrix::from_triplets(n, n, &diffusion).map_err(algebra)?,
            source,
        ))
    }

    /// Integra con el backend de referencia de engine-linalg.
    pub fn solve(
        &self,
        history: &[LoadPoint],
        output_times_s: &[Real],
        options: ConsolidationOptions,
    ) -> GeoResult<ConsolidationResult> {
        self.solve_with_cancel(history, output_times_s, options, &mut || false)
    }

    /// Integra consultando el cierre antes de cada intervalo, factorización,
    /// bloque de hasta 16 columnas y subpaso, sin cambiar la
    /// partición temporal ni la aritmética de `solve`.
    /// Si devuelve true, termina con API-I-CANCELLED y sin resultados parciales.
    pub fn solve_with_cancel(
        &self,
        history: &[LoadPoint],
        output_times_s: &[Real],
        options: ConsolidationOptions,
        cancelled: &mut dyn FnMut() -> bool,
    ) -> GeoResult<ConsolidationResult> {
        self.solve_with_backend_and_cancel::<_, true>(
            &mut ReferenceBackend,
            history,
            output_times_s,
            options,
            cancelled,
        )
    }

    /// Integra con un backend existente; no implementa eliminación ni solver propio.
    /// Un error no modifica el perfil, historia ni salidas previas del llamador.
    pub fn solve_with_backend<B: LinAlgBackend>(
        &self,
        backend: &mut B,
        history: &[LoadPoint],
        output_times_s: &[Real],
        options: ConsolidationOptions,
    ) -> GeoResult<ConsolidationResult> {
        self.solve_with_backend_and_cancel::<_, false>(
            backend,
            history,
            output_times_s,
            options,
            &mut || false,
        )
    }

    fn solve_with_backend_and_cancel<B: LinAlgBackend, const REUSE_FACTOR: bool>(
        &self,
        backend: &mut B,
        history: &[LoadPoint],
        output_times_s: &[Real],
        options: ConsolidationOptions,
        cancelled: &mut dyn FnMut() -> bool,
    ) -> GeoResult<ConsolidationResult> {
        finite(options.theta, "theta")?;
        positive(options.max_step_s, "max_step_s")?;
        if !(0.0..=1.0).contains(&options.theta) {
            return Err(invalid("theta debe pertenecer a [0,1]."));
        }
        if history.len() < 2
            || history.len() > 100000
            || output_times_s.is_empty()
            || output_times_s.len() > 100000
        {
            return Err(invalid(
                "Se requieren 2..100000 puntos de carga y 1..100000 tiempos de salida.",
            ));
        }
        if output_times_s
            .len()
            .checked_mul(self.depths.len())
            .is_none_or(|n| n > 8000000)
        {
            return Err(invalid(
                "La salida supera el límite de 8000000 valores nodales.",
            ));
        }
        let mut assumed = self.assumed.clone();
        let mut loads = Vec::with_capacity(history.len());
        for (i, point) in history.iter().enumerate() {
            check_cancelled(cancelled).map_err(algebra)?;
            finite(point.time_s, "tiempo de carga")?;
            if (i == 0 && point.time_s != 0.0) || (i > 0 && point.time_s <= history[i - 1].time_s) {
                return Err(invalid(
                    "La historia debe empezar en 0 y tener tiempos estrictamente crecientes.",
                ));
            }
            loads.push(parameter(
                point.delta_stress_kpa,
                &format!("history[{i}].delta_stress_kpa"),
                &mut assumed,
            )?);
        }
        for (i, &t) in output_times_s.iter().enumerate() {
            check_cancelled(cancelled).map_err(algebra)?;
            finite(t, "tiempo de salida")?;
            if t < 0.0
                || t > history[history.len() - 1].time_s
                || (i > 0 && t <= output_times_s[i - 1])
            {
                return Err(invalid(
                    "Salidas estrictamente crecientes dentro de la historia, sin extrapolar.",
                ));
            }
        }
        let (m, k, source) = self.matrices(cancelled)?;
        let (first, end) = self.free_range();
        let n = end - first;
        let mut u = vec![loads[0]; n];
        let mut states = Vec::with_capacity(output_times_s.len());
        let mut time = 0.0;
        let mut stress = loads[0];
        let mut segment = 0;
        let mut cached_factor: Option<(u64, Box<dyn SymFactorization>)> = None;
        for &target in output_times_s {
            check_cancelled(cancelled).map_err(algebra)?;
            while time < target {
                while segment + 1 < history.len() - 1 && time >= history[segment + 1].time_s {
                    segment += 1;
                }
                let stop = target.min(history[segment + 1].time_s);
                let span = positive(stop - time, "intervalo temporal")?;
                let steps_real =
                    finite((span / options.max_step_s).ceil(), "número de subpasos")?.max(1.0);
                if steps_real > 1000000.0 {
                    return Err(invalid("Más de 1000000 subpasos en un intervalo."));
                }
                let steps = steps_real as usize;
                let dt = positive(span / steps as Real, "dt")?;
                check_cancelled(cancelled).map_err(algebra)?;
                check_stability(&m, &k, options.theta, dt)?;
                // Solo ReferenceBackend: M, K y theta son inmutables; el mismo
                // dt por bits produce la misma matriz y LDLᵀ determinista.
                // Los backends del llamador mantienen todas sus llamadas.
                if !REUSE_FACTOR
                    || cached_factor
                        .as_ref()
                        .is_none_or(|(bits, _)| *bits != dt.to_bits())
                {
                    let a = combine(&m, &k, options.theta * dt)?;
                    let factor = backend
                        .factorize_with_check(&a, &mut || check_cancelled(cancelled))
                        .map_err(algebra)?;
                    cached_factor = Some((dt.to_bits(), factor));
                }
                let factor = &cached_factor
                    .as_ref()
                    .ok_or_else(|| invalid("Falta la factorización temporal."))?
                    .1;
                let load_span = positive(
                    history[segment + 1].time_s - history[segment].time_s,
                    "intervalo de carga",
                )?;
                let mut previous_time = time;
                for step in 1..=steps {
                    if cancelled() {
                        return Err(Diagnostic::new(
                            "API-I-CANCELLED",
                            Severity::Info,
                            "Consolidación cancelada en un subpaso.",
                        ));
                    }
                    let next_time = if step == steps {
                        stop
                    } else {
                        time + dt * step as Real
                    };
                    if next_time <= previous_time {
                        return Err(invalid(
                            "El paso temporal se pierde en f64; revise las escalas.",
                        ));
                    }
                    let r = (next_time - history[segment].time_s) / load_span;
                    let next_stress = finite(
                        (1.0 - r) * loads[segment] + r * loads[segment + 1],
                        "Δσ interpolada",
                    )?;
                    let increment = finite(next_stress - stress, "incremento Δσ")?;
                    let mut rhs = vec![0.0; n];
                    for row in 0..n {
                        let mut value = source[row] * increment;
                        for p in m.row_offsets()[row]..m.row_offsets()[row + 1] {
                            value += (m.values()[p] - (1.0 - options.theta) * dt * k.values()[p])
                                * u[m.column_indices()[p]];
                        }
                        rhs[row] = finite(value, "RHS temporal")?;
                    }
                    let solution = factor
                        .solve_with_check(&DMat::new(n, 1, rhs).map_err(algebra)?, &mut || {
                            check_cancelled(cancelled)
                        })
                        .map_err(algebra)?;
                    u.copy_from_slice(solution.values());
                    stress = next_stress;
                    previous_time = next_time;
                }
                time = stop;
            }
            let mut full = vec![0.0; self.depths.len()];
            full[first..end].copy_from_slice(&u);
            let mut settlement = 0.0;
            let mut compliance = 0.0;
            for (e, el) in self.elements.iter().enumerate() {
                check_cancelled(cancelled).map_err(algebra)?;
                let c = finite(el.mv * el.h, "mv h")?;
                compliance = finite(compliance + c, "∫mv dz")?;
                settlement = finite(
                    settlement + c * (stress - 0.5 * full[e] - 0.5 * full[e + 1]),
                    "s(t)",
                )?;
            }
            let mean_degree = if stress == 0.0 {
                None
            } else {
                let ultimate = finite(stress * compliance, "s∞")?;
                if ultimate == 0.0 {
                    return Err(invalid("s∞ subdesbordado; revise las escalas."));
                }
                Some(finite(settlement / ultimate, "U(t)")?)
            };
            states.push(ConsolidationState {
                time_s: target,
                delta_stress_kpa: stress,
                excess_pore_pressure_kpa: full,
                settlement_m: settlement,
                mean_degree,
            });
        }
        Ok(ConsolidationResult {
            depths_m: self.depths.clone(),
            states,
            assumed_parameters: assumed,
        })
    }
}

fn check_cancelled(cancelled: &mut dyn FnMut() -> bool) -> Result<(), EngineError> {
    if cancelled() {
        Err(EngineError::new(
            "API-I-CANCELLED",
            "Consolidación cancelada antes o durante la factorización.",
        ))
    } else {
        Ok(())
    }
}

fn combine(m: &SparseMatrix, k: &SparseMatrix, scale: Real) -> GeoResult<SymMatrix> {
    let entries: Vec<_> = (0..m.rows())
        .flat_map(|row| {
            (m.row_offsets()[row]..m.row_offsets()[row + 1]).map(move |p| Triplet {
                row,
                col: m.column_indices()[p],
                value: m.values()[p] + scale * k.values()[p],
            })
        })
        .collect();
    SymMatrix::new(
        SparseMatrix::from_triplets(m.rows(), m.cols(), &entries)
            .map_err(algebra)?
            .to_dense()
            .map_err(algebra)?,
    )
    .map_err(algebra)
}

// λmax(M⁻¹K) ≤ max row absolute sum(K) / min row(diag(M)−sum offdiag(M)).
// M is symmetric strictly diagonally dominant for these linear elements.
fn check_stability(m: &SparseMatrix, k: &SparseMatrix, theta: Real, dt: Real) -> GeoResult<()> {
    if theta >= 0.5 {
        return Ok(());
    }
    let mut lower = Real::INFINITY;
    let mut upper: Real = 0.0;
    for row in 0..m.rows() {
        let mut margin = 0.0;
        let mut ksum = 0.0;
        for p in m.row_offsets()[row]..m.row_offsets()[row + 1] {
            margin += if m.column_indices()[p] == row {
                m.values()[p]
            } else {
                -m.values()[p].abs()
            };
            ksum += k.values()[p].abs();
        }
        lower = lower.min(margin);
        upper = upper.max(ksum);
    }
    positive(lower, "cota inferior de M")?;
    positive(upper, "cota superior de K")?;
    let limit = positive(
        (2.0 / (1.0 - 2.0 * theta)) * (lower / upper),
        "límite dt estable",
    )?;
    if dt > limit {
        return Err(invalid(&format!("θ={theta}: dt={dt} s supera cota suficiente de estabilidad {limit} s; reduzca max_step_s.")));
    }
    Ok(())
}
