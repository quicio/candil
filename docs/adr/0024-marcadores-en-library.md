# ADR-0024: Marcadores de posición en `library.json`

- **Estado:** aceptado
- **Fecha:** 2026-11-10
- **Área:** lectura, datos, navegación

## Contexto

Candil guarda progreso, ajustes y anotaciones, pero no permite marcar una página o
un pasaje para volver después. Un marcador es una posición, no un rango de texto:
no necesita cita, color ni nota. Guardarlo como anotación vacía ensuciaría el
panel de destacados y el dibujo de rangos.

## Decisión

Guardar los marcadores dentro de `BookEntry.bookmarks` en `library.json`:

```json
{
  "id": 1,
  "progress": { "chapter": 4, "ratio": 0.42, "anchor": 10, "anchor_ratio": 0.3 },
  "label": "",
  "created": 1760000000
}
```

- `progress` reutiliza el mismo tipo que la persistencia de lectura, así el
  marcador sobrevive cambios de tipografía y layout.
- `label` queda vacío por ahora; la UI muestra el título del capítulo y el
  porcentaje. El campo permite agregar nombres en el futuro sin migrar.
- `id` es local al libro y se calcula como `max(id) + 1`; no se reutiliza tras
  borrar.
- Los marcadores se ordenan por capítulo y fracción.

### UI

- `B` guarda la posición actual; si ya hay un marcador a menos de 1 % en el mismo
  ancla, avisa y no duplica.
- `Mayús+B` abre el panel **Marcadores**, también disponible en la barra del
  lector.
- El panel permite añadir, saltar y eliminar. Saltar usa `load_chapter` con
  `pending_restore`, la misma ruta que el progreso.

### Persistencia

`library.json` ya se escribe atómico y entra completo en el respaldo de datos.
El escaneo incremental y `relink` copian `bookmarks` al reemplazar una entrada,
igual que `progress` y `settings`. No se agrega un archivo nuevo ni una entrada
al respaldo.

**Por qué no `annotations.json`:** una anotación exige cita/rango y aparece en el
panel de destacados; un marcador es una posición sin texto. **Por qué no
`config.json`:** el marcador pertenece a un libro y debe seguir su ruta al
reindexar o reasociar.

## Alternativas descartadas

- **Anotación con cita vacía:** contamina el panel y el render de rangos.
- **`bookmarks.json` aparte:** un quinto archivo de usuario, fuera del respaldo
  actual y con otra migración.
- **Solo guardar el último marcador por libro:** pierde el caso de varias
  posiciones de estudio.
- **Etiqueta obligatoria:** agrega fricción al gesto rápido; se deja para después.

## Consecuencias

- `library.json` gana un campo opcional; archivos viejos cargan sin migración.
- El respaldo existente incluye los marcadores sin cambios.
- Un escaneo o `relink` conserva los marcadores.
- No se agregan dependencias.
- Tests cubren alta, orden, borrado e idempotencia de la posición.
