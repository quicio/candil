# candil-core

Núcleo Rust portable compartido por Candil escritorio y Candil Mobile.

Contiene únicamente:

- `epub`: lectura de ZIP/OPF, spine, TOC, capítulos e imágenes.
- `xhtml`: modelo intermedio de bloques y runs.
- `search`: índice y búsqueda de texto.

No depende de egui, Flutter, `dirs`, `keyring`, `rfd` ni de una plataforma.
Los adaptadores de UI, archivos, respaldos y claves viven en cada aplicación.

```bash
cargo test
cargo clippy --all-targets -- -D warnings
```

Consumo local desde los dos proyectos:

```toml
candil-core = { path = "crates/candil-core" } # raíz del monorepo
candil-core = { path = "../../crates/candil-core" } # mobile/rust
```
