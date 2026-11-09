## 1. Modelo y migración

- [x] 1.1 Subir `SCHEMA_VERSION` a 2 y agregar `end_block: Option<usize>` y `parts: Vec<String>` a `Annotation`, con `#[serde(default)]`. Verificación: un JSON v1 sin esos campos deserializa.
- [x] 1.2 Implementar la migración en memoria al cargar: normalizar `parts` desde `quote`, dejar `end_block = None` y actualizar `schema_version`, sin reescribir el archivo. Verificación: test de carga v1 conserva todas las anotaciones.
- [x] 1.3 Agregar helpers de rango: `is_multi_block`, `paragraph_count`, `start_anchor`/`end_anchor` y `quote` unido desde `parts`. Verificación: tests de rangos de uno y varios bloques.
- [x] 1.4 Extender `rekey`, `sorted_for_book` y `sorted_all` para rangos multi-bloque sin cambiar su contrato. Verificación: tests existentes verdes y test de `rekey` con rango multi-bloque.

## 2. Selección extendida

- [x] 2.1 Reemplazar los campos planos de `Selection` por `anchor`/`focus` (bloque + offset), conservando `quote`, `parts`, `rect` y `edit_id`. Verificación: `cargo check`.
- [x] 2.2 Recolectar por frame la geometría de los bloques visibles (`índice`, rect, `Arc<Galley>`) y usarla para mapear el puntero a un bloque y offset. Verificación: la selección dentro de un bloque sigue igual.
- [x] 2.3 Implementar la extensión por arrastre cuando el puntero sale del bloque ancla: conservar el ancla, mover el foco al bloque bajo el puntero y reconstruir `parts`/`quote`. Verificación: selección visual de dos párrafos.
- [x] 2.4 Implementar Mayúsculas + clic en otro bloque como extensión desde el ancla actual. Verificación: mismo rango que por arrastre.
- [x] 2.5 Evitar que el `TextEdit` borre la selección mientras el botón está presionado; limpiar solo al soltar, con Escape o clic fuera. Verificación: la barra flotante permanece estable durante el arrastre.
- [x] 2.6 Ignorar selecciones vacías o formadas solo por espacios, imágenes y reglas al habilitar Destacar/Nota/Consultar. Verificación: no se ofrece la barra sin texto.

## 3. Resolución y render

- [x] 3.1 Convertir `resolve_annotation` a `Option<Vec<Segment>>` con offsets exactos, búsqueda secuencial por `parts` y fallo total si alguna parte no se encuentra. Verificación: tests de offsets, texto desplazado y coincidencia parcial.
- [x] 3.2 Adaptar `resolve_highlights` para volcar todos los segmentos al `HashMap` por bloque y resaltar los bloques intermedios completos. Verificación: test de rangos y revisión visual.
- [x] 3.3 Adaptar los usos de `resolve_annotation` (panel, `missing`, exportación y salto) al nuevo tipo. Verificación: `cargo clippy --all-targets -- -D warnings`.
- [x] 3.4 Hacer que el salto a una anotación multi-bloque use el bloque inicial y la primera parte. Verificación: “Ir al texto” deja visible el inicio del rango.

## 4. Panel, consulta y capturas

- [x] 4.1 Agregar un helper de vista previa de cita (colapsa espacios y saltos) y mostrar `· N párrafos` en las filas multi-bloque. Verificación: test del helper y captura del panel.
- [x] 4.2 Ajustar la creación de destacado/nota para guardar `block`, `start`, `end`, `end_block` y `parts` desde la selección. Verificación: roundtrip en `annotations.json`.
- [x] 4.3 Ajustar “Consultar” para usar la cita completa normalizada y el contexto del primer bloque, sin cambiar los límites actuales. Verificación: consulta sobre dos párrafos.
- [x] 4.4 Extender el flag de desarrollo `--selection` para sembrar una selección de dos bloques y agregar la captura a `scripts/captures.sh`. Verificación: captura a 980×780 y 720×600.

## 5. Pruebas, documentación y cierre

- [x] 5.1 Agregar tests de migración v1→v2, roundtrip, rangos invertidos, `rekey`, preview y resolución multi-bloque. Verificación: `cargo test`.
- [x] 5.2 Escribir ADR-0014 con el modelo `end_block` + `parts`, la migración y las alternativas descartadas. Verificación: ADR en `docs/adr/`.
- [x] 5.3 Actualizar `docs/HANDOFF.md`, `docs/ROADMAP.md` (M9), `docs/ARCHITECTURE.md`, `AGENTS.md`, `README.md` y `docs/CONVENTIONS.md`; quitar la limitación “no cruza párrafos”. Verificación: `rg` sin referencias obsoletas.
- [x] 5.4 Ejecutar `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `scripts/check.sh` y `--inspect tests/fixtures/edge.epub`. Verificación: todo verde.
- [x] 5.5 Medir `dist`, confirmar < 8 MB y actualizar los documentos con el tamaño. Verificación: `cargo build --profile dist` y `stat`.
- [x] 5.6 Validar y archivar el cambio OpenSpec. Verificación: `openspec validate seleccion-entre-parrafos --strict` y spec actualizada en `openspec/specs/reading-workspace/`.
