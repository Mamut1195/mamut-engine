//! Superposición por factores del usuario y envolventes algebraicas en SI.
//!
//! No se aplican coeficientes normativos. Cada componente de la combinación es
//! `Σ factor * valor`, con multiplicación y suma f64 separadas, en el orden de
//! los términos (sin interpolación ni redondeo adicional). `CaseResults::factor`
//! es metadato de cargas ya escaladas: no se multiplica una segunda vez.
//! Con asentamientos la respuesta es afín: `r(F) = r(0) + L(F)`. Sumar casos
//! completos multiplicaría `r(0)` por la suma de factores (por ejemplo, 2.8 en
//! `1.2D + 1.6L`). Los resultados no separan fuerzas/reacciones de `r(0)`;
//! sustraerlo automáticamente exigiría ampliar el contrato de recuperación.
//! Por seguridad se rechazan con ENG-E-0010 y entidades Case los términos con
//! más de un caso de prescritos no nulos, incluso si tienen factor cero o los
//! mismos valores. Se rechazan también metadatos prescritos incoherentes.
//! Resuelva D y L con prescritos cero, y el asentamiento S sin cargas, manteniendo
//! los mismos GDL prescritos y rigidez; combine `1.2D + 1.6L + αS`, con factor
//! explícito `α` (1 para contar el asentamiento una vez). Un único caso con
//! asentamiento se pondera íntegramente por su factor; no se corrige en silencio.
//!
//! Las entidades se emparejan por identificador, aunque su orden difiera. Cada
//! miembro exige idénticas posiciones de estación y tamaño de acciones de extremo.
//! Los diagramas son las estaciones sumadas. Los extremos se recalculan sobre
//! esas estaciones, sin usar los máximos/mínimos almacenados en cada entrada ni
//! deducir nuevas estaciones críticas interiores a partir de muestras.
//!
//! La envolvente conserva identificador, valor y, en extremos de miembro,
//! posición. Un empate exacto conserva la primera entrada; dentro de un miembro,
//! la primera estación (menor x/L). Casos y combinaciones tienen identidades
//! distintas aun con el mismo nombre. Una envolvente no es un caso de equilibrio.

use crate::results::{station_extremes, CaseResults, Extreme, Field, MemberResult, NodeResult};
use engine_contracts::CaseRef;
use engine_core::{EResult, EngineError, EntityRef, MemberId, NodeId, Real};
use engine_model::Combination;
use std::collections::{BTreeMap, BTreeSet};

/// Resultado de una combinación del usuario, reutilizando el modelo de resultados.
#[derive(Clone, Debug, PartialEq)]
pub struct CombinationResults {
    /// Identificador de `engine_model::Combination`.
    pub id: String,
    /// Suma ponderada; `case` es el id de combinación y `factor` vale 1.
    pub results: CaseResults,
}

/// Identidad del resultado gobernante, con espacios de nombres separados.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResultId {
    /// Nombre de caso de carga.
    Case(String),
    /// Identificador de combinación.
    Combination(String),
}

/// Entrada prestada de una envolvente, conservada en orden del usuario.
#[derive(Clone, Copy, Debug)]
pub enum EnvelopeInput<'a> {
    /// Caso resuelto, incluso si ya tiene un factor de carga aplicado.
    Case(&'a CaseResults),
    /// Combinación calculada.
    Combination(&'a CombinationResults),
}

impl<'a> EnvelopeInput<'a> {
    fn id(self) -> ResultId {
        match self {
            Self::Case(r) => ResultId::Case(r.case.0.clone()),
            Self::Combination(r) => ResultId::Combination(r.id.clone()),
        }
    }

    fn results(self) -> &'a CaseResults {
        match self {
            Self::Case(r) => r,
            Self::Combination(r) => &r.results,
        }
    }
}

/// Componente SI e identidad que la gobierna.
#[derive(Clone, Debug, PartialEq)]
pub struct GoverningValue {
    /// Caso o combinación gobernante.
    pub source: ResultId,
    /// Valor algebraico, sin módulo ni escalado adicional.
    pub value: Real,
}

