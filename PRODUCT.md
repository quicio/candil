# Product

<!-- impeccable:product-schema 1 -->

## Platform

desktop

<!-- Candil es una app nativa de escritorio (Rust + egui) para macOS, Linux y
Windows. El esquema actual de Impeccable solo contempla web, iOS, Android y
adaptive; se registra `desktop` para no aplicar guías web ni móviles. -->

## Users

Lectores personales y estudiosos que pasan sesiones largas con un libro. Usan
Candil en su computadora, con una biblioteca propia de EPUB, y esperan poder
concentrarse, subrayar, anotar y consultar una palabra o una idea sin salir del
texto. No es una app social ni un servicio en la nube: el usuario es el dueño de
sus libros, su progreso y sus notas.

## Product Purpose

Candil existe para leer EPUB con la menor interfaz posible entre el lector y el
texto. Una columna, scroll continuo, tipografía cuidada, biblioteca local,
anotaciones estables y consultas ocasionales. El éxito es una sesión de lectura
cómoda y sin fricción, no una demostración de funciones.

## Positioning

Un lector nativo, local-first y editorial, construido sobre Rust + egui y un
parser XHTML propio. No hay webview, motor CSS ni dependencia de un servicio:
el EPUB aporta estructura y Candil decide la apariencia. Esa separación permite
tipografía y composición cuidadas con un binario pequeño y arranque instantáneo.

## Operating Context

- Lectura en escritorio, con teclado y mouse/trackpad, en macOS, Linux y Windows.
- Bibliotecas locales indexadas por carpeta; los EPUB no se copian ni se mueven.
- Sesiones largas, a menudo de estudio o relectura, con interrupciones.
- Trabajo offline por defecto; las consultas online son una acción explícita.
- Datos en el directorio de configuración del sistema (`candil/`), con migración
  automática desde la instalación previa `lector/`.

## Capabilities and Constraints

**Implementado:** biblioteca con portadas, búsqueda, orden, filtros, series/colecciones, carpetas y edición de
metadatos/portada del EPUB; lector con temas, tipografía, anclas, progreso por
bloque, enlaces internos e imágenes; destacados y notas con tres colores,
selección entre párrafos, notas al pie con vista previa y regreso, vista global,
filtro de notas y export Markdown;
búsqueda dentro del libro; navegación por porcentaje/capítulo; diccionario,
traducción e IA con historial local; centro de configuración, actualización
asistida y respaldo/restauración local de datos.

**Restricciones técnicas:** Rust + egui/eframe; sin WebView ni motor CSS; sin
dependencias innecesarias; renderizado EPUB existente; atajos, persistencia,
posición de lectura, anotaciones y rendimiento deben preservarse; multiplataforma.
El binario `dist` debe seguir por debajo de 8 MB.

**Privacidad:** sin telemetría; online apagado por defecto (`online_enabled=false`);
la clave de IA vive en el llavero del sistema o en `OPENCODE_API_KEY`; el historial es local y no guarda
el párrafo de contexto.

**Fuera de alcance por ahora:** sincronización entre dispositivos, export
PDF/DOCX, búsqueda dentro de notas, override manual del idioma de origen, motor
CSS, webview y paginado por columnas.

## Brand Commitments

- Nombre: **Candil**. Repo `quicio/candil`.
- Voz: sobria, cálida, literaria, en español rioplatense neutro.
- Lema: **“Leer despacio, sin ruido.”**
- Identidad editorial: papel, noche, ámbar, tipografía Literata; nada de
  dashboards SaaS, gradientes decorativos ni estética genérica de IA.
- La lectura manda; la interfaz se subordina al texto.

## Evidence on Hand

- Documentación de producto y arquitectura en `README.md`, `docs/HANDOFF.md`,
  `docs/ARCHITECTURE.md`, `docs/ROADMAP.md`, `docs/specs/` y `docs/adr/`.
- 14 capturas en `docs/img/` generadas con libros de dominio público.
- Corpus de prueba de 43 EPUB reales; 0 capítulos vacíos y 2279/2279 imágenes.
- 94 tests unitarios (1 live ignorado) + 9 de integración.
- No hay testimonios, clientes ni métricas de uso; no inventarlos.

## Product Principles

1. **El texto es el protagonista.** Todo lo demás se subordina o desaparece.
2. **Silencio funcional.** La interfaz aparece cuando se la necesita y vuelve a
   retirarse; sin decoración gratuita ni indicadores permanentes.
3. **Divulgación progresiva.** Biblioteca, lector, paneles y configuración
   comparten jerarquía; lo avanzado no compite con la lectura.
4. **Local y predecible.** El usuario controla sus libros, su progreso y sus
   notas; nada sale sin una acción explícita.
5. **Cuidado editorial.** La calidad surge de tipografía, proporción, color,
   espaciado y consistencia, no de efectos.

## Accessibility & Inclusion

Objetivo de la consolidación: lectura cómoda durante sesiones prolongadas,
contraste de texto AA, foco visible, navegación completa por teclado y estados
que no dependan solo del color. La app es de escritorio; no se promete soporte
de lector de pantalla nativo en esta etapa.
