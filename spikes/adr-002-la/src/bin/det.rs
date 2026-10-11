use spike_la::*;
fn main() {
    let th: usize = std::env::args().nth(1).map(|s| s.parse().unwrap()).unwrap_or(0);
    #[cfg(feature = "rayon")]
    { if th > 0 { faer::set_global_parallelism(faer::Par::rayon(th)); } else { faer::set_global_parallelism(faer::Par::Seq); } }
    #[cfg(not(feature = "rayon"))]
    { let _ = th; }
    for &n in &[50usize, 200, 1000] {
        let h = run_all(n);
        println!("{n} {:016x} {:016x} {:016x} {:016x}", h[0],h[1],h[2],h[3]);
    }
}
