## Why

Candil ya tiene una experiencia de lectura sólida, pero la interfaz que la rodea creció por acumulación: la barra de la biblioteca compite con el contenido, los paneles no comparten medidas ni jerarquía, el chrome por defecto de egui (azules, sliders, ventanas) se filtra en un producto editorial, y varios textos secundarios no alcanzan contraste AA. La lectura sigue siendo buena; lo que falla es la coherencia del resto.

Este cambio consolida la UX/UI antes de agregar funciones nuevas. El objetivo no es impresionar: es que la interfaz deje de ser una preocupación y que Candil se sienta cuidado, deliberado y silencioso en todas sus superficies.

## What Changes

- Sistema de diseño explícito: tokens de color, tipografía, espaciado, radios, elevación y estados; aplicados a `theme.rs` y a los widgets de `app.rs`.
- Corrección de contraste: textos secundarios y errores alcanzan 4.5:1 en los cinco temas; la selección mantiene contraste en temas oscuros.
- Componentes compartidos: botón primario/secundario/ghost, input, chip, tarjeta de libro, encabezado de panel, fila de resultado y swatch de color.
- Biblioteca: barra superior simplificada y sin solapamientos a 980 px y 720 px; búsqueda protagonista; acciones secundarias en un menú; estados vacío/error/faltante con acciones claras.
- Lector: barra superior más silenciosa; título con más espacio; navegación de capítulo y porcentaje agrupados; foco visible y selección de texto consistente.
- Paneles: un único frame, un ancho por rol (compacto 340 / medio 420 / amplio 760), padding y tipografía comunes; sin flechas de colapso en ventanas no colapsables.
- Anotaciones y consultas: swatches de color reales (no glifos que caen a tofu), estados hover/foco, acciones destructivas con confirmación o undo, y retorno al texto sin fricción.
- Accesibilidad: foco de 2 px, objetivos mínimos de 32×32, navegación por teclado, estados que no dependen solo del color y textos de ayuda contextual.
- Capturas y documentación actualizadas; `DESIGN.md` registra el sistema resultante.

**No es un rediseño desde cero.** Se preservan nombre, voz, modo de lectura, atajos, parser, persistencia, anotaciones y arquitectura.

## Capabilities

### New Capabilities

- `design-system`: tokens normativos de color, tipografía, espaciado, radios, elevación y componentes; reglas de contraste y estados interactivos.
- `library-experience`: jerarquía, barra superior, grilla de portadas, búsqueda/orden/filtros y estados vacío, error y faltante.
- `reading-workspace`: superficie de lectura, barra del lector, selección, destacados, notas, búsqueda, índice, configuración y consultas.
- `interaction-accessibility`: teclado, foco, hover/active/disabled, feedback, confirmaciones y contraste.

### Modified Capabilities

- Ninguna: las specs de `docs/specs/` describen milestones funcionales ya cerrados; este cambio agrega capabilities de experiencia sin alterar el comportamiento funcional.

## Scope

**Incluido:** biblioteca, lector, paneles, configuración, anotaciones, consultas, interacciones globales, estados de error/vacío, tema y tipografía de la UI, capturas y documentación de diseño.

**No incluido:** nuevas funciones de producto, cambios al parser EPUB, modelo de datos, sincronización, export distinto a Markdown, webview, motor CSS, paginado por columnas, animaciones decorativas, dependencias nuevas.

## Acceptance Criteria

- [ ] Todos los pares texto/fondo de la UI alcanzan 4.5:1; los textos grandes y elementos gráficos alcanzan 3:1.
- [ ] La barra de la biblioteca no se solapa ni recorta controles a 980×780 ni a 720×600.
- [ ] Biblioteca, lector y paneles usan las mismas escalas de espaciado, radios y tipografía de UI.
- [ ] Los paneles comparten frame, header y padding; los anchos se limitan a los tres roles definidos.
- [ ] Los swatches de color se distinguen y comunican selección sin depender del glifo `●`.
- [ ] Todo control interactivo tiene hover, foco visible y estado disabled coherentes.
- [ ] Los estados vacío, error y EPUB faltante ofrecen una acción siguiente concreta.
- [ ] Los atajos existentes siguen funcionando y hay una ayuda de atajos accesible desde la app.
- [ ] `cargo test`, `cargo fmt --check` y `cargo clippy --all-targets -- -D warnings` pasan.
- [ ] Las capturas de `docs/img/` se regeneran en sandbox con libros de dominio público.
- [ ] `DESIGN.md` refleja los tokens y componentes implementados.

## Risks and Dependencies

- **Riesgo:** tocar `app.rs` (≈5.000 líneas) puede introducir regresiones en lectura, scroll o anclas. Mitigación: cambios por componente, capturas comparativas y tests existentes.
- **Riesgo:** egui 0.32 no expone todos los estados como CSS; algunos widgets requieren `Frame`/`Painter` propios. Mitigación: helpers compartidos y nada de dependencias nuevas.
- **Riesgo:** el contraste AA puede apagar la paleta. Mitigación: ajustar solo `dim`, `error` y `accent` de Papel; preservar la calidez.
- **Dependencia:** egui/eframe 0.32 fijado por ADR-0003; no se actualiza en este cambio.
- **Dependencia:** capturas con Project Gutenberg y `scripts/captures.sh`; no se usan datos reales.
