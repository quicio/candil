# AGENTS.md

Punto de entrada para cualquier agente que tome este repo. Leer en este orden:

1. **`docs/HANDOFF.md`** — estado real, reglas de trabajo, cómo verificar.
2. **`docs/ROADMAP.md`** — qué sigue y cuándo se considera terminado.
3. **`docs/ARCHITECTURE.md`** — módulos, modelo de bloques, puntos de extensión.
4. **`docs/CONVENTIONS.md`** — estilo, tests, commits.
5. La spec del milestone activo en `docs/specs/` (archivo histórico M1–M6).
6. Los ADR que toquen el área: `docs/adr/`.
7. **`openspec/`** — fuente de verdad para cambios nuevos. Las specs vigentes
   viven en `openspec/specs/`; los cambios históricos, en
   `openspec/changes/archive/`.

## Comandos

```bash
cargo test                                  # 149 unit (1 live ignorado) + 10 integración
cargo clippy --all-targets -- -D warnings   # cero warnings
scripts/check.sh                            # fmt + clippy + tests + inspect
openspec validate --specs --strict           # validar specs de OpenSpec
cargo run --profile fast -- <libro.epub>    # iterar: ~3–6 s por cambio
cargo build --profile dist                  # artefacto final: ~7.2 MB
cargo run --profile fast                    # biblioteca
cargo run --profile fast -- --resume        # reabrir el último libro
cargo run --profile fast -- --scan ~/Libros # índice + portadas sin UI
cargo run --profile fast -- --inspect <libro.epub>          # diagnóstico de parseo
cargo run --profile fast -- --shot out.png --screen library  # captura de biblioteca
cargo run --profile fast -- --shot out.png --anchor 6 <libro.epub> # captura en un ancla
cargo run --profile fast -- --shot out.png --chapter 13 <libro.epub>   # captura visual
```

Para capturas/pruebas sin tocar la config real: `HOME=$(mktemp -d) cargo run …`.

## Reglas duras

- El disco de la máquina está al límite: **no dejes perfiles de compilación
  acumulados**. `cargo clean --profile dist` / `--profile release` antes de builds
  grandes. Un link que falla con `errno=28` es disco lleno, no código.
- No subir `egui`/`eframe` sin ADR: 0.37 es una reescritura (`App::ui`, sin
  `TopBottomPanel`). Ver `docs/adr/0003`.
- No agregar motor CSS ni webview: el modelo `Block` es el contrato.
- La biblioteca indexa rutas canónicas; no copiar ni mover los EPUB al
  directorio de datos (ADR-0007).
- No rasterizar SVG vectorial ni sumar `resvg` sin ADR: solo `<svg><image>`
  raster (ADR-0009).
- `config.json` se escribe atómico (temporal + `rename`); no volver a `fs::write`.
- Toda dependencia nueva necesita justificación en un ADR (peso, licencia, mantenimiento).
- Nada de `unwrap()` en rutas de datos de usuario (EPUB roto, archivo faltante).
- Los destacados y notas se guardan como **offsets de caracteres + cita** en
  `annotations.json`; no los conviertas a píxeles ni los mezcles con
  `library.json`. El formato v2 admite rangos entre párrafos (`end_block` +
  `parts`); mantené la lectura de archivos v1 (ADR-0014).
- Un enlace interno abre una **vista previa**; no navegues hasta “Ir al texto”.
  El regreso guarda un único `Progress` y se limpia con la navegación manual
  (ADR-0015). Los enlaces externos no se abren todavía.
- La actualización es **asistida**: consulta GitHub solo a pedido, verifica
  SHA-256 y guarda en Descargas; nunca reemplaza el binario/.app ni ejecuta el
  archivo descargado (ADR-0016). No agregues chequeo automático al iniciar.
- Las consultas online están **apagadas por defecto** (`online_enabled=false`) y
  nunca salen sin una acción explícita. Las peticiones van en un hilo; no
  bloquees la UI.
- La clave de IA vive en el llavero del sistema o en `OPENCODE_API_KEY`; nunca
  en `config.json`, `consultas.json`, logs ni URLs. El historial es local y no
  incluye el párrafo de contexto.
- Editar un EPUB exige backup previo y validación con `Book::open`; si falla, el
  original no se toca (ADR-0013).
- No bumpees la versión ni crees tags a mano: push a `main` y `auto-release`
  publica. Si falla, relanzá `gh workflow run release.yml -f tag=vX.Y.Z`
  (ADR-0017). macOS publica `Candil.app.zip` ad-hoc; no prometas doble clic sin
  Gatekeeper ni notarización.
- La biblioteca se refresca al recuperar el foco (debounce 30 s); no agregues un
  watcher del sistema sin ADR (ADR-0018). “Continuar leyendo” usa `last_opened`,
  no el orden visible de la grilla.
- La serie sale del OPF (`calibre:series`/`belongs-to-collection`), no del nombre
  de archivo. Si cambia la extracción, subí `META_VERSION`; no reabras toda la
  biblioteca en cada arranque (ADR-0020).
- Al iniciar manda `StartupMode`: último libro por defecto, biblioteca opcional;
  si falta el archivo, biblioteca. Los enlaces externos se abren solo con
  confirmación y esquemas `http`/`https`/`mailto`/`tel` (ADR-0021).
- El respaldo de datos incluye solo los cuatro JSON de usuario (`config`,
  `library`, `annotations`, `consultas`); nunca EPUB, portadas ni claves. La
  importación valida manifiesto/checksums/JSON y guarda un backup previo
  (ADR-0019).
- Cada tarea termina con: `cargo test` verde, `cargo fmt`, `cargo clippy
  --all-targets -- -D warnings`, y verificación visual con `--shot` si tocó UI.
  Si tocó el parser, correr también `--inspect tests/fixtures/edge.epub`.