/// Máximo y mínimo de una componente entre entradas.
#[derive(Clone, Debug, PartialEq)]
pub struct ValueEnvelope {
    /// Máximo algebraico; primer gobernante en empates.
    pub max: GoverningValue,
    /// Mínimo algebraico; primer gobernante en empates.
    pub min: GoverningValue,
}

/// Envolventes nodales en el orden de GDL de `NodeResult`.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeEnvelope {
    /// Identificador nodal.
    pub id: NodeId,
    /// Desplazamientos `[Ux,Uy,Uz,Rx,Ry,Rz]`, m/rad.
    pub displacements: [ValueEnvelope; 6],
    /// Reacciones `[Fx,Fy,Fz,Mx,My,Mz]`, N/N·m.
    pub reactions: [ValueEnvelope; 6],
}

/// Envolventes de las seis resultantes de sección en una posición común.
#[derive(Clone, Debug, PartialEq)]
pub struct StationEnvelope {
    /// Posición adimensional x/L.
    pub position: Real,
    /// `[N,Vy,Vz,T,My,Mz]`, N/N·m.
    pub forces_local: [ValueEnvelope; 6],
}

/// Extremo de miembro gobernante, con ubicación e identidad.
#[derive(Clone, Debug, PartialEq)]
pub struct GoverningExtreme {
    /// Caso o combinación gobernante.
    pub source: ResultId,
    /// Valor SI y posición del extremo.
    pub extreme: Extreme,
}

/// Envolvente global de un campo sobre entradas y estaciones del miembro.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldEnvelope {
    /// Campo N, Vy, Vz, T, My o Mz.
    pub field: Field,
    /// Máximo de los máximos recalculados de cada entrada.
    pub max: GoverningExtreme,
    /// Mínimo de los mínimos recalculados de cada entrada.
    pub min: GoverningExtreme,
}

/// Envolventes de acciones de extremo, diagramas y extremos de sección.
#[derive(Clone, Debug, PartialEq)]
pub struct MemberEnvelope {
    /// Identificador del miembro.
    pub id: MemberId,
    /// En el orden de GDL local de `MemberResult::end_forces_local`.
    pub end_forces_local: Vec<ValueEnvelope>,
    /// Estaciones comunes, en orden creciente.
    pub stations: Vec<StationEnvelope>,
    /// Seis campos, en orden N, Vy, Vz, T, My, Mz.
    pub extremes: Vec<FieldEnvelope>,
}

/// Envolvente algebraica; cada componente puede tener distinto gobernante.
#[derive(Clone, Debug, PartialEq)]
pub struct EnvelopeResults {
    /// Identidades de las entradas en el orden usado para resolver empates.
    pub sources: Vec<ResultId>,
    /// Nodos por identificador ascendente.
    pub nodes: Vec<NodeEnvelope>,
    /// Miembros por identificador ascendente.
    pub members: Vec<MemberEnvelope>,
}

/// Superpone resultados existentes según los términos de una combinación.
///
/// ENG-E-0003: caso referenciado ausente, incluso con factor cero.
/// ENG-E-0008: factor/dato no finito o desborde de producto o suma parcial.
/// ENG-E-0010: términos/repositorio vacíos, casos o entidades duplicados,
/// topología/tamaños/estaciones incompatibles, varios casos con prescritos no
/// nulos o metadatos prescritos incoherentes (entidades Case). Todos los términos
/// deben compartir los GDL prescritos, aunque sus valores sean distintos.
/// Se validan también los términos
/// de factor cero. Los casos no referenciados solo se comprueban por duplicidad.
/// No modifica las entradas. Los extremos se calculan después de sumar.
pub fn combine(combination: &Combination, cases: &[CaseResults]) -> EResult<CombinationResults> {
    combine_with_check(combination, cases, &mut || Ok(()))
}

