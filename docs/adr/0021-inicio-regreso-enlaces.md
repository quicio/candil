# ADR-0021: Inicio, regreso a la biblioteca y enlaces externos

- **Estado:** aceptado
- **Fecha:** 2026-11-09
- **Área:** navegación, configuración, seguridad

## Contexto

El lector solo permitía volver a la biblioteca con `Esc`; no había un control
visible. Al abrir Candil siempre aparecía la biblioteca, aunque el usuario
estuviera leyendo un libro. Los enlaces externos del EPUB se descartaban, así que
una referencia a una web no hacía nada.

## Decisión

### Inicio configurable

`Config.startup: StartupMode` (`last_book` por defecto, `library` opcional). En el
arranque:

1. Un path explícito manda.
2. `--resume` o `StartupMode::LastBook` abren `cfg.last_book` si el archivo
   existe.
3. `--screen library` fuerza la biblioteca.
4. Si el libro no existe, se abre la biblioteca.

El ajuste se edita en Configuración → Biblioteca y se guarda en `config.json`.
No hay migración: un archivo viejo recibe `last_book` por default.

**Por qué:** el usuario pidió retomar la lectura al reabrir, pero también quiere
poder dejarlo en modo estantería. **Alternativa descartada:** siempre biblioteca
(comportamiento anterior, pierde contexto). **Alternativa descartada:** siempre
último libro sin opción (no permite el flujo de biblioteca).

### Volver desde el lector

Botón `←` Ghost al inicio de la barra del lector, con tooltip “Biblioteca (Esc)”.
Llama a `go_library`; `reader` corta el render si la pantalla cambió. No se agrega
un atajo nuevo porque `Esc` ya existe.

### Enlaces externos con confirmación

`Book::external_url` acepta solo `http:`, `https:`, `mailto:` y `tel:`. Al hacer
clic se guarda la URL y se muestra un modal con la URL, “Abrir enlace” y
“Cancelar”. Solo la confirmación llama a `open_external` (`open`/`xdg-open`/
`cmd /C start`). `Esc` cierra el modal.

Se rechazan `javascript:`, `data:` y `file:` para no ejecutar contenido ni abrir
rutas locales. El EPUB es contenido no confiable y el clic debe ser explícito.

**Alternativas descartadas:** abrir directo (sin control), permitir cualquier
esquema (riesgo), visor web interno (contradice la arquitectura sin webview).

## Consecuencias

- Cambia el comportamiento por defecto: al reabrir se retoma el último libro. El
  combo permite volver a la biblioteca.
- Un path inválido no rompe el arranque; cae a la biblioteca.
- Los enlaces externos requieren un clic y una confirmación; los esquemas
  inseguros no se ofrecen.
- `--resume` y `--screen library` siguen mandando para desarrollo/capturas.
- No se agregan dependencias ni datos nuevos fuera de `startup` en
  `config.json`.
- Tests cubren el default de configuración y el allowlist de esquemas.
