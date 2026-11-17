---
id: handoff
status: vigente
version: 1.33.0
updated: 2026-11-16
---

# Handoff — estado del proyecto y cómo seguir

## Qué es

**Candil** es un lector de EPUB de escritorio en Rust: una columna, scroll
continuo, tipografía propia, temas prearmados. egui/eframe para la ventana,
parser XHTML propio para el contenido. Sin webview, sin motor CSS. El repo es
`quicio/candil` y el binario se llama `candil`.

Los datos viven en `~/Library/Application Support/candil/` (macOS),
`~/.config/candil/` (Linux) o `%APPDATA%\candil\` (Windows). Al arrancar por
primera vez, Candil migra automáticamente el directorio viejo `lector` si existe;
si el rename falla, sigue usando `lector` para no arrancar con la biblioteca
vacía.

**M1 (lector), M1.1 (endurecimiento), M2 (biblioteca), M3 (lectura fina),
M4 (extras), M5 (segunda etapa), M6 (consultas) y M7 (IA configurable) están
cerrados.** M3 incluye
anclas/saltos y progreso por fracción. M4 incluye destacados/notas y búsqueda
interna. M5 agrega anotaciones globales, colores, export Markdown y navegación.
M6 agrega diccionario Wiktionary, traducción MyMemory, IA multi-proveedor,
historial en `consultas.json` y el centro de configuración. La API key se guarda
en el llavero del sistema; `OPENCODE_API_KEY` sigue como override. M8 agrega
edición de metadatos y portada dentro del EPUB, con backup y validación. M9
agrega selección y anotación entre párrafos, con migración v2 de
`annotations.json` y reanclaje por partes. M10 agrega vista previa de enlaces
internos y notas al pie, con “Ir al texto” y un regreso al punto exacto de
lectura. M11 agrega actualización asistida desde Configuración: consulta manual
de GitHub, descarga verificada y ningún reemplazo automático. M12 automatiza la
entrega: bump/tag/release después de CI, `Candil.app.zip` y cask de Homebrew. M13
agrega “Continuar leyendo” y refresco incremental de la biblioteca al recuperar
el foco. M14 agrega respaldo/restauración local de datos y filtro en el panel de
notas. M15 agrega series/colecciones desde el OPF y orden por serie. M16 agrega
inicio configurable (último libro por defecto), regreso visible a la biblioteca
y confirmación de enlaces externos. M17 agrega el motor de traducción
configurable: MyMemory recorta a 500 caracteres en palabra y avisa cuando la
traducción es parcial; Ollama local traduce sin API key y sin activar consultas
online. M18 agrega filtro por serie o **Sin serie** y encabezados de grupo en el
orden por serie. M19 agrega detección conservadora de posibles duplicados por
título y autor, con filtro e insignia, sin borrar archivos. M20 separa las claves
de IA por proveedor en el llavero y limita `OPENCODE_API_KEY` a OpenCode Go. M21
sube el default de tokens de IA a 1536, migra el 512 viejo y avisa cuando el
proveedor corta una respuesta por límite. M22 hace tolerante el parser de IA a
las variantes de `content` y explica por qué una respuesta llegó sin texto. M23
agrega marcadores de posición guardados en `library.json`, con atajo `B`,
panel `Mayús+B`, salto y borrado. M24 virtualiza la grilla de biblioteca a partir
de 240 libros visibles: solo construye y dibuja las filas cercanas al viewport.
M25 agrega búsqueda full-text bajo demanda en la biblioteca, en un hilo, con
progreso, cancelación y salto al pasaje, sin persistir índices. M26 agrega un
renderer Markdown propio para respuestas de IA y notas. M27 agrega etiquetas
opcionales y editables a los marcadores. M28 agrega el toggle **Contenido**, un
panel de resultados más amplio y la carpeta local de Google Drive sin OAuth. M29
agrega el envío por copia a un Kobo montado o a cualquier carpeta/dispositivo. M30
cachea en memoria los índices full-text (64 MB) para acelerar búsquedas repetidas.
M31 agrega el **modo enfoque**: un velo vertical deja clara la línea central y
atenúa suavemente lo que se aleja de ella, configurable en Aa o en Lectura. M32
agrega la **instalación desde la UI en Linux**: re-verifica el SHA-256, exige un
ELF, guarda `candil.bak`, reemplaza con `rename` atómico y ofrece reiniciar;
fuera de una instalación de usuario sigue siendo asistida (ADR-0030). M33 agrega
**compartir citas como imagen**: Capturar guarda el destacado y abre una vista
previa con formatos 9:16, 4:5, 1:1, 3:4 y 16:9, fondos Papel, Noche, Portada,
Color o Imagen, y exporta un PNG local. Configuración → Compartir muestra una
cita de ejemplo en vivo (ADR-0031). M34 agrega el **modo estudio**: sesiones
múltiples por libro en `studies.json`, panel dividido lector/chat, bandeja de
contexto (selección, página, destacado, marcador), conversación multi-turno y
“Guardar nota” con `origin: ai` (ADR-0032). M35 separa la búsqueda general de
metadatos de una búsqueda avanzada en el contenido, con modos de frase/palabras,
inclusión/exclusión de libros y progreso/cancelación. M36 agrega una caché local
descartable de EPUB en montajes remotos (`~/.cache/candil/books`, 1 GB, LRU):
acelera apertura y búsqueda sin cambiar la ruta original, permite **Preparar
offline** los libros visibles y **Limpiar caché** desde Configuración (ADR-0033).
La búsqueda avanzada construye los índices de hasta 4 libros en paralelo y
conserva el orden, los topes y la cancelación. M37 convierte el proyecto en un
monorepo: `crates/candil-core` es el motor portable compartido por escritorio y
móvil, y `mobile/` contiene Flutter más el adaptador FFI (ADR-0034).

**Online está apagado por defecto**: se activa en Configuración o desde el panel
Consulta. La traducción y la IA con Ollama local son la excepción: corren sin
salir de la Mac. Cada proveedor guarda su propia API key en el llavero;
`OPENCODE_API_KEY` solo aplica a OpenCode Go. Si falta la clave y el proveedor la
requiere, el botón de IA queda deshabilitado. La prueba real con la clave del usuario queda pendiente;
con una clave inválida se verificó el 401 sin congelar la UI.

Fuera de alcance por ahora: regex/stemming, sincronización entre dispositivos,
export a PDF/DOCX, búsqueda dentro de las notas y override manual del idioma de
origen.

## OpenSpec

`openspec/` es la fuente de verdad para cambios nuevos. Las specs vigentes
(`design-system`, `library-experience`, `reading-workspace`,
`interaction-accessibility`, `ai-providers`, `ai-credentials`, `epub-editing`,
`app-updates`, `data-backup` y `study-workspace`) viven en `openspec/specs/`; los cambios
`consolidar-ux-ui`, `proveedores-ia-clave`, `editar-metadata-epub`,
`seleccion-entre-parrafos`, `notas-al-pie-y-volver`,
`actualizaciones-desde-la-app`, `biblioteca-continuar-leyendo`,
`respaldo-y-filtro-notas`, `series-desde-opf`, `atajo-traducir-seleccion`,
`inicio-regreso-enlaces`, `motor-traduccion-local`, `filtrar-y-agrupar-series`,
`detectar-duplicados`, `claves-ia-por-proveedor`, `respuestas-ia-completas` y
`contenido-ia-variantes`, `marcadores-lector`, `virtualizar-grilla`,
`busqueda-full-text-biblioteca`, `markdown-ia-notas`, `etiquetas-marcadores` y
`usabilidad-busqueda-drive`, `enviar-dispositivo` y `cache-busqueda-contenido`
quedaron archivados en
`openspec/changes/archive/`. `docs/specs/` queda como archivo histórico de M1–M6.
Las fases 1 (tokens, contraste AA, `src/ui.rs`, foco y swatches), 2
(biblioteca: cabecera, tarjetas focusables, estados de error/faltante y escaneo
en segundo plano), 3 (lector y paneles: barra, selección, anotaciones, búsqueda,
Ir a, índice, configuración y consulta), 4 (atajos, foco, objetivos, señales no
cromáticas y feedback) y 5 (capturas públicas en Papel 19 px, documentación y
validación) están implementadas. El cambio `consolidar-ux-ui` quedó archivado y
sus specs son la referencia vigente.

## Estado medido (no estimado)

Medición del 2026-11-06 sobre `~/Libros` (43 EPUB; el corpus de `~/Downloads` es
el mismo):

| métrica | valor | cómo se midió |
|---|---|---|
| corpus real | 43 EPUB, 0 errores de parseo | `--inspect` sobre `~/Libros/*.epub` |
| capítulos vacíos | 0 | ídem (M1.1 corrigió SVG y nav en spine) |
| imágenes resueltas | 2279/2279 | ídem, incluye `<svg><image>` raster |
| biblioteca (medición) | 43 libros, 41 portadas (95,3 %) | `--scan ~/Libros` (2026-11-06) |
| cierre M2 | 50 EPUB (43 reales + 7 copias): 1.65 s; 48/50 portadas | `--scan` en corpus temporal |
| capturas | 19 en `docs/img/` (biblioteca ×4, respaldo de datos, inicio, editor de EPUB, selección entre párrafos, enlace externo, tipografía, destacados, panel, nota, global, búsqueda, Ir a, consulta, configuración e historial), todas en Papel 19 px y con libros de dominio público | `scripts/captures.sh` (sandbox + Project Gutenberg) |
| escaneo | 1.57 s primera vez; 0.02 s con caché | `/usr/bin/time` + `--scan` |
| `covers/` | 4,2 MB (~98 KB/libro) | `du -sh` |
| tests | workspace (escritorio + núcleo + FFI) | `cargo test --workspace` |
| clippy | 0 warnings | `cargo clippy --workspace --all-targets -- -D warnings` |
| binario `dist` | 7.450.864 B arm64; 8.466.112 B Intel | `cargo build --profile dist` (2026-11-13) |
| `target/` completo | ~1,4 GB (dev 836 MB + fast 589 MB) | `du -sh target` (2026-11-10) |
| cambio de una línea | ~16 s | `cargo build --profile fast` (sin incremental) |
| fixtures | `mini.epub` 2.517 B; `edge.epub` 1.854 B; `notes.epub` 2.183 B | `scripts/make_fixture.py` |

La medición de biblioteca se hizo con 43 EPUB. Después `~/Libros` quedó en 40
(limpieza manual de duplicados); hoy la biblioteca tiene 40 entradas y 40
portadas. Los números de 43/41 siguen siendo válidos para el corpus de prueba.

El corpus cambió desde M1 (antes 44 libros/2168 imágenes): las métricas viejas
no se comparan directamente. Para reproducir, usar `--inspect` y anotar la fecha.
El 1.3 s de ADR-0006 era la referencia de M1; con el parser más grande, un cambio
en `xhtml.rs` cuesta ~6 s y en `app.rs`/`config.rs` ~2.7–3.5 s.

Advertencia de entorno: el disco sigue trabajando al límite (95% usado). Los
perfiles `dev` y `fast` tienen `incremental = false` y `debug = false` para no
acumular GB en `target/`; un cambio de una línea en `fast` tarda ~16 s. Si
aparece un error de linker raro (`errno=28`), revisar `df -h` antes que el
código.

## Mapa de módulos

```
src/lib.rs     API pública (los tests de integración dependen de ella)
src/main.rs    arranque, flags --inspect / --scan / --shot / --resume / --chapter / --panel
src/app.rs     ventana, biblioteca, lector, barra, scroll, paneles, teclado
crates/candil-core/src/epub.rs   zip → container.xml → OPF → spine + índice + portada
crates/candil-core/src/xhtml.rs  XHTML/HTML → Vec<Block> (entidades, pre/tablas, SVG raster)
crates/candil-core/src/search.rs índice efímero por libro y búsqueda full-text
mobile/        app Flutter + adaptador FFI en mobile/rust
src/library.rs índice de carpetas, escaneo incremental, BookEntry
src/annotations.rs destacados/notas: rangos multi-bloque, offsets + partes + origen, JSON atómico v3
src/share.rs   render local de citas como PNG (Literata + fondos + plantillas)
src/study.rs   sesiones de estudio por libro, mensajes, contexto e historial de IA
src/markdown.rs renderer Markdown mínimo para IA y notas, sin dependencias
src/ai.rs      catálogo de proveedores y protocolos (OpenAI, Anthropic, Gemini)
src/secrets.rs API key por proveedor en el llavero; `OPENCODE_API_KEY` solo para OpenCode Go
src/epub_edit.rs edición de metadatos/portada, backup y reemplazo validado
src/update.rs  releases, asset por plataforma, descarga, SHA-256 e instalación atómica (Linux)
src/backup.rs  respaldo ZIP de datos, manifiesto, checksums e importación segura
src/consulta.rs diccionario/traducción (MyMemory u Ollama local)/IA, HTTP ureq, parsers e historial
src/covers.rs  miniatura de portada (sin upscaling), hash FNV y caché
src/book_cache.rs  caché LRU de EPUB remotos (1 GB) e índice index.json
src/store.rs   persistencia JSON atómica y backup de archivos corruptos
src/theme.rs   presets de color
src/config.rs  settings globales en config.json; `Progress` con ratio/ancla y offset legado
src/fonts.rs   familias Literata en egui
```

Build en otros sistemas: [`BUILD.md`](BUILD.md) y la matriz de CI
`.github/workflows/build.yml` (macOS/Linux/Windows). En macOS,
`scripts/make_app.sh` arma `Candil.app` con el icono (`assets/candil.icns`).

## Releases

La entrega es automática (`docs/RELEASING.md`, ADR-0017): push a `main` →
`build` → `auto-release` decide minor/patch por conventional commits, actualiza
`Cargo.toml`/`Cargo.lock`, crea el tag y dispara `release.yml`. La release
publica Linux, Windows y macOS arm64/Intel, los `Candil.app` por arquitectura y
`SHA256SUMS.txt`. No se bumpea la versión ni
se crea el tag a mano. Si la publicación falla:
`gh workflow run release.yml -f tag=vX.Y.Z`.

En macOS el `.app` se firma ad-hoc y no está notarizado. El cask
`quicio/tap/candil` instala, quita la cuarentena y evita el bloqueo inicial; el
zip manual sigue necesitando clic derecho → Abrir la primera vez.

## Cómo verificar sin ojos humanos

La app sabe autofotografiarse: `--shot ruta.png` renderiza N frames, guarda el
PNG por `ViewportCommand::Screenshot` y cierra. No depende de permisos de
pantalla. Combinaciones útiles:

```bash
# biblioteca: índice + portadas sin abrir la UI
cargo run --profile fast -- --scan "$HOME/Libros"

# capturas del README en un sandbox aislado (no toca ~/Libros ni los datos reales)
scripts/captures.sh

# capturas: sin path = biblioteca; con path = lector
cargo run --profile fast -- --shot /tmp/biblioteca.png --screen library
cargo run --profile fast -- --shot /tmp/busqueda.png --screen library --search flores
cargo run --profile fast -- --shot /tmp/otra.png --library-path /tmp/library.json
cargo run --profile fast -- --shot /tmp/libro.png --chapter 13 libro.epub
cargo run --profile fast -- --shot /tmp/ancla.png --anchor 6 libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel toc libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel settings libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel notes libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel note libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel search --book-search frase libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel search --book-search frase --search-jump libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel notes --annotations-all libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel goto libro.epub
cargo run --profile fast -- --shot /tmp/p.png --export-md /tmp/destacados.md libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel share --selection libro.epub
cargo run --profile fast -- --shot /tmp/p.png --screen library --panel share-config
cargo run --profile fast -- --selection --share-save /tmp/cita.png libro.epub
cargo run --profile fast -- --shot /tmp/p.png --study-demo libro.epub
cargo run --profile fast -- --shot /tmp/p.png --selection --chat-demo libro.epub
cargo run --profile fast -- --shot /tmp/p.png --panel config libro.epub
cargo run --profile fast -- --shot /tmp/p.png --consulta "sleep" --consulta-kind diccionario --consulta-run libro.epub
cargo run --profile fast -- --shot /tmp/p.png --consulta "sleep" --consulta-kind historial libro.epub

# QA de la biblioteca: tamaño, foco, modales y escaneo
cargo run --profile fast -- --shot /tmp/p.png --screen library --size 720x600
cargo run --profile fast -- --shot /tmp/p.png --screen library --focus-card
cargo run --profile fast -- --shot /tmp/p.png --screen library --confirm-remove
cargo run --profile fast -- --shot /tmp/p.png --screen library --confirm-folder
cargo run --profile fast -- --shot /tmp/p.png --screen library --recover
cargo run --profile fast -- --shot /tmp/p.png --screen library --rescan
cargo run --profile fast -- --shot /tmp/p.png --selection libro.epub
cargo run --profile fast -- --shot /tmp/p.png --link-preview libro.epub
cargo run --profile fast -- --shot /tmp/p.png --screen library --update-demo
cargo run --profile fast -- --shot /tmp/p.png --confirm-delete libro.epub
cargo run --profile fast -- --shot /tmp/p.png --help libro.epub
cargo run --profile fast -- --shot /tmp/p.png --screen library --edit

# test live (no corre en el gate normal)
cargo test --lib red_real_wiktionary_y_mymemory -- --ignored

# diagnóstico de parser
cargo run --profile fast -- --inspect tests/fixtures/edge.epub

# abrir el último libro (el arranque normal es la biblioteca)
cargo run --profile fast -- --resume
```

Para probar sin tocar la configuración real, compilar y ejecutar el binario con un
`HOME` falso (no usar `cargo run` con `HOME` falso: rustup pierde su toolchain):

```bash
cargo build --profile fast
HOME=$(mktemp -d) ./target/fast/candil --shot /tmp/p.png libro.epub
```

`scripts/captures.sh` descarga cinco EPUB de Project Gutenberg en
`target/captures-sandbox/Libros` (P&P, Don Quijote, Dracula, Frankenstein y The
Yellow Wallpaper), arma un `HOME` falso, siembra destacados/historial de ejemplo
y regenera `docs/img/`. Nunca lee `~/Libros` ni `~/Library/Application Support`.
Para reutilizar un sandbox: `CANDIL_CAPTURE_SANDBOX=/ruta scripts/captures.sh`.
El script exporta `CANDIL_NO_KEYRING=1` porque el HOME falso no tiene llavero de login.

Para leer una captura: sí, hay que mirarla (herramienta de visión). No alcanza
con que el build pase: dos bugs reales (entidades `&quot;` en el OPF, aviso
flotante tapando texto) solo aparecieron así.

## Invariantes que no se rompen

1. **`Block` es el único contrato** entre parseo y render. Agregar un `Kind`
   nuevo es la forma correcta de agregar capacidad visual.
2. **El EPUB aporta estructura, el lector aporta apariencia.** No se interpreta
   CSS del libro.
3. **Un capítulo nunca puede quedar vacío** si el XHTML tiene texto: para eso
   existe el parser tolerante. El test `ningun_capitulo_queda_vacio` lo fija.
4. **Los recursos se resuelven dentro del zip** con rutas normalizadas
   (`resolve()`); aceptar rutas percent-encoded y `..`.
5. **La UI no bloquea**: abrir un libro lee el EPUB completo en un hilo y lo
   deja en memoria; cambiar de capítulo o cargar una imagen ya no toca el disco.
   El escaneo usa `Book::open_streaming` y solo lee metadatos/portada. `--shot`
   sigue abriendo en forma síncrona para que el primer frame tenga el libro.
6. **El texto no se inventa ni se pega**: el whitespace de frontera entre runs se
   conserva, `<pre>` mantiene saltos/indentación y las entidades conocidas se
   decodifican. Un capítulo con `<svg><image>` raster no queda vacío.
7. **`config.json` se reemplaza atómicamente**: nunca se escribe el archivo final
   a medias; si está corrupto, se guarda una copia y se arranca con defaults.
   Lo mismo vale para `library.json` (`src/store.rs`).
8. **La biblioteca indexa, no copia**: las entradas usan la ruta canónica; si un
   archivo desaparece se marca `missing` y no se borra la entrada.
9. **El progreso de libros indexados vive en `library.json`**; `config.progress`
   solo se conserva para leer archivos viejos y no recibe escrituras nuevas.
10. **Las anclas del EPUB se preservan** en `Block.anchor` y `TocEntry.anchor`;
    nunca se descarta el `#fragmento` al parsear nav/ncx.
11. **El progreso se guarda como `{chapter, ratio, anchor, anchor_ratio}`**, no
    como píxeles. El ancla es el índice de bloque visible superior y manda al
    restaurar; `anchor_ratio` interpola dentro del bloque. Cualquier posición que
    se persista debe venir de coordenadas de contenido, no de pantalla.
12. **Los ajustes por libro no contaminan el global**: `entry.settings` gana si
    existe; desmarcar "Solo para este libro" borra el override. El tema se
    resuelve por frame con `effective_settings`.
13. **Los destacados son offsets de caracteres + cita**, no píxeles ni índices
    de byte. Si el bloque cambió, se reanclan buscando la cita en el capítulo; si
    no aparece, no se dibujan. Borrar una anotación nunca toca el EPUB.
14. **El índice de búsqueda es efímero**: vive en memoria mientras el libro está
    abierto y se libera al volver a la biblioteca. No se persiste en
    `library.json`. La normalización es 1:1 para que los offsets coincidan.
15. **El color del destacado es `u8` en `annotations.json`** (0 acento, 1 ámbar,
    2 verde), no una propiedad del tema ni de `library.json`. El modo global del
    panel no cambia el archivo; solo cambia la vista y el alcance del export.
16. **Online apagado por defecto**: `Config.consulta.online_enabled = false`.
    Ninguna consulta remota sale sin click explícito; apagarlo corta toda salida
    externa. La traducción y la IA con Ollama local no salen de la máquina y
    siguen disponibles (ADR-0022).
17. **Cada proveedor tiene su propia clave en el llavero** (cuenta
    `ai-api-key/<id>`); la histórica compartida se migra una sola vez.
    `OPENCODE_API_KEY` solo aplica a OpenCode Go. Nunca se escribe una clave en
    `config.json`, `consultas.json`, logs ni URLs; el estado se muestra como
    llavero/entorno/no configurada, nunca el valor.
18. **El historial es local** (`consultas.json`), con límite y borrado; no guarda
    el párrafo de contexto, claves ni se envía a proveedores. Las peticiones
    corren en un hilo y la UI no se bloquea.
19. **Todo control nuevo es focusable**: usar `Button` o `ui.interact` con un
    id persistente, y Enter/Espacio para activarlo. No dejar `Label` clicables
    sin foco de teclado.
20. **Sin movimiento de layout**: `animation_time = 0`; egui no expone la
    preferencia de reduced motion del SO, así que no se anima nada decorativo.
21. **Editar un EPUB no es leerlo**: reescribir el ZIP exige backup, validación
    y reemplazo seguro. Nunca se modifica el original sin esas tres cosas.
22. **Las anotaciones pueden cruzar párrafos**: `annotations.json` v3 guarda
    `end_block`, una parte de cita por bloque y `origin` (`user`/`ai`);
    `block`/`start`/`end` siguen siendo el ancla y siguen siendo válidos para
    archivos v1/v2. No se convierten a píxeles ni se separan en varias
    anotaciones.
23. **Un enlace interno no navega solo**: abre una vista previa; “Ir al texto”
    es la acción explícita y guarda un único `Progress` de regreso. Los enlaces
    externos no se abren sin una decisión aparte.
24. **La actualización es explícita y segura**: consulta GitHub solo a pedido y
    verifica SHA-256. En Linux, si el binario vive bajo `$HOME` y no es un build
    de `target/`, la UI puede instalarlo con backup + `rename` atómico y ofrecer
    reiniciar (ADR-0030); en macOS, Windows y paquetes del sistema solo se
    descarga el archivo y nunca se ejecuta ni se reemplaza nada (ADR-0016).
25. **La entrega es automática**: no bumpees `Cargo.toml` ni crees tags a mano;
    push a `main` y `auto-release` publica. Si falla, relanza `release.yml` con
    el tag existente (ADR-0017).
26. **La biblioteca se refresca al recuperar el foco**, con debounce de 30 s y
    sin bloquear la UI; no es un watcher en tiempo real (ADR-0018). “Continuar
    leyendo” usa el mayor `last_opened` válido, no el orden de la grilla.
27. **El respaldo de datos incluye solo los cinco JSON de usuario**; nunca EPUB,
    portadas ni claves. La importación valida manifiesto, checksums y JSON, y
    guarda un `pre-import-*.zip` antes de escribir (ADR-0019).
28. **La serie sale del OPF**, no del nombre del archivo. `BookEntry.meta_version`
    fuerza una relectura única cuando cambia la extracción; el escaneo conserva
    progreso, ajustes, portada y fechas (ADR-0020).
29. **Al iniciar manda `StartupMode`**: último libro por defecto; biblioteca como
    opción; si el archivo falta, biblioteca. `--resume`/`--screen library` siguen
    teniendo prioridad en desarrollo (ADR-0021).
30. **Los enlaces externos se abren solo con confirmación** y solo si el esquema
    es `http`, `https`, `mailto` o `tel`; `javascript:`, `data:` y `file:` se
    rechazan (ADR-0021).
31. **La traducción remota se recorta en palabra a 500 caracteres** (MyMemory) y
    avisa si es parcial; no se hace chunking. La traducción con Ollama local no
    exige `online_enabled` y nunca lleva API key (ADR-0022).
32. **La serie se puede filtrar y agrupar**: el filtro por serie o **Sin serie**
    es estado de UI y no se persiste; el nombre sale de `library.json` y el orden
    Serie dibuja encabezados por grupo. No se edita la serie desde la biblioteca.
33. **Los posibles duplicados usan título y autor normalizados**, no hashes ni
    contenido; la UI dice “posible” y nunca borra ni mueve un archivo sin una
    acción explícita. La detección es estado de UI y no se persiste.
34. **Las claves de IA son por proveedor** y `OPENCODE_API_KEY` solo aplica a
    OpenCode Go. Cambiar de proveedor recarga su clave; la clave histórica
    compartida se migra una sola vez (ADR-0023).
35. **Una respuesta de IA no se corta en silencio**: el default es 1536 tokens,
    el 512 viejo migra una vez y un `finish_reason`/`stop_reason` de límite agrega
    un **Aviso** con la acción para subir «máx. tokens».
36. **El parser de IA acepta `content` string, array de partes u objeto**, además
    del `choices[0].text` legado. Si no hay texto, el error dice si fue límite de
    tokens, razonamiento interno u otro `finish_reason`; nunca se inventa una
    respuesta ni se muestra el razonamiento como texto final.
37. **Los marcadores son posiciones, no anotaciones**: viven en
    `BookEntry.bookmarks` dentro de `library.json`, reutilizan `Progress` y se
    conservan al escanear o reasociar. `B` guarda, `Mayús+B` abre el panel; el
    EPUB nunca se modifica (ADR-0024).
38. **La grilla se virtualiza a partir de 240 libros visibles**: solo se
    construyen las filas del viewport (más margen) y se conservan encabezados de
    serie. Por debajo del umbral se renderiza todo y el foco de teclado sigue
    siendo completo; en bibliotecas grandes, el foco llega a las filas
    renderizadas.
39. **La búsqueda full-text es bajo demanda y efímera**: recorre los EPUB en un
    hilo, un índice por libro, con topes de 100 resultados, 20 por libro y 120
    libros. No se persiste texto ni índice; el resultado salta al pasaje con el
    mismo resaltado de la búsqueda interna.
40. **El Markdown es propio y acotado**: `src/markdown.rs` cubre encabezados,
    listas, citas, código, separadores y énfasis inline, sin HTML ni
    dependencias. Se aplica solo a IA y notas; diccionario/traducción siguen en
    texto plano (ADR-0025).
41. **La etiqueta del marcador es opcional y editable**: se guarda en
    `Bookmark.label`; vacía cae al título del capítulo. `B` guarda sin etiqueta y
    el panel permite crearla o renombrarla sin cambiar el esquema (ADR-0024).
42. **Google Drive es una carpeta local**, no una cuenta: **Añadir Google Drive…**
    abre el selector en la raíz de Drive para elegir una subcarpeta y se indexa
    como cualquier otra, sin OAuth, API de Google ni credenciales (ADR-0026).
    Elegir la raíz de Drive se rechaza: indexar todo Drive es demasiado lento. Si
    los archivos están solo en la nube, Drive los descarga al abrirlos; se avisa y
    se recomienda disponible sin conexión. La búsqueda full-text exige el toggle
    **Contenido** y su panel escala con la ventana.
43. **Enviar a dispositivo es una copia, no un movimiento**: se detecta Kobo por
    `.kobo`, se prefiere `Books/`, nunca se sobrescribe y no se borra nada. Kindle
    no lee EPUB por USB; no se convierte a AZW3/MOBI (ADR-0027).
44. **La búsqueda full-text cachea índices en memoria, no en disco**: LRU de
    64 MB, clave ruta+tamaño+mtime, `Arc<SearchIndex>` compartido. La primera
    búsqueda de la sesión parsea; las siguientes reutilizan. Un EPUB editado
    invalida su clave y el caché se pierde al cerrar (ADR-0028).
45. **El modo enfoque es un velo del tema, no un cambio de layout**: se guarda
    como `Settings.focus_mode` (global o por libro), la banda clara queda fija
    en el centro del viewport y el texto se atenúa hacia el color de fondo. No
    mueve el contenido, no altera el progreso ni la selección. En el inicio y el
    final del capítulo el borde sin más texto no se vela, para que las primeras
    y últimas líneas se lean enfocadas.
46. **Compartir una cita es local**: Capturar crea el destacado si falta y
    renderiza un PNG a 1080 px de ancho con `ab_glyph` + Literata. La imagen
    nunca se sube a una red social ni a un servidor; se guarda en Imágenes con
    temporal + `rename`. `Config.share` guarda plantilla y toggles (ADR-0031).
47. **El estudio es un cuaderno, no una consulta**: vive en `studies.json` con
    varias sesiones por libro; cada mensaje conserva su contexto visible. Las
    respuestas de IA viven en la sesión y solo pasan a `annotations.json` si el
    usuario pulsa “Guardar nota”, con `origin: ai`. No hay envío automático:
    online apagado bloquea el chat remoto y la bandeja muestra qué se manda
    (ADR-0032).
48. **Escribir no dispara atajos**: si un campo de texto tiene foco, las letras
    son texto y no abren paneles ni consultas; solo Escape sigue cerrando. El
    botón “Chat” de la selección guarda el destacado, abre el estudio con ese
    pasaje en la bandeja y enfoca el compositor sin enviar nada.
49. **La caché de EPUB remotos es descartable y no cambia la identidad**: vive en
    `~/.cache/candil/books` (1 GB, LRU), solo copia montajes remotos y se abre
    con `Book::open_from` conservando la ruta original. **Preparar offline**
    copia los visibles en un hilo; **Limpiar caché** está en Configuración. No
    entra en el respaldo ni en `library.json` (ADR-0033).

## Definición de terminado (por tarea)

- [ ] `cargo test --workspace` verde y sin warnings nuevos.
- [ ] `cargo fmt --check` limpio.
- [ ] `cargo clippy --workspace --all-targets -- -D warnings` limpio.
- [ ] Si tocó UI: captura con `--shot` revisada.
- [ ] Si tocó parseo: `--inspect` sobre el fixture `edge.epub` y al menos 5 EPUB
      del corpus sin capítulos vacíos nuevos.
- [ ] Si tocó biblioteca: `--scan` sobre un fixture o `~/Libros` y revisar
      `library.json`/`covers/`.
- [ ] Spec actualizada (estado, versión, decisiones tomadas).
- [ ] Commit atómico con mensaje en imperativo.

## Qué NO hacer

- No migrar a egui 0.37 "para estar al día" (ADR-0003).
- No introducir SQLite para la biblioteca sin revisar ADR-0007 (el corpus es de
  cientos, no de millones).
- No copiar ni mover los libros al directorio de datos: la biblioteca indexa
  rutas (ADR-0007). La única copia permitida es la caché descartable de montajes
  remotos (ADR-0033).
- No agregar un watcher del sistema; la biblioteca se refresca al recuperar el
  foco o a mano (ADR-0018).
- No implementar paginado por columnas: la decisión de M1 fue scroll continuo.
- No rasterizar SVG vectorial ni agregar `resvg` sin ADR (solo se soporta
  `<svg><image>` raster, ver ADR-0009).
- No volver a `fs::write` para `config.json` ni `library.json`: la garantía es
  atómica (`src/store.rs`).
- No tocar la tipografía ni los temas sin mirar una captura.
- No agregar dependencias por comodidad (cada MB del binario se nota).
- No agregar cliente HTTP, diccionario online o IA sin ADR y sin medir `dist`
  (spec 60, ADR-0011 pendiente).
- No enviar texto del libro a un proveedor externo sin una acción explícita del
  usuario, y nunca guardar claves de API en `config.json` ni en logs.
