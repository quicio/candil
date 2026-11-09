# ADR-0017: Entrega automática y empaquetado para macOS

- **Estado:** aceptado
- **Fecha:** 2026-11-09
- **Área:** CI/CD, distribución, macOS

## Contexto

El release se hacía a mano: subir `Cargo.toml`, commit, tag, push, esperar el
workflow, editar las notas, descargar el asset e instalar. Eso hace que la
versión publicada se atrase respecto de `main`.

En macOS el release solo incluía el binario crudo. El usuario tenía que crear el
`.app` a mano y, al descargarlo, Gatekeeper mostraba “no se puede abrir porque es
de un desarrollador no identificado”. No hay Developer ID ni notarización, así
que la advertencia no se puede eliminar del todo.

## Decisión

### Release automático después de CI

Un workflow `auto-release` se dispara con `workflow_run` cuando `build` termina
bien en `main`. No publica si el build falla, si el push no es a `main`, si el
commit es `chore(release): …` o si desde el último tag no cambió código
(`src/`, `Cargo.toml`, `Cargo.lock`, `assets/`, `resources/`).

La versión sigue conventional commits:

- `feat…` → minor.
- `feat!` o `BREAKING CHANGE` → major.
- Resto → patch.

El workflow actualiza `Cargo.toml` y `Cargo.lock`, commitea
`chore(release): vX.Y.Z`, crea el tag anotado, pushea y dispara `release.yml`
por `workflow_dispatch` con el tag como input. El push con `GITHUB_TOKEN` no
dispara workflows, así que no hay loop; el dispatch es explícito.

### `release.yml` con tag de entrada

`release.yml` sigue funcionando con push de tags, y además acepta
`workflow_dispatch` con `tag`. Todos los checkouts usan ese tag; la verificación
de versión y la creación del release también. Así el auto-release puede pedir el
build del tag recién creado sin depender de un PAT.

### macOS: `.app.zip` ad-hoc

El job de macOS ejecuta `scripts/make_app.sh` (firma ad-hoc) y empaqueta el
bundle como `candil-macos-arm64-app.zip` con `ditto` y como
`candil-macos-arm64-app.tar.gz` para Homebrew. El release publica:

- `candil-linux-x86_64`
- `candil-macos-arm64`
- `candil-macos-arm64-app.zip`
- `candil-macos-arm64-app.tar.gz`
- `candil-windows-x86_64.exe`
- `SHA256SUMS.txt`

El updater de la app prefiere el zip en macOS y el binario en el resto. La
instalación sigue siendo manual: no se reemplaza el `.app` en ejecución.

### Homebrew cask

Se agrega `Casks/candil.rb` y un tap `quicio/homebrew-tap`:

```bash
brew install --cask --no-quarantine quicio/tap/candil
```

El cask usa `version :latest` y `sha256 :no_check` contra
`releases/latest/download/candil-macos-arm64-app.tar.gz`; no hay que actualizarlo
por release. Un `postflight` intenta quitar `com.apple.quarantine`; si falla, el
usuario puede abrir con clic derecho → Abrir. El cask declara la advertencia en
`caveats`.

### Firma/notarización futura

El pipeline queda preparado para agregar Developer ID y notarización cuando
exista la cuenta: importar el `.p12`, firmar con hardened runtime, `notarytool
submit --wait` y `stapler staple`. Sin esos secretos, la firma ad-hoc es el
límite.

## Alternativas descartadas

- **Release manual con un script:** reduce pasos, pero sigue dependiendo de una
  persona y no publica al entregar código.
- **Release en cada push sin esperar CI:** puede publicar código roto.
- **PAT para que el tag dispare `release.yml`:** funciona, pero agrega un secret
  de larga duración y permisos extra. `workflow_dispatch` con `GITHUB_TOKEN`
  cubre el caso.
- **Solo binario crudo:** es lo que había; obliga a armar el `.app` a mano.
- **Notarización ahora:** no hay Developer ID; no se puede completar.
- **Autoinstalación del `.app`:** reemplazar un bundle en ejecución, re-firmar y
  reiniciar es un proyecto aparte y con más riesgo.

## Consecuencias

- `main` publicado queda al día sin intervención; los docs-only no generan
  release.
- Los commits de release los firma `github-actions[bot]`; el tag y el commit
  quedan en el historial.
- Si el dispatch o el build del release fallan, hay que relanzar
  `release.yml` con `-f tag=vX.Y.Z`; el tag ya existe.
- La advertencia de Gatekeeper sigue en la primera apertura. El zip y el cask
  reducen la fricción, pero no la eliminan sin Developer ID.
- El tap con `:latest` no necesita actualización por release, a costa de no
  fijar una versión/sha en el cask.
- El `dist` no cambia por el empaquetado; el workflow tarda más por el `.app` y
  el zip y el tar.gz.
