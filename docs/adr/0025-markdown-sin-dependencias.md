# ADR-0025: Markdown mínimo sin dependencias

- **Estado:** aceptado
- **Fecha:** 2026-11-11
- **Área:** UI, consultas, notas

## Contexto

Las respuestas de IA llegan en Markdown: encabezados, negritas, listas, citas y
bloques de código. La UI lo mostraba como texto plano, con `**`, `-` y ``` a la
vista. Las notas tienen el mismo problema en el panel de anotaciones y en el
editor.

Traer un motor Markdown completo agrega dependencias, peso y una superficie que
no se necesita: los modelos producen un subconjunto acotado y la app no interpreta
HTML ni CSS.

## Decisión

Implementar `src/markdown.rs`, un renderer propio sobre egui:

- Bloques: párrafos, encabezados `#`–`######`, listas `-`/`*`/`+`, listas
  numeradas, citas `>`, bloques ``` ``` y separadores `---`/`***`/`___`.
- Inline: `**negrita**`, `*itálica*` y `` `código` ``; el código usa la familia
  monoespaciada y el acento del tema.
- Se renderiza con `LayoutJob`/`Label`, respetando tipografía, colores, wrapping y
  temas. No hay HTML, tablas, imágenes ni enlaces clicables.
- Se aplica a respuestas de IA (panel e historial), a la vista previa inline de
  notas y a la vista previa completa del editor de notas.
- Diccionario y traducción siguen con texto plano: no son Markdown y podrían
  contener asteriscos legítimos.

**Por qué propio:** cero dependencias, cero HTML y control total del estilo
Literata. **Alternativa descartada:** `pulldown-cmark` + renderer, que agrega
peso y obliga a mapear eventos HTML a egui. **Alternativa descartada:** mostrar
Markdown crudo, que es lo que el usuario reportó como ilegible.

## Consecuencias

- Las respuestas de IA y las notas se leen con jerarquía y énfasis.
- El subconjunto es deliberadamente incompleto; si aparece una tabla o un enlace
  Markdown, se muestra como texto.
- Los tests cubren el parser de bloques y el énfasis inline; no hay dependencias
  nuevas ni cambios de datos.
- El binario crece solo por el módulo y sus tests no se incluyen en `dist`.
