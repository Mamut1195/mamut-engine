//! Curvas axiales locales t-z y q-z, en SI sin prefijos (m, Pa, N).
//!
//! `z > 0` significa que el pilote baja respecto al suelo (compresión).
//! Las ordenadas son resistencias movilizadas con ese signo; la acción del
//! suelo sobre el pilote tiene signo opuesto. No son tensiones geostáticas.
//!
//! # Fuente y alcance
//!
//! API RP 2GEO, *Geotechnical and Foundation Design Considerations*, primera
//! edición, abril de 2011, Addendum 1, octubre de 2014, reafirmada en 2021
//! (ISO 19901-4:2003 modificada): §8.4.2, figura 2 y tabla 2 para t-z;
//! §8.4.3, figura 3 para q-z. Se implementan solo los vértices numéricos
//! aportados en la ficha de J2; no se reproduce texto ni figuras.
//! La interpolación lineal entre vértices y la reflexión de forma en tracción
//! con escala independiente son derivaciones propias, no reglas atribuidas
//! a la fuente. Véase `provenance/ENG-GEO-004.md`.
//!
//! # Consumo y errores
//!
//! Las tablas son inmutables, incluyen (0, 0) y no tienen malla ni parámetros
//! por defecto. El evaluador prolonga los tramos extremos, como la ley
//! multilineal elástica de engine-laws (ENG-MAT-002); las API cierran ambos
//! extremos con mesetas. Las tablas de usuario controlan su extrapolación.
//! Los adaptadores convierten a (ε, σ), tracción positiva, para
//! `MultilinearElastic::new(points, MultilinearMode::Both)`, sin dependencia
//! de engine-laws. Esta ley rechaza pendientes negativas: el adaptador devuelve
//! `ENG-E-0010` ante ablandamiento, aunque el generador sí lo conserva.
//! No se implementan historia, barra axial, pilote ni equilibrio estructural.
//!
//! `ENG-E-0003`: parámetro requerido ausente; `ENG-E-0010`: dominio, orden,
//! origen, subdesbordamiento o incompatibilidad con la ley;
//! `ENG-E-0008`: entrada o cálculo no finito. Los mensajes identifican el dato.
//!
//! # Extensiones
//!
//! La identidad `tz.coyle_reese_clay` queda **no implementada — pendiente de
//! la fuente original**, Coyle & Reese (1966), ASCE JSMFD. En esta entrega solo
//! se pueden ingresar puntos propios mediante [`user`]; no hay función ni
//! constantes de ese método. No se usó Reese/Isenhower/Wang (2006).

use engine_core::{EResult, EngineError, Real};

/// Datos obligatorios de `tz.api_sand` y `tz.api_clay`, sin sustituciones.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ApiTzParams {
    /// Fricción unitaria máxima en compresión, Pa, estrictamente positiva.
    /// Debe calcularla el usuario según §8.1; ecuaciones 17–21 pendientes.
    pub tmax_compression_pa: Option<Real>,
    /// Fricción unitaria máxima en tracción ft(z), Pa, estrictamente positiva.
    /// Puede coincidir con la de compresión, solo por elección del usuario.
    pub tmax_tension_pa: Option<Real>,
    /// Desplazamiento de pico, m, estrictamente positivo, aportado por el usuario.
    /// §8.4.2 recomienda zpeak/D = 0.01 para diseño rutinario y señala
    /// 0.0025–0.02; aquí no se adopta ni impone ninguno de esos valores.
    pub zpeak_m: Option<Real>,
    /// Extremo positivo de la meseta, m; también cierra la tabla en −z.
    /// Debe superar zpeak para arena y 2·zpeak para arcilla.
    pub plateau_z_m: Option<Real>,
}

/// Elección obligatoria de la rama negativa en tablas de usuario.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UserTraction {
    /// Se aportan puntos negativos y positivos, además de (0, 0).
    /// Exige resistencia ≤ 0 si z < 0 y resistencia ≥ 0 si z > 0.
    /// Permite escalas independientes y ramas negativas de resistencia cero.
    Explicit,
    /// Se aportan solo (0, 0) y puntos positivos; se añaden (−z, −resistencia).
    Symmetric,
}

/// Tabla inmutable (z en m, t en Pa); admite ablandamiento post-pico.
#[derive(Clone, Debug, PartialEq)]
pub struct TzCurve(Table);

impl TzCurve {
    /// Vértices ordenados; solo lectura, incluidos origen y ramas declaradas.
    pub fn points(&self) -> &[(Real, Real)] {
        &self.0.points
    }

    /// Resistencia unitaria, Pa, interpolada; prolonga los tramos extremos.
    /// Errores numéricos según la documentación del módulo.
    pub fn at(&self, z_m: Real) -> EResult<Real> {
        self.0.at(z_m)
    }

