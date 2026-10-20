//! Curvas laterales estáticas propias: `py.matlock_soft_clay`, `py.reese_sand`,
//! `py.user`. SI sin prefijos: y, x, d en m; p en N/m; c, σ'v en Pa;
//! γ' y k en N/m³. p tiene el signo de y; la reacción del suelo es −p.
//!
//! Fuentes: Matlock (1970), OTC 1204, Ec. 1–5 y Fig. 5a; Reese, Cox y Koop
//! (1974), OTC 2080, Ec. 4–9 y 11–16, pasos 2 y 4–8. Ficha de J2 y
//! derivaciones propias: `provenance/ENG-GEO-003.md`. No se incluyen ciclos.
//!
//! El perfil solo aporta σ'v mediante [`effective_stress_pa`]; no almacena c,
//! εc, φ, k, A ni B. Estos datos y la geometría son explícitos y obligatorios.
//! Reese supone γ' y φ constantes para determinar Xt; no se sustituye γ' por
//! σ'v/x en un perfil variable. Matlock admite σ'v de un perfil variable.
//!
//! [`PyCurve::points`] contiene tuplas consumibles numéricamente por
//! `engine-laws::uniaxial::MultilinearElastic`. Su contrato físico es ε/σ:
//! para conservar unidades, el adaptador del futuro pilote debe normalizar
//! ε=y/L y σ=p·L/área, con L y área explícitos, y recuperar p=σ·área/L.
//! Pasar (y,p) directamente solo reutiliza la interpolación, no sus unidades.
//! La ley prolonga los segmentos extremos: las curvas generadas terminan en
//! mesetas; `py.user` conserva sus extremos y [`PyCurve::at`] rechaza consultas
//! fuera de la tabla. El consumidor debe decidir explícitamente su extrapolación.
//!
//! ```
//! use engine_geo::py::{matlock_soft_clay, MatlockParams, Sampling};
//! let soil = MatlockParams {
//!     depth_m: Some(2.0), diameter_m: Some(1.0),
//!     undrained_strength_pa: Some(20_000.0),
//!     effective_vertical_stress_pa: Some(20_000.0),
//!     j: Some(0.5), epsilon_50: Some(0.01),
//! };
//! let sampling = Sampling {
//!     positive_y_m: vec![0.003125, 0.025], plateau_y_m: Some(0.4),
//! };
//! let result = matlock_soft_clay(&soil, &sampling)?;
//! assert_eq!(result.curve.at(0.025)?, 50_000.0);
//! # Ok::<(), engine_core::EngineError>(())
//! ```

use crate::StressProfile;
use engine_core::math;
use engine_core::{EResult, EngineError, Real};

/// Identidad propia del generador estático de arcilla blanda.
pub const MATLOCK_SOFT_CLAY: &str = "py.matlock_soft_clay";
/// Identidad propia del generador estático de arena.
pub const REESE_SAND: &str = "py.reese_sand";
/// Identidad propia de la tabla suministrada por el usuario.
pub const USER: &str = "py.user";

fn invalid(field: &str) -> EngineError {
    EngineError::new(
        "ENG-E-0010",
        format!("p-y: {field}: fuera de dominio o geometría numérica no representable."),
    )
}
fn finite(value: Real, field: &str) -> EResult<Real> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(EngineError::new(
            "ENG-E-0008",
            format!("p-y: {field}: valor no finito o desbordamiento."),
        ))
    }
}
fn positive(value: Real, field: &str) -> EResult<Real> {
    finite(value, field)?;
    if value > 0.0 {
        Ok(value)
    } else {
        Err(invalid(field))
    }
}
fn nonnegative(value: Real, field: &str) -> EResult<Real> {
    finite(value, field)?;
    if value >= 0.0 {
        Ok(value)
    } else {
        Err(invalid(field))
    }
}
fn required(value: Option<Real>, field: &str) -> EResult<Real> {
    value.ok_or_else(|| {
        EngineError::new(
            "ENG-E-0003",
            format!("p-y: falta {field}; aporte el parámetro explícitamente."),
        )
    })
}
fn pos(value: Option<Real>, field: &str) -> EResult<Real> {
    positive(required(value, field)?, field)
}

