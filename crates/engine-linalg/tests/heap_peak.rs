//! Regresión J4: cotas del presupuesto frente a bytes de heap realmente vivos.
//! Cada medición excluye las entradas, construidas antes de reiniciar el pico.
//! El contador mide tamaños de Layout, sin metadatos internos de System.
//! Solo cuenta el hilo medidor; la actividad de otros hilos de libtest se ignora.
use engine_linalg::{
    dense_bytes, eigen_working_bytes, factor_bytes, DMat, MemoryBudget, ReferenceBackend, SymMatrix,
};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::sync::atomic::{AtomicUsize, Ordering::SeqCst};
use std::sync::Mutex;

struct CountingAllocator;

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static MEASUREMENT: Mutex<()> = Mutex::new(());

thread_local! {
    // Inicialización constante y sin destructor: acceder desde GlobalAlloc no
    // reserva memoria ni necesita inicialización perezosa que pueda reentrar.
    static COUNTING: Cell<bool> = const { Cell::new(false) };
}

struct CountingThread;

impl CountingThread {
    fn enter() -> Self {
        COUNTING.with(|counting| {
            assert!(!counting.replace(true), "medición anidada");
        });
        Self
    }
}

impl Drop for CountingThread {
    fn drop(&mut self) {
        // También se desactiva al desenrollar un pánico de la prueba.
        let _ = COUNTING.try_with(|counting| counting.set(false));
    }
}

fn when_counting(update: impl FnOnce()) {
    // try_with ignora TLS no disponible durante la destrucción del hilo.
    // Desactivar mientras se actualiza protege también frente a reentrada.
    let _ = COUNTING.try_with(|counting| {
        if counting.replace(false) {
            update();
            counting.set(true);
        }
    });
}

fn allocated(bytes: usize) {
    let live = LIVE.fetch_add(bytes, SeqCst) + bytes;
    PEAK.fetch_max(live, SeqCst);
}

fn deallocated(bytes: usize) {
    // Una liberación fuera del intervalo contado nunca puede hacer underflow.
    let mut live = LIVE.load(SeqCst);
    while let Err(current) =
        LIVE.compare_exchange_weak(live, live.saturating_sub(bytes), SeqCst, SeqCst)
    {
        live = current;
    }
}

// SAFETY: se delegan punteros y Layout intactos a System, respetando el contrato
// de GlobalAlloc. Solo se cuentan reservas exitosas y liberaciones válidas.
// TLS constante, Cell y contadores atómicos no reservan memoria. La guarda de
// reentrada excluye llamadas anidadas y try_with tolera TLS destruido.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: el llamador garantiza un Layout válido; System posee la reserva.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            when_counting(|| allocated(layout.size()));
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: mismo contrato que alloc, con inicialización delegada a System.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            when_counting(|| allocated(layout.size()));
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: el llamador aporta la reserva viva y su Layout original.
        unsafe { System.dealloc(pointer, layout) };
        when_counting(|| deallocated(layout.size()));
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: reserva y tamaño válidos según el contrato de GlobalAlloc.
        let resized = unsafe { System.realloc(pointer, layout, new_size) };
        if !resized.is_null() {
            when_counting(|| {
                if new_size >= layout.size() {
                    allocated(new_size.saturating_sub(layout.size()));
                } else {
                    deallocated(layout.size().saturating_sub(new_size));
                }
            });
        }
        // Si falla, la reserva original sigue viva y la cuenta queda intacta.
        resized
    }
}

fn measure<T>(operation: impl FnOnce() -> T) -> (T, usize, usize) {
    let baseline = LIVE.load(SeqCst);
    PEAK.store(baseline, SeqCst);
    let result = operation();
    let peak = PEAK.load(SeqCst).saturating_sub(baseline);
    let retained = LIVE.load(SeqCst).saturating_sub(baseline);
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
fn releasing_preexisting_storage_has_zero_measured_delta() {
    let _serial = MEASUREMENT.lock().unwrap();
    let _counting = CountingThread::enter();
    let existing = vec![0_u8; 64];
    std::hint::black_box(&existing);
    // LIVE baja legítimamente de la base; la diferencia debe saturarse en cero.
    let (_, peak, retained) = measure(|| drop(existing));
    assert_eq!((peak, retained), (0, 0));
}

#[test]
fn allocations_on_other_threads_do_not_change_measured_counters() {
    use std::sync::atomic::AtomicBool;

    let _serial = MEASUREMENT.lock().unwrap();
    let start = AtomicBool::new(false);
    std::thread::scope(|scope| {
        // Crear el hilo antes de activar el contador excluye su infraestructura.
        let worker = scope.spawn(|| {
            while !start.load(SeqCst) {
                std::thread::yield_now();
            }
            // Ejercita alloc, alloc_zeroed, realloc (crecer/reducir) y dealloc.
            let boxed = Box::new(42_usize);
            let mut values = vec![0_u8; 64];
            values.reserve_exact(128);
            values.truncate(16);
            values.shrink_to_fit();
            std::hint::black_box((&boxed, &values));
        });
        let counting = CountingThread::enter();
        let live = LIVE.load(SeqCst);
        // Reiniciar evita que un pico anterior oculte las reservas del otro hilo.
        PEAK.store(live, SeqCst);
        start.store(true, SeqCst);
        while !worker.is_finished() {
            std::thread::yield_now();
        }
        assert_eq!(LIVE.load(SeqCst), live);
        assert_eq!(PEAK.load(SeqCst), live);
        drop(counting);
        worker.join().unwrap();
    });
}

#[test]
fn charged_bounds_cover_actual_factorize_and_eigen_heap_peaks() {
    // El Mutex serializa los hilos medidores si se añaden más tests. TLS excluye
    // otros hilos de libtest; se cuentan entradas y drops para conservar la base.
    let _serial = MEASUREMENT.lock().unwrap();
    let _counting = CountingThread::enter();
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
