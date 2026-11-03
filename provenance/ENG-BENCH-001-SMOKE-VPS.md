# ENG-BENCH-001 · Humo de OpenSees 3.8.0 en el VPS (MAM-253)

Registro de procedencia (plantilla "02 · Protocolo clean-room"). No hay código del Engine en este
cambio: solo se documenta el CI del repositorio de oráculos y una corrida manual de humo.

Autor del registro: sesión J2 (Grok Bot), 2026-10-20 (America/Guayaquil). Revisor clean-room: J4, PENDIENTE.

## (a) CI de mamut-engine-oracles#3

- Repositorio: `Mamut1195/mamut-engine-oracles`, PR #3, head `2d4322cc85c00d960a51d594e4e1a524f1e1c286`.
- Runs: `39471838998` (push) y `39471845334` (pull_request), ambos con conclusión `success`
  (verificado con `gh run view`), en el runner self-hosted `contabo-mamut-engine-oracles`.
- Resultado: ruff OK, pytest 134 passed, `check_no_packaging` OK.

## (b) Corrida manual de humo de OpenSees en el VPS

- Ejecutor: sesión J2, 2026-10-20, ≈13:09 ECT (mtime del script de humo `barra.py` en el VPS).
- Ruta de instalación: `/opt/opensees/3.8.0` (aislada; no se instalaron ni cambiaron paquetes del sistema).
- Origen: `openseespy` 3.8.0.0 + `openseespylinux` vía pip (PyPI) en un venv propio.
  BLAS/LAPACK/gfortran extraídos de paquetes `.deb` de Ubuntu obtenidos con `apt-get download`, sin instalarlos.
- sha256:
  - `bin/opensees-py`: `62e38e3270650ecf2d1420d3e505c5b4c556f90e41cf6c02f826313a1a0f2010`
  - `venv/lib/python3.12/site-packages/openseespylinux/opensees.so`:
    `b1bfa97ceb8f0f5d3c3c49eb498ac4e9dc57f86971f0f54c03074968a0f830a3`
- Humo: ok. Entrada del caso, salida, estado del adaptador y comando: [redactado: ver mamut-engine-oracles,
  rama `q2/bench-smoke-provenance`, `results/provenance/ENG-BENCH-001-SMOKE-VPS.md`, sección (b)].
  Por la opción A, esa sección solo la leen Q2 y J4; J2, J0 y las sesiones autoras no ven los valores.

## Declaraciones clean-room

- No se abrió ni se copió código fuente de OpenSees; solo se usó como ejecutable externo.
- Stabileo no se instaló ni se ejecutó.
- OpenSees no se incluye ni se empaqueta en ningún repositorio.
- Esta sesión escribió el script de humo, por lo que queda vetada para implementar la barra axial
  y el truss elástico. Desde la opción A, solo Q2 ejecuta los oráculos, en sesión aislada; J2 solo
  recibe "difiere en caso X".
