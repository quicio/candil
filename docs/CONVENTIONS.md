---
id: conventions
status: vigente
version: 1.0.0
updated: 2026-11-03
---

# Convenciones

## Código

- `cargo fmt` es obligatorio; `cargo fmt --check` debe pasar.
- Cero warnings: `cargo build` sin warnings. Un warning es deuda que después
  nadie limpia.
- Sin `unwrap()`/`expect()` sobre datos del usuario (archivos, zip, XML). Sí en
  tests.
- Errores como `Result<T, String>` con mensaje accionable en español
  ("no pude leer X: ..."), mostrados en UI.
- Identificadores: inglés para conceptos genéricos (`Book`, `Block`, `Settings`),
  español para lo del dominio lector (`capitulo`, `renglon`) sólo si ya existe.
  No renombrar por gusto: el churn cuesta más de lo que ordena.
- Comentarios y docs en español; nombres de API en inglés cuando son estándar.

## Tests

- Unitarios en el mismo archivo (`#[cfg(test)] mod tests`), para funciones puras.
- Integración en `tests/`, contra el fixture `tests/fixtures/mini.epub`
  (regenerable con `python3 scripts/make_fixture.py`).
- El fixture cubre a propósito: XHTML válido, HTML roto, imagen relativa, ancla.
- Ningún test depende de libros del disco del usuario.
- Un cambio de parser sin test nuevo es un cambio incompleto.

## Commits

- Imperativo, en español, una intención por commit:
  `feat: importar carpetas a la biblioteca`, `fix: no partir palabras con guion`.
- Sin commits de "wip" ni de "fix typo" mezclados con lógica.
- Rama única (`master`), sin remoto. El historial es la bitácora.

## Flujo spec → código

```
spec (draft) → revisión de Hugo → spec (aprobada) → implementación → spec (implementada)
```

- No se codifica contra una spec inexistente.
- Si aparece una decisión nueva, se escribe un ADR **antes** de implementarla.
- Si la implementación contradice la spec, se corrige la spec en el mismo commit
  y se explica en el mensaje.

## Documentos

Todo documento lleva frontmatter auditable:

```yaml
id: nombre-unico
status: draft | aprobada | vigente | historico
version: 1.2.0        # semver; sube con cambios de contenido
updated: YYYY-MM-DD
```

ADRs: `docs/adr/NNNN-titulo.md` con **Contexto / Decisión / Consecuencias /
Alternativas**. Numeración correlativa, nunca se reescribe un ADR aceptado: se
supersede con uno nuevo (`supersedes: 0003`).

## Presupuesto de recursos

- Dependencia nueva = ADR obligatorio (peso, licencia, mantenimiento, alternativa
  stdlib o código propio).
- Binario `dist` objetivo: < 8 MB. Hoy 5.1 MB.
- Arranque objetivo: < 300 ms en frío.
- Disco: no acumular `target/` de varios perfiles (ver HANDOFF; el disco de la
  máquina trabaja al límite).
