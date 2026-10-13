# Bitácora clean-room · ENG-LOAD-001

- Ticket / PR: ENG-LOAD-001 «Casos de carga»; sin commit, push ni PR en esta sesión.
- Autor(es): J2 (agente, Grok Bot), 2026-10-11 (America/Guayaquil), etiqueta solicitada para el ticket. Modelo de ejecución de esta sesión: Codex basado en GPT-6.
- Rol: especificador e implementador mediante derivaciones propias; sin rol de auditor u operador de oráculo.
- Contrato implementado: crate `engine-loads` 0.1.0; adaptación al tipo uniforme `engine-contracts::MemberLoad` de element/1; reutilización de `engine-model::{NodalLoad,LoadCase,DofValue}` y `engine-assembly::{DofMap,PrescribedDof}`.
- Fuentes permitidas usadas: exclusivamente especificación del usuario, contratos y código interno pertinente de este repositorio; ninguna fuente externa ni red.
- Derivaciones propias: ecuaciones (1)-(7) de esta nota, escritas para este ticket.
- Código de terceros copiado: ninguno. No hay dependencias externas nuevas; dependencias por ruta internas.
- Validación: analítica; tablas cerradas, equilibrio, superposición, transformación de ejes y concordancia con el elemento frame2d interno. Fixtures con `source` y `citation` en `crates/engine-loads/tests/fixtures/mod.rs`.
- Comparación con oráculo (repo hermano, informativa): no. No se ejecutó ningún oráculo.
- Declaración: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Revisor clean-room: PENDIENTE (J4).

## Derivaciones propias

### (1) Trabajo virtual e interpolación

En una viga 2D Euler–Bernoulli sin liberaciones, sean r=x/L y t=1−r.
La interpolación axial compatible con los valores de extremo es
u=t·u_i+r·u_j. Para la transversal, un polinomio cúbico que satisface
v(0)=v_i, v'(0)=theta_i, v(L)=v_j, v'(L)=theta_j determina:

H_i=t²(1+2r), H_theta_i=Lrt², H_j=r²(1+2t), H_theta_j=−Lr²t.

Cada polinomio cumple su condición de valor o pendiente unitaria y las otras
tres condiciones nulas. De ∫(qx·δu+qy·δv)dx=f_eq·δd se obtiene:

f_eq=∫[t·qx, H_i·qy, Lrt²·qy, r·qx, H_j·qy, −Lr²t·qy]dx.

Para una fuerza puntual se evalúa el integrando reemplazando qdx por P.
Las acciones de empotramiento sobre el miembro son f_fix=−f_eq. No se
confunden con resultantes de sección según normal exterior.

### (2) Uniforme y axial

Integrando (1) entre r=0 y r=1 y usando dx=Ldr:

f_eq=[qxL/2, qyL/2, qyL²/12, qxL/2, qyL/2, −qyL²/12].

La implementación uniforme ordena los productos como q(L/2) y [q(L/12)]L,
evitando calcular L² previamente. Coincide con la fórmula del frame2d interno.

### (3) Trapecio completo y triangular

Para cada componente q(r)=qi(1−r)+qj·r, multiplicar por los polinomios (1)
e integrar monomios mediante ∫₀¹r^k dr=1/(k+1) da:

N_i=L(2qx_i+qx_j)/6, N_j=L(qx_i+2qx_j)/6;
V_i=L(7qy_i+3qy_j)/20, V_j=L(3qy_i+7qy_j)/20;
M_i=L²(3qy_i+2qy_j)/60, M_j=−L²(2qy_i+3qy_j)/60.

Con qi=qj=w se reduce a (2). Para la triangular creciente qi=0, qj=w:
axiales wL/6 y wL/3; cortantes 3wL/20 y 7wL/20; momentos wL²/30
y −wL²/20. La triangular decreciente intercambia extremos y magnitudes de
momento. Un trapecio es una uniforme qi más una triangular de altura qj−qi,
también cuando ambas ocupan el mismo intervalo parcial.

### (4) Fuerza puntual

Con a=x y b=L−a, sustituir r=a/L y t=b/L en (1):

N_i=Px·b/L, N_j=Px·a/L;
V_i=Py·b²(L+2a)/L³, V_j=Py·a²(L+2b)/L³;
M_i=Py·ab²/L², M_j=−Py·a²b/L².

En a=b=L/2: cortantes Py/2, momentos ±PyL/8. La expresión «PL/2» para
el cortante puntual del enunciado se interpreta como P/2 por dimensiones
(N, mientras PL tiene unidades de momento). Para x=0 o L, la fuerza se
aplica exactamente al nodo correspondiente y los momentos son cero.

### (5) Intervalos parciales e integración exacta

