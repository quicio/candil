---
id: roadmap
status: vigente
version: 1.12.0
updated: 2026-11-09
---

# Roadmap

Dieciséis milestones. Cada uno tiene criterio de salida **verificable**, no una
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
| M13 | Biblioteca viva I | **Hecho** | [library-experience](../openspec/specs/library-experience/spec.md) | Retomar el último libro con un clic y refrescar la biblioteca al recuperar el foco |
| M14 | Respaldo y notas | **Hecho** | [data-backup](../openspec/specs/data-backup/spec.md), [reading-workspace](../openspec/specs/reading-workspace/spec.md) | Exportar/importar los datos de usuario con validación y filtrar el panel de anotaciones |
| M15 | Series y colecciones | **Hecho** | [library-experience](../openspec/specs/library-experience/spec.md) | Leer serie/posición del OPF, mostrarla en la tarjeta y ordenar por serie |
| M16 | Inicio y enlaces | **Hecho** | [library-experience](../openspec/specs/library-experience/spec.md), [reading-workspace](../openspec/specs/reading-workspace/spec.md) | Elegir inicio (último libro/biblioteca), volver con un icono y confirmar enlaces externos |
| M17 | Traducción sin fricción | **Hecho** | [reading-workspace](../openspec/specs/reading-workspace/spec.md), [ADR-0022](../docs/adr/0022-traduccion-local-y-limite-mymemory.md) | Traducir con MyMemory hasta 500 caracteres sin partir palabras u Ollama local sin activar online |
| M18 | Series en la estantería | **Hecho** | [library-experience](../openspec/specs/library-experience/spec.md) | Filtrar por serie o **Sin serie** y agrupar la grilla con encabezados en el orden por serie |
| M19 | Posibles duplicados | **Hecho** | [library-experience](../openspec/specs/library-experience/spec.md) | Detectar mismo título y autor, filtrar **Duplicados** e insignia en la tarjeta sin borrar archivos |
| M20 | Claves por proveedor | **Hecho** | [ai-credentials](../openspec/specs/ai-credentials/spec.md), [ADR-0023](../docs/adr/0023-claves-ia-por-proveedor.md) | Una cuenta de llavero por proveedor, migración de la clave histórica y `OPENCODE_API_KEY` solo para OpenCode Go |
| M21 | Respuestas completas | **Hecho** | [reading-workspace](../openspec/specs/reading-workspace/spec.md) | Default de 1536 tokens, migración del 512 viejo y aviso cuando el proveedor corta la respuesta |
| M22 | Contenido de IA variable | **Hecho** | [reading-workspace](../openspec/specs/reading-workspace/spec.md) | Aceptar `content` string/array/objeto y explicar límite de tokens, razonamiento o `finish_reason` |
| M23 | Marcadores | **Hecho** | [reading-workspace](../openspec/specs/reading-workspace/spec.md), [ADR-0024](../docs/adr/0024-marcadores-en-library.md) | Guardar posiciones con B, listarlas con Mayús+B, saltar y borrar desde el panel |
| M24 | Grilla virtualizada | **Hecho** | [library-experience](../openspec/specs/library-experience/spec.md) | Desde 240 libros visibles, construir y dibujar solo las filas cercanas al viewport, conservando series |
| M25 | Búsqueda full-text | **Hecho** | [library-experience](../openspec/specs/library-experience/spec.md) | Buscar dentro del texto de la biblioteca en un hilo, con progreso, cancelación y salto al pasaje |
| M26 | Markdown en IA y notas | **Hecho** | [reading-workspace](../openspec/specs/reading-workspace/spec.md), [ADR-0025](../docs/adr/0025-markdown-sin-dependencias.md) | Renderer propio para encabezados, listas, citas, código y énfasis en respuestas y notas |
| M27 | Etiquetas de marcadores | **Hecho** | [reading-workspace](../openspec/specs/reading-workspace/spec.md), [ADR-0024](../docs/adr/0024-marcadores-en-library.md) | Etiqueta opcional al crear y renombrar desde el panel; B sigue guardando sin etiqueta |
| M28 | Búsqueda visible y Drive | **Hecho** | [library-experience](../openspec/specs/library-experience/spec.md), [ADR-0026](../docs/adr/0026-google-drive-carpeta-local.md) | Toggle Contenido, panel de resultados amplio y carpeta local de Google Drive sin OAuth |

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
- Verificación: **134 unit (1 live ignorado) + 10 integración**, clippy limpio,
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
- Verificación: **134 unit (1 live ignorado) + 10 integración**; `dist`
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
- Verificación: **134 unit (133 + 1 live ignorado) + 10 integración**; `dist`
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
- Verificación: **134 unit (133 + 1 live ignorado) + 10 integración**; `dist`
  **7.250.944 B**.

