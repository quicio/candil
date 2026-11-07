## 1. Base del sistema de diseño

- [x] 1.1 Extender `Theme` en `src/theme.rs` con roles semánticos (`panel` como superficie, `dim` como texto apagado, `selection`, `error`, `border`) y aplicar los valores accesibles de `design.md` a los cinco temas. Verificación: `cargo test --lib theme` y revisión de contraste.
- [x] 1.2 Crear `UiTokens` con escala de espaciado (4/8/12/16/24/32/48/64), radios (4/6/8) y tamaños de UI (11/12/13.5/16/24). Verificación: los valores viven en un solo lugar y `cargo clippy` pasa.
- [x] 1.3 Crear `src/ui.rs` con helpers compartidos: `button` (primary/secondary/ghost/destructive), `icon_button`, `input`, `panel_frame`, `section_header`, `chip`, `color_swatch`, `theme_swatch`, `meta` y `label`. Verificación: el módulo compila, clippy pasa y las capturas de tipografía y notas muestran los componentes.
- [x] 1.4 Exportar `ui` desde `src/lib.rs` y migrar una primera pantalla de prueba (tipografía) a los helpers. Verificación: la pantalla se ve igual o mejor en `--shot`.
- [x] 1.5 Reemplazar `Color32::from_rgb(190, 92, 82)` y demás colores literales por `theme.error` / tokens semánticos. Verificación: `rg "from_rgb\\(190, 92, 82\\)"` no devuelve resultados en `src/`.
- [x] 1.6 Ajustar `Visuals` de egui: `widgets.inactive/hovered/active/open`, `selection` y foco de 2 px en acento; eliminar el azul por defecto en sliders, checkboxes y botones. Verificación: capturas de `settings` en Papel y Noche sin azul genérico.
- [x] 1.7 Agregar test unitario de contraste WCAG en `src/theme.rs` para `fg/bg`, `muted/bg`, `muted/surface`, `error/bg`, `error/surface` y `fg/selection` en los cinco temas. Verificación: el test falla si algún par baja de 4.5:1.
- [x] 1.8 Definir helpers de tipografía de UI y reemplazar tamaños literales en `app.rs` por roles (`META`, `LABEL`, `BODY_SMALL`, `BODY`, `TITLE`, `HEADLINE`, `DISPLAY`, `ICON`). Verificación: `rg "size\\(1[0-9](\\.5)?\\)" src/app.rs` queda limitado a lectura/títulos.
- [x] 1.9 Reemplazar el glifo `●` por un swatch pintado (círculo relleno + anillo de selección) en la barra de selección, la lista de notas y el editor de nota. Verificación: captura de notas en Papel y Noche muestra tres círculos distinguibles.

## 2. Biblioteca

- [x] 2.1 Reestructurar la cabecera de biblioteca en dos niveles: identidad + búsqueda + acción primaria + overflow; segunda fila para carpetas, orden y filtro. Verificación: `--shot --screen library` a 980×780 sin solapamientos.
- [x] 2.2 Implementar el menú overflow con `Refrescar`, `Configuración` y `Carpetas`; conservar los atajos existentes. Verificación: teclado y mouse abren/cierran el menú y las acciones funcionan.
- [x] 2.3 Hacer que la búsqueda mantenga el foco y el ancho flexible; eliminar el solapamiento con `Orden` en 980 px y 720 px. Verificación: capturas en ambos anchos.
- [x] 2.4 Compactar orden/filtro en controles con valor visible y etiquetas solo cuando haya espacio. Verificación: los cuatro filtros y cuatro órdenes siguen accesibles.
- [x] 2.5 Extraer `library_card` a un componente con cover, título, autor, progreso y estados hover/focus. Verificación: la tarjeta abre con click y con Enter/Espacio cuando tiene foco.
- [x] 2.6 Mejorar el estado vacío: una acción primaria `Añadir carpeta`, una secundaria `Abrir un EPUB` y la pista de arrastre. Verificación: captura de `library-empty` y recorrido por teclado.
- [x] 2.7 Implementar estado de error de EPUB con detalle y acciones `Ver detalle` / `Quitar de la biblioteca`; confirmar la eliminación y aclarar que el archivo no se borra. Verificación: `library-error` muestra el detalle y la acción.
- [x] 2.8 Implementar estado `missing` con acción `Ubicar archivo…` o `Quitar de la biblioteca`, sin borrar el EPUB. Verificación: fixture con archivo faltante y captura.
- [x] 2.9 Añadir confirmación al quitar una carpeta y mantener los chips solo cuando existan carpetas. Verificación: quitar carpeta no borra libros ni archivos.
- [x] 2.10 Añadir feedback de escaneo (estado busy + resumen) sin bloquear la ventana. Verificación: `--scan` y `Refrescar` muestran el resumen y la UI responde.

## 3. Lector y paneles

