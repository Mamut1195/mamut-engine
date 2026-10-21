//! Deterministic assembly without intermediate dense storage.
use crate::*;
use std::collections::BTreeMap;

/// Numerical contribution to an entry; indices counted from zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triplet {
    /// Global row.
    pub row: usize,
    /// Global column.
    pub col: usize,
    /// Finite magnitude in that entry's units.
    pub value: Real,
}

/// Internal general CSR sparse matrix: unique columns sorted per row.
///
/// Duplicates are summed sequentially in input order, including explicit
/// zeros. O(rows + unique entries) memory; without HashMap or threads.
#[derive(Clone, Debug, PartialEq)]
pub struct SparseMatrix {
    rows: usize,
    cols: usize,
    offsets: Vec<usize>,
    columns: Vec<usize>,
    values: Vec<Real>,
}
impl SparseMatrix {
    /// Validates indices/finiteness and sums duplicates; ENG-E-0010/0008 on violation.
    pub fn from_triplets(rows: usize, cols: usize, entries: &[Triplet]) -> EResult<Self> {
        let len = rows
            .checked_add(1)
            .ok_or_else(|| invalid("Número de filas desbordado."))?;
        let mut sums = BTreeMap::new();
        for t in entries {
            if t.row >= rows || t.col >= cols {
                return Err(invalid("Tripleta fuera de las dimensiones de ensamblaje."));
            }
            finite(&[t.value])?;
            let sum = sums.entry((t.row, t.col)).or_insert(0.0);
            *sum += t.value;
            finite(&[*sum])?;
        }
        let mut result = Self {
            rows,
            cols,
            offsets: alloc_vec(len, 0usize)?,
            columns: Vec::with_capacity(sums.len()),
            values: Vec::with_capacity(sums.len()),
        };
        for ((row, col), value) in sums {
            result.offsets[row + 1] += 1;
            result.columns.push(col);
            result.values.push(value);
        }
        for row in 0..rows {
            result.offsets[row + 1] += result.offsets[row];
        }
        Ok(result)
    }
    /// Number of rows.
    pub fn rows(&self) -> usize {
        self.rows
    }
    /// Number of columns.
    pub fn cols(&self) -> usize {
        self.cols
    }
    /// Start of each row and end of storage, length rows + 1.
    pub fn row_offsets(&self) -> &[usize] {
        &self.offsets
    }
    /// Sorted columns, parallel to values.
    pub fn column_indices(&self) -> &[usize] {
        &self.columns
    }
    /// CSR numerical values, including explicit zeros.
    pub fn values(&self) -> &[Real] {
        &self.values
    }
    /// Expands for tests/small problems; O(rows × columns) memory.
    /// ENG-E-0010 for size overflow. This is not a sparse solver (ENG-LA-008).
    pub fn to_dense(&self) -> EResult<DMat> {
        let mut result = DMat::zeros(self.rows, self.cols)?;
        for row in 0..self.rows {
            for p in self.offsets[row]..self.offsets[row + 1] {
                result.values[row * self.cols + self.columns[p]] = self.values[p];
            }
        }
        Ok(result)
    }
    /// Obtains the square symmetric pattern; ENG-E-0010 if rectangular
    /// or if any coordinate lacks its structural mirror.
    pub fn symmetric_pattern(&self) -> EResult<SparsityPattern> {
        size(self.rows, self.cols)?;
        let coordinates = (0..self.rows)
            .flat_map(|row| {
                (self.offsets[row]..self.offsets[row + 1]).map(move |p| (row, self.columns[p]))
            })
            .collect();
        SparsityPattern::from_coordinates(self.rows, coordinates)
    }
}
