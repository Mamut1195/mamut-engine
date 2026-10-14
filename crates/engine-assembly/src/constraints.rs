//! Transformación maestro-esclavo y eliminación de prescritos constantes en SI.
use crate::{
    diaphragm::{relations, Relations},
    error, finite, size, DofMap, RigidDiaphragm,
};
use engine_core::{Dof, DofIndex, EResult, NodeId, Real};
use engine_linalg::DMat;
use engine_model::Support;
use std::collections::BTreeSet;

/// Desplazamiento o giro impuesto sobre un GDL global nodal.
#[derive(Clone, Copy, Debug)]
pub struct PrescribedDof {
    /// Nodo receptor.
    pub node: NodeId,
    /// Componente en ejes globales.
    pub dof: Dof,
    /// Traslación en m o giro en rad, constante e independiente del factor de carga.
    pub value: Real,
}

/// Transformación inmutable y partición de GDL independientes libres/prescritos.
///
/// Los apoyos del dominio fijan cero. Una declaración PrescribedDof puede
/// sustituir ese cero o restringir un GDL previamente libre. Repetir un GDL
/// entre apoyos, o entre declaraciones prescritas, es un error aunque coincida
/// el valor. La intersección apoyo/prescrito sí está permitida deliberadamente.
#[derive(Clone, Debug)]
pub struct Constraints {
    values: Vec<Option<Real>>,
    free: Vec<DofIndex>,
    reduced: Vec<Option<DofIndex>>,
    relations: Relations,
    independent: Vec<DofIndex>,
    rows: Relations,
    diaphragms: Vec<RigidDiaphragm>,
}

impl Constraints {
    /// Construye la partición sobre map; todos los GDL restringidos es ENG-E-0010.
    ///
    /// Propaga referencias/índices inválidos del mapa; ENG-E-0002 por apoyo o
    /// GDL duplicado; ENG-E-0008 por valor no finito. Equivale a with_diaphragms sin diafragmas.
    pub fn new(map: &DofMap, supports: &[Support], prescribed: &[PrescribedDof]) -> EResult<Self> {
        Self::with_diaphragms(map, supports, prescribed, &[])
    }

    /// Construye u = T_f q_f + u_0 con diafragmas XY en D3.
    ///
    /// Se transforman primero todos los GDL independientes (incluidos los
    /// prescritos), y luego se eliminan los prescritos. Un maestro puede tener
    /// apoyo o valor impuesto. Cualquier GDL prescrito o apoyado de un esclavo
    /// se rechaza con ENG-E-0013, también fuera de plano, para evitar ambigüedad.
    /// Uz/Rx/Ry siguen siendo independientes y no reciben restricciones nuevas.
    /// Propaga los diagnósticos de new y de RigidDiaphragm; todos los
    /// independientes prescritos se rechazan con ENG-E-0010.
    pub fn with_diaphragms(
        map: &DofMap,
        supports: &[Support],
        prescribed: &[PrescribedDof],
        diaphragms: &[RigidDiaphragm],
    ) -> EResult<Self> {
        let relations = relations(map, diaphragms)?;
        let slaves: BTreeSet<_> = diaphragms
            .iter()
            .flat_map(|d| d.slaves.iter().copied())
            .collect();
        let mut values = vec![None; map.len()];
        let mut ids = BTreeSet::new();
        for support in supports {
            // Incluso un apoyo vacío debe apuntar a un nodo existente.
            if !map.contains_node(support.node) {
                let mut e = error(
                    "ENG-E-0003",
                    format!("Apoyo {} sobre nodo inexistente.", support.id),
                );
                e.entities.push(engine_core::EntityRef::Node(support.node));
                return Err(e);
            }
            if !ids.insert(&support.id) {
                return Err(error(
                    "ENG-E-0002",
                    format!("Apoyo {} duplicado.", support.id),
                ));
            }
            for &dof in &support.restrained {
                let i = map.global_index(support.node, dof)?.0 as usize;
                if values[i].replace(0.0).is_some() {
                    return Err(error(
                        "ENG-E-0002",
                        format!("Apoyo repetido: nodo {}, GDL {dof}.", support.node.0),
                    ));
                }
            }
        }
        let mut seen = BTreeSet::new();
        for p in prescribed {
            finite(&[p.value])?;
            let i = map.global_index(p.node, p.dof)?.0 as usize;
            if !seen.insert(i) {
                return Err(error("ENG-E-0002", "Desplazamiento impuesto duplicado."));
            }
            values[i] = Some(p.value);
        }
        for (i, value) in values.iter().enumerate() {
            if value.is_some() {
                let (node, _) = map.label(DofIndex(i as u32))?;
                if slaves.contains(&node) {
                    let mut e = error(
                        "ENG-E-0013",
                        "No se admite un GDL apoyado o prescrito en un esclavo.",
                    );
                    e.entities.push(engine_core::EntityRef::Node(node));
                    return Err(e);
                }
            }
        }
        let independent: Vec<_> = relations
            .iter()
            .enumerate()
            .filter_map(|(i, row)| {
                (row.as_slice() == [(DofIndex(i as u32), 1.0)]).then_some(DofIndex(i as u32))
            })
            .collect();
        let mut free = Vec::new();
        let mut reduced = vec![None; map.len()];
        for index in &independent {
            let i = index.0 as usize;
            if values[i].is_none() {
                reduced[i] = Some(DofIndex(free.len() as u32));
                free.push(DofIndex(i as u32));
            }
        }
        if free.is_empty() {
            return Err(error("ENG-E-0010", "Todos los GDL están restringidos."));
        }
        let rows = relations
            .iter()
            .map(|row| {
                row.iter()
                    .filter_map(|&(i, a)| reduced[i.0 as usize].map(|r| (r, a)))
                    .collect()
            })
            .collect();
        Ok(Self {
            relations,
            independent,
            rows,
            diaphragms: diaphragms.to_vec(),
            values,
            free,
            reduced,
        })
    }