## M10 — Lectura de estudio II (terminado)

Cerrado el 2026-11-09:

- Vista previa flotante de enlaces internos y notas al pie, en el mismo capítulo
  y en otro documento del spine.
- “Ir al texto” confirma el salto; “← Volver al texto” restaura capítulo, bloque
  y fracción de scroll.
- Cierre por “Volver”, “×”, Escape o desplazamiento, sin navegar.
- Fixture `notes.epub`, tests de anclas/extracción y ADR-0015.
- Verificación: **134 unit (133 + 1 live ignorado) + 10 integración**; `dist`
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
- Verificación: **134 unit (133 + 1 live ignorado) + 10 integración**; `dist`
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
- Verificación: **134 unit (133 + 1 live ignorado) + 10 integración**; `dist`
  **7.284.288 B**.

## M13 — Biblioteca viva I (terminado)

Cerrado el 2026-11-09:

- **Continuar leyendo** en la cabecera: abre el libro con mayor `last_opened`
  válido, con etiqueta compacta y tooltip.
- **Refresco al recuperar el foco**: escaneo incremental en un hilo, con debounce
  de 30 s y guardas de escaneo/carpetas/captura.
- Capturas de biblioteca con un libro abierto y ADR-0018.
- Verificación: **134 unit (133 + 1 live ignorado) + 10 integración**; `dist`
  **7.284.320 B**.



## M14 — Respaldo y notas (terminado)

Cerrado el 2026-11-09:

- **Exportar datos…**: ZIP con `config.json`, `library.json`,
  `annotations.json`, `consultas.json` y un manifiesto con checksums.
- **Importar datos…**: valida manifiesto, lista blanca, SHA-256 y JSON; guarda
  `pre-import-*.zip`, escribe atómico, recarga la app y rescanea.
- El respaldo no incluye EPUB, portadas ni claves.
- **Filtro en Destacados y notas** por cita, nota, libro o capítulo; la
  exportación Markdown respeta el filtro.
- ADR-0019, tests de respaldo/checksum/lista blanca y captura de Configuración.
- Verificación: **134 unit (133 + 1 live ignorado) + 10 integración**; `dist`
  **7.367.584 B**.

## M15 — Series y colecciones (terminado)

Cerrado el 2026-11-09:

- Serie y posición desde Calibre (`calibre:series` / `calibre:series_index`) y
  EPUB3 (`belongs-to-collection` + `collection-type=series` +
  `group-position`).
- `BookEntry.meta_version` fuerza una relectura única de metadatos conservando
  progreso, ajustes, portada y fechas.
- La tarjeta muestra `Serie · N` debajo del autor; nuevo orden **Serie**.
- Fixtures `mini.epub` (Calibre) y `notes.epub` (EPUB3), ADR-0020.
- Verificación: **134 unit (133 + 1 live ignorado) + 10 integración**; `dist`
  **7.367.664 B**.

## M16 — Inicio y enlaces (terminado)

Cerrado el 2026-11-09:

- Botón **←** discreto en la barra del lector para volver a la biblioteca.
- Configuración → Biblioteca → **Al iniciar**: “Último libro abierto” (default) o
  “Biblioteca”; si el archivo falta, abre la biblioteca.
- Enlaces externos `http`/`https`/`mailto`/`tel` con confirmación y apertura en
  la aplicación del sistema; `javascript:`, `data:` y `file:` se rechazan.
- ADR-0021, tests de `StartupMode` y del allowlist de enlaces, capturas de inicio
  y confirmación.
- Verificación: **134 unit (133 + 1 live ignorado) + 10 integración**; `dist`
  **7.367.696 B**.

## M17 — Traducción sin fricción (terminado)

Cerrado el 2026-11-09:

- MyMemory recibe como máximo 500 caracteres: `truncate_at_word` corta en el
  último espacio y nunca parte una palabra; la respuesta avisa cuando la
  traducción es parcial.
- Configuración → Consultas permite elegir **MyMemory (gratuita)** u
  **Ollama (local)**, con modelo configurable (`llama3.2` por defecto).
- La traducción local usa el endpoint OpenAI-compatible de Ollama
  (`localhost:11434/v1`), sin API key, con prompt de solo traducción y al menos
  1024 tokens de salida.
