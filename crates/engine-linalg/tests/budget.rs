//! ENG-LA-008: presupuesto acumulado de memoria.
use engine_linalg::{dense_bytes, sparse_bytes, DMat, MemoryBudget, ReferenceBackend};

const USZ: usize = std::mem::size_of::<usize>();

fn dmat(r: usize, c: usize, v: &[f64]) -> DMat {
    DMat::new(r, c, v.to_vec()).unwrap()
}

#[test]
fn many_individually_allowed_allocations_exceed_the_cumulative_budget() {
    // Cada matriz (8·64·64 = 32 KiB) cabe sola; la suma no cabe en 100 KiB.
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
    // El fallo no altera la cuenta; liberar permite volver a reservar.
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
    b.release(100); // liberar de más satura en 0
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
    // Un byte menos del conjunto de trabajo: rechazado sin tocar la cuenta.
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
    // Error del backend (matriz singular): la cuenta vuelve a cero.
    let sing = dmat(2, 2, &[1.0, 1.0, 1.0, 1.0]);
    let e = ok
        .solve_dense(&mut ReferenceBackend, &sing, &b)
        .unwrap_err();
    assert_eq!(e.code, "ENG-E-0006");
    assert_eq!(ok.used(), 0);
}

#[test]
fn empty_system_with_huge_column_count_does_not_panic() {
    // J4 r3: A 0×0 con B 0×usize::MAX cargaba 0 bytes y el backend paniqueaba
    // por reservar vectores por columna.
    let a = DMat::zeros(0, 0).unwrap();
    let b = DMat::zeros(0, usize::MAX).unwrap();
    let mut budget = MemoryBudget::new(1 << 20);
    let x = budget.solve_dense(&mut ReferenceBackend, &a, &b).unwrap();
    assert_eq!((x.rows(), x.cols()), (0, usize::MAX));
    assert_eq!(budget.used(), 0);
    // La variante 0×n con n grande pero no vacía de memoria tampoco paniquea.
    let b2 = DMat::zeros(0, 1 << 30).unwrap();
    assert!(budget.solve_dense(&mut ReferenceBackend, &a, &b2).is_ok());
}

/// Backend que entra en pánico en solve_dense (simula un fallo no recuperado).
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
