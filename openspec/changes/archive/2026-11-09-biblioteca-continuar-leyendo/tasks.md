## 1. Continuar leyendo

- [x] 1.1 Calcular el libro reciente por `last_opened`, filtrando faltantes y errores. Verificación: con P&P abierto aparece el atajo; sin libros abiertos no.
- [x] 1.2 Agregar el botón a la cabecera después del buscador, con reserva de ancho, etiqueta compacta y tooltip. Verificación: captura a 980×780 y 720×600.
- [x] 1.3 Abrir el libro reciente con el mismo camino que una tarjeta. Verificación: el botón abre el EPUB y actualiza `last_opened`.

## 2. Refresco al foco

- [x] 2.1 Agregar `was_focused` y `last_auto_scan` a `Lector`. Verificación: `cargo check`.
- [x] 2.2 Detectar la transición de foco en la biblioteca y llamar a `start_scan` con debounce de 30 s, guardas de escaneo/carpetas/captura. Verificación: el indicador “Escaneando…” aparece al volver a la ventana.
- [x] 2.3 Verificar que búsqueda, orden y filtro sobrevivan al refresco. Verificación: revisión manual.

## 3. Capturas, documentación y cierre

- [x] 3.1 Sembrar `last_opened` y progreso de P&P en `scripts/captures.sh` y regenerar `library-*.png`. Verificación: la captura muestra “Continuar leyendo”.
- [x] 3.2 Escribir ADR-0018 sobre el refresco al foco y las alternativas (watcher, timer). Verificación: ADR en `docs/adr/`.
- [x] 3.3 Actualizar `README.md`, `docs/HANDOFF.md` y `docs/ROADMAP.md` (M13). Verificación: `rg` sin referencias obsoletas.
- [x] 3.4 Ejecutar `cargo fmt --check`, `cargo test`, `cargo clippy --all-targets -- -D warnings`, `scripts/check.sh` y medir `dist`. Verificación: todo verde y < 8 MB.
- [x] 3.5 Validar y archivar el cambio OpenSpec. Verificación: `openspec validate biblioteca-continuar-leyendo --strict` y `openspec archive`.
