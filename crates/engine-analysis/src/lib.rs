#![forbid(unsafe_code)]
//! Análisis `static.linear`, [`newton`] (`static.newton`), [`pdelta`], [`modal`] y [`spectrum`], en unidades SI.
//!
//! `static.linear` resuelve casos independientes con una única factorización
//! de K_red = T_fᵀ K T_f sobre los independientes libres. Recupera dependientes
//! y prescritos con u = T_f q_f + u_0. Incluye asentamientos sin escalarlos por el factor de carga; no modifica el estado
//! aceptado o de prueba. El módulo `combination` superpone resultados usando
//! combinaciones declarativas del modelo; `StaticLinear` resuelve solo casos.
//! Con prescritos nulos, u(A+B)=u(A)+u(B); con asentamientos, se resta una vez
//! la solución sin carga u(0). No incorpora Kg, iteraciones ni regularización.
//!
//! Ejes globales diestros; D2 usa XY y Rz, D3 XYZ y Rx/Ry/Rz. Coordenadas
//! originales en m (D2 requiere Z=0), traslaciones en m, giros en rad, fuerzas
//! en N y momentos en N·m. Se usan posiciones de referencia sin deformar.
//! El equilibrio suma TODAS las cargas físicas globales, incluidas las cargas
//! equivalentes elementales y las aplicadas sobre apoyos, más las reacciones
//! de engine-assembly. El momento añade `(posición-punto) × fuerza`, con signo
//! de mano derecha, y los pares nodales. No usa el RHS reducido para esta suma.
#![deny(missing_docs)]

pub mod combination;
pub mod modal;
pub mod pdelta;
pub mod results;
pub mod spectrum;

use engine_assembly::{LinearSystem, Reaction};
use engine_contracts::CaseRef;
use engine_core::{DofIndex, EResult, EngineError, Real};
use engine_linalg::LinAlgBackend;

pub mod newton;

/// Método de análisis estático lineal sin estado propio.
pub struct StaticLinear;

/// Resultados en el mismo orden que la lista solicitada.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticResults {
    /// Un resultado independiente por caso, sin aceptación automática.
    pub cases: Vec<StaticCaseResult>,
}

/// Solución física y verificaciones de un caso escalado.
#[derive(Clone, Debug, PartialEq)]
pub struct StaticCaseResult {
    /// Caso declarado del sistema.
    pub case: CaseRef,
    /// Factor aplicado solo a las cargas.
    pub factor: Real,
    /// Desplazamientos independientes libres en orden reducido, m/rad.
    pub free_displacements: Vec<Real>,
    /// Desplazamientos completos en orden global, incluidos dependientes y prescritos, m/rad.
    pub global_displacements: Vec<Real>,
    /// Reacciones en todos los GDL prescritos, N/N·m.
    pub reactions: Vec<Reaction>,
    /// Equilibrio de fuerzas y momentos globales.
    pub equilibrium: EquilibriumReport,
    /// `‖T_fᵀ(K u-F)‖∞` en componentes SI (N/N·m).
    pub free_residual_norm: Real,
    /// Norma anterior / max(1, ‖T_fᵀ K u‖∞, ‖T_fᵀ F‖∞).
    /// Indicador numérico de componentes SI; mezcla fuerzas y momentos,
    /// por lo que no es una norma física invariante al cambio de unidades.
    pub relative_free_residual: Real,
}

/// Equilibrio global con sumas por componente y momentos con brazo.
#[derive(Clone, Debug, PartialEq)]
pub struct EquilibriumReport {
    /// Suma física por GDL `[Fx,Fy,Fz,Mx,My,Mz]`; pares SIN brazo de fuerzas.
    pub applied_force_sum: [Real; 6],
    /// Suma de reacciones por GDL, mismo orden y convención.
    pub reaction_sum: [Real; 6],
    /// Suma de los dos vectores anteriores; los últimos tres valores son
    /// pares nodales residuales, no el momento total del equilibrio.
    pub residual_force: [Real; 6],
    /// Momento total residual `[Mx,My,Mz]`, incluidos brazos de fuerzas, N·m.
    pub residual_moment: [Real; 3],
    /// Punto global de referencia de los momentos, m.
    pub moment_point: [Real; 3],
    /// Máximo de los seis cocientes: fuerzas residuales / max(1 N, suma de
    /// módulos de fuerzas de esa componente) y momentos totales residuales /
    /// max(1 N·m, suma de módulos de pares y términos del producto cruzado de
    /// esa componente). Cargas y reacciones aportan ambas a las escalas.
    /// Un sistema sin acciones y en equilibrio tiene residual relativo cero.
    pub relative_residual: Real,
}

impl StaticLinear {
    /// Identificador público del método.
    pub const KIND: &'static str = "static.linear";

