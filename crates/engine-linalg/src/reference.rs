//! Algoritmos propios derivados de eliminación, congruencias y rotaciones.
use crate::*;

/// Segundo backend mínimo para demostrar intercambiabilidad y validar contratos.
///
/// Factorización LDLᵀ densa con pivotes simétricos 1×1/2×2; autovalores por
/// Jacobi y reducción Cholesky de la masa. Memoria O(n²), trabajo O(n³) de
/// eliminación y hasta O(n⁴) de búsqueda Jacobi. Se destina a referencia y
/// pruebas, sin ordenamiento AMD ni solver productivo disperso.
#[derive(Default, Debug)]
pub struct ReferenceBackend;

impl LinAlgBackend for ReferenceBackend {
    fn name(&self) -> &'static str {
        "reference.dense"
    }
    fn matmul(&mut self, a: &DMat, b: &DMat) -> EResult<DMat> {
        size(a.cols(), b.rows())?;
        let mut c = DMat::zeros(a.rows(), b.cols())?;
        for i in 0..a.rows() {
            for j in 0..b.cols() {
                for k in 0..a.cols() {
                    c.values[i * b.cols() + j] +=
                        a.values()[i * a.cols() + k] * b.values()[k * b.cols() + j];
                }
            }
        }
        finite(&c.values)?;
        Ok(c)
    }
    fn transpose(&mut self, a: &DMat) -> EResult<DMat> {
        DMat::new(
            a.cols(),
            a.rows(),
            (0..a.cols())
                .flat_map(|j| (0..a.rows()).map(move |i| a.values()[i * a.cols() + j]))
                .collect(),
        )
    }
    fn solve_dense(&mut self, a: &DMat, b: &DMat) -> EResult<DMat> {
        size(a.rows(), a.cols())?;
        size(b.rows(), a.rows())?;
        let n = a.rows();
        if n == 0 {
            // Sin incógnitas: la solución es B (vacía) sea cual sea el número de
            // columnas; evita reservar vectores por columna (pánico con cols enormes).
            return Ok(b.clone());
        }
        let scale = magnitude(a.values());
        let mut u: Vec<_> = a.values().iter().map(|v| v / scale).collect();
        let mut x = b.clone();
        let cols = b.cols();
        // Escala independiente por RHS: dividir por la escala de A puede desbordar
        // aunque la solución sea finita (J4 ENG-LA-007, medio).
        let rhs_scales: Vec<Real> = (0..cols)
            .map(|c| magnitude_iter((0..n).map(|i| b.values()[i * cols + c])))
            .collect();
        for i in 0..n {
            for (c, &rs) in rhs_scales.iter().enumerate() {
                x.values[i * cols + c] /= rs;
            }
        }
        for k in 0..n {
            let pivot = (k..n)
                .max_by(|&i, &j| u[i * n + k].abs().total_cmp(&u[j * n + k].abs()))
                .unwrap();
            if u[pivot * n + k].abs() <= cutoff(n) {
                return Err(singular("Referencia solve denso", k));
            }
            for j in 0..n {
                u.swap(k * n + j, pivot * n + j);
            }
            for j in 0..b.cols() {
                x.values.swap(k * b.cols() + j, pivot * b.cols() + j);
            }
            for i in k + 1..n {
                let ratio = u[i * n + k] / u[k * n + k];
                for j in k + 1..n {
                    u[i * n + j] -= ratio * u[k * n + j];
                }
                for j in 0..b.cols() {
                    x.values[i * b.cols() + j] -= ratio * x.values[k * b.cols() + j];
                }
            }
        }
        for i in (0..n).rev() {
            for j in 0..b.cols() {
                for k in i + 1..n {
                    x.values[i * b.cols() + j] -= u[i * n + k] * x.values[k * b.cols() + j];
                }
                x.values[i * b.cols() + j] /= u[i * n + i];
            }
        }
        for i in 0..n {
            for (c, &rs) in rhs_scales.iter().enumerate() {
                x.values[i * cols + c] = rescale(x.values[i * cols + c], rs, scale);
            }
        }
        finite(&x.values)?;
        Ok(x)
    }
    fn factorize(&mut self, a: &SymMatrix) -> EResult<Box<dyn SymFactorization>> {
        Ok(Box::new(Ldlt::new(a)?))
    }
    fn generalized_eigen(&mut self, k: &SymMatrix, m: &SymMatrix) -> EResult<GeneralizedEigen> {
        size(k.order(), m.order())?;
        let l = cholesky(m)?;
        let inverse = self.solve_dense(&l, &identity(m.order())?)?;
        let left = self.matmul(&inverse, k.dense())?;
        let inverse_t = self.transpose(&inverse)?;
        let mut reduced = self.matmul(&left, &inverse_t)?;
        // Una sola mitad fija la congruencia simétrica pese al redondeo.
        for i in 0..m.order() {
            for j in 0..i {
                reduced.values[j * m.order() + i] = reduced.values[i * m.order() + j];
            }
        }
        let (eigenvalues, q) = jacobi(&SymMatrix::new(reduced)?)?;
        let vectors = self.matmul(&inverse_t, &q)?;
        normalize_modes(eigenvalues, vectors, m)
    }
}