    /// Índices globales independientes libres; su posición es el índice reducido.
    pub fn free_dofs(&self) -> &[DofIndex] {
        &self.free
    }

    /// Valores prescritos en orden global; None incluye libres y dependientes.
    pub fn prescribed_values(&self) -> &[Option<Real>] {
        &self.values
    }

    /// Índice reducido de un independiente; None si es prescrito o esclavo.
    /// Para un esclavo use free_transformation_row: puede depender de varios GDL.
    /// ENG-E-0010 si el índice global está fuera de rango.
    pub fn reduced_index(&self, global: DofIndex) -> EResult<Option<DofIndex>> {
        self.reduced.get(global.0 as usize).copied().ok_or_else(|| {
            error(
                "ENG-E-0010",
                "Índice global fuera de rango en restricciones.",
            )
        })
    }

    /// Reconstruye u global con u_f (m/rad) y los valores u_p constantes.
    /// ENG-E-0010 por tamaño, ENG-E-0008 por desplazamientos no finitos.
    pub fn expand(&self, free: &[Real]) -> EResult<Vec<Real>> {
        size(free.len(), self.free.len())?;
        finite(free)?;
        let mut result: Vec<_> = self
            .values
            .iter()
            .map(|v| match v {
                Some(x) => *x,
                None => 0.0,
            })
            .collect();
        for (index, value) in self.free.iter().zip(free) {
            result[index.0 as usize] = *value;
        }
        let expanded: Vec<_> = self
            .relations
            .iter()
            .map(|row| row.iter().map(|&(i, a)| a * result[i.0 as usize]).sum())
            .collect();
        finite(&expanded)?;
        Ok(expanded)
    }

    /// GDL globales independientes, incluidos los prescritos, en orden estable.
    pub fn independent_dofs(&self) -> &[DofIndex] {
        &self.independent
    }

    /// Matriz T: independientes completos → GDL globales completos.
    /// Las columnas siguen independent_dofs; incluye los maestros prescritos.
    /// Las filas son adimensionales salvo los brazos que acoplan m con rad.
    /// ENG-E-0010 por tamaño/reserva no representable según DMat.
    pub fn transformation(&self) -> EResult<DMat> {
        let mut matrix = DMat::zeros(self.relations.len(), self.independent.len())?
            .values()
            .to_vec();
        for (r, row) in self.relations.iter().enumerate() {
            for &(i, a) in row {
                let c = self
                    .independent
                    .binary_search(&i)
                    .map_err(|_| error("ENG-E-0012", "Transformación inconsistente."))?;
                matrix[r * self.independent.len() + c] = a;
            }
        }
        DMat::new(self.relations.len(), self.independent.len(), matrix)
    }

    /// Fila dispersa de T_f: pares (índice reducido libre, coeficiente).
    /// Una fila prescrita puede ser vacía; u_0 se obtiene con expand(q_f=0).
    /// ENG-E-0010 por índice global inválido.
    pub fn free_transformation_row(&self, global: DofIndex) -> EResult<&[(DofIndex, Real)]> {
        self.rows
            .get(global.0 as usize)
            .map(Vec::as_slice)
            .ok_or_else(|| error("ENG-E-0010", "Fila de transformación fuera de rango."))
    }

    /// Diafragmas validados en el orden de declaración.
    pub fn diaphragms(&self) -> &[RigidDiaphragm] {
        &self.diaphragms
    }

    /// Proyecta fuerzas globales mediante T_fᵀ, en N/N·m.
    /// ENG-E-0010 por tamaño; ENG-E-0008 por datos o acumulaciones no finitos.
    pub fn project_force(&self, force: &[Real]) -> EResult<Vec<Real>> {
        let projected = self.project_independent(force)?;
        Ok(self.free.iter().map(|i| projected[i.0 as usize]).collect())
    }

    pub(crate) fn project_independent(&self, force: &[Real]) -> EResult<Vec<Real>> {
        size(force.len(), self.relations.len())?;
        finite(force)?;
        let mut result = vec![0.0; force.len()];
        for (row, &f) in self.relations.iter().zip(force) {
            for &(i, a) in row {
                result[i.0 as usize] += a * f;
            }
        }
        finite(&result)?;
        Ok(result)
    }
}
