//! Composición Euler–Bernoulli + subrasante consistente Hermite.
use crate::{finite, geometry, invalid, EulerFrame2D, Matrix};
use engine_contracts::{Element, ElementCtx, ElementOutput, ElementResponse, MassKind, MemberLoad};
use engine_core::{ContractId, Dof, EResult, Real};
use engine_geo::winkler;
use engine_linalg::DMat;

/// Viga `frame2d.winkler`, lecho bilateral lineal fijo a tierra, sin masa propia.
/// k en N/m² por longitud de viga: reacción local −k·desplazamiento (N/m).
/// Dirección transversal y local.
/// No admite liberaciones de extremo. Conserva masa, cargas y Kg del frame.
/// El lecho elimina los modos rígidos transversales; K del lecho completo
/// es semidefinida por sus ceros axiales.
/// Derivación propia: provenance/ENG-GEO-002.md (Winkler 1867; Hetényi 1946).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WinklerFrame2D {
    beam: EulerFrame2D,
    k: Real,
}
impl WinklerFrame2D {
    /// Añade lecho al frame sin liberaciones; ENG-E-0010 por liberaciones o k<0,
    /// ENG-E-0008 por k no finito. Las propiedades del frame ya están validadas.
    pub fn new(beam: EulerFrame2D, k: Real) -> EResult<Self> {
        winkler::validate(k)?;
        if beam.releases != crate::MomentReleases::default() {
            return Err(invalid("Winkler: liberaciones de extremo no admitidas."));
        }
        Ok(Self { beam, k })
    }
    /// Matriz del lecho en ejes locales y orden nodal del frame, sin EI.
    pub fn foundation_matrix(&self, ctx: &ElementCtx<'_>) -> EResult<DMat> {
        let g = geometry::Geometry::new(ctx)?;
        DMat::new(
            6,
            6,
            self.local_matrix(g.length)?.into_iter().flatten().collect(),
        )
    }
    fn local_matrix(&self, l: Real) -> EResult<Matrix> {
        let mut out = [[0.0; 6]; 6];
        for (ids, sign, k) in [([1, 2, 4, 5], 1.0, self.k)] {
            let block = winkler::matrix(k, l)?;
            let signs = [1.0, sign, 1.0, sign];
            for (i, &r) in ids.iter().enumerate() {
                for (j, &s) in ids.iter().enumerate() {
                    out[r][s] = signs[i] * signs[j] * block[i][j];
                }
            }
        }
        Ok(out)
    }
}
impl Element for WinklerFrame2D {
    fn kind(&self) -> &'static str {
        "frame2d.winkler"
    }
    fn contract(&self) -> ContractId {
        self.beam.contract()
    }
    fn n_nodes(&self) -> usize {
        self.beam.n_nodes()
    }
    fn dofs_per_node(&self) -> &'static [Dof] {
        self.beam.dofs_per_node()
    }
    fn respond(
        &self,
        ctx: &ElementCtx<'_>,
        u_e: &[Real],
        committed: &[Real],
    ) -> EResult<ElementResponse> {
        // Validación de historia, geometría y vector idéntica al frame base.
        let mut response = self.beam.respond(ctx, u_e, committed)?;
        let g = geometry::Geometry::new(ctx)?;
        let bed = self.local_matrix(g.length)?;
        let force = g.global_vector(&geometry::multiply(&bed, &g.local_vector(u_e)?)?)?;
        let global = g.global_matrix(&bed)?;
        let values = response
            .tangent
            .values()
            .iter()
            .zip(global.values())
            .map(|(a, b)| finite(a + b, "K Euler + Winkler"))
            .collect::<EResult<Vec<_>>>()?;
        response.tangent = DMat::new(6, 6, values)?;
        for (f, b) in response.internal_force.iter_mut().zip(force) {
            *f = finite(*f + b, "F Euler + Winkler")?;
        }
        Ok(response)
    }
    fn mass(&self, ctx: &ElementCtx<'_>, kind: MassKind) -> EResult<Option<DMat>> {
        self.beam.mass(ctx, kind)
    }
    fn geometric_stiffness(
        &self,
        ctx: &ElementCtx<'_>,
        u: &[Real],
        committed: &[Real],
    ) -> EResult<Option<DMat>> {
        self.beam.geometric_stiffness(ctx, u, committed)
    }
    fn load_vector(&self, ctx: &ElementCtx<'_>, load: &MemberLoad) -> EResult<Vec<Real>> {
        self.beam.load_vector(ctx, load)
    }
    fn recover(
        &self,
        ctx: &ElementCtx<'_>,
        u: &[Real],
        committed: &[Real],
        loads: &[MemberLoad],
        stations: &[Real],
    ) -> EResult<ElementOutput> {
        let mut out = self.beam.recover(ctx, u, committed, loads, stations)?;
        let g = geometry::Geometry::new(ctx)?;
        let u = g.local_vector(u)?;
        let bed = geometry::multiply(&self.local_matrix(g.length)?, &u)?;
        for (f, b) in out.end_forces_local.iter_mut().zip(bed) {
            *f = finite(*f + b, "Extremo Winkler")?;
        }
        for (ids, sign, k) in [([1, 2, 4, 5], 1.0, self.k)] {
            let d = [u[ids[0]], sign * u[ids[1]], u[ids[2]], sign * u[ids[3]]];
            let shear = if ids[0] == 1 { 1 } else { 2 };
            let moment = if ids[0] == 1 { 5 } else { 4 };
            for station in &mut out.stations {
                let x = station.position * g.length;
                let [force, torque] = winkler::integrals(k, g.length, d, station.position)?;
                station.forces_local[shear] = finite(
                    station.forces_local[shear] - bed[ids[0]] + force,
                    "Cortante Winkler",
                )?;
                station.forces_local[moment] = finite(
                    station.forces_local[moment] - bed[ids[1]] + sign * (bed[ids[0]] * x - torque),
                    "Momento Winkler",
                )?;
            }
        }
        Ok(out)
    }
}
