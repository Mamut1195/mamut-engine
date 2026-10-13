//! Ejes explícitos y validación del contexto, sin inferir longitud ni dirección.
use crate::{finite, invalid, vector, LinearSpring};
use engine_contracts::ElementCtx;
use engine_core::{Dim, Dof, EResult, Real};

/// Ejes fijos del resorte; se deben rotar junto con el modelo global.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpringAxes {
    /// GDL en ejes globales.
    Global,
    /// Dirección de Ux local, normalizada internamente; finita y no nula.
    Direction([Real; 3]),
    /// Rotación global→local, filas ortonormales y determinante +1.
    /// D2 conserva XY y +Z; tolerancia absoluta 1e-12 en productos escalares.
    Rotation([[Real; 3]; 3]),
}

const COMPONENTS: [Dof; 6] = [Dof::Ux, Dof::Uy, Dof::Uz, Dof::Rx, Dof::Ry, Dof::Rz];
// El contrato exige &'static [Dof]; tabla finita sin fugas ni registro mutable.
static DOFS: [[Dof; 6]; 64] = {
    let mut table = [[Dof::Ux; 6]; 64];
    let mut mask = 0;
    while mask < 64 {
        let mut axis = 0;
        let mut count = 0;
        while axis < 6 {
            if mask & (1 << axis) != 0 {
                table[mask][count] = COMPONENTS[axis];
                count += 1;
            }
            axis += 1;
        }
        mask += 1;
    }
    table
};
pub(crate) fn dofs(mask: usize) -> &'static [Dof] {
    &DOFS[mask][..mask.count_ones() as usize]
}

pub(crate) fn coupling(
    dim: Dim,
    dofs: &[Dof],
    axes: SpringAxes,
) -> EResult<([Real; 6], [Real; 6], usize)> {
    let Some(first) = dofs.first() else {
        return Err(invalid("Resorte: GDL vacíos."));
    };
    let rotation = first.index() >= 3;
    let mut local = [0.0; 6];
    let mut mask = 0;
    for &dof in dofs {
        let i = dof.index();
        if !dim.dofs().contains(&dof) || local[i] != 0.0 || (i >= 3) != rotation {
            return Err(invalid(
                "Resorte: GDL ajeno a dimensión, duplicado o familias mezcladas.",
            ));
        }
        local[i] = 1.0;
        mask |= 1 << i;
    }
    let mut global = [0.0; 6];
    match axes {
        SpringAxes::Global => global = local,
        SpringAxes::Direction(d) => {
            if dofs != [Dof::Ux] {
                return Err(invalid("Dirección requiere solo Ux local."));
            }
            vector(&d, 3, "Dirección")?;
            if dim == Dim::D2 && d[2] != 0.0 {
                return Err(invalid("Dirección D2 requiere Z=0."));
            }
            let scale = d.iter().fold(0.0_f64, |m, v| m.max(v.abs()));
            if scale == 0.0 {
                return Err(invalid("Dirección nula."));
            }
            // Escalado previo: normaliza incluso MAX o subnormales sin overflow.
            let scaled = d.map(|v| v / scale);
            let norm = scaled.iter().fold(0.0_f64, |m, &v| m.hypot(v));
            for i in 0..3 {
                global[i] = finite(scaled[i] / norm, "Dirección unitaria")?;
            }
        }
        SpringAxes::Rotation(q) => {
            validate_rotation(dim, &q)?;
            let start = if rotation { 3 } else { 0 };
            for j in 0..3 {
                for i in 0..3 {
                    global[start + j] = finite(
                        global[start + j] + local[start + i] * q[i][j],
                        "Acoplamiento global",
                    )?;
                }
            }
        }
    }
    if !matches!(axes, SpringAxes::Global) {
        mask = 0;
        for &dof in dim.dofs() {
            if (dof.index() >= 3) == rotation {
                mask |= 1 << dof.index();
            }
        }
    }
    Ok((local, global, mask))
}

fn validate_rotation(dim: Dim, q: &[[Real; 3]; 3]) -> EResult<()> {
    for row in q {
        vector(row, 3, "Rotación")?;
    }
    for i in 0..3 {
        for j in 0..=i {
            let mut dot = 0.0;
            for (&a, &b) in q[i].iter().zip(&q[j]) {
                dot = finite(
                    dot + finite(a * b, "Producto de rotación")?,
                    "Producto escalar",
                )?;
            }
            let target = if i == j { 1.0 } else { 0.0 };
            if (dot - target).abs() > 1e-12 {
                return Err(invalid("Rotación no ortonormal."));
            }
        }
    }
    let det = q[0][0] * (q[1][1] * q[2][2] - q[1][2] * q[2][1])
        - q[0][1] * (q[1][0] * q[2][2] - q[1][2] * q[2][0])
        + q[0][2] * (q[1][0] * q[2][1] - q[1][1] * q[2][0]);
    if (finite(det, "Determinante")? - 1.0).abs() > 1e-12 {
        return Err(invalid("Rotación requiere determinante +1."));
    }
    if dim == Dim::D2 && (q[0][2] != 0.0 || q[1][2] != 0.0 || q[2] != [0.0, 0.0, 1.0]) {
        return Err(invalid("Rotación D2 debe conservar XY y +Z."));
    }
    Ok(())
}

pub(crate) fn validate(spring: &LinearSpring, ctx: &ElementCtx<'_>) -> EResult<()> {
    use engine_contracts::Element;
    if ctx.dim != spring.dim || ctx.coords.len() != spring.n_nodes() {
        return Err(invalid(
            "Resorte: dimensión o número de coordenadas incompatible.",
        ));
    }
    for p in ctx.coords {
        vector(p, 3, "Coordenadas (m)")?;
        if ctx.dim == Dim::D2 && p[2] != 0.0 {
            return Err(invalid("D2 requiere Z=0."));
        }
    }
    Ok(())
}