/// Superposición cooperativa por término, nodo y miembro, sin reordenar sumas.
pub fn combine_with_check(
    combination: &Combination,
    cases: &[CaseResults],
    check: &mut dyn FnMut() -> EResult<()>,
) -> EResult<CombinationResults> {
    check()?;
    if combination.terms.is_empty() || cases.is_empty() {
        return Err(invalid("Empty combination or case repository."));
    }
    let mut available = BTreeMap::new();
    for case in cases {
        check()?;
        if available.insert(case.case.0.as_str(), case).is_some() {
            return Err(invalid("Duplicate case in repository."));
        }
    }
    let mut seen = BTreeSet::new();
    let mut terms = Vec::new();
    let mut settlement_case = None;
    for term in &combination.terms {
        check()?;
        number(term.factor)?;
        if !seen.insert(term.case.as_str()) {
            return Err(invalid("Duplicate case in terms."));
        }
        let case = available.get(term.case.as_str()).ok_or_else(|| {
            let mut e = EngineError::new("ENG-E-0003", "Missing case in combination.");
            e.entities.push(EntityRef::Case(term.case.clone()));
            e
        })?;
        let input = layout(case)?;
        if case.nodes.iter().any(|node| {
            node.prescribed_displacements
                .iter()
                .any(|p| p.is_some_and(|value| value != 0.0))
        }) {
            if let Some(previous) = settlement_case {
                let mut e = invalid(
                    "Multiple cases with nonzero prescribed displacements. Solve the loads with zero prescribed displacements and use a separate settlement case with an explicit factor.",
                );
                e.entities.push(EntityRef::Case(previous));
                e.entities.push(EntityRef::Case(term.case.clone()));
                return Err(e);
            }
            settlement_case = Some(term.case.clone());
        }
        terms.push((term.factor, input));
    }
    let (_, first) = terms
        .first()
        .ok_or_else(|| invalid("Combination has no terms."))?;
    for (_, next) in &terms {
        check()?;
        compatible(first, next)?;
        for (a, b) in first.nodes.values().zip(next.nodes.values()) {
            if a.prescribed_displacements.map(|p| p.is_some())
                != b.prescribed_displacements.map(|p| p.is_some())
            {
                let mut e = invalid("Incompatible prescribed DOF sets across cases.");
                e.entities.push(EntityRef::Case(first.case.into()));
                e.entities.push(EntityRef::Case(next.case.into()));
                return Err(e);
            }
        }
    }
    let mut output = CaseResults {
        case: CaseRef(combination.id.clone()),
        factor: 1.0,
        nodes: first.nodes.values().map(|n| (*n).clone()).collect(),
        members: first.members.values().map(|m| (*m).clone()).collect(),
    };
    for n in &mut output.nodes {
        n.displacements.fill(0.0);
        n.reactions.fill(0.0);
    }
    for m in &mut output.members {
        m.end_forces_local.fill(0.0);
        for s in &mut m.stations {
            s.forces_local.fill(0.0);
        }
        m.extremes.clear();
    }
    for (factor, input) in &terms {
        check()?;
        for (n, source) in output.nodes.iter_mut().zip(input.nodes.values()) {
            check()?;
            add(&mut n.displacements, &source.displacements, *factor)?;
            add(&mut n.reactions, &source.reactions, *factor)?;
        }
        for (m, source) in output.members.iter_mut().zip(input.members.values()) {
            check()?;
            add(&mut m.end_forces_local, &source.end_forces_local, *factor)?;
            for (s, source) in m.stations.iter_mut().zip(&source.stations) {
                add(&mut s.forces_local, &source.forces_local, *factor)?;
            }
        }
    }
    for m in &mut output.members {
        m.extremes = station_extremes(&m.stations)?;
    }
    for n in &mut output.nodes {
        for (prescribed, &value) in n.prescribed_displacements.iter_mut().zip(&n.displacements) {
            if prescribed.is_some() {
                *prescribed = Some(value);
            }
        }
    }
    Ok(CombinationResults {
        id: combination.id.clone(),
        results: output,
    })
}