- [x] 3.1 Reorganizar la barra del lector: índice, tipografía, notas y búsqueda a la izquierda; título/autor truncados; capítulo y porcentaje agrupados a la derecha; configuración en overflow/engranaje. Verificación: `--shot` a 980×780 y 720×600 sin recortes.
- [x] 3.2 Mantener la barra del lector en 40 px con padding horizontal de 16, coherente con la biblioteca, y el progreso de 2 px. Verificación: comparación visual y `cargo test`.
- [x] 3.3 Migrar la barra de selección a los helpers: swatches reales, botones `Destacar`/`Nota`/`Consultar` con foco visible y cierre con Esc. Verificación: seleccionar texto y recorrer con teclado.
- [x] 3.4 Unificar el panel de anotaciones: frame, header, ancho `panel-md`, filas con cita dominante, metadata secundaria, `Ir al texto` y swatch seleccionable. Verificación: captura de `reader-notes` y salto a la cita.
- [x] 3.5 Añadir confirmación o undo al borrar un destacado. Verificación: borrar una nota muestra confirmación/undo y persiste solo tras confirmar.
- [x] 3.6 Unificar el editor de nota con el frame y los swatches; etiquetas y foco consistentes. Verificación: crear y editar una nota en Papel y Noche.
- [x] 3.7 Unificar el panel de búsqueda: filas con hover/foco, contexto, capítulo y contador; conservar Enter, n/N y Esc. Verificación: buscar y recorrer resultados con teclado.
- [x] 3.8 Unificar `Ir a` con botones explícitos para porcentaje y capítulo, validación y foco. Verificación: saltar por porcentaje y por capítulo.
- [x] 3.9 Unificar el índice (`T`) con el frame y ancho de panel; marcar la entrada activa sin depender solo del color. Verificación: captura y salto a un ancla.
- [x] 3.10 Unificar Configuración con `panel-lg`, navegación por tabs, section headers y controles agrupados; eliminar la flecha de colapso. Verificación: captura de cada tab y recorrido por teclado.
- [x] 3.11 Unificar Consulta con `panel-md`, estados offline/busy/error y acciones de historial (`Ir al texto`, copiar, borrar) consistentes. Verificación: capturas de diccionario, historial y offline.

## 4. Interacción y accesibilidad

- [ ] 4.1 Añadir una ayuda de atajos accesible desde la app (panel o ventana) con lectura, navegación, anotaciones y paneles. Verificación: abrir con el atajo y con el menú; Esc cierra.
- [ ] 4.2 Verificar y corregir el orden de tabulación en biblioteca, lector y paneles; foco visible en todos los controles. Verificación: recorrido completo solo con teclado.
- [ ] 4.3 Garantizar objetivos mínimos de 32×32 y separación entre botones de barra. Verificación: inspección de capturas y `UiTokens`.
- [ ] 4.4 Añadir señales no cromáticas a tabs, chips, errores y swatches (peso, borde, icono o etiqueta). Verificación: capturas en escala de grises y con daltonismo simulado.
- [ ] 4.5 Revisar acciones destructivas: borrar destacado, quitar libro, quitar carpeta y limpiar historial. Verificación: cada una pide confirmación o permite undo.
- [ ] 4.6 Confirmar que no hay animaciones decorativas ni movimiento de layout; documentar la limitación de reduced motion en egui. Verificación: `rg "animate|request_repaint"` justificado.
- [ ] 4.7 Añadir tooltips a controles solo-icono y a acciones no obvias, con el atajo cuando exista. Verificación: hover en `☰`, `Aa`, `‹`, `›`, engranaje y swatches.
- [ ] 4.8 Normalizar feedback de guardado, escaneo y consulta offline (mensaje + duración + no bloqueo). Verificación: probar guardar nota, refrescar y consultar offline.

## 5. Validación, capturas y documentación

- [ ] 5.1 Ejecutar `cargo test`, `cargo fmt --check` y `cargo clippy --all-targets -- -D warnings`. Verificación: todo verde.
- [ ] 5.2 Ejecutar `scripts/check.sh` y `--inspect tests/fixtures/edge.epub` si se tocó algo del parser. Verificación: salida `ok`.
- [ ] 5.3 Capturar biblioteca, lector, paneles y estados en 980×780 y 720×600 con el sandbox de `scripts/captures.sh`. Verificación: revisar solapamientos, foco y contraste.
- [ ] 5.4 Regenerar `docs/img/` con libros de dominio público y sin datos reales. Verificación: `docs/img` actualizado y sandbox aislado.
- [ ] 5.5 Actualizar `DESIGN.md` con los tokens y componentes finales. Verificación: frontmatter y secciones canónicas completas.
- [ ] 5.6 Actualizar `README.md`, `docs/HANDOFF.md`, `docs/ROADMAP.md` y enlaces a specs/ADRs. Verificación: sin referencias obsoletas y conteos correctos.
- [ ] 5.7 Ejecutar `openspec validate consolidar-ux-ui --strict` y revisar que cada tarea completada tenga evidencia. Verificación: validación limpia antes de archivar.
