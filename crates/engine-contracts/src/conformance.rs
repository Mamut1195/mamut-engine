//! Kit reutilizable de invariantes, independiente de los ejemplos y del backend.
//!
//! Los llamadores aportan muestras interiores al rango diferenciable declarado y
//! un incremento absoluto FD en sus unidades; no se infiere el rango físico.
//! Una lista vacía o un incremento inválido produce diagnóstico (nunca pasa vacío).
//! El error de tangente es norma infinito de la diferencia / máximo de las
//! normas infinito, estrictamente < 1e-6; ambos nulos implican error cero.
//! Comparaciones de aceptación/covarianza usan 1e-10 relativo + 1e-12 absoluto.
//! Modos rígidos son infinitesimales desde origen sin precarga; su residuo se
//! escala por norma de tangente y desplazamiento (umbral 1e-10 + 1e-12).
//! Esto comprueba las muestras, no demuestra corrección para todo el dominio.
use crate::*;
use engine_core::math;
use engine_core::{EntityRef, Severity};

fn issue(out: &mut Vec<Diagnostic>, context: &str, detail: impl Into<String>) {
    let mut d = Diagnostic::from_code("ENG-E-0009").expect("Código estable existente");
    d.entities.push(EntityRef::Law(context.into()));
    d.hint = Some(format!("{context}: {}", detail.into()));
    out.push(d);
}
fn take<T>(out: &mut Vec<Diagnostic>, context: &str, value: EResult<T>) -> Option<T> {
    match value {
        Ok(value) => Some(value),
        Err(e) => {
            let mut d = Diagnostic::new(e.code, Severity::Error, e.message);
            d.entities = e.entities;
            d.hint = Some(context.into());
            out.push(d);
            None
        }
    }
}
fn valid(out: &mut Vec<Diagnostic>, context: &str, values: &[Real], len: usize) -> bool {
    let ok = values.len() == len && values.iter().all(|v| v.is_finite());
    if !ok {
        issue(
            out,
            context,
            format!(
                "Longitud esperada {len}, recibida {}; todos deben ser finitos: {values:?}",
                values.len()
            ),
        );
    }
    ok
}
fn magnitude(values: &[Real]) -> Real {
    values.iter().fold(0.0, |m, v| m.max(v.abs()))
}
fn relative(a: &[Real], b: &[Real]) -> Real {
    if a.len() != b.len() || a.iter().chain(b).any(|v| !v.is_finite()) {
        return Real::INFINITY;
    }
    let scale = magnitude(a).max(magnitude(b));
    if scale == 0.0 {
        return 0.0;
    }
    a.iter()
        .zip(b)
        .fold(0.0, |m, (x, y)| m.max((x / scale - y / scale).abs()))
}
fn close(out: &mut Vec<Diagnostic>, context: &str, a: &[Real], b: &[Real], label: &str) {
    if a.len() != b.len()
        || a.iter().zip(b).any(|(x, y)| {
            !x.is_finite() || !y.is_finite() || (x - y).abs() > 1e-12 + 1e-10 * x.abs().max(y.abs())
        })
    {
        issue(
            out,
            context,
            format!("{label}: esperado {a:?}, recibido {b:?}"),
        );
    }
}
fn contract(out: &mut Vec<Diagnostic>, context: &str, actual: ContractId, name: &'static str) {
    if !actual.is_compatible_with(&ContractId::new(name, 1, 0)) {
        issue(
            out,
            context,
            format!("Contrato {actual}; se esperaba {name}/1"),
        );
    }
}
fn matrix(out: &mut Vec<Diagnostic>, context: &str, m: &DMat, n: usize) -> bool {
    if m.rows() != n || m.cols() != n {
        issue(
            out,
            context,
            format!("Matriz {}×{}, esperada {n}×{n}", m.rows(), m.cols()),
        );
        return false;
    }
    valid(out, context, m.values(), n * n)
}
fn symmetric(out: &mut Vec<Diagnostic>, context: &str, m: &DMat, label: &str) {
    for i in 0..m.rows() {
        for j in 0..i {
            close(
                out,
                context,
                &[m.get(i, j).unwrap()],
                &[m.get(j, i).unwrap()],
                &format!("Simetría {label} ({i},{j})"),
            );
        }
    }
}
/// Rango del incremento de diferencias finitas centrales: por debajo, el
/// redondeo domina (error ~ε/h); por encima, el truncamiento (error ~h²) invalida
/// la comparación con tolerancia 1e-6. T2 (J4): h acotado, no solo finito > 0.
pub const FD_STEP_MIN: Real = 1e-9;
/// Cota superior del incremento de diferencias finitas (véase [`FD_STEP_MIN`]).
pub const FD_STEP_MAX: Real = 1e-2;
fn setup(out: &mut Vec<Diagnostic>, context: &str, count: usize, h: Real) -> bool {
    if count == 0 || !h.is_finite() || h <= 0.0 {
        issue(
            out,
            context,
            "Se requieren muestras no vacías e incremento FD finito positivo.",
        );
        false
    } else if !(FD_STEP_MIN..=FD_STEP_MAX).contains(&h) {
        issue(
            out,
            context,
            format!(
                "incremento FD {h} fuera de [{FD_STEP_MIN}, {FD_STEP_MAX}]: el error de redondeo o truncamiento invalida la comparación."
            ),
        );
        false
    } else {
        true
    }
}

