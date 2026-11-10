//! Matriz simétrica dispersa. Fase 2 de MAM-1055.
//!
//! Guarda solo el triángulo superior en CSC: en la columna `j` están las
//! filas `i ≤ j`, crecientes, con la diagonal siempre presente. No convierte
//! unidades y no factoriza. El corte entre la ruta densa y la dispersa no se
//! decide aquí.

use crate::{
    alloc_vec, invalid, unsupported, EResult, EngineError, Real, Triplet, MAX_ALLOC_BYTES,
};

/// Límites que el llamador pasa a la construcción.
///
/// `max_order` y `max_bytes` son de esta llamada. No son el corte entre la
/// ruta densa y la dispersa, que se mide en la fase 10.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SymCscLimits {
    /// Orden máximo aceptado. Por encima, ENG-E-0013 y no se reserva.
    pub max_order: usize,
    /// Cota máxima de bytes. Por encima, ENG-E-0013 y no se reserva.
    pub max_bytes: usize,
}

/// Matriz simétrica con el triángulo superior en CSC.
///
/// La otra mitad no se guarda. Un cero explícito permanece. Una diagonal que
/// nadie aportó se guarda como cero, para poder nombrar el pivote más adelante.
#[derive(Clone, Debug, PartialEq)]
pub struct SymCscMatrix {
    order: usize,
    col_ptr: Vec<usize>,
    row_idx: Vec<usize>,
    values: Vec<Real>,
}

/// Registro de una tripleta ya llevada al triángulo superior.
#[derive(Clone, Copy)]
struct Aporte {
    columna: usize,
    fila: usize,
    lado: Lado,
    valor: Real,
}

/// Lado en el que el llamador escribió la tripleta, antes de plegarla.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Lado {
    Superior,
    Inferior,
}

/// Cota de bytes de la construcción, a partir de `n` y `nnz`.
///
/// `nnz` es el número de tripletas, no el de entradas únicas. La cota suma:
///
/// - `(n + 1)` punteros de columna;
/// - como máximo `nnz + n` índices de fila y otros tantos valores, para cubrir
///   la diagonal aunque ya venga entre las tripletas;
/// - un borrador de `nnz` registros, que se ordena en el sitio.
///
/// No incluye la cabecera del asignador. `None` si la cuenta desborda `usize`.
#[must_use]
pub fn sym_csc_bytes(n: usize, nnz: usize) -> Option<usize> {
    let almacenadas = nnz.checked_add(n)?;
    let punteros = n
        .checked_add(1)?
        .checked_mul(std::mem::size_of::<usize>())?;
    let indices = almacenadas.checked_mul(std::mem::size_of::<usize>())?;
    let valores = almacenadas.checked_mul(std::mem::size_of::<Real>())?;
    let borrador = nnz.checked_mul(std::mem::size_of::<Aporte>())?;
    punteros
        .checked_add(indices)?
        .checked_add(valores)?
        .checked_add(borrador)
}

impl SymCscMatrix {
    /// Construye la matriz desde tripletas.
    ///
    /// El orden de las comprobaciones, todas antes de la reserva cuando
    /// rechazan por datos o por cupo, es:
    ///
    /// 1. índices y valores no finitos de cada tripleta;
    /// 2. `order` frente a `limits.max_order`;
    /// 3. [`sym_csc_bytes`] frente a `limits.max_bytes`, o desborde de la cota.
    ///
    /// Solo después se reserva. Los duplicados de una misma posición y de un
    /// mismo lado se suman de menor a mayor según [`f64::total_cmp`], así que
    /// dos órdenes de inserción dejan los mismos bits. Si llegan las dos
    /// mitades, la comparación ocurre al ordenar, con el cupo ya aceptado:
    /// las sumas tienen que ser iguales y se guarda la de la mitad superior.
    /// Si difieren, ENG-E-0010 y no queda matriz.
    ///
    /// ```
    /// use engine_linalg::{SymCscLimits, SymCscMatrix, Triplet};
    /// let limites = SymCscLimits {
    ///     max_order: 8,
    ///     max_bytes: 1 << 20,
    /// };
    /// let a = SymCscMatrix::from_triplets(
    ///     2,
    ///     &[
    ///         Triplet { row: 0, col: 0, value: 2.0 },
    ///         Triplet { row: 0, col: 1, value: 1.0 },
    ///         Triplet { row: 1, col: 1, value: 4.0 },
    ///     ],
    ///     limites,
    /// )?;
    /// let mut y = [0.0, 0.0];
    /// a.matvec(&[3.0, 1.0], &mut y)?;
    /// assert_eq!(y, [7.0, 7.0]);
    /// # Ok::<(), engine_core::EngineError>(())
    /// ```
    pub fn from_triplets(order: usize, entries: &[Triplet], limits: SymCscLimits) -> EResult<Self> {
        validar_entradas(order, entries)?;
        if order > limits.max_order {
            return Err(unsupported(&format!(
                "Ruta dispersa: orden {order}, por encima del máximo configurado {}. No se reserva memoria.",
                limits.max_order
            )));
        }
        let nnz = entries.len();
        match sym_csc_bytes(order, nnz) {
            Some(bytes) if bytes <= limits.max_bytes => {}
            Some(bytes) => {
                return Err(unsupported(&format!(
                    "Ruta dispersa: orden {order}, {nnz} tripletas, estimación {bytes} bytes, límite {} bytes. No se reserva memoria.",
                    limits.max_bytes
                )));
            }
            None => {
                return Err(unsupported(&format!(
                    "Ruta dispersa: la estimación de memoria desborda usize para orden {order} y {nnz} tripletas. No se reserva memoria."
                )));
            }
        }
        ensamblar(order, entries)
    }

