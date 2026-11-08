---
name: Candil
description: Lector de EPUB minimalista y editorial. Leer despacio, sin ruido.
colors:
  paper-bg: "#F4ECD8"
  paper-surface: "#EADFC6"
  paper-ink: "#3B3226"
  paper-muted: "#6E624A"
  paper-accent: "#8A582A"
  paper-error: "#9C4B43"
  night-bg: "#14161A"
  night-surface: "#1C1F25"
  night-ink: "#C9C7C1"
  night-muted: "#84878E"
  night-accent: "#7FA7C8"
  night-error: "#C56E65"
  light-bg: "#FBFAF7"
  light-surface: "#F0EEE8"
  light-ink: "#24221F"
  light-muted: "#6F6C66"
  light-accent: "#3F6E9A"
  light-error: "#A95249"
  amber-bg: "#1E1710"
  amber-surface: "#271E15"
  amber-ink: "#DACBA9"
  amber-muted: "#93856B"
  amber-accent: "#D9A441"
  amber-error: "#C56E65"
  contrast-bg: "#000000"
  contrast-surface: "#101010"
  contrast-ink: "#FFFFFF"
  contrast-muted: "#9A9A9A"
  contrast-accent: "#E8C547"
  contrast-error: "#BF5F55"
typography:
  display:
    fontFamily: "Literata, Georgia, serif"
    fontSize: "24px"
    fontWeight: 600
    lineHeight: 1.2
  reading-body:
    fontFamily: "Literata, Georgia, serif"
    fontSize: "19px"
    fontWeight: 400
    lineHeight: 1.55
  reading-heading:
    fontFamily: "Literata, Georgia, serif"
    fontSize: "1.75em"
    fontWeight: 600
    lineHeight: 1.9
  ui-title:
    fontFamily: "Literata, Georgia, serif"
    fontSize: "16px"
    fontWeight: 600
    lineHeight: 1.3
  ui-body:
    fontFamily: "Literata, Georgia, serif"
    fontSize: "13.5px"
    fontWeight: 400
    lineHeight: 1.4
  ui-label:
    fontFamily: "Literata, Georgia, serif"
    fontSize: "12px"
    fontWeight: 500
    lineHeight: 1.3
  ui-meta:
    fontFamily: "Literata, Georgia, serif"
    fontSize: "11px"
    fontWeight: 400
    lineHeight: 1.3
rounded:
  sm: "4px"
  md: "6px"
  lg: "8px"
spacing:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "24px"
  xxl: "32px"
components:
  button-primary:
    backgroundColor: "{colors.paper-accent}"
    textColor: "{colors.paper-bg}"
    typography: "{typography.ui-body}"
    rounded: "{rounded.md}"
    padding: "10px 16px"
    height: "32px"
  button-secondary:
    backgroundColor: "transparent"
    textColor: "{colors.paper-ink}"
    typography: "{typography.ui-body}"
    rounded: "{rounded.md}"
    padding: "10px 16px"
    height: "32px"
  input:
    backgroundColor: "{colors.paper-bg}"
    textColor: "{colors.paper-ink}"
    typography: "{typography.ui-body}"
    rounded: "{rounded.md}"
    padding: "8px 10px"
    height: "32px"
  card-book:
    backgroundColor: "{colors.paper-bg}"
    textColor: "{colors.paper-ink}"
    rounded: "{rounded.md}"
    width: "160px"
  panel-md:
    backgroundColor: "{colors.paper-surface}"
    textColor: "{colors.paper-ink}"
    rounded: "{rounded.lg}"
    padding: "16px"
    width: "420px"
  chip-folder:
    backgroundColor: "{colors.paper-surface}"
    textColor: "{colors.paper-ink}"
    typography: "{typography.ui-meta}"
    rounded: "{rounded.sm}"
    padding: "4px 8px"
    height: "24px"
---

# Design System: Candil

> **Estado:** sistema objetivo de la consolidación `consolidar-ux-ui`. Los valores
> actuales que difieren se corrigen en ese cambio; este documento es el contrato
> visual y no describe todavía el código en todos sus detalles.

## Overview

**Creative North Star: "La página iluminada"**

Candil es una lámpara sobre una página: la luz existe para que el texto se lea,
no para exhibirse. La interfaz es papel, tinta y ámbar; aparece cuando se la
necesita y se retira cuando empieza la lectura. La densidad es baja, el contraste
es alto y la decoración es casi inexistente. Lo que se ve está ahí porque ayuda a
leer, encontrar o recordar.

La identidad es editorial, no tecnológica. Literata lleva la voz; los controles
se comportan como anotaciones al margen, no como un panel de control. La calidez
del papel y la noche convive con una precisión de taller: medidas constantes,
estados claros y nada que compita con el libro.

**Key Characteristics:**
- Texto primero; chrome subordinado.
- Papel cálido, tinta profunda, ámbar para la acción.
- Literata en lectura y en UI; una sola voz tipográfica.
- Superficies planas, separadas por tono y una línea fina.
- Divulgación progresiva: lo avanzado aparece cuando se pide.
- Movimiento mínimo; el estado cambia, el layout no.

