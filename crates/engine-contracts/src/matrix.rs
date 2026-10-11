//! Almacenamiento provisional; se reemplaza por el backend de ENG-LA-007.
use crate::*;

/// Matriz densa fila-mayor, sin operaciones de álgebra lineal.
///
/// Se reemplaza por el backend de ENG-LA-007. El constructor exige longitud
/// rows × cols y datos finitos. Las unidades dependen de cada fila/columna
/// (p. ej. fuerza/desplazamiento para una tangente).
#[derive(Clone, Debug, PartialEq)]
pub struct DMat {
    rows: usize,
    cols: usize,
    values: Vec<Real>,
}
impl DMat {
    /// Construye desde valores fila-mayor; ENG-E-0010 por tamaño/desbordamiento
    /// y ENG-E-0008 por datos no finitos. No impone simetría.
    pub fn new(rows: usize, cols: usize, values: Vec<Real>) -> EResult<Self> {
        let len = rows
            .checked_mul(cols)
            .ok_or_else(|| invalid("Tamaño matricial desbordado."))?;
        size(values.len(), len)?;
        finite(&values)?;
        Ok(Self { rows, cols, values })
    }
    /// Crea almacenamiento nulo; ENG-E-0010 si el tamaño desborda.
    pub fn zeros(rows: usize, cols: usize) -> EResult<Self> {
        let len = rows
            .checked_mul(cols)
            .ok_or_else(|| invalid("Tamaño matricial desbordado."))?;
        Self::new(rows, cols, vec![0.0; len])
    }
    /// Número de filas.
    pub fn rows(&self) -> usize {
        self.rows
    }
    /// Número de columnas.
    pub fn cols(&self) -> usize {
        self.cols
    }
    /// Datos inmutables en orden fila-mayor.
    pub fn values(&self) -> &[Real] {
        &self.values
    }
    /// Entrada (fila, columna), o None si está fuera de rango.
    pub fn get(&self, row: usize, col: usize) -> Option<Real> {
        if row < self.rows && col < self.cols {
            Some(self.values[row * self.cols + col])
        } else {
            None
        }
    }
}

/// Matriz simétrica provisional densa fila-mayor.
///
/// Se reemplaza por el backend de ENG-LA-007. Almacena ambas mitades;
/// exige simetría exacta para no introducir una tolerancia física implícita.
#[derive(Clone, Debug, PartialEq)]
pub struct SymMatrix(DMat);
impl SymMatrix {
    /// Valida cuadratura y simetría exacta; ENG-E-0010 si incumple.
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
    /// Construye una matriz nula n × n; ENG-E-0010 si desborda.
    pub fn zeros(n: usize) -> EResult<Self> {
        Self::new(DMat::zeros(n, n)?)
    }
    /// Orden de la matriz.
    pub fn order(&self) -> usize {
        self.0.rows()
    }
    /// Vista inmutable del almacenamiento denso.
    pub fn dense(&self) -> &DMat {
        &self.0
    }
}

/// Patrón estructural provisional con máscara densa fila-mayor.
///
/// Se reemplaza por el backend de ENG-LA-007; true declara una entrada
/// potencialmente no nula. No realiza ordenamiento ni análisis numérico.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SparsityPattern {
    order: usize,
    entries: Vec<bool>,
}
impl SparsityPattern {
    /// Construye patrón simétrico n × n; ENG-E-0010 por tamaño o asimetría.
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
        Ok(Self { order, entries })
    }
    /// Orden global del patrón.
    pub fn order(&self) -> usize {
        self.order
    }
    /// Máscara inmutable fila-mayor, sin unidades.
    pub fn entries(&self) -> &[bool] {
        &self.entries
    }
}