    /// Orden de la matriz.
    #[must_use]
    pub fn order(&self) -> usize {
        self.order
    }

    /// Punteros de columna, longitud `order + 1`.
    #[must_use]
    pub fn column_pointers(&self) -> &[usize] {
        &self.col_ptr
    }

    /// Filas almacenadas, crecientes dentro de cada columna, con `i ≤ j`.
    #[must_use]
    pub fn row_indices(&self) -> &[usize] {
        &self.row_idx
    }

    /// Valores paralelos a las filas, diagonal incluida.
    #[must_use]
    pub fn values(&self) -> &[Real] {
        &self.values
    }

    /// Entradas almacenadas, diagonal y ceros explícitos incluidos.
    #[must_use]
    pub fn stored_nnz(&self) -> usize {
        self.values.len()
    }

    /// Entrada `(fila, columna)`.
    ///
    /// `None` si el índice no cabe o si esa posición no está almacenada.
    /// Un cero explícito devuelve `Some(0.0)`. La diagonal siempre está.
    /// La mitad inferior se lee en su reflejo superior.
    #[must_use]
    pub fn get(&self, row: usize, col: usize) -> Option<Real> {
        if row >= self.order || col >= self.order {
            return None;
        }
        let (row, col) = if row <= col { (row, col) } else { (col, row) };
        let inicio = self.col_ptr[col];
        let fin = self.col_ptr[col + 1];
        self.row_idx[inicio..fin]
            .binary_search(&row)
            .ok()
            .map(|pos| self.values[inicio + pos])
    }

    /// Calcula `y = A x` sin modificar `y` si hay un error.
    ///
    /// Dentro de cada columna las filas crecen. Para `i < j` primero se suma
    /// en la fila `i` y después en la columna `j`. No convierte unidades.
    pub fn matvec(&self, x: &[Real], y: &mut [Real]) -> EResult<()> {
        if x.len() != self.order || y.len() != self.order {
            return Err(invalid(&format!(
                "Álgebra lineal: el producto pide longitud {} y recibió {} y {}.",
                self.order,
                x.len(),
                y.len()
            )));
        }
        if x.iter().any(|valor| !valor.is_finite()) {
            return Err(no_finito());
        }
        let mut acc = alloc_vec(self.order, 0.0)?;
        for columna in 0..self.order {
            let xj = x[columna];
            for p in self.col_ptr[columna]..self.col_ptr[columna + 1] {
                let fila = self.row_idx[p];
                let a = self.values[p];
                acc[fila] += a * xj;
                if fila != columna {
                    acc[columna] += a * x[fila];
                }
            }
        }
        if acc.iter().any(|valor| !valor.is_finite()) {
            return Err(no_finito());
        }
        y.copy_from_slice(&acc);
        Ok(())
    }
}

fn no_finito() -> EngineError {
    EngineError::new(
        "ENG-E-0008",
        "Álgebra lineal: entrada o resultado no finito.",
    )
}

