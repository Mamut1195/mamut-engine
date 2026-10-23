//! Resultados estructurales SI y consultas de un caso estático lineal.
//!
//! Datos propios con campos públicos, sin referencias a elementos en la salida.
//! Están preparados para una futura serialización: el esquema `results/1`, JSON,
//! hashes y procedencia serializada corresponden a ENG-CORE-005, aún pendiente.
//! Este módulo apila sobre ENG-ANL-001 y no implementa esa frontera.
//!
//! Las estaciones se normalizan de −0.0 a +0.0, se ordenan y deduplican,
//! siempre incluyendo los extremos.
//! Por defecto se usan `[0, 0.25, 0.5, 0.75, 1]`. En `frame2d.euler_elastic`
//! se añaden raíces exactas de cortante y de la derivada del momento bajo cargas
//! uniformes; para otros elementos los extremos son los de las estaciones
//! recuperadas (sin prometer extremos interiores de campos arbitrarios).
//! Los signos locales son los de `Element::recover`: extremos como acciones
//! sobre el miembro, estaciones por cara positiva `[N,Vy,Vz,T,My,Mz]`.

use crate::{finite, StaticCaseResult};
use engine_assembly::LinearSystem;
use engine_contracts::{CaseRef, Element, ElementCtx, ElementOutput, ElementParams, MemberLoad};
use engine_core::math;
use engine_core::{Dim, DofIndex, EResult, EngineError, EntityRef, MemberId, NodeId, Real};
use engine_linalg::LinAlgBackend;
use std::collections::{BTreeMap, BTreeSet};

/// Estaciones adimensionales utilizadas cuando no se proporciona una lista.
pub const DEFAULT_STATIONS: [Real; 5] = [0.0, 0.25, 0.5, 0.75, 1.0];

/// Vista del miembro resuelto; debe corresponder al mismo modelo/caso del sistema.
/// No ensambla, acepta ni modifica la historia. Las cargas se escalan una vez
/// por `StaticCaseResult::factor`, incluidos factores nulos o negativos.
/// La colección debe incluir todos los miembros para verificar equilibrio nodal.
pub struct MemberRecovery<'a> {
    /// Identificador único del miembro; no se interpreta como índice de vector.
    pub id: MemberId,
    /// Nodos en el orden exigido por el elemento.
    pub nodes: &'a [NodeId],
    /// Elemento que produjo el aporte ensamblado.
    pub element: &'a dyn Element,
    /// Dimensión del contexto elemental.
    pub dim: Dim,
    /// Parámetros SI resueltos del elemento.
    pub params: &'a ElementParams,
    /// Historia aceptada, nunca historia de prueba.
    pub committed: &'a [Real],
    /// Cargas uniformes de este caso y miembro, SIN escalar.
    pub loads: &'a [MemberLoad],
    /// Estaciones x/L; `None` usa las predeterminadas, lista vacía solo extremos.
    pub stations: Option<&'a [Real]>,
}

/// Campo seleccionable; las resultantes locales mantienen ambas direcciones D3.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    /// Vector nodal global de traslaciones m y giros rad.
    Displacements,
    /// Vector nodal global de fuerzas N y pares N·m.
    Reactions,
    /// Acciones locales de extremo en el orden de GDL del elemento.
    EndForcesLocal,
    /// Fuerza axial N, positiva en tracción.
    N,
    /// Cortante local y, N (V de frame2d).
    Vy,
    /// Cortante local z, N.
    Vz,
    /// Torsión local x, N·m.
    T,
    /// Momento local y, N·m.
    My,
    /// Momento local z, N·m (M de frame2d).
    Mz,
}

const FORCE_FIELDS: [Field; 6] = [
    Field::N,
    Field::Vy,
    Field::Vz,
    Field::T,
    Field::My,
    Field::Mz,
];

/// Desplazamientos y reacciones agrupados por nodo.
#[derive(Clone, Debug, PartialEq)]
pub struct NodeResult {
    /// Identificador del nodo.
    pub id: NodeId,
    /// `[Ux,Uy,Uz,Rx,Ry,Rz]` en m/rad; componentes ausentes valen cero.
    pub displacements: [Real; 6],
    /// Valores prescritos `[Ux,Uy,Uz,Rx,Ry,Rz]` en m/rad, sin factor de carga.
    /// `None` indica un GDL libre, dependiente o ausente. Al construir resultados
    /// manualmente se deben conservar estos datos para combinar asentamientos.
    pub prescribed_displacements: [Option<Real>; 6],
    /// `[Fx,Fy,Fz,Mx,My,Mz]` en N/N·m; GDL libres o ausentes valen cero.
    pub reactions: [Real; 6],
}

