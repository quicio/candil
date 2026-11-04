---
id: adr-0002
status: aceptada
version: 1.0.0
updated: 2026-11-03
---

# ADR-0002 — Dos parsers de HTML: estricto y tolerante

## Contexto

Los EPUB deberían ser XHTML bien formado. En la práctica no lo son. Sobre 44
libros reales del corpus:

- Calibre escribe `&quot;` dentro de atributos del OPF (rompía el parseo: el
  reemplazo por `"` literal inválida el XML).
- Libros generados por OCR declaran `text/html` en el spine y traen atributos sin
  comillas, `:` sueltos y etiquetas sin cerrar: 51 de 92 capítulos salían vacíos
  con un parser XML.

Un solo parser no cubre ambos mundos: XML estricto rechaza lo que la gente
realmente tiene, y un parser tolerante pierde estructura cuando el documento sí
es válido.

## Decisión

Dos caminos, elegidos por resultado:

1. `sanitize()` normaliza siempre: quita DOCTYPE, convierte entidades nombradas a
   referencias numéricas (`&quot;` → `&#34;`, para no romper atributos) y
   reemplaza NBSP.
2. `roxmltree::Document::parse` — si acepta el documento, camino estricto (árbol
   real, namespaces, atributos).
3. Si falla, `parse_lenient()`: scanner propio de una pasada que reconoce bloques
   (`p`, `div`, `h1..h6`, `blockquote`, `ul/ol/li`, `pre`, `hr`, `br`, `img`),
   estilos inline (`b/strong`, `i/em/cite`, `small/sup/sub`, `a`) y saltea
   `head`/`script`/`style`.

## Consecuencias

- Todos los libros del corpus abren con texto (verificado con `--inspect`).
- El camino tolerante duplica el mapeo de etiquetas: **cada `Kind` nuevo debe
  agregarse en los dos** (está documentado en `ARCHITECTURE.md`).
- El tolerante no respeta anidamiento raro: aplana. Aceptado para lectura.

## Alternativas descartadas

- **Solo `html5ever`/`scraper`**: resuelve el caso roto, pero agrega ~1.5 MB de
  dependencias y un DOM completo; el 90% del corpus ya es XML válido.
- **Solo parser tolerante**: pierde namespaces y atributos con precisión; los
  índices por `id` (M3) necesitan el árbol.
- **Reparar el HTML con expresiones regulares**: frágil e impredecible, imposible
  de testear en serio.