fn validar_entradas(order: usize, entries: &[Triplet]) -> EResult<()> {
    for t in entries {
        if t.row >= order || t.col >= order {
            return Err(invalid(
                "Álgebra lineal: la tripleta está fuera del orden de la matriz simétrica.",
            ));
        }
        if !t.value.is_finite() {
            return Err(no_finito());
        }
    }
    Ok(())
}

fn reservar<T>(destino: &mut Vec<T>, len: usize) -> EResult<()> {
    let bytes = len
        .checked_mul(std::mem::size_of::<T>())
        .filter(|bytes| *bytes <= MAX_ALLOC_BYTES)
        .ok_or_else(|| {
            invalid(&format!(
                "Álgebra lineal: la reserva de {len} elementos supera el tope de {MAX_ALLOC_BYTES} bytes."
            ))
        })?;
    destino.try_reserve_exact(len).map_err(|_| {
        invalid(&format!(
            "Álgebra lineal: no se pudo reservar {bytes} bytes."
        ))
    })?;
    Ok(())
}

fn ensamblar(order: usize, entries: &[Triplet]) -> EResult<SymCscMatrix> {
    let mut aportes = Vec::new();
    reservar(&mut aportes, entries.len())?;
    for t in entries {
        let (fila, columna, lado) = if t.row <= t.col {
            (t.row, t.col, Lado::Superior)
        } else {
            (t.col, t.row, Lado::Inferior)
        };
        aportes.push(Aporte {
            columna,
            fila,
            lado,
            valor: t.value,
        });
    }
    aportes.sort_unstable_by(|a, b| {
        a.columna
            .cmp(&b.columna)
            .then(a.fila.cmp(&b.fila))
            .then(a.lado.cmp(&b.lado))
            .then(a.valor.total_cmp(&b.valor))
    });

    let almacenadas = contar(order, &aportes)?;
    let punteros = order
        .checked_add(1)
        .ok_or_else(|| invalid("Álgebra lineal: el orden de los punteros desborda."))?;
    let mut col_ptr = alloc_vec(punteros, 0usize)?;
    let mut filas = Vec::new();
    let mut valores = Vec::new();
    reservar(&mut filas, almacenadas)?;
    reservar(&mut valores, almacenadas)?;
    escribir(order, &aportes, &mut col_ptr, &mut filas, &mut valores)?;
    if filas.len() != almacenadas || valores.len() != almacenadas {
        return Err(EngineError::new(
            "ENG-E-0012",
            "Álgebra lineal: el conteo de la matriz dispersa no coincide con lo escrito.",
        ));
    }
    Ok(SymCscMatrix {
        order,
        col_ptr,
        row_idx: filas,
        values: valores,
    })
}

fn fin_de_grupo(aportes: &[Aporte], inicio: usize) -> usize {
    let columna = aportes[inicio].columna;
    let fila = aportes[inicio].fila;
    let mut fin = inicio + 1;
    while fin < aportes.len() && aportes[fin].columna == columna && aportes[fin].fila == fila {
        fin += 1;
    }
    fin
}

fn sumar(grupo: &[Aporte]) -> EResult<Real> {
    // Sin identidad +0: un único −0 tiene que conservar el signo.
    let Some(primero) = grupo.first() else {
        return Err(invalid(
            "Álgebra lineal: el grupo de tripletas de la matriz dispersa quedó vacío.",
        ));
    };
    let mut acc = primero.valor;
    if !acc.is_finite() {
        return Err(no_finito());
    }
    for aporte in &grupo[1..] {
        acc += aporte.valor;
        if !acc.is_finite() {
            return Err(no_finito());
        }
    }
    Ok(acc)
}

fn valor_declarado(grupo: &[Aporte]) -> EResult<Real> {
    let mut corte = 0;
    while corte < grupo.len() && grupo[corte].lado == Lado::Superior {
        corte += 1;
    }
    debug_assert!(grupo[corte..].iter().all(|a| a.lado == Lado::Inferior));
    let superior = if corte > 0 {
        Some(sumar(&grupo[..corte])?)
    } else {
        None
    };
    let inferior = if corte < grupo.len() {
        Some(sumar(&grupo[corte..])?)
    } else {
        None
    };
    match (superior, inferior) {
        (Some(sum_sup), Some(sum_inf)) if sum_sup == sum_inf => Ok(sum_sup),
        (Some(_), Some(_)) => Err(invalid(
            "Álgebra lineal: la simetría declarada no se cumple en la matriz dispersa.",
        )),
        (Some(sum_sup), None) => Ok(sum_sup),
        (None, Some(sum_inf)) => Ok(sum_inf),
        (None, None) => Err(invalid(
            "Álgebra lineal: el grupo de tripletas de la matriz dispersa quedó vacío.",
        )),
    }
}