/// Resultantes locales exactas entregadas por el elemento en una estación.
#[derive(Clone, Debug, PartialEq)]
pub struct StationResult {
    /// Posición adimensional x/L en `[0,1]`.
    pub position: Real,
    /// `[N,Vy,Vz,T,My,Mz]` en N/N·m, por cara positiva local.
    pub forces_local: [Real; 6],
}

/// Valor extremo algebraico y ubicación (no extremo de valor absoluto).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Extreme {
    /// Valor SI del campo.
    pub value: Real,
    /// Estación x/L donde ocurre; en empates se conserva la primera (menor x/L).
    pub at_station: Real,
}

/// Máximo y mínimo algebraicos de un campo de miembro.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldExtremes {
    /// Uno de N, Vy, Vz, T, My, Mz.
    pub field: Field,
    /// Máximo y su ubicación.
    pub max: Extreme,
    /// Mínimo y su ubicación.
    pub min: Extreme,
}

/// Acciones, diagramas y extremos de un miembro.
#[derive(Clone, Debug, PartialEq)]
pub struct MemberResult {
    /// Identificador del miembro.
    pub id: MemberId,
    /// Acciones locales sobre extremos i/j, nodo y después GDL del elemento.
    /// Frame2d: `[Ni,Vi,Mi,Nj,Vj,Mj]`; truss: traslaciones locales por nodo.
    pub end_forces_local: Vec<Real>,
    /// Estaciones pedidas más extremos y estaciones críticas, en orden creciente.
    pub stations: Vec<StationResult>,
    /// Seis pares máximo/mínimo, en orden N, Vy, Vz, T, My, Mz.
    pub extremes: Vec<FieldExtremes>,
}

/// Modelo de datos propio del postproceso; serialización `results/1` pendiente.
#[derive(Clone, Debug, PartialEq)]
pub struct CaseResults {
    /// Caso nominal del sistema.
    pub case: CaseRef,
    /// Factor aplicado a cargas, no a asentamientos.
    pub factor: Real,
    /// Todos los nodos del mapa, por identificador ascendente.
    pub nodes: Vec<NodeResult>,
    /// Miembros recuperados, por identificador ascendente.
    pub members: Vec<MemberResult>,
}

/// Proyección nodal de una consulta; `None` significa campo excluido.
#[derive(Clone, Debug, PartialEq)]
pub struct QueriedNode {
    /// Nodo seleccionado.
    pub id: NodeId,
    /// Desplazamientos globales en el orden de [`NodeResult`].
    pub displacements: Option<[Real; 6]>,
    /// Reacciones globales en el orden de [`NodeResult`].
    pub reactions: Option<[Real; 6]>,
}

/// Valor de un campo local seleccionado en una estación.
#[derive(Clone, Debug, PartialEq)]
pub struct FieldValue {
    /// Campo local.
    pub field: Field,
    /// Valor en N o N·m según el campo.
    pub value: Real,
}

/// Estación proyectada sobre los campos locales seleccionados.
#[derive(Clone, Debug, PartialEq)]
pub struct QueriedStation {
    /// Estación x/L.
    pub position: Real,
    /// Solo campos locales seleccionados, en orden N, Vy, Vz, T, My, Mz.
    pub values: Vec<FieldValue>,
}

/// Proyección de un miembro, sin valores de campos excluidos.
#[derive(Clone, Debug, PartialEq)]
pub struct QueriedMember {
    /// Miembro seleccionado.
    pub id: MemberId,
    /// Acciones de extremo, o `None` si se excluyó ese campo.
    pub end_forces_local: Option<Vec<Real>>,
    /// Vacía si no se seleccionó ningún campo local.
    pub stations: Vec<QueriedStation>,
    /// Solo extremos de campos locales seleccionados.
    pub extremes: Vec<FieldExtremes>,
}

/// Consulta propia serializable en el futuro, sin esquema JSON en este ticket.
#[derive(Clone, Debug, PartialEq)]
pub struct QueryResults {
    /// Caso consultado.
    pub case: CaseRef,
    /// Factor del caso.
    pub factor: Real,
    /// Nodos seleccionados.
    pub nodes: Vec<QueriedNode>,
    /// Miembros seleccionados.
    pub members: Vec<QueriedMember>,
}

