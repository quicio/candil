---
id: adr-0003
status: aceptada
version: 1.0.0
updated: 2026-11-03
---

# ADR-0003 — Fijar egui/eframe 0.32

## Contexto

`cargo search` ofrece `eframe 0.37`. Al intentar compilar contra 0.37 aparecieron
cambios incompatibles:

- `App::update(ctx, frame)` ya no existe: el trait pide `fn ui(&mut self, ui, frame)`.
- `egui::TopBottomPanel` no existe (los paneles se reorganizaron en
  `containers/panel.rs` con otra API).
- `Context::style()` / `set_style()` desaparecieron a favor de `style_of(theme)` /
  `set_style_of(theme, ...)`.
- `Event::Screenshot` y el manejo de `ScrollAreaOutput.state` también cambiaron.

Es una reescritura, no una evolución incremental. El proyecto necesitaba un lector
funcionando, no una migración.

## Decisión

Fijar `eframe = "0.32"` (`default-features = false`, features `glow`,
`default_fonts`, `wayland`, `x11`). El código compila sin warnings contra 0.32 y
anda en macOS 26.5 / aarch64.

## Consecuencias

- Estabilidad inmediata: la API usada (`TopBottomPanel`, `Frame::new`,
  `Margin::symmetric`, `LayoutJob.justify`, `Event::Screenshot`) existe tal cual.
- Deuda: si alguna vez se quiere 0.37+, es una migración planificada, con su
  propio milestone, no un `cargo update`.
- Bloquea dependencias que exijan egui 0.37 (ninguna hoy).

## Alternativas descartadas

- **Migrar a 0.37 ahora**: costo alto (reescribir paneles, estilo por tema,
  captura de pantalla, scroll) sin beneficio para el usuario.
- **Volver a 0.27/0.28**: más viejo, sin ganancia frente a 0.32.

## Cómo revisar esta decisión

Antes de subir de versión: leer el changelog de egui del tramo 0.32→X, hacer un
branch de spike con un solo archivo migrado (la barra superior), medir esfuerzo y
recién entonces escribir el ADR que supersede a este.
