---
id: roadmap
status: vigente
version: 1.11.0
updated: 2026-11-09
---

# Roadmap

Doce milestones. Cada uno tiene criterio de salida **verificable**, no una
lista de deseos. Un milestone no se cierra si su criterio no se puede mostrar.

| # | Milestone | Estado | Specs | Criterio de salida |
|---|---|---|---|---|
| M1 | Lector | **Hecho** | [10](specs/10-lector-mvp.md) | 43 EPUB abren sin capítulos vacíos; 2279/2279 imágenes; temas y progreso persistentes |
| M1.1 | Endurecimiento | **Hecho** | [15](specs/15-endurecimiento-lector.md) | Espacios, entidades, pre/tablas, SVG raster y config atómica verificados sobre el corpus |
| M2 | Biblioteca | **Hecho** | [20](specs/20-biblioteca.md), [21](specs/21-metadatos-y-portadas.md) | Escanear una carpeta de 50 EPUB en ≤ 5 s y verla como estantería con portadas, orden, filtro y apertura |
| M3 | Lectura fina | **Hecho** | [30](specs/30-indice-por-ancla.md), [35](specs/35-preferencias-por-libro.md) | Click en cualquier entrada del índice aterriza en el párrafo exacto, incluso si todo el libro es un solo XHTML |
| M4 | Extras | **Hecho** | [40](specs/40-busqueda.md), [45](specs/45-citas-y-marcadores.md) | Buscar una frase en el libro y saltar al resultado en < 300 ms |
| M5 | Segunda etapa | **Hecho** | [50](specs/50-anotaciones-globales.md), [55](specs/55-navegacion.md) | Ver y saltar anotaciones de cualquier libro, elegir color, exportar Markdown y saltar por porcentaje/capítulo |
| M6 | Consultas externas | **Hecho** | [60](specs/60-diccionario-traduccion-ia.md), [65](specs/65-centro-configuracion.md) | Seleccionar una palabra y obtener diccionario/traducción; seleccionar una frase y analizarla con IA sin bloquear la UI |
| M7 | IA configurable | **Hecho** | [ai-providers](../openspec/specs/ai-providers/spec.md), [ai-credentials](../openspec/specs/ai-credentials/spec.md) | Elegir entre nueve proveedores y guardar la API key en el llavero sin escribirla en `config.json` |
| M8 | Editar EPUB | **Hecho** | [epub-editing](../openspec/specs/epub-editing/spec.md) | Corregir título/autor/idioma y agregar portada con backup y validación |
| M9 | Lectura de estudio | **Hecho** | [reading-workspace](../openspec/specs/reading-workspace/spec.md) | Seleccionar y anotar entre párrafos del mismo capítulo, con migración v2 y reanclaje por partes |
| M10 | Lectura de estudio II | **Hecho** | [reading-workspace](../openspec/specs/reading-workspace/spec.md) | Previsualizar una nota o enlace interno sin salir del texto y volver al punto exacto |
| M11 | Actualizaciones | **Hecho** | [app-updates](../openspec/specs/app-updates/spec.md) | Comprobar una release a pedido, descargar el binario correcto y verificar SHA-256 sin autoinstalar |
| M12 | Entrega automática | **Hecho** | [RELEASING](../docs/RELEASING.md), [ADR-0017](../docs/adr/0017-entrega-automatica.md) | Release automático al pasar CI, `.app.zip` ad-hoc y cask de Homebrew |

## M1 — Lector (terminado)

Contrato vigente, detallado en `specs/10-lector-mvp.md`: parseo OPF+spine,
índice nav/ncx, modelo `Block`, render en columna centrada con scroll continuo,
5 temas, Literata embebida, progreso por libro.

Deuda conocida y aceptada: índice por archivo (no por ancla), sin CSS del libro,
sin paginado.

## M1.1 — Endurecimiento (terminado)

Spec [15](specs/15-endurecimiento-lector.md). Cerró los bordes que la auditoría
del 2026-11-05 encontró en libros reales: espacios en límites inline, entidades
Latin-1/griegas/matemáticas, `<pre>` con indentación, tablas/`<dl>` sin pegar
celdas, `<svg><image>` raster y escritura atómica de `config.json`.

Verificación: 43 EPUB, 0 fallos, **0 capítulos vacíos**, 2279/2279 imágenes;
25 unit + 7 integración al cerrar M1.1 (hoy 103 + 9 (1 live ignorado) con M2–M7); clippy sin
warnings. Capturas de `flores-del-mal`
(título) y `Elantris`/`El camino de los reyes` (mapas SVG) revisadas.

