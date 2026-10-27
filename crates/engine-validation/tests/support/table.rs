//! SI data table; each row is a problem rather than an isolated check.
#[derive(Clone, Copy)]
pub enum Kind {
    CantileverPoint,
    CantileverUniform,
    CantileverMoment,
    SimpleUniform,
    SimplePoint,
    FixedUniform,
    FixedPoint,
    Propped,
    Continuous,
    TwoBars,
    Triangle,
    Concurrent,
    Settlement,
    PortalLateral,
    PortalVertical,
    Release,
    Combination,
}

#[derive(Clone, Copy)]
pub struct Parameters {
    pub e: f64,
    pub area: f64,
    pub inertia: f64,
    pub length: f64,
    pub height: f64,
    pub p: f64,
    pub w: f64,
    pub fraction: f64,
    pub horizontal: f64,
    pub settlement: f64,
    pub angle: f64,
    pub factors: [f64; 2],
    pub rigid_ratio: f64,
}
const DATA: Parameters = Parameters {
    e: 200.0,
    area: 2.0,
    inertia: 3.0,
    length: 4.0,
    height: 3.0,
    p: 12.0,
    w: 6.0,
    fraction: 0.5,
    horizontal: 0.0,
    settlement: 0.03,
    angle: 0.0,
    factors: [1.0, 1.0],
    rigid_ratio: 100_000.0,
};

#[derive(Clone, Copy)]
pub struct Tolerance {
    pub absolute: f64,
    pub relative: f64,
    pub justification: &'static str,
}
const EXACT: Tolerance = Tolerance {
    absolute: 1e-9,
    relative: 1e-10,
    justification: "Modelos pequeños, interpolación nodal exacta; margen para eliminación y recuperación f64. §tolerancias.",
};
const RIGID: Tolerance = Tolerance {
    absolute: 1e-8,
    relative: 5e-5,
    justification: "Límite rígido: error relativo de flexibilidad finita <2e-5 para r=1e5; margen absoluto 1e-8 por condicionamiento y recuperación f64. §portal-lateral.",
};