pub(crate) fn identity(n: usize) -> EResult<DMat> {
    let mut a = DMat::zeros(n, n)?;
    for i in 0..n {
        a.values[i * n + i] = 1.0;
    }
    Ok(a)
}
fn magnitude_iter(values: impl Iterator<Item = Real>) -> Real {
    let scale = values.fold(0.0_f64, |a, v| a.max(v.abs()));
    if scale == 0.0 {
        1.0
    } else {
        scale
    }
}
/// Devuelve `v * num / den` evitando un desbordamiento intermedio evitable;
/// si el resultado real no es representable queda infinito y `finite` lo
/// reporta como ENG-E-0008.
fn rescale(v: Real, num: Real, den: Real) -> Real {
    let exact_zero = v == 0.0 || num == 0.0;
    let good = |x: Real| x.is_finite() && (exact_zero || x.abs() >= Real::MIN_POSITIVE);
    // Tres órdenes de evaluación: cada uno evita un desbordamiento o un
    // subdesbordamiento intermedio distinto; se elige el primero normal y finito.
    let candidates = [(v * num) / den, (v / den) * num, v * (num / den)];
    for c in candidates {
        if good(c) {
            return c;
        }
    }
    if candidates[0].is_finite() {
        candidates[0]
    } else {
        candidates[1]
    }
}
fn magnitude(values: &[Real]) -> Real {
    let scale = values.iter().fold(0.0_f64, |a, v| a.max(v.abs()));
    if scale == 0.0 {
        1.0
    } else {
        scale
    }
}
/// Equilibrado diagonal simétrico iterativo (tipo Ruiz) `B = S A S`: parte de
/// `s_i = 1` y repite `s_i ← s_i / √(máx_j |s_i a_ij s_j|)` (filas nulas: 1) hasta
/// que todas las filas de B tengan máximo ≈ 1 (±1 %) o 40 pasadas. Hace que
/// matrices diagonales y casi diagonales con GDL de escalas muy distintas
/// (diag(1e6, 1e-9) → identidad) y el reescalado `D A D` de matrices acopladas
/// moderadamente escaladas se clasifiquen igual. NO garantiza invariancia exacta
/// al reescalado diagonal en matrices acopladas con rangos extremos (T5, J4 r3).
fn equilibration(values: &[Real], n: usize) -> Vec<Real> {
    let mut s = vec![1.0; n];
    for _ in 0..40 {
        let mut worst: Real = 0.0;
        let rows: Vec<Real> = (0..n)
            .map(|i| {
                let m = (0..n).fold(0.0_f64, |m, j| {
                    m.max(((values[i * n + j] * s[i]) * s[j]).abs())
                });
                worst = worst.max((m - 1.0).abs());
                m
            })
            .collect();
        if worst <= 0.01 {
            break;
        }
        for (si, &m) in s.iter_mut().zip(&rows) {
            if m > 0.0 && m.is_finite() {
                *si /= m.sqrt();
            }
        }
    }
    s
}
/// Deshace equilibrado y normalizaciones: `x = s · (y · rs / scale)` evitando
/// desbordes y subdesbordes intermedios (se prueban varios órdenes de evaluación).
fn scale_back(y: Real, equil: Real, rs: Real, scale: Real) -> Real {
    let exact_zero = y == 0.0;
    let good = |x: Real| x.is_finite() && (exact_zero || x.abs() >= Real::MIN_POSITIVE);
    let candidates = [
        rescale(y * equil, rs, scale),
        rescale(y, rs, scale) * equil,
        rescale(y, equil, scale) * rs,
        rescale(y, rs, scale / equil),
    ];
    for c in candidates {
        if good(c) {
            return c;
        }
    }
    if candidates[0].is_finite() {
        candidates[0]
    } else {
        candidates[1]
    }
}
fn cutoff(n: usize) -> Real {
    64.0 * f64::EPSILON * (n.max(1) as Real)
}

