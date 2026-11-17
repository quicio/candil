# AGENTS.md — Candil Mobile

Léase junto con `README.md`, `docs/ARCHITECTURE.md` y los ADR.

## Comandos

```bash
source tool/env.sh

cargo test --manifest-path rust/Cargo.toml
cargo clippy --manifest-path rust/Cargo.toml --all-targets -- -D warnings
flutter analyze
flutter test

cargo build --manifest-path rust/Cargo.toml
CANDIL_CORE_LIB="$PWD/../target/debug/libcandil_mobile_core.so" \
  dart run tool/smoke_ffi.dart rust/tests/fixtures/mini.epub
```

## Reglas duras

- El parser EPUB, el modelo de bloques y la búsqueda viven en `rust/`. No
  dupliques esa lógica en Dart.
- Toda respuesta FFI se libera en Dart (`candil_free_string`/`candil_free_buffer`)
  y cada libro se cierra con `candil_close`. No agregues `Box::leak`.
- Los JSON de usuario mantienen el formato de escritorio: `config.json`,
  `library.json`, `annotations.json`, `consultas.json` y `studies.json`. Los
  campos desconocidos se conservan al guardar.
- `origin_path` es la ruta de escritorio de un libro importado que todavía no
  está en el móvil. Al importar el EPUB, actualiza `path` y reasigna las
  anotaciones; no borres la entrada.
- Importar copia el EPUB al sandbox (`books/<sha256>.epub`); no se guardan rutas
  externas como identidad permanente.
- Escritura de JSON con temporal + `rename`, nunca `writeAsString` directo.
- El respaldo incluye solo los cinco JSON. Nunca EPUB, portadas, caché ni claves.
- No agregues consultas online, IA ni claves sin acción explícita del usuario.
  Las claves irán al llavero, nunca a JSON ni logs.
- No subas artefactos: `rust/target`, `build`, `jniLibs`, `ios/Frameworks`.
- Este repo no compila iOS en Linux. No prometas builds de iPhone sin macOS/Xcode.