    /// Adapta a la ley uniaxial: ε = −z/L, σ = −t (Pa); invierte el orden.
    /// L (m) es una longitud de referencia explícita, positiva y finita;
    /// no se usa D ni se asigna una longitud de elemento implícitamente.
    /// Es una representación de la interfaz unitaria: el futuro pilote debe
    /// integrar t sobre su superficie para obtener fuerza (N).
    /// Rechaza ablandamiento con `ENG-E-0010`, sin modificar el generador.
    pub fn multilinear_points(
        &self,
        reference_length_m: Option<Real>,
    ) -> EResult<Vec<(Real, Real)>> {
        self.0.multilinear_points(reference_length_m, 1.0)
    }
}

/// Arena `tz.api_sand`: API RP 2GEO §8.4.2, figura 2 y tabla 2.
///
/// Ascenso tabulado y meseta t/tmax = 1 desde zpeak. Interpolación lineal
/// y rama negativa con igual forma y escala tmax_tension: derivación propia.
/// Requiere todos los campos de [`ApiTzParams`]; errores según el módulo.
pub fn api_sand(params: ApiTzParams) -> EResult<TzCurve> {
    api_tz(params, None)
}

/// Arcilla `tz.api_clay`: API RP 2GEO §8.4.2, figura 2 y tabla 2.
///
/// `residual_ratio` es obligatorio, 0 < tres/tmax ≤ 1. La fuente indica
/// 0.70–0.90, pero aquí no se impone ese intervalo. Se fija zres = 2·zpeak
/// según la tabla 2; otros zres admitidos por la fuente quedan fuera de alcance.
/// Entre pico y residual se interpola linealmente (derivación propia);
/// después hay meseta. La rama negativa refleja la forma, con escala ft(z).
/// tmax y zpeak quedan a cargo del usuario, véase [`ApiTzParams`].
pub fn api_clay(params: ApiTzParams, residual_ratio: Option<Real>) -> EResult<TzCurve> {
    let ratio = positive(residual_ratio, "tres/tmax")?;
    if ratio > 1.0 {
        return Err(domain("tres/tmax debe ser ≤ 1."));
    }
    api_tz(params, Some(ratio))
}

/// Tabla `tz.user`, sin ordenar, corregir ni imponer monotonía de t.
///
/// Incluye (0, 0), z estrictamente creciente y valores finitos. La elección
/// de tracción es explícita; véase [`UserTraction`]. Se admiten pendientes
/// negativas: engine-laws actual no las admite y el adaptador las diagnostica.
/// `None` da `ENG-E-0003`; demás errores según la documentación del módulo.
pub fn user(points: Option<Vec<(Real, Real)>>, traction: UserTraction) -> EResult<TzCurve> {
    Ok(TzCurve(user_table(points, traction)?))
}

// Vértices numéricos de §8.4.2 tabla 2, suministrados por J2.
const ASCENDING: [(Real, Real); 6] = [
    (0.0, 0.0),
    (0.16, 0.30),
    (0.31, 0.50),
    (0.57, 0.75),
    (0.80, 0.90),
    (1.0, 1.0),
];

fn api_tz(params: ApiTzParams, residual: Option<Real>) -> EResult<TzCurve> {
    let compression = positive(params.tmax_compression_pa, "tmax_compression_pa")?;
    let tension = positive(params.tmax_tension_pa, "tmax_tension_pa")?;
    let peak = positive(params.zpeak_m, "zpeak_m")?;
    let plateau = positive(params.plateau_z_m, "plateau_z_m")?;
    let last_z = finite(peak * if residual.is_some() { 2.0 } else { 1.0 }, "zres_m")?;
    if plateau <= last_z {
        return Err(domain(
            "plateau_z_m debe superar el último vértice normativo.",
        ));
    }
    let mut shape: Vec<_> = ASCENDING
        .iter()
        .map(|&(z, t)| Ok((product(z, peak, "z del vértice")?, t)))
        .collect::<EResult<_>>()?;
    let end_ratio = match residual {
        Some(ratio) => {
            shape.push((last_z, ratio));
            ratio
        }
        None => 1.0, // Meseta de arena, no un parámetro omitido.
    };
    shape.push((plateau, end_ratio));
    let mut points: Vec<_> = shape
        .iter()
        .skip(1)
        .rev()
        .map(|&(z, t)| Ok((-z, -product(t, tension, "t de tracción")?)))
        .collect::<EResult<_>>()?;
    for &(z, t) in &shape {
        points.push((z, product(t, compression, "t de compresión")?));
    }
    Ok(TzCurve(Table::new(points)?))
}

/// Curvas de punta q-z: API RP 2GEO §8.4.3, figura 3, o tabla explícita.
/// También exportado como `engine_geo::qz`.
pub mod qz {
    use super::*;