struct Ldlt {
    n: usize,
    l: Vec<Real>,
    d: Vec<Real>,
    off: Vec<Real>,
    blocks: Vec<usize>,
    permutation: Vec<usize>,
    /// Equilibrado diagonal `s` (B = S A S) previo a factorizar.
    equil: Vec<Real>,
    scale: Real,
    info: FactorInfo,
}
// El factor se devuelve en un Box: su cabecera también ocupa heap, incluso n=0.
pub(crate) const FACTOR_HEADER_BYTES: usize = std::mem::size_of::<Ldlt>();

impl Ldlt {
    fn new(matrix: &SymMatrix) -> EResult<Self> {
        let n = matrix.order();
        let equil = equilibration(matrix.dense().values(), n);
        let balanced: Vec<Real> = (0..n * n)
            .map(|k| matrix.dense().values()[k] * equil[k / n] * equil[k % n])
            .collect();
        let scale = magnitude(&balanced);
        let mut a: Vec<_> = balanced.iter().map(|v| v / scale).collect();
        let mut f = Self {
            equil,
            n,
            l: identity(n)?.values,
            d: vec![0.0; n],
            off: vec![0.0; n],
            // Como máximo hay n pivotes 1×1; evitar crecimiento y capacidad sobrante.
            blocks: Vec::with_capacity(n),
            permutation: (0..n).collect(),
            scale,
            info: FactorInfo {
                inertia: Some((0, 0, 0)),
                min_pivot_ratio: None,
                singular_dofs: vec![],
            },
        };
        let mut smallest = f64::INFINITY;
        let mut largest = 0.0_f64;
        let mut k = 0;
        while k < n {
            let diag = (k..n)
                .max_by(|&i, &j| a[i * n + i].abs().total_cmp(&a[j * n + j].abs()))
                .unwrap();
            let mut pair = (k, k);
            let mut offmax = 0.0;
            for i in k..n {
                for j in k..i {
                    if a[i * n + j].abs() > offmax {
                        offmax = a[i * n + j].abs();
                        pair = (j, i);
                    }
                }
            }
            let block = if a[diag * n + diag].abs() >= 0.5 * offmax {
                1
            } else {
                2
            };
            if block == 1 {
                f.swap(&mut a, k, diag, k);
                let d = a[k * n + k];
                if d.abs() <= cutoff(n) {
                    // Todo el bloque restante es numéricamente nulo: el GDL culpable
                    // es el de menor índice original, independiente del orden de
                    // pivoteo (determinista bajo permutación de GDL).
                    let culprit = f.permutation[k..].iter().copied().min().unwrap();
                    return Err(singular("Referencia LDLᵀ", culprit));
                }
                f.d[k] = d;
                f.count(d, &mut smallest, &mut largest);
                for i in k + 1..n {
                    f.l[i * n + k] = a[i * n + k] / d;
                }
            } else {
                f.swap(&mut a, k, pair.0, k);
                f.swap(&mut a, k + 1, pair.1, k);
                let (d, e, g) = (a[k * n + k], a[(k + 1) * n + k], a[(k + 1) * n + k + 1]);
                let det = d * g - e * e;
                if det.abs() <= cutoff(n) * offmax {
                    // Menor índice original del par, independiente del orden de pivoteo.
                    let culprit = f.permutation[k].min(f.permutation[k + 1]);
                    return Err(singular("Referencia LDLᵀ bloque", culprit));
                }
                f.d[k] = d;
                f.d[k + 1] = g;
                f.off[k] = e;
                let half = (d + g) * 0.5;
                let radius = ((d - g) * 0.5).hypot(e);
                f.count(half + radius, &mut smallest, &mut largest);
                f.count(half - radius, &mut smallest, &mut largest);
                for i in k + 2..n {
                    let (v, w) = (a[i * n + k], a[i * n + k + 1]);
                    f.l[i * n + k] = (v * g - w * e) / det;
                    f.l[i * n + k + 1] = (w * d - v * e) / det;
                }
            }
            for i in k + block..n {
                for j in k + block..=i {
                    let mut update = f.l[i * n + k] * f.d[k] * f.l[j * n + k];
                    if block == 2 {
                        update += f.off[k]
                            * (f.l[i * n + k] * f.l[j * n + k + 1]
                                + f.l[i * n + k + 1] * f.l[j * n + k])
                            + f.l[i * n + k + 1] * f.d[k + 1] * f.l[j * n + k + 1];
                    }
                    a[i * n + j] -= update;
                    a[j * n + i] = a[i * n + j];
                }
            }
            finite(&a)?;
            f.blocks.push(block);
            k += block;
        }
        if n > 0 {
            f.info.min_pivot_ratio = Some(smallest / largest);
        }
        Ok(f)
    }
    fn swap(&mut self, a: &mut [Real], i: usize, j: usize, done: usize) {
        for col in 0..self.n {
            a.swap(i * self.n + col, j * self.n + col);
        }
        for row in 0..self.n {
            a.swap(row * self.n + i, row * self.n + j);
        }
        for col in 0..done {
            self.l.swap(i * self.n + col, j * self.n + col);
        }
        self.permutation.swap(i, j);
    }
    fn count(&mut self, value: Real, min: &mut Real, max: &mut Real) {
        let inertia = self.info.inertia.as_mut().unwrap();
        if value > 0.0 {
            inertia.0 += 1;
        } else {
            inertia.1 += 1;
        }
        *min = min.min(value.abs());
        *max = max.max(value.abs());
    }
}
impl SymFactorization for Ldlt {
    fn info(&self) -> &FactorInfo {
        &self.info
    }
    fn solve_in_place(&self, b: &mut DMat) -> EResult<()> {
        size(b.rows(), self.n)?;
        if self.n == 0 {
            // Sin incógnitas no hay nada que resolver: evita recorrer `cols`
            // columnas (con B 0×usize::MAX el bucle no terminaba).
            return Ok(());
        }
        let mut x = b.clone();
        for rhs in 0..b.cols() {
            // La escala de la RHS es la de b (sin equilibrar): b·s puede desbordar
            // aunque la solución sea finita (J4 r3).
            let rhs_scale = magnitude_iter((0..self.n).map(|i| b.values()[i * b.cols() + rhs]));
            let mut y: Vec<_> = self
                .permutation
                .iter()
                .map(|&i| (b.values()[i * b.cols() + rhs] / rhs_scale) * self.equil[i])
                .collect();
            for i in 0..self.n {
                for j in 0..i {
                    y[i] -= self.l[i * self.n + j] * y[j];
                }
            }
            let mut k = 0;
            for &block in &self.blocks {
                if block == 1 {
                    y[k] /= self.d[k];
                } else {
                    // Bloque normalizado por su mayor entrada: evita productos
                    // diminutos que subdesbordan (J4 ENG-LA-007, medio).
                    let m = self.d[k]
                        .abs()
                        .max(self.d[k + 1].abs())
                        .max(self.off[k].abs());
                    let (d0, d1, e) = (self.d[k] / m, self.d[k + 1] / m, self.off[k] / m);
                    let det = d0 * d1 - e * e;
                    let (v, w) = (y[k] / m, y[k + 1] / m);
                    y[k] = (d1 * v - e * w) / det;
                    y[k + 1] = (d0 * w - e * v) / det;
                }
                k += block;
            }
            for i in (0..self.n).rev() {
                for j in i + 1..self.n {
                    y[i] -= self.l[j * self.n + i] * y[j];
                }
            }
            for (i, &v) in y.iter().enumerate() {
                let orig = self.permutation[i];
                x.values[orig * b.cols() + rhs] =
                    scale_back(v, self.equil[orig], rhs_scale, self.scale);
            }
        }
        finite(&x.values)?;
        *b = x;
        Ok(())
    }
}

