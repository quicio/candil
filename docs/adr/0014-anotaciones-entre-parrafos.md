# ADR-0014: Rangos de anotación entre párrafos

- **Estado:** aceptado
- **Fecha:** 2026-11-09
- **Área:** lectura, anotaciones, persistencia

## Contexto

El lector dibuja cada bloque con su propio `TextEdit`. La selección nativa de
egui solo alcanza el widget con foco, así que un destacado o una nota no podía
cruzar de un párrafo a otro. Los offsets de `Annotation` (`block`, `start`,
`end`) describen un único bloque y `annotations.json` tiene `schema_version: 1`.

Extender el rango sin reescribir el lector exige conservar la selección nativa,
sumar geometría de los bloques visibles y guardar suficiente información para
reanclar el rango cuando el libro cambia.

## Decisión

### Modelo de datos v2

`Annotation` conserva `block`, `start` y `end` como extremos del rango y agrega:

- `end_block: Option<usize>` — bloque final; `None` significa “mismo bloque”,
  que es exactamente el formato v1.
- `parts: Vec<String>` — texto seleccionado en cada bloque cubierto, en orden.
- `schema_version` sube a 2.

`quote` sigue siendo el texto completo para mostrar y exportar, con las partes
unidas por `\n`. La fuente de verdad para reanclar es `parts`; así una cita que
contiene saltos de línea no se confunde con la separación entre bloques.

La migración es en memoria al cargar: si `schema_version < 2`, se completa
`parts` desde `quote` y se deja `end_block = None`. El archivo se reescribe solo
en la próxima mutación. Una versión anterior que lea un archivo v2 ignora los
campos nuevos y sigue mostrando la anotación en su primer bloque.

### Selección

La selección activa se modela con `anchor` y `focus` (bloque + offset). El
usuario puede:

1. Arrastrar dentro de un bloque, como antes.
2. Arrastrar fuera del bloque: el lector guarda por frame un mapa de bloques
   visibles (`rect` + `Arc<Galley>`) y mueve el foco al bloque bajo el puntero
   usando `Galley::cursor_from_pos`.
3. **Mayúsculas + clic** en otro párrafo: extensión desde el ancla actual, sin
   depender del arrastre.

Mientras el botón está presionado no se borra la selección por un cursor
colapsado. Al soltar, una selección multi-bloque se conserva hasta un clic
normal, Escape o un desplazamiento.

El rango se previsualiza con el mismo `HashMap` de resaltados que usan las
anotaciones guardadas, así que todos los bloques cubiertos se pintan con el
color activo.

### Resolución y reanclaje

`resolve_annotation` devuelve una lista de segmentos por bloque:

1. Prueba los offsets guardados y valida cada segmento contra su parte.
2. Si fallan, busca las partes en orden desde el bloque inicial, sin retroceder.
3. Si solo coincide una parte, devuelve `None`: el panel muestra “no encontrada”
   en vez de resaltar un rango engañoso.

Los bloques intermedios se resaltan completos. Los bloques sin texto (imágenes,
reglas, anclas invisibles) no generan parte ni segmento.

## Alternativas descartadas

- **Un único `TextEdit` con todo el capítulo:** rompe el progreso por bloque,
  las anclas y la carga incremental; además obliga a recalcular el layout de
  todo el capítulo en cada frame.
- **Un widget de texto nuevo:** pierde edición, IME, selección por teclado y los
  atajos existentes; es una reescritura del lector, no una extensión.
- **Guardar cada segmento en un `Vec`:** un rango contiguo se describe con dos
  extremos; los segmentos se derivan. Guardarlos duplica información y complica
  la migración.
- **`end_block: usize` con `serde(default)`:** el default `0` sería incorrecto
  para anotaciones de bloques posteriores; `Option` distingue “mismo bloque”.
- **Resaltar solo por la cita completa:** una cita multi-línea no se puede
  buscar de forma fiable en un bloque; las partes evitan falsos positivos.

## Consecuencias

- `annotations.json` crece con una parte por bloque; el impacto es despreciable
  frente a `library.json`.
- El arrastre fuera del viewport no hace auto-scroll; el rango se recorta al
  último bloque visible. Queda como mejora posterior.
- Copiar todo el rango al portapapeles, editar el rango de una anotación
  existente y seleccionar entre capítulos quedan fuera de alcance.
- No se agregan dependencias ni cambia el tamaño del binario de forma
  significativa.
- Tests nuevos en `annotations.rs` (migración, roundtrip, normalización) y
  `app.rs` (rangos exactos, reanclaje, coincidencia parcial, partes y preview).