## M2 — Biblioteca (terminado)

Implementado y medido el 2026-11-06 sobre `~/Libros` (43 EPUB en la medición; hoy
quedaron 40 tras limpiar duplicados):

- Modelo `library.json` con `schema_version`, rutas canónicas, `size`/`mtime`,
  `missing` y progreso; escritura atómica compartida en `src/store.rs`.
- Escaneo recursivo incremental: **1.57 s** la primera vez, **0.02 s** con caché.
- Portadas cacheadas: **41/43 (95,3 %)**, **4,2 MB** en `covers/`.
- Pantalla con grilla, apertura, `Esc` para volver, búsqueda incremental, 4
  órdenes y 4 filtros; estados vacío/sin resultados; marcador cuando no hay
  portada.
- `--scan <carpeta>` para probar el núcleo sin UI; `--resume` para abrir el
  último libro (el arranque normal es la biblioteca).

- Migración de progreso a `library.json` (T8): `config.json` ya no recibe nuevas
  posiciones y las viejas se vuelcan al arrancar/después de escanear.
- Gestión de carpetas y errores (T9): chips con `×` para dejar de vigilar,
  contador de errores con tooltip y estados vacíos diferenciados.
- Verificación reproducible (T10): `--screen library`, `--library-path` y
  `--search`; 4 capturas en `docs/img/` (vacía, poblada, búsqueda, error).

Cierre (T11): corpus de **50 EPUB** (43 reales + 7 copias) escaneado en
**1.65 s** la primera vez y **0.01 s** con caché; **48/50 portadas (96 %)** y
**5,4 MB** en `covers/` (incluye 7 copias de una portada pesada; el corpus real
dio 98 KB/libro). `--inspect` sobre los 43 reales: 0 fallos, 0 capítulos vacíos,
2279/2279 imágenes. `cargo test`: 103 + 9 (1 live ignorado).

Objetivo original: dejar de abrir un archivo por vez. Una estantería que indexa
carpetas del disco, muestra portadas y permite abrir, ordenar, filtrar y buscar.

Entra:

- Índice persistente en `library.json` (ADR-0007): rutas vigiladas + entradas.
- Extracción de metadatos y portada al importar (ADR-0008).
- Grilla de portadas con título, autor y progreso.
- Orden (reciente, título, autor), filtro por idioma/estado, búsqueda incremental.
- Abrir desde la grilla y volver con `Esc`.
- Migración del progreso existente de `config.json` a `library.json`.

No entra (a M3/M4): índices por ancla, paginado, notas, sincronización, edición
de metadatos, estantes manuales.

Criterio de salida, medible:

1. Carpeta con 50 EPUB escaneada en ≤ 5 s (con caché de portadas).
2. ≥ 90% de los libros con portada visible (los que declaran una).
3. Reabrir la app no reescanear: biblioteca al instante desde `library.json`.
4. `cargo test` verde + capturas de estantería vacía, llena y filtrada.
5. `--inspect` sobre el corpus sin regresiones.

## M3 — Lectura fina (terminado)

Cerrado el 2026-11-06 con specs 30 y 35:

- `Block.anchor`, fragmentos en `TocEntry`, dedupe por `(spine, anchor)`, mapa por
  capítulo, saltos intra/intercapítulo y enlaces internos clicables.
- `Progress {chapter, ratio, anchor}` con migración del `offset` viejo; el ancla
  de bloque manda al reabrir y el ratio ajusta dentro del bloque.
- Preferencias de tema/tipografía por libro con precedencia
  `entry.settings → config.settings`, checkbox "Solo para este libro" e
  indicador `Aa·`.
- Verificación: 52 unit + 8 integración al cerrar (hoy 69 + 9); capturas de tema propio, panel de
  tipografía y mismo párrafo con fuente 27 → 17. `flores-del-mal` lista 8
  entradas y el ancla `#sigil_toc_*` de *Marco Aurelio* aterriza en su sección.

Queda como deuda menor (no bloquea M3): salto a porcentaje exacto desde la barra
y salto por número de capítulo; hoy se navega con las flechas y el índice.

## M4 — Extras (terminado)

Cerrado el 2026-11-06 con specs 40 y 45:

- **Destacados y notas (spec 45):** selección dentro de un párrafo, barra
  `Destacar`/`Nota`, `annotations.json` con offsets de caracteres + cita, panel
  por libro con salto/edición/borrado y reanclaje por cita.
- **Búsqueda en el libro (spec 40):** índice efímero por bloques, normalización
  1:1 (sin acentos/mayúsculas), resultados con contexto, salto al match, `n`/`N`
  y resaltado en el texto. Índice caliente < 300 ms; construcción inicial
  medida en 0.65 s para GEB (14 MB) incluyendo captura.
