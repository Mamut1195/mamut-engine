//! Spike desechable: factorización de SPD con faer y nalgebra, determinista.
#[cfg(feature = "with-faer")]
use faer::linalg::solvers::Solve;
#[cfg(feature = "with-faer")]
use faer::{Mat, Side};

/// Matriz SPD determinista: A = B*B^T/n + n*I con B por LCG fijo.
pub fn spd(n: usize) -> Vec<f64> {
    let mut s: u64 = 0x9E3779B97F4A7C15 ^ n as u64;
    let mut b = vec![0.0f64; n * n];
    for v in b.iter_mut() {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *v = ((s >> 11) as f64 / (1u64 << 53) as f64) - 0.5;
    }
    let mut a = vec![0.0f64; n * n]; // column-major
    for j in 0..n { for i in 0..=j {
        let mut acc = 0.0;
        for k in 0..n { acc += b[i + k * n] * b[j + k * n]; }
        acc /= n as f64;
        if i == j { acc += n as f64; }
        a[i + j * n] = acc; a[j + i * n] = acc;
    }}
    a
}
pub fn rhs(n: usize) -> Vec<f64> { (0..n).map(|i| 1.0 + (i % 7) as f64).collect() }

#[cfg(feature = "with-faer")]
pub fn faer_chol(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    let m = Mat::<f64>::from_fn(n, n, |i, j| a[i + j * n]);
    let r = Mat::<f64>::from_fn(n, 1, |i, _| b[i]);
    let f = m.llt(Side::Lower).expect("spd");
    let x = f.solve(&r);
    (0..n).map(|i| x[(i, 0)]).collect()
}
#[cfg(feature = "with-faer")]
pub fn faer_lu(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    let m = Mat::<f64>::from_fn(n, n, |i, j| a[i + j * n]);
    let r = Mat::<f64>::from_fn(n, 1, |i, _| b[i]);
    let f = m.partial_piv_lu();
    let x = f.solve(&r);
    (0..n).map(|i| x[(i, 0)]).collect()
}
#[cfg(feature = "with-na")]
pub fn na_chol(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    let m = nalgebra::DMatrix::<f64>::from_column_slice(n, n, a);
    let r = nalgebra::DVector::<f64>::from_column_slice(b);
    let f = m.cholesky().expect("spd");
    f.solve(&r).as_slice().to_vec()
}
#[cfg(feature = "with-na")]
pub fn na_lu(a: &[f64], b: &[f64], n: usize) -> Vec<f64> {
    let m = nalgebra::DMatrix::<f64>::from_column_slice(n, n, a);
    let r = nalgebra::DVector::<f64>::from_column_slice(b);
    m.lu().solve(&r).expect("nonsingular").as_slice().to_vec()
}
pub fn resid(a: &[f64], x: &[f64], b: &[f64], n: usize) -> f64 {
    let (mut rn, mut bn) = (0.0f64, 0.0f64);
    for i in 0..n {
        let mut s = 0.0; for j in 0..n { s += a[i + j * n] * x[j]; }
        rn += (s - b[i]).powi(2); bn += b[i].powi(2);
    }
    (rn / bn).sqrt()
}
pub fn fnv(x: &[f64]) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for v in x { for byte in v.to_bits().to_le_bytes() { h ^= byte as u64; h = h.wrapping_mul(0x100000001b3); } }
    h
}
#[cfg(all(feature = "with-faer", feature = "with-na"))]
pub fn run_all(n: usize) -> [u64; 4] {
    let (a, b) = (spd(n), rhs(n));
    [fnv(&faer_chol(&a,&b,n)), fnv(&faer_lu(&a,&b,n)), fnv(&na_chol(&a,&b,n)), fnv(&na_lu(&a,&b,n))]
}
/// Exportes C para wasm: devuelven hash FNV de la solución y escriben el residuo.
#[cfg(feature = "with-faer")]
#[no_mangle] pub extern "C" fn faer_chol_hash(n: u32) -> u64 { let n=n as usize; let (a,b)=(spd(n),rhs(n)); fnv(&faer_chol(&a,&b,n)) }
#[cfg(feature = "with-faer")]
#[no_mangle] pub extern "C" fn faer_lu_hash(n: u32) -> u64 { let n=n as usize; let (a,b)=(spd(n),rhs(n)); fnv(&faer_lu(&a,&b,n)) }
#[cfg(feature = "with-faer")]
#[no_mangle] pub extern "C" fn faer_chol_resid(n: u32) -> f64 { let n=n as usize; let (a,b)=(spd(n),rhs(n)); resid(&a,&faer_chol(&a,&b,n),&b,n) }
#[cfg(feature = "with-na")]
#[no_mangle] pub extern "C" fn na_chol_hash(n: u32) -> u64 { let n=n as usize; let (a,b)=(spd(n),rhs(n)); fnv(&na_chol(&a,&b,n)) }
#[cfg(feature = "with-na")]
#[no_mangle] pub extern "C" fn na_lu_hash(n: u32) -> u64 { let n=n as usize; let (a,b)=(spd(n),rhs(n)); fnv(&na_lu(&a,&b,n)) }
#[cfg(feature = "with-na")]
#[no_mangle] pub extern "C" fn na_chol_resid(n: u32) -> f64 { let n=n as usize; let (a,b)=(spd(n),rhs(n)); resid(&a,&na_chol(&a,&b,n),&b,n) }
/// Solo generación de datos (para restar de los tiempos wasm).
#[no_mangle] pub extern "C" fn gen_hash(n: u32) -> u64 { let n=n as usize; let (a,b)=(spd(n),rhs(n)); fnv(&a) ^ fnv(&b) }
