//! Eliminación de GDL con valores prescritos constantes en SI.
use crate::{error, finite, size, DofMap};
use engine_core::{Dof, DofIndex, EResult, NodeId, Real};
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

/// Partición inmutable en GDL libres f y prescritos p.
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
}

impl Constraints {
    /// Construye la partición sobre map; todos los GDL restringidos es ENG-E-0010.
    ///
    /// Propaga referencias/índices inválidos del mapa; ENG-E-0002 por apoyo o
    /// GDL duplicado; ENG-E-0008 por valor no finito. No hay maestro-esclavo.
    pub fn new(map: &DofMap, supports: &[Support], prescribed: &[PrescribedDof]) -> EResult<Self> {
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
        let mut free = Vec::new();
        let mut reduced = vec![None; map.len()];
        for (i, value) in values.iter().enumerate() {
            if value.is_none() {
                reduced[i] = Some(DofIndex(free.len() as u32));
                free.push(DofIndex(i as u32));
            }
        }
        if free.is_empty() {
            return Err(error("ENG-E-0010", "Todos los GDL están restringidos."));
        }
        Ok(Self {
            values,
            free,
            reduced,
        })
    }

    /// Índices globales libres ordenados; su posición es el índice reducido.
    pub fn free_dofs(&self) -> &[DofIndex] {
        &self.free
    }

    /// Valores prescritos en orden global; None identifica un GDL libre.
    pub fn prescribed_values(&self) -> &[Option<Real>] {
        &self.values
    }

    /// Índice reducido para un GDL global, o None si está restringido.
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
        let mut result: Vec<_> = self.values.iter().map(|v| v.unwrap_or(0.0)).collect();
        for (index, value) in self.free.iter().zip(free) {
            result[index.0 as usize] = *value;
        }
        Ok(result)
    }
}
