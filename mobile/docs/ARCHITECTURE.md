# Arquitectura de Candil Mobile

## Decisión de stack

```text
Flutter (Dart) ─── FFI C/JSON ─── mobile/rust ─── crates/candil-core (Rust)
      │                                │
      ├─ pantallas y gestos            ├─ epub.rs
      ├─ almacenamiento JSON           ├─ xhtml.rs
      └─ respaldos                     └─ search.rs
```

- **Flutter** dibuja toda la UI y administra navegación, selección de archivos,
  almacenamiento y respaldos.
- **Rust** es la única fuente de verdad del parseo EPUB, del modelo de bloques y
  de la búsqueda de texto. No se duplica el parser en Dart.
- La frontera es una **API C estable** que devuelve JSON UTF-8. Evita generar
  código con `flutter_rust_bridge` en esta primera etapa y deja el ABI explícito.
- `BookHandle` es un puntero opaco con `Mutex<Book>` y un índice de búsqueda
  perezoso. Dart no conoce su layout.

## API C

| Función | Respuesta |
|---|---|
| `candil_open(path)` | `CandilBook { handle, error }` |
| `candil_metadata(handle)` | JSON con título, autor, idioma, serie, capítulos y TOC |
| `candil_chapter(handle, index)` | JSON con bloques y runs |
| `candil_cover(handle)` | Buffer con la portada |
| `candil_resource(handle, path)` | Buffer con una imagen del ZIP |
| `candil_search(handle, query, mode, max)` | JSON con hits y `truncated` |
| `candil_close(handle)` | Libera el libro |
| `candil_free_string` / `candil_free_buffer` | Liberan la respuesta |

Reglas de memoria: cada respuesta es propiedad del llamador. Dart siempre llama
al `free` correspondiente. Rust no usa `Box::leak` para responder.

## Flujo de lectura

1. `LibraryScreen` importa un EPUB y lo copia a `books/<sha256>.epub`.
2. `RustCore.open` crea el `BookHandle`; `metadata` llena la tarjeta y la TOC;
   `cover` genera la miniatura.
3. `ReaderScreen` pide un capítulo por vez y renderiza los bloques con Literata.
4. El scroll guarda `{chapter, ratio}` en `library.json` con debounce.
5. Las anotaciones guardan offsets de caracteres y cita, igual que escritorio.
6. Las imágenes se piden como bytes del ZIP y se cachean solo en memoria.

## Búsqueda

`candil_search` construye un `SearchIndex` la primera vez y lo conserva dentro
del `BookHandle`. El modo es frase exacta, todas las palabras o cualquiera. Los
hits traen capítulo, bloque y offsets para resaltar y saltar.

La búsqueda avanzada de toda la biblioteca queda para la siguiente fase: se
reutilizará el mismo núcleo y se paralelizará desde Dart con un límite de
trabajadores, igual que la mejora de escritorio.

## Almacenamiento móvil

- Sandbox de la app (`getApplicationDocumentsDirectory()/candil`).
- JSON atómico: temporal + `rename`.
- Copia de EPUB al sandbox para que el sistema no revoque el acceso.
- `library.json` conserva el esquema de escritorio y agrega `origin_path` cuando
  una entrada importada todavía no tiene archivo local. Escritorio ignora los
  campos desconocidos.
- Al importar el EPUB que completa una entrada faltante, se actualiza `path`, se
  conserva el progreso y se reasignan las anotaciones con `rekey`.

## Respaldos

`candil-backup.json` versión 1, con nombre, tamaño y SHA-256 de cada JSON. La
importación valida todo antes de escribir, guarda una copia previa y reemplaza
los cinco archivos. Nunca incluye EPUB, portadas ni claves.
