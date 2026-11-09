## Context

Ver `proposal.md` y `specs/reading-workspace/spec.md`. Hoy el lector resuelve un enlace interno con `Book::link_target(href, chapter)` y llama a `jump_to(spine, anchor)`: el usuario pierde el párrafo y no hay retorno. Los enlaces externos se descartan en `link_target`, y el parser ya conserva `Run.href` y `Block.anchor`, así que no hace falta tocar el modelo `Block`.

La posición de lectura se persiste como `Progress { chapter, ratio, anchor, anchor_ratio }`; `top_block` y `block_fraction` ya calculan el ancla visible. El lector sabe restaurar un `Progress` con `pending_restore` o `load_chapter`.

## Goals / Non-Goals

**Goals:**

- Previsualizar el destino de un enlace interno sin cambiar de capítulo ni de scroll.
- Permitir el salto confirmado y un regreso exacto a la posición anterior.
- Funcionar con notas en el mismo XHTML y en otro documento del spine.
- Mantener la lectura como prioridad: popover chico, sin modal, sin robar foco.

**Non-Goals:**

- Abrir enlaces externos.
- Detectar `epub:type="noteref"` como metadato del parser.
- Editar notas al pie, historial de saltos de varios niveles o popovers anidados.
- Cambiar la persistencia de progreso o el formato del EPUB.

## Decisions

### 1. Vista previa genérica de enlaces internos, no solo `noteref`

Cualquier enlace interno resoluble abre la vista previa. No se agrega un campo `note_ref` al parser.

**Por qué:** muchos EPUB no declaran `epub:type="noteref"` y el lector ya resuelve el destino; el popover ofrece "Ir al texto", así que no se pierde ninguna acción. **Alternativa descartada:** detectar solo `noteref`; es más preciso, pero deja afuera libros reales y obliga a cambiar `Run`, ambos parsers y sus fixtures. Si más adelante molesta para referencias cruzadas, se puede restringir sin cambiar el modelo.

### 2. Extraer la nota del bloque destino en el momento del clic

Para el mismo capítulo se usa `self.blocks`; para otro documento se llama a `Book::chapter_blocks(spine)` y se descarta el `Vec` después de extraer el texto. La extracción toma el bloque del ancla; si el ancla es un bloque invisible vacío, toma el siguiente bloque con texto; si el ancla no existe, el primer bloque con texto del capítulo. Se muestra un solo bloque, truncado a 600 caracteres.

**Por qué:** una nota suele ser un párrafo; mostrar más bloques puede arrastrar la nota siguiente. El salto completo sigue disponible con "Ir al texto". **Alternativa descartada:** renderizar el capítulo destino completo en un modal; es más pesado, tapa la lectura y duplica el trabajo del lector.

### 3. La vista previa no navega

El clic guarda `LinkPreview { spine, anchor, chapter_title, text, rect, is_note }`. Solo "Ir al texto" llama a `jump_to`. Cerrar, Escape o desplazarse limpian el estado sin tocar `chapter`, `blocks` ni `scroll`.

**Por qué:** el objetivo es no abandonar el texto. **Alternativa descartada:** navegar al destino y volver con un botón; sigue perdiendo el contexto y obliga a repintar el capítulo.

### 4. Regreso de un nivel con `Progress`

Antes de saltar se captura `current_progress()` con la misma lógica que `persist` (`top_block` + `block_fraction`). Se guarda en `back_spot: Option<Progress>`; "Ir al texto" lo setea después del `jump_to` y muestra un botón flotante "← Volver al texto". Al volver: si el capítulo cambió, `load_chapter(spot.chapter, Some(spot))`; si es el mismo, `pending_restore = Some(spot)`.

**Por qué:** un solo nivel cubre el caso nota → texto → volver sin agregar una pila ni un historial visible. **Alternativa descartada:** pila de saltos; útil, pero más estado, más UI y fuera del alcance de esta iteración. La navegación manual (índice, capítulos, búsqueda, anotaciones) limpia `back_spot` para no ofrecer un regreso engañoso.

### 5. Popover anclado al clic

Se dibuja con `egui::Area` en `Order::Foreground`, cerca del punto clickeado, con ancho máximo 380 y alto máximo 200 con scroll interno. Se coloca debajo del clic si hay lugar y arriba si no; se recorta a la pantalla. Título "Nota al pie" si el texto del enlace parece un marcador (≤ 6 caracteres numéricos/romanos), "Enlace interno" en el resto. Acciones: "Ir al texto" (primaria) y "Volver" (secundaria), más una "×".

**Por qué:** sigue el patrón de la barra de selección y no cambia el layout. **Alternativa descartada:** ventana modal; interrumpe la lectura y compite con los paneles existentes.

### 6. Enlaces externos intactos

`Book::link_target` sigue devolviendo `None` para `http/https/mailto/...`; no se abre el navegador en este cambio. Se puede sumar después con una confirmación explícita.

## Risks / Trade-offs

- **Parsear otro capítulo en el clic** → puede costar decenas de ms en un capítulo enorme. Mitigación: se hace una sola vez por clic y se guarda solo el texto; si aparece un caso real lento, se agrega caché de capítulos previsualizados.
- **Vista previa genérica en referencias cruzadas** → agrega un paso a enlaces que antes saltaban directo. Mitigación: "Ir al texto" es la acción primaria; el título distingue "Enlace interno".
- **Nota partida en varios párrafos** → la vista previa muestra solo el primero. Mitigación: "Ir al texto" está siempre; el truncado evita popovers gigantes.
- **Un solo nivel de regreso** → dos saltos seguidos pisan el punto anterior. Mitigación: la navegación manual limpia el estado; una pila queda como mejora posterior.
- **Restauración same-chapter** → depende de `pending_restore` y de que `block_positions` estén pobladas. Se reutiliza la lógica de progreso ya probada; la captura y los tests cubren el caso.
- **Popover cerca del borde** → se recorta a la pantalla y se prueba a 720×600.

## Migration Plan

Sin migración de datos. `Progress`, `library.json` y `annotations.json` no cambian. Rollback: quitar el estado de vista previa y volver a `jump_to` directo en el clic de enlace.

## Open Questions

Ninguna que cambie el alcance. Restringir la vista previa a `noteref`, abrir enlaces externos y agregar una pila de saltos quedan como iteraciones posteriores.