/// Convierte σ'v de `stress-profile/1` (kPa) a Pa, sin recortarla ni extrapolar.
/// Propaga código, entidades y contexto del diagnóstico del perfil. El llamador
/// conserva su reporte de procedencia mediante `StressProfile::stresses_at`.
/// Rechaza tensiones negativas, no finitas y desbordamiento de conversión.
pub fn effective_stress_pa(profile: &dyn StressProfile, depth_m: Real) -> EResult<Real> {
    nonnegative(depth_m, "x")?;
    let stress = profile.stresses_at(depth_m).map_err(|d| EngineError {
        code: d.code,
        message: format!("{}; {:?}", d.message, d.hint),
        entities: d.entities,
    })?;
    nonnegative(stress.effective_vertical_kpa * 1000.0, "σ'v (Pa)")
}

/// Tabla inmutable, bilateral e impar. Incluye (0,0), con pendientes no negativas.
#[derive(Clone, Debug)]
pub struct PyCurve {
    kind: &'static str,
    points: Vec<(Real, Real)>,
}
impl PyCurve {
    /// Identidad del generador.
    pub fn kind(&self) -> &'static str {
        self.kind
    }
    /// Puntos ordenados (y (m), p (N/m)); incluye ambas ramas y el origen.
    pub fn points(&self) -> &[(Real, Real)] {
        &self.points
    }
    /// Transfiere la tabla para adaptadores de leyes, sin cambiar unidades.
    pub fn into_points(self) -> Vec<(Real, Real)> {
        self.points
    }
    /// Interpolación secante acotada al intervalo tabulado; nunca extrapola.
    /// ENG-E-0010 fuera de tabla; ENG-E-0008 por consulta/resultado no finito.
    pub fn at(&self, y_m: Real) -> EResult<Real> {
        finite(y_m, "y")?;
        let first = self.points.first().ok_or_else(|| invalid("tabla vacía"))?;
        let last = self.points.last().ok_or_else(|| invalid("tabla vacía"))?;
        if y_m < first.0 || y_m > last.0 {
            return Err(invalid("y fuera de tabla"));
        }
        let i = self.points.partition_point(|&(y, _)| y < y_m);
        let &(y1, p1) = self
            .points
            .get(i)
            .ok_or_else(|| invalid("y fuera de tabla"))?;
        if y1 == y_m {
            return Ok(p1);
        }
        let &(y0, p0) = self
            .points
            .get(i.saturating_sub(1))
            .ok_or_else(|| invalid("tramo"))?;
        finite(p0 + (p1 - p0) * ((y_m - y0) / (y1 - y0)), "p interpolada")
    }
}

/// Valida la rama positiva, luego refleja sin duplicar el origen.
fn curve(kind: &'static str, points: Vec<(Real, Real)>) -> EResult<PyCurve> {
    if points.len() < 2 || points.first() != Some(&(0.0, 0.0)) {
        return Err(invalid("se requieren origen y al menos un punto positivo"));
    }
    for &(y, p) in &points {
        nonnegative(y, "y tabulada")?;
        nonnegative(p, "p tabulada")?;
    }
    for pair in points.windows(2) {
        let dy = positive(pair[1].0 - pair[0].0, "Δy estrictamente positivo")?;
        let dp = nonnegative(pair[1].1 - pair[0].1, "Δp no decreciente")?;
        let slope = finite(dp / dy, "pendiente secante")?;
        if dp > 0.0 && slope == 0.0 {
            return Err(invalid("pendiente subdesbordada"));
        }
    }
    // También deben ser representables las diferencias de la tabla bilateral.
    if let Some(&(y, p)) = points.last() {
        finite(2.0 * y, "rango bilateral y")?;
        finite(2.0 * p, "rango bilateral p")?;
    }
    let mut both: Vec<_> = points
        .iter()
        .skip(1)
        .rev()
        .map(|&(y, p)| (-y, -p))
        .collect();
    both.extend(points);
    Ok(PyCurve { kind, points: both })
}

/// Construye `py.user` desde la rama y≥0. Exige origen, al menos dos puntos
/// finitos, y estrictamente creciente y p≥0 no decreciente. No ordena, añade
/// mesetas ni extrapola; refleja exactamente la tabla del usuario.
pub fn user(points: Vec<(Real, Real)>) -> EResult<PyCurve> {
    curve(USER, points)
}

