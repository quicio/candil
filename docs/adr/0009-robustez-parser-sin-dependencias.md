---
id: adr-0009
status: aceptada
version: 1.0.0
updated: 2026-11-05
---

# ADR-0009 — Robustez del parser sin dependencias nuevas

## Contexto

La auditoría de M1 encontró cuatro fallas de fidelidad del parser:

1. Se perdían espacios en los límites de runs inline (`Holamundo`,
   `Las flores del malCharles Baudelaire`).
2. La tabla de entidades nombradas era mínima: `&eacute;`, `&le;`, `&omega;` y
   otras quedaban literales; una entidad desconocida hacía fallar el XML y
   enviaba todo el documento al parser tolerante.
3. `<pre>` perdía saltos e indentación, y tablas/`<dl>` se aplanaban en un solo
   bloque con celdas pegadas.
4. Los 41 capítulos que solo tenían `<svg><image …/></svg>` salían vacíos porque
   el parser solo reconocía `<img>`.

Las alternativas “obvias” agregan dependencias: `html-escape` para entidades y
`resvg` para SVG. El proyecto tiene una regla explícita de no sumar peso sin ADR
y un binario objetivo de < 8 MB.

## Decisión

Resolver los cuatro problemas en el parser propio, sin dependencias nuevas:

1. **Whitespace con estado de bloque.** `normalize_ws` conserva un espacio en los
   bordes de cada nodo y `push_text` evita duplicarlo contra el run anterior. El
   resultado sigue la semántica HTML: un espacio en el fuente es un espacio; no
   se inventan separaciones donde no las hay.
2. **Entidades en código.** Una tabla `&[(&str, &str)]` con Latin-1, griego,
   matemáticas y puntuación comunes. `amp/lt/gt/quot/apos` se emiten como
   referencias numéricas para no romper atributos. Lo desconocido queda literal
   (visible y depurable) en vez de desaparecer.
3. **`<pre>`, tablas y definiciones.** Dentro de `Kind::Pre` no se normaliza el
   whitespace. `table/tr/td/th/caption` y `dl/dt/dd` se separan en bloques
   (`th`/`dt` en negrita, `caption` como `Caption`). No hay `Kind::Table`: se
   aplana, pero sin pegar texto.
4. **SVG raster.** Un `<svg>` cuyo primer `<image>` tenga `href` o `xlink:href`
   se convierte en `Kind::Image`; el recurso se resuelve dentro del zip como
   cualquier `<img>`. No se rasterizan vectores: un SVG sin `<image>` no produce
   bloque.

El guardado atómico de `config.json` no necesita ADR nuevo: ADR-0005 ya lo
promete y el código simplemente se alinea con esa decisión.

## Consecuencias

Buenas:

- El texto de libros reales deja de tener palabras pegadas, entidades visibles y
  capítulos SVG en blanco.
- Cero dependencias y cero MB nuevos.
- Ambos parsers (estricto y tolerante) comparten las mismas reglas para
  whitespace, entidades, `pre`, tablas y SVG raster.
- El fixture `edge.epub` fija los casos como regresión.

Malas:

- La tabla de entidades es finita: HTML5 tiene ~2.200 referencias. Se cubre el
  corpus y lo común; lo raro queda literal.
- No hay render de tablas ni de SVG vectorial. Las tablas se leen como bloques y
  un SVG de paths no se ve.
- La tabla de entidades hay que mantenerla a mano si aparece un corpus nuevo con
  símbolos no cubiertos.

## Alternativas descartadas

- **`html-escape`**: correcta y pequeña, pero es una dependencia nueva para algo
  que se resuelve con una tabla de ~150 entradas y tests; además no resuelve el
  problema del whitespace ni el de SVG.
- **Tabla HTML5 completa embebida**: ~2.200 entradas y decenas de KB de fuente
  para un lector de narrativa; sobra para el corpus actual.
- **`resvg` / rasterizar SVG**: agrega una dependencia grande para portadas y
  mapas que en la práctica envuelven un raster en `<image>`. Si algún día hacen
  falta vectores reales, va con ADR propio.
- **Motor CSS / webview**: contradice ADR-0001 y el modelo `Block`.