/// Comprueba uniaxial/1: historia inicial/de prueba, finitud, repetibilidad,
/// tensión aceptada, tangente inicial y derivada central con error < 1e-6.
///
/// samples en deformación adimensional (o m para resorte), h en la misma unidad;
/// cada sample ± h debe estar en rango diferenciable. Recorre una trayectoria
/// aceptando explícitamente trial_state entre muestras sin modificar el objeto.
/// Devuelve diagnósticos con kind, índice, valores y error relativo, nunca panic
/// por una respuesta con longitud incorrecta; propaga errores con contexto.
///
/// ```
/// use engine_contracts::{conformance::check_uniaxial_law, examples::LinearElastic};
/// let law = LinearElastic::new(12.0).unwrap();
/// assert!(check_uniaxial_law(&law, &[0.0, 0.2, -0.3], 1e-5).is_empty());
/// ```
pub fn check_uniaxial_law(law: &dyn UniaxialLaw, samples: &[Real], h: Real) -> Vec<Diagnostic> {
    let mut out = vec![];
    let name = law.kind();
    contract(&mut out, name, law.contract(), "uniaxial");
    let mut state = law.initial_state();
    if !valid(&mut out, name, &state, law.state_len()) || !setup(&mut out, name, samples.len(), h) {
        return out;
    }
    valid(&mut out, name, &[law.initial_tangent()], 1);
    for (i, x) in samples.iter().enumerate() {
        let ctx = format!("{name}, muestra {i}, deformación {x}");
        let Some(base) = take(&mut out, &ctx, law.respond(*x, &state)) else {
            continue;
        };
        valid(&mut out, &ctx, &[base.stress, base.tangent], 2);
        if !valid(&mut out, &ctx, &base.trial_state, law.state_len()) {
            continue;
        }
        if let Some(repeat) = take(&mut out, &ctx, law.respond(*x, &state)) {
            close(
                &mut out,
                &ctx,
                &[base.stress, base.tangent],
                &[repeat.stress, repeat.tangent],
                "Respuesta repetida",
            );
            close(
                &mut out,
                &ctx,
                &base.trial_state,
                &repeat.trial_state,
                "Historia repetida",
            );
        }
        if let Some(accepted) = take(&mut out, &ctx, law.respond(*x, &base.trial_state)) {
            close(
                &mut out,
                &ctx,
                &[base.stress],
                &[accepted.stress],
                "Tensión aceptada",
            );
            valid(&mut out, &ctx, &[accepted.stress, accepted.tangent], 2);
            valid(&mut out, &ctx, &accepted.trial_state, law.state_len());
        }
        if let (Some(plus), Some(minus)) = (
            take(&mut out, &ctx, law.respond(x + h, &state)),
            take(&mut out, &ctx, law.respond(x - h, &state)),
        ) {
            valid(&mut out, &ctx, &plus.trial_state, law.state_len());
            valid(&mut out, &ctx, &minus.trial_state, law.state_len());
            valid(
                &mut out,
                &ctx,
                &[plus.stress, plus.tangent, minus.stress, minus.tangent],
                4,
            );
            let fd = (plus.stress - minus.stress) / (2.0 * h);
            let error = relative(&[base.tangent], &[fd]);
            if error >= 1e-6 {
                issue(
                    &mut out,
                    &ctx,
                    format!(
                        "Tangente {}, FD {fd}, error relativo {error} >= 1e-6",
                        base.tangent
                    ),
                );
            }
        }
        state = base.trial_state;
    }
    if let Some(origin) = take(&mut out, name, law.respond(0.0, &law.initial_state())) {
        valid(&mut out, name, &[origin.stress, origin.tangent], 2);
        valid(&mut out, name, &origin.trial_state, law.state_len());
        close(
            &mut out,
            name,
            &[law.initial_tangent()],
            &[origin.tangent],
            "Tangente inicial",
        );
    }
    out
}

