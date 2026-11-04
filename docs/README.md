---
id: docs-index
status: vigente
version: 1.0.0
updated: 2026-11-03
---

# Documentación

Orden de lectura para un agente nuevo:

| documento | para qué |
|---|---|
| [`../AGENTS.md`](../AGENTS.md) | comandos y reglas duras (leer primero) |
| [`HANDOFF.md`](HANDOFF.md) | estado real, invariantes, cómo verificar sin ojos |
| [`ROADMAP.md`](ROADMAP.md) | milestones y criterios de salida |
| [`ARCHITECTURE.md`](ARCHITECTURE.md) | módulos, modelo `Block`, puntos de extensión |
| [`CONVENTIONS.md`](CONVENTIONS.md) | estilo, tests, commits, flujo spec → código |

## Specs

| id | título | estado |
|---|---|---|
| [10](specs/10-lector-mvp.md) | Lector — contrato del MVP | implementada |
| [20](specs/20-biblioteca.md) | Biblioteca — índice, estantería y apertura | **draft (siguiente a revisar)** |
| [21](specs/21-metadatos-y-portadas.md) | Metadatos y portadas | draft |
| [30](specs/30-indice-por-ancla.md) | Índice por ancla | draft |
| [35](specs/35-preferencias-por-libro.md) | Preferencias por libro y progreso estable | draft |
| [40](specs/40-busqueda.md) | Búsqueda dentro del libro | borrador inicial |
| [45](specs/45-citas-y-marcadores.md) | Citas y marcadores | borrador inicial |

## ADR

| id | decisión | estado |
|---|---|---|
| [0001](adr/0001-ui-egui-nativo.md) | UI nativa con egui, no webview | aceptada |
| [0002](adr/0002-parser-xhtml-doble.md) | Dos parsers de HTML: estricto y tolerante | aceptada |
| [0003](adr/0003-pin-egui-0.32.md) | Fijar egui/eframe 0.32 | aceptada |
| [0004](adr/0004-tipografia-literata.md) | Literata embebida, instanciada | aceptada |
| [0005](adr/0005-persistencia-json.md) | Configuración y progreso en JSON | aceptada |
| [0006](adr/0006-perfiles-compilacion.md) | Tres perfiles de compilación | aceptada |
| [0007](adr/0007-biblioteca-indice-json.md) | La biblioteca indexa carpetas, no copia | propuesta |
| [0008](adr/0008-portadas-cache.md) | Portadas: extraer, reducir, cachear | propuesta |

## Flujo

```
spec (draft) ──revisión──► aprobada ──implementación──► implementada
                   │
                   └── decisión nueva ──► ADR (antes de codear)
```

Una spec sin aprobar no se implementa. Un ADR aceptado no se reescribe: se
supersede con otro (`supersedes: NNNN`).
