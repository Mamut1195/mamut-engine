//! Definitive engine types, independent of any numerical library.
use crate::*;

/// Row-major dense matrix for element blocks and multiple RHS.
///
/// The constructor requires length
/// rows × cols and finite data. Units depend on each row/column
/// (e.g. force/displacement for a tangent).
#[derive(Clone, Debug, PartialEq)]
pub struct DMat {
    rows: usize,
    cols: usize,
    pub(crate) values: Vec<Real>,
}
impl DMat {
    /// Constructs from row-major values; ENG-E-0010 for size/overflow
    /// and ENG-E-0008 for nonfinite data. Does not impose symmetry.
    pub fn new(rows: usize, cols: usize, values: Vec<Real>) -> EResult<Self> {
        let len = rows
            .checked_mul(cols)
            .ok_or_else(|| invalid("Tamaño matricial desbordado."))?;
        size(values.len(), len)?;
        finite(&values)?;
        Ok(Self { rows, cols, values })
    }
    /// Creates zero storage; ENG-E-0010 if size overflows or exceeds the
    /// allocation limit (8 GiB), without panicking.
    pub fn zeros(rows: usize, cols: usize) -> EResult<Self> {
        let len = rows
            .checked_mul(cols)
            .ok_or_else(|| invalid("Tamaño matricial desbordado."))?;
        Self::new(rows, cols, alloc_vec(len, 0.0)?)
    }
    /// Number of rows.
    pub fn rows(&self) -> usize {
        self.rows
    }
    /// Number of columns.
    pub fn cols(&self) -> usize {
        self.cols
    }
    /// Immutable data in row-major order.
    pub fn values(&self) -> &[Real] {
        &self.values
    }
    /// Entry (row, column), or None if out of range.
    pub fn get(&self, row: usize, col: usize) -> Option<Real> {
        if row < self.rows && col < self.cols {
            Some(self.values[row * self.cols + col])
        } else {
            None
        }
    }
}

/// Row-major dense symmetric matrix for small problems.
///
/// Stores both halves;
/// requires exact symmetry to avoid introducing an implicit physical tolerance.
#[derive(Clone, Debug, PartialEq)]
pub struct SymMatrix(DMat);
impl SymMatrix {
    /// Validates squareness and exact symmetry; ENG-E-0010 on violation.
    pub fn new(matrix: DMat) -> EResult<Self> {
        size(matrix.rows(), matrix.cols())?;
        for i in 0..matrix.rows() {
            for j in 0..i {
                if matrix.get(i, j) != matrix.get(j, i) {
                    return Err(invalid("Matriz no simétrica."));
                }
            }
        }
        Ok(Self(matrix))
    }
    /// Constructs an n × n zero matrix; ENG-E-0010 on overflow.
    pub fn zeros(n: usize) -> EResult<Self> {
        Self::new(DMat::zeros(n, n)?)
    }
    /// Matrix order.
    pub fn order(&self) -> usize {
        self.0.rows()
    }
    /// Immutable view of dense storage.
    pub fn dense(&self) -> &DMat {
        &self.0
    }
}

