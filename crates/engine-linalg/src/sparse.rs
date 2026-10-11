//! Ensamblaje determinista sin almacenamiento denso intermedio.
use crate::*;
use std::collections::BTreeMap;

/// Contribución numérica a una entrada; índices contados desde cero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Triplet {
    /// Fila global.
    pub row: usize,
    /// Columna global.
    pub col: usize,
    /// Magnitud finita en las unidades de esa entrada.
    pub value: Real,
}

/// Matriz general dispersa CSR propia: columnas únicas y ordenadas por fila.
///
/// Los duplicados se suman secuencialmente en su orden de entrada, incluidos
/// ceros explícitos. Memoria O(filas + entradas únicas); sin HashMap ni hilos.
#[derive(Clone, Debug, PartialEq)]
pub struct SparseMatrix {
    rows: usize,
    cols: usize,
    offsets: Vec<usize>,
    columns: Vec<usize>,
    values: Vec<Real>,
}
impl SparseMatrix {
    /// Valida índices/finitud y suma duplicados; ENG-E-0010/0008 por incumplimiento.
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
            offsets: vec![0; len],
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
    /// Número de filas.
    pub fn rows(&self) -> usize {
        self.rows
    }
    /// Número de columnas.
    pub fn cols(&self) -> usize {
        self.cols
    }
    /// Inicio de cada fila y final del almacenamiento, longitud filas + 1.
    pub fn row_offsets(&self) -> &[usize] {
        &self.offsets
    }
    /// Columnas ordenadas, paralelas a los valores.
    pub fn column_indices(&self) -> &[usize] {
        &self.columns
    }
    /// Valores numéricos CSR, incluidos los ceros explícitos.
    pub fn values(&self) -> &[Real] {
        &self.values
    }
    /// Expande para pruebas/problemas pequeños; memoria O(filas × columnas).
    /// ENG-E-0010 por tamaño desbordado. No es un solver disperso (ENG-LA-008).
    pub fn to_dense(&self) -> EResult<DMat> {
        let mut result = DMat::zeros(self.rows, self.cols)?;
        for row in 0..self.rows {
            for p in self.offsets[row]..self.offsets[row + 1] {
                result.values[row * self.cols + self.columns[p]] = self.values[p];
            }
        }
        Ok(result)
    }
    /// Obtiene el patrón cuadrado simétrico; ENG-E-0010 si es rectangular
    /// o si alguna coordenada carece de su reflejo estructural.
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
