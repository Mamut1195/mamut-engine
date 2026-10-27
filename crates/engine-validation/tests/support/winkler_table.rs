//! Independent SI fixtures shared by native tests and the wasm runner.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Kind {
    Infinite,
    SemiInfinite,
    Rigid,
}
#[derive(Clone, Copy)]
pub struct Fixture {
    pub name: &'static str,
    pub source: &'static str,
    pub citation: &'static str,
    pub spatial: bool,
    pub kind: Kind,
    pub ei: [f64; 2], // EIz, EIy: bending in local y, z.
    pub k: [f64; 2],
    pub p: [f64; 2],
    pub meshes: [usize; 3],
    pub bound: f64,
    pub justification: &'static str,
}
const fn row(name: &'static str, spatial: bool, kind: Kind) -> Fixture {
    let rigid = matches!(kind, Kind::Rigid);
    let semi = matches!(kind, Kind::SemiInfinite);
    Fixture {
        name, spatial, kind,
        source: "Derivación propia: EI v''''+k v=Pδ, continuidad/salto y extremos libres; sin oráculo externo.",
        citation: if rigid {
            "Derivación propia: [límite de viga corta rígida y cota energética](provenance/ENG-GEO-002.md#viga-corta-rígida)."
        } else {
            "Derivación propia: [viga infinita/semiinfinita, salto y condiciones de extremo](provenance/ENG-GEO-002.md#viga-infinita-y-semiinfinita)."
        },
        ei: if rigid { [1e5,2e5] } else { [2.0,5.0] },
        k: if rigid { [3.0,6.0] } else { [8.0,20.0] },
        p: [12.0,-7.0],
        meshes: if rigid { [2,4,8] } else if semi { [12,24,48] } else { [16,32,64] },
        bound: if rigid { 2e-5 } else { 2e-4 },
        justification: if rigid {
            "ε=kL⁴/EI=4.8e-4; cota energética relativa ε/320 y margen solver <2e-5."
        } else {
            "λL=20 (infinita) o 12 (semi); reflexión <1e-6. λh≤.3125 al final, error Hermite O(h⁴); cota de regresión relativa 2e-4 con refinamiento decreciente."
        },
    }
}
pub const FIXTURES: &[Fixture] = &[
    row("Hetenyi-centro-2d", false, Kind::Infinite),
    row("Hetenyi-extremo-2d", false, Kind::SemiInfinite),
    row("rigida-centro-2d", false, Kind::Rigid),
    row("Hetenyi-centro-3d-dos-planos", true, Kind::Infinite),
    row("Hetenyi-extremo-3d-dos-planos", true, Kind::SemiInfinite),
    row("rigida-centro-3d-dos-planos", true, Kind::Rigid),
];
