# Bitácora clean-room · ENG-ANL-001

- Ticket / PR: ENG-ANL-001 «Análisis estático lineal». Sin commit ni push.
- Autor(es): J2 (agente, Grok Bot) · identificación solicitada por el usuario;
  agente ejecutor Codex (GPT-6) · 2026-10-11 (America/Guayaquil).
- Rol: implementador y autor de derivaciones analíticas propias.
- Contrato implementado: `static.linear`, API `StaticLinear::run` / `run_about`
  sobre `LinearSystem`, según la API concreta solicitada. La adaptación al
  trait genérico `AnalysisMethod` (`analysis/1`, sink/control/settings) queda
  fuera de esta interfaz.
- Fuentes permitidas usadas: exclusivamente la petición del ticket, APIs propias
  de engine-core, engine-contracts, engine-model, engine-assembly,
  engine-elements-truss, engine-elements-frame y la frontera pública de
  engine-linalg. Plantilla: `provenance/TEMPLATE.md`. Sin fuentes externas,
  búsquedas en red, libros, resultados de otros motores ni oráculos.
- Derivaciones propias: [nota de esta bitácora](#derivaciones-propias) y
  [benchmarks lineales embebidos](../crates/engine-analysis/tests/analytical.rs).
- Código de terceros copiado: ninguno. Solo dependencias internas por `path`,
  incluidas las dependencias de pruebas; sin dependencias externas nuevas.
- Validación: analítica, formas cerradas derivadas abajo y contador de llamadas
  al backend de referencia existente. No se implementó un solver numérico nuevo.
- Comparación con oráculo (repo hermano, informativa): no.
- Declaración: "No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados".
- Revisor clean-room: PENDIENTE (J4).

## Diseño, convención y límites

`solve_cases` valida una lista no vacía de nombres distintos y todos sus RHS
antes de factorizar una sola vez K_ff. Reutiliza ese factor para cada RHS en el
orden solicitado y traduce los errores igual que `solve`. No cambia la semántica
de `solve`, que sigue factorizando cada llamada. Ninguna operación acepta ni
modifica el estado de prueba. El pivote de un mecanismo señala un nodo/GDL
implicado, sin describir todo el modo. El mapa conserva las coordenadas originales
para los brazos de fuerzas; no vuelve a requerirse el modelo en el análisis.

Las cargas físicas incluyen acciones nodales, equivalentes elementales y cargas
sobre apoyos. Se escala F, nunca u_p. La partición de Ku=F+R da:

```text
K_ff u_f = λ F_f − K_fp u_p
R_p = K_pf u_f + K_pp u_p − λ F_p
```

Los seis componentes se ordenan Fx,Fy,Fz,Mx,My,Mz. Los campos
`applied_force_sum`, `reaction_sum` y `residual_force` suman por componente;
sus tres componentes rotacionales son pares nodales sin brazo. El momento total
`residual_moment` es Σ(M + (x−p)×F) de cargas físicas y reacciones respecto a p,
el origen por defecto. Por ello los pares solos pueden tener suma distinta de
cero aun en equilibrio: el voladizo con carga terminal presenta ΣMz=PL, cancelado
por los brazos de fuerzas. El signo es de mano derecha, con posiciones sin
deformar. D2 representa XY, Z=0 y Rz; D3 usa XYZ y Rx,Ry,Rz. Se respetan las
coordenadas originales sin proyección; la geometría D2 debe cumplir Z=0.

El residual relativo global es el máximo de las fuerzas residuales divididas por
max(1 N, suma de módulos de fuerzas por componente) y los momentos totales
residuales divididos por max(1 N·m, suma de módulos de pares y términos del brazo
por componente). Se incluyen cargas y reacciones en ambos denominadores. No se
normaliza por la suma algebraica, que sería cero en equilibrio. El residual libre
es ‖(Ku−λF)_f‖∞ / max(1, ‖(Ku)_f‖∞, ‖λF_f‖∞), con todos los valores en SI.
Esta última norma mezcla fuerzas y momentos: es un indicador numérico, no una
norma física invariante ante cambios de unidades. Se conserva también la norma
absoluta. Se rechazan entradas y productos/sumas/reportes no finitos.

El análisis es lineal de primer orden: no usa Kg, no regulariza mecanismos y no
actualiza geometría. Las combinaciones de engine-model son declarativas; los
casos combinados deben aportarse como cargas de un caso del sistema. Con u_p=0,
u(A+B)=u(A)+u(B), y lo mismo para las reacciones. Con asentamientos,
u(A+B)=u(A)+u(B)−u(0), donde u(0) incluye el mismo asentamiento una sola vez.
El backend de referencia y la reducción densa sirven para modelos pequeños;
no se promete escalabilidad ni identidad binaria entre nativo y wasm.

## Derivaciones propias

Todos los fixtures están embebidos en Rust, sin acceso a archivos, red, reloj o
procesos durante las pruebas. Tolerancias absolutas: 1e-12 m/rad para
desplazamientos y 1e-9 N/N·m para reacciones y equilibrio; indicadores relativos
inferiores a 1e-12. Son datos SI artificiales para comprobar ecuaciones.

### Armadura de dos barras simétrica y superposición

Bases (−4,0,0), (4,0,0), ápice (0,3,0); E=200 Pa, A=2 m², L=5 m.
Con las bases fijas, cada barra tiene energía EA/(2L)(d·u)². Los cosenos
c=4/5, s=3/5 dan Kxx=2(EA/L)c²=102.4 N/m,
Kyy=2(EA/L)s²=57.6 N/m y Kxy=0 por simetría.
El caso A aplica H=10 N, el B aplica −P=−12 N en Y; AB aplica ambos.
u_A=(10/102.4,0), u_B=(0,−12/57.6); u_AB es su suma.
En B, Ry=P/2 por base; Rx izquierdo=Pa/(2h)=8 N y derecho=−8 N.
El contador comprueba exactamente una factorización y tres resoluciones,
incluyendo superposición de desplazamientos y reacciones. Se verifica que el
estado aceptado/de prueba previo permanece intacto.

### Barra con asentamiento y carga sobre apoyo

Nodos en x=0,4,6 m; EA=400 N. Rigideces axiales 100 y 200 N/m.
u1=0, u3=s=0.03 m, u2 libre; los Uy se restringen. El equilibrio del nodo 2 es
100u2+200(u2−s)=λP, por lo que u2=(λP+6)/300 m.
Sin carga u2=0.02 m, R1=−2 N, R3=2 N. El caso A aplica P=12 N en nodo 2 y
Q=7 N directamente en el apoyo 1: R1=−100u2−λQ,
R3=200(s−u2). Con λ=2, u2=0.1 m, R1=−24 N, R3=−14 N;
la carga física total es 2(P+Q)=38 N y ΣF+ΣR=0. Los prescritos siguen valiendo
s, no 2s. Otro caso B con P=5 N verifica la superposición afín documentada.

### Voladizo con carga puntual y distribuida

E=200 Pa, I=3 m⁴, A=2 m², longitud L=4 m. Empotramiento en (2,3,0), extremo
en (6,3,0); así el brazo debe usar coordenadas y no índices nodales.
Con fuerza terminal −P en Y, EI v''=−P(L−x). Integrando con
v(0)=v'(0)=0: v(L)=−PL³/(3EI), θ(L)=−PL²/(2EI).
Ry=P y Mz=PL en el empotramiento. Respecto al origen,
ΣMz=−P·6+P·2+PL=0; respecto a p, se sustituye cada x por x−p_x.
Para q=−6 N/m constante, EI v''=−6(L−x)²/2; integrando:
v(L)=−6L⁴/(8EI), θ(L)=−6L³/(6EI), Ry=6L, Mz=6L²/2.
La prueba usa el vector de cargas equivalentes del elemento frame, incluidos
sus pares, y comprueba equilibrio físico global. Un backend que devuelve u/2
genera residual libre P/2, residual Fy=−P/2 e indicador relativo global 1/3;
se comprueba además M(p)=M(0)−p×ΣF.

### Pórtico simétrico con rigidez axial finita

Altura h=3 m, luz b=4 m; dos columnas empotradas y dintel rígidamente conectado.
Cada esquina superior recibe H/2=6 N en X. Simetría: ambos desplazamientos
horizontales Δ y giros θ son iguales; los verticales son v y −v. Sumando energía
de flexión de las columnas y dintel, y axial de las columnas:

```text
V = 12EI/h³ Δ² + 12EI/h² Δθ + (4EI/h + 6EI/b)θ²
  + (EA/h + 24EI/b³)v² + 24EI/b² vθ − HΔ
```

El dintel no se alarga bajo esta simetría. De ∂V/∂v=0,
v=−βθ, β=(12EI/b²)/(EA/h+24EI/b³).
De ∂V/∂θ=0, θ=−6EIΔ/(h²D), D=4EI/h+6EI/b−12EIβ/b².
Finalmente ∂V/∂Δ=0 da
Δ=H/(24EI/h³−72(EI)²/(h⁴D)). Las bases tienen Rx=−H/2 cada una;
Ry izquierdo=−EA v/h y derecho=EA v/h. Se comprueban Δ,v,θ, las reacciones,
ΣH+ΣRx=0 y el momento global, incluyendo el brazo de altura h.

### Barra espacial y entradas inválidas

Barra entre (1,2,3) y (3,5,7) m: L=√29, d=(2,3,4)/L. Solo Uz del extremo
queda libre; Kzz=EA·16/L³ y u_z=PL³/(16EA). Los apoyos toman las restantes
fuerzas y pares; se aplican además Rx=7, Ry=−8, Rz=9 N·m sobre GDL prescritos.
Las fuerzas de barra son colineales: su momento conjunto es cero. Los pares
aplicados se cancelan con reacciones de signo contrario. La prueba invierte el
orden de nodos y usa un punto de momentos fuera del origen.

Una barra horizontal D2 con Uy terminal libre carece de rigidez transversal:
el mecanismo debe informar ENG-E-0006 con nodo 2 y Uy, e índice global 3 en
entidades. Se comprueban listas vacías/duplicadas (ENG-E-0010), caso ausente
(ENG-E-0003), factores NaN/±∞ y desbordamiento (ENG-E-0008), factor cero y
negativo, punto no finito y diagnósticos inválidos del backend (ENG-E-0012).
Se omiten resortes porque engine-elements-spring no existe en main.

## Verificación local

Verificación del 2026-10-11, sin acceso a red:

- `cargo fmt` y `cargo fmt --check`: OK.
- `cargo clippy --all-targets -- -D warnings`: OK.
- `cargo test --workspace`: OK; diez pruebas de integración nuevas (ocho de
  análisis y dos de resolución por lotes), además de las pruebas existentes y
  la ampliación del ensayo de diagnóstico ENG-E-0012.
- `RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace`: OK.
- `cargo check --workspace --target wasm32-unknown-unknown`: OK.
- `cargo check --workspace --all-targets --target wasm32-unknown-unknown`: OK;
  incluye los fixtures analíticos y el backend contador.
- `bash clean-room/check.sh origin/main`: OK. Además se ejecutó el mismo
  script con metadatos e índice Git temporales en `/tmp`, compartiendo los
  objetos locales de solo lectura y registrando los archivos nuevos en ese
  índice temporal, para que también quedaran incluidos en el diff revisado.
  No se modificó el índice real ni se crearon commits.
- `git diff --check`: OK, incluido el diff con archivos nuevos de la comprobación
  temporal. Sin `panic!`, `.unwrap()` ni `.expect()` en engine-analysis/src.

Pruebas nativas, Clippy, formato y documentación: Cargo/rustc 1.85.1 del sistema.
El chequeo wasm con ese compilador falló por ausencia de la biblioteca std para
ese destino. Se resolvió usando el toolchain **ya instalado** `stable` de rustup
(rustc 1.99.0), con `/home/box/.cargo/bin` al inicio de PATH y
`RUSTUP_TOOLCHAIN=stable`; no se descargó nada. `cargo check` para wasm comprueba
compilación; no equivale a ejecutar los tests en un runtime wasm. La ejecución
en runtime wasm y la revisión clean-room de J4 siguen pendientes.
