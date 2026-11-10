# 17 · Ruta densa sin cambio de hashes

- Estado: abierto. Aprobado por J0 el 2026-11-10, prioridad High.
- Relación: MAM-1064, ENG-PERF-002. Parte del head de #33 (MAM-1056). Esta ficha no cierra MAM-107 y no aprueba un presupuesto de tiempo o memoria.
- Reglas comunes: [README.md](README.md).
- Número: en las ramas remotas `j2/*` consultadas al abrir la ficha no hay un archivo `17-*`. El 12, el 13, el 14 y el 15 tampoco aparecen. El 17 está libre y es el que usa esta ficha.

## Qué ya existe

El perfil de MAM-1056 midió el pórtico `portal_250` y dejó seis propuestas. La fachada estática llama al solver una vez por caso de carga, y cada llamada factoriza K. P-Delta factoriza K en la gravedad y vuelve a factorizar esa misma K para la sustitución de Picard. La LDLᵀ de referencia recorre toda la matriz con un chequeo de finitud al cerrar cada columna. El armado del sistema densifica K, las dos masas y Kg aunque el estático solo lee K.

## Qué hacer

Aplicar solo las propuestas (2), (3), (4) y (5) de [MAM-1056-perfil-ruta-densa.md](../perf/MAM-1056-perfil-ruta-densa.md):

- (2) En el estático, K se factoriza una sola vez y esa factorización se reusa para todos los casos de carga.
- (3) En P-Delta, K no se vuelve a factorizar después de la gravedad.
- (4) Se quita el repaso de finitud de toda la matriz en cada columna de la LDLᵀ. Queda un solo chequeo de finitud antes de factorizar. El error, con el mismo código y el mismo mensaje, y el punto donde se detecta un NaN o un Inf, no cambian. Una prueba lo fija.
- (5) El estático no densifica M ni Kg, porque no los usa.

Quedan fuera la propuesta (1), el núcleo LDLᵀ, y la (6), el Jacobi del modal. No se cambia el orden de las sumas ni la secuencia de pivotes.

## Qué no puede cambiar

Los 58 golden de `crates/engine-cli/tests/golden/stdout-sha256.json`, los 16 digestos de determinismo nativo/wasm y el digesto de describe salen idénticos. Si alguno cambia, el trabajo se para: no se actualiza ningún golden. Lo que no se mida queda como DESCONOCIDO.

## Hecho cuando

- La ficha está en el índice y el 17 sigue siendo el número usado.
- Las cuatro propuestas están en el motor y la prueba de NaN e Inf compara el error con el de la base.
- La nota de tiempos trae la mediana de cinco repeticiones en release, antes y después, para las fases del estático, el run completo del estático y el run completo del P-Delta en `portal_250`.
- El fragmento de changelog y la bitácora `provenance/ENG-PERF-002.md` están escritos. El revisor clean-room queda en PENDING.