struct Response {
    force: Vec<Real>,
    tangent: DMat,
    state: Vec<Real>,
}
fn check_response(
    out: &mut Vec<Diagnostic>,
    context: &str,
    r: &Response,
    n: usize,
    ns: usize,
) -> bool {
    let f = valid(out, context, &r.force, n);
    let s = valid(out, context, &r.state, ns);
    let m = matrix(out, context, &r.tangent, n);
    if m {
        symmetric(out, context, &r.tangent, "tangente");
    }
    f && s && m
}
#[allow(clippy::too_many_arguments)]
fn vector_law(
    out: &mut Vec<Diagnostic>,
    name: &str,
    n: usize,
    ns: usize,
    initial: Vec<Real>,
    samples: &[Vec<Real>],
    h: Real,
    evaluate: impl Fn(&[Real], &[Real]) -> EResult<Response>,
) {
    if !setup(out, name, samples.len(), h) || !valid(out, name, &initial, ns) {
        return;
    }
    let mut state = initial;
    for (index, u) in samples.iter().enumerate() {
        let context = format!("{name}, muestra {index}");
        if !valid(out, &context, u, n) {
            continue;
        }
        let Some(base) = take(out, &context, evaluate(u, &state)) else {
            continue;
        };
        if !check_response(out, &context, &base, n, ns) {
            continue;
        }
        if let Some(repeat) = take(out, &context, evaluate(u, &state)) {
            if check_response(out, &context, &repeat, n, ns) {
                close(
                    out,
                    &context,
                    &base.force,
                    &repeat.force,
                    "Respuesta repetida",
                );
                close(
                    out,
                    &context,
                    base.tangent.values(),
                    repeat.tangent.values(),
                    "Tangente repetida",
                );
                close(
                    out,
                    &context,
                    &base.state,
                    &repeat.state,
                    "Historia repetida",
                );
            }
        }
        if let Some(accepted) = take(out, &context, evaluate(u, &base.state)) {
            if check_response(out, &context, &accepted, n, ns) {
                close(
                    out,
                    &context,
                    &base.force,
                    &accepted.force,
                    "Respuesta aceptada",
                );
            }
        }
        let mut fd = vec![0.0; n * n];
        let mut complete = true;
        for j in 0..n {
            let mut up = u.clone();
            let mut um = u.clone();
            up[j] += h;
            um[j] -= h;
            let plus = take(out, &context, evaluate(&up, &state));
            let minus = take(out, &context, evaluate(&um, &state));
            if let (Some(p), Some(m)) = (plus, minus) {
                if check_response(out, &context, &p, n, ns)
                    && check_response(out, &context, &m, n, ns)
                {
                    for i in 0..n {
                        fd[i * n + j] = (p.force[i] - m.force[i]) / (2.0 * h);
                    }
                } else {
                    complete = false;
                }
            } else {
                complete = false;
            }
        }
        if complete {
            let error = relative(base.tangent.values(), &fd);
            if error >= 1e-6 {
                issue(
                    out,
                    &context,
                    format!("Tangente inconsistente: error relativo {error} >= 1e-6; FD={fd:?}"),
                );
            }
        }
        state = base.state;
    }
}

/// Comprueba section/1 con deformaciones D2 de longitud 2 o D3 de longitud 4.
/// Usa la trayectoria de estados aceptados, finitud, repetibilidad, simetría y
/// tangente FD central (<1e-6). h es incremento absoluto (ε y m⁻¹ respectivamente);
/// el llamador escoge escalas SI compatibles y muestras interiores suaves.
/// Devuelve diagnósticos detallados de contrato, tamaños, estado o derivada.
pub fn check_section_law(law: &dyn SectionLaw, samples: &[Vec<Real>], h: Real) -> Vec<Diagnostic> {
    let mut out = vec![];
    contract(&mut out, law.kind(), law.contract(), "section");
    let n = if law.dim() == Dim::D2 { 2 } else { 4 };
    vector_law(
        &mut out,
        law.kind(),
        n,
        law.state_len(),
        law.initial_state(),
        samples,
        h,
        |u, s| {
            law.respond(u, s).map(|r| Response {
                force: r.resultants,
                tangent: r.tangent,
                state: r.trial_state,
            })
        },
    );
    out
}

