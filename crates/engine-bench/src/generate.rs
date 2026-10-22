//! Tres pórticos oficiales. Las propiedades salen de estas constantes; no hay azar.
//!
//! La inercia `Izz` es la de una losa rectangular uniforme respecto del eje
//! vertical que pasa por el nudo maestro, no respecto del centroide:
//! `Izz = m (Lx² + Ly²) / 12 + m d²`.

use engine_core::math;

use crate::error::BenchError;
use crate::model::{
    Axis, BenchModel, Combination, Diaphragm, DirectionalCombination, Dof, DofValue, Grid,
    LoadPatterns, Member, MemberKind, Modal, NodalAction, Node, Orientation, Restraint, SaTable,
    Section, Source, Spectrum, SpectrumDirection, Story, StoryMass, ORIENTATION_KIND, SA_UNIT,
    SCHEMA,
};

/// Vanos en X y en Y.
pub const BAYS: usize = 3;
/// Líneas de la retícula, incluida la de borde. Son 16 nudos por nivel.
pub const LINES: usize = BAYS + 1;
/// Nudos de un nivel, incluida la base.
pub const NODES_PER_LEVEL: usize = LINES * LINES;
/// Luz de cada vano, en m.
pub const SPAN_M: f64 = 6.0;
/// Altura de cada piso, en m.
pub const STORY_HEIGHT_M: f64 = 3.0;
/// Lado de la planta, en m.
pub const PLAN_M: f64 = BAYS as f64 * SPAN_M;
/// Espesor de la losa, en m.
pub const SLAB_THICKNESS_M: f64 = 0.15;
/// Densidad del hormigón, en kg/m³.
pub const CONCRETE_DENSITY_KG_M3: f64 = 2400.0;
/// Carga superficial además de la losa, en N/m².
pub const SUPERIMPOSED_N_M2: f64 = 3000.0;
/// Gravedad normal, en m/s². Sirve para masas y para pasar Sa de g a m/s².
pub const GRAVITY_M_S2: f64 = 9.80665;
/// f'c, en MPa.
pub const FC_MPA: f64 = 28.0;
/// Coeficiente de Poisson del hormigón.
pub const POISSON: f64 = 0.20;
/// Lado de la columna cuadrada, en m.
pub const COLUMN_SIDE_M: f64 = 0.70;
/// Ancho de la viga según el eje local y, en m.
pub const BEAM_WIDTH_Y_M: f64 = 0.30;
/// Canto de la viga según el eje local z, en m.
pub const BEAM_DEPTH_Z_M: f64 = 0.70;
/// Fracción fija del peso de piso aplicada en el eje del espectro.
pub const LATERAL_WEIGHT_FRACTION: f64 = 0.10;
/// Modos pedidos.
pub const MODE_COUNT: u32 = 12;
/// Zona NEC-SE-DS.
pub const ZONE_LABEL: &str = "V";
/// Factor Z de la zona V.
pub const ZONE_FACTOR: f64 = 0.40;
/// Perfil de suelo.
pub const SOIL: &str = "D";
/// Fa de suelo D y Z = 0.40.
pub const FA: f64 = 1.20;
/// Fd de suelo D y Z = 0.40.
pub const FD: f64 = 1.19;
/// Fs de suelo D.
pub const FS: f64 = 1.02;
/// η de la Sierra.
pub const ETA: f64 = 1.80;
/// R de pórticos especiales de hormigón.
pub const RESPONSE_MODIFICATION: f64 = 8.0;
/// ζ de la tabla y de CQC.
pub const ZETA: f64 = 0.05;
/// Exponente de la rama descendente.
pub const SPECTRUM_EXPONENT: f64 = 1.0;
/// Último periodo tabulado, en s.
pub const PERIOD_END_S: f64 = 10.0;
/// Paso de la tabla, en s.
pub const PERIOD_STEP_S: f64 = 0.05;

