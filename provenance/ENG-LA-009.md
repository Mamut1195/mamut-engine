# Bitácora clean-room · ENG-LA-009

- Ticket / PR: ENG-LA-009, parte IMPLEMENTABLE «Autovalores generalizados K φ = λ M φ»; rama encontrada `j2/eng-la-009`. Sin commit ni push.
- Autor(es): J2 (agente, Grok Bot) · 2026-10-11 (America/Guayaquil), identificación solicitada por el usuario; ejecución de esta sesión por Codex (GPT-6). Sin delegación a otros agentes.
- Rol: implementador y autor de derivaciones analíticas propias.
- Contrato implementado: API interna `lowest_modes(k, m, count)` como método por defecto de `LinAlgBackend` y función libre; `LowestModes`, `sturm_count` y `symmetric_inertia`. Backend propio de referencia; sin cambio de contrato externo ni conexión con MemoryBudget (#38/#50).
- Fuentes permitidas usadas: ninguna externa. Contratos y código propios existentes de `engine-linalg`, `engine-elements-frame`, catálogo de `engine-core`, `docs/CLEAN_ROOM.md` y `provenance/TEMPLATE.md`; condiciones y valores β dados por el usuario.
- Derivaciones propias: notas analíticas de esta bitácora; implementación en `crates/engine-linalg/src/modes.rs` y ampliación de LDLᵀ en `reference.rs`; fixtures analíticos en `engine-linalg/tests/modes.rs` y `engine-elements-frame/tests/modes.rs`.
- Código de terceros copiado: ninguno.
- Validación: exclusivamente analítica; cadena masa-resorte exacta, espectros de matrices pequeñas, vibración de vigas Euler–Bernoulli con refinamiento de malla.
- Comparación con oráculo (repo hermano, informativa): no. Sin red, fuentes externas ni ejecución de motores externos.
- Declaración: 'No consulté código fuente de OpenSees, Stabileo, CalculiX, code_aster, OOFEM, carapace, anaStruct, Kratos, LAPACK, Eigen, faer, nalgebra ni derivados'
- Revisor clean-room: PENDIENTE (J4)

## Alcance y decisiones pendientes

Se conserva el camino denso existente, Cholesky de M y Jacobi para el espectro
completo, seleccionando después las primeras columnas. Se añade un control
independiente por congruencia LDLᵀ de K−σM. El método por defecto es compatible
con implementaciones anteriores del trait y funciona también mediante
`&mut dyn LinAlgBackend`. Sin solver disperso, iteración parcial ni cambios en
la contabilidad de memoria. No se elimina automáticamente ningún GDL sin masa.

**PENDIENTE (J0): cualquier dependencia externa nueva requiere su decisión.**
No se añade ninguna dependencia, externa ni local. **Faer PENDIENTE / no usado**:
forma parte de las bibliotecas que no se consultan en esta sesión por instrucción
del usuario, complementaria al protocolo de `docs/CLEAN_ROOM.md`; tampoco se
consulta ninguna de las otras bibliotecas de la declaración. Su feature stub y
la evaluación histórica de ENG-LA-007 se conservan. Una eventual decisión de J0
no se sustituye por la aprobación de esta implementación propia.

Para verificar vigas con el elemento real se ubica la integración en el crate
`engine-elements-frame`, que ya depende de `engine-linalg` y `engine-contracts`.
Así no se necesita añadir frame como dev-dependency inversa ni modificar manifests
o lockfile. Los tests ejercitan directamente la nueva API de linalg.

Se interpreta «los tres tickets» como ENG-LA-007, ENG-LA-008 y ENG-LA-009;
se registra en los dos anteriores una declaración adicional de esta sesión,
sin sustituir sus fuentes ni resultados históricos.

## Derivación del conteo de Sturm e inercia

Si M=LLᵀ es definida positiva, C=L⁻¹KL⁻ᵀ es simétrica y
K−σM=L(C−σI)Lᵀ. Por congruencia invertible, el número de signos negativos
de K−σM es el de λᵢ−σ<0: el número de autovalores estrictamente menores que σ.
LDLᵀ con permutaciones simétricas y equilibrado diagonal positivo conserva esa
inercia. No se utiliza una cadena de determinantes de menores principales,
que puede fallar sin pivoteo; se cuenta en D por bloques.

Un pivote escalar d<0 aporta uno. Para B=[[d,e],[e,g]], las raíces cumplen
μ²−(d+g)μ+(dg−e²)=0. Si det<0 hay un positivo y un negativo; si det>0
ambos signos son el de tr=d+g. Si det=0, una raíz es cero y la otra tr;
si det=tr=0, ambas son cero. Por ello no es correcto contar solamente
las entradas diagonales de un bloque 2×2. Se implementa clasificación por
determinante/traza; para el indicador de pivote se conservan los módulos
de tr/2±hypot((d−g)/2,e).

La elección propia de pivote usa el máximo diagonal si domina al máximo
fuera de diagonal por al menos 1/2. En otro caso, |d|,|g|<|e|/2 implica
dg−e²<0, de modo que el bloque 2×2 seleccionado es indefinido y regular.
Un bloque semidefinido de rango uno se reduce mediante pivote 1×1 y queda
un resto nulo. El camino de inercia admite ese resto cuando todas sus entradas
son ≤64 ε max(n,1) tras equilibrado y normalización; cuenta nulos y termina.
El camino que entrega factores para resolver sigue rechazando singularidades
con ENG-E-0006. Nunca se entrega el factor incompleto del camino de inercia.
El conteo estrictamente en una raíz se interpreta con esa resolución numérica:
no se promete signo exacto cerca de raíces en problemas mal condicionados.

## Modos faltantes y ceros

Para count>0 se exige espectro denso completo y ordenado, dimensiones n×n,
finitud, M definida positiva y ausencia de λ negativos significativos.
Después se comprueba Sturm(σ)=count. Se usa σ=λ_count(1+1e−7), con piso
λ_count+16·tol_cero y tol_cero=128 ε n max|λ|. Con espectro totalmente cero
se usa σ=1e−7. El piso impide evaluar σ=0 en los movimientos rígidos;
no regulariza las matrices y escala inversamente con una escala uniforme de M.

Si el conteo no coincide se devuelve ENG-E-0007 con «MODOS FALTANTES» y
los conteos y desplazamiento. Esto también rechaza count dentro de un grupo
múltiple (o muy próximo dentro del margen); solicitar el grupo completo
permite devolverlo. Una prueba simula un backend que omite el primer valor
y repite el último manteniendo el tamaño: el control independiente lo detecta.
Otra simula un espectro de longitud menor que n. No se devuelven menos modos.

Los |λ|≤tol_cero se representan como cero y se cuentan en `rigid_mode_count`.
Pueden ser movimientos rígidos o mecanismos: las matrices aisladas no permiten
decidir si corresponden a un movimiento rígido permitido o a conectividad/
liberaciones incorrectas. Esto se documenta explícitamente en el resultado;
en un análisis estático un mecanismo corresponde a ENG-E-0006. No se rechaza
K semidefinida, requisito de esta entrega. λ<−tol_cero se rechaza con
ENG-E-0010, diagnóstico de K indefinida/posible inestabilidad, sin producir
frecuencias imaginarias. Masa singular: ENG-E-0006; masa indefinida,
órdenes incompatibles o count>n: ENG-E-0010. count=0 devuelve n×0 sin
factorización ni comprobación de positividad de M, validando los órdenes.
ENG-E-0008 cubre entradas o resultados no finitos.

La normalización reusa φ/√(φᵀMφ), con signo dominante positivo. Al cambiar
M→cM, λ→λ/c y φ→φ/√c: las formas modales se conservan. La tolerancia de
cero es numérica, relativa al espectro completo; no separa por criterio físico
autovalores pequeños de grandes en espectros extremadamente mal condicionados.

## Soluciones analíticas de los tests

Cadena de n masas iguales entre extremos fijos, resortes k y masa m:
qⱼ=sin(jθ), θ=rπ/(n+1), q₀=qₙ₊₁=0. Al sumar senos vecinos,
−qⱼ₋₁+2qⱼ−qⱼ₊₁=2(1−cosθ)qⱼ=4sin²(θ/2)qⱼ, luego
λᵣ=4(k/m)sin²(rπ/[2(n+1)]). Tests con n=1,4,9, tolerancia relativa
2e−12 en λ y M-ortogonalidad absoluta 2e−12. Residual modal relativo
2e−11, con piso absoluto para modos nulos. Conteo de Sturm exacto en
intervalos entre raíces; matrices diagonales y bloques 2×2 verifican además
igualdad estricta en las raíces, multiplicidades e inercia singular.

Viga homogénea: EI w''''+ρA ẅ=0. Para w=W(x)e^{iωt},
W''''=β⁴W, β⁴=ρAω²/EI, solución combinación de cos, sin, cosh, sinh.
Simplemente apoyada: W=W''=0 en ambos extremos ⇒ βᵣL=rπ.
Voladizo: W=W'=0 en x=0 y W''=W'''=0 en x=L ⇒
cos(βL)cosh(βL)+1=0. Raíces indicadas por el usuario (1.8751, 4.6941,
7.8548), conservadas con más cifras para evitar que su redondeo domine el error.
Frecuencia fᵣ=(βᵣL)²√(EI/(ρA))/(2πL²).

Se ensamblan K y M consistentes del elemento propio EulerFrame2D para
E=ρ=A=I=L=1; se seleccionan solamente uy/rz porque la viga horizontal
desacopla flexión y axial. Simplemente apoyada restringe uy en ambos extremos;
voladizo restringe uy/rz en el primero. Mallas de 4 y 12 elementos Hermite;
los tres primeros errores relativos en f deben mejorar al menos ×10 y quedar
<3e−4 en la malla fina. No se mezcla inercia rotatoria ni deformación de corte
con esta solución de Euler–Bernoulli. Estos son tests de convergencia de malla,
no de exactitud del elemento a malla finita.

## Verificación

PENDIENTE de completar tras ejecutar los comandos solicitados; se usan
dependencias locales/caché, sin red (`CARGO_NET_OFFLINE=true`). No se ejecuta
wasm: la comprobación solicitada verifica compilación para ese destino.
