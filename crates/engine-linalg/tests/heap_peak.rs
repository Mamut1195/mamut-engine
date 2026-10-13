//! Regresión J4: cotas del presupuesto frente a bytes de heap realmente vivos.
//! Cada medición excluye las entradas, construidas antes de reiniciar el pico.
//! El contador mide tamaños de Layout, sin metadatos internos de System.
use engine_linalg::{
    dense_bytes, eigen_working_bytes, factor_bytes, DMat, MemoryBudget, ReferenceBackend, SymMatrix,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::Mutex;

struct CountingAllocator;

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static MEASUREMENT: Mutex<()> = Mutex::new(());

fn allocated(bytes: usize) {
    let live = LIVE.fetch_add(bytes, SeqCst) + bytes;
    PEAK.fetch_max(live, SeqCst);
}

// SAFETY: se delegan punteros y Layout intactos a System, respetando el contrato
// de GlobalAlloc. Solo se cuentan reservas exitosas y liberaciones válidas.
// Los contadores atómicos no reservan memoria ni provocan recursión del allocator.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: el llamador garantiza un Layout válido; System posee la reserva.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            allocated(layout.size());
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: mismo contrato que alloc, con inicialización delegada a System.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            allocated(layout.size());
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: el llamador aporta la reserva viva y su Layout original.
        unsafe { System.dealloc(pointer, layout) };
        LIVE.fetch_sub(layout.size(), SeqCst);
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: reserva y tamaño válidos según el contrato de GlobalAlloc.
        let resized = unsafe { System.realloc(pointer, layout, new_size) };
        if !resized.is_null() {
            if new_size >= layout.size() {
                allocated(new_size - layout.size());
            } else {
                LIVE.fetch_sub(layout.size() - new_size, SeqCst);
            }
        }
        // Si falla, la reserva original sigue viva y la cuenta queda intacta.
        resized
    }
}

fn measure<T>(operation: impl FnOnce() -> T) -> (T, usize, usize) {
    let baseline = LIVE.load(SeqCst);
    PEAK.store(baseline, SeqCst);
    let result = operation();
    let peak = PEAK.load(SeqCst) - baseline;
    let retained = LIVE.load(SeqCst) - baseline;
    (result, peak, retained)
}

fn spd(n: usize) -> SymMatrix {
    // Simétrica estrictamente dominante con diagonal positiva: SPD, acoplada
    // y determinista. La reserva de la entrada queda fuera de la medición.
    let mut values = vec![0.125; n * n];
    for i in 0..n {
        values[i * n + i] = n as f64 + 1.0 + i as f64 / n as f64;
    }
    SymMatrix::new(DMat::new(n, n, values).unwrap()).unwrap()
}

fn eigen_inputs(n: usize) -> (SymMatrix, SymMatrix) {
    // Bloques SPD 2×2: ejercitan rotaciones y ordenamiento Jacobi sin convertir
    // el caso n=256 en una prueba de rendimiento O(n⁴).
    let mut k = vec![0.0; n * n];
    let mut m = vec![0.0; n * n];
    for i in 0..n {
        k[i * n + i] = 2.0 + i as f64;
        m[i * n + i] = 1.0;
        if i % 2 == 1 {
            k[i * n + i - 1] = 0.125;
            k[(i - 1) * n + i] = 0.125;
        }
    }
    (
        SymMatrix::new(DMat::new(n, n, k).unwrap()).unwrap(),
        SymMatrix::new(DMat::new(n, n, m).unwrap()).unwrap(),
    )
}

#[test]
fn charged_bounds_cover_actual_factorize_and_eigen_heap_peaks() {
    // Un único test en este ejecutable evita actividad concurrente del harness;
    // el Mutex serializa todas las mediciones (también si se añaden más tests).
    let _serial = MEASUREMENT.lock().unwrap();
    let mut old_bound_failed = false;
    // También órdenes pequeños/no potencia de dos: cabecera y capacidad de Vec.
    for n in [0, 1, 3, 8, 9, 64, 65, 256] {
        let a = spd(n);
        let retained_bound = factor_bytes(n).unwrap();
        let charged = retained_bound + 2 * dense_bytes(n, n).unwrap();
        let mut budget = MemoryBudget::new(charged);
        let (factor, measured, retained) =
            measure(|| budget.factorize(&mut ReferenceBackend, &a).unwrap());
        assert_eq!(budget.peak(), charged);
        assert_eq!(budget.used(), retained_bound);
        assert!(
            measured <= budget.peak(),
            "factorize n={n}: {measured} > {charged}"
        );
        assert!(
            retained <= budget.used(),
            "factor retenido n={n}: {retained}"
        );
        let old_bound = 2 * n * n * 8 + 40 * n;
        if [8, 64, 256].contains(&n) {
            old_bound_failed |= old_bound < measured;
            eprintln!(
                "factorize n={n}: pico={measured} B, carga={charged} B, cota vieja={old_bound} B"
            );
        }
        drop(factor);
    }
    assert!(
        old_bound_failed,
        "la cota vieja debe subestimar algún pico medido"
    );

    for n in [0, 1, 3, 8, 9, 64, 65, 256] {
        let (k, m) = eigen_inputs(n);
        let result_bound = dense_bytes(n, n).unwrap() + n * 8;
        let charged = eigen_working_bytes(n).unwrap() + result_bound;
        let mut budget = MemoryBudget::new(charged);
        let (result, measured, retained) = measure(|| {
            budget
                .generalized_eigen(&mut ReferenceBackend, &k, &m)
                .unwrap()
        });
        assert_eq!(budget.peak(), charged);
        assert_eq!(budget.used(), result_bound);
        assert!(
            measured <= budget.peak(),
            "generalized_eigen n={n}: {measured} > {charged}"
        );
        assert!(
            retained <= budget.used(),
            "modos retenidos n={n}: {retained}"
        );
        if [8, 64, 256].contains(&n) {
            eprintln!("generalized_eigen n={n}: pico={measured} B, carga={charged} B");
        }
        drop(result);
    }
}