/// Modelos oficiales: 3, 8 y 15 pisos.
#[must_use]
pub fn official_models() -> [BenchModel; 3] {
    [
        build("rc3d-3s", 3),
        build("rc3d-8s", 8),
        build("rc3d-15s", 15),
    ]
}

/// Nudos y barras de un pórtico de `stories` pisos y 16 nudos por nivel.
#[must_use]
pub fn mesh_counts(stories: usize) -> (usize, usize) {
    let nodes = (stories + 1) * NODES_PER_LEVEL;
    let beams_per_level = LINES * BAYS + LINES * BAYS;
    let members = stories * NODES_PER_LEVEL + stories * beams_per_level;
    (nodes, members)
}

/// Masa traslacional de un piso, en kg.
#[must_use]
pub fn story_mass_kg() -> f64 {
    let area = PLAN_M * PLAN_M;
    area * (SLAB_THICKNESS_M * CONCRETE_DENSITY_KG_M3 + SUPERIMPOSED_N_M2 / GRAVITY_M_S2)
}

/// `Izz` de una losa rectangular uniforme respecto del maestro, en kg·m².
#[must_use]
pub fn izz_about_master(mass: f64, master_xy: [f64; 2], centroid_xy: [f64; 2]) -> f64 {
    let about_centroid = mass * (PLAN_M * PLAN_M + PLAN_M * PLAN_M) / 12.0;
    let dx = master_xy[0] - centroid_xy[0];
    let dy = master_xy[1] - centroid_xy[1];
    about_centroid + mass * (dx * dx + dy * dy)
}

/// E = 4700 sqrt(f'c), con f'c en MPa y el resultado en Pa.
#[must_use]
pub fn concrete_young_pa(fc_mpa: f64) -> f64 {
    4_700.0e6 * math::sqrt(fc_mpa)
}

/// G = E / (2 (1 + ν)).
#[must_use]
pub fn shear_modulus_pa(young: f64, poisson: f64) -> f64 {
    young / (2.0 * (1.0 + poisson))
}

/// Área e inercias de un rectángulo. `width_y` es el lado local y, `depth_z` el local z.
#[must_use]
pub fn rectangle_section(width_y: f64, depth_z: f64) -> (f64, f64, f64, f64) {
    let area = width_y * depth_z;
    let iy = width_y * math::powi(depth_z, 3) / 12.0;
    let iz = depth_z * math::powi(width_y, 3) / 12.0;
    let torsion = saint_venant_j(width_y, depth_z);
    (area, iy, iz, torsion)
}

/// J ≈ a b³ (1/3 − 0.21 (b/a) (1 − (b/a)⁴ / 12)), con a el lado mayor.
#[must_use]
pub fn saint_venant_j(side_a: f64, side_b: f64) -> f64 {
    let (long, short) = if side_a >= side_b {
        (side_a, side_b)
    } else {
        (side_b, side_a)
    };
    let ratio = short / long;
    long * math::powi(short, 3) * (1.0 / 3.0 - 0.21 * ratio * (1.0 - math::powi(ratio, 4) / 12.0))
}

/// Sa(T) en m/s². Ya incluye la división por R y el paso de g a m/s².
#[must_use]
pub fn spectral_acceleration_m_s2(period: f64) -> f64 {
    let t0 = corner_t0();
    let tc = corner_tc();
    let tl = corner_tl();
    let plateau = ETA * ZONE_FACTOR * FA;
    let elastic_g = if period <= t0 {
        ZONE_FACTOR * FA * (1.0 + (ETA - 1.0) * (period / t0))
    } else if period <= tc {
        plateau
    } else if period <= tl {
        plateau * math::powf(tc / period, SPECTRUM_EXPONENT)
    } else {
        plateau * math::powf(tc / tl, SPECTRUM_EXPONENT) * math::powi(tl / period, 2)
    };
    elastic_g / RESPONSE_MODIFICATION * GRAVITY_M_S2
}

/// T0 = 0.10 Fs Fd / Fa, en s.
#[must_use]
pub fn corner_t0() -> f64 {
    0.10 * FS * FD / FA
}

