use spike_la::*;
use std::time::Instant;
fn med<F: FnMut() -> Vec<f64>>(mut f: F, reps: usize) -> (f64, f64, Vec<f64>) {
    let mut out = f(); // calentamiento
    let mut t = vec![];
    for _ in 0..reps { let s = Instant::now(); out = f(); t.push(s.elapsed().as_secs_f64()*1e3); }
    t.sort_by(|a,b| a.partial_cmp(b).unwrap());
    (t[t.len()/2], t[0], out)
}
fn main() {
    println!("hilos_disponibles={} RAYON_NUM_THREADS={:?}", std::thread::available_parallelism().unwrap(), std::env::var("RAYON_NUM_THREADS"));
    println!("n,op,lib,mediana_ms,min_ms,residual_rel,hash");
    for &n in &[50usize, 200, 1000] {
        let reps = if n == 1000 { 5 } else if n == 200 { 50 } else { 500 };
        let (a, b) = (spd(n), rhs(n));
        type F = fn(&[f64], &[f64], usize) -> Vec<f64>;
        let cases: [(&str,&str,F);4] = [("cholesky","faer",faer_chol),("cholesky","nalgebra",na_chol),("lu","faer",faer_lu),("lu","nalgebra",na_lu)];
        for (op, lib, f) in cases {
            let (m, mn, x) = med(|| f(&a,&b,n), reps);
            println!("{n},{op},{lib},{m:.4},{mn:.4},{:.2e},{:016x}", resid(&a,&x,&b,n), fnv(&x));
        }
    }
}
