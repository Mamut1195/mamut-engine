//! Tipos definitivos del motor, independientes de cualquier librería numérica.
use crate::*;

/// Matriz densa fila-mayor para bloques de elemento y varias RHS.
///
/// El constructor exige longitud
/// rows × cols y datos finitos. Las unidades dependen de cada fila/columna
/// (p. ej. fuerza/desplazamiento para una tangente).
#[derive(Clone, Debug, PartialEq)]
pub struct DMat {
    rows: usize,
    cols: usize,
    pub(crate) values: Vec<Real>,
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
    /// Crea almacenamiento nulo; ENG-E-0010 si el tamaño desborda o excede el
    /// límite de reserva (8 GiB), sin pánico.
    pub fn zeros(rows: usize, cols: usize) -> EResult<Self> {
        let len = rows
            .checked_mul(cols)
            .ok_or_else(|| invalid("Tamaño matricial desbordado."))?;
        Self::new(rows, cols, alloc_vec(len, 0.0)?)
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

/// Matriz simétrica densa fila-mayor para problemas pequeños.
///
/// Almacena ambas mitades;
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

/// Patrón simétrico con coordenadas ordenadas y únicas.
///
/// El constructor compatible recibe una máscara: true declara una entrada
/// potencialmente no nula. No realiza ordenamiento ni análisis numérico.
#[derive(Clone, Debug)]
pub struct SparsityPattern {
    order: usize,
    coordinates: Vec<(usize, usize)>,
    mask: std::sync::OnceLock<Vec<bool>>,
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
    /// Construye desde coordenadas; las ordena y deduplica sin una máscara densa.
    /// ENG-E-0010 por desbordamiento, índices inválidos o asimetría estructural.
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
    /// Entradas estructurales ordenadas por fila y columna, sin unidades.
    pub fn coordinates(&self) -> &[(usize, usize)] {
        &self.coordinates
    }
    /// Orden global del patrón.
    pub fn order(&self) -> usize {
        self.order
    }
    /// ¿Contiene la entrada estructural (i, j)? O(log nnz), sin memoria extra.
    /// Índices fuera de rango devuelven `false`.
    pub fn contains(&self, row: usize, col: usize) -> bool {
        self.coordinates.binary_search(&(row, col)).is_ok()
    }
    /// Máscara fila-mayor perezosa: `order²` valores `bool` en orden de fila y
    /// columna, generados bajo demanda en memoria O(1) (sin materializar nada).
    /// El número de elementos es `order²`, que puede ser enorme: no la recoja
    /// con `collect` sin acotar el tamaño; use `try_entries` para eso.
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
    /// Máscara inmutable fila-mayor compatible con los contratos anteriores.
    /// Se materializa bajo demanda con memoria O(orden²) y **falla** en vez de
    /// provocar pánico o aborto: ENG-E-0013 (capacidad no soportada) si
    /// `orden²` desborda o supera el límite de reserva por destino
    /// (`MAX_ALLOC_BYTES`: 8 GiB en 64 bits, 1 GiB en 32 bits), y ENG-E-0010 si
    /// la reserva falla. Para patrones grandes use `coordinates`, `contains` o
    /// `mask_iter`, que no materializan la máscara.
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