/// Tc = 0.55 Fs Fd / Fa, en s.
#[must_use]
pub fn corner_tc() -> f64 {
    0.55 * FS * FD / FA
}

/// TL = 2.4 Fd, en s.
#[must_use]
pub fn corner_tl() -> f64 {
    2.4 * FD
}

/// Periodos de la tabla: paso fijo, más las esquinas T0, Tc y TL.
#[must_use]
pub fn spectrum_periods() -> Vec<f64> {
    let steps = (PERIOD_END_S / PERIOD_STEP_S).round() as i32;
    let mut periods: Vec<f64> = (0..=steps).map(|i| f64::from(i) * PERIOD_STEP_S).collect();
    periods.push(corner_t0());
    periods.push(corner_tc());
    periods.push(corner_tl());
    periods.sort_by(f64::total_cmp);
    periods.dedup();
    periods
}

fn build(id: &str, stories: usize) -> BenchModel {
    let mass = story_mass_kg();
    let (master_ix, master_iy) = master_line();
    let master_xy = [master_ix as f64 * SPAN_M, master_iy as f64 * SPAN_M];
    let centroid = [PLAN_M / 2.0, PLAN_M / 2.0];
    let izz = izz_about_master(mass, master_xy, centroid);
    let young = concrete_young_pa(FC_MPA);
    let shear = shear_modulus_pa(young, POISSON);
    let (column_a, column_iy, column_iz, column_j) =
        rectangle_section(COLUMN_SIDE_M, COLUMN_SIDE_M);
    let (beam_a, beam_iy, beam_iz, beam_j) = rectangle_section(BEAM_WIDTH_Y_M, BEAM_DEPTH_Z_M);
    let grid = grid();
    let nodes = nodes(stories);
    let diaphragms = diaphragms(stories, &nodes, master_ix, master_iy);
    let restraints = restraints(&nodes);
    let load_patterns = load_patterns(stories, &diaphragms, mass);
    BenchModel {
        schema: SCHEMA.to_owned(),
        id: id.to_owned(),
        units: "SI".to_owned(),
        source: source(),
        stories: (1..=stories)
            .map(|level| Story {
                level: level as u32,
                h: STORY_HEIGHT_M,
                mass: StoryMass { m: mass, izz },
            })
            .collect(),
        grid,
        nodes,
        members: members(stories),
        sections: vec![
            section(
                "column", young, shear, column_a, column_iy, column_iz, column_j,
            ),
            section("beam", young, shear, beam_a, beam_iy, beam_iz, beam_j),
        ],
        diaphragms,
        restraints,
        load_patterns,
        modal: Modal {
            n: MODE_COUNT,
            mass_kind: "lumped".to_owned(),
            directions: vec![Axis::X.vector(), Axis::Y.vector(), Axis::Z.vector()],
        },
        spectrum: spectrum(),
        combination: Combination {
            method: "cqc".to_owned(),
            zeta: ZETA,
        },
        nodal_masses: None,
        p_delta: None,
        static_combinations: Vec::new(),
    }
}

