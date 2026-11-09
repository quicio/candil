# Releases

Candil se publica solo. No hay que tocar `Cargo.toml` ni crear tags a mano.

## Flujo automático

1. Push a `main`.
2. `build` corre tests y compila en macOS/Linux/Windows.
3. Si `build` termina bien, `auto-release` decide si hay una entrega:
   - Ignora docs-only, cambios de CI y commits `chore(release): …`.
   - Bump **minor** si hay `feat…` desde el último tag.
   - Bump **major** si hay `feat!` o `BREAKING CHANGE`.
   - En el resto, **patch**.
4. Actualiza `Cargo.toml` y `Cargo.lock`, commitea
   `chore(release): vX.Y.Z`, crea el tag y lo pushea.
5. Dispara `release.yml` con ese tag.
6. `release.yml` compila los tres binarios, arma `Candil.app`,
   `candil-macos-arm64-app.zip` y `SHA256SUMS.txt`, y publica la release con
   notas generadas desde los commits.

## Assets

| asset | uso |
|---|---|
| `candil-linux-x86_64` | binario Linux |
| `candil-macos-arm64` | binario macOS (CLI) |
| `candil-macos-arm64-app.zip` | `Candil.app` firmado ad-hoc |
| `candil-windows-x86_64.exe` | binario Windows |
| `SHA256SUMS.txt` | checksums; la app los verifica |

La app (Configuración → Avanzado → Buscar actualizaciones) prefiere el zip del
`.app` en macOS y el binario en Linux/Windows. Descarga y verifica; **no
instala**.

## Instalación en macOS

```bash
brew install --cask quicio/tap/candil
```

o descargar `candil-macos-arm64-app.zip`, descomprimir, arrastrar `Candil.app` a
`/Applications` y, la primera vez, **clic derecho → Abrir**. Si macOS sigue
bloqueando:

```bash
xattr -dr com.apple.quarantine /Applications/Candil.app
```

Candil no está notarizado por Apple. El cask intenta quitar la cuarentena y
declara la advertencia en `caveats`. Para eliminar el aviso hace falta Developer
ID + notarización; el workflow está listo para agregarlos cuando exista la
cuenta.

## Manual (si el automático falla)

El tag puede existir aunque la release no:

```bash
gh workflow run release.yml -f tag=vX.Y.Z
```

Para forzar una nueva entrega, hacé un push de código a `main`; el
`auto-release` vuelve a evaluar los commits desde el último tag. No edites un
tag publicado: creá una versión nueva.

Ambos comandos requieren permisos de escritura en el repo. El tag debe coincidir
con `Cargo.toml`; si no, `release.yml` falla en “Verificar versión del tag”.

## Rollback

- Si el release quedó mal, corregir el código, push a `main` y dejar que salga
  la siguiente versión.
- Si solo falló la publicación del tag actual, relanzar `release.yml`.
- Nunca reescribir un tag publicado; crear una versión nueva.