/// Symmetric pattern with sorted, unique coordinates.
///
/// The compatible constructor takes a mask: true declares a
/// potentially nonzero entry. Performs no ordering or numerical analysis.
#[derive(Clone, Debug)]
pub struct SparsityPattern {
    order: usize,
    coordinates: Vec<(usize, usize)>,
    mask: std::sync::OnceLock<Vec<bool>>,
}
impl SparsityPattern {
    /// Constructs an n × n symmetric pattern; ENG-E-0010 for size or asymmetry.
    pub fn new(order: usize, entries: Vec<bool>) -> EResult<Self> {
        size(
            entries.len(),
            order
                .checked_mul(order)
                .ok_or_else(|| invalid("Patrón desbordado."))?,
        )?;
        for i in 0..order {
            for j in 0..i {
                if entries[i * order + j] != entries[j * order + i] {
                    return Err(invalid("Patrón no simétrico."));
                }
            }
        }
        Ok(Self {
            order,
            coordinates: entries
                .iter()
                .enumerate()
                .filter_map(|(i, &present)| present.then_some((i / order, i % order)))
                .collect(),
            mask: std::sync::OnceLock::from(entries),
        })
    }
    /// Constructs from coordinates; sorts and deduplicates them without a dense mask.
    /// ENG-E-0010 for overflow, invalid indices, or structural asymmetry.
    pub fn from_coordinates(order: usize, mut coordinates: Vec<(usize, usize)>) -> EResult<Self> {
        order
            .checked_mul(order)
            .ok_or_else(|| invalid("Patrón desbordado."))?;
        coordinates.sort_unstable();
        coordinates.dedup();
        for &(row, col) in &coordinates {
            if row >= order || col >= order || coordinates.binary_search(&(col, row)).is_err() {
                return Err(invalid("Coordenadas fuera de rango o patrón no simétrico."));
            }
        }
        Ok(Self {
            order,
            coordinates,
            mask: std::sync::OnceLock::new(),
        })
    }
    /// Structural entries sorted by row and column, without units.
    pub fn coordinates(&self) -> &[(usize, usize)] {
        &self.coordinates
    }
    /// Global pattern order.
    pub fn order(&self) -> usize {
        self.order
    }
    /// Does it contain structural entry (i, j)? O(log nnz), without extra memory.
    /// Out-of-range indices return `false`.
    pub fn contains(&self, row: usize, col: usize) -> bool {
        self.coordinates.binary_search(&(row, col)).is_ok()
    }
    /// Lazy row-major mask: `order²` `bool` values in row and
    /// column order, generated on demand in O(1) memory (without materializing anything).
    /// The number of elements is `order²`, which may be enormous: do not gather it
    /// with `collect` without bounding the size; use `try_entries` for that.
    pub fn mask_iter(&self) -> impl Iterator<Item = bool> + '_ {
        let order = self.order;
        let mut next = self.coordinates.iter().peekable();
        (0..order)
            .flat_map(move |i| (0..order).map(move |j| (i, j)))
            .map(move |ij| {
                if next.peek().is_some_and(|&&c| c == ij) {
                    next.next();
                    true
                } else {
                    false
                }
            })
    }
    /// Immutable row-major mask compatible with previous contracts.
    /// Materialized on demand with O(order²) memory and **fails** rather than
    /// panicking or aborting: ENG-E-0013 (unsupported capability) if
    /// `order²` overflows or exceeds the target's per-allocation limit
    /// (`MAX_ALLOC_BYTES`: 8 GiB on 64-bit, 1 GiB on 32-bit), and ENG-E-0010 if
    /// allocation fails. For large patterns use `coordinates`, `contains`, or
    /// `mask_iter`, which do not materialize the mask.
    pub fn try_entries(&self) -> EResult<&[bool]> {
        if let Some(mask) = self.mask.get() {
            return Ok(mask);
        }
        let len = self
            .order
            .checked_mul(self.order)
            .ok_or_else(|| unsupported("Patrón sin máscara densa: orden² desborda usize."))?;
        if len > crate::MAX_ALLOC_BYTES {
            return Err(unsupported(&format!(
                "Patrón sin máscara densa: orden {} requiere {len} bytes, sobre el límite de {} bytes; use coordinates(), contains() o mask_iter().",
                self.order,
                crate::MAX_ALLOC_BYTES
            )));
        }
        let mut entries = crate::alloc_vec(len, false)?;
        for &(i, j) in &self.coordinates {
            entries[i * self.order + j] = true;
        }
        Ok(self.mask.get_or_init(|| entries))
    }
}

impl PartialEq for SparsityPattern {
    fn eq(&self, other: &Self) -> bool {
        self.order == other.order && self.coordinates == other.coordinates
    }
}
impl Eq for SparsityPattern {}
