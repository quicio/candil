# Roadmap de Candil Mobile

## 0.1 — Base funcional (actual)

- [x] Proyecto Flutter Android/iOS.
- [x] Toolchain Android instalado y APK debug/release compilado.
- [x] App verificada en emulador Android 36: importar, abrir y leer un EPUB.
- [x] Núcleo Rust con parser EPUB, bloques y búsqueda.
- [x] FFI C/JSON probado desde Dart en Linux.
- [x] Biblioteca, importación, portadas, búsqueda y orden.
- [x] Lector con TOC, imágenes, progreso, temas, tipografía y modo enfoque.
- [x] Búsqueda dentro del libro y salto al capítulo/bloque.
- [x] Anotaciones por párrafo y notas.
- [x] Respaldo compatible e importación con reasignación de rutas.

## 0.2 — Paridad de búsqueda y lectura

- [ ] Búsqueda avanzada de toda la biblioteca con progreso y cancelación.
- [ ] Índices en paralelo (2–4 trabajadores) y caché acotada.
- [ ] Selección de texto real por rangos, no solo párrafo completo.
- [ ] Anotaciones entre párrafos (`end_block` + `parts`) como escritorio.
- [ ] Exportar anotaciones a Markdown.
- [ ] Compartir una cita como PNG local.

## 0.3 — Estudio e IA

- [ ] Claves en llavero del sistema.
- [ ] Proveedores de IA configurables, sin claves en JSON.
- [ ] Consultas online apagadas por defecto y aviso explícito.
- [ ] Sesiones de estudio en `studies.json`.
- [ ] Diccionario y traducción.

## 0.4 — Móvil y distribución

- [ ] Compartir/abrir EPUB desde otras apps.
- [ ] Importar respaldo desde el selector de archivos y exportarlo por compartir.
- [ ] Firma Android, TestFlight y App Store.
- [ ] Accesibilidad: escalado de texto del sistema, lectores de pantalla y foco.
- [ ] Pruebas en dispositivos de referencia.

## Futuro

- Caché de EPUB remotos (rclone/Drive) y lectura offline.
- Sincronización opcional entre escritorio y móvil.
- Widgets de progreso y “continuar leyendo”.