## Colors

La paleta es una biblioteca de luz: papel, noche, claro, ámbar y contraste. Cada
tema conserva los mismos roles y el mismo carácter; solo cambia la luz.

### Primary

- **Ámbar de candil** (`#8A582A` en Papel): acento de acción, enlaces, foco y
  selección. Es cálido, no saturado, y nunca cubre grandes superficies.
- **Azul de noche** (`#7FA7C8`): acento frío para el tema Noche, donde el ámbar
  competiría con el texto.
- **Azul de tinta** (`#3F6E9A`): acento del tema Claro, más sobrio que el azul
  de sistema.
- **Ámbar de lámpara** (`#D9A441`): acento del tema Ámbar, el más luminoso.
- **Amarillo de alto contraste** (`#E8C547`): acento del tema Contraste.

### Secondary

- **Ámbar de destacado** (`#D6A03C`): segundo color de anotación, siempre con
  etiqueta o tooltip.
- **Verde de destacado** (`#4A966E`): tercer color de anotación, siempre con
  etiqueta o tooltip.

### Neutral

- **Papel** (`#F4ECD8`): fondo de lectura del tema Papel.
- **Papel sombra** (`#EADFC6`): superficie de barras y paneles.
- **Tinta** (`#3B3226`): texto principal.
- **Tinta apagada** (`#6E624A`): metadata y etiquetas; cumple AA sobre papel y
  sobre superficie.
- **Noche** (`#14161A`): fondo del tema Noche.
- **Noche superficie** (`#1C1F25`): barras y paneles del tema Noche.
- **Tinta clara** (`#C9C7C1`): texto principal del tema Noche.
- **Tinta apagada nocturna** (`#84878E`): metadata del tema Noche; cumple AA.
- **Error** (`#9C4B43` en Papel, `#C56E65` en Noche/Ámbar, `#A95249` en Claro,
  `#BF5F55` en Contraste): errores y estados destructivos. Nunca es el único
  indicador: siempre va con texto o icono.

### Named Rules

**La luz no decora.** El acento ocupa menos del 10% de cualquier pantalla y nunca
se usa como fondo de grandes áreas.

**El papel manda.** El fondo de lectura no compite con el texto: sin texturas,
gradientes ni imágenes de fondo.

**El error no grita.** El error usa el tono accesible del tema y se explica con
palabras; no se confía solo en el rojo.

## Typography

**Display Font:** Literata (con Georgia, serif de respaldo)
**Body Font:** Literata (con Georgia, serif de respaldo)
**Label/Mono Font:** Literata para UI; monospace del sistema solo para `<pre>`.

**Character:** Una sola voz serif, cálida y legible, con contraste suficiente
entre lectura y chrome. La tipografía no es un adorno: es la interfaz.

### Hierarchy

- **Display** (600, 24 px, 1.2): estados vacíos y títulos de página. No se usa
  dentro del texto del libro.
- **Reading body** (400, 19 px por defecto, 1.55): párrafos, citas y listas; la
  medida objetivo es 60–75 caracteres por línea.
- **Reading heading** (600, 1.75× / 1.45× / 1.25× del cuerpo): títulos de
  capítulo y secciones; el espaciado se deriva del tamaño de fuente.
- **UI title** (600, 16 px, 1.3): títulos de panel y secciones de configuración.
- **UI body** (400, 13.5 px, 1.4): botones, filas y contenido de panel.
- **UI label** (500, 12 px, 1.3): etiquetas de campos y controles.
- **UI meta** (400, 11 px, 1.3): metadata no esencial, capítulos, porcentajes y
  ayudas; nunca para acciones primarias.

### Named Rules

**Una sola voz.** No se agrega una sans para la UI; Literata y sus pesos alcanzan
para distinguir jerarquías.

**La medida manda.** El ancho de lectura se mide en caracteres, no en píxeles
absolutos: si el usuario agranda la letra, la columna se adapta.

**La metadata no compite.** Si un texto es prescindible, va en `ui-meta` y en
tinta apagada; si es una acción, va en `ui-label` o `ui-body` y tiene contraste
de texto.

## Layout

La biblioteca usa dos niveles: una cabecera con identidad, búsqueda y acción
primaria, y una segunda fila solo cuando hay carpetas, orden o filtros que
mostrar. La grilla de portadas es fluida, con tarjetas de 160 px y separación de
20 px; el número de columnas se calcula por ancho disponible.

El lector usa una columna centrada de 660 px por defecto, con 34 px de aire
superior y 140 px al final. La barra superior mide 38–40 px y el progreso 2 px.
Los paneles flotan sobre la lectura sin mover la columna: `panel-sm` 340 px,
`panel-md` 420 px, `panel-lg` 760 px.

Escala de espaciado de UI: 4, 8, 12, 16, 24, 32, 48, 64. El ritmo de lectura se
deriva del tamaño de fuente y del interlineado, no de la escala de UI.