- Verificación: **69 unit + 9 integración**, clippy sin warnings, capturas de
  destacado/panel/nota/búsqueda.

Fuera del MVP y para una segunda etapa: índice global de anotaciones, varios
colores, export Markdown, citas exportables y búsqueda con regex/stemming.

## M5 — Segunda etapa (terminado)

Cerrado el 2026-11-06 con specs 50 y 55:

- **Anotaciones globales (spec 50):** selector “Este libro / Todos”, salto a
  anotaciones de otros libros, tres colores (acento, ámbar, verde) elegibles en
  la barra flotante y el editor de nota, y exportación Markdown con libro,
  capítulo, cita y nota.
- **Navegación (spec 55):** ventana “Ir a…” con slider de porcentaje y número de
  capítulo; se abre con `G` o click en el porcentaje de la barra.
- Verificación: **74 unit + 9 integración**, clippy sin warnings, capturas de
  destacados multicolor, panel global e “Ir a…”.
- Queda fuera: regex/stemming, sincronización, export a PDF/DOCX y búsqueda
  dentro de las notas.

## M6 — Consultas externas (terminado)

Cerrado el 2026-11-06 con specs 60 y 65:

- **Diccionario/traducción (spec 60):** Wiktionary y MyMemory en vivo,
  normalización de idioma, panel con contexto, resultados y enlaces de fuente.
- **IA (spec 60):** OpenCode Go (`inference/go/openai/v1`), prompts predefinidos,
  petición en hilo de trabajo y errores claros. La clave se guarda en el llavero
  del sistema; `OPENCODE_API_KEY` sigue como override. Probado con 401 real;
  falta la prueba de una respuesta exitosa con la clave del usuario.
- **Historial:** `consultas.json` atómico, local, con límite y borrado; guarda
  consultas y errores, no claves ni el párrafo completo.
- **Configuración (spec 65):** centro con Lectura, Biblioteca, Anotaciones,
  Consultas, Privacidad y Avanzado; `⌘,`/Ajustes. Online apagado por defecto.
- Medición: `dist` **6.664.160 B** (ureq+rustls sumó 1.064.832 B; < 8 MB).
- Verificación: **126 unit (1 live ignorado) + 10 integración**, clippy limpio,
  capturas de consulta, configuración e historial.

Fuera de alcance: regex/stemming, sincronización, export a PDF/DOCX, búsqueda
dentro de las notas y override manual del idioma de origen.

## M7 — IA configurable (terminado)

Cerrado el 2026-11-08:

- Nueve proveedores: OpenCode Go, OpenAI, Anthropic, Google Gemini, OpenRouter,
  Groq, Mistral, DeepSeek y Ollama local.
- Tres protocolos: OpenAI-compatible, Anthropic y Gemini (clave en header).
- API key en el llavero del sistema (Keychain/Secret Service/Credential Manager);
  `OPENCODE_API_KEY` mantiene prioridad.
- Selector, campo de clave, Guardar/Borrar con confirmación y estado en
  Configuración; el valor nunca se muestra ni se persiste en JSON/logs/URLs.
- ADR-0012, tests de protocolos y captura de Configuración sin llavero.
- Verificación: **126 unit (1 live ignorado) + 10 integración**; `dist`
  **7.250.944 B**.

## M8 — Editar EPUB (terminado)

Cerrado el 2026-11-09:

- Edición de título, autor e idioma desde el menú contextual de la tarjeta.
- Agregar o reemplazar portada (JPG/PNG/WebP, máximo 1600×2400).
- Reescritura del ZIP con `mimetype` primero, backup obligatorio y validación
  con `Book::open`; el original no se toca si algo falla.
- Actualización de biblioteca preservando progreso, ajustes, fechas y
  anotaciones.
- ADR-0013, tests con `mini.epub` y `edge.epub`, captura del editor.
- Verificación: **126 unit (125 + 1 live ignorado) + 10 integración**; `dist`
  **7.250.944 B**.

## M9 — Lectura de estudio (terminado)

Cerrado el 2026-11-09:

- Selección de texto a través de párrafos del mismo capítulo: arrastre fuera
  del bloque o **Mayúsculas + clic**.
- Resaltado completo del rango y vista previa con el color activo.
- `annotations.json` v2: `end_block` + `parts` por bloque, con migración
  transparente de v1.
- Reanclaje por offsets exactos y, si el texto cambió, por partes de cita;
  coincidencia parcial se informa como “no encontrada”.
