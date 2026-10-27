//! Internal procedures derived from elimination, congruences, and rotations.
use crate::*;
use engine_core::math;

/// Second minimal backend to demonstrate interchangeability and validate contracts.
///
/// Dense LDLᵀ factorization with symmetric 1×1/2×2 pivots; eigenvalues through
/// Jacobi and Cholesky mass reduction. O(n²) memory, O(n³) elimination work,
/// and up to O(n⁴) Jacobi search work. Intended for reference and
/// tests, without AMD ordering or a production sparse solver.
/// V1 is sequential: `i,j,k` products, reductions by increasing index, and
/// back substitution by decreasing row. No explicit SIMD or CPU-based
/// selection; tie-breaking and portable scope are in docs/DETERMINISM.md.
#[derive(Default, Debug)]
pub struct ReferenceBackend;

impl LinAlgBackend for ReferenceBackend {
    fn name(&self) -> &'static str {
        "reference.dense"
    }
    fn matmul(&mut self, a: &DMat, b: &DMat) -> EResult<DMat> {
        self.matmul_with_check(a, b, &mut || Ok(()))
    }
    fn matmul_with_check(
        &mut self,
        a: &DMat,
        b: &DMat,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<DMat> {
        check()?;
        size(a.cols(), b.rows())?;
        let mut c = DMat::zeros(a.rows(), b.cols())?;
        for i in 0..a.rows() {
            check()?;
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
        self.solve_dense_with_check(a, b, &mut || Ok(()))
    }
    fn solve_dense_with_check(
        &mut self,
        a: &DMat,
        b: &DMat,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<DMat> {
        check()?;
        size(a.rows(), a.cols())?;
        size(b.rows(), a.rows())?;
        let n = a.rows();
        if n == 0 {
            // No unknowns: the solution is B (empty) regardless of the number of
            // columns; avoids allocating vectors per column (panic with huge cols).
            return Ok(b.clone());
        }
        let scale = magnitude(a.values());
        let mut u: Vec<_> = a.values().iter().map(|v| v / scale).collect();
        let mut x = b.clone();
        let cols = b.cols();
        // Independent scaling per RHS: dividing by A's scale may overflow
        // even if the solution is finite (J4 ENG-LA-007, medium).
        let rhs_scales: Vec<Real> = (0..cols)
            .map(|c| magnitude_iter((0..n).map(|i| b.values()[i * cols + c])))
            .collect();
        for i in 0..n {
            for (c, &rs) in rhs_scales.iter().enumerate() {
                x.values[i * cols + c] /= rs;
            }
        }
        for k in 0..n {
            check()?;
            let pivot = (k..n)
                .max_by(|&i, &j| u[i * n + k].abs().total_cmp(&u[j * n + k].abs()))
                .ok_or_else(|| invalid("Reference solve: no pivot candidate."))?;
            if u[pivot * n + k].abs() <= cutoff(n) {
                return Err(singular("Reference dense solve", k));
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
            check()?;
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
    fn factorize_with_check(
        &mut self,
        a: &SymMatrix,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<Box<dyn SymFactorization>> {
        Ok(Box::new(Ldlt::build_with_check(a, false, check)?))
    }
    fn symmetric_inertia(&mut self, a: &SymMatrix) -> EResult<(usize, usize, usize)> {
        self.symmetric_inertia_with_check(a, &mut || Ok(()))
    }
    fn symmetric_inertia_with_check(
        &mut self,
        a: &SymMatrix,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<(usize, usize, usize)> {
        Ldlt::build_with_check(a, true, check)?
            .info
            .inertia
            .ok_or_else(|| unsupported("Reference LDLᵀ: inertia is unavailable."))
    }
    fn generalized_eigen(&mut self, k: &SymMatrix, m: &SymMatrix) -> EResult<GeneralizedEigen> {
        self.generalized_eigen_with_check(k, m, &mut || Ok(()))
    }
    fn generalized_eigen_with_check(
        &mut self,
        k: &SymMatrix,
        m: &SymMatrix,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<GeneralizedEigen> {
        check()?;
        size(k.order(), m.order())?;
        let l = cholesky_with_check(m, check)?;
        let inverse = self.solve_dense_with_check(&l, &identity(m.order())?, check)?;
        let left = self.matmul_with_check(&inverse, k.dense(), check)?;
        let inverse_t = self.transpose(&inverse)?;
        let mut reduced = self.matmul_with_check(&left, &inverse_t, check)?;
        // A single half determines the symmetric congruence despite rounding.
        for i in 0..m.order() {
            for j in 0..i {
                reduced.values[j * m.order() + i] = reduced.values[i * m.order() + j];
            }
        }
        let (eigenvalues, q) = jacobi(&SymMatrix::new(reduced)?, check)?;
        let vectors = self.matmul_with_check(&inverse_t, &q, check)?;
        normalize_modes_with_check(eigenvalues, vectors, m, check)
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
/// Returns `v * num / den`, avoiding avoidable intermediate overflow;
/// if the real result is unrepresentable, it remains infinite and `finite`
/// reports it as ENG-E-0008.
fn rescale(v: Real, num: Real, den: Real) -> Real {
    let exact_zero = v == 0.0 || num == 0.0;
    let good = |x: Real| x.is_finite() && (exact_zero || x.abs() >= Real::MIN_POSITIVE);
    // Three evaluation orders: each avoids a different intermediate overflow or
    // underflow; the first normal and finite one is chosen.
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
/// Iterative symmetric diagonal balancing (Ruiz-type) `B = S A S`: starts from
/// `s_i = 1` and repeats `s_i ← s_i / √(máx_j |s_i a_ij s_j|)` (zero rows: 1) until
/// all rows of B have maximum ≈ 1 (±1 %) or 40 passes. Makes
/// diagonal and nearly diagonal matrices with very differently scaled DOFs
/// (diag(1e6, 1e-9) → identity) and the `D A D` rescaling of moderately scaled
/// coupled matrices receive the same classification. Does NOT guarantee exact invariance
/// under diagonal rescaling in coupled matrices with extreme ranges (T5, J4 r3).
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
                *si /= math::sqrt(m);
            }
        }
    }
    s
}
/// Undoes balancing and normalizations: `x = s · (y · rs / scale)` avoiding
/// intermediate overflows and underflows (several evaluation orders are tried).
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
    /// Diagonal balancing `s` (B = S A S) before factorization.
    equil: Vec<Real>,
    scale: Real,
    info: FactorInfo,
}
// The factor is returned in a Box: its header also occupies heap, even for n=0.
pub(crate) const FACTOR_HEADER_BYTES: usize = std::mem::size_of::<Ldlt>();

impl Ldlt {
    fn new(matrix: &SymMatrix) -> EResult<Self> {
        Self::build(matrix, false)
    }
    // `allow_null` is only used for inertia; never to provide a solvable factor.
    fn build(matrix: &SymMatrix, allow_null: bool) -> EResult<Self> {
        Self::build_with_check(matrix, allow_null, &mut || Ok(()))
    }
    fn build_with_check(
        matrix: &SymMatrix,
        allow_null: bool,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<Self> {
        check()?;
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
            // There are at most n 1×1 pivots; avoid growth and excess capacity.
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
        // A pivot may consume two columns: threshold 15 ensures that no more
        // than 16 columns pass between checks.
        let mut next_check = 15;
        while k < n {
            if k >= next_check {
                check()?;
                next_check = k + 15;
            }
            let diag = (k..n)
                .max_by(|&i, &j| a[i * n + i].abs().total_cmp(&a[j * n + j].abs()))
                .ok_or_else(|| invalid("LDLᵀ: empty active block."))?;
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
            if allow_null && a[diag * n + diag].abs().max(offmax) <= cutoff(n) {
                let inertia = f.info.inertia.as_mut().ok_or_else(|| {
                    EngineError::new("ENG-E-0012", "LDLᵀ: missing inertia accumulator.")
                })?;
                inertia.2 += n - k;
                smallest = 0.0;
                break;
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
                    // The entire remaining block is numerically zero: the offending DOF
                    // has the smallest original index, independent of pivoting
                    // order (deterministic under DOF permutation).
                    let culprit = f.permutation[k..]
                        .iter()
                        .copied()
                        .min()
                        .ok_or_else(|| invalid("LDLᵀ: empty active permutation."))?;
                    return Err(singular("Reference LDLᵀ", culprit));
                }
                f.d[k] = d;
                f.count(d, &mut smallest, &mut largest)?;
                for i in k + 1..n {
                    f.l[i * n + k] = a[i * n + k] / d;
                }
            } else {
                f.swap(&mut a, k, pair.0, k);
                f.swap(&mut a, k + 1, pair.1, k);
                let (d, e, g) = (a[k * n + k], a[(k + 1) * n + k], a[(k + 1) * n + k + 1]);
                let det = d * g - e * e;
                if det.abs() <= cutoff(n) * offmax {
                    // Smallest original index of the pair, independent of pivoting order.
                    let culprit = f.permutation[k].min(f.permutation[k + 1]);
                    return Err(singular("Reference LDLᵀ block", culprit));
                }
                f.d[k] = d;
                f.d[k + 1] = g;
                f.off[k] = e;
                let half = (d + g) * 0.5;
                let radius = math::hypot((d - g) * 0.5, e);
                // Product of eigenvalues = det. Choosing the one with greatest
                // magnitude first avoids cancellation in half ± radius for the small one.
                // LDLᵀ congruence preserves the sign counts of A.
                let large = half + radius.copysign(half);
                f.count(large, &mut smallest, &mut largest)?;
                f.count(det / large, &mut smallest, &mut largest)?;
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
            f.info.min_pivot_ratio = Some(if largest == 0.0 {
                0.0
            } else {
                smallest / largest
            });
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
    fn count(&mut self, value: Real, min: &mut Real, max: &mut Real) -> EResult<()> {
        finite(&[value])?;
        let inertia =
            self.info.inertia.as_mut().ok_or_else(|| {
                EngineError::new("ENG-E-0012", "LDLᵀ: missing inertia accumulator.")
            })?;
        if value > 0.0 {
            inertia.0 += 1;
        } else if value < 0.0 {
            inertia.1 += 1;
        } else {
            inertia.2 += 1;
        }
        *min = min.min(value.abs());
        *max = max.max(value.abs());
        Ok(())
    }
}
impl SymFactorization for Ldlt {
    fn uses_2x2_pivots(&self) -> Option<bool> {
        Some(self.blocks.contains(&2))
    }
    fn info(&self) -> &FactorInfo {
        &self.info
    }
    fn solve_in_place(&self, b: &mut DMat) -> EResult<()> {
        self.solve_in_place_with_check(b, &mut || Ok(()))
    }
    fn solve_in_place_with_check(
        &self,
        b: &mut DMat,
        check: &mut dyn FnMut() -> EResult<()>,
    ) -> EResult<()> {
        check()?;
        size(b.rows(), self.n)?;
        if self.n == 0 {
            // Without unknowns there is nothing to solve: avoids traversing `cols`
            // columns (with B 0×usize::MAX the loop did not terminate).
            return Ok(());
        }
        let mut x = b.clone();
        for rhs in 0..b.cols() {
            check()?;
            // The RHS scale is that of b (unbalanced): b·s may overflow
            // even if the solution is finite (J4 r3).
            let rhs_scale = magnitude_iter((0..self.n).map(|i| b.values()[i * b.cols() + rhs]));
            let mut y: Vec<_> = self
                .permutation
                .iter()
                .map(|&i| (b.values()[i * b.cols() + rhs] / rhs_scale) * self.equil[i])
                .collect();
            for i in 0..self.n {
                check()?;
                for j in 0..i {
                    y[i] -= self.l[i * self.n + j] * y[j];
                }
            }
            let mut k = 0;
            for &block in &self.blocks {
                if block == 1 {
                    y[k] /= self.d[k];
                } else {
                    // Block normalized by its largest entry: avoids tiny
                    // products that underflow (J4 ENG-LA-007, medium).
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
                check()?;
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
    cholesky_with_check(m, &mut || Ok(()))
}

pub(crate) fn cholesky_with_check(
    m: &SymMatrix,
    check: &mut dyn FnMut() -> EResult<()>,
) -> EResult<DMat> {
    check()?;
    let n = m.order();
    let mut l = DMat::zeros(n, n)?;
    let equil = equilibration(m.dense().values(), n);
    let balanced: Vec<Real> = (0..n * n)
        .map(|k| m.dense().values()[k] * equil[k / n] * equil[k % n])
        .collect();
    let scale = magnitude(&balanced);
    for i in 0..n {
        check()?;
        for j in 0..=i {
            let mut value = balanced[i * n + j] / scale;
            for k in 0..j {
                value -= l.values[i * n + k] * l.values[j * n + k];
            }
            if i == j {
                if value.abs() <= cutoff(n) || value < 0.0 {
                    // A zero Cholesky pivot may also belong to an
                    // invertible indefinite mass; LDLᵀ distinguishes the two cases.
                    return match Ldlt::build_with_check(m, false, check) {
                        Err(error) => Err(error),
                        Ok(factor)
                            if factor
                                .info
                                .inertia
                                .is_some_and(|(_, negative, _)| negative == 0) =>
                        {
                            Err(singular("Mass Cholesky: numerically insufficient pivot", i))
                        }
                        Ok(_) => Err(invalid(&format!(
                            "Mass Cholesky: mass is not positive definite, DOF {i}."
                        ))),
                    };
                }
                l.values[i * n + j] = math::sqrt(value);
            } else {
                l.values[i * n + j] = value / l.values[j * n + j];
            }
        }
    }
    // M = S⁻¹ B S⁻¹ with B = L_B L_Bᵀ ⇒ L = S⁻¹ L_B (rows), plus the global scale.
    for (i, &e) in equil.iter().enumerate() {
        for j in 0..n {
            l.values[i * n + j] = l.values[i * n + j] * math::sqrt(scale) / e;
        }
    }
    finite(&l.values)?;
    Ok(l)
}

fn jacobi(
    matrix: &SymMatrix,
    check: &mut dyn FnMut() -> EResult<()>,
) -> EResult<(Vec<Real>, DMat)> {
    let n = matrix.order();
    let scale = magnitude(matrix.dense().values());
    let mut a: Vec<_> = matrix.dense().values().iter().map(|v| v / scale).collect();
    let mut q = identity(n)?;
    let iterations = 100usize
        .checked_mul(n)
        .and_then(|v| v.checked_mul(n))
        .ok_or_else(|| invalid("Jacobi iteration count overflow."))?;
    let mut converged = n < 2;
    for _ in 0..iterations {
        check()?;
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
        let t = if tau >= 0.0 { 1.0 } else { -1.0 } / (tau.abs() + math::hypot(tau, 1.0));
        let c = 1.0 / math::sqrt(1.0 + t * t);
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
            "Reference Jacobi: rotation limit exhausted.",
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

pub(crate) fn normalize_modes_with_check(
    eigenvalues: Vec<Real>,
    mut vectors: DMat,
    m: &SymMatrix,
    check: &mut dyn FnMut() -> EResult<()>,
) -> EResult<GeneralizedEigen> {
    finite(&eigenvalues)?;
    let n = m.order();
    for col in 0..n {
        check()?;
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
                "Modal normalization: invalid mass norm.",
            ));
        }
        let pivot = (0..n)
            .max_by(|&i, &j| {
                vectors.values[i * n + col]
                    .abs()
                    .total_cmp(&vectors.values[j * n + col].abs())
            })
            .ok_or_else(|| invalid("Modal normalization: empty vector."))?;
        let divisor = math::sqrt(norm)
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
        // v·num = 1e-400 underflows even though the real result (1e-100) is normal.
        let r = rescale(1e-200, 1e-200, 1e-300);
        assert!((r / 1e-100 - 1.0).abs() < 1e-12, "{r}");
        // v·num = 1e400 overflows even though the real result (1e100) is normal.
        let r = rescale(1e200, 1e200, 1e300);
        assert!((r / 1e100 - 1.0).abs() < 1e-12, "{r}");
        // Ordinary case and exact zeros.
        assert_eq!(rescale(3.0, 4.0, 2.0), 6.0);
        assert_eq!(rescale(0.0, 4.0, 2.0), 0.0);
        // Unrepresentable result: remains infinite (the caller returns ENG-E-0008).
        assert!(!rescale(1e300, 1e300, 1e-300).is_finite());
    }
}
