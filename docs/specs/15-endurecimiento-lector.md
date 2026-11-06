---
id: spec-15
title: Endurecimiento del lector (M1.1)
status: implementada
version: 1.1.0
updated: 2026-11-05
milestone: M1.1
depends_on: [spec-10, adr-0002, adr-0005, adr-0009]
---

# Spec 15 — Endurecimiento del lector (M1.1)

> Aprobada por Hugo e implementada el 2026-11-05. No agrega funcionalidad de
> producto: cierra deuda de correctitud de M1 antes de empezar la biblioteca
> (M2).

## Contexto

La auditoría del 2026-11-05 midió, contra 43 EPUB reales:

- 0 fallos de apertura y 2239/2239 imágenes resueltas.
- **41 capítulos vacíos** en 22 libros, todos con `<svg><image …/></svg>` (mapas,
  portadillas). El lector solo entiende `<img>`.
- Palabras pegadas en límites de runs inline (`Holamundo`,
  `Las flores del malCharles Baudelaire`).
- Entidades HTML fuera de la tabla mínima (`&le;`, `&omega;`, `&kappa;`) que
  quedan literales y, peor, hacen caer todo el documento al parser tolerante.
- `<pre>` sin saltos ni indentación, y tablas/`<dl>` aplanados en un solo bloque.
- `config.json` se escribe con `fs::write`, no de forma atómica como promete
  ADR-0005.

## Objetivo

Que el texto que el lector muestra sea fiel al XHTML en los casos que ya dice
soportar, y que un corte al guardar no destruya ajustes ni progreso.

## Alcance

Entra:

1. Espacios correctos en límites de runs inline.
2. Entidades HTML Latin-1, griego y matemáticas comunes.
3. `<pre>` con whitespace preservado.
4. Tablas, `<dl>`, `<dt>`, `<dd>` y `<caption>` sin pegar celdas.
5. `<svg><image href="…"/></svg>` como `Kind::Image` (solo raster embebido).
6. `config.json` con escritura atómica y reporte de configuración corrupta.
7. Clamps y rangos de tipografía consistentes.
8. Fixture de bordes + tests de integración + actualización de métricas.

**No entra:** rutas canónicas, `schema_version`, `Progress` por fracción,
biblioteca M2, virtualización de imágenes, selección de texto, búsqueda,
renderizado de SVG vectorial, CSS del libro.

## Decisiones

- **ADR-0009:** las entidades, el whitespace, `<pre>`, tablas y SVG raster se
  resuelven en el parser propio, sin dependencias nuevas.
- **ADR-0005:** se implementa tal como está escrito; no se crea otro ADR.

## Tareas

### T1 — Espacios en límites inline

**Archivos:** `src/xhtml.rs`.

**Pasos**

1. Tests primero:
   - `<b>Hola</b> <i>mundo</i>` → `Hola mundo`.
   - `Hola<b> mundo</b>` → `Hola mundo`.
   - `a <em> b </em> c` → `a b c` (un solo espacio, sin duplicar).
   - `uno<em>dos</em>tres` → `unodostres` (sin espacio inventado).
2. Cambiar `normalize_ws`: conservar un espacio en los bordes de cada nodo de
   texto y colapsar solo los interiores.
3. En `push_text`, quitar el espacio inicial si el run anterior ya termina en
   espacio, aunque cambie el estilo. No tocar `Kind::Pre`.

**Criterio:** los cuatro tests pasan y `parses_basic_blocks`,
`collapses_source_whitespace` y `br_is_hard_break` siguen verdes.

### T2 — Entidades y saneamiento

**Archivos:** `src/xhtml.rs`.

**Pasos**

1. Test: `caf&eacute; ni&ntilde;o &le; &omega;` → `café niño ≤ ω`.
2. Ampliar la tabla de entidades nombradas: Latin-1 (incluye `ntilde` y
   mayúsculas), griego, matemáticas comunes y puntuación. Mantener `amp`, `lt`,
   `gt`, `quot`, `apos` como referencias numéricas para no romper atributos.
