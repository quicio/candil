---
id: adr-0011
title: HTTP para consultas externas
status: aceptada
version: 1.1.0
updated: 2026-11-06
---

# ADR-0011 — HTTP para diccionario, traducción e IA

## Contexto

La spec 60 necesita hacer HTTPS + JSON para Wiktionary, MyMemory y OpenCode Go.
El lector es una app egui de escritorio, sin runtime async ni webview, con un
presupuesto de binario < 8 MB (hoy 5.599.328 B) y la regla de no agregar
dependencias sin justificarlas.

Las peticiones son pocas, explícitas y de una sola respuesta (no streaming en el
MVP). La UI no puede bloquearse mientras espera.

## Decisión

Usar **`ureq` 3.5** con **rustls** (`default-features = false, features =
["rustls"]`), bloqueante, sin runtime async.

- Las llamadas se ejecutan en un `std::thread` por consulta y el resultado viaja
  por `std::sync::mpsc` al hilo de UI.
- Timeout global configurable (30 s por defecto), errores HTTP concretos y
  request id para descartar respuestas viejas.
- `serde_json` (ya presente) parsea las respuestas.
- Se envía `User-Agent: lector/<versión>` y, para OpenCode Go,
  `x-opencode-session` estable por sesión.
- Las claves van en headers, nunca en la URL ni en logs.

## Alternativas descartadas

- **`reqwest` + tokio**: maduro, pero arrastra runtime async y un árbol de
  dependencias grande para cuatro endpoints. No se necesita streaming.
- **`curl` por subprocess**: cero dependencia Rust, pero depende de curl del
  sistema, complica el manejo de errores y expone datos sensibles en argumentos
  si no se usa stdin. Menos portable.
- **Solo APIs locales/offline**: descartado por el alcance del MVP
  (Wiktionary/MyMemory/OpenCode Go son online).
- **Webview/JS**: prohibido por la arquitectura del lector.

## Consecuencias

- `dist` crece por TLS/rustls y ureq. Hay que medirlo cuando el código use la
  librería (antes del primer uso el linker la elimina). Si supera el presupuesto,
  se reevalúa con `curl` o un cliente más chico antes de aceptar.
- Sin gzip: las APIs devuelven texto/JSON chico; se evita `flate2` y se reduce
  el binario. Si un proveedor exige compresión, se reevalúa.
- Sin async: una consulta en vuelo por panel; los botones se deshabilitan.
- Los certificados usan `webpki-roots` (portable), no el almacén del sistema.
  Si un usuario corporativo necesita un CA propio, se evalúa
  `rustls-platform-verifier` o `native-tls` en una fase posterior.
- Tests sin red: los parsers se prueban con fixtures JSON; la capa HTTP se
  inyecta con un trait/mock.

## Medición (2026-11-06)

- `dist` antes de integrar consultas (M5): **5.599.328 B**.
- `dist` con `ureq`/rustls y la UI de consultas: **6.664.128 B**.
- Delta: **+1.064.800 B**; sigue por debajo del presupuesto de 8 MB.

La medición se hizo con `cargo build --profile dist` después de que
`src/consulta.rs` usara la librería (antes el linker la eliminaba). El live test
de Wiktionary/MyMemory pasó; OpenCode Go devolvió 401 con una clave inválida y
la UI mostró el error sin bloquearse.

## Revisar si

- El binario supera 8 MB o `cargo tree` muestra un peso desproporcionado.
- Se necesita streaming de IA (ahí conviene evaluar un cliente con SSE/async).
- Se necesita un proxy corporativo o CA propio.
- Los proveedores cambian a HTTP/2 obligatorio o compresión obligatoria.