    /// Parámetros obligatorios de `qz.api`, en SI sin prefijos.
    #[derive(Clone, Copy, Debug, PartialEq)]
    pub struct ApiQzParams {
        /// Capacidad de punta Qp (N), positiva; cálculo de §8.1 fuera de alcance.
        pub qp_n: Option<Real>,
        /// Diámetro D (m), estrictamente positivo.
        pub diameter_m: Option<Real>,
        /// Extremo de meseta z (m), estrictamente mayor que 0.10·D.
        pub plateau_z_m: Option<Real>,
    }

    /// Tabla inmutable (z en m, Q en N), distinta dimensionalmente de t-z.
    #[derive(Clone, Debug, PartialEq)]
    pub struct QzCurve(Table);

    impl QzCurve {
        /// Vértices ordenados, incluyendo (0, 0) y rama negativa declarada.
        pub fn points(&self) -> &[(Real, Real)] {
            &self.0.points
        }

        /// Resistencia de punta, N, interpolada; prolonga tramos extremos.
        /// Para [`api`], Q = 0 en todo z < 0 y Q = Qp en z ≥ 0.10·D.
        pub fn at(&self, z_m: Real) -> EResult<Real> {
            self.0.at(z_m)
        }

        /// Adapta a (ε, σ): ε = −z/L y σ = −Q/A, invirtiendo el orden.
        /// L (m) y A (m²) son obligatorios, positivos y finitos; no se
        /// identifica Q (N) con σ (Pa). El gap de [`api`] está incluido en
        /// la tabla, con compresión ε < 0. Para [`api`], `CompressionOnly`
        /// conserva en el origen la tangente del lado de compresión; `Both`
        /// da las mismas resistencias pero tangente cero en el origen (gap
        /// derecho). Para tablas de usuario bilaterales se usa `Both`.
        /// Rechaza pendientes negativas con `ENG-E-0010`.
        pub fn multilinear_points(
            &self,
            reference_length_m: Option<Real>,
            reference_area_m2: Option<Real>,
        ) -> EResult<Vec<(Real, Real)>> {
            let area = positive(reference_area_m2, "reference_area_m2")?;
            self.0.multilinear_points(reference_length_m, area)
        }
    }

    /// `qz.api`: API RP 2GEO §8.4.3, figura 3.
    ///
    /// Interpolación lineal propia entre vértices Q/Qp frente a z/D, meseta
    /// desde 0.10·D y cierre explícito posterior. Se añade (−zmeseta, 0):
    /// punta sin tracción para z < 0, idealización propia del contacto.
    /// No se calculan Qp ni D; todos los parámetros son del usuario.
    pub fn api(params: ApiQzParams) -> EResult<QzCurve> {
        let qp = positive(params.qp_n, "qp_n")?;
        let diameter = positive(params.diameter_m, "diameter_m")?;
        let plateau = positive(params.plateau_z_m, "plateau_z_m")?;
        let last = finite(0.1 * diameter, "0.10·D")?;
        if plateau <= last {
            return Err(domain("plateau_z_m debe superar 0.10·D."));
        }
        // Vértices numéricos de §8.4.3 figura 3, suministrados por J2.
        let normalized = [
            (0.0, 0.0),
            (0.002, 0.25),
            (0.013, 0.50),
            (0.042, 0.75),
            (0.073, 0.90),
            (0.100, 1.00),
        ];
        let mut points = vec![(-plateau, 0.0)];
        for (z, q) in normalized {
            points.push((
                product(z, diameter, "z del vértice q-z")?,
                product(q, qp, "Q del vértice")?,
            ));
        }
        points.push((plateau, qp));
        Ok(QzCurve(Table::new(points)?))
    }

