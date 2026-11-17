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
cargo test --workspace                      # escritorio + núcleo + FFI móvil
cargo clippy --workspace --all-targets -- -D warnings
scripts/check.sh                            # fmt + clippy + tests + inspect
openspec validate --specs --strict           # validar specs de OpenSpec
cargo run --profile fast -- <libro.epub>    # iterar: ~3–6 s por cambio
cargo build --profile dist                  # artefacto final: ~7.2 MB
cargo run --profile fast                    # biblioteca
cargo run --profile fast -- --resume        # reabrir el último libro
cargo run --profile fast -- --scan ~/Libros # índice + portadas sin UI
cargo run --profile fast -- --inspect <libro.epub>          # diagnóstico de parseo
cargo run --profile fast -- --shot out.png --screen library  # captura de biblioteca
cd mobile && flutter analyze && flutter test                # app Flutter
cd mobile && source tool/env.sh && tool/run_android.sh      # emulador/APK
python3 -m http.server 4174 --directory site                # landing local
cargo run --profile fast -- --shot out.png --anchor 6 <libro.epub> # captura en un ancla
cargo run --profile fast -- --shot out.png --chapter 13 <libro.epub>   # captura visual
```

Para capturas/pruebas sin tocar la config real: `HOME=$(mktemp -d) cargo run …`.

## Estructura

- Raíz `src/`: app de escritorio (egui).
- `crates/candil-core/`: parser EPUB, bloques y búsqueda. Sin UI ni plataforma.
- `mobile/`: Flutter + `mobile/rust` (adaptador C/JSON). Android/iOS.
- `site/`: landing estática publicada por `.github/workflows/pages.yml`.

## Reglas duras

- El disco de la máquina está al límite: **no dejes perfiles de compilación
  acumulados**. `cargo clean --profile dist` / `--profile release` antes de builds
  grandes. `dev` y `fast` corren sin incremental ni debug info para no llenar
  `target/`; no los reactives sin medir. Un link que falla con `errno=28` es disco
  lleno, no código.
- No subir `egui`/`eframe` sin ADR: 0.37 es una reescritura (`App::ui`, sin
  `TopBottomPanel`). Ver `docs/adr/0003`.
- No agregar motor CSS ni webview: el modelo `Block` es el contrato y vive en
  `crates/candil-core`.
- El núcleo portable no importa egui, Flutter, `dirs`, `keyring`, `rfd` ni rutas
  de plataforma. Esa regla de dependencia es ADR-0034.
- La app Flutter consume el FFI; no copia el parser ni la búsqueda.
- La landing vive en `site/`, usa las descargas de la última release y no
  promete funciones que no existan. GitHub Pages se despliega solo desde `main`.
- La biblioteca indexa rutas canónicas; no copiar ni mover los EPUB al
  directorio de datos (ADR-0007).
- La caché de EPUB remotos vive en `~/.cache/candil/books` (1 GB, LRU), nunca en
  datos ni en el respaldo. La ruta original sigue siendo la identidad del libro;
  no cambies `Book.path` a la copia. `Book::open_from` abre la copia conservando
  la identidad (ADR-0033).
- No rasterizar SVG vectorial ni sumar `resvg` sin ADR: solo `<svg><image>`
  raster (ADR-0009).
- `config.json` se escribe atómico (temporal + `rename`); no volver a `fs::write`.
- Toda dependencia nueva necesita justificación en un ADR (peso, licencia, mantenimiento).
- Nada de `unwrap()` en rutas de datos de usuario (EPUB roto, archivo faltante).
- Los destacados y notas se guardan como **offsets de caracteres + cita** en
  `annotations.json`; no los conviertas a píxeles ni los mezcles con
  `library.json`. El formato v3 admite rangos entre párrafos (`end_block` +
  `parts`) y `origin` (`user`/`ai`); mantené la lectura de archivos v1/v2
  (ADR-0014, ADR-0032).
- Compartir una cita genera un PNG **local** con `ab_glyph` (ADR-0031). No lo
  subas a ninguna red ni servicio; guardalo con temporal + `rename`. La captura
  no reemplaza al destacado: si falta, lo crea una sola vez.
- Un enlace interno abre una **vista previa**; no navegues hasta “Ir al texto”.
  El regreso guarda un único `Progress` y se limpia con la navegación manual
  (ADR-0015). Los enlaces externos no se abren todavía.
- La actualización se consulta solo a pedido y verifica SHA-256. En Linux, si
  el binario vive bajo `$HOME` y no es un build de `target/`, la UI puede
  instalarlo con backup + `rename` atómico y ofrecer reiniciar (ADR-0030); el
  resto de los casos sigue siendo asistido y nunca se ejecuta el archivo
  descargado. No agregues chequeo automático al iniciar.
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
- El respaldo de datos incluye solo los cinco JSON de usuario (`config`,
  `library`, `annotations`, `consultas`, `studies`); nunca EPUB, portadas ni
  claves. La importación valida manifiesto/checksums/JSON y guarda un backup
  previo (ADR-0019).
- El estudio vive en `studies.json`, separado de `consultas.json`: varias
  sesiones por libro, mensajes/contextos y referencias a anotaciones. Las
  respuestas de IA se guardan como mensajes; si el usuario las convierte en
  nota, la anotación lleva `origin: ai`. Nada se envía sin una acción de enviar
  y online sigue apagado por defecto (ADR-0032).
- Cada tarea termina con: `cargo test` verde, `cargo fmt`, `cargo clippy
  --all-targets -- -D warnings`, y verificación visual con `--shot` si tocó UI.
  Si tocó el parser, correr también `--inspect tests/fixtures/edge.epub`.
