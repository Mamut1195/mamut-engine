//! ENG-LA-008: cumulative memory budget.
use engine_linalg::{dense_bytes, sparse_bytes, DMat, MemoryBudget, ReferenceBackend};

const USZ: usize = std::mem::size_of::<usize>();

fn dmat(r: usize, c: usize, v: &[f64]) -> DMat {
    DMat::new(r, c, v.to_vec()).unwrap()
}

#[test]
fn many_individually_allowed_allocations_exceed_the_cumulative_budget() {
    // Each matrix (8·64·64 = 32 KiB) fits alone; their sum does not fit in 100 KiB.
    let one = dense_bytes(64, 64).unwrap();
    assert_eq!(one, 32 * 1024);
    let mut budget = MemoryBudget::new(100 * 1024);
    let mut held = vec![];
    for _ in 0..3 {
        held.push(budget.zeros(64, 64).unwrap());
    }
    assert_eq!(budget.used(), 3 * one);
    let e = budget.zeros(64, 64).unwrap_err();
    assert_eq!(e.code, "ENG-E-0013");
    assert!(!e.message.is_empty());
    // Failure does not alter accounting; releasing allows allocation again.
    assert_eq!(budget.used(), 3 * one);
    budget.release_dense(&held.pop().unwrap());
    assert!(budget.zeros(64, 64).is_ok());
    assert_eq!(budget.peak(), 3 * one);
    assert!(budget.remaining() <= budget.limit());
}

#[test]
fn charge_release_and_overflow_edges() {
    let mut b = MemoryBudget::new(10);
    assert!(b.charge(10).is_ok());
    assert_eq!(b.charge(1).unwrap_err().code, "ENG-E-0013");
    assert_eq!(b.charge(usize::MAX).unwrap_err().code, "ENG-E-0013");
    assert_eq!(b.used(), 10);
    b.release(100); // excess release saturates at 0
    assert_eq!(b.used(), 0);
    assert!(b.charge(0).is_ok());
    assert_eq!(
        MemoryBudget::new(0).charge(1).unwrap_err().code,
        "ENG-E-0013"
    );
    assert!(MemoryBudget::for_target().limit() >= 1 << 30);
    assert_eq!(dense_bytes(usize::MAX, 2), None);
    assert_eq!(dense_bytes(0, usize::MAX), Some(0));
    assert_eq!(sparse_bytes(usize::MAX, 0), None);
    assert_eq!(sparse_bytes(2, 3), Some(3 * USZ + 3 * (USZ + 8)));
    assert_eq!(
        MemoryBudget::new(usize::MAX)
            .zeros(usize::MAX, 2)
            .unwrap_err()
            .code,
        "ENG-E-0010"
    );
}

#[test]
fn solve_dense_charges_working_set_and_keeps_only_the_result() {
    let a = dmat(2, 2, &[2.0, 0.0, 0.0, 4.0]);
    let b = dmat(2, 1, &[6.0, 8.0]);
    let need = dense_bytes(2, 2).unwrap() + 2 * dense_bytes(2, 1).unwrap();
    // One byte less than the working set: rejected without touching accounting.
    let mut tight = MemoryBudget::new(need - 1);
    assert_eq!(
        tight
            .solve_dense(&mut ReferenceBackend, &a, &b)
            .unwrap_err()
            .code,
        "ENG-E-0013"
    );
    assert_eq!(tight.used(), 0);
    let mut ok = MemoryBudget::new(need);
    let x = ok.solve_dense(&mut ReferenceBackend, &a, &b).unwrap();
    assert_eq!(x.values(), &[3.0, 2.0]);
    assert_eq!(ok.used(), dense_bytes(2, 1).unwrap());
    assert_eq!(ok.peak(), need);
    ok.release_dense(&x);
    assert_eq!(ok.used(), 0);
    // Backend error (singular matrix): accounting returns to zero.
    let sing = dmat(2, 2, &[1.0, 1.0, 1.0, 1.0]);
    let e = ok
        .solve_dense(&mut ReferenceBackend, &sing, &b)
        .unwrap_err();
    assert_eq!(e.code, "ENG-E-0006");
    assert_eq!(ok.used(), 0);
}

#[test]
fn empty_system_with_huge_column_count_does_not_panic() {
    // J4 r3: A 0×0 with B 0×usize::MAX charged 0 bytes and the backend panicked
    // when allocating vectors per column.
    let a = DMat::zeros(0, 0).unwrap();
    let b = DMat::zeros(0, usize::MAX).unwrap();
    let mut budget = MemoryBudget::new(1 << 20);
    let x = budget.solve_dense(&mut ReferenceBackend, &a, &b).unwrap();
    assert_eq!((x.rows(), x.cols()), (0, usize::MAX));
    assert_eq!(budget.used(), 0);
    // The 0×n variant with large n but nonempty memory also does not panic.
    let b2 = DMat::zeros(0, 1 << 30).unwrap();
    assert!(budget.solve_dense(&mut ReferenceBackend, &a, &b2).is_ok());
}