fn contar(order: usize, aportes: &[Aporte]) -> EResult<usize> {
    let mut almacenadas = 0usize;
    let mut cursor = 0usize;
    for columna in 0..order {
        let mut vio_diagonal = false;
        while cursor < aportes.len() && aportes[cursor].columna == columna {
            let fin = fin_de_grupo(aportes, cursor);
            let _ = valor_declarado(&aportes[cursor..fin])?;
            if aportes[cursor].fila == columna {
                vio_diagonal = true;
            }
            almacenadas += 1;
            cursor = fin;
        }
        if !vio_diagonal {
            almacenadas += 1;
        }
    }
    if cursor != aportes.len() {
        return Err(invalid(
            "Álgebra lineal: quedó una tripleta fuera de las columnas de la matriz dispersa.",
        ));
    }
    Ok(almacenadas)
}

fn escribir(
    order: usize,
    aportes: &[Aporte],
    col_ptr: &mut [usize],
    filas: &mut Vec<usize>,
    valores: &mut Vec<Real>,
) -> EResult<()> {
    let mut cursor = 0usize;
    for (columna, puntero) in col_ptr.iter_mut().take(order).enumerate() {
        *puntero = filas.len();
        let mut vio_diagonal = false;
        while cursor < aportes.len() && aportes[cursor].columna == columna {
            let fin = fin_de_grupo(aportes, cursor);
            let fila = aportes[cursor].fila;
            debug_assert!(fila <= columna);
            let valor = valor_declarado(&aportes[cursor..fin])?;
            if fila == columna {
                vio_diagonal = true;
            }
            filas.push(fila);
            valores.push(valor);
            cursor = fin;
        }
        if !vio_diagonal {
            filas.push(columna);
            valores.push(0.0);
        }
    }
    if cursor != aportes.len() {
        return Err(invalid(
            "Álgebra lineal: quedó una tripleta fuera de las columnas de la matriz dispersa.",
        ));
    }
    col_ptr[order] = filas.len();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limites(order: usize, nnz: usize) -> SymCscLimits {
        SymCscLimits {
            max_order: order,
            max_bytes: sym_csc_bytes(order, nnz).unwrap(),
        }
    }

    fn t(row: usize, col: usize, value: f64) -> Triplet {
        Triplet { row, col, value }
    }

    fn bits(matriz: &SymCscMatrix) -> Vec<u64> {
        matriz
            .values()
            .iter()
            .map(|valor| valor.to_bits())
            .collect()
    }

    #[test]
    fn la_cota_sigue_la_formula_de_n_y_nnz() {
        assert_eq!(sym_csc_bytes(0, 0), Some(std::mem::size_of::<usize>()));
        assert_eq!(sym_csc_bytes(usize::MAX, 0), None);
        assert_eq!(sym_csc_bytes(usize::MAX, 1), None);
        let n = 4usize;
        let nnz = 6usize;
        let almacenadas = nnz + n;
        let esperado = (n + 1) * std::mem::size_of::<usize>()
            + almacenadas * std::mem::size_of::<usize>()
            + almacenadas * std::mem::size_of::<f64>()
            + nnz * std::mem::size_of::<Aporte>();
        assert_eq!(sym_csc_bytes(n, nnz), Some(esperado));
    }

    #[test]
    fn construccion_duplicados_y_orden_dan_los_mismos_bits() {
        // 0.1, 0.2 y 0.3 en orden total_cmp se suman (0.1 + 0.2) + 0.3.
        // Ese binary64 es 0x3fe3333333333334. El pliegue inverso no coincide.
        const CANONICO: u64 = 0x3fe3_3333_3333_3334;
        let directo = [t(1, 1, 0.1), t(1, 1, 0.2), t(1, 1, 0.3), t(0, 0, 2.0)];
        let inverso = [t(0, 0, 2.0), t(1, 1, 0.3), t(1, 1, 0.2), t(1, 1, 0.1)];
        let a = SymCscMatrix::from_triplets(2, &directo, limites(2, directo.len())).unwrap();
        let b = SymCscMatrix::from_triplets(2, &inverso, limites(2, inverso.len())).unwrap();
        assert_eq!(a.column_pointers(), b.column_pointers());
        assert_eq!(a.row_indices(), b.row_indices());
        assert_eq!(bits(&a), bits(&b));
        assert_eq!(a.get(1, 1).unwrap().to_bits(), CANONICO);
        let insercion: f64 = (0.3 + 0.2) + 0.1;
        assert_ne!(insercion.to_bits(), CANONICO);

        // total_cmp ordena -1e16, 1, 1e16. La unidad se pierde junto a 1e16
        // (ulp 2) y -1e16 + 1e16 deja +0. El orden de inserción opuesto, si se
        // sumara tal cual, dejaría 1.
        let grandes = [t(0, 0, 1e16), t(0, 0, -1e16), t(0, 0, 1.0)];
        let grandes_al_reves = [t(0, 0, 1.0), t(0, 0, -1e16), t(0, 0, 1e16)];
        let c = SymCscMatrix::from_triplets(1, &grandes, limites(1, 3)).unwrap();
        let d = SymCscMatrix::from_triplets(1, &grandes_al_reves, limites(1, 3)).unwrap();
        assert_eq!(c.values()[0].to_bits(), 0);
        assert_eq!(d.values()[0].to_bits(), 0);
        let como_insercion: f64 = (1e16 + -1e16) + 1.0;
        assert_eq!(como_insercion.to_bits(), 1.0f64.to_bits());
    }

    #[test]
    fn las_dos_mitades_se_validan_y_no_se_suman() {
        let coinciden = [t(0, 1, 0.25), t(0, 1, 0.25), t(1, 0, 0.5), t(1, 1, 4.0)];
        let al_reves = [t(1, 0, 0.5), t(1, 1, 4.0), t(0, 1, 0.25), t(0, 1, 0.25)];
        let a = SymCscMatrix::from_triplets(2, &coinciden, limites(2, 4)).unwrap();
        let b = SymCscMatrix::from_triplets(2, &al_reves, limites(2, 4)).unwrap();
        assert_eq!(a.get(0, 1).unwrap().to_bits(), 0.5f64.to_bits());
        assert_eq!(a.get(1, 0).unwrap().to_bits(), 0.5f64.to_bits());
        assert_eq!(bits(&a), bits(&b));
        // Solo la mitad inferior: se guarda ese valor, sin duplicarlo.
        let solo_inf = SymCscMatrix::from_triplets(2, &[t(1, 0, 5.0)], limites(2, 1)).unwrap();
        assert_eq!(solo_inf.column_pointers(), &[0, 1, 3]);
        assert_eq!(solo_inf.row_indices(), &[0, 0, 1]);
        assert_eq!(solo_inf.values(), &[0.0, 5.0, 0.0]);
        // Cero explícito fuera de la diagonal, distinto de no traer la entrada.
        let explicito = SymCscMatrix::from_triplets(2, &[t(0, 1, 0.0)], limites(2, 1)).unwrap();
        let ausente = SymCscMatrix::from_triplets(2, &[], limites(2, 0)).unwrap();
        assert_eq!(explicito.get(0, 1), Some(0.0));
        assert_eq!(ausente.get(0, 1), None);
        assert_eq!(ausente.stored_nnz(), 2);
        let signo = SymCscMatrix::from_triplets(1, &[t(0, 0, -0.0)], limites(1, 1)).unwrap();
        assert_eq!(signo.values()[0].to_bits(), (-0.0f64).to_bits());

        let err = SymCscMatrix::from_triplets(2, &[t(0, 1, 1.0), t(1, 0, 2.0)], limites(2, 2))
            .unwrap_err();
        assert_eq!(err.code, "ENG-E-0010");
    }

    #[test]
    fn indices_fuera_de_rango_y_no_finitos() {
        let err = SymCscMatrix::from_triplets(3, &[t(3, 0, 1.0)], limites(3, 1)).unwrap_err();
        assert_eq!(err.code, "ENG-E-0010");
        let err =
            SymCscMatrix::from_triplets(1, &[t(0, usize::MAX, 1.0)], limites(1, 1)).unwrap_err();
        assert_eq!(err.code, "ENG-E-0010");
        for valor in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let err = SymCscMatrix::from_triplets(1, &[t(0, 0, valor)], limites(1, 1)).unwrap_err();
            assert_eq!(err.code, "ENG-E-0008");
        }
        // 1e308 + 1e308 no cabe en binary64.
        let err = SymCscMatrix::from_triplets(1, &[t(0, 0, 1e308), t(0, 0, 1e308)], limites(1, 2))
            .unwrap_err();
        assert_eq!(err.code, "ENG-E-0008");
    }

    #[test]
    fn rechazo_de_orden_y_de_memoria_antes_de_seguir() {
        let vacia = SymCscLimits {
            max_order: 0,
            max_bytes: sym_csc_bytes(0, 0).unwrap(),
        };
        assert_eq!(
            SymCscMatrix::from_triplets(0, &[], vacia)
                .unwrap()
                .stored_nnz(),
            0
        );
        let por_orden = SymCscMatrix::from_triplets(
            4,
            &[],
            SymCscLimits {
                max_order: 3,
                max_bytes: usize::MAX,
            },
        )
        .unwrap_err();
        assert_eq!(por_orden.code, "ENG-E-0013");
        assert!(por_orden.message.contains('4'));
        assert!(por_orden.message.contains('3'));
        assert!(por_orden.message.contains("No se reserva"));

        let bytes = sym_csc_bytes(2, 1).unwrap();
        let ok = SymCscMatrix::from_triplets(
            2,
            &[t(1, 0, 4.0)],
            SymCscLimits {
                max_order: 2,
                max_bytes: bytes,
            },
        );
        assert!(ok.is_ok());
        let justo = SymCscMatrix::from_triplets(
            2,
            &[t(1, 0, 4.0)],
            SymCscLimits {
                max_order: 2,
                max_bytes: bytes - 1,
            },
        )
        .unwrap_err();
        assert_eq!(justo.code, "ENG-E-0013");
        assert!(justo.message.contains(&bytes.to_string()));
        assert!(justo.message.contains(&(bytes - 1).to_string()));
        assert!(justo.message.contains("No se reserva"));

        let desborde = SymCscMatrix::from_triplets(
            usize::MAX,
            &[],
            SymCscLimits {
                max_order: usize::MAX,
                max_bytes: usize::MAX,
            },
        )
        .unwrap_err();
        assert_eq!(desborde.code, "ENG-E-0013");
        assert!(desborde.message.contains("No se reserva"));
    }

    #[test]
    fn matvec_coincide_con_la_densa_en_un_caso_pequeno() {
        // A = [[2, 0.5, 0], [0.5, 3, -1], [0, -1, 4]]
        // x = [1, 2, -1]
        // y0 = 2*1 + 0.5*2 + 0*(-1) = 3
        // y1 = 0.5*1 + 3*2 + (-1)*(-1) = 7.5
        // y2 = 0*1 + (-1)*2 + 4*(-1) = -6
        let entradas = [
            t(0, 0, 2.0),
            t(0, 1, 0.25),
            t(0, 1, 0.25),
            t(1, 0, 0.5),
            t(1, 1, 3.0),
            t(1, 2, -1.0),
            t(2, 2, 4.0),
        ];
        let a = SymCscMatrix::from_triplets(3, &entradas, limites(3, entradas.len())).unwrap();
        let x = [1.0, 2.0, -1.0];
        let mut y = [9.0, 9.0, 9.0];
        a.matvec(&x, &mut y).unwrap();
        assert_eq!(y, [3.0, 7.5, -6.0]);

        // La densa se escribe a mano, con las dos mitades, y se multiplica por filas.
        let plana = [
            2.0, 0.5, 0.0, //
            0.5, 3.0, -1.0, //
            0.0, -1.0, 4.0,
        ];
        let densa = crate::SymMatrix::new(crate::DMat::new(3, 3, plana.to_vec()).unwrap()).unwrap();
        let mut por_filas = [0.0; 3];
        for (i, destino) in por_filas.iter_mut().enumerate() {
            let mut acc = 0.0;
            for (j, xj) in x.iter().enumerate() {
                acc += densa.dense().get(i, j).unwrap() * xj;
            }
            *destino = acc;
        }
        assert_eq!(por_filas, y);

        let mut intacto = [9.0, 9.0];
        let err = a.matvec(&[1.0, 2.0], &mut intacto).unwrap_err();
        assert_eq!(err.code, "ENG-E-0010");
        assert_eq!(intacto, [9.0, 9.0]);
        let err = a.matvec(&[1.0, f64::NAN, 0.0], &mut [0.0; 3]).unwrap_err();
        assert_eq!(err.code, "ENG-E-0008");
    }
}
