## Context

`consulta_ui` y el historial usaban `ui.label(RichText::new(&body))`. Las notas
usaban un `label` truncado. Los modelos devuelven Markdown básico de forma
habitual; el usuario lo veía crudo.

## Goals / Non-Goals

**Goals:**

- Dar jerarquía visual a respuestas largas sin agregar dependencias.
- Reutilizar la tipografía y los temas de la app.
- Cubrir el subconjunto que producen los modelos y las notas.

**Non-Goals:**

- Compatibilidad Markdown completa, HTML, tablas o imágenes.
- Enlaces clicables dentro de un párrafo.
- Renderizar diccionario/traducción como Markdown.

## Decisions

### 1. Renderer propio sobre `LayoutJob`

`src/markdown.rs` parsea líneas a bloques y usa `LayoutJob` para el énfasis
inline. Así respeta el wrapping, la fuente Literata, los colores del tema y el
foco de la UI sin depender de un motor externo.

### 2. Subconjunto explícito

Bloques: párrafos, `#`–`######`, listas `-`/`*`/`+`, numeradas, citas `>`, código
``` y reglas. Inline: `**negrita**`, `*itálica*`, `` `código` ``. El código usa
monoespaciada y el acento del tema.

Una marca sin cerrar no pierde texto: se muestra el contenido restante con el
estilo activo.

### 3. Dónde se aplica

- Respuestas de IA del panel y del historial (solo `ConsultaKind::Ia`).
- Vista previa inline de 220 caracteres en el panel de notas.
- Vista previa completa debajo del editor de notas.
- Diccionario/traducción siguen en texto plano para no interpretar asteriscos
  legítimos.

### 4. Sin dependencias

No se agrega `pulldown-cmark` ni un motor HTML. El costo de mantenimiento es un
módulo pequeño y testeado; el binario no crece de forma significativa.

## Risks / Trade-offs

- **Subconjunto incompleto** → tablas, enlaces e imágenes se muestran como texto.
  Aceptado por ahora.
- **Asteriscos literales** → un `*` suelto puede abrir énfasis. En IA/notas es
  raro; diccionario/traducción no pasan por el renderer.
- **Altura variable** → las notas con listas ocupan más en el editor; la vista
  previa tiene scroll propio.

## Migration Plan

Sin migración ni datos nuevos. Rollback: volver a `ui.label` en los cuatro
puntos de uso y quitar el módulo.
