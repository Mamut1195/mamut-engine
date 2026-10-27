//! Independently derived SI data. Each row is a complete problem with an independent oracle.
#[derive(Clone, Copy)]
pub enum Parameters {
    Shear {
        n: usize,
        k: f64,
        m: f64,
    },
    Variable {
        stiffness: [f64; 2],
        masses: [f64; 2],
    },
    Chain {
        n: usize,
        k: f64,
        m: f64,
    },
    Beam {
        mesh: usize,
        cantilever: bool,
        angle: f64,
    },
    Column {
        mesh: usize,
        ratio: f64,
    },
    Portal {
        mesh: usize,
        ratio: f64,
        rigid: f64,
    },
    Iteration {
        k: f64,
        ratio: f64,
        force: f64,
        tolerance: f64,
        limit: usize,
    },
    Spectrum {
        k: f64,
        m: f64,
        sa: f64,
        damping: f64,
    },
    Interpolation {
        log: bool,
        points: [(f64, f64); 2],
        query: f64,
        expected: f64,
    },
    Cqc {
        frequencies: [f64; 2],
        damping: [f64; 2],
        responses: [f64; 2],
    },
    Directional {
        mass: f64,
        stiffness: [f64; 2],
        srss: bool,
        factors: [f64; 2],
        accelerations: [f64; 2],
    },
}
#[derive(Clone, Copy)]
pub struct FrameData {
    pub e: f64,
    pub area: f64,
    pub inertia: f64,
    pub density: f64,
    pub length: f64,
    pub width: f64,
    pub force: f64,
}
const BEAM_DATA: FrameData = FrameData {
    e: 1.0,
    area: 1.0,
    inertia: 1e-4,
    density: 1.0,
    length: 1.0,
    width: 0.0,
    force: 0.0,
};
const PDELTA_DATA: FrameData = FrameData {
    e: 200.0,
    area: 2.0,
    inertia: 3.0,
    density: 0.0,
    length: 4.0,
    width: 4.0,
    force: 0.1,
};
#[derive(Clone, Copy)]
pub struct Tolerance {
    pub absolute: f64,
    pub relative: f64,
    pub justification: &'static str,
}
pub struct Fixture {
    pub name: &'static str,
    pub source: &'static str,
    pub citation: &'static str,
    pub parameters: Parameters,
    pub frame: FrameData,
    pub tolerance: Tolerance,
}
const EXACT: Tolerance = Tolerance {
    absolute: 1e-9,
    relative: 1e-9,
    justification: "Álgebra de modelos pequeños; margen f64, sin error de discretización.",
};
const fn row(name: &'static str, parameters: Parameters) -> Fixture {
    let tolerance = match parameters {
        Parameters::Beam { mesh: 4, .. } => Tolerance {
            absolute: 1e-9, relative: 0.019,
            justification: "Hermite consistente, 4 elementos: margen 1.9% para los tres primeros modos; refinamiento a 12 elementos.",
        },
        Parameters::Beam { .. } => Tolerance {
            absolute: 1e-9, relative: 3e-4,
            justification: "Hermite consistente, 12 elementos: margen 0.03% para los tres primeros modos; error disminuye más de 10 veces.",
        },
        Parameters::Column { .. } => Tolerance {
            absolute: 1e-9, relative: 0.016,
            justification: "1/(1-r): error de modelo de polo ≤1.6% para estos r; contraste continuo exacto separado a 2e-6 con malla 16.",
        },
        Parameters::Portal { .. } => Tolerance {
            absolute: 1e-9, relative: 0.026,
            justification: "Piso Pcr=12nEI/h²: aproximación de cuerda, error ≤2.6% para r≤0.1; contraste continuo separado a 1e-4 con 12 elementos y dintel/EA×1e5.",
        },
        _ => EXACT,
    };
    Fixture {
        name,
        source: match parameters {
            Parameters::Shear { .. } | Parameters::Chain { .. } => "Derivación propia D1/D3: ωj=2√(k/m)sin((2j−1)π/(4n+2)); modos normalizados a masa y ΣM*=Σm.",
            Parameters::Variable { .. } => "Derivación propia D2/D3: det(K−λM)=0, ecuación cuadrática; φᵀMφ=1, Γ=φᵀMr, M*=Γ².",
            Parameters::Beam { .. } => "Derivación propia D4: ωj=βj²√(EI/ρA)/L²; βj=jπ (apoyada) o cosβ coshβ=−1 (voladizo), raíces por bisección.",
            Parameters::Column { .. } => "Derivación propia D5/D6: Pcr=π²EI/(4L²); flecha continua de EI v''''+Pv''=0; amplificación aproximada 1/(1−P/Pcr). P>Pcr implica inestabilidad.",
            Parameters::Portal { .. } => "Derivación propia D7: deriva continua con giros impedidos; z=√(12r), Δ/Δ0=12(2tan(z/2)−z)/z³; aproximación 1/(1−r).",
            Parameters::Iteration { .. } => "Derivación propia D8: ui=F(1−r^i)/(k(1−r)), residual=r^i; suma geométrica y límite de iteraciones.",
            Parameters::Spectrum { .. } => "Derivación propia D2/D9: M*±=m(1±2/√5), q=ΓSa/ω², Vb=√(Σ(M*Sa)²).",
            Parameters::Interpolation { .. } => "Derivación propia D10: interpolación afín Sa(2.5)=6; log-log Sa=3T², Sa(2)=12.",
            Parameters::Cqc { .. } => "Evaluación algebraica propia D11 de la fórmula CQC suministrada; R=√(A²+B²+2ρAB). Sin tablas publicadas.",
            Parameters::Directional { .. } => "Derivación propia D12: Vx=mSa,x, Vy=mSa,y; suma ponderada o raíz de suma de cuadrados, ux=Sa,x/ωx².",
        },
        citation: "provenance/ENG-VAL-001.md#ampliación-dinámica-y-2º-orden",
        parameters,
        frame: match parameters { Parameters::Beam { .. } => BEAM_DATA, _ => PDELTA_DATA },
        tolerance,
    }
}
pub const MODAL: &[Fixture] = &[
    row(
        "modal-cortante-1",
        Parameters::Shear {
            n: 1,
            k: 23.0,
            m: 4.0,
        },
    ),
    row(
        "modal-cortante-2",
        Parameters::Shear {
            n: 2,
            k: 23.0,
            m: 4.0,
        },
    ),
    row(
        "modal-cortante-3",
        Parameters::Shear {
            n: 3,
            k: 23.0,
            m: 4.0,
        },
    ),
    row(
        "modal-cortante-4",
        Parameters::Shear {
            n: 4,
            k: 23.0,
            m: 4.0,
        },
    ),
    row(
        "modal-cortante-5",
        Parameters::Shear {
            n: 5,
            k: 23.0,
            m: 4.0,
        },
    ),
    row(
        "modal-variable-2x2",
        Parameters::Variable {
            stiffness: [19.0, 7.0],
            masses: [2.0, 5.0],
        },
    ),
    row(
        "modal-cadena-libre-fija",
        Parameters::Chain {
            n: 3,
            k: 31.0,
            m: 6.0,
        },
    ),
    row(
        "modal-apoyada-malla4",
        Parameters::Beam {
            mesh: 4,
            cantilever: false,
            angle: 0.0,
        },
    ),
    row(
        "modal-apoyada-malla12",
        Parameters::Beam {
            mesh: 12,
            cantilever: false,
            angle: 0.0,
        },
    ),
    row(
        "modal-voladizo-malla4",
        Parameters::Beam {
            mesh: 4,
            cantilever: true,
            angle: 0.0,
        },
    ),
    row(
        "modal-voladizo-malla12",
        Parameters::Beam {
            mesh: 12,
            cantilever: true,
            angle: 0.0,
        },
    ),
    row(
        "modal-voladizo-rotado",
        Parameters::Beam {
            mesh: 12,
            cantilever: true,
            angle: 0.83,
        },
    ),
];
pub const PDELTA: &[Fixture] = &[
    row(
        "pdelta-cero",
        Parameters::Column {
            mesh: 16,
            ratio: 0.0,
        },
    ),
    row(
        "pdelta-compresion-020",
        Parameters::Column {
            mesh: 16,
            ratio: 0.2,
        },
    ),
    row(
        "pdelta-compresion-050",
        Parameters::Column {
            mesh: 16,
            ratio: 0.5,
        },
    ),
    row(
        "pdelta-compresion-080",
        Parameters::Column {
            mesh: 16,
            ratio: 0.8,
        },
    ),
    row(
        "pdelta-compresion-090",
        Parameters::Column {
            mesh: 16,
            ratio: 0.9,
        },
    ),
    row(
        "pdelta-traccion",
        Parameters::Column {
            mesh: 16,
            ratio: -0.5,
        },
    ),
    row(
        "pdelta-inestable",
        Parameters::Column {
            mesh: 16,
            ratio: 1.05,
        },
    ),
    row(
        "pdelta-piso-005",
        Parameters::Portal {
            mesh: 12,
            ratio: 0.05,
            rigid: 100_000.0,
        },
    ),
    row(
        "pdelta-piso-010",
        Parameters::Portal {
            mesh: 12,
            ratio: 0.1,
            rigid: 100_000.0,
        },
    ),
    row(
        "pdelta-iteracion-geometrica",
        Parameters::Iteration {
            k: 80.0,
            ratio: 0.3,
            force: 8.0,
            tolerance: 1e-7,
            limit: 32,
        },
    ),
];
pub const SPECTRUM: &[Fixture] = &[
    row(
        "espectro-srss-2pisos",
        Parameters::Spectrum {
            k: 23.0,
            m: 4.0,
            sa: 2.7,
            damping: 0.04,
        },
    ),
    row(
        "espectro-desplazamiento-modal",
        Parameters::Spectrum {
            k: 23.0,
            m: 4.0,
            sa: 5.1,
            damping: 0.08,
        },
    ),
    row(
        "espectro-interpolacion-lineal",
        Parameters::Interpolation {
            log: false,
            points: [(1.0, 3.0), (5.0, 11.0)],
            query: 2.5,
            expected: 6.0,
        },
    ),
    row(
        "espectro-interpolacion-loglog",
        Parameters::Interpolation {
            log: true,
            points: [(1.0, 3.0), (4.0, 48.0)],
            query: 2.0,
            expected: 12.0,
        },
    ),
    row(
        "espectro-cqc-separados",
        Parameters::Cqc {
            frequencies: [1.0, 100.0],
            damping: [0.05, 0.05],
            responses: [3.0, -4.0],
        },
    ),
    row(
        "espectro-cqc-cercanos",
        Parameters::Cqc {
            frequencies: [10.0, 10.2],
            damping: [0.05, 0.07],
            responses: [3.0, -4.0],
        },
    ),
    row(
        "espectro-cqc-diagonal",
        Parameters::Cqc {
            frequencies: [7.0, 7.0],
            damping: [0.04, 0.04],
            responses: [3.0, 4.0],
        },
    ),
    row(
        "espectro-direccional-ponderado",
        Parameters::Directional {
            mass: 2.0,
            stiffness: [18.0, 50.0],
            srss: false,
            factors: [1.4, 0.35],
            accelerations: [2.0, 5.0],
        },
    ),
    row(
        "espectro-direccional-srss",
        Parameters::Directional {
            mass: 2.0,
            stiffness: [18.0, 50.0],
            srss: true,
            factors: [0.65, 1.2],
            accelerations: [3.0, 4.0],
        },
    ),
];