Para 0≤a<b≤L, definir z∈[0,1], x=a+(b−a)z y q=qi(1−z)+qjz.
El integrando (1) es polinómico de grado ≤4 en z. La regla de tres puntos
con z=(1−√(3/5))/2, 1/2, (1+√(3/5))/2 y pesos 5/18, 4/9, 5/18
tiene momentos exactos 1/(k+1) para k=0,…,5: por simetría sus momentos
centrales impares son cero, y los pares k=0,2,4 son 1,1/12,1/80.
Estos son los momentos de la integral uniforme en [0,1]. Es por tanto una
integración analítica exacta para este problema, salvo redondeo f64.
No se usa cuadratura aproximada, refinamiento ni fuentes externas.

Tabla parcial independiente, derivada por antiderivadas de monomios entre
r=1/4 y 3/4, para ambas componentes iguales:

| Distribución | N_i/(wL) | V_i/(wL) | M_i/(wL²) | N_j/(wL) | V_j/(wL) | M_j/(wL²) |
| --- | --- | --- | --- | --- | --- | --- |
| Uniforme w | 1/4 | 1/4 | 11/192 | 1/4 | 1/4 | −11/192 |
| Triangular 0 → w | 5/48 | 61/640 | 31/1280 | 7/48 | 99/640 | −127/3840 |

En la segunda fila q(r)=w(2r−1/2). Se multiplica cada polinomio de (1)
por (2r−1/2) y se integra término a término. Resultante wL/4 y momento
transversal respecto a i de 7wL²/48. Para intervalos extremadamente pequeños
puede perderse precisión relativa por subdesbordamiento; no se promete
recuperar cantidades fuera de la representación IEEE-754.

### (6) Equilibrio y ejes

Las identidades t+r=1, H_i+H_j=1 y
H_theta_i+H_theta_j+L·H_j=x demuestran, respectivamente:

N_i+N_j=∫qx dx; V_i+V_j=∫qy dx;
M_i+M_j+L·V_j=∫x·qy dx.

Para reacciones de empotramiento, las sumas con las acciones externas son cero.
Una carga axial no aporta momento respecto a i. Las pruebas verifican estas
identidades para uniformes, trapecios, triangulares, parciales y puntuales.
Para trapecio completo: ∫q dx=L(qi+qj)/2 y ∫xq dx=L²(qi+2qj)/6.

Para c=cos(angle), s=sin(angle), q_local=[cQx+sQy,−sQx+cQy].
La transformación inversa es Q_global=[cqx−sqy,sqx+cqy]. Los momentos Z
se conservan. La coordenada x de una carga siempre mide distancia sobre el
miembro y no una proyección global. Se prueba una viga rotada con los tres
tipos de carga y concordancia de carga uniforme con load_vector y recover del
elemento frame2d sin liberaciones. El contrato MemberLoad recibe componentes
globales y momentos por longitud nulos.

### (7) Peso propio, casos nodales y desplazamientos

Una longitud dx tiene masa rho·A·dx y fuerza rho·A·g·dx. Integrar da
rho·A·L·g, y para varios miembros Σrho·A·L·g. La dirección y magnitud de g
son entradas explícitas tridimensionales; sin factores normativos, rho≥0, A>0,
L>0. Se prueban fuerzas totales, por longitud y suma de miembros.

Las acciones nodales reutilizan los valores y casos de engine-model; la posición
de cada componente la proporciona DofMap, incluido Rz en la tercera posición
D2. assemble_vector realiza el ensamblado global. No se define otra numeración.
ImposedDisplacement es un alias de PrescribedDof; el vector Option distingue
libre de cero prescrito. Se prueba su aplicación por Constraints, dimensiones
D2/D3, subconjuntos de GDL, referencias ausentes, duplicados y no finitos.
Las repeticiones de nodo/GDL en la misma lista del caso se rechazan, incluso
con valores iguales. Fuerza y desplazamiento sobre un mismo GDL son compatibles.

## Verificación de la sesión

Resultados de esta sesión, todos con salida final 0 y sin red (`CARGO_NET_OFFLINE=true`):

- `cargo fmt`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK, incluidos 13 tests de integración y 1 doctest
  de engine-loads. Las pruebas nativas usan rustc 1.85.1, la versión mínima del workspace.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK.
  Se corrigieron cuatro enlaces interpretados por rustdoc antes del resultado final.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK usando el
  toolchain ya instalado en `/home/box/.cargo/bin` (rustc 1.99.0). El rustc del
  sistema (1.85.1) carece de la biblioteca estándar wasm; el primer intento
  informó el error de compilación E0463. No se descargó ningún componente.
  Esta comprobación verifica compilación; no ejecución ni equivalencia numérica wasm.
- `bash clean-room/check.sh origin/main`: OK, incluyendo todos los archivos
  nuevos mediante `git add -N` en una copia temporal del índice en `/tmp`.
  Los objetos temporales también se alojaron en `/tmp`; el índice real se
  conservó intacto. `git diff --check` sobre esa copia: OK.

Sin commit ni push. Revisión clean-room: PENDIENTE (J4).