Puntos de control: 980×780 (ventana por defecto) y 720×600 (ventana angosta).
Por debajo de 900 px, las acciones secundarias se agrupan; la búsqueda conserva
prioridad.

## Elevation & Depth

El sistema es plano. La profundidad se comunica con tono de superficie y una
línea fina (`border`), no con sombras. Las ventanas flotantes pueden usar una
sombra muy suave para separarse del texto, pero nunca una sombra dramática ni un
efecto de vidrio. Las tarjetas no tienen sombra en reposo; el hover usa borde y
tono, no elevación.

### Named Rules

**Plano por defecto.** Si una superficie necesita una sombra para entenderse, el
problema es la jerarquía, no la falta de sombra.

## Shapes

Los radios vienen de tres pasos: 4 px para chips e inputs, 6 px para botones y
tarjetas, 8 px para paneles. Las portadas conservan su proporción y no se
redondean en exceso. Los swatches de color son círculos; las chips son píldoras
solo cuando representan filtros o carpetas.

## Components

### Buttons

- **Shape:** radio 6 px, altura 32 px, padding 10×16.
- **Primary:** fondo acento, texto papel; una sola acción primaria por superficie.
- **Secondary:** fondo transparente, borde 1 px, texto tinta.
- **Ghost:** sin fondo ni borde; para acciones terciarias y enlaces de fila.
- **Destructive:** texto/icono error; confirmación obligatoria.
- **Hover / Focus:** hover con acento al 12%; foco con anillo de 2 px; active al
  20%; disabled al 40% y sin hover.

### Inputs / Fields

- **Style:** fondo papel, borde 1 px `border`, radio 6 px, altura 32 px.
- **Focus:** anillo de 2 px en acento, sin cambiar el layout.
- **Error:** borde error + mensaje de texto; nunca solo color.
- **Disabled:** fondo superficie, texto apagado, sin foco.

### Cards / Containers

- **Book card:** portada 160 px de ancho, título `ui-body`, autor `ui-meta`,
  progreso como línea de 2 px y porcentaje; hover con borde acento.
- **Panel:** fondo superficie, borde 1 px, radio 8 px, header de 44 px, padding
  16 px.
- **Row:** altura mínima 32 px, hover con tinte acento al 8%, foco con anillo.

### Chips

- **Folder chip:** fondo superficie, texto `ui-meta`, radio 4 px, altura 24 px;
  la × tiene tooltip y confirmación al quitar la carpeta.
- **Scope chip / tab:** seleccionado con fondo acento al 15%, borde acento y
  peso 600; no solo color.

### Navigation

- **Library:** identidad + búsqueda + acción primaria; overflow para refrescar,
  configuración y carpetas.
- **Reader:** índice, tipografía, notas y búsqueda a la izquierda; título y autor
  truncados; capítulo y porcentaje a la derecha.
- **States:** hover con tinte, foco con anillo, activo con subrayado o peso; los
  iconos siempre tienen tooltip y, cuando existe, atajo.

### App icon

- **Composición:** libro abierto en papel sobre fondo Noche, con una llama ámbar
  que nace del lomo. Sin texto; legible a 16 px.
- **Archivos:** `assets/candil-icon.svg` (fuente), `assets/candil-icon.png`
  (1024), `assets/candil-icon-256.png` y `assets/candil.icns` (macOS).
- **Uso:** icono de ventana (`with_icon`) y bundle `Candil.app` en macOS.

### Color swatch

- **Shape:** círculo de 14 px con anillo de 2 px cuando está seleccionado.
- **States:** normal, hover y seleccionado; siempre acompañado de nombre o
  tooltip (`acento`, `ámbar`, `verde`).
- **Prohibido:** usar el glifo `●` como único indicador; puede caer a tofu.

## Do's and Don'ts

### Do:

- **Do** usar los tokens de `theme.rs` y los helpers de `ui.rs` para cualquier
  control nuevo.
- **Do** mantener el texto del libro como el elemento de mayor peso visual.
- **Do** verificar contraste AA y foco visible en los cinco temas.
- **Do** usar una sola acción primaria por pantalla y agrupar el resto.
- **Do** conservar los atajos existentes y documentar los nuevos en la ayuda.
- **Do** acompañar el color con texto, icono o forma cuando comunique estado.
- **Do** dar a los controles un objetivo mínimo de 32×32 y un anillo de foco de 2 px.
- **Do** mantener `animation_time = 0`: el estado cambia sin movimiento.

### Don't:

- **Don't** introducir azules, grises o sombras por defecto de egui.
- **Don't** usar gradientes, glassmorphism, animaciones decorativas ni iconos
  sin etiqueta.
- **Don't** cubrir la columna de lectura con paneles que no se puedan cerrar.
- **Don't** agregar una tipografía de UI distinta de Literata.
- **Don't** cambiar el parser, el modelo de datos o la persistencia para resolver
  un problema visual.
- **Don't** usar el color como único indicador de error, selección o estado.
- **Don't** inventar funciones nuevas dentro de un cambio de consolidación.