/// Muestreo explícito: posiciones positivas adicionales y extremo de meseta.
/// Se añaden origen, vértices del método y sus reflejos. No se usa una malla
/// predeterminada. La interpolación entre muestras aproxima la curva analítica.
#[derive(Clone, Debug)]
pub struct Sampling {
    /// y>0 finitas y estrictamente crecientes; puede estar vacía. No exceden la meseta.
    pub positive_y_m: Vec<Real>,
    /// Obligatoria, estrictamente mayor que 8yc (Matlock) o yu (Reese).
    pub plateau_y_m: Option<Real>,
}
fn sample(
    sampling: &Sampling,
    end: Real,
    landmarks: &[Real],
    kind: &'static str,
    evaluate: impl Fn(Real) -> EResult<Real>,
) -> EResult<PyCurve> {
    let plateau = pos(sampling.plateau_y_m, "y de meseta")?;
    if plateau <= end {
        return Err(invalid("meseta debe superar el inicio de fluencia"));
    }
    let mut prev = 0.0;
    for &y in &sampling.positive_y_m {
        positive(y, "y de muestreo")?;
        if y <= prev || y > plateau {
            return Err(invalid("orden/rango del muestreo"));
        }
        prev = y;
    }
    let mut ys = vec![0.0, plateau];
    ys.extend_from_slice(&sampling.positive_y_m);
    for &y in landmarks {
        positive(y, "vértice de muestreo")?;
        if y > plateau {
            return Err(invalid("vértice fuera de muestreo"));
        }
        ys.push(y);
    }
    ys.sort_by(Real::total_cmp);
    ys.dedup();
    let points = ys
        .into_iter()
        .map(|y| evaluate(y).map(|p| (y, p)))
        .collect::<EResult<Vec<_>>>()?;
    curve(kind, points)
}

/// Parámetros obligatorios de Matlock estático, sin valores asumidos.
#[derive(Clone, Debug)]
pub struct MatlockParams {
    /// Profundidad x≥0 (m).
    pub depth_m: Option<Real>,
    /// Diámetro d>0 (m).
    pub diameter_m: Option<Real>,
    /// Resistencia no drenada c>0 (Pa).
    pub undrained_strength_pa: Option<Real>,
    /// Sobrecarga efectiva σ'v≥0 (Pa), explícita o de [`effective_stress_pa`].
    pub effective_vertical_stress_pa: Option<Real>,
    /// J>0 adimensional, obligatorio. OTC 1204 menciona 0.5 y 0.25 según sitio.
    pub j: Option<Real>,
    /// εc>0 adimensional a mitad de tensión máxima (ε50); sin imponer el
    /// intervalo orientativo 0.005–0.020 de OTC 1204 como límite universal.
    pub epsilon_50: Option<Real>,
}
/// Curva y magnitudes de control de Matlock (OTC 1204).
#[derive(Clone, Debug)]
pub struct MatlockCurve {
    /// Tabla bilateral en SI.
    pub curve: PyCurve,
    /// Np de Ec. 2/3, limitado a 9.
    pub np: Real,
    /// pu de Ec. 1 (N/m).
    pub ultimate_n_m: Real,
    /// yc de Ec. 5 (m).
    pub yc_m: Real,
}
/// Matlock (1970), OTC 1204, Ec. 1–3 y 5, curva estática Fig. 5a.
/// Se aplica min(Np,9) punto a punto, incluso con perfil variable. Ec. 4 es
/// informativa para suelo constante: xr=6d/(γ'd/c+J), no se necesita aquí.
/// Incluye siempre 8yc y una meseta posterior explícita. En el origen la
/// derivada analítica es infinita: el primer segmento es secante al primer y>0.
/// ENG-E-0003 por ausentes; ENG-E-0010 por dominio/muestreo/precisión inválidos;
/// ENG-E-0008 por entradas o resultados no finitos. No acepta carga cíclica.
pub fn matlock_soft_clay(params: &MatlockParams, sampling: &Sampling) -> EResult<MatlockCurve> {
    let x = nonnegative(required(params.depth_m, "x")?, "x")?;
    let d = pos(params.diameter_m, "d")?;
    let c = pos(params.undrained_strength_pa, "c")?;
    let stress = nonnegative(required(params.effective_vertical_stress_pa, "σ'v")?, "σ'v")?;
    let j = pos(params.j, "J")?;
    let epsilon = pos(params.epsilon_50, "ε50")?;
    let np = finite(3.0 + stress / c + j * (x / d), "Np sin limitar")?.min(9.0);
    let pu = positive(np * c * d, "pu")?;
    let yc = positive(2.5 * epsilon * d, "yc")?;
    let end = positive(8.0 * yc, "8yc")?;
    let curve = sample(sampling, end, &[end], MATLOCK_SOFT_CLAY, |y| {
        if y >= end {
            Ok(pu)
        } else {
            finite(pu * 0.5 * math::cbrt(y / yc), "p Matlock")
        }
    })?;
    Ok(MatlockCurve {
        curve,
        np,
        ultimate_n_m: pu,
        yc_m: yc,
    })
}

