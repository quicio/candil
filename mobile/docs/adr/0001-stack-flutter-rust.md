# ADR-0001: Flutter + núcleo Rust compartido

- **Estado:** aceptado
- **Fecha:** 2026-11-16
- **Área:** arquitectura, móvil

## Contexto

Candil de escritorio tiene un parser EPUB, un modelo de bloques y una búsqueda
propios, sin WebView ni CSS. Se necesita Android e iOS con la misma lectura y,
si es posible, el mismo comportamiento.

Las opciones eran:

1. Flutter y reescribir todo en Dart.
2. Flutter con un núcleo Rust compartido.
3. Tauri 2 y una UI web.
4. SwiftUI + Jetpack Compose con dos UIs.

## Decisión

**Flutter para la UI y un crate Rust compartido para el núcleo.** La frontera es
una API C con respuestas JSON y punteros opacos. El núcleo vive en
`crates/candil-core` del monorepo; `mobile/rust` es solo el adaptador FFI.

- El parser y la búsqueda no se duplican.
- Una sola UI para Android e iOS.
- La UI conserva selección, arrastre, teclado y accesibilidad de Flutter.
- No se introduce `flutter_rust_bridge` en la primera etapa: el contrato C es
  pequeño, explícito y fácil de probar. Si la API crece, se puede migrar sin
  cambiar la UI.

## Consecuencias

- Hay que compilar Rust para cada ABI móvil (`cargo-ndk`, Xcode).
- La memoria de cada string/buffer se libera explícitamente desde Dart.
- El núcleo debe mantenerse sin dependencias de egui ni de escritorio.
- iOS requiere macOS/Xcode; Linux solo puede verificar el núcleo y Dart.