/// Calcula máximos/mínimos por componente, estación y campo de miembro.
///
/// ENG-E-0010 por entradas vacías/duplicadas o resultados incompatibles;
/// ENG-E-0008 por datos no finitos. Empates: primera entrada; a igualdad de
/// valores en un mismo miembro: primera estación. Se recalculan los extremos
/// de cada entrada sobre sus estaciones, sin confiar en extremos almacenados.
pub fn envelope(inputs: &[EnvelopeInput<'_>]) -> EResult<EnvelopeResults> {
    let first_input = inputs
        .first()
        .ok_or_else(|| invalid("Envelope has no entries."))?;
    let mut seen = BTreeSet::new();
    let mut layouts = Vec::new();
    for &input in inputs {
        if !seen.insert(input.id()) {
            return Err(invalid("Duplicate entry in envelope."));
        }
        layouts.push(layout(input.results())?);
    }
    let first = layouts
        .first()
        .ok_or_else(|| invalid("Envelope has no results."))?;
    for next in &layouts {
        compatible(first, next)?;
    }
    let id = first_input.id();
    let mut output = EnvelopeResults {
        sources: inputs.iter().map(|i| i.id()).collect(),
        nodes: first
            .nodes
            .values()
            .map(|n| NodeEnvelope {
                id: n.id,
                displacements: n.displacements.map(|v| initial(v, &id)),
                reactions: n.reactions.map(|v| initial(v, &id)),
            })
            .collect(),
        members: Vec::new(),
    };
    for m in first.members.values() {
        output.members.push(MemberEnvelope {
            id: m.id,
            end_forces_local: m
                .end_forces_local
                .iter()
                .map(|&v| initial(v, &id))
                .collect(),
            stations: m
                .stations
                .iter()
                .map(|s| StationEnvelope {
                    position: s.position,
                    forces_local: s.forces_local.map(|v| initial(v, &id)),
                })
                .collect(),
            extremes: station_extremes(&m.stations)?
                .into_iter()
                .map(|e| FieldEnvelope {
                    field: e.field,
                    max: GoverningExtreme {
                        source: id.clone(),
                        extreme: e.max,
                    },
                    min: GoverningExtreme {
                        source: id.clone(),
                        extreme: e.min,
                    },
                })
                .collect(),
        });
    }
    for (&input, data) in inputs.iter().zip(&layouts).skip(1) {
        let id = input.id();
        for (n, source) in output.nodes.iter_mut().zip(data.nodes.values()) {
            update_values(&mut n.displacements, &source.displacements, &id);
            update_values(&mut n.reactions, &source.reactions, &id);
        }
        for (m, source) in output.members.iter_mut().zip(data.members.values()) {
            update_values(&mut m.end_forces_local, &source.end_forces_local, &id);
            for (s, source) in m.stations.iter_mut().zip(&source.stations) {
                update_values(&mut s.forces_local, &source.forces_local, &id);
            }
            for (e, source) in m
                .extremes
                .iter_mut()
                .zip(station_extremes(&source.stations)?)
            {
                if source.max.value > e.max.extreme.value {
                    e.max = GoverningExtreme {
                        source: id.clone(),
                        extreme: source.max,
                    };
                }
                if source.min.value < e.min.extreme.value {
                    e.min = GoverningExtreme {
                        source: id.clone(),
                        extreme: source.min,
                    };
                }
            }
        }
    }
    Ok(output)
}

struct Layout<'a> {
    case: &'a str,
    nodes: BTreeMap<NodeId, &'a NodeResult>,
    members: BTreeMap<MemberId, &'a MemberResult>,
}

