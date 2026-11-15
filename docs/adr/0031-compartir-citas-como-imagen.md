# ADR-0031: Compartir citas como imagen

- **Estado:** aceptado
- **Fecha:** 2026-11-14
- **Área:** lector, tipografía, exportación

## Contexto

Los destacados se guardan en `annotations.json` y se pueden exportar como
Markdown, pero no hay una forma visual de compartir una frase. La idea es
seleccionar texto, pulsar **Capturar** y obtener una imagen lista para una
historia de Instagram o una publicación: fondo tenue, cita legible y libro/autor
al pie.

La app ya tiene Literata embebida, el tema actual, las portadas cacheadas y el
crate `image` para PNG/JPEG/WebP/GIF. No hay webview ni motor de render HTML.

## Decisión

1. **Render propio fuera de pantalla.** La imagen se compone en un
   `RgbaImage` con estos formatos: historia 9:16 (1080×1920), publicación 4:5
   (1080×1350), cuadrado 1:1 (1080×1080), retrato 3:4 (1080×1440) y horizontal
   16:9 (1920×1080). La vista previa usa el mismo renderer a escala 0.5; el
   archivo final se genera a escala 1.0. No se depende del tamaño ni del DPI de
   la ventana.
2. **Texto con `ab_glyph`.** `ab_glyph` ya estaba en el árbol por egui; se
   declara como dependencia directa (no agrega un crate nuevo al build). Se usa
   con las caras de Literata y las fuentes de reserva de egui para glifos que
   Literata no cubre. El layout tiene wrap, centrado, autoajuste de tamaño y
   truncado con “…” cuando la cita no entra.
3. **Plantillas y toggles.** Fondo Papel (tema actual), Noche, Portada
   difuminada, Color sólido o Imagen propia. El usuario puede mostrar u ocultar
   libro, autor, capítulo y la marca “Candil”. La configuración se guarda en
   `config.json` (`Config.share`) y se aplica a futuras capturas. En
   **Configuración → Compartir** hay una muestra en vivo con una cita de dominio
   público para probar formato, fondo y toggles sin abrir un libro.
4. **Portada e imagen propia.** La portada sale de `covers/` y se desenfoca con
   una versión reducida + `image::imageops::blur` para que sea barato. La imagen
   propia se elige con `rfd`, se decodifica en un hilo y se limita a 2000 px en
   su lado mayor; se guarda solo la ruta, no se copia el archivo.
5. **Guardado local.** El PNG va a `~/Imágenes` (o Descargas como fallback) con
   un nombre `candil-cita-<libro>-<timestamp>.png`, temporal + `rename` atómico.
   La UI muestra la ruta y un botón “Mostrar archivo”. No se sube nada a la red
   ni se integra una API de Instagram/WhatsApp.
6. **Capturar también destaca.** Si la selección aún no está en
   `annotations.json`, **Capturar** crea el destacado antes de abrir la imagen.
   Si ya existe el mismo rango, lo reutiliza para no duplicar. El panel de
   destacados tiene su propia acción **Compartir**.
7. **Sin bloquear la UI.** La previsualización y el PNG final corren en hilos;
   los cambios de plantilla se debouncean. La ventana de compartir no consulta
   red y no modifica el EPUB.

## Alternativas descartadas

- **Capturar la ventana con `ViewportCommand::Screenshot` y recortar:** depende
  del DPI y del tamaño de ventana; el PNG saldría borroso o con resolución
  variable. El render offscreen es determinista.
- **HTML/SVG + webview o `resvg`:** agrega un motor gráfico y dependencias
  pesadas para algo que se resuelve con texto y un canvas.
- **Copiar la imagen al portapapeles:** requiere otra dependencia y no es
  necesario para guardar/compartir desde el archivo.
- **Varias imágenes para citas largas:** multiplica la UI y el layout; en v1 se
  autoajusta y se recorta con aviso.

## Consecuencias

- Una dependencia directa nueva (`ab_glyph`) que ya estaba en el árbol; sin
  aumento de crates ni de binario por sí misma.
- Las imágenes se generan localmente; la cita y el libro salen del archivo que
  el usuario decide compartir.
- El texto de la imagen usa Literata; glifos ausentes caen a las fuentes de egui.
- Tests: dimensiones, wrap/autoajuste/truncado, fondo con/sin portada, color
  sólido con contraste, nombre seguro y guardado PNG.
- Quedan fuera: compartir directo a una red social, carruseles, fuentes
  personalizadas y edición libre de la cita.