    /// `qz.user`: (z, Q) explícitos, mismos requisitos que [`super::user`].
    /// La rama negativa es explícita o simétrica por opción; puede expresar
    /// Q = 0 mediante puntos negativos de ordenada cero. A diferencia de
    /// [`api`], no se impone un gap a los datos del usuario.
    pub fn user(points: Option<Vec<(Real, Real)>>, traction: UserTraction) -> EResult<QzCurve> {
        Ok(QzCurve(user_table(points, traction)?))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Table {
    points: Vec<(Real, Real)>,
}

impl Table {
    fn new(points: Vec<(Real, Real)>) -> EResult<Self> {
        for &(z, value) in &points {
            finite(z, "z del vértice (m)")?;
            finite(value, "resistencia del vértice")?;
        }
        if points.len() < 2 || !points.contains(&(0.0, 0.0)) {
            return Err(domain(
                "La tabla requiere al menos dos puntos y el origen (0, 0).",
            ));
        }
        for pair in points.windows(2) {
            let ((z0, v0), (z1, v1)) = (pair[0], pair[1]);
            if z1 <= z0 {
                return Err(domain(
                    "z debe ser estrictamente creciente; no se ordenan los datos.",
                ));
            }
            let dz = finite(z1 - z0, "Δz del tramo")?;
            let dv = finite(v1 - v0, "Δresistencia del tramo")?;
            let slope = finite(dv / dz, "pendiente del tramo")?;
            if dv != 0.0 && slope == 0.0 {
                return Err(domain(
                    "Pendiente no nula reducida a cero por subdesbordamiento.",
                ));
            }
        }
        Ok(Self { points })
    }

    fn at(&self, z: Real) -> EResult<Real> {
        finite(z, "z de evaluación (m)")?;
        let upper = self.points.partition_point(|&(x, _)| x <= z);
        let i = upper.saturating_sub(1).min(self.points.len() - 2);
        let ((z0, v0), (z1, v1)) = (self.points[i], self.points[i + 1]);
        if z == z0 || v0 == v1 {
            return Ok(v0);
        }
        if z == z1 {
            return Ok(v1);
        }
        let slope = finite((v1 - v0) / (z1 - z0), "pendiente de evaluación")?;
        let (anchor_z, anchor_v) = if z <= z0 * 0.5 + z1 * 0.5 {
            (z0, v0)
        } else {
            (z1, v1)
        };
        let dz = finite(z - anchor_z, "Δz de evaluación")?;
        let dv = finite(slope * dz, "incremento de resistencia")?;
        finite(anchor_v + dv, "resistencia interpolada/extrapolada")
    }

    fn multilinear_points(&self, length: Option<Real>, area: Real) -> EResult<Vec<(Real, Real)>> {
        let length = positive(length, "reference_length_m")?;
        if self.points.windows(2).any(|p| p[1].1 < p[0].1) {
            return Err(domain("La ley multilineal elástica ENG-MAT-002 no admite ablandamiento (pendiente negativa)."));
        }
        let points: Vec<_> = self
            .points
            .iter()
            .rev()
            .map(|&(z, v)| (-z / length, -v / area))
            .collect();
        // Revalida orden, finitud y pendientes tras cambiar escala y signo.
        let table = Self::new(points)?;
        for (&(_, v), &(_, scaled)) in self.points.iter().rev().zip(&table.points) {
            if v != 0.0 && scaled == 0.0 {
                return Err(domain("Resistencia reducida a cero al adaptar unidades."));
            }
        }
        Ok(table.points)
    }
}

fn user_table(points: Option<Vec<(Real, Real)>>, traction: UserTraction) -> EResult<Table> {
    let mut table = Table::new(points.ok_or_else(|| missing("points"))?)?;
    match traction {
        UserTraction::Explicit => {
            if !table.points.iter().any(|&(z, _)| z < 0.0)
                || !table.points.iter().any(|&(z, _)| z > 0.0)
            {
                return Err(domain(
                    "La opción Explicit requiere puntos negativos y positivos.",
                ));
            }
            if table
                .points
                .iter()
                .any(|&(z, v)| (z < 0.0 && v > 0.0) || (z > 0.0 && v < 0.0))
            {
                return Err(domain(
                    "La opción Explicit requiere resistencia ≤ 0 para z < 0 y ≥ 0 para z > 0.",
                ));
            }
        }
        UserTraction::Symmetric => {
            if table.points[0] != (0.0, 0.0) {
                return Err(domain(
                    "La opción Symmetric requiere solo origen y puntos positivos.",
                ));
            }
            let mut reflected: Vec<_> = table
                .points
                .iter()
                .skip(1)
                .rev()
                .map(|&(z, v)| (-z, -v))
                .collect();
            reflected.append(&mut table.points);
            table = Table::new(reflected)?;
        }
    }
    Ok(table)
}

fn missing(name: &str) -> EngineError {
    EngineError::new(
        "ENG-E-0003",
        format!("Falta el parámetro obligatorio {name}."),
    )
}

fn domain(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}

fn finite(value: Real, name: &str) -> EResult<Real> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(EngineError::new(
            "ENG-E-0008",
            format!("{name}: entrada o resultado no finito."),
        ))
    }
}

fn positive(value: Option<Real>, name: &str) -> EResult<Real> {
    let value = finite(value.ok_or_else(|| missing(name))?, name)?;
    if value <= 0.0 {
        return Err(domain(&format!("{name} debe ser estrictamente positivo.")));
    }
    Ok(value)
}

fn product(a: Real, b: Real, name: &str) -> EResult<Real> {
    let result = finite(a * b, name)?;
    if a != 0.0 && b != 0.0 && result == 0.0 {
        return Err(domain(&format!(
            "{name}: producto no nulo reducido a cero."
        )));
    }
    Ok(result)
}