/// Parámetros obligatorios de Reese estático para arena homogénea, sin tablas.
#[derive(Clone, Debug)]
pub struct ReeseParams {
    /// Profundidad x≥0 (m); x=0 produce curva nula tras validar parámetros.
    pub depth_m: Option<Real>,
    /// Diámetro b>0 (m).
    pub diameter_m: Option<Real>,
    /// φ en radianes, 0<φ<π/2.
    pub friction_angle_rad: Option<Real>,
    /// γ'>0 constante (N/m³); no se obtiene implícitamente del perfil.
    pub effective_unit_weight_n_m3: Option<Real>,
    /// α en radianes, 0<α<π/2. Paso 2 propone α=φ/2; aporte explícito.
    pub alpha_rad: Option<Real>,
    /// K0>0. Paso 2 fija 0.4 como valor del método; aporte explícito.
    pub k0: Option<Real>,
    /// A>0, factor de pu (Ec. 9); Fig. 8, a cargo del usuario.
    pub a: Option<Real>,
    /// B>0, factor de pm (Ec. 11); Fig. 9, a cargo del usuario.
    pub b: Option<Real>,
    /// k>0 (N/m³) de Ec. 4; no se copia la tabla del paper.
    pub subgrade_modulus_n_m3: Option<Real>,
}
/// Curva y controles de Reese (OTC 2080), SI sin prefijos.
#[derive(Clone, Debug)]
pub struct ReeseCurve {
    /// Tabla bilateral con k, m, u y meseta (en superficie solo m, u y meseta).
    pub curve: PyCurve,
    /// pct cercano de Ec. 5 (N/m).
    pub pct_n_m: Real,
    /// pcd profundo de Ec. 8 (N/m).
    pub pcd_n_m: Real,
    /// Xt>0 (m), intersección no trivial de Ec. 5/8.
    pub transition_depth_m: Real,
    /// pu de Ec. 9 (N/m).
    pub ultimate_n_m: Real,
    /// pm de Ec. 11 (N/m).
    pub pm_n_m: Real,
    /// ym=b/60 (m), paso 7.
    pub ym_m: Real,
    /// yu=3b/80 (m), paso 6.
    pub yu_m: Real,
    /// yk de Ec. 16 (m); None solo para x=0 (curva nula).
    pub yk_m: Option<Real>,
    /// n>1 de Ec. 14, también validado en x=0 por su forma independiente de pc.
    pub n: Real,
}
/// Reese, Cox y Koop (1974), OTC 2080, pasos 2 y 4–8, Ec. 4–9 y 11–16.
/// β=π/4+φ/2 y Ka=tan²(π/4−φ/2) (Ec. 6/7). Xt se halla dividiendo
/// pct−pcd por γ'H: queda una función afín. Se exige pendiente positiva
/// y término independiente negativo;
/// así hay una raíz positiva única. Se acota desde b duplicando hasta 128
/// veces y se biseca 96 veces, con orden fijo. Se exige ancho final
/// ≤8·EPSILON·cota superior; una raíz no resuelta es error tipado.
/// Por encima de Xt se usa pct; por debajo pcd (paso 4).
///
/// Se rechazan A≤B, n≤1, yk≥ym y escalas no representables; no se inventa
/// una rama alternativa. Para x=0, Ec. 5/8 dan p=0 en toda la tabla; no se
/// calcula yk (división por cero). Todos los datos siguen siendo obligatorios.
/// La parábola se evalúa como pm·(y/ym)^(1/n), forma de Ec. 12/15 que evita
/// potencias dimensionales. yk=ym·[pm/(k·x·ym)]^(n/(n−1)) deriva de Ec. 16.
/// Errores: ENG-E-0003 ausentes, ENG-E-0010 dominio/degeneración/muestreo,
/// ENG-E-0008 no finitos/desbordamientos. Solo carga estática.
pub fn reese_sand(params: &ReeseParams, sampling: &Sampling) -> EResult<ReeseCurve> {
    let x = nonnegative(required(params.depth_m, "x")?, "x")?;
    let diameter = pos(params.diameter_m, "b diámetro")?;
    let phi = pos(params.friction_angle_rad, "φ")?;
    let alpha = pos(params.alpha_rad, "α")?;
    if phi >= std::f64::consts::FRAC_PI_2 || alpha >= std::f64::consts::FRAC_PI_2 {
        return Err(invalid("φ y α deben ser menores que π/2"));
    }
    let gamma = pos(params.effective_unit_weight_n_m3, "γ'")?;
    let k0 = pos(params.k0, "K0")?;
    let a = pos(params.a, "A")?;
    let b = pos(params.b, "B")?;
    let k = pos(params.subgrade_modulus_n_m3, "k")?;
    if a <= b {
        return Err(invalid("A debe superar B"));
    }
    let ym = positive(diameter / 60.0, "ym")?;
    let yu = positive(3.0 * diameter / 80.0, "yu")?;
    let n = positive((b / (a - b)) * ((yu - ym) / ym), "n")?;
    if n <= 1.0 {
        return Err(invalid("n debe superar 1"));
    }
    let beta = std::f64::consts::FRAC_PI_4 + phi / 2.0;
    let tb = math::tan(beta);
    let tf = math::tan(phi);
    let ta = math::tan(alpha);
    let divisor = math::tan(beta - phi);
    let ka = math::powi(math::tan(std::f64::consts::FRAC_PI_4 - phi / 2.0), 2);
    // Ec. 5 / (γ'H) = q1 H + q0; Ec. 8 / (γ'H) = deep.
    let q1 = positive(
        k0 * tf * math::sin(beta) / (divisor * math::cos(alpha))
            + tb * tb * ta / divisor
            + k0 * tb * (tf * math::sin(beta) - ta),
        "coeficiente lineal pct",
    )?;
    let q0 = positive(diameter * (tb / divisor - ka), "coeficiente constante pct")?;
    let deep = positive(
        ka * diameter * (math::powi(tb, 8) - 1.0) + k0 * diameter * tf * math::powi(tb, 4),
        "coeficiente pcd",
    )?;
    let offset = positive(deep - q0, "intersección positiva pct/pcd")?;
    let residual = |h: Real| finite(q1 * h - offset, "residuo Xt");
    let mut hi = diameter;
    let mut bracketed = false;
    for _ in 0..128 {
        if residual(hi)? >= 0.0 {
            bracketed = true;
            break;
        }
        hi = positive(hi * 2.0, "cota Xt")?;
    }
    if !bracketed {
        return Err(invalid("Xt sin cota en 128 duplicaciones"));
    }
    let mut lo = 0.0;
    for _ in 0..96 {
        let mid = lo + (hi - lo) * 0.5;
        if residual(mid)? >= 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    if hi - lo > 8.0 * Real::EPSILON * hi {
        return Err(invalid(
            "Xt no resuelta a precisión relativa binary64 en 96 bisecciones",
        ));
    }
    let xt = positive(lo + (hi - lo) * 0.5, "Xt")?;
    let pct = nonnegative(gamma * x * (q1 * x + q0), "pct")?;
    let pcd = nonnegative(gamma * x * deep, "pcd")?;
    let pc = if x <= xt { pct } else { pcd };
    let pu = nonnegative(a * pc, "pu")?;
    let pm = nonnegative(b * pc, "pm")?;
    let mut landmarks = vec![ym, yu];
    let (yk, initial) = if x == 0.0 {
        (None, 0.0)
    } else {
        positive(pc, "pc")?;
        positive(pu, "pu")?;
        positive(pm, "pm")?;
        let initial = positive(k * x, "k·x")?;
        let ratio = positive(pm / positive(initial * ym, "k·x·ym")?, "pm/(k·x·ym)")?;
        let yk = positive(ym * math::powf(ratio, n / (n - 1.0)), "yk")?;
        if yk >= ym {
            return Err(invalid("yk debe ser menor que ym; aumente k o revise A/B"));
        }
        landmarks.push(yk);
        (Some(yk), initial)
    };
    let curve = sample(sampling, yu, &landmarks, REESE_SAND, |y| {
        if x == 0.0 || y == 0.0 {
            return Ok(0.0);
        }
        if y >= yu {
            return Ok(pu);
        }
        if y >= ym {
            return finite(pm + (pu - pm) * ((y - ym) / (yu - ym)), "recta m-u");
        }
        if yk.is_some_and(|yk| y <= yk) {
            return finite(initial * y, "recta inicial");
        }
        finite(pm * math::powf(y / ym, 1.0 / n), "parábola Reese")
    })?;
    Ok(ReeseCurve {
        curve,
        pct_n_m: pct,
        pcd_n_m: pcd,
        transition_depth_m: xt,
        ultimate_n_m: pu,
        pm_n_m: pm,
        ym_m: ym,
        yu_m: yu,
        yk_m: yk,
        n,
    })
}
