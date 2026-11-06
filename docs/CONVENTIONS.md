---
id: conventions
status: vigente
version: 1.2.0
updated: 2026-11-06
---

# Convenciones

## Código

- `cargo fmt` es obligatorio; `cargo fmt --check` debe pasar.
- Cero warnings: `cargo build` y `cargo clippy --all-targets -- -D warnings` sin
  warnings. Un warning es deuda que después nadie limpia.
- Sin `unwrap()`/`expect()` sobre datos del usuario (archivos, zip, XML). Sí en
  tests.
- Tests y capturas no escriben en la configuración real: usar un `HOME` temporal
  o un directorio de datos explícito.
- Errores como `Result<T, String>` con mensaje accionable en español
  ("no pude leer X: ..."), mostrados en UI.
- Identificadores: inglés para conceptos genéricos (`Book`, `Block`, `Settings`),
  español para lo del dominio lector (`capitulo`, `renglon`) sólo si ya existe.
  No renombrar por gusto: el churn cuesta más de lo que ordena.
- Comentarios y docs en español; nombres de API en inglés cuando son estándar.

## Tests

- Unitarios en el mismo archivo (`#[cfg(test)] mod tests`), para funciones puras.
- Integración en `tests/`, contra los fixtures `tests/fixtures/mini.epub` y
  `tests/fixtures/edge.epub` (regenerables con `python3 scripts/make_fixture.py`,
  con fecha fija para que el binario no cambie).
- El fixture `mini` cubre el contrato M1: XHTML válido, HTML roto, imagen
  relativa, ancla.
- El fixture `edge` cubre los bordes de M1.1: espacios inline, entidades,
  `<pre>`, tabla/`<dl>`, SVG raster y `nav` dentro del spine.
- Ningún test depende de libros del disco del usuario.
- La biblioteca se testea con `Library::with_data_dir` (cubre en directorio
  temporal), nunca contra la config real; `--scan` es para medir el corpus.
- Un cambio de parser sin test nuevo es un cambio incompleto.
- Un cambio de escaneo/persistencia de la biblioteca se mide con `--scan` y se
  reporta tiempo, fallos y portadas.

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
- Binario `dist` objetivo: < 8 MB. Hoy 5.516.112 B (medido 2026-11-06).
- Arranque objetivo: < 300 ms en frío.
- Disco: no acumular `target/` de varios perfiles (ver HANDOFF; el disco de la
  máquina trabaja al límite).
