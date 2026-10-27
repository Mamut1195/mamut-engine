//! Analytical SI fixtures; figures derived in provenance/ENG-GEO-003.md.
#[derive(Clone, Copy)]
pub struct Source {
    pub kind: &'static str,
    pub citation: &'static str,
    pub derivation: &'static str,
}
#[derive(Clone, Copy)]
pub struct Fixture {
    pub name: &'static str,
    pub source: Source,
    pub depth_m: f64,
    pub checks: &'static [f64],
}
pub const ABSOLUTE: f64 = 1e-8;
pub const RELATIVE: f64 = 1e-12;
pub const JUSTIFICATION: &str = "Redondeo binary64 de radicales y bisección fija; sin error de malla en los vértices analíticos. A=1e-8 en la unidad SI del control, R=1e-12.";
pub const FIXTURES: &[Fixture] = &[
    Fixture {
        name: "py-matlock-estatico", depth_m: 2.0,
        source: Source {
            kind: "analytical",
            citation: "Matlock, H. (1970), OTC 1204, Ec. 1–5 y Fig. 5a (estática).",
            derivation: "provenance/ENG-GEO-003.md#matlock-derivación-analítica",
        },
        // Np, pu [N/m], yc [m], p(yc/8), p(yc), p(8yc), xr [m].
        checks: &[5.0,100000.0,0.025,25000.0,50000.0,100000.0,6.0],
    },
    Fixture {
        name: "py-reese-superficial", depth_m: 2.0,
        source: Source {
            kind: "analytical",
            citation: "Reese, L. C., Cox, W. R., Koop, F. D. (1974), OTC 2080, Ec. 4–9, 11–16, pasos 2 y 4–8.",
            derivation: "provenance/ENG-GEO-003.md#reese-derivación-analítica",
        },
        // pct, pcd [N/m], Xt [m], yk, ym, yu [m], pk, pm, pu [N/m].
        checks: &[187189.73979388434,574902.5527149864,7.792966107085287,
            0.000016276041666666666,0.016666666666666666,0.0375,
            365.60496053493034,93594.86989694217,187189.73979388434],
    },
    Fixture {
        name: "py-reese-profundo", depth_m: 10.0,
        source: Source {
            kind: "analytical",
            citation: "Reese, L. C., Cox, W. R., Koop, F. D. (1974), OTC 2080, Ec. 5, 8, 9; paso 4.",
            derivation: "provenance/ENG-GEO-003.md#reese-derivación-analítica",
        },
        // pct, pcd, and pu [N/m].
        checks: &[3613076.828180442,2874512.763574932,2874512.763574932],
    },
];