- `⇧K` y el botón Traducir usan el motor elegido. La traducción y la IA con
  Ollama funcionan con `online_enabled=false`; MyMemory/Wiktionary y los
  proveedores remotos siguen bloqueados.
- Errores accionables si Ollama no está abierto o falta el modelo
  (`ollama pull`).
- ADR-0022, tests de recorte/aviso/routing local y captura actualizada de
  Configuración → Consultas.
- Verificación: **139 unit (138 + 1 live ignorado) + 10 integración**; `dist`
  **7.384.240 B**.

## M18 — Series en la estantería (terminado)

Cerrado el 2026-11-09:

- El filtro de la biblioteca lista las series del OPF y la opción **Sin serie**,
  sin duplicados ni nombres vacíos.
- Filtrar por serie conserva búsqueda, orden y el resto de los filtros.
- Con orden **Serie**, cada grupo empieza con un encabezado y la cantidad de
  libros; los libros sin serie quedan al final bajo **Sin serie**.
- Banderas `--order series` y `--series "Nombre"` para capturas y verificación.
- Tests de nombres de serie, filtro y etiqueta truncada.
- Verificación: **142 unit (141 + 1 live ignorado) + 10 integración**; `dist`
  **7.400.800 B**.

## M19 — Posibles duplicados (terminado)

Cerrado el 2026-11-09:

- Clave conservadora por **título + autor normalizados** (minúsculas y acentos
  plegados); un título vacío nunca agrupa.
- El filtro **Duplicados** muestra solo entradas con al menos otra copia.
- Insignia **Posible duplicado ×N** en la portada y tooltip con el criterio.
- No se borra, mueve ni fusiona nada: quitar una entrada sigue siendo una acción
  explícita del menú contextual.
- Bandera `--filter duplicates` para capturas.
- Tests de normalización, conteo, filtro y título vacío.
- Verificación: **143 unit (142 + 1 live ignorado) + 10 integración**; `dist`
  **7.400.816 B**.

## M20 — Claves por proveedor (terminado)

Cerrado el 2026-11-10:

- El llavero usa una cuenta por proveedor: `ai-api-key/<id>`.
- Cambiar de proveedor recarga su clave, limpia el campo de entrada y cancela
  cualquier confirmación de borrado pendiente.
- `OPENCODE_API_KEY` solo aplica a OpenCode Go; ya no se envía a otros
  proveedores.
- La clave histórica `ai-api-key` se migra una sola vez a la cuenta del primer
  proveedor que la cargue, sin duplicarla.
- ADR-0023, tests del formato de cuenta y del alcance de la variable de entorno.
- Verificación: **144 unit (143 + 1 live ignorado) + 10 integración**; `dist`
  **7.400.848 B**.

## M21 — Respuestas completas (terminado)

Cerrado el 2026-11-10:

- **máx. tokens** por defecto pasa de 512 a **1536**; una configuración con el
  valor viejo exacto migra una vez y un valor personalizado se respeta.
- OpenAI/Ollama (`finish_reason=length`), Anthropic (`stop_reason=max_tokens`) y
  Gemini (`finishReason=MAX_TOKENS`) detectan el corte.
- La respuesta cortada agrega un **Aviso** con el límite actual y la acción de
  subirlo; no se presenta una respuesta incompleta como completa.
- Una selección de más de 2000 caracteres avisa que se enviarán los primeros
  2000.
- Tests de migración de tokens, detección por protocolo y sección de aviso.
- Verificación: **148 unit (147 + 1 live ignorado) + 10 integración**; `dist`
  **7.400.832 B**.

## M22 — Contenido de IA variable (terminado)

Cerrado el 2026-11-10:

- `parse_openai` acepta `content` como string, array de partes, objeto con `text`
  y el `choices[0].text` legado.
- Si no hay texto y `finish_reason` es `length`, el error pide subir
  **máx. tokens**.
- Si solo hay `reasoning_content`/`reasoning`/`thinking`, el error lo informa y no
  muestra el razonamiento como respuesta.
- El error genérico incluye `finish_reason` cuando existe.
- Tests de variantes, límite y razonamiento.
- Verificación: **149 unit (148 + 1 live ignorado) + 10 integración**; `dist`
  **7.400.848 B**.

## M23 — Marcadores (terminado)

Cerrado el 2026-11-10:

- `B` guarda la posición actual; no duplica un marcador a menos de 1 % en el
  mismo ancla.
- `Mayús+B` y el botón **Marcador** abren el panel; cada fila salta a su posición
  exacta y se puede eliminar.
- Los marcadores viven en `BookEntry.bookmarks` (`library.json`) y reutilizan
  `Progress`; el escaneo y `relink` los conservan.
