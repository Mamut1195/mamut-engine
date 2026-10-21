//! J4 regression: budget bounds versus actually live heap bytes.
//! Each measurement excludes inputs, constructed before resetting the peak.
//! The counter measures Layout sizes, without System internal metadata.
//! Only counts the measuring thread; activity of other libtest threads is ignored.
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
    // Constant initialization without a destructor: access from GlobalAlloc does not
    // allocate memory or need lazy initialization that could reenter.
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
        // Also deactivates during test panic unwinding.
        let _ = COUNTING.try_with(|counting| counting.set(false));
    }
}

fn when_counting(update: impl FnOnce()) {
    // try_with ignores TLS unavailable during thread destruction.
    // Deactivating during updates also protects against reentry.
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
    // A release outside the counted interval can never cause underflow.
    let mut live = LIVE.load(SeqCst);
    while let Err(current) =
        LIVE.compare_exchange_weak(live, live.saturating_sub(bytes), SeqCst, SeqCst)
    {
        live = current;
    }
}

// SAFETY: unchanged pointers and Layout are delegated to System, respecting the
// GlobalAlloc contract. Only successful allocations and valid releases are counted.
// Constant TLS, Cell, and atomic counters do not allocate memory. The reentry
// guard excludes nested calls and try_with tolerates destroyed TLS.
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: the caller guarantees a valid Layout; System owns the allocation.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            when_counting(|| allocated(layout.size()));
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        // SAFETY: same contract as alloc, with initialization delegated to System.
        let pointer = unsafe { System.alloc_zeroed(layout) };
        if !pointer.is_null() {
            when_counting(|| allocated(layout.size()));
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: the caller supplies the live allocation and its original Layout.
        unsafe { System.dealloc(pointer, layout) };
        when_counting(|| deallocated(layout.size()));
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        // SAFETY: valid allocation and size according to the GlobalAlloc contract.
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
        // On failure, the original allocation remains live and accounting stays intact.
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
    // Strictly dominant symmetric matrix with positive diagonal: SPD, coupled,
    // and deterministic. Input allocation is excluded from measurement.
    let mut values = vec![0.125; n * n];
    for i in 0..n {
        values[i * n + i] = n as f64 + 1.0 + i as f64 / n as f64;
    }
    SymMatrix::new(DMat::new(n, n, values).unwrap()).unwrap()
}

fn eigen_inputs(n: usize) -> (SymMatrix, SymMatrix) {
    // 2×2 SPD blocks: exercise Jacobi rotations and ordering without turning
    // the n=256 case into an O(n⁴) performance test.
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
    // LIVE legitimately falls below baseline; the difference must saturate at zero.
    let (_, peak, retained) = measure(|| drop(existing));
    assert_eq!((peak, retained), (0, 0));
}

#[test]
fn allocations_on_other_threads_do_not_change_measured_counters() {
    use std::sync::atomic::AtomicBool;

    let _serial = MEASUREMENT.lock().unwrap();
    let start = AtomicBool::new(false);
    std::thread::scope(|scope| {
        // Creating the thread before activating the counter excludes its infrastructure.
        let worker = scope.spawn(|| {
            while !start.load(SeqCst) {
                std::thread::yield_now();
            }
            // Exercises alloc, alloc_zeroed, realloc (grow/shrink), and dealloc.
            let boxed = Box::new(42_usize);
            let mut values = vec![0_u8; 64];
            values.reserve_exact(128);
            values.truncate(16);
            values.shrink_to_fit();
            std::hint::black_box((&boxed, &values));
        });
        let counting = CountingThread::enter();
        let live = LIVE.load(SeqCst);
        // Resetting prevents a previous peak from hiding allocations of the other thread.
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
    // The Mutex serializes measuring threads if more tests are added. TLS excludes
    // other libtest threads; entries and drops are counted to preserve the baseline.
    let _serial = MEASUREMENT.lock().unwrap();
    let _counting = CountingThread::enter();
    let mut old_bound_failed = false;
    // Also small/non-power-of-two orders: header and Vec capacity.
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
