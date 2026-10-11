//! Fixtures analíticos propios, embebidos: no se necesita E/S en wasm.
pub struct Fixture {
    pub source: &'static str,
    pub citation: &'static str,
    pub stiffnesses: &'static [f64],
    pub expected_k: &'static [f64],
    pub expected_u: &'static [f64],
}

// Derivación: energía V = Σ k_i (u_{i+1}−u_i)²/2. Su Hessiano suma
// [[k_i,−k_i],[−k_i,k_i]] en los dos índices conectados. Con u_1=0 y
// fuerza terminal P=12 N, cada resorte lleva 12 N y Δu_i=12/k_i m.
pub const TWO_SPRINGS: Fixture = Fixture {
    source: "analítico: derivación propia por energía y equilibrio",
    citation:
        "provenance/ENG-ASM-001.md, Derivaciones propias; cita bibliográfica precisa PENDIENTE",
    stiffnesses: &[100.0, 200.0],
    expected_k: &[
        100.0, -100.0, 0.0, -100.0, 300.0, -200.0, 0.0, -200.0, 200.0,
    ],
    expected_u: &[0.12, 0.18],
};
pub const THREE_SPRINGS: Fixture = Fixture {
    source: "analítico: derivación propia por energía y equilibrio",
    citation:
        "provenance/ENG-ASM-001.md, Derivaciones propias; cita bibliográfica precisa PENDIENTE",
    stiffnesses: &[100.0, 200.0, 300.0],
    expected_k: &[
        100.0, -100.0, 0.0, 0.0, -100.0, 300.0, -200.0, 0.0, 0.0, -200.0, 500.0, -300.0, 0.0, 0.0,
        -300.0, 300.0,
    ],
    expected_u: &[0.12, 0.18, 0.22],
};
// Barra: E=200 Pa, A=2 m², L_1=2 m, L_2=4 m => EA/L=200 y 100 N/m.
// De ε=Δu/L, σ=Eε y N=Aσ se obtiene la misma energía cuadrática.
// P=12 N => u_2=12/200=0.06 m, u_3=0.06+12/100=0.18 m.
pub const AXIAL_BAR: Fixture = Fixture {
    source: "analítico: derivación propia N=EA Δu/L para dos segmentos axiales",
    citation:
        "provenance/ENG-ASM-001.md, Derivaciones propias; cita bibliográfica precisa PENDIENTE",
    stiffnesses: &[200.0, 100.0],
    expected_k: &[
        200.0, -200.0, 0.0, -200.0, 300.0, -100.0, 0.0, -100.0, 100.0,
    ],
    expected_u: &[0.06, 0.18],
};

pub const SETTLEMENT_SOURCE: &str =
    "analítico: equilibrio 100u_2+200(u_2−0.03)=0; u_2=0.02 m, R_1=−2 N, R_3=2 N";
pub const SETTLEMENT_CITATION: &str =
    "provenance/ENG-ASM-001.md, Derivaciones propias; cita bibliográfica precisa PENDIENTE";
