---
id: adr-0005
status: aceptada
version: 1.1.0
updated: 2026-11-06
---

# ADR-0005 — Configuración y progreso en JSON

## Contexto

Necesitamos persistir: preferencias (tema, tamaño, interlineado, ancho, espaciado),
último libro abierto, recientes y posición de lectura por libro. Volumen: decenas
de entradas, escrituras frecuentes (cada 700 ms mientras se lee).

## Decisión

Un único JSON en `~/Library/Application Support/lector/config.json`
(`dirs::config_dir()` en otros SO), serializado con `serde_json`, escritura
completa y atómica por reemplazo de archivo.

Estructura: `settings`, `last_book`, `recent[]`, `progress{ ruta → {chapter, offset} }`.

Guardado con debounce (700 ms de cambios) + `on_exit`. Si el archivo está corrupto
o falta, se arranca con `Default` — nunca se bloquea el arranque.

## Consecuencias

- Cero dependencias extra, cero C, cero migraciones de esquema.
- Legible y editable a mano (útil para depurar y para que Hugo lo inspeccione).
- `#[serde(default)]` en las structs permite agregar campos sin romper archivos
  viejos (así se agregó `paragraph_space`).
- Rewrite completo del archivo por guardado: aceptable hasta miles de entradas.
- Sin soporte para escrituras concurrentes (una sola instancia de la app; si se
  abren dos, gana la última en escribir).

## Alternativas descartadas

- **SQLite (`rusqlite`)**: transacciones y consultas, pero suma ~1.5 MB al binario
  y una dependencia con C. No hay consultas que lo justifiquen a esta escala.
- **TOML**: cómodo para configuración escrita a mano, incómodo para mapas
  anidados que se reescriben seguido.
- **Un archivo por libro**: muchos archivos chicos, peor para "recientes" y para
  la estantería de M2.

## Revisar si

- La biblioteca supera ~5.000 libros (ADR-0007 define el mismo umbral).
- Se necesita búsqueda full-text (ahí sí conviene un índice aparte).

## Enmienda (2026-11-06) — el progreso vive en `library.json`

M2 cambia la propiedad del progreso: `library.json` es la fuente para los libros
indexados y `config.json` deja de recibir nuevas entradas. El campo `progress` se
mantiene en `Config` con `#[serde(default)]` para leer archivos viejos; al
arrancar (y después de cada escaneo) `Library::migrate_progress_from` vuelca las
claves que coinciden con una entrada, no pisa un progreso más nuevo de la
biblioteca y deja las claves de libros no indexados para cuando se agregue su
carpeta.

La escritura de ambos archivos pasa por `src/store.rs` (temporal + `rename` +
backup de corruptos), así que la garantía atómica de este ADR se mantiene. Esta
enmienda no supersede el ADR: solo documenta el cambio de dueño del progreso que
pedía la spec 20 (T8).