- No se modifica el EPUB ni se agrega un archivo nuevo; el respaldo ya incluye
  `library.json`.
- Ayuda actualizada con `B` y `Mayús+B`; script de capturas con `--bookmarks`.
- Tests de alta, orden, borrado y comparación de posiciones.
- Verificación: **151 unit (150 + 1 live ignorado) + 10 integración**; `dist`
  **7.417.504 B**.

## M24 — Grilla virtualizada (terminado)

Cerrado el 2026-11-10:

- A partir de **240 libros visibles**, la grilla usa `ScrollArea::show_viewport` y
  solo llama a `cards()` para las filas que intersectan el viewport (más margen).
- El layout se arma como encabezados y filas; el orden **Serie** conserva sus
  encabezados y la cantidad de libros.
- Por debajo del umbral se mantiene el render completo, con todas las tarjetas
  focusables.
- Verificado con una biblioteca sintética de 300 libros en orden normal y Serie.
- Tests de filas, agrupación y alturas.
- Verificación: **152 unit (151 + 1 live ignorado) + 10 integración**; `dist`
  **7.417.536 B**.

## M25 — Búsqueda full-text (terminado)

Cerrado el 2026-11-10:

- **Enter** en el buscador de la biblioteca inicia una búsqueda dentro del texto;
  si no hay coincidencias de metadatos, el estado vacío también lo ofrece.
- La búsqueda corre en un hilo, un libro a la vez, y muestra `n de total` con
  opción de cancelar.
- Resultados con libro, capítulo y fragmento; al elegir uno se abre el libro y se
  resalta la coincidencia.
- Topes: 100 resultados, 20 por libro y 120 libros; se avisa si la lista es
  parcial.
- Sin índice persistente ni cambios en `library.json`.
- Verificación: **152 unit (151 + 1 live ignorado) + 10 integración**; `dist`
  **7.434.160 B** arm64; **8.433.248 B** Intel.

## M26 — Markdown en IA y notas (terminado)

Cerrado el 2026-11-11:

- `src/markdown.rs` parsea párrafos, encabezados, listas, citas, código,
  separadores y énfasis inline sin dependencias.
- Se aplica a respuestas de IA en el panel y el historial, a la vista previa de
  notas y al editor de notas.
- Diccionario y traducción siguen en texto plano.
- ADR-0025, tests de bloques, énfasis y código inline; capturas de historial y
  editor de nota.
- Verificación: **156 unit (155 + 1 live ignorado) + 10 integración**; `dist`
  **7.450.752 B** arm64; **8.445.568 B** Intel.

## M27 — Etiquetas de marcadores (terminado)

Cerrado el 2026-11-11:

- El panel **Marcadores** permite escribir una etiqueta y añadir; `B` sigue
  guardando sin etiqueta.
- Cada fila tiene **Editar** para renombrar, con Guardar/Cancelar en el mismo
  panel.
- Una etiqueta vacía vuelve al título del capítulo y porcentaje.
- La etiqueta persiste en `library.json` y se conserva al escanear/reasociar.
- Captura actualizada de marcadores con etiqueta; test de `set_bookmark_label`.
- Verificación: **156 unit (155 + 1 live ignorado) + 10 integración**; `dist`
  **7.450.784 B** arm64; **8.445.568 B** Intel.

## M28 — Búsqueda visible y Google Drive (terminado)

Cerrado el 2026-11-12:

- Toggle **Contenido** en la barra: con él activo, Enter busca dentro del texto;
  activarlo con una consulta válida inicia la búsqueda.
- El panel de resultados escala con la ventana (hasta 920×760) y el scroll interno
  acompaña.
- **Añadir Google Drive…** detecta la raíz local de Drive para escritorio
  (macOS `~/Library/CloudStorage/GoogleDrive-*/My Drive`, además de
  `~/Google Drive`) y abre el selector para elegir una **subcarpeta**; no indexa
  `My Drive` completo. Sin OAuth ni credenciales.
- Aviso si los EPUB están solo en la nube: Drive los descarga al abrirlos; se
  recomienda marcarlos como disponible sin conexión.
- Fallback explicado si no se detecta Drive; ADR-0026.
- Test de candidatos de Google Drive y captura actualizada del panel.
- Verificación: **158 unit (157 + 1 live ignorado) + 10 integración**; `dist`
  **7.450.800 B** arm64; **8.449.688 B** Intel.

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
- Verificación: **134 unit (1 live ignorado) + 10 integración**, clippy limpio,
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
