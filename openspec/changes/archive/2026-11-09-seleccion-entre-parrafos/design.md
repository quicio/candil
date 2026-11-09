## Context

Ver `proposal.md` para la motivación y `specs/reading-workspace/spec.md` para el contrato. Hoy cada bloque se dibuja con su propio `TextEdit::multiline`; egui solo selecciona dentro del widget con foco. `Annotation` guarda `block`, `start`, `end` y `quote`; `resolve_highlights` ya produce un `HashMap<bloque, Vec<(inicio, fin, color)>>`, así que el render por bloque está preparado para recibir varios rangos. `annotations.json` usa `#[serde(default)]` y escritura atómica, sin validador de versión.

Restricciones: egui 0.32, sin dependencias nuevas, sin reemplazar el modelo `Block` ni el parser, y el texto debe seguir siendo seleccionable, editable por IME y accesible por teclado dentro de cada bloque.

## Goals / Non-Goals

**Goals:**

- Extender la selección a bloques consecutivos del mismo capítulo con una interacción natural y una alternativa de teclado.
- Persistir el rango con offsets y citas por bloque, migrando v1 sin intervención.
- Reutilizar el render y el panel existentes, con cambios acotados.
- Mantener el coste en memoria/CPU proporcional a los bloques visibles.

**Non-Goals:**

- Reescribir el lector como un único widget de texto continuo.
- Selección entre capítulos, copia multi-bloque al portapapeles o edición del rango de una anotación existente.
- Cambiar el parser, el formato de `library.json` o la escritura atómica.
- Auto-scroll durante el arrastre fuera del viewport (se recorta al último bloque visible).

## Decisions

### 1. Modelo de rango: `end_block` opcional + `parts`

`Annotation` conserva `block` y `start` como ancla inicial y `end` como offset final. Se agregan:

- `end_block: Option<usize>` — `None` significa “el mismo bloque” (compatibilidad v1).
- `parts: Vec<String>` — texto seleccionado en cada bloque cubierto, en orden.

`quote` sigue siendo el texto completo para mostrar y exportar; se construye uniendo `parts` con `\n`. `schema_version` sube a 2.

**Por qué no un `Vec<Segment>`:** un rango contiguo se describe con dos extremos; los segmentos se recalculan. Guardar todos los segmentos duplica información y hace más frágil la migración. **Por qué no `end_block: usize` con default:** el default `0` sería incorrecto para anotaciones de bloques posteriores; `None` es inequívoco y compatible.

La migración es en memoria, al cargar: si `schema_version < 2`, se completa `end_block = None` y `parts = vec![quote]` si está vacío. No se reescribe el archivo hasta la próxima mutación, para no tocar datos del usuario al abrir.

### 2. Anclas y normalización

La selección en curso se modela como `anchor` (bloque + offset) y `focus` (bloque + offset), ambos dentro del capítulo actual. El rango guardado normaliza por orden `(bloque, offset)`; si el usuario extiende hacia arriba, `anchor` y `focus` se intercambian al persistir. Se descartan selecciones cuyos `parts` estén vacíos o solo tengan espacios, y los bloques sin texto (`Image`, `Rule`, anclas invisibles) no generan parte ni segmento.

### 3. Interacción: arrastre con geometría por frame + Mayúsculas + clic

- Dentro de un bloque se mantiene el `TextEdit` actual: arrastre, doble clic, teclado e IME siguen igual.
- Durante el arrastre, el lector arma un mapa de los bloques visibles con `(índice, rect, Arc<Galley>)`. Si el puntero sale del bloque ancla, se busca el bloque bajo el puntero (o el más cercano si cae en una imagen o en el espacio entre párrafos) y se calcula el offset con `Galley::cursor_from_pos`. El foco se mueve a ese punto y el ancla se conserva.
- **Mayúsculas + clic** en otro bloque usa el mismo cálculo y la misma reconstrucción del rango; sirve como alternativa de teclado/ratón y como caso testeable sin arrastre.
- El `TextEdit` no debe borrar la selección mientras el puntero está presionado: la limpieza por cursor colapsado se pospone a `pointer.any_released()`/`primary_up`.
- La barra flotante usa la unión del rect del primer y último extremo; si queda fuera de pantalla, se aplica la lógica existente de arriba/abajo.

