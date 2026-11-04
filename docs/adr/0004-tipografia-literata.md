---
id: adr-0004
status: aceptada
version: 1.0.0
updated: 2026-11-03
---

# ADR-0004 — Literata embebida, instanciada desde la variable

## Contexto

Un lector de libros vive o muere por la fuente. La default de egui (Ubuntu-Light)
es de UI, no de lectura. Además `egui` **no sintetiza bold**: si solo hay un peso,
títulos y negritas se ven iguales que el texto normal.

Opciones: cargar fuentes del sistema (NewYork/Georgia en macOS), o embeber una
fuente libre.

## Decisión

Embeber **Literata** (SIL OFL 1.1), instanciada localmente a partir de la fuente
variable de Google Fonts con `fonttools`:

| archivo | peso | uso |
|---|---|---|
| `Literata-Regular.ttf` | 400 | cuerpo |
| `Literata-SemiBold.ttf` | 600 | títulos (fallback de la familia base) |
| `Literata-Bold.ttf` | 700 | negritas y encabezados |
| `Literata-Italic.ttf` | 400 | citas y énfasis |
| `Literata-BoldItalic.ttf` | 700 | combinación |

`opsz` fijado en 12 (tamaño de lectura). Total: ~1.3 MB en el binario.

## Consecuencias

- Tipografía idéntica en cualquier SO, licencia redistribuible (OFL).
- Cuatro familias registradas en egui (`serif`, `serif-bold`, `serif-italic`,
  `serif-bold-italic`) porque egui no tiene pesos: `fonts::family(bold, italic)`
  resuelve el nombre.
- La UI usa la misma familia (Proportional con Literata al frente), con la sans
  de egui como fallback de glifos faltantes.
- Sumar una fuente nueva = 4 archivos por familia + actualizar `fonts.rs`.

## Alternativas descartadas

- **Fuentes del sistema** (`/System/Library/Fonts/NewYork.ttf`): cero peso en el
  binario, pero no redistribuible, cambia según SO y en macOS los `.ttc` no los
  carga `ab_glyph`.
- **Una sola fuente variable**: egui usa la instancia por defecto; negritas y
  títulos se aplanan (verificado).
- **Fuentes de Google por red en runtime**: rompe el arranque offline.

## Reproducir los archivos

```bash
curl -O 'https://raw.githubusercontent.com/google/fonts/main/ofl/literata/Literata%5Bopsz,wght%5D.ttf'
python -c "from fontTools.ttLib import TTFont; from fontTools.varLib import instancer; \
f=TTFont('Literata[opsz,wght].ttf'); instancer.instantiateVariableFont(f,{'wght':400,'opsz':12}).save('Literata-Regular.ttf')"
```
