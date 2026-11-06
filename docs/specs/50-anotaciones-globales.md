---
id: spec-50
title: Anotaciones globales, colores y exportación
status: implementada
version: 1.0.0
updated: 2026-11-06
milestone: M5
depends_on: [spec-45, spec-20, adr-0005]
---

# Spec 50 — Anotaciones globales, colores y exportación

> Segunda etapa de la spec 45. El MVP por libro ya está cerrado; esto agrega
> índice global, varios colores y exportación Markdown.

## Objetivo

Ver y administrar los destacados de **todos** los libros, elegir un color al
destacar y exportar las citas/notas a Markdown con el libro y el capítulo.

## Alcance

Entra:

1. **Panel global**: el panel “Destacados y notas” gana un selector
   `Este libro / Todos`.
   - “Este libro”: comportamiento actual (capítulo + título + salto).
   - “Todos”: lista ordenada por libro, capítulo y posición; muestra el título
     del libro y `Cap. N`. `Ir` abre el libro si hace falta y salta al bloque.
   - Borrar y editar nota siguen funcionando en ambos modos.
2. **Tres colores**: `color: 0` = acento del tema (default), `1` = ámbar,
   `2` = verde. El color se elige en la barra flotante y en el editor de nota;
   los swatches de la barra no destacan solos: seleccionan el color y
   `Destacar`/`H` lo aplica.
3. **Exportación Markdown** con `rfd` (ya es dependencia): botón “Exportar”
   en el panel; exporta el alcance visible (libro o todos) a un `.md` elegido
   por el usuario.
4. Los destacados viejos (`color` ausente) se leen como `0` por `serde(default)`.

No entra:

- Sincronización entre dispositivos.
- Export a PDF/DOCX/HTML.
- Varios colores por fragmento o subrayado tipo Kindle.
- Notas sueltas sin cita.

## Modelo

`Annotation.color: u8` ya existe; se usa sin cambiar el esquema. Se agrega:

```rust
impl Annotations {
    pub fn sorted_all(&self) -> Vec<&Annotation>; // (book, chapter, block, start, id)
    pub fn update_color(&mut self, id: u64, color: u8) -> bool;
}
```

El color no se guarda en `library.json`; sigue viviendo en `annotations.json`.

## UI

```
barra flotante:
  [●][●][●]  Destacar  Nota
   ↑ color activo (se recuerda entre selecciones)

panel:
  [ Este libro ] [ Todos ]                [ Exportar… ]
  “cita…”                                      ●
  nota
  Libro / Cap. 7             [Ir] [Editar] [×]
```

- Los swatches opacos usan el color real; el fondo del texto usa el mismo color
  con alfa (~90) para no tapar la tipografía.
- El color activo es un campo del lector (`active_color`), default 0.
- El editor de nota muestra los mismos tres swatches; al guardar una nota nueva
  se usa el color elegido y al editar una existente se puede cambiar.
- El panel lista un punto del color de cada anotación; el mismo botón cicla
  0→1→2 al hacer click (sin abrir menú).

## Exportación

Formato estable y legible:

```markdown
# Destacados y notas

## El mito de Sísifo

### Cap. 10 — El suicidio filosófico

> Este salto puede, por lo menos, aclararnos un poco

La duda como método.

## Otro libro
...
```

- Las citas con saltos de línea se aplanan a una sola línea.
- Si no hay anotaciones, no se abre el diálogo y se avisa en el toast.
- El archivo se escribe con `std::fs::write` en la ruta elegida (es una
  exportación del usuario, no configuración interna).

## Tareas

### T1 — Modelo (hecha)
- `sorted_all` y `update_color`; tests de orden y color.

### T2 — Colores (hecha)
- `active_color`, swatches en barra y editor, `H` y `Destacar` usan el color.
- Render con `(inicio, fin, color)` por rango; los destacados de colores
  distintos pueden convivir en el mismo bloque.
- Test de roundtrip del color y de rangos solapados.

### T3 — Panel global (hecha)
- Selector de alcance; título de libro en modo global; `Ir` abre otro libro.
- Borrar/editar en ambos modos; los archivos faltantes se avisan sin romper.

### T4 — Exportación (hecha)
- `export_markdown(entries)` puro + test; botón con `rfd::FileDialog`.
- Nombre sugerido `destacados.md`; toast con la ruta.

### T5 — Verificación (hecha)
- Tests de modelo/export; captura del panel global con dos libros y una
  exportación de prueba; `cargo test` verde y clippy limpio.

## Criterio de salida

1. Un libro puede tener destacados de los tres colores y se ven distintos.
2. El panel “Todos” lista anotaciones de varios libros y `Ir` abre el libro
   correcto en el bloque correcto.
3. Exportar produce un Markdown con títulos, citas y notas; sin anotaciones no
   genera archivo.
4. Borrar/editar desde el panel global funciona y persiste.
5. Tests y capturas revisadas.
