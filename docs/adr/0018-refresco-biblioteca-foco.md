# ADR-0018: Refrescar la biblioteca al recuperar el foco

- **Estado:** aceptado
- **Fecha:** 2026-11-09
- **Área:** biblioteca, escritorio, rendimiento

## Contexto

La biblioteca indexa carpetas externas y no tiene watcher. Si el usuario agrega,
borra o reemplaza EPUB mientras Candil está en segundo plano, al volver ve una
lista vieja hasta pulsar “Refrescar”. Un watcher del sistema resuelve el caso en
tiempo real, pero se comporta distinto en macOS, Linux y Windows, consume
recursos y puede disparar muchos eventos por una sola operación.

`Library::rescan()` ya es incremental por `mtime`/`size` y con caché tarda
milisegundos en el corpus actual. `start_scan` corre en un hilo y la UI muestra
“Escaneando…”.

## Decisión

Refrescar al recuperar el foco:

- Se observa la transición de `InputState.focused` de falso a verdadero.
- Solo en la pantalla de biblioteca.
- Se llama a `start_scan(None, ctx)` si no hay un escaneo en curso, hay carpetas
  vigiladas, no es una captura `--shot` y pasaron al menos **30 segundos** desde
  el último refresco automático.
- El estado es efímero: `was_focused: bool` y `last_auto_scan: Option<Instant>`.
  No se persiste.
- El escaneo reutiliza el hilo, el indicador y la actualización incremental
  existentes; no se bloquea la UI.

## Alternativas descartadas

- **Watcher del sistema (`notify`, FSEvents, inotify, ReadDirectoryChangesW):**
  es la solución completa, pero suma una dependencia con comportamiento distinto
  por plataforma, manejo de eventos duplicados y un ciclo de vida que hay que
  probar en las tres. Para una biblioteca personal no compensa todavía.
- **Timer periódico:** escanea aunque el usuario no haya vuelto a la app; gasta
  CPU y disco sin una acción.
- **Solo manual:** es lo que había; el usuario no ve cambios hasta recordar
  “Refrescar”.
- **Escanear en cada frame:** inviable.

## Consecuencias

- La biblioteca se actualiza al volver a la ventana, sin intervención.
- Un alt-tab rápido no dispara más de un escaneo cada 30 s.
- Si el usuario cambia de app mientras se escanea, el hilo sigue y `poll_scan`
  actualiza al volver.
- El orden “Reciente” puede cambiar después del refresco; búsqueda, orden y
  filtro se conservan.
- No cubre cambios mientras la ventana está enfocada; para eso sigue estando
  “Refrescar” y, a futuro, un watcher.
- No cambia `library.json` ni el formato de progreso; rollback trivial.
