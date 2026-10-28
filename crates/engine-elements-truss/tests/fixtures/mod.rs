//! Own closed-form solutions from equilibrium and strength of materials, SI.
pub const SOURCE: &str = "analítico: derivación propia por equilibrio, energía y σ=Eε";
pub const CITATION: &str =
    "provenance/ENG-ELM-001.md, Derivaciones propias; cita bibliográfica exacta PENDIENTE";

pub struct Fixture {
    pub source: &'static str,
    pub citation: &'static str,
    pub coords: &'static [[f64; 3]],
    pub members: &'static [[usize; 2]],
    pub force: [f64; 3],
}

// A=(0,0), B=(2,0), C=(1,1). A pin, B restrains Uy.
// ΣM_A=2R_By−12−6=0 => R_By=9; R_A=(-6,3).
// Equilibrium at C: N_AC=-3√2, N_BC=-9√2; equilibrium at B: N_AB=9 N.
pub const TRIANGLE: Fixture = Fixture {
    source: SOURCE,
    citation: CITATION,
    coords: &[[0.0, 0.0, 0.0], [2.0, 0.0, 0.0], [1.0, 1.0, 0.0]],
    members: &[[0, 1], [0, 2], [1, 2]],
    force: [6.0, -12.0, 0.0],
};

// Restrained bases (1,0,0), (0,1,0), (-1,-1,0), apex (0,0,1).
// At the apex: a-c+6=0, b-c-3=0, -a-b-c-12=0, with
// a=N_1/√2, b=N_2/√2, c=N_3/√3 => a=-9,b=0,c=-3 N.
// R_1=(-9,0,9), R_2=(0,0,0), R_3=(3,3,3) N.
pub const TRIPOD: Fixture = Fixture {
    source: SOURCE,
    citation: CITATION,
    coords: &[
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [-1.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
    ],
    members: &[[0, 3], [1, 3], [2, 3]],
    force: [6.0, -3.0, -12.0],
};

// E=200 Pa, A=2 m², L=2 m, rho=3 kg/m³: k=200 N/m, m=12 kg.
// P=12 N => δ=PL/EA=0.06 m; N=12 N, axial reaction -12 N.
pub const AXIAL: Fixture = Fixture {
    source: SOURCE,
    citation: CITATION,
    coords: &[[0.0, 0.0, 0.0], [2.0, 0.0, 0.0]],
    members: &[[0, 1]],
    force: [12.0, 0.0, 0.0],
};

pub fn cited(f: &Fixture) {
    assert!(!f.source.is_empty() && !f.citation.is_empty());
}
