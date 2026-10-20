# Checklist de revisión de un método Rust

- [ ] Ficha anterior al código, con fuentes locales permitidas y citas completas;
  derivación propia identificada, sin tablas/figuras/listados copiados.
- [ ] Roles y sesión limpia registrados; ninguna consulta a código, datos o
  informes prohibidos; revisión J4 diferenciada de las pruebas del autor.
- [ ] Método distinto de los existentes; hipótesis, SI, signos, dominio físico
  y numérico, historia y puntos no diferenciables explícitos.
- [ ] Trait vigente respetado; tangente derivada a historia fija, origen y
  aceptación/descarte correctos; salidas finitas y estado del llamador intacto.
- [ ] Constructor valida parámetros y longitud antes de indexar; errores del
  catálogo con mensajes en español; sin unsafe ni unwrap/expect/panic nuevos
  en biblioteca; documentación de toda API pública.
- [ ] Descriptor, versión, fábrica y exportaciones coinciden con el objeto;
  Send + Sync, sin estado global, E/S ni hilos en la ley.
- [ ] Conformidad sin diagnósticos; muestras/h justificados, fronteras
  verificadas por separado; negativos de rangos, tamaños, no finitos y overflow.
- [ ] Benchmark nativo con source/cita, esperados analíticos independientes
  fijados antes de correr y tolerancias justificadas; tabla no vacía.
- [ ] Si procede fixture/1: entrada API real, source válido, JSON Pointers y
  ejecución con runner ENG-CORE-009; si no procede, límite documentado.
- [ ] fmt, Clippy -D warnings, tests workspace, rustdoc -D warnings, check
  wasm y clean-room origin/main en verde, incluyendo archivos nuevos.
- [ ] 16 hashes idénticos a la referencia; cruce nativo/wasm debug/release
  registrado sin actualizar hashes para ocultar diferencias.
- [ ] Sin cambios en rutas protegidas ni dependencias/manifests injustificados;
  ningún cambio numérico en métodos existentes.
- [ ] CONTRIBUTING/guía, CHANGELOG y provenance completos en español, sin rutas
  locales absolutas; logs con marcador `$CARGO_TARGET_DIR`.
- [ ] Conteo passed/failed y límites de ejecución reportados; revisor clean-room
  **PENDIENTE (J4)** hasta revisión efectiva. Merge requiere aprobación de Jonathan
  según el protocolo; esta checklist no la concede.