**Alternativas descartadas:** un único `TextEdit` con todos los párrafos separados por líneas en blanco (rompe anclas, progreso por bloque y carga de capítulos); un widget de texto nuevo (pierde edición/IME y aumenta el riesgo); pintar una capa de selección sobre el texto sin usar el cursor real (duplica hit-testing y no funciona con teclado).

### 4. Resolución y render

`resolve_annotation` pasa de `Option<(bloque, inicio, fin)>` a `Option<Vec<Segment>>`. Para un rango multi-bloque:

1. Se prueban los offsets exactos; cada segmento se valida contra su `part` (o contra el texto del bloque si `parts` está vacío por herencia).
2. Si no coinciden, se buscan las `parts` en orden, desde el bloque inicial en adelante, sin retroceder; cada coincidencia fija un bloque y offsets.
3. Si alguna parte falla, se devuelve `None` (el panel muestra “no encontrada”) para no resaltar un rango engañoso.

Los segmentos se vuelcan al `HashMap` existente: el primer bloque desde `start`, los intermedios completos y el último hasta `end`. El color y el solapamiento con la búsqueda no cambian.

### 5. Panel, salto y consulta

- El salto usa el bloque inicial y la primera parte; el render resalta todo el rango al llegar.
- La fila del panel colapsa espacios y saltos de línea con un helper de vista previa y agrega `· N párrafos` cuando `parts.len() > 1`.
- “Ir al texto” y el borrado operan sobre el `id`, sin cambios.
- Consultar usa la cita completa (con los espacios normalizados) y el contexto del primer bloque, manteniendo el límite actual de 1.500 caracteres de contexto.
- Editar una nota o cambiar el color no altera el rango.

### 6. Pruebas

- Unitarias puras en `annotations.rs`: migración v1→v2, roundtrip v2, rangos invertidos, `rekey`, `parts` vacías.
- Unitarias en `app.rs`: construcción de `parts` desde bloques (incluidos vacíos), `resolve_annotation` multi-bloque con offsets exactos, con texto desplazado y con coincidencia parcial; preview del panel.
- Verificación visual con `--selection` extendido para sembrar una selección de dos bloques; captura a 980×780 y 720×600.
- `cargo test`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `scripts/check.sh` y `--inspect tests/fixtures/edge.epub`.

## Risks / Trade-offs

- **Arrastre fuera del `TextEdit`** → el widget puede perder el foco o colapsar el cursor; se conserva el ancla en `Selection` y se reconstruye el rango mientras el botón siga presionado. Mitigación adicional: Mayúsculas + clic siempre disponible.
- **Bloques sin texto entre párrafos** → el mapa de geometría los salta; el rango se ancla al bloque de texto más cercano y `parts` omite los vacíos. Se documenta que una imagen entre dos párrafos no se incluye en la cita.
- **Texto del libro modificado** → la búsqueda secuencial por partes puede fallar; en ese caso se informa “no encontrada” en vez de resaltar mal. El backup de EPUB (ADR-0013) no cubre ediciones externas.
- **JSON más grande** → `parts` duplica la cita por bloque; el tamaño sigue siendo despreciable frente a `library.json`. Sin cambios de rendimiento perceptibles.
- **Selección muy larga** (capítulo entero) → `quote` y `parts` crecen; el panel trunca y la consulta mantiene sus límites. No se agrega un tope artificial porque el usuario puede querer citar un pasaje largo.
- **Tests de UI limitados** → la lógica de rangos se concentra en funciones puras; el arrastre se valida visualmente con capturas y la alternativa Shift+clic.

## Migration Plan

1. Subir `SCHEMA_VERSION` a 2 y migrar en memoria al cargar; el archivo se reescribe solo cuando el usuario crea, edita o borra una anotación.
2. Rollback: una versión anterior ignora `end_block` y `parts` gracias a `serde` y sigue leyendo `block`/`start`/`end`; las anotaciones multi-bloque se verían solo en su primer bloque, sin corromper datos.
3. No hay migración de `library.json` ni de progreso.

## Open Questions

Ninguna que cambie el alcance; el comportamiento de auto-scroll al arrastrar fuera del viewport queda explícitamente fuera y se puede agregar después sin cambiar el modelo.
