## Context

Ver `proposal.md` y las specs. `Config` ya guarda `last_book`; `wrap_app` abre un
libro solo con `--resume` o un path explícito. `Esc` llama a `go_library`. El
lector resuelve enlaces internos con `Book::link_target` y descarta los externos.
La app ya tiene `open_external` para abrir URLs del release.

## Goals / Non-Goals

**Goals:**

- Volver a la biblioteca con un control visible y discreto.
- Elegir el destino inicial; último libro por defecto.
- Confirmar enlaces externos y limitar esquemas.
- Mantener `Esc`, `--resume`, `--screen library` y capturas.

**Non-Goals:**

- Webview, visor interno, historial de enlaces, atajos configurables, abrir un
  libro concreto distinto del último.

## Decisions

### 1. `StartupMode` en `Config`

`StartupMode { LastBook, Library }`, `#[serde(rename_all="snake_case")]`,
`Default = LastBook`. `Config.startup` usa `serde(default)`. En `wrap_app`, la
precedencia es:

1. Path explícito.
2. `--resume` o `StartupMode::LastBook` → `cfg.last_book` si el archivo existe.
3. `--screen library` fuerza biblioteca.
4. Si no hay libro válido, biblioteca.

El ajuste se edita en Configuración → Biblioteca con un combo y se guarda con el
resto de `config.json`.

**Por qué default último libro:** es lo que el usuario pidió y evita buscar el
libro al reabrir. Quien prefiera la estantería lo cambia una vez.

**Alternativa descartada:** siempre biblioteca; es el comportamiento anterior,
pero ignora el contexto de lectura. **Alternativa descartada:** siempre último
libro sin opción; no permite el flujo de estantería.

### 2. Volver desde el lector

Se agrega un botón `←` (Ghost) al inicio de la barra del lector, antes del índice.
El clic llama a `go_library`; `reader` retorna si la pantalla cambió. El tooltip
dice “Biblioteca (Esc)”. No se agrega un atajo nuevo: `Esc` ya existe.

### 3. Enlaces externos con confirmación

`Book::external_url(href)` acepta `http:`, `https:`, `mailto:` y `tel:`; rechaza
`javascript:`, `data:`, `file:` y esquemas desconocidos. Al hacer clic en un
enlace externo se guarda la URL en `Lector.external_link` y se abre un modal con
la URL, “Abrir enlace” y “Cancelar”. Solo “Abrir enlace” llama a `open_external`
(`open`/`xdg-open`/`cmd /C start`). `Esc` cierra el modal.

**Por qué confirmar:** el EPUB es contenido no confiable; abrir el navegador
automáticamente al pasar el mouse o al hacer clic por accidente es indeseable.
**Alternativa descartada:** abrir directo; menos pasos, pero sin control.
**Alternativa descartada:** permitir cualquier esquema; `javascript:`/`data:`
pueden ejecutar contenido.

### 4. Capturas y compatibilidad

`--screen library` sigue forzando la estantería para las capturas. Se agregan
`--external-link-demo` y una captura del modal. `--resume` sigue teniendo
prioridad sobre el ajuste.

## Risks / Trade-offs

- **Cambio de comportamiento por defecto** → los usuarios existentes abrirán el
  último libro; el ajuste permite volver a biblioteca.
- **Último libro faltante** → se cae a biblioteca; no se borra la entrada.
- **Enlaces con mayúsculas/espacios** → se normaliza el esquema en minúsculas.
- **Modal por cada enlace** → más seguro; el clic sigue siendo explícito.
- **Sin navegador disponible** → `open_external` falla silenciosamente; no rompe
  la app.

## Migration Plan

Sin migración de datos: `startup` usa default `last_book` al cargar un
`config.json` viejo. Rollback: borrar la rama de inicio y el botón; el modal de
enlaces se puede quitar sin tocar datos.

## Open Questions

Ninguna que cambie el alcance.
