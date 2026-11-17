# Candil Mobile

Lector de EPUB minimalista para **Android e iOS**, con la misma idea que
[Candil de escritorio](..): una columna, scroll continuo, tipografía
Literata, temas de papel y una biblioteca sin ruido.

Vive en `mobile/` del monorepo `candil`, junto a `crates/candil-core`. La UI es
Flutter; el parser y la búsqueda son el mismo crate Rust que usa escritorio.

La UI está en **Flutter** y el parser EPUB, el modelo de bloques y la búsqueda
viven en un **núcleo Rust compartido** con la app de escritorio. La
comunicación es una API C mínima con respuestas JSON, sin WebView.

## Estado

Esta es la **primera base funcional** (0.1.0). No es todavía paridad completa
con escritorio.

### Incluido

- Biblioteca local: importar uno o varios EPUB, portada, metadatos, búsqueda
  por título/autor/serie/archivo, orden por recientes/título/autor y borrado.
- Lector: índice, capítulos, párrafos, encabezados, citas, listas, imágenes,
  progreso por capítulo y scroll, búsqueda dentro del libro y salto al pasaje.
- Tipografía y temas: Literata embebida, tamaño, interlínea, espacio entre
  párrafos, modo enfoque con bordes de capítulo legibles y los cinco temas de
  Candil.
- Anotaciones básicas: mantener pulsado un párrafo para destacarlo, agregar una
  nota o quitarlo; lista de anotaciones por libro.
- Datos compatibles con escritorio: `config.json`, `library.json`,
  `annotations.json`, `consultas.json` y `studies.json`; respaldo ZIP con
  manifiesto y SHA-256, copia previa antes de importar.
- Al importar un respaldo de escritorio, los libros que aún no están en el
  móvil quedan como faltantes con `origin_path`; al elegir el EPUB se recupera
  la entrada, el progreso y se reasignan las anotaciones.

### Siguientes fases

- Búsqueda avanzada en todo el contenido de la biblioteca, en paralelo, como en
  escritorio.
- Modo estudio, IA, diccionario y traducción; claves en el llavero del sistema y
  consultas online apagadas por defecto.
- Compartir citas como imagen.
- Caché local de EPUB remotos y descarga offline.
- Importación/exportación por compartir del sistema; firma y publicación en
  Play Store/App Store.
- Sincronización entre dispositivos (no incluida: no hay nube propia).

### Diferencias deliberadas con escritorio

- En móvil, importar un EPUB **copia** el archivo a la carpeta privada de la
  app. No hay carpetas vigiladas ni acceso permanente a carpetas arbitrarias:
  Android/iOS limitan ese acceso.
- No se incluyen actualización automática, envío a Kobo ni edición del EPUB.
- Las claves de IA no se almacenan en JSON. Esa parte se implementará con
  `flutter_secure_storage`/llavero cuando llegue el modo estudio.

## Requisitos

- Flutter 3.50 o compatible, Dart 3.14+.
- Rust estable con Cargo.
- Android: Android SDK, NDK, `cargo-ndk` y los targets de Android.
- iOS: macOS con Xcode y los targets de Rust para iOS. **iOS no se puede
  compilar en Linux.**

En esta máquina ya quedó instalado:

- Flutter 3.50 en `$HOME/.local/flutter-sdk/flutter`.
- JDK 17 en `$HOME/.local/jdk-17` (Flutter lo tiene configurado).
- Android SDK en `$HOME/Android/Sdk`: platform 36, build-tools 36, NDK
  `28.2.13676358`, platform-tools, CMake y emulador.
- `cargo-ndk` y los targets Rust de Android e iOS.
- AVD `candil` (Android 36 x86_64) y la app de prueba ya instalada.

Abrí una terminal nueva o ejecutá:

```bash
source tool/env.sh
```

`flutter doctor` puede seguir marcando Chrome y el toolchain Linux desktop:
no son necesarios para Android/iOS.

## Desarrollo y verificación

```bash
export PATH="$HOME/.local/flutter-sdk/flutter/bin:$PATH"

# Núcleo Rust
cd rust
cargo test
cargo clippy --all-targets -- -D warnings
cd ..

# Flutter
flutter pub get
flutter analyze
flutter test

# Prueba real del puente FFI en Linux (compila el cdylib y lo consume Dart)
cargo build --manifest-path rust/Cargo.toml
CANDIL_CORE_LIB="$PWD/../target/debug/libcandil_mobile_core.so" \
  dart run tool/smoke_ffi.dart rust/tests/fixtures/mini.epub
```

## Android

```bash
source tool/env.sh
tool/build_android_rust.sh
flutter emulators --launch candil   # o conectá un dispositivo por USB
flutter run
# Atajo: compila el núcleo, levanta el AVD candil si hace falta y ejecuta:
tool/run_android.sh
# APK de distribución (firmado con debug en esta base):
flutter build apk --release
```

El script copia `libcandil_mobile_core.so` a
`android/app/src/main/jniLibs/<abi>/`. Si el archivo no está, la app inicia pero
falla al abrir un libro; el mensaje de error recuerda compilar el núcleo.

## iOS

Los targets Rust de iOS ya están instalados, pero **la compilación iOS solo se
puede ejecutar en macOS con Xcode**. En Linux no hay linker ni SDK de Apple.

En macOS:

```bash
tool/build_ios_rust.sh
open ios/Runner.xcworkspace
```

Luego, en Xcode:

1. Agregar `ios/Frameworks/CandilCore.xcframework` al target **Runner**.
2. Marcar **Embed & Sign**.
3. En **Other Linker Flags**, agregar
   `-force_load $(PROJECT_DIR)/Frameworks/CandilCore.xcframework/ios-arm64/libcandil_mobile_core.a`.
4. Compilar y ejecutar con `flutter run`.

`RustCore` usa `DynamicLibrary.process()` en iOS/macOS porque la biblioteca se
enlaza estáticamente al binario.

## Arquitectura

```
lib/                  UI Flutter
  src/models.dart     modelos compatibles con JSON de escritorio
  src/data_store.dart JSON atómico, respaldos y migración de rutas
  src/rust_core.dart  FFI tipado sobre la API C
  src/screens/        biblioteca, lector, anotaciones y ajustes
rust/                 adaptador FFI móvil
  src/ffi.rs          API C: abrir, metadatos, capítulo, portada, recurso, buscar
../crates/candil-core/  parser EPUB, bloques y búsqueda compartidos
  src/epub.rs         ZIP → container.xml → OPF → spine/TOC
  src/xhtml.rs        XHTML → bloques y runs
  src/search.rs       índice y búsqueda frase/palabras
docs/                 arquitectura, roadmap y ADR
tool/                 compilación del núcleo para Android/iOS y smoke FFI
```

La API C devuelve JSON y punteros opacos `BookHandle`. Dart libera cada string y
buffer con `candil_free_string` / `candil_free_buffer`; el lector cierra el libro
con `candil_close`.

## Datos y respaldo

Los datos viven en la carpeta de documentos privada de la app:

```
candil/
  config.json
  library.json
  annotations.json
  consultas.json       (se conserva al importar; aún sin UI)
  studies.json         (se conserva al importar; aún sin UI)
  books/               copias de los EPUB importados
  covers/
  exports/
  backups/
```

El respaldo es un ZIP con `candil-backup.json` y los cinco JSON. No incluye
EPUB, portadas ni claves. La importación valida formato, tamaño y checksum, y
crea un respaldo previo en `backups/`.

## Licencia

MIT, igual que Candil.