fn layout(results: &CaseResults) -> EResult<Layout<'_>> {
    number(results.factor)?;
    if results.nodes.is_empty() {
        return Err(invalid("Result has no nodes."));
    }
    let mut nodes = BTreeMap::new();
    for n in &results.nodes {
        if nodes.insert(n.id, n).is_some() {
            return Err(invalid("Duplicate node."));
        }
        numbers(&n.displacements)?;
        numbers(&n.reactions)?;
        for (prescribed, &value) in n.prescribed_displacements.iter().zip(&n.displacements) {
            if let Some(prescribed) = prescribed {
                number(*prescribed).map_err(|mut e| {
                    e.entities.push(EntityRef::Case(results.case.0.clone()));
                    e
                })?;
                if *prescribed != value {
                    let mut e =
                        invalid("Prescribed displacement is inconsistent with the nodal result.");
                    e.entities.push(EntityRef::Case(results.case.0.clone()));
                    return Err(e);
                }
            }
        }
    }
    let mut members = BTreeMap::new();
    for m in &results.members {
        if members.insert(m.id, m).is_some() {
            return Err(invalid("Duplicate member."));
        }
        if m.end_forces_local.is_empty() || m.stations.len() < 2 {
            return Err(invalid("Member has no end actions or stations."));
        }
        numbers(&m.end_forces_local)?;
        let mut previous = None;
        for s in &m.stations {
            number(s.position)?;
            numbers(&s.forces_local)?;
            if !(0.0..=1.0).contains(&s.position) || previous.is_some_and(|p| p >= s.position) {
                return Err(invalid(
                    "Stations are out of range, unordered, or duplicated.",
                ));
            }
            previous = Some(s.position);
        }
        if m.stations.first().is_none_or(|s| s.position != 0.0)
            || m.stations.last().is_none_or(|s| s.position != 1.0)
        {
            return Err(invalid("Missing end stations 0 and 1."));
        }
        // Los extremos públicos pueden estar desactualizados: no se usan al calcular.
        for e in &m.extremes {
            numbers(&[e.max.value, e.max.at_station, e.min.value, e.min.at_station])?;
        }
    }
    Ok(Layout {
        case: &results.case.0,
        nodes,
        members,
    })
}

fn compatible(a: &Layout<'_>, b: &Layout<'_>) -> EResult<()> {
    if !a.nodes.keys().eq(b.nodes.keys()) || !a.members.keys().eq(b.members.keys()) {
        return Err(invalid("Incompatible node or member sets."));
    }
    for (a, b) in a.members.values().zip(b.members.values()) {
        if a.end_forces_local.len() != b.end_forces_local.len()
            || a.stations.len() != b.stations.len()
            || !a
                .stations
                .iter()
                .zip(&b.stations)
                .all(|(a, b)| a.position == b.position)
        {
            let mut e = invalid("Incompatible end actions or stations.");
            e.entities.push(EntityRef::Member(a.id));
            return Err(e);
        }
    }
    Ok(())
}

fn add(target: &mut [Real], source: &[Real], factor: Real) -> EResult<()> {
    for (sum, &value) in target.iter_mut().zip(source) {
        let product = factor * value;
        number(product)?;
        *sum += product;
        number(*sum)?;
    }
    Ok(())
}

fn initial(value: Real, id: &ResultId) -> ValueEnvelope {
    let v = GoverningValue {
        source: id.clone(),
        value,
    };
    ValueEnvelope {
        max: v.clone(),
        min: v,
    }
}

fn update_values(target: &mut [ValueEnvelope], source: &[Real], id: &ResultId) {
    for (e, &value) in target.iter_mut().zip(source) {
        if value > e.max.value {
            e.max = GoverningValue {
                source: id.clone(),
                value,
            };
        }
        if value < e.min.value {
            e.min = GoverningValue {
                source: id.clone(),
                value,
            };
        }
    }
}

fn number(value: Real) -> EResult<()> {
    if !value.is_finite() {
        return Err(EngineError::new(
            "ENG-E-0008",
            "Combination/envelope: non-finite data or result.",
        ));
    }
    Ok(())
}

fn numbers(values: &[Real]) -> EResult<()> {
    for &value in values {
        number(value)?;
    }
    Ok(())
}

fn invalid(message: &str) -> EngineError {
    EngineError::new("ENG-E-0010", message)
}
