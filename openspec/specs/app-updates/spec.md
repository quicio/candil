# app-updates Specification

## Purpose
Permitir que Candil compruebe, descargue y verifique una versión nueva desde la propia aplicación, sin reemplazar la instalación ni enviar datos del usuario.
## Requirements
### Requirement: Comprobación manual

La app SHALL consultar la última release estable de GitHub solo cuando el usuario pulse “Buscar actualizaciones”, en un hilo separado, y SHALL mostrar el resultado sin bloquear la lectura ni la configuración.

#### Scenario: Hay una versión nueva

- **WHEN** el usuario pide buscar actualizaciones y la release publicada es más nueva
- **THEN** se muestran el número de versión, la fecha y un resumen de las novedades, con las acciones de descarga y de abrir la release

#### Scenario: Está al día

- **WHEN** la versión publicada es igual o menor que la instalada
- **THEN** se muestra que Candil está al día y no se ofrece descargar nada

#### Scenario: Sin conexión o error de GitHub

- **WHEN** la consulta falla, vence el tiempo o GitHub responde con error
- **THEN** se muestra un mensaje claro y se ofrece reintentar, sin bloquear la UI

#### Scenario: Sin release publicado

- **WHEN** GitHub no devuelve una release estable
- **THEN** se informa que no hay releases disponibles y no se rompe la app

### Requirement: Selección de binario por plataforma

La app SHALL elegir el asset correspondiente a la plataforma actual y SHALL informar cuando no exista uno compatible.

#### Scenario: Plataforma soportada

- **WHEN** la release incluye `candil-macos-arm64`, `candil-linux-x86_64` o `candil-windows-x86_64.exe` según la plataforma
- **THEN** la acción de descarga usa ese asset y `SHA256SUMS.txt`

#### Scenario: Plataforma sin binario

- **WHEN** la release no incluye un asset para la plataforma actual
- **THEN** se informa que no hay binario compatible y se ofrece abrir la página del release

### Requirement: Descarga verificada

La descarga SHALL ejecutarse en un hilo, SHALL verificar el SHA-256 del asset contra `SHA256SUMS.txt` y SHALL guardar el archivo solo si la verificación coincide.

#### Scenario: Descarga correcta

- **WHEN** el asset y el checksum se descargan y coinciden
- **THEN** el archivo queda en el directorio de Descargas con un nombre que incluye la versión y la plataforma, con permiso de ejecución en Unix

#### Scenario: Checksum incorrecto

- **WHEN** el SHA-256 calculado no coincide con el publicado
- **THEN** el archivo no se guarda, se borra el temporal y se muestra un error

#### Scenario: Fallo de red o de disco

- **WHEN** la descarga o la escritura falla
- **THEN** no queda un archivo parcial en Descargas y se puede reintentar

### Requirement: La app no se autoinstala

La app SHALL limitar la actualización a comprobar, descargar y verificar; SHALL NOT reemplazar el binario en ejecución, el bundle `.app`, ni ejecutar el archivo descargado.

#### Scenario: Archivo descargado

- **WHEN** la descarga termina correctamente
- **THEN** se ofrece mostrar el archivo en Finder/Explorer y abrir la página del release, pero la instalación queda en manos del usuario

#### Scenario: App en ejecución

- **WHEN** el usuario descarga una versión nueva
- **THEN** la versión instalada y el proceso actual no cambian

### Requirement: Privacidad de la comprobación

La comprobación SHALL ser una acción explícita, SHALL NOT ejecutarse al iniciar ni en segundo plano y SHALL NOT enviar datos del libro, la biblioteca, las anotaciones o la clave de IA.

#### Scenario: Sin chequeo automático

- **WHEN** Candil arranca o el usuario lee
- **THEN** no se hace ninguna petición de actualización hasta que pulse “Buscar actualizaciones”

#### Scenario: Contenido de la petición

- **WHEN** se consulta GitHub
- **THEN** la petición solo lleva el `User-Agent` de Candil y los headers estándar de la API; no incluye datos del usuario

### Requirement: Estados y errores en la UI

La sección Avanzado SHALL mostrar el estado actual (inactivo, buscando, al día, disponible, descargando, descargado o error) y SHALL mantener las acciones coherentes con ese estado.

#### Scenario: Mientras se busca o descarga

- **WHEN** hay una operación en curso
- **THEN** el botón correspondiente queda deshabilitado y se muestra el progreso sin bloquear el resto de la configuración

#### Scenario: Ventana angosta

- **WHEN** la configuración se muestra a 720×600
- **THEN** el bloque de actualizaciones se lee y se opera sin desbordar ni tapar otras secciones