impl CaseResults {
    /// Recupera un caso usando sus desplazamientos globales (incluidos prescritos).
    /// Recalcula reacciones con K y las cargas físicas del sistema; no depende
    /// de las copias reducidas o de reacciones guardadas en el resultado de análisis.
    /// El llamador aporta todos los miembros del sistema y sus cargas del caso.
    /// Verifica por GDL global `−Σ(f_int,e − f_carga,e) + F_nodal + R = 0`:
    /// las acciones de extremo SOBRE el miembro se calculan en global mediante
    /// `Element::respond` y `Element::load_vector`, sin inferir ejes locales.
    /// Incluye acciones de diafragmas cuando existen. Exige residual relativo
    /// ≤ 1e-9 respecto a la suma de módulos de los términos de ese GDL, con
    /// piso 1 N o 1 N·m, como las verificaciones SI del análisis. La tolerancia
    /// admite redondeo de recuperación/ensamblaje, no diferencias de cargas.
    /// Una incoherencia devuelve ENG-E-0010 con caso, nodo, GDL y miembros
    /// incidentes. Es una comprobación de equilibrio, no de identidad: aportes
    /// erróneos que se cancelen nodalmente pueden ser indistinguibles.
    /// ENG-E-0002 por ids/conectividad/GDL duplicados, ENG-E-0003 por referencias
    /// ausentes, ENG-E-0010 por tamaños/estaciones/prescritos incoherentes,
    /// ENG-E-0008 por no finitos y ENG-E-0012 por salida elemental incoherente.
    /// Propaga diagnósticos de elementos con el miembro en entidades.
    pub fn recover<B: LinAlgBackend>(
        solution: &StaticCaseResult,
        system: &LinearSystem<B>,
        members: &[MemberRecovery<'_>],
    ) -> EResult<Self> {
        Self::recover_with_check(solution, system, members, &mut || Ok(()))
    }

    /// Recuperación cooperativa, consultada entre nodos y miembros.
    pub fn recover_with_check<B: LinAlgBackend>(
        solution: &StaticCaseResult,
        system: &LinearSystem<B>,
        members: &[MemberRecovery<'_>],
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<Self> {
        check()?;
        let map = system.dof_map();
        let global = &solution.global_displacements;
        size(global.len(), map.len())?;
        finite(global)?;
        finite(&[solution.factor])?;
        for (&value, prescribed) in global.iter().zip(system.constraints().prescribed_values()) {
            if prescribed.is_some_and(|p| p != value) {
                return Err(error("ENG-E-0010", "Inconsistent prescribed displacement."));
            }
        }
        let free: Vec<_> = system
            .constraints()
            .free_dofs()
            .iter()
            .map(|i| global[i.0 as usize])
            .collect();
        let reactions = system.reactions(&free, &solution.case, solution.factor)?;
        let mut balance = NodalBalance::new(map.len());
        for (i, value) in system
            .global_nodal_force(&solution.case, solution.factor)?
            .into_iter()
            .enumerate()
        {
            balance.add(i, value)?;
        }
        for r in &reactions {
            balance.add(map.global_index(r.node, r.dof)?.0 as usize, r.value)?;
        }
        for diaphragm in system.diaphragm_forces(&free, &solution.case, solution.factor)? {
            for action in std::iter::once(diaphragm.master).chain(diaphragm.slaves) {
                for (dof, value) in [
                    engine_core::Dof::Ux,
                    engine_core::Dof::Uy,
                    engine_core::Dof::Rz,
                ]
                .into_iter()
                .zip(action.values)
                {
                    balance.add(map.global_index(action.node, dof)?.0 as usize, value)?;
                }
            }
        }
        let mut nodes = BTreeMap::new();
        for (i, &value) in global.iter().enumerate() {
            check()?;
            let index =
                u32::try_from(i).map_err(|_| error("ENG-E-0010", "Global index exceeds u32."))?;
            let (id, dof) = map.label(DofIndex(index))?;
            let node = nodes.entry(id).or_insert(NodeResult {
                id,
                displacements: [0.0; 6],
                prescribed_displacements: [None; 6],
                reactions: [0.0; 6],
            });
            node.displacements[dof.index()] = value;
            node.prescribed_displacements[dof.index()] =
                system.constraints().prescribed_values()[i];
        }
        for r in reactions {
            let node = nodes
                .get_mut(&r.node)
                .ok_or_else(|| error("ENG-E-0012", "Reaction has no node."))?;
            node.reactions[r.dof.index()] = r.value;
        }
        let mut recovered = BTreeMap::new();
        for member in members {
            check()?;
            if recovered.contains_key(&member.id) {
                return Err(with_member(
                    error("ENG-E-0002", "Duplicate member."),
                    member.id,
                ));
            }
            let result = recover_member(member, map, global, solution.factor, &mut balance)
                .map_err(|e| with_member(e, member.id))?;
            recovered.insert(member.id, result);
        }
        balance.verify(map, &solution.case, members)?;
        Ok(Self {
            case: solution.case.clone(),
            factor: solution.factor,
            nodes: nodes.into_values().collect(),
            members: recovered.into_values().collect(),
        })
    }

    /// Filtra por ids y campos. `None` selecciona todos; `Some(&[])`, ninguno.
    /// Mantiene el orden original y no duplica entidades/campos pedidos varias veces.
    /// ENG-E-0003 si un nodo o miembro pedido no está en este resultado.
    /// Los campos que no aplican a un bloque no añaden valores a ese bloque.
    pub fn query(
        &self,
        nodes: Option<&[NodeId]>,
        members: Option<&[MemberId]>,
        fields: Option<&[Field]>,
    ) -> EResult<QueryResults> {
        if let Some(ids) = nodes {
            for &id in ids {
                if !self.nodes.iter().any(|n| n.id == id) {
                    let mut e = error("ENG-E-0003", "Missing node in query.");
                    e.entities.push(EntityRef::Node(id));
                    return Err(e);
                }
            }
        }
        if let Some(ids) = members {
            for &id in ids {
                if !self.members.iter().any(|m| m.id == id) {
                    return Err(with_member(
                        error("ENG-E-0003", "Missing member in query."),
                        id,
                    ));
                }
            }
        }
        let selected = |f| fields.is_none_or(|fs| fs.contains(&f));
        let local: Vec<_> = FORCE_FIELDS
            .iter()
            .enumerate()
            .filter(|(_, f)| selected(**f))
            .collect();
        Ok(QueryResults {
            case: self.case.clone(),
            factor: self.factor,
            nodes: self
                .nodes
                .iter()
                .filter(|n| nodes.is_none_or(|ids| ids.contains(&n.id)))
                .map(|n| QueriedNode {
                    id: n.id,
                    displacements: selected(Field::Displacements).then_some(n.displacements),
                    reactions: selected(Field::Reactions).then_some(n.reactions),
                })
                .collect(),
            members: self
                .members
                .iter()
                .filter(|m| members.is_none_or(|ids| ids.contains(&m.id)))
                .map(|m| QueriedMember {
                    id: m.id,
                    end_forces_local: selected(Field::EndForcesLocal)
                        .then(|| m.end_forces_local.clone()),
                    stations: if local.is_empty() {
                        vec![]
                    } else {
                        m.stations
                            .iter()
                            .map(|s| QueriedStation {
                                position: s.position,
                                values: local
                                    .iter()
                                    .map(|&(i, &field)| FieldValue {
                                        field,
                                        value: s.forces_local[i],
                                    })
                                    .collect(),
                            })
                            .collect()
                    },
                    extremes: m
                        .extremes
                        .iter()
                        .filter(|e| selected(e.field))
                        .cloned()
                        .collect(),
                })
                .collect(),
        })
    }
}

fn recover_member(
    m: &MemberRecovery<'_>,
    map: &engine_assembly::DofMap,
    global: &[Real],
    factor: Real,
    balance: &mut NodalBalance,
) -> EResult<MemberResult> {
    size(m.nodes.len(), m.element.n_nodes())?;
    size(m.committed.len(), m.element.state_len())?;
    finite(m.committed)?;
    if m.nodes.is_empty() || m.element.dofs_per_node().is_empty() {
        return Err(error("ENG-E-0010", "Member has no nodes or DOFs."));
    }
    let mut coords = Vec::new();
    let mut u = Vec::new();
    let mut seen = BTreeSet::new();
    for &node in m.nodes {
        coords.push(map.coordinates(node)?);
        for &dof in m.element.dofs_per_node() {
            let index = map.global_index(node, dof)?;
            if !seen.insert(index) {
                return Err(error(
                    "ENG-E-0002",
                    "Duplicate connectivity or element DOF.",
                ));
            }
            let value = global
                .get(index.0 as usize)
                .ok_or_else(|| error("ENG-E-0010", "Element DOF is outside the global vector."))?;
            u.push(*value);
        }
    }
    let ctx = ElementCtx {
        dim: m.dim,
        coords: &coords,
        params: m.params,
    };
    let mut loads = m.loads.to_vec();
    for load in &mut loads {
        for value in load
            .force_per_length
            .iter_mut()
            .chain(&mut load.moment_per_length)
        {
            finite(&[*value])?;
            *value *= factor;
            finite(&[*value])?;
        }
    }
    let mut stations = m.stations.unwrap_or(&DEFAULT_STATIONS).to_vec();
    finite(&stations)?;
    if stations.iter().any(|s| !(0.0..=1.0).contains(s)) {
        return Err(error("ENG-E-0010", "Station is outside [0,1]."));
    }
    stations.extend([0.0, 1.0]);
    sort_stations(&mut stations);
    let mut output = recover_output(m, &ctx, &u, &loads, &stations)?;
    if m.element.kind() == "frame2d.euler_elastic" {
        let first = output
            .stations
            .first()
            .ok_or_else(|| error("ENG-E-0012", "Recovery has no ends."))?;
        let v_i = first.forces_local[1];
        let [a, b] = coords.as_slice() else {
            return Err(error("ENG-E-0012", "Frame2d must have two nodes."));
        };
        let dx = b[0] - a[0];
        let dy = b[1] - a[1];
        let length = math::hypot(dx, dy);
        finite(&[dx, dy, length])?;
        if length <= 0.0 {
            return Err(error("ENG-E-0005", "Zero-length member."));
        }
        let mut qy = 0.0;
        let mut mz = 0.0;
        for load in &loads {
            qy += -(dy / length) * load.force_per_length[0]
                + (dx / length) * load.force_per_length[1];
            mz += load.moment_per_length[2];
            finite(&[qy, mz])?;
        }
        if qy != 0.0 {
            // V(x)=Vi−qy*x. M′=−V−mz: con mz=0 las dos raíces coinciden.
            let denominator = qy * length;
            finite(&[denominator, v_i + mz])?;
            if denominator == 0.0 {
                return Err(error("ENG-E-0010", "Transverse load per length underflow."));
            }
            for numerator in [v_i, v_i + mz] {
                // Comprobar interior antes de dividir evita desbordar una raíz exterior.
                if (denominator > 0.0 && numerator > 0.0 && numerator < denominator)
                    || (denominator < 0.0 && numerator < 0.0 && numerator > denominator)
                {
                    let position = numerator / denominator;
                    finite(&[position])?;
                    if position > 0.0 && position < 1.0 {
                        stations.push(position);
                    }
                }
            }
            sort_stations(&mut stations);
            output = recover_output(m, &ctx, &u, &loads, &stations)?;
        }
    }
    let stations: Vec<_> = output
        .stations
        .into_iter()
        .map(|s| StationResult {
            position: canonical_station(s.position),
            forces_local: s.forces_local,
        })
        .collect();
    let internal = m.element.respond(&ctx, &u, m.committed)?.internal_force;
    output_size(internal.len(), u.len())?;
    finite(&internal)?;
    let mut global_loads = Vec::new();
    for load in &loads {
        let values = m.element.load_vector(&ctx, load)?;
        output_size(values.len(), u.len())?;
        finite(&values)?;
        global_loads.push(values);
    }
    for (i, (&node, &dof)) in m
        .nodes
        .iter()
        .flat_map(|node| m.element.dofs_per_node().iter().map(move |dof| (node, dof)))
        .enumerate()
    {
        let index = map.global_index(node, dof)?.0 as usize;
        balance.add(index, -internal[i])?;
        for load in &global_loads {
            balance.add(index, load[i])?;
        }
    }
    Ok(MemberResult {
        id: m.id,
        end_forces_local: output.end_forces_local,
        extremes: station_extremes(&stations)?,
        stations,
    })
}

// Escala por GDL: no mezcla fuerzas con momentos; conserva los términos previos
// a cancelar f_int − f_carga, para admitir redondeo en extremos de momento nulo.
struct NodalBalance {
    residual: Vec<Real>,
    scale: Vec<Real>,
}

impl NodalBalance {
    fn new(size: usize) -> Self {
        Self {
            residual: vec![0.0; size],
            scale: vec![0.0; size],
        }
    }

    fn add(&mut self, index: usize, value: Real) -> EResult<()> {
        self.residual[index] += value;
        self.scale[index] += value.abs();
        finite(&[self.residual[index], self.scale[index]])
    }

    fn verify(
        &self,
        map: &engine_assembly::DofMap,
        case: &CaseRef,
        members: &[MemberRecovery<'_>],
    ) -> EResult<()> {
        for (i, (&residual, &scale)) in self.residual.iter().zip(&self.scale).enumerate() {
            let relative = residual.abs() / scale.max(1.0);
            if relative > 1e-9 {
                let index = DofIndex(
                    u32::try_from(i)
                        .map_err(|_| error("ENG-E-0010", "Global index exceeds u32."))?,
                );
                let (node, dof) = map.label(index)?;
                let mut e = error(
                    "ENG-E-0010",
                    &format!(
                        "Inconsistent recovery at node {} / {dof:?}: relative residual {relative:e} > 1e-9.",
                        node.0
                    ),
                );
                e.entities.extend([
                    EntityRef::Case(case.0.clone()),
                    EntityRef::Node(node),
                    EntityRef::Dof(index),
                ]);
                for member in members {
                    if member.nodes.contains(&node) && member.element.dofs_per_node().contains(&dof)
                    {
                        e.entities.push(EntityRef::Member(member.id));
                    }
                }
                return Err(e);
            }
        }
        Ok(())
    }
}

/// Extremos algebraicos de las estaciones, conservando la primera en empates.
pub(crate) fn station_extremes(stations: &[StationResult]) -> EResult<Vec<FieldExtremes>> {
    let first = stations
        .first()
        .ok_or_else(|| error("ENG-E-0010", "Member has no stations."))?;
    Ok(FORCE_FIELDS
        .iter()
        .enumerate()
        .map(|(i, &field)| {
            let start = Extreme {
                value: first.forces_local[i],
                at_station: first.position,
            };
            let mut max = start;
            let mut min = start;
            for s in stations {
                let candidate = Extreme {
                    value: s.forces_local[i],
                    at_station: s.position,
                };
                if candidate.value > max.value {
                    max = candidate;
                }
                if candidate.value < min.value {
                    min = candidate;
                }
            }
            FieldExtremes { field, max, min }
        })
        .collect())
}

fn recover_output(
    m: &MemberRecovery<'_>,
    ctx: &ElementCtx<'_>,
    u: &[Real],
    loads: &[MemberLoad],
    stations: &[Real],
) -> EResult<ElementOutput> {
    let output = m.element.recover(ctx, u, m.committed, loads, stations)?;
    if output.end_forces_local.len() != u.len() || output.stations.len() != stations.len() {
        return Err(error("ENG-E-0012", "Inconsistent element recovery size."));
    }
    finite(&output.end_forces_local)?;
    for (s, &position) in output.stations.iter().zip(stations) {
        finite(&[s.position])?;
        finite(&s.forces_local)?;
        if s.position != position {
            return Err(error(
                "ENG-E-0012",
                "Recovered station differs from the requested station.",
            ));
        }
    }
    Ok(output)
}

fn sort_stations(stations: &mut Vec<Real>) {
    for position in stations.iter_mut() {
        *position = canonical_station(*position);
    }
    stations.sort_by(Real::total_cmp);
    stations.dedup_by(|a, b| *a == *b);
}

fn canonical_station(position: Real) -> Real {
    if position == 0.0 {
        0.0
    } else {
        position
    }
}

fn output_size(actual: usize, expected: usize) -> EResult<()> {
    if actual != expected {
        return Err(error("ENG-E-0012", "Inconsistent element output size."));
    }
    Ok(())
}

fn size(actual: usize, expected: usize) -> EResult<()> {
    if actual != expected {
        return Err(error("ENG-E-0010", "Incompatible size in postprocessing."));
    }
    Ok(())
}

fn error(code: &'static str, message: &str) -> EngineError {
    EngineError::new(code, message)
}

fn with_member(mut e: EngineError, id: MemberId) -> EngineError {
    e.entities.push(EntityRef::Member(id));
    e
}
