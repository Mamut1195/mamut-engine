//! Modelos `bench-model/1` y su traducción a `engine-input/1`.
//!
//! El límite de la entrada traducida es el de `engine-input/1`: 256 nudos y 1024 barras.
//! Los tres pórticos oficiales quedan dentro de ese límite; el de 15 pisos tiene 256 nudos.
//! No hay azar ni calibres ajustados a una salida numérica.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

mod error;
mod generate;
mod index;
mod model;
mod translate;

pub use error::BenchError;
pub use generate::{
    concrete_young_pa, corner_t0, corner_tc, corner_tl, izz_about_master, mesh_counts,
    official_master_line, official_models, portal_250, portal_250_counts, portal_250_story_mass_kg,
    rectangle_section, saint_venant_j, shear_modulus_pa, spectral_acceleration_m_s2,
    spectrum_periods, story_mass_kg, BAYS, BEAM_DEPTH_Z_M, BEAM_WIDTH_Y_M, COLUMN_SIDE_M,
    CONCRETE_DENSITY_KG_M3, FC_MPA, GRAVITY_M_S2, LATERAL_WEIGHT_FRACTION, LINES, MODE_COUNT,
    NODES_PER_LEVEL, PLAN_M, POISSON, PORTAL_250_ID, PORTAL_250_LINES,
    PORTAL_250_PDELTA_DISPLACEMENT_M, PORTAL_250_PDELTA_ITERATIONS, PORTAL_250_PDELTA_ROTATION_RAD,
    PORTAL_250_PDELTA_TOLERANCE, PORTAL_250_STORIES, SPAN_M, STORY_HEIGHT_M,
};
pub use index::{
    file_sha256, repository_index, roof_node, write_fixtures, BenchIndex, BENCH_MODEL_SCHEMA_PATH,
    FIXTURE_ROOT, INDEX_PATH,
};
pub use model::{AnalysisKind, Axis, BenchModel, Dof, MemberKind, SA_UNIT, SCHEMA};
pub use translate::{translate, translate_variant};

/// Genera fixtures o traduce un modelo ya escrito.
pub fn cli(args: &[String]) -> Result<(), BenchError> {
    index::cli(args)
}