3. Eliminar `U+00AD` (soft hyphen) y el BOM inicial en `sanitize`.
4. Test: `a&shy;b` y `a\u{ad}b` → `ab`.

**Criterio:** entidades del corpus (`le`, `omega`, `kappa`, `pi`, `Phi`,
`epsilon`) dejan de aparecer literales.

### T3 — `<pre>`, tablas y listas de definición

**Archivos:** `src/xhtml.rs`.

**Pasos**

1. Test: `<pre>def f(x):\n    return x</pre>` conserva el salto y los cuatro
   espacios.
2. Test: una tabla produce bloques separados para `celda uno` y `celda dos`; un
   `<dl>` produce `término` y `definición`.
3. En el parser estricto, agregar brazos para `table/thead/tbody/tfoot/tr`,
   `td/th`, `caption`, `dl/dt/dd`. Cada celda/entrada hace `flush` y genera su
   propio bloque (`th`/`dt` en negrita; `caption` como `Caption`).
4. En el tolerante, agregar esos nombres a `blockish` y manejar `th`/`dt` y
   `caption` para que el comportamiento sea el mismo.

**Criterio:** no queda texto de celdas/entradas pegado; `<pre>` conserva
indentación interna. No se promete render de tabla: se aplana en bloques.

### T4 — Imágenes SVG raster

**Archivos:** `src/xhtml.rs`.

**Pasos**

1. Test estricto con `<svg xmlns="http://www.w3.org/2000/svg"><image
   xlink:href="img/a.png"/></svg>` → `Kind::Image` con `src == "img/a.png"`.
2. Test tolerante con `<svg><image xlink:href="img/b.png"></svg>`.
3. Permitir el namespace SVG en `visit_node`, y en el brazo `svg` tomar el primer
   `<image>` con `href` o `xlink:href` (nombre local del atributo).
4. En el tolerante, tratar `image` como `img` y buscar `xlink:href`/`href`.
5. No rasterizar vectores ni agregar `resvg`: si el SVG no tiene `<image>`
   raster, no genera bloque.
6. Descartar del spine los ítems con `properties="nav"` cuando haya otros
   capítulos: algunos EPUB meten el índice y se ve como una página en blanco. Si
   el `nav` es lo único que hay, conservarlo.

**Criterio:** los 41 capítulos SVG-only del corpus dejan de salir vacíos y sus
imágenes resuelven dentro del zip; el `nav` del spine no aparece como capítulo
(el fixture de bordes lo incluye a propósito y el test exige 1 capítulo).

### T5 — Configuración atómica

**Archivos:** `src/config.rs`, `src/app.rs`.

**Pasos**

1. Separar `load_from(path)` / `load_with_report_from(path)` y
   `save_to(path)` / `save()`; `save()` devuelve `Result<(), String>`.
2. `save_to`: escribir `config.json.tmp` en el mismo directorio,
   `write_all` + `sync_all` + `rename`. En error, borrar el temporal.
3. `load_with_report`: si el JSON es inválido, renombrar a
   `config.json.corrupt-<epoch>` y arrancar con `Default`; devolver un mensaje
   para el toast.
4. Tests: roundtrip, JSON viejo sin campos nuevos, JSON corrupto con backup,
   ausencia de `.tmp` después de guardar.
5. `Lector::new` muestra el aviso de configuración corrupta en un toast corto al
   pie (no tapa el título); `persist` muestra error de guardado y reintenta en el
   próximo debounce.

**Criterio:** nunca se pierde el archivo anterior por un guardado fallido; un
JSON corrupto no bloquea el arranque y deja copia.

### T6 — Clamps y rangos

**Archivos:** `src/config.rs`, `src/app.rs`.

**Pasos**

1. Constantes únicas (`FONT_MIN/MAX`, `LINE_MIN/MAX`, `COLUMN_MIN/MAX`,
   `PARAGRAPH_MIN/MAX`) usadas por `Settings::clamp` y por los sliders.