/// Backend that panics in solve_dense (simulates an uncaught failure).
struct Panicking;
impl engine_linalg::LinAlgBackend for Panicking {
    fn name(&self) -> &'static str {
        "panicking"
    }
    fn matmul(&mut self, _: &DMat, _: &DMat) -> engine_core::EResult<DMat> {
        unreachable!()
    }
    fn transpose(&mut self, _: &DMat) -> engine_core::EResult<DMat> {
        unreachable!()
    }
    fn solve_dense(&mut self, _: &DMat, _: &DMat) -> engine_core::EResult<DMat> {
        panic!("fallo simulado del backend");
    }
    fn factorize(
        &mut self,
        _: &engine_linalg::SymMatrix,
    ) -> engine_core::EResult<Box<dyn engine_linalg::SymFactorization>> {
        unreachable!()
    }
    fn generalized_eigen(
        &mut self,
        _: &engine_linalg::SymMatrix,
        _: &engine_linalg::SymMatrix,
    ) -> engine_core::EResult<engine_linalg::GeneralizedEigen> {
        unreachable!()
    }
}

#[test]
fn recovered_backend_panic_does_not_leave_budget_charged() {
    let a = dmat(2, 2, &[1.0, 0.0, 0.0, 1.0]);
    let b = dmat(2, 1, &[1.0, 1.0]);
    let mut budget = MemoryBudget::new(1 << 20);
    budget.charge(100).unwrap();
    let before = budget.used();
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = budget.solve_dense(&mut Panicking, &a, &b);
    }));
    assert!(r.is_err(), "el pánico se propaga al llamador");
    assert_eq!(
        budget.used(),
        before,
        "used no queda cargado tras el pánico"
    );
    assert!(
        budget.peak() > before,
        "peak documenta el conjunto de trabajo cargado"
    );
}

#[test]
fn error_order_does_not_depend_on_the_budget() {
    // Incompatible shape: same error with a generous or tiny budget.
    let a = dmat(2, 2, &[1.0, 0.0, 0.0, 1.0]);
    let b = dmat(3, 1, &[1.0, 1.0, 1.0]);
    let big = MemoryBudget::new(1 << 30)
        .solve_dense(&mut ReferenceBackend, &a, &b)
        .unwrap_err();
    let tiny = MemoryBudget::new(1)
        .solve_dense(&mut ReferenceBackend, &a, &b)
        .unwrap_err();
    assert_eq!(big.code, tiny.code);
    // With valid shape and insufficient budget: ENG-E-0013.
    let ok_b = dmat(2, 1, &[1.0, 1.0]);
    let e = MemoryBudget::new(1)
        .solve_dense(&mut ReferenceBackend, &a, &ok_b)
        .unwrap_err();
    assert_eq!(e.code, "ENG-E-0013");
}

fn sym(n: usize, diag: f64) -> engine_linalg::SymMatrix {
    let mut v = vec![0.0; n * n];
    for i in 0..n {
        v[i * n + i] = diag;
    }
    engine_linalg::SymMatrix::new(dmat(n, n, &v)).unwrap()
}

#[test]
fn factorize_is_charged_keeps_only_the_factorization_and_respects_the_limit() {
    use engine_linalg::{factor_bytes, LinAlgBackend};
    let a = sym(8, 4.0);
    let retained = factor_bytes(8).unwrap();
    let mut budget = MemoryBudget::new(1 << 20);
    let f = budget.factorize(&mut ReferenceBackend, &a).unwrap();
    assert_eq!(budget.used(), retained);
    assert!(
        budget.peak() > retained,
        "peak incluye el trabajo transitorio"
    );
    let mut b = dmat(8, 1, &[1.0; 8]);
    f.solve_in_place(&mut b).unwrap();
    assert!((b.values()[0] - 0.25).abs() < 1e-14);
    budget.release(retained);
    assert_eq!(budget.used(), 0);
    // Insufficient budget: ENG-E-0013 without altering `used`.
    let mut tiny = MemoryBudget::new(retained);
    assert_eq!(
        tiny.factorize(&mut ReferenceBackend, &a)
            .err()
            .unwrap()
            .code,
        "ENG-E-0013"
    );
    assert_eq!(tiny.used(), 0);
    // Backend error (singular): `used` intact.
    let singular = sym(4, 0.0);
    let mut budget = MemoryBudget::new(1 << 20);
    assert!(budget.factorize(&mut ReferenceBackend, &singular).is_err());
    assert_eq!(budget.used(), 0);
    let _ = ReferenceBackend.name();
}

#[test]
fn generalized_eigen_is_charged_and_respects_the_limit() {
    use engine_linalg::eigen_working_bytes;
    let k = sym(6, 3.0);
    let m = sym(6, 1.0);
    let work = eigen_working_bytes(6).unwrap();
    let mut budget = MemoryBudget::new(1 << 22);
    let r = budget
        .generalized_eigen(&mut ReferenceBackend, &k, &m)
        .unwrap();
    assert_eq!(r.eigenvalues.len(), 6);
    assert!((r.eigenvalues[0] - 3.0).abs() < 1e-12);
    assert_eq!(budget.used(), dense_bytes(6, 6).unwrap() + 6 * 8);
    assert!(budget.peak() >= work);
    let mut small = MemoryBudget::new(work);
    assert_eq!(
        small
            .generalized_eigen(&mut ReferenceBackend, &k, &m)
            .err()
            .unwrap()
            .code,
        "ENG-E-0013"
    );
    assert_eq!(small.used(), 0);
    // Different orders: the error does not depend on budget.
    let m3 = sym(3, 1.0);
    let a = MemoryBudget::new(1)
        .generalized_eigen(&mut ReferenceBackend, &k, &m3)
        .err()
        .unwrap();
    let b = MemoryBudget::new(1 << 30)
        .generalized_eigen(&mut ReferenceBackend, &k, &m3)
        .err()
        .unwrap();
    assert_eq!(a.code, b.code);
}