pub(crate) fn cholesky(m: &SymMatrix) -> EResult<DMat> {
    let n = m.order();
    let mut l = DMat::zeros(n, n)?;
    let equil = equilibration(m.dense().values(), n);
    let balanced: Vec<Real> = (0..n * n)
        .map(|k| m.dense().values()[k] * equil[k / n] * equil[k % n])
        .collect();
    let scale = magnitude(&balanced);
    for i in 0..n {
        for j in 0..=i {
            let mut value = balanced[i * n + j] / scale;
            for k in 0..j {
                value -= l.values[i * n + k] * l.values[j * n + k];
            }
            if i == j {
                if value.abs() <= cutoff(n) || value < 0.0 {
                    // Un pivote Cholesky nulo también puede pertenecer a una
                    // masa invertible indefinida; LDLᵀ distingue ambos casos.
                    return match Ldlt::new(m) {
                        Err(error) => Err(error),
                        Ok(factor)
                            if factor
                                .info
                                .inertia
                                .is_some_and(|(_, negative, _)| negative == 0) =>
                        {
                            Err(singular(
                                "Cholesky de masa: pivote numéricamente insuficiente",
                                i,
                            ))
                        }
                        Ok(_) => Err(invalid(&format!(
                            "Cholesky de masa: masa no definida positiva, GDL {i}."
                        ))),
                    };
                }
                l.values[i * n + j] = value.sqrt();
            } else {
                l.values[i * n + j] = value / l.values[j * n + j];
            }
        }
    }
    // M = S⁻¹ B S⁻¹ con B = L_B L_Bᵀ ⇒ L = S⁻¹ L_B (filas), más la escala global.
    for (i, &e) in equil.iter().enumerate() {
        for j in 0..n {
            l.values[i * n + j] = l.values[i * n + j] * scale.sqrt() / e;
        }
    }
    finite(&l.values)?;
    Ok(l)
}