pub struct Fixture {
    pub name: &'static str,
    pub source: &'static str,
    pub citation: &'static str,
    pub kind: Kind,
    pub parameters: Parameters,
    pub tolerance: Tolerance,
}
const fn row(name: &'static str, kind: Kind, parameters: Parameters) -> Fixture {
    let (source, citation) = match kind {
        Kind::CantileverPoint => ("Derivación propia: integrar EI v''=-P(L-x) con v(0)=v'(0)=0.", "provenance/ENG-VAL-001.md#voladizos: δ=-PL³/(3EI), θ=-PL²/(2EI), Ry=P, Rz=PL"),
        Kind::CantileverUniform => ("Derivación propia: integrar EI v''=-w(L-x)²/2 con extremo fijo.", "provenance/ENG-VAL-001.md#voladizos: δ=-wL⁴/(8EI), θ=-wL³/(6EI)"),
        Kind::CantileverMoment => ("Derivación propia: curvatura constante EI v''=P (par terminal).", "provenance/ENG-VAL-001.md#voladizos: δ=PL²/(2EI), θ=PL/EI"),
        Kind::SimpleUniform => ("Derivación propia: integrar EI v''=wx(L-x)/2 con v(0)=v(L)=0.", "provenance/ENG-VAL-001.md#simplemente-apoyadas: δcentro=-5wL⁴/(384EI), θextremos=∓wL³/(24EI), Mmax=wL²/8"),
        Kind::SimplePoint => ("Derivación propia: integrar por tramos M=Pb x/L y M=Pa(L-x)/L, continuidad en a.", "provenance/ENG-VAL-001.md#simplemente-apoyadas: δ(a)=-Pa²b²/(3EIL), θA=-Pb(L²-b²)/(6EIL), θB=Pa(L²-a²)/(6EIL), Mmax=Pab/L"),
        Kind::FixedUniform => ("Derivación propia: imponer v=θ=0 en ambos extremos al polinomio de cuarto grado.", "provenance/ENG-VAL-001.md#biempotradas: Mext=-wL²/12, δcentro=-wL⁴/(384EI)"),
        Kind::FixedPoint => ("Derivación propia: integrar por tramos con cuatro condiciones de empotramiento y continuidad.", "provenance/ENG-VAL-001.md#biempotradas: MA=-Pab²/L², MB=-Pa²b/L², δ(a)=-Pa³b³/(3EIL³)"),
        Kind::Propped | Kind::Release => ("Derivación propia: compatibilidad v(L)=0, superponer carga uniforme y reacción terminal al voladizo.", "provenance/ENG-VAL-001.md#voladizo-apuntalado: RA=5wL/8, RB=3wL/8, M(L)=0"),
        Kind::Continuous => ("Derivación propia: simetría θcentral=0; cada vano equivale a voladizo apuntalado invertido.", "provenance/ENG-VAL-001.md#dos-vanos: Rext=3wL/8, Rcentral=5wL/4, Mcentral=-wL²/8"),
        Kind::TwoBars => ("Derivación propia: energía Σ EA(d·u)²/(2ℓ) y equilibrio del nudo libre.", "provenance/ENG-VAL-001.md#armaduras: ux=Hℓ³/(2EAa²), uy=-Pℓ³/(2EAh²), Nizq=Hℓ/(2a)-Pℓ/(2h)"),
        Kind::Triangle => ("Derivación propia: equilibrio de tres nudos y trabajo virtual de las tres barras.", "provenance/ENG-VAL-001.md#armaduras: fuerzas isostáticas y desplazamientos por Σ Ni ni ℓi/(EA)"),
        Kind::Concurrent => ("Derivación propia: añadir barra vertical concurrente; compatibilidad y suma de energías.", "provenance/ENG-VAL-001.md#armaduras: Ky=2EAh²/ℓ³+EA/h, Ni=(EA/ℓ)di·u"),
        Kind::Settlement => ("Derivación propia: integrar deformación axial constante y compatibilidad u(L)-u(0)=s.", "provenance/ENG-VAL-001.md#asentamiento: N=EA s/L, u(L/2)=s/2, R=∓N"),
        Kind::PortalLateral => ("Derivación propia: energía de dos columnas con giros y movimiento vertical impedidos en el límite rígido.", "provenance/ENG-VAL-001.md#portal-lateral: K=2·12EI/h³, Δ=Hh³/(24EI), ΣRx=-H"),
        Kind::PortalVertical => ("Derivación propia: simetría y estacionariedad de energía en traslación lateral t y giro θ opuestos.", "provenance/ENG-VAL-001.md#portal-vertical: v=-wbh/(2EA), t=-γθ, θ=-(wb²/12)/D, vmedio=v+bθ/4-wb⁴/(384EI)"),
        Kind::Combination => ("Derivación propia: sumar K ui=Fi y los operadores lineales de reacciones y recuperación.", "provenance/ENG-VAL-001.md#superposicion: δ=α(-PL³/3EI)+β(-wL⁴/8EI), R=αRP+βRw"),
    };
    Fixture {
        name,
        source: if parameters.angle != 0.0 {
            "Derivación propia: transformar u y R mediante Q(α), manteniendo θ y momentos; solución cerrada del voladizo."
        } else {
            source
        },
        citation: if parameters.angle != 0.0 {
            "provenance/ENG-VAL-001.md#rotacion: uglobal=Q(α)(0,-PL³/3EI), Rglobal=Q(α)(0,P), θ=-PL²/2EI"
        } else {
            citation
        },
        kind,
        parameters,
        tolerance: if matches!(kind, Kind::PortalLateral) {
            RIGID
        } else {
            EXACT
        },
    }
}

