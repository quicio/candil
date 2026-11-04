---
id: spec-45
title: Citas y marcadores
status: borrador-inicial
version: 0.1.0
updated: 2026-11-03
milestone: M4
depends_on: [spec-30, spec-35, spec-20]
---

# Spec 45 — Citas y marcadores (borrador)

> **Estado: borrador.** Requiere decisión de producto: ¿es un lector o un lector
> con cuaderno?

## Objetivo

Guardar fragmentos de un libro con su contexto y poder exportarlos (markdown).

## Diseño candidato

1. **Selección de texto**: `egui::Label` con `selectable(true)` (hoy está en
   `false`). Recuperar el texto seleccionado vía `LabelOutput`/`TextSelection`.
2. **Guardado**: al seleccionar aparece un botón flotante "Citar"; se guarda
   `{ libro, capítulo, anchor (spec 30), texto, nota?, timestamp }` en
   `library.json` o en `quotes.json` aparte.
3. **Listado**: panel "Citas" del libro abierto y vista global en la biblioteca.
4. **Exportación**: generar `citas-<libro>.md` con formato:

   ```markdown
   > texto de la cita
   — Autor, *Título*, capítulo 7
   ```

## Decisiones abiertas

1. ¿Marcador (posición) y cita (fragmento) son la misma entidad?
2. ¿Exportar a archivo o al portapapeles?
3. ¿Las citas viven en la biblioteca o en un archivo propio?

## Fuera de alcance

- Subrayado multicolor tipo Kindle.
- Sincronización entre dispositivos.