fn jacobi(matrix: &SymMatrix) -> EResult<(Vec<Real>, DMat)> {
    let n = matrix.order();
    let scale = magnitude(matrix.dense().values());
    let mut a: Vec<_> = matrix.dense().values().iter().map(|v| v / scale).collect();
    let mut q = identity(n)?;
    let iterations = 100usize
        .checked_mul(n)
        .and_then(|v| v.checked_mul(n))
        .ok_or_else(|| invalid("Iteraciones Jacobi desbordadas."))?;
    let mut converged = n < 2;
    for _ in 0..iterations {
        let mut p = 0;
        let mut r = 0;
        let mut max = 0.0;
        for i in 0..n {
            for j in 0..i {
                if a[i * n + j].abs() > max {
                    max = a[i * n + j].abs();
                    p = j;
                    r = i;
                }
            }
        }
        if max <= 4.0 * f64::EPSILON * (n.max(1) as Real) {
            converged = true;
            break;
        }
        let tau = (a[r * n + r] - a[p * n + p]) / (2.0 * a[p * n + r]);
        let t = if tau >= 0.0 { 1.0 } else { -1.0 } / (tau.abs() + tau.hypot(1.0));
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;
        let ap = a[p * n + p];
        let ar = a[r * n + r];
        let off = a[p * n + r];
        a[p * n + p] = ap - t * off;
        a[r * n + r] = ar + t * off;
        a[p * n + r] = 0.0;
        a[r * n + p] = 0.0;
        for i in 0..n {
            if i != p && i != r {
                let (v, w) = (a[i * n + p], a[i * n + r]);
                a[i * n + p] = c * v - s * w;
                a[p * n + i] = a[i * n + p];
                a[i * n + r] = s * v + c * w;
                a[r * n + i] = a[i * n + r];
            }
            let (v, w) = (q.values[i * n + p], q.values[i * n + r]);
            q.values[i * n + p] = c * v - s * w;
            q.values[i * n + r] = s * v + c * w;
        }
    }
    if !converged {
        return Err(EngineError::new(
            "ENG-E-0007",
            "Referencia Jacobi: límite de rotaciones agotado.",
        ));
    }
    let mut order: Vec<_> = (0..n).collect();
    order.sort_by(|&i, &j| a[i * n + i].total_cmp(&a[j * n + j]));
    let values = order.iter().map(|&i| a[i * n + i] * scale).collect();
    let sorted = DMat::new(
        n,
        n,
        (0..n)
            .flat_map(|i| {
                order.iter().map({
                    let q = &q;
                    move |&j| q.values[i * n + j]
                })
            })
            .collect(),
    )?;
    Ok((values, sorted))
}