    /// Resuelve todos los casos con UNA factorización y momentos en el origen.
    /// ENG-E-0010 por lista vacía/duplicada; ENG-E-0003 por caso ausente;
    /// ENG-E-0008 por no finitos; ENG-E-0006 por mecanismo con nodo/GDL en
    /// mensaje y entidades; ENG-E-0012 por diagnóstico inválido del backend.
    pub fn run<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        cases: &[(CaseRef, Real)],
    ) -> EResult<StaticResults> {
        Self::run_about(system, cases, [0.0; 3])
    }

    /// Como [`Self::run`], con momentos respecto a un punto global finito en m.
    /// No altera ni acepta el estado del sistema, incluso ante errores.
    pub fn run_about<B: LinAlgBackend>(
        system: &mut LinearSystem<B>,
        cases: &[(CaseRef, Real)],
        point: [Real; 3],
    ) -> EResult<StaticResults> {
        finite(&point)?;
        let solutions = system.solve_cases(cases)?;
        let mut results = Vec::new();
        for ((case, factor), free) in cases.iter().zip(solutions) {
            let global = system.constraints().expand(&free)?;
            let reactions = system.reactions(&free, case, *factor)?;
            let force = system.global_force(case, *factor)?;
            let (free_residual_norm, relative_free_residual) =
                free_residual(system, &global, &force)?;
            let equilibrium = equilibrium(system, &force, &reactions, point)?;
            results.push(StaticCaseResult {
                case: case.clone(),
                factor: *factor,
                free_displacements: free,
                global_displacements: global,
                reactions,
                equilibrium,
                free_residual_norm,
                relative_free_residual,
            });
        }
        Ok(StaticResults { cases: results })
    }
}

fn finite(values: &[Real]) -> EResult<()> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err(EngineError::new(
            "ENG-E-0008",
            "Análisis: entrada o resultado no finito.",
        ));
    }
    Ok(())
}

fn free_residual<B: LinAlgBackend>(
    system: &LinearSystem<B>,
    global: &[Real],
    force: &[Real],
) -> EResult<(Real, Real)> {
    let k = system.stiffness();
    let mut internal = vec![0.0; global.len()];
    for (row, value) in internal.iter_mut().enumerate() {
        for p in k.row_offsets()[row]..k.row_offsets()[row + 1] {
            let term = k.values()[p] * global[k.column_indices()[p]];
            finite(&[term])?;
            *value += term;
            finite(&[*value])?;
        }
    }
    let residual: Vec<_> = internal.iter().zip(force).map(|(k, f)| k - f).collect();
    let residual = system.constraints().project_force(&residual)?;
    let internal = system.constraints().project_force(&internal)?;
    let external = system.constraints().project_force(force)?;
    let mut norm: Real = 0.0;
    let mut scale: Real = 1.0;
    for ((residual, internal), external) in residual.into_iter().zip(internal).zip(external) {
        finite(&[residual])?;
        norm = norm.max(residual.abs());
        scale = scale.max(internal.abs()).max(external.abs());
    }
    Ok((norm, norm / scale))
}

#[derive(Default)]
struct Sum {
    components: [Real; 6],
    moments: [Real; 3],
    scales: [Real; 6],
}

impl Sum {
    fn add(&mut self, axis: usize, value: Real, arm: [Real; 3]) -> EResult<()> {
        self.components[axis] += value;
        if axis < 3 {
            self.scales[axis] += value.abs();
            // r × F para una fuerza que solo tiene componente axis.
            let mut f = [0.0; 3];
            f[axis] = value;
            let moment = [
                arm[1] * f[2] - arm[2] * f[1],
                arm[2] * f[0] - arm[0] * f[2],
                arm[0] * f[1] - arm[1] * f[0],
            ];
            for (i, m) in moment.iter().enumerate() {
                self.moments[i] += m;
                self.scales[i + 3] += m.abs();
            }
        } else {
            self.moments[axis - 3] += value;
            self.scales[axis] += value.abs();
        }
        finite(&self.components)?;
        finite(&self.moments)?;
        finite(&self.scales)
    }
}

fn equilibrium<B: LinAlgBackend>(
    system: &LinearSystem<B>,
    force: &[Real],
    reactions: &[Reaction],
    point: [Real; 3],
) -> EResult<EquilibriumReport> {
    let map = system.dof_map();
    let mut applied = Sum::default();
    let mut reaction = Sum::default();
    for (i, &value) in force.iter().enumerate() {
        let (node, dof) = map.label(DofIndex(i as u32))?;
        let coordinates = map.coordinates(node)?;
        let arm = std::array::from_fn(|i| coordinates[i] - point[i]);
        finite(&arm)?;
        applied.add(dof.index(), value, arm)?;
    }
    for r in reactions {
        let coordinates = map.coordinates(r.node)?;
        let arm = std::array::from_fn(|i| coordinates[i] - point[i]);
        finite(&arm)?;
        reaction.add(r.dof.index(), r.value, arm)?;
    }
    let residual_force = std::array::from_fn(|i| applied.components[i] + reaction.components[i]);
    let residual_moment = std::array::from_fn(|i| applied.moments[i] + reaction.moments[i]);
    finite(&residual_force)?;
    finite(&residual_moment)?;
    let mut relative_residual: Real = 0.0;
    for i in 0..6 {
        let scale = applied.scales[i] + reaction.scales[i];
        finite(&[scale])?;
        let residual = if i < 3 {
            residual_force[i]
        } else {
            residual_moment[i - 3]
        };
        relative_residual = relative_residual.max(residual.abs() / scale.max(1.0));
    }
    Ok(EquilibriumReport {
        applied_force_sum: applied.components,
        reaction_sum: reaction.components,
        residual_force,
        residual_moment,
        moment_point: point,
        relative_residual,
    })
}
