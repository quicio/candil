## Context

Ver `proposal.md` y `specs/library-experience/spec.md`. La biblioteca ya tiene `LibraryOrder::Recent` (ordena por `last_opened` con fallback a `added`) y `Library::rescan()` incremental por mtime/size. La cabecera ya calcula un modo compacto y reserva espacio para búsqueda, añadir carpeta y overflow. `start_scan` corre en un hilo y `poll_scan` actualiza el estado.

## Goals / Non-Goals

**Goals:**

- Retomar el último libro con un clic, sin buscar la tarjeta.
- Refrescar la biblioteca al volver a la ventana, de forma barata y no bloqueante.
- Reutilizar el escaneo incremental y el feedback existentes.

**Non-Goals:**

- Watcher del sistema en tiempo real.
- Series/colecciones, duplicados, virtualización o búsqueda full-text.
- Cambiar `library.json` o el formato de progreso.

## Decisions

### 1. El “reciente” se calcula por `last_opened`

Se elige el libro con mayor `last_opened` entre los que no están `missing` y no tienen `error`. Si no hay ninguno, el botón no se muestra. `last_opened` se actualiza al abrir y al guardar progreso, así que refleja la última lectura real.

**Alternativa descartada:** usar el primer elemento de la grilla; depende del orden elegido por el usuario y del filtro activo. El atajo debe ser independiente de la vista.

### 2. Botón en la cabecera, con reserva explícita

Se inserta después del buscador y antes de “Añadir carpeta…”. El cálculo de ancho del buscador reserva el espacio del botón (`92` compacto, `158` amplio) para que no se superpongan. En modo compacto el texto es “Continuar”; si el ancho disponible es menor a 380 px, no se muestra. El tooltip lleva título y autor.

**Alternativa descartada:** un banner arriba de la grilla; ocupa altura y compite con las tarjetas. **Alternativa descartada:** una tarjeta “seguir leyendo” separada; duplica el libro y rompe la grilla uniforme.

### 3. Refresco al recuperar foco con debounce

Se observa la transición `InputState.focused` de falso a verdadero en la pantalla de biblioteca. Se llama a `start_scan(None, ctx)` si:

- No hay un escaneo en curso.
- Hay carpetas vigiladas.
- No es una captura (`--shot`).
- Pasaron al menos 30 s desde el último escaneo automático.

`start_scan` ya guarda la biblioteca, escanea en un hilo y muestra “Escaneando…”. El debounce evita ráfagas al cambiar de app.

**Por qué no un watcher:** un watcher del sistema es distinto por plataforma, consume recursos y notifica múltiples eventos; para una biblioteca personal el foco es suficiente y multiplataforma. **Alternativa descartada:** escanear en cada frame o cada N segundos; gasta CPU sin que el usuario lo haya pedido.

### 4. Estado y persistencia

Se agregan dos campos efímeros a `Lector`: `was_focused: bool` y `last_auto_scan: Option<Instant>`. No se persisten. El escaneo conserva búsqueda, orden, filtro y selección porque solo reemplaza la lista de libros y las texturas de portada.

### 5. Capturas

El sandbox de capturas necesita un libro con `last_opened` para mostrar el botón. Se siembra P&P con progreso y `last_opened` antes de las capturas de biblioteca; así `library-full.png` refleja el atajo.

## Risks / Trade-offs

- **Escaneos al alt-tab** → debounce de 30 s y guardas por escaneo/carpetas/captura.
- **El orden cambia tras el refresco** → es el comportamiento esperado de “Reciente”; la búsqueda y el filtro se conservan.
- **El reciente cambia mientras se mira la grilla** → el botón puede apuntar a otro libro; se recalcula por frame y el tooltip lo aclara.
- **Cabecera angosta** → se oculta el botón antes de superponer controles; se prueba a 720×600 y 560 de ancho.
- **Coste del escaneo** → incremental por mtime/size; en el corpus actual es de milisegundos con caché.

## Migration Plan

Sin migración. No cambia ningún archivo persistido. Rollback: quitar el botón y la transición de foco.

## Open Questions

Ninguna que cambie el alcance. Series/colecciones, duplicados y virtualización quedan para las siguientes fases de M13.
