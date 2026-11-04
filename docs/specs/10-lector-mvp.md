---
id: spec-10
title: Lector — contrato del MVP
status: implementada
version: 1.0.0
updated: 2026-11-03
milestone: M1
depends_on: [adr-0001, adr-0002, adr-0004, adr-0005]
---

# Spec 10 — Lector (M1)

Spec retroactiva: describe lo que ya está implementado y verificado. Su función es
congelar el contrato para que M2/M3 no lo rompan sin darse cuenta.

## Comportamiento garantizado

1. **Abrir**: por argumento (`lector libro.epub`), por arrastre, por `⌘O`, o
   resumiendo el último libro guardado. Si el archivo no existe o no es un zip
   válido, se informa en un toast y la app sigue viva.
2. **Metadatos**: título, autor e idioma desde `metadata` del OPF. Sin título
   declarado, se usa el nombre del archivo.
3. **Estructura**: spine completo en orden; índice desde `nav.xhtml` (EPUB3) con
   fallback a `toc.ncx` (EPUB2) y de ahí a "Capítulo N" con el primer encabezado
   del capítulo como título.
4. **Contenido**: cada capítulo se convierte en `Vec<Block>`; nunca queda vacío si
   el XHTML tiene texto (parser tolerante, ADR-0002).
5. **Lectura**: columna centrada, scroll continuo, párrafos justificados, títulos
   jerárquicos, citas con barra, listas con viñeta/numeración, reglas, imágenes
   escaladas al ancho de columna.
6. **Temas**: 5 presets; tamaño, interlineado, ancho de columna y espaciado son
   ajustables en vivo.
7. **Progreso**: capítulo + offset guardados por libro en `config.json`, con
   debounce de 700 ms y al salir.
8. **Teclado**: `Space/PgDn/↓` avanzan, `PgUp/↑` retroceden, `←/→` cambian de
   capítulo, `Home` va al inicio, `+/-` cambian tamaño, `T` índice, `S`
   tipografía, `Esc` cierra, `⌘O` abre.

## Verificación (reproducible)

```bash
cargo test                                        # 14 unit + 6 integración
cargo run --profile fast -- --inspect <libro>     # metadatos, spine, bloques, imágenes
cargo run --profile fast -- --shot /tmp/p.png --chapter 13 <libro>
```

Estado medido el 2026-11-03: 44 EPUB abiertos sin error, 2168/2168 imágenes
resueltas, 0 capítulos vacíos salvo portadas/mapas (solo imagen).

## Fuera de alcance (a propósito)

- Interpretar el CSS del libro.
- Paginado por columnas o por páginas fijas.
- Búsqueda, notas, marcadores, sincronización.
- Índice por ancla (M3).
- Bibliotecas de varios libros (M2).

## Deuda conocida

| deuda | impacto | dónde se paga |
|---|---|---|
| índice por archivo, no por ancla | libros con todo en un XHTML tienen 1 entrada | M3 / spec 30 |
| offset en píxeles | cambiar fuente descoloca la posición | spec 35 |
| sin detección de capítulos vacíos por imagen | capítulos `[]` confunden al índice | aceptado |
| `Result<T, String>` | sin tipos de error | si crece, enum + thiserror |