pub(crate) fn normalize_modes(
    eigenvalues: Vec<Real>,
    mut vectors: DMat,
    m: &SymMatrix,
) -> EResult<GeneralizedEigen> {
    finite(&eigenvalues)?;
    let n = m.order();
    for col in 0..n {
        let mut norm = 0.0;
        for i in 0..n {
            for j in 0..n {
                norm += vectors.values[i * n + col]
                    * m.dense().values()[i * n + j]
                    * vectors.values[j * n + col];
            }
        }
        if norm <= 0.0 || !norm.is_finite() {
            return Err(EngineError::new(
                "ENG-E-0008",
                "Normalización modal: norma de masa inválida.",
            ));
        }
        let pivot = (0..n)
            .max_by(|&i, &j| {
                vectors.values[i * n + col]
                    .abs()
                    .total_cmp(&vectors.values[j * n + col].abs())
            })
            .unwrap();
        let divisor = norm.sqrt()
            * if vectors.values[pivot * n + col] < 0.0 {
                -1.0
            } else {
                1.0
            };
        for row in 0..n {
            vectors.values[row * n + col] /= divisor;
        }
    }
    finite(&vectors.values)?;
    Ok(GeneralizedEigen {
        eigenvalues,
        vectors,
    })
}

#[cfg(test)]
mod rescale_tests {
    use super::rescale;
    #[test]
    fn avoids_intermediate_underflow_and_overflow() {
        // v·num = 1e-400 subdesborda aunque el resultado real (1e-100) es normal.
        let r = rescale(1e-200, 1e-200, 1e-300);
        assert!((r / 1e-100 - 1.0).abs() < 1e-12, "{r}");
        // v·num = 1e400 desborda aunque el resultado real (1e100) es normal.
        let r = rescale(1e200, 1e200, 1e300);
        assert!((r / 1e100 - 1.0).abs() < 1e-12, "{r}");
        // Caso ordinario y ceros exactos.
        assert_eq!(rescale(3.0, 4.0, 2.0), 6.0);
        assert_eq!(rescale(0.0, 4.0, 2.0), 0.0);
        // Resultado no representable: queda infinito (el llamador da ENG-E-0008).
        assert!(!rescale(1e300, 1e300, 1e-300).is_finite());
    }
}
