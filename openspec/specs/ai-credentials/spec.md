# ai-credentials Specification

## Purpose
Definir cómo Candil guarda, prioriza, muestra y borra la API key de IA sin escribirla en archivos de configuración, historial, logs ni URLs.
## Requirements
### Requirement: Almacenamiento en el llavero

La API key SHALL guardarse en el llavero nativo del sistema (Keychain en macOS, Secret Service en Linux, Credential Manager en Windows), bajo el servicio `com.quicio.candil` y la cuenta `ai-api-key`.

#### Scenario: Guardar la clave
- **WHEN** el usuario introduce una clave en Configuración y pulsa Guardar
- **THEN** la clave se guarda en el llavero, el campo se limpia y el estado pasa a “Clave guardada en el llavero”

#### Scenario: Reemplazar la clave
- **WHEN** el usuario guarda una clave nueva
- **THEN** la entrada anterior se reemplaza y no quedan dos claves activas

#### Scenario: Borrar la clave
- **WHEN** el usuario confirma Borrar
- **THEN** la entrada se elimina del llavero y el estado pasa a “Sin API key configurada”

### Requirement: Prioridad del entorno

`OPENCODE_API_KEY` SHALL tener prioridad sobre la clave guardada en el llavero.

#### Scenario: Variable presente
- **WHEN** `OPENCODE_API_KEY` existe y no está vacía
- **THEN** las consultas usan esa clave y la UI indica que el entorno tiene prioridad

#### Scenario: Variable ausente
- **WHEN** `OPENCODE_API_KEY` no existe
- **THEN** se usa la clave del llavero si está disponible

### Requirement: Ninguna clave en archivos, logs ni URLs

La clave SHALL NOT escribirse en `config.json`, `consultas.json`, logs, mensajes de error ni URLs; el historial SHALL guardar solo proveedor, modelo, consulta y respuesta.

#### Scenario: Serializar la configuración
- **WHEN** se guarda `config.json`
- **THEN** el JSON no contiene `OPENCODE_API_KEY`, `api_key` ni `ai_key`

#### Scenario: Consulta Gemini
- **WHEN** se consulta Gemini
- **THEN** la clave viaja en el header `x-goog-api-key`, no en la query string

#### Scenario: Error de red
- **WHEN** falla una consulta
- **THEN** el mensaje de error no incluye la clave

### Requirement: Estado y borrado en la UI

Configuración SHALL mostrar el origen de la clave (llavero, entorno o sin configurar), SHALL permitir guardar una nueva y SHALL pedir confirmación antes de borrar.

#### Scenario: Sin clave
- **WHEN** no hay clave en el entorno ni en el llavero
- **THEN** se muestra “Sin API key configurada” y el botón Borrar no aparece

#### Scenario: Clave en el llavero
- **WHEN** hay una clave guardada
- **THEN** se muestra el estado del llavero y aparece Borrar con confirmación

#### Scenario: Error de llavero
- **WHEN** el llavero no está disponible
- **THEN** se muestra el error, la variable de entorno sigue funcionando y no se escribe ningún archivo con la clave

### Requirement: Entornos de captura y CI

Las capturas y pruebas con `HOME` falso SHALL poder desactivar el llavero con `CANDIL_NO_KEYRING=1`, sin afectar el comportamiento normal.

#### Scenario: Capturas
- **WHEN** `CANDIL_NO_KEYRING=1` está definido
- **THEN** Candil no consulta el llavero y muestra “Sin API key configurada”