fn source() -> Source {
    Source {
        geometry: format!(
            "Pórtico espacial de hormigón, {BAYS}×{BAYS} vanos de {SPAN_M} m, \
             altura de piso {STORY_HEIGHT_M} m, {NODES_PER_LEVEL} nudos por nivel incluida la base \
             y diafragma rígido en cada piso. ENG-VAL-006. Las secciones son las mismas en los tres tamaños."
        ),
        material: format!(
            "Hormigón f'c = {FC_MPA} MPa. E = 4700·sqrt(f'c) MPa (ACI 318-19 §19.2.2.1.a). \
             ν = {POISSON}. G = E / (2 (1 + ν)). Densidad {CONCRETE_DENSITY_KG_M3} kg/m³."
        ),
        torsion: "J de Saint-Venant del rectángulo: a·b³·(1/3 − 0.21·(b/a)·(1 − (b/a)⁴/12)), con a el lado mayor.".to_owned(),
        mass: format!(
            "m = área·(espesor·densidad + sobrecarga/g), área {PLAN_M} m × {PLAN_M} m, \
             espesor {SLAB_THICKNESS_M} m, sobrecarga {SUPERIMPOSED_N_M2} N/m², g = {GRAVITY_M_S2} m/s². \
             Izz es la inercia de esa masa uniforme respecto del eje global Z que pasa por el nudo maestro: \
             m·(Lx²+Ly²)/12 + m·d². Unidades: m en kg, Izz en kg·m²."
        ),
        spectrum: format!(
            "NEC-SE-DS 2015, zona {ZONE_LABEL} (Z = {ZONE_FACTOR}), suelo {SOIL}, η = {ETA}, \
             Fa = {FA}, Fd = {FD}, Fs = {FS}, R = {RESPONSE_MODIFICATION}, ζ = {ZETA}. \
             Sa está en m/s²: (ordenada elástica en g) / R · g. No está en g. \
             El motor recibe la tabla; no interpreta la norma."
        ),
    }
}

fn grid() -> Grid {
    let axis: Vec<f64> = (0..LINES).map(|i| i as f64 * SPAN_M).collect();
    Grid {
        x: axis.clone(),
        y: axis,
    }
}

fn node_id(story: usize, ix: usize, iy: usize) -> u32 {
    (story * NODES_PER_LEVEL + iy * LINES + ix + 1) as u32
}

fn master_line() -> (usize, usize) {
    // Distancia al centroide por math::hypot. En empate de bits gana (iy, ix) menor.
    let centroid = PLAN_M / 2.0;
    let mut best = (0usize, 0usize);
    let mut best_distance = f64::INFINITY;
    for iy in 0..LINES {
        for ix in 0..LINES {
            let dx = ix as f64 * SPAN_M - centroid;
            let dy = iy as f64 * SPAN_M - centroid;
            let distance = math::hypot(dx, dy);
            if distance < best_distance
                || (distance.to_bits() == best_distance.to_bits() && (iy, ix) < (best.1, best.0))
            {
                best_distance = distance;
                best = (ix, iy);
            }
        }
    }
    best
}

fn nodes(stories: usize) -> Vec<Node> {
    let mut nodes = Vec::new();
    for story in 0..=stories {
        let z = story as f64 * STORY_HEIGHT_M;
        for iy in 0..LINES {
            for ix in 0..LINES {
                nodes.push(Node {
                    id: node_id(story, ix, iy),
                    story: story as u32,
                    ix: ix as u32,
                    iy: iy as u32,
                    coordinates: [ix as f64 * SPAN_M, iy as f64 * SPAN_M, z],
                });
            }
        }
    }
    nodes
}

fn members(stories: usize) -> Vec<Member> {
    let mut members = Vec::new();
    let mut id = 1_u32;
    for story in 0..stories {
        for iy in 0..LINES {
            for ix in 0..LINES {
                members.push(member(
                    id,
                    MemberKind::Column,
                    [node_id(story, ix, iy), node_id(story + 1, ix, iy)],
                    "column",
                    [1.0, 0.0, 0.0],
                ));
                id += 1;
            }
        }
    }
    for story in 1..=stories {
        for iy in 0..LINES {
            for ix in 0..BAYS {
                members.push(member(
                    id,
                    MemberKind::Beam,
                    [node_id(story, ix, iy), node_id(story, ix + 1, iy)],
                    "beam",
                    [0.0, 0.0, 1.0],
                ));
                id += 1;
            }
        }
        for ix in 0..LINES {
            for iy in 0..BAYS {
                members.push(member(
                    id,
                    MemberKind::Beam,
                    [node_id(story, ix, iy), node_id(story, ix, iy + 1)],
                    "beam",
                    [0.0, 0.0, 1.0],
                ));
                id += 1;
            }
        }
    }
    members
}