2. Clamp inmediato después de `+`/`-`.
3. `goto_chapter` usa `n.saturating_sub(1)` para no depender de que `n > 0`.

**Criterio:** no se puede renderizar un tamaño fuera de rango; los sliders y el
clamp no se contradicen.

### T7 — Fixture, integración y métricas

**Archivos:** `scripts/make_fixture.py`, `tests/fixtures/edge.epub`,
`tests/parser_edges.rs`, docs.

**Pasos**

1. Generar un segundo fixture `edge.epub` con espacios inline, entidades,
   `<pre>`, tabla, `<dl>`, SVG raster y el `nav` dentro del spine. Fijar
   `date_time` de las entradas del zip para que la regeneración sea determinística.
2. Test de integración que abra `edge.epub` y verifique texto, `Pre`, celdas,
   definiciones y `src` resuelto del SVG.
3. Repetir la medición del corpus (43 EPUB): fallos, capítulos vacíos, imágenes
   resueltas, entidades literales.
4. Capturas: portada de `flores-del-mal` sin palabras pegadas y un mapa SVG
   visible (por ejemplo `Elantris`, capítulo 4).
5. Actualizar `HANDOFF.md`, `README.md`, `ROADMAP.md`, `AGENTS.md`,
   `ARCHITECTURE.md`, `CONVENTIONS.md` y `docs/README.md`.

**Criterio:** los números de la documentación se pueden reproducir con
`--inspect` y las capturas muestran los casos corregidos.

## Verificación

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run --profile fast -- --inspect tests/fixtures/edge.epub
cargo run --profile fast -- --inspect <corpus>/libro.epub
cargo run --profile fast -- --shot /tmp/mapa.png --chapter 4 <libro>.epub
```

## Criterio de salida

1. `cargo test` verde, con tests nuevos para T1–T7.
2. `cargo fmt --check` limpio y clippy sin warnings.
3. Corpus: 0 fallos de apertura; 0 capítulos vacíos salvo XHTML realmente vacío;
   100% de las imágenes (incluidas las de SVG) resolubles.
4. Capturas revisadas: título sin palabras pegadas y mapa/portada SVG visible.
5. Docs con métricas fechadas y reproducibles.

## Resultados medidos (2026-11-05)

| métrica | antes | después |
|---|---|---|
| EPUB con error de apertura | 0/43 | **0/43** |
| capítulos vacíos | 41 en 22 libros | **0** |
| imágenes resueltas | 2239/2239 | **2279/2279** (incluye SVG) |
| entidades nombradas literales | `le`, `omega`, `kappa`, `pi`, `Phi`, `epsilon` | **0** (quedan 2 `&#…;` doble-codificados de OCR) |
| soft hyphens en texto | 48.008 en un libro | **0** |
| tests | 14 + 6 | **25 unit + 7 integración** |
| clippy | 2 warnings | **0** |
| binario `dist` | 5.298.032 B | **5.347.888 B** |

Capturas revisadas: `flores-del-mal` (título sin palabras pegadas),
`Elantris` capítulo 4 (portada SVG) y `El camino de los reyes` capítulo 4 (mapa
SVG). El escaneo de palabras de ≥28 caracteres bajó de 134 a 97; las que quedan
son compuestos con guion y secuencias de ADN, no palabras pegadas.

## Riesgos

| riesgo | mitigación |
|---|---|
| La tabla de entidades no cubre HTML5 completo | cubrir corpus + Latin-1/griego/matemáticas; lo desconocido queda literal y visible |
| SVG vectorial (paths) no se renderiza | documentado; solo `<image>` raster |
| Tablas complejas se ven como bloques apilados | aceptado en M1.1; tabla real queda para M3/M4 |
| `rename` no es atómico en Windows si el destino existe | el objetivo es macOS/Linux; documentar si se porta |
| `U+00AD` podría ser significativo en otro idioma | se elimina porque el lector no hace hyphenation |
| Descartar el `nav` del spine podría ocultar contenido | solo se descarta si hay otros capítulos y el ítem declara `properties="nav"` |
