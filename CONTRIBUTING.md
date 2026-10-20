# Contribuir a MAMUT Engine (nombre provisional)

Toda contribución debe seguir el [protocolo clean-room](docs/CLEAN_ROOM.md).

Para implementar un método en Rust, sigue la [guía de métodos](docs/GUIA_METODOS.md):
incluye plantillas, revisión y un piloto uniaxial reproducible.

Los contribuyentes externos deben firmar un **CLA obligatorio** antes de que se acepte su contribución. Este requisito responde a la licencia dual **AGPL-3.0 + licencia comercial**: el titular debe poder distribuir las contribuciones tanto bajo AGPL como bajo licencia comercial.

El CLA se documentará en [docs/CLA.md](docs/CLA.md), que llegará con **ENG-GOV-007**. El texto del CLA es **PENDIENTE de revisión legal**.

Las decisiones fundacionales se registran en el [ADR-001](docs/adr/ADR-001-fundacionales.md). El titular del copyright sigue **PENDIENTE**. El texto de licencia y el aviso legal viven solo en Notion, en **ENG-GOV-004**: **PENDIENTE (ver Notion)**.

## Pruebas

El toolchain está fijado a Rust **1.99.0** en [rust-toolchain.toml](rust-toolchain.toml), con rustfmt, Clippy y el destino `wasm32-unknown-unknown`. Usa rustup desde la raíz para ejecutar las mismas comprobaciones que CI:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
RUSTDOCFLAGS='-D warnings' cargo doc --no-deps --workspace
cargo check --workspace --target wasm32-unknown-unknown
```