pub const FIXTURES: &[Fixture] = &[
    row("voladizo-punta-L4", Kind::CantileverPoint, DATA),
    row(
        "voladizo-punta-L7",
        Kind::CantileverPoint,
        Parameters {
            length: 7.0,
            ..DATA
        },
    ),
    row("voladizo-uniforme-L4", Kind::CantileverUniform, DATA),
    row(
        "voladizo-uniforme-L7",
        Kind::CantileverUniform,
        Parameters {
            length: 7.0,
            ..DATA
        },
    ),
    row("voladizo-par-terminal", Kind::CantileverMoment, DATA),
    row("simple-uniforme-L4", Kind::SimpleUniform, DATA),
    row(
        "simple-uniforme-L7",
        Kind::SimpleUniform,
        Parameters {
            length: 7.0,
            ..DATA
        },
    ),
    row("simple-punto-centro-L4", Kind::SimplePoint, DATA),
    row(
        "simple-punto-centro-L7",
        Kind::SimplePoint,
        Parameters {
            length: 7.0,
            ..DATA
        },
    ),
    row(
        "simple-punto-cuarto",
        Kind::SimplePoint,
        Parameters {
            fraction: 0.25,
            ..DATA
        },
    ),
    row(
        "simple-punto-40porciento",
        Kind::SimplePoint,
        Parameters {
            fraction: 0.4,
            ..DATA
        },
    ),
    row(
        "simple-punto-70porciento",
        Kind::SimplePoint,
        Parameters {
            fraction: 0.7,
            ..DATA
        },
    ),
    row("biempotrada-uniforme-L4", Kind::FixedUniform, DATA),
    row(
        "biempotrada-uniforme-L7",
        Kind::FixedUniform,
        Parameters {
            length: 7.0,
            ..DATA
        },
    ),
    row("biempotrada-punto-centro", Kind::FixedPoint, DATA),
    row(
        "biempotrada-punto-cuarto",
        Kind::FixedPoint,
        Parameters {
            fraction: 0.25,
            ..DATA
        },
    ),
    row(
        "biempotrada-punto-70porciento",
        Kind::FixedPoint,
        Parameters {
            fraction: 0.7,
            ..DATA
        },
    ),
    row("apuntalado-uniforme-L4", Kind::Propped, DATA),
    row(
        "apuntalado-uniforme-L7",
        Kind::Propped,
        Parameters {
            length: 7.0,
            ..DATA
        },
    ),
    row("continua-dos-vanos-L4", Kind::Continuous, DATA),
    row(
        "continua-dos-vanos-L7",
        Kind::Continuous,
        Parameters {
            length: 7.0,
            ..DATA
        },
    ),
    row("dos-barras-vertical", Kind::TwoBars, DATA),
    row(
        "dos-barras-mixta",
        Kind::TwoBars,
        Parameters {
            horizontal: 5.0,
            ..DATA
        },
    ),
    row("triangulo-vertical", Kind::Triangle, DATA),
    row(
        "triangulo-mixta",
        Kind::Triangle,
        Parameters {
            horizontal: 5.0,
            ..DATA
        },
    ),
    row("tres-concurrentes-vertical", Kind::Concurrent, DATA),
    row(
        "tres-concurrentes-mixta",
        Kind::Concurrent,
        Parameters {
            horizontal: 5.0,
            ..DATA
        },
    ),
    row("barra-asentamiento-positivo", Kind::Settlement, DATA),
    row(
        "barra-asentamiento-negativo",
        Kind::Settlement,
        Parameters {
            settlement: -0.02,
            ..DATA
        },
    ),
    row("portal-lateral-h3", Kind::PortalLateral, DATA),
    row(
        "portal-lateral-h5",
        Kind::PortalLateral,
        Parameters {
            height: 5.0,
            ..DATA
        },
    ),
    row("portal-vertical-h3", Kind::PortalVertical, DATA),
    row(
        "portal-vertical-h5",
        Kind::PortalVertical,
        Parameters {
            height: 5.0,
            ..DATA
        },
    ),
    row("articulacion-extremo-L4", Kind::Release, DATA),
    row(
        "articulacion-extremo-L7",
        Kind::Release,
        Parameters {
            length: 7.0,
            ..DATA
        },
    ),
    row(
        "rotacion-modelo-30grados",
        Kind::CantileverPoint,
        Parameters {
            angle: std::f64::consts::PI / 6.0,
            ..DATA
        },
    ),
    row(
        "rotacion-modelo-90grados",
        Kind::CantileverPoint,
        Parameters {
            angle: std::f64::consts::PI / 2.0,
            ..DATA
        },
    ),
    row(
        "rotacion-modelo-137grados",
        Kind::CantileverPoint,
        Parameters {
            angle: 137.0 * std::f64::consts::PI / 180.0,
            ..DATA
        },
    ),
    row("superposicion-P-mas-w", Kind::Combination, DATA),
    row(
        "combinacion-1_3-menos-0_7",
        Kind::Combination,
        Parameters {
            factors: [1.3, -0.7],
            ..DATA
        },
    ),
    row(
        "combinacion-menos-0_5-mas-2",
        Kind::Combination,
        Parameters {
            factors: [-0.5, 2.0],
            ..DATA
        },
    ),
    row(
        "combinacion-cero-mas-0_5",
        Kind::Combination,
        Parameters {
            factors: [0.0, 0.5],
            ..DATA
        },
    ),
];