fn member(id: u32, kind: MemberKind, nodes: [u32; 2], section: &str, vector: [f64; 3]) -> Member {
    Member {
        id,
        kind,
        nodes,
        section: section.to_owned(),
        orientation: Some(Orientation {
            kind: ORIENTATION_KIND.to_owned(),
            vector,
        }),
    }
}

fn diaphragms(
    stories: usize,
    nodes: &[Node],
    master_ix: usize,
    master_iy: usize,
) -> Vec<Diaphragm> {
    (1..=stories)
        .map(|story| {
            let master = node_id(story, master_ix, master_iy);
            let slaves = nodes
                .iter()
                .filter(|node| node.story == story as u32 && node.id != master)
                .map(|node| node.id)
                .collect();
            Diaphragm {
                level: story as u32,
                z: story as f64 * STORY_HEIGHT_M,
                master,
                slaves,
            }
        })
        .collect()
}

fn restraints(nodes: &[Node]) -> Vec<Restraint> {
    nodes
        .iter()
        .filter(|node| node.story == 0)
        .map(|node| Restraint {
            node: node.id,
            dofs: vec![Dof::Ux, Dof::Uy, Dof::Uz, Dof::Rx, Dof::Ry, Dof::Rz],
        })
        .collect()
}

fn load_patterns(stories: usize, diaphragms: &[Diaphragm], mass: f64) -> LoadPatterns {
    let share = mass / NODES_PER_LEVEL as f64;
    let mut gravity = Vec::new();
    for story in 1..=stories {
        for slot in 0..NODES_PER_LEVEL {
            let ix = slot % LINES;
            let iy = slot / LINES;
            gravity.push(NodalAction {
                node: node_id(story, ix, iy),
                values: vec![DofValue {
                    dof: Dof::Uz,
                    value: -share * GRAVITY_M_S2,
                }],
            });
        }
    }
    let lateral = diaphragms
        .iter()
        .map(|diaphragm| NodalAction {
            node: diaphragm.master,
            values: vec![DofValue {
                dof: Dof::Ux,
                value: LATERAL_WEIGHT_FRACTION * mass * GRAVITY_M_S2,
            }],
        })
        .collect();
    LoadPatterns {
        gravity,
        lateral,
        patterns: Vec::new(),
    }
}

fn section(id: &str, young: f64, shear: f64, area: f64, iy: f64, iz: f64, torsion: f64) -> Section {
    Section {
        id: id.to_owned(),
        young,
        shear: Some(shear),
        area,
        iy: Some(iy),
        iz: Some(iz),
        j: Some(torsion),
    }
}

fn spectrum() -> Spectrum {
    let pairs = spectrum_periods()
        .into_iter()
        .map(|period| [period, spectral_acceleration_m_s2(period)])
        .collect();
    Spectrum {
        zone: ZONE_LABEL.to_owned(),
        zone_factor: ZONE_FACTOR,
        soil: SOIL.to_owned(),
        response_modification: RESPONSE_MODIFICATION,
        eta: ETA,
        fa: FA,
        fd: FD,
        fs: FS,
        zeta: ZETA,
        g: GRAVITY_M_S2,
        r: SPECTRUM_EXPONENT,
        direction: Some(SpectrumDirection {
            axis: Axis::X,
            vector: Axis::X.vector(),
        }),
        directions: Vec::new(),
        directional_combination: DirectionalCombination {
            method: "weightedSum".to_owned(),
            factors: vec![1.0],
        },
        variants: Vec::new(),
        interpolation: "linear".to_owned(),
        extrapolation: "error".to_owned(),
        sa: SaTable {
            unit: SA_UNIT.to_owned(),
            pairs,
        },
    }
}

/// El maestro de la retícula oficial es la línea interior de menor índice.
pub fn official_master_line() -> Result<(usize, usize), BenchError> {
    let master = master_line();
    if master != (1, 1) {
        return Err(BenchError::new(
            "La retícula oficial dejó de elegir el nudo (6 m, 6 m) como maestro.",
        ));
    }
    Ok(master)
}
