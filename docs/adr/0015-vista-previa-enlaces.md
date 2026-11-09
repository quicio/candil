# ADR-0015: Vista previa de enlaces internos y regreso a la lectura

- **Estado:** aceptado
- **Fecha:** 2026-11-09
- **Área:** lectura, navegación, notas al pie

## Contexto

Los enlaces internos ya saltan al destino con `Book::link_target` y
`jump_to`. En un libro con notas al pie, ese salto obliga a abandonar el párrafo
y no hay forma de volver al punto exacto. El lector tampoco distingue una nota de
una referencia cruzada: cualquier `href` interno es un salto.

El parser conserva `Run.href` y `Block.anchor`, y el progreso ya se modela como
`Progress { chapter, ratio, anchor, anchor_ratio }`. No hace falta cambiar el
modelo `Block` ni la persistencia.

## Decisión

### Vista previa genérica

Cualquier enlace interno resoluble abre una vista previa flotante con el texto
del bloque destino. No se agrega un campo `note_ref` ni se parsea
`epub:type="noteref"`.

La vista previa:

- Se ancla cerca del clic, con ancho máximo 380 y alto máximo 200 con scroll.
- Muestra el texto del bloque del ancla; si el ancla es un bloque invisible,
  toma el siguiente bloque con texto; si el ancla no existe, el primero con
  texto. Si el bloque es un encabezado, muestra el párrafo siguiente.
- Trunca a 600 caracteres.
- Título “Nota al pie” si el texto del enlace parece un marcador (`1`, `iv`,
  `[2]`), “Enlace interno” en el resto.
- Acciones “Ir al texto” y “Volver”, más “×”.

**Por qué no `noteref`:** muchos EPUB no lo declaran y el lector ya resuelve el
destino. El popover conserva “Ir al texto”, así que una referencia cruzada no
pierde funcionalidad. **Alternativa descartada:** cambiar `Run`, ambos parsers
y los fixtures para detectar solo notas; más preciso, pero deja afuera libros
reales.

### Sin navegar hasta confirmar

El clic solo guarda `LinkPreview { spine, anchor, chapter_title, text, rect,
is_note }`. Para el mismo capítulo se lee `self.blocks`; para otro documento se
llama a `Book::chapter_blocks(spine)` y se descarta el `Vec` después de extraer
el texto. El capítulo visible no cambia hasta “Ir al texto”.

**Alternativa descartada:** navegar al destino y volver con un botón; pierde el
contexto y repinta el capítulo. **Alternativa descartada:** renderizar el
capítulo destino completo en un modal; es más pesado y tapa la lectura.

### Regreso de un nivel

Antes de saltar se captura `current_progress()` con la misma lógica que
`persist` (`top_block` + `block_fraction`). Se guarda en
`back_spot: Option<Progress>`. “Ir al texto” lo setea después del `jump_to` y
muestra un botón flotante “← Volver al texto”. Al volver:

- Otro capítulo: `load_chapter(spot.chapter, Some(spot))`.
- Mismo capítulo: `pending_restore = Some(spot)`.

La navegación manual (índice, capítulos, búsqueda, anotaciones, Ir a) limpia el
`back_spot` para no ofrecer un regreso engañoso.

**Por qué un solo nivel:** cubre nota → texto → volver sin agregar una pila ni
un historial visible. **Alternativa descartada:** pila de saltos; útil, pero
fuera de alcance. Se puede sumar después sin cambiar este contrato.

### Cierre

“Volver”, “×”, Escape o desplazarse cierran la vista previa sin navegar. Los
enlaces externos siguen ignorados: no se abre el navegador en este cambio.

## Consecuencias

- El clic en una referencia cruzada agrega un paso (preview → “Ir al texto”).
  Se acepta porque la acción primaria es visible y el título distingue una nota.
- Parsear otro capítulo en el clic puede costar decenas de ms en un capítulo
  enorme. Se hace una vez por clic y se guarda solo el texto; si aparece un caso
  real lento, se agrega caché.
- Una nota partida en varios párrafos muestra solo el primero; “Ir al texto”
  sigue disponible.
- No hay historial de varios niveles ni popovers anidados.
- No se agregan dependencias ni cambia el formato de `annotations.json`,
  `library.json` o `config.json`.
- Tests nuevos para extracción, marcadores, anclas intra/intercapítulo y
  `notes.epub`; la vista previa se verifica con `--link-preview`.