fn response(
    element: &dyn Element,
    ctx: &ElementCtx<'_>,
    u: &[Real],
    state: &[Real],
) -> EResult<Response> {
    element.respond(ctx, u, state).map(|r| Response {
        force: r.internal_force,
        tangent: r.tangent,
        state: r.trial_state,
    })
}
fn cross(a: [Real; 3], b: [Real; 3]) -> [Real; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn multiply(q: &[[Real; 3]; 3], x: [Real; 3]) -> [Real; 3] {
    std::array::from_fn(|i| (0..3).map(|j| q[i][j] * x[j]).sum())
}
fn rotated(dofs: &[Dof], values: &[Real], q: &[[Real; 3]; 3]) -> Vec<Real> {
    let mut out = vec![];
    for node in values.chunks_exact(dofs.len()) {
        let mut translation = [0.0; 3];
        let mut rotation = [0.0; 3];
        for (d, v) in dofs.iter().zip(node) {
            if d.index() < 3 {
                translation[d.index()] = *v;
            } else {
                rotation[d.index() - 3] = *v;
            }
        }
        let translation = multiply(q, translation);
        let rotation = multiply(q, rotation);
        out.extend(dofs.iter().map(|d| {
            if d.index() < 3 {
                translation[d.index()]
            } else {
                rotation[d.index() - 3]
            }
        }));
    }
    out
}
fn transformed(dofs: &[Dof], m: &DMat, q: &[[Real; 3]; 3]) -> Vec<Real> {
    // Forma Q M Qᵀ únicamente para verificar covarianza, sin exponer álgebra pública.
    let n = m.rows();
    let columns: Vec<_> = (0..n)
        .map(|j| {
            let mut e = vec![0.0; n];
            e[j] = 1.0;
            rotated(dofs, &e, q)
        })
        .collect();
    let mut out = vec![0.0; n * n];
    for i in 0..n {
        for j in 0..n {
            for a in 0..n {
                for b in 0..n {
                    out[i * n + j] += columns[a][i] * m.get(a, b).unwrap() * columns[b][j];
                }
            }
        }
    }
    out
}

/// Comprueba element/1: tamaños, finitud, estado aceptado, repetibilidad,
/// tangente FD <1e-6 y simetría de tangente/ambas masas/Kg si existen.
///
/// ctx debe tener geometría válida sin precarga y samples desplazamientos totales
/// SI de n_nodes × dofs_per_node. h absoluto en m/rad. Ejecuta 3 modos rígidos
/// D2 o 6 D3 proyectados a los GDL activos. Para conjuntos cerrados bajo rotación
/// (todas las traslaciones y ningún giro o todos los giros de la dimensión),
/// verifica fuerzas, tangente y masa ante dos rotaciones no triviales por eje.
/// Para otros conjuntos la covarianza no aplica; use un elemento orientado con
/// GDL completos para comprobarla. Historia inicial sin orientación propia;
/// parámetros escalares se conservan al rotar. Orientaciones globales propias
/// deben transformarse en un fixture específico, fuera de este helper mínimo.
/// También comprueba carga nula y recuperación sin cargas en [0,0.5,1].
/// Devuelve todos los diagnósticos con kind e invariante; no resuelve ecuaciones.
pub fn check_element(
    element: &dyn Element,
    ctx: &ElementCtx<'_>,
    samples: &[Vec<Real>],
    h: Real,
) -> Vec<Diagnostic> {
    let mut out = vec![];
    let name = element.kind();
    let dofs = element.dofs_per_node();
    contract(&mut out, name, element.contract(), "element");
    if ctx.coords.len() != element.n_nodes() || dofs.is_empty() || element.n_nodes() == 0 {
        issue(&mut out, name, "Contexto/nodos/GDL incompatibles o vacíos.");
        return out;
    }
    for (i, d) in dofs.iter().enumerate() {
        if !ctx.dim.dofs().contains(d) || dofs[..i].contains(d) {
            issue(&mut out, name, "GDL duplicado o ajeno a la dimensión.");
            return out;
        }
    }
    for p in ctx.coords {
        if !valid(&mut out, name, p, 3) {
            return out;
        }
    }
    if ctx.dim == Dim::D2 && ctx.coords.iter().any(|p| p[2] != 0.0) {
        issue(&mut out, name, "D2 requiere Z=0.");
        return out;
    }
    let n = element.n_nodes() * dofs.len();
    let ns = element.state_len();
    let state = element.initial_state();
    if !valid(&mut out, name, &state, ns) {
        return out;
    }
    vector_law(&mut out, name, n, ns, state.clone(), samples, h, |u, s| {
        response(element, ctx, u, s)
    });
    let zero = vec![0.0; n];
    let Some(origin) = take(&mut out, name, response(element, ctx, &zero, &state)) else {
        return out;
    };
    if !check_response(&mut out, name, &origin, n, ns) {
        return out;
    }
    let axes = if ctx.dim == Dim::D2 { 2 } else { 3 };
    for mode in 0..if ctx.dim == Dim::D2 { 3 } else { 6 } {
        let mut u = vec![];
        for p in ctx.coords {
            let mut t = [0.0; 3];
            let mut r = [0.0; 3];
            if mode < axes {
                t[mode] = 1.0;
            } else {
                r[if ctx.dim == Dim::D2 { 2 } else { mode - 3 }] = 1.0;
                t = cross(r, *p);
            }
            u.extend(dofs.iter().map(|d| {
                if d.index() < 3 {
                    t[d.index()]
                } else {
                    r[d.index() - 3]
                }
            }));
        }
        // Traslaciones: el modo es exacto también en el sentido no lineal. Giros:
        // un giro finito (amplitud 1 rad) no es rígido para un elemento no
        // lineal válido (p. ej. no corrotacional); el modo rígido es entonces
        // INFINITESIMAL: se exige K₀·u ≈ 0 con la tangente del origen.
        if mode < axes {
            if let Some(r) = take(&mut out, name, response(element, ctx, &u, &state)) {
                if check_response(&mut out, name, &r, n, ns) {
                    let scale = magnitude(origin.tangent.values()) * magnitude(&u) * (n as Real);
                    let residual = magnitude(&r.force);
                    if !scale.is_finite() || residual > 1e-12 + 1e-10 * scale {
                        issue(
                            &mut out,
                            name,
                            format!(
                                "Modo rígido {mode}: fuerza residual {residual}, escala {scale}"
                            ),
                        );
                    }
                }
            }
        } else {
            // Se conserva la validación de respuesta (longitudes, finitud) de ese estado.
            if let Some(r) = take(&mut out, name, response(element, ctx, &u, &state)) {
                check_response(&mut out, name, &r, n, ns);
            }
            let k = origin.tangent.values();
            let ku: Vec<Real> = (0..n)
                .map(|i| (0..n).map(|j| k[i * n + j] * u[j]).sum())
                .collect();
            let scale = magnitude(k) * magnitude(&u) * (n as Real);
            let residual = magnitude(&ku);
            if !scale.is_finite() || residual > 1e-12 + 1e-10 * scale {
                issue(
                    &mut out,
                    name,
                    format!(
                        "Modo rígido {mode} (infinitesimal): K·u residual {residual}, escala {scale}"
                    ),
                );
            }
        }
    }
    let kinds = [MassKind::Lumped, MassKind::Consistent];
    let masses: Vec<_> = kinds
        .iter()
        .map(|kind| {
            take(&mut out, name, element.mass(ctx, *kind))
                .flatten()
                .filter(|m| {
                    if matrix(&mut out, name, m, n) {
                        symmetric(&mut out, name, m, "masa");
                        true
                    } else {
                        false
                    }
                })
        })
        .collect();
    for u in samples {
        if u.len() != n || u.iter().any(|v| !v.is_finite()) {
            continue;
        }
        if let Some(Some(kg)) = take(&mut out, name, element.geometric_stiffness(ctx, u, &state)) {
            if matrix(&mut out, name, &kg, n) {
                symmetric(&mut out, name, &kg, "rigidez geométrica");
            }
        }
    }
    let translations = ctx
        .dim
        .dofs()
        .iter()
        .filter(|d| d.index() < 3)
        .all(|d| dofs.contains(d));
    let rotations = ctx
        .dim
        .dofs()
        .iter()
        .filter(|d| d.index() >= 3)
        .all(|d| dofs.contains(d))
        || dofs.iter().all(|d| d.index() < 3);
    if translations && rotations {
        for axis in if ctx.dim == Dim::D2 { 2..3 } else { 0..3 } {
            for angle in [0.37_f64, 1.13_f64] {
                let (s, c) = math::sin_cos(angle);
                let mut q = [[0.0; 3]; 3];
                q[axis][axis] = 1.0;
                let a = (axis + 1) % 3;
                let b = (axis + 2) % 3;
                q[a][a] = c;
                q[a][b] = -s;
                q[b][a] = s;
                q[b][b] = c;
                let coords: Vec<_> = ctx.coords.iter().map(|p| multiply(&q, *p)).collect();
                let rotated_ctx = ElementCtx {
                    dim: ctx.dim,
                    coords: &coords,
                    params: ctx.params,
                };
                let context = format!("{name}, rotación eje {axis}, ángulo {angle}");
                for u in samples {
                    if u.len() != n || u.iter().any(|v| !v.is_finite()) {
                        continue;
                    }
                    let base = take(&mut out, &context, response(element, ctx, u, &state));
                    let ru = rotated(dofs, u, &q);
                    let rot = take(
                        &mut out,
                        &context,
                        response(element, &rotated_ctx, &ru, &state),
                    );
                    if let (Some(base), Some(rot)) = (base, rot) {
                        if check_response(&mut out, &context, &base, n, ns)
                            && check_response(&mut out, &context, &rot, n, ns)
                        {
                            close(
                                &mut out,
                                &context,
                                &rotated(dofs, &base.force, &q),
                                &rot.force,
                                "Covarianza fuerza",
                            );
                            close(
                                &mut out,
                                &context,
                                &transformed(dofs, &base.tangent, &q),
                                rot.tangent.values(),
                                "Covarianza tangente",
                            );
                        }
                    }
                }
                for (i, kind) in kinds.iter().enumerate() {
                    if let Some(rot) = take(&mut out, &context, element.mass(&rotated_ctx, *kind)) {
                        match (&masses[i], rot) {
                            (Some(base), Some(rot)) if matrix(&mut out, &context, &rot, n) => {
                                close(
                                    &mut out,
                                    &context,
                                    &transformed(dofs, base, &q),
                                    rot.values(),
                                    "Covarianza masa",
                                )
                            }
                            (None, None) => (),
                            _ => issue(&mut out, &context, "Masa incompatible tras rotación."),
                        }
                    }
                }
            }
        }
    }
    let load = MemberLoad {
        force_per_length: [0.0; 3],
        moment_per_length: [0.0; 3],
    };
    if let Some(f) = take(&mut out, name, element.load_vector(ctx, &load)) {
        close(&mut out, name, &zero, &f, "Carga nula");
    }
    for u in samples {
        if u.len() != n || u.iter().any(|v| !v.is_finite()) {
            continue;
        }
        if let Some(r) = take(
            &mut out,
            name,
            element.recover(ctx, u, &state, &[], &[0.0, 0.5, 1.0]),
        ) {
            valid(&mut out, name, &r.end_forces_local, n);
            if r.stations.len() != 3 {
                issue(
                    &mut out,
                    name,
                    "Recuperación: número de estaciones incorrecto.",
                );
            }
            for (station, x) in r.stations.iter().zip([0.0, 0.5, 1.0]) {
                close(
                    &mut out,
                    name,
                    &[x],
                    &[station.position],
                    "Posición estación",
                );
                valid(&mut out, name, &station.forces_local, 6);
            }
        }
    }
    out
}

fn validate_report(report: &ConvergenceReport) -> EResult<()> {
    let mut previous = 0;
    for step in &report.steps {
        let t = step.tolerances;
        finite(&[
            step.load_factor,
            step.residual_norm,
            step.displacement_norm,
            step.energy_norm,
            t.force_rel,
            t.disp_rel,
            t.energy_rel,
        ])?;
        if step.step <= previous
            || [step.residual_norm, step.displacement_norm, step.energy_norm]
                .iter()
                .any(|v| *v < 0.0)
            || [t.force_rel, t.disp_rel, t.energy_rel]
                .iter()
                .any(|v| *v <= 0.0)
        {
            return Err(invalid(
                "Informe con pasos desordenados, normas negativas o tolerancias no positivas.",
            ));
        }
        if step.converged
            && (step.residual_norm > t.force_rel
                || step.displacement_norm > t.disp_rel
                || step.energy_norm > t.energy_rel)
        {
            return Err(invalid(
                "Paso declarado convergente fuera de sus tolerancias.",
            ));
        }
        previous = step.step;
    }
    Ok(())
}

/// Valida clave/valor mínimo results/1, sin escribir ni serializar.
/// Exige bloque conocido, campo/entidad no vacíos, numéricos finitos, posiciones
/// `[0,1]` y normas/tolerancias coherentes. No impone cardinalidad por campo:
/// corresponde al esquema de ENG-CORE-005. ENG-E-0010 por estructura/rango,
/// ENG-E-0008 por no finitos y ENG-E-0009 por diagnóstico fuera del catálogo.
///
/// ```
/// use engine_contracts::*;
/// let key=ResultKey {block:"nodes".into(),entity:Some("1".into()),field:"u".into()};
/// assert!(conformance::validate_result(&key,&ResultValue::Vector(vec![f64::NAN])).is_err());
/// ```
pub fn validate_result(key: &ResultKey, value: &ResultValue) -> EResult<()> {
    if ![
        "nodes",
        "members",
        "modes",
        "cases",
        "combinations",
        "envelopes",
        "convergence",
        "diagnostics",
        "provenance",
    ]
    .contains(&key.block.as_str())
        || key.field.trim().is_empty()
        || key.entity.as_ref().is_some_and(|s| s.trim().is_empty())
    {
        return Err(invalid("Clave de resultado inválida."));
    }
    match value {
        ResultValue::Scalar(v) => finite(&[*v]),
        ResultValue::Vector(v) => finite(v),
        ResultValue::Text(_) => Ok(()),
        ResultValue::Convergence(r) => validate_report(r),
        ResultValue::Diagnostics(d) => {
            if d.iter()
                .any(|d| engine_core::catalog::lookup(d.code).is_none())
            {
                Err(EngineError::new(
                    "ENG-E-0009",
                    "Código de diagnóstico desconocido.",
                ))
            } else {
                Ok(())
            }
        }
        ResultValue::Element(r) => {
            finite(&r.end_forces_local)?;
            for s in &r.stations {
                finite(&[s.position])?;
                finite(&s.forces_local)?;
                if !(0.0..=1.0).contains(&s.position) {
                    return Err(invalid("Estación fuera del tramo."));
                }
            }
            Ok(())
        }
    }
}

/// Construye un elemento mediante una fábrica y ejecuta check_element.
/// params debe ser válido según su esquema; el contexto debe corresponder al
/// producto. Verifica esquema no vacío y coincidencia de kind. Los ejemplos
/// negativos de parámetros se prueban aparte porque su rango depende del kind.
/// Devuelve fallos de construcción con contexto y todos los invariantes físicos.
pub fn check_element_factory(
    factory: &dyn ElementFactory,
    params: &JsonValue,
    model: &ModelView<'_>,
    ctx: &ElementCtx<'_>,
    samples: &[Vec<Real>],
    h: Real,
) -> Vec<Diagnostic> {
    let mut out = vec![];
    let name = factory.kind();
    if factory.params_schema().trim().is_empty() {
        issue(&mut out, name, "Esquema de parámetros vacío.");
    }
    if let Some(element) = take(&mut out, name, factory.build(params, model)) {
        if element.kind() != name {
            issue(&mut out, name, "Kind del producto distinto de la fábrica.");
        }
        out.extend(check_element(element.as_ref(), ctx, samples, h));
    }
    out
}

/// Factor del centinela respecto a la magnitud de la matriz limpia: debe superar
/// con holgura la tolerancia relativa de `close` (1e-10) para que una
/// implementación que acumule el centinela no quede dentro de ella.
const SENTINEL_FACTOR: Real = 1e3;
/// Evalúa una operación que debe SOBRESCRIBIR una matriz de salida: una vez con
/// salida en ceros y otra prellenada con un centinela simétrico no nulo; ambas
/// deben dar el mismo resultado (una implementación que acumule en `out` falla).
/// Devuelve la matriz de la primera llamada si ambas tuvieron éxito.
fn overwritten(
    out: &mut Vec<Diagnostic>,
    context: &str,
    label: &str,
    n: usize,
    mut call: impl FnMut(&mut SymMatrix) -> EResult<()>,
) -> Option<SymMatrix> {
    let mut clean = take(out, context, SymMatrix::zeros(n))?;
    take(out, context, call(&mut clean))?;
    // Centinela simétrico PROPORCIONAL a la magnitud de la matriz limpia
    // (mínimo 1): con K~1e12 un centinela fijo de 7.25 caería dentro de la tolerancia.
    let magnitude = clean
        .dense()
        .values()
        .iter()
        .fold(1.0_f64, |m, v| m.max(v.abs()));
    // Con magnitud ~1e306 el factor desbordaría: se usa entonces la propia
    // magnitud (acumular la duplicaría: muy por encima de la tolerancia de `close`).
    let scaled = SENTINEL_FACTOR * magnitude;
    let sentinel = if scaled.is_finite() {
        scaled
    } else {
        magnitude
    };
    let dirty = take(out, context, DMat::new(n, n, vec![sentinel; n * n]))?;
    let mut dirty = take(out, context, SymMatrix::new(dirty))?;
    take(out, context, call(&mut dirty))?;
    if dirty.dense().rows() != n || dirty.dense().cols() != n {
        issue(
            out,
            context,
            format!(
                "{label}: la segunda salida cambió de dimensión ({}×{}, esperada {n}×{n}).",
                dirty.dense().rows(),
                dirty.dense().cols()
            ),
        );
        return Some(clean);
    }
    if clean.dense().rows() == n && clean.dense().cols() == n {
        close(
            out,
            context,
            clean.dense().values(),
            dirty.dense().values(),
            &format!("{label}: debe sobrescribir `out` (no acumular)"),
        );
    }
    Some(clean)
}

/// Comprueba SystemView mediante muestras totales SI y caso válido.
/// Sobrescritura de fuerzas (salida prellenada con NaN) y de matrices (salida
/// en ceros y con centinela simétrico: ambas deben coincidir), patrón del
/// orden correcto, masa/Kg finitas, tangente FD <1e-6, escalado lineal del caso,
/// aceptación y rollback conservando la respuesta del último aceptado.
/// Muta el sistema: al finalizar queda aceptada la última muestra exitosa.
/// No comprueba modos rígidos, pues el sistema reducido puede contener apoyos.
/// Devuelve diagnósticos; muestras vacías/h inválido nunca pasan silenciosamente.
pub fn check_system_view(
    sys: &mut dyn SystemView,
    samples: &[Vec<Real>],
    case: &CaseRef,
    h: Real,
) -> Vec<Diagnostic> {
    let mut out = vec![];
    let name = "SystemView";
    let n = sys.n_free_dofs();
    if !setup(&mut out, name, samples.len(), h) {
        return out;
    }
    if n == 0 || sys.pattern().order() != n {
        issue(&mut out, name, "Patrón u orden vacío/incompatible.");
        return out;
    }
    for kind in [MassKind::Lumped, MassKind::Consistent] {
        if let Some(mass) = overwritten(&mut out, name, "Masa", n, |m| sys.mass(kind, m)) {
            matrix(&mut out, name, mass.dense(), n);
        }
    }
    let mut f = vec![Real::NAN; n];
    let mut twice = vec![Real::NAN; n];
    if take(&mut out, name, sys.external_force(case, 1.0, &mut f)).is_some()
        && take(&mut out, name, sys.external_force(case, 2.0, &mut twice)).is_some()
    {
        valid(&mut out, name, &f, n);
        valid(&mut out, name, &twice, n);
        close(
            &mut out,
            name,
            &f.iter().map(|v| v * 2.0).collect::<Vec<_>>(),
            &twice,
            "Escalado de caso",
        );
    }
    for (index, u) in samples.iter().enumerate() {
        let context = format!("SystemView, muestra {index}");
        if !valid(&mut out, &context, u, n) {
            continue;
        }
        let mut base = vec![Real::NAN; n];
        if take(&mut out, &context, sys.internal_force(u, &mut base)).is_none()
            || !valid(&mut out, &context, &base, n)
        {
            continue;
        }
        let Some(tangent) = overwritten(&mut out, &context, "Tangente", n, |m| sys.tangent(u, m))
        else {
            continue;
        };
        if !matrix(&mut out, &context, tangent.dense(), n) {
            continue;
        }
        if let Some(geometric) = overwritten(&mut out, &context, "Kg", n, |m| sys.geometric(u, m)) {
            matrix(&mut out, &context, geometric.dense(), n);
        }
        let mut fd = vec![0.0; n * n];
        let mut complete = true;
        for j in 0..n {
            let mut plus = u.clone();
            let mut minus = u.clone();
            plus[j] += h;
            minus[j] -= h;
            let mut fp = vec![Real::NAN; n];
            let mut fm = vec![Real::NAN; n];
            if take(&mut out, &context, sys.internal_force(&plus, &mut fp)).is_some()
                && take(&mut out, &context, sys.internal_force(&minus, &mut fm)).is_some()
                && valid(&mut out, &context, &fp, n)
                && valid(&mut out, &context, &fm, n)
            {
                for i in 0..n {
                    fd[i * n + j] = (fp[i] - fm[i]) / (2.0 * h);
                }
            } else {
                complete = false;
            }
        }
        if complete {
            let error = relative(tangent.dense().values(), &fd);
            if error >= 1e-6 {
                issue(
                    &mut out,
                    &context,
                    format!("Tangente del sistema: error relativo {error}"),
                );
            }
        }
        if take(&mut out, &context, sys.accept(u)).is_some() {
            let mut accepted = vec![Real::NAN; n];
            if take(&mut out, &context, sys.internal_force(u, &mut accepted)).is_some() {
                close(
                    &mut out,
                    &context,
                    &base,
                    &accepted,
                    "Respuesta aceptada del sistema",
                );
            }
            let mut trial = u.clone();
            trial[0] += h;
            take(
                &mut out,
                &context,
                sys.internal_force(&trial, &mut vec![0.0; n]),
            );
            sys.rollback();
            sys.rollback();
            let mut rolled = vec![Real::NAN; n];
            if take(&mut out, &context, sys.internal_force(u, &mut rolled)).is_some() {
                close(
                    &mut out,
                    &context,
                    &accepted,
                    &rolled,
                    "Rollback idempotente",
                );
            }
        }
        for i in 0..n {
            if let Ok(i) = u32::try_from(i) {
                sys.dof_label(DofIndex(i));
            } else {
                issue(&mut out, &context, "Índice no representable como DofIndex.");
                break;
            }
        }
    }
    out
}

/// Comprueba reports y contrato analysis/1 ejecutando una vez el método.
/// No impone estrategia ni solución específica: las muestras analíticas se prueban
/// por método. Verifica esquema/id, finitud, tolerancias, estado Completed y
/// último paso fallido para NotConverged. Propaga errores como diagnósticos.
/// Usa los ajustes SI y destinos/control aportados; muta el sistema y el sink.
pub fn check_analysis_method(
    method: &dyn AnalysisMethod,
    sys: &mut dyn SystemView,
    settings: &JsonValue,
    sink: &mut dyn ResultSink,
    ctl: &dyn RunControl,
) -> Vec<Diagnostic> {
    let mut out = vec![];
    let name = method.id();
    contract(&mut out, name, method.contract(), "analysis");
    if name.is_empty() || method.settings_schema().trim().is_empty() {
        issue(&mut out, name, "Id o esquema vacío.");
    }
    let already_cancelled = ctl.cancelled();
    if let Some(outcome) = take(&mut out, name, method.run(sys, settings, sink, ctl)) {
        take(&mut out, name, validate_report(&outcome.report));
        if already_cancelled && outcome.status != Status::Cancelled {
            issue(&mut out, name, "Se ignoró cancelación previa.");
        }
        if outcome.status == Status::Completed
            && (outcome.report.steps.is_empty()
                || outcome.report.steps.iter().any(|s| !s.converged))
        {
            issue(&mut out, name, "Completed sin pasos convergentes.");
        }
        if outcome.status == Status::NotConverged
            && outcome.report.steps.last().is_none_or(|s| s.converged)
        {
            issue(
                &mut out,
                name,
                "NotConverged sin intento fallido reportado.",
            );
        }
        for d in outcome.diagnostics {
            if engine_core::catalog::lookup(d.code).is_none() {
                issue(&mut out, name, "Diagnóstico fuera del catálogo.");
            }
        }
    }
    out
}

/// Prueba de recepción: entrega un valor SI válido y dos entradas inválidas.
/// Exige rechazo de NaN con ENG-E-0008 y clave vacía con ENG-E-0010.
/// Muta el destino; solo la implementación puede observar retención y atomicidad,
/// que se deben verificar con su acceso de lectura específico.
pub fn check_result_sink(sink: &mut dyn ResultSink) -> Vec<Diagnostic> {
    let mut out = vec![];
    let name = "ResultSink";
    let key = ResultKey {
        block: "nodes".into(),
        entity: Some("1".into()),
        field: "u".into(),
    };
    take(
        &mut out,
        name,
        sink.put(key.clone(), ResultValue::Vector(vec![0.0, 1.0])),
    );
    for (key, value, code) in [
        (key, ResultValue::Scalar(Real::NAN), "ENG-E-0008"),
        (
            ResultKey {
                block: String::new(),
                entity: None,
                field: String::new(),
            },
            ResultValue::Scalar(0.0),
            "ENG-E-0010",
        ),
    ] {
        match sink.put(key, value) {
            Err(e) if e.code == code => (),
            _ => issue(
                &mut out,
                name,
                format!("Entrada inválida no rechazada con {code}"),
            ),
        }
    }
    out
}

/// Comprueba trazas observables de RunControl: fracciones finitas en `[0,1]`,
/// monótonas y notas no vacías. Devuelve diagnósticos por índice. Las trazas
/// pueden provenir de cualquier observador; no presupone mutex ni hilos.
/// Una traza vacía falla; la cancelación se prueba mediante check_analysis_method.
pub fn check_run_control(events: &[(Real, String)]) -> Vec<Diagnostic> {
    let mut out = vec![];
    let mut previous = 0.0;
    if events.is_empty() {
        issue(&mut out, "RunControl", "Traza de progreso vacía.");
    }
    for (index, (fraction, note)) in events.iter().enumerate() {
        if !fraction.is_finite()
            || !(0.0..=1.0).contains(fraction)
            || *fraction < previous
            || note.trim().is_empty()
        {
            issue(
                &mut out,
                "RunControl",
                format!("Evento {index} inválido: {fraction}, {note:?}"),
            );
        }
        previous = *fraction;
    }
    out
}