- Panel con vista previa de una línea, cantidad de párrafos y salto al inicio.
- Consulta y nota sobre el texto completo; `rekey` conserva los rangos.
- ADR-0014, tests de migración/rangos/reanclaje y captura de selección.
- Verificación: **126 unit (125 + 1 live ignorado) + 10 integración**; `dist`
  **7.250.944 B**.

## M10 — Lectura de estudio II (terminado)

Cerrado el 2026-11-09:

- Vista previa flotante de enlaces internos y notas al pie, en el mismo capítulo
  y en otro documento del spine.
- “Ir al texto” confirma el salto; “← Volver al texto” restaura capítulo, bloque
  y fracción de scroll.
- Cierre por “Volver”, “×”, Escape o desplazamiento, sin navegar.
- Fixture `notes.epub`, tests de anclas/extracción y ADR-0015.
- Verificación: **126 unit (125 + 1 live ignorado) + 10 integración**; `dist`
  **7.250.976 B**.

## M11 — Actualizaciones (terminado)

Cerrado el 2026-11-09:

- Configuración → Avanzado: “Buscar actualizaciones” consulta la API de GitHub
  en un hilo, solo a pedido.
- Comparación de versiones, fecha y novedades del release; estado “al día”.
- Descarga del asset de la plataforma, verificación SHA-256 contra
  `SHA256SUMS.txt` y guardado en Descargas con permisos de ejecución.
- “Abrir release” y “Mostrar en Finder/Explorer”; la instalación sigue siendo
  manual y no se reemplaza el binario ni el `.app`.
- ADR-0016, tests de parseo/versiones/checksum y descarga contra un servidor
  local; captura con `--update-demo`.
- Verificación: **126 unit (125 + 1 live ignorado) + 10 integración**; `dist`
  **7.284.288 B**.

## M12 — Entrega automática (terminado)

Cerrado el 2026-11-09:

- `auto-release` se dispara cuando `build` termina bien en `main`; bump minor
  por `feat`, major por breaking, patch en el resto; ignora docs-only y commits
  de release.
- Actualiza `Cargo.toml`/`Cargo.lock`, commitea `chore(release): vX.Y.Z`, crea el
  tag y dispara `release.yml` con ese tag.
- `release.yml` acepta `workflow_dispatch` con tag y publica los binarios,
  `candil-macos-arm64-app.zip` y `SHA256SUMS.txt`.
- `Candil.app` se firma ad-hoc; cask `quicio/tap/candil` para instalar en macOS
  con Homebrew.
- ADR-0017 y `docs/RELEASING.md`; el flujo manual queda como fallback.
- Verificación: **126 unit (125 + 1 live ignorado) + 10 integración**; `dist`
  **7.284.288 B**.

## Consolidación UX/UI (2026-11-08)

Cerrada la consolidación `consolidar-ux-ui` (OpenSpec), sin cambiar el
comportamiento funcional de M1–M6:

- Sistema de diseño explícito en `DESIGN.md` y `src/ui.rs`: tokens de color,
  tipografía de UI, espaciado, radios, botones, paneles y swatches.
- Contraste AA en los cinco temas, foco visible, objetivos de 32×32 y estados
  hover/active/disabled consistentes.
- Biblioteca en dos niveles, tarjetas focusables, recuperación de EPUB
  roto/faltante y escaneo en segundo plano.
- Lector con barra de 40 px, paneles unificados y confirmaciones para acciones
  destructivas.
- Ayuda de atajos (`F1`/`?`), feedback de guardado/copiado y capturas públicas
  en Papel 19 px.
- Verificación: **126 unit (1 live ignorado) + 10 integración**, clippy limpio,
  `scripts/check.sh` verde y `dist` **7.250.944 B**.

## Orden de ataque recomendado

1. M2 cerrado (T1–T11): 50 EPUB en 1.65 s, 96 % de portadas, capturas y tests.
2. M3 cerrado: anclas, saltos, enlaces internos, progreso por fracción y
   preferencias por libro.
3. M4: destacados y notas (spec 45) — **hecho**.
4. M4: búsqueda dentro del libro (spec 40) — **hecho**.
5. M5: anotaciones globales, colores, export Markdown y navegación por
   porcentaje/capítulo — **hecho**.
6. M6: diccionario, traducción, IA e historial (spec 60) + centro de
   configuración (spec 65) — **hecho**.
7. Futuro: búsqueda con regex/stemming, sincronización y export a PDF/DOCX.

Regla: **una spec, un milestone, un PR lógico**. Si una tarea no encaja en la
spec, se discute antes de codear (flujo spec → código, nunca al revés).
