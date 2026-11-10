# ai-credentials Specification

## Purpose
Definir cómo Candil guarda, prioriza, muestra y borra la API key de IA sin escribirla en archivos de configuración, historial, logs ni URLs.
## Requirements
### Requirement: Almacenamiento en el llavero

La API key SHALL guardarse en el llavero nativo del sistema (Keychain en macOS, Secret Service en Linux, Credential Manager en Windows), bajo el servicio `com.quicio.candil` y una cuenta por proveedor `ai-api-key/<id>`.

#### Scenario: Guardar la clave
- **WHEN** el usuario introduce una clave en Configuración y pulsa Guardar
- **THEN** la clave se guarda en la cuenta del proveedor seleccionado, el campo se limpia y el estado pasa a “Clave guardada en el llavero”

#### Scenario: Reemplazar la clave
- **WHEN** el usuario guarda una clave nueva para el mismo proveedor
- **THEN** la entrada anterior de ese proveedor se reemplaza y no quedan dos claves activas

#### Scenario: Borrar la clave
- **WHEN** el usuario confirma Borrar
- **THEN** la entrada del proveedor seleccionado se elimina del llavero y el estado pasa a “Sin API key configurada”

#### Scenario: Proveedores distintos
- **WHEN** el usuario cambia de proveedor en Configuración
- **THEN** Candil carga la cuenta de ese proveedor, limpia el campo de entrada y no reutiliza la clave del proveedor anterior

### Requirement: Prioridad del entorno

`OPENCODE_API_KEY` SHALL tener prioridad sobre la clave guardada en el llavero solo para el proveedor OpenCode Go; para los demás proveedores SHALL ignorarse.

#### Scenario: Variable presente
- **WHEN** `OPENCODE_API_KEY` existe, no está vacía y el proveedor es OpenCode Go
- **THEN** las consultas usan esa clave y la UI indica que el entorno tiene prioridad

#### Scenario: Variable ausente
- **WHEN** `OPENCODE_API_KEY` no existe o el proveedor no es OpenCode Go
- **THEN** se usa la clave del llavero del proveedor si está disponible

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

Configuración SHALL mostrar el origen de la clave del proveedor seleccionado (llavero, entorno o sin configurar), SHALL permitir guardar una nueva y SHALL pedir confirmación antes de borrar.

#### Scenario: Sin clave
- **WHEN** no hay clave para el proveedor seleccionado en el entorno ni en el llavero
- **THEN** se muestra “Sin API key configurada” y el botón Borrar no aparece

#### Scenario: Clave en el llavero
- **WHEN** hay una clave guardada para el proveedor seleccionado
- **THEN** se muestra el estado del llavero y aparece Borrar con confirmación

#### Scenario: Cambio de proveedor con confirmación pendiente
- **WHEN** el usuario cambia de proveedor mientras hay una confirmación de borrado abierta
- **THEN** la confirmación se cancela y se muestra el estado del nuevo proveedor

#### Scenario: Error de llavero
- **WHEN** el llavero no está disponible
- **THEN** se muestra el error, la variable de entorno de OpenCode Go sigue funcionando y no se escribe ningún archivo con la clave

### Requirement: Entornos de captura y CI

Las capturas y pruebas con `HOME` falso SHALL poder desactivar el llavero con `CANDIL_NO_KEYRING=1`, sin afectar el comportamiento normal.

#### Scenario: Capturas
- **WHEN** `CANDIL_NO_KEYRING=1` está definido
- **THEN** Candil no consulta el llavero y muestra “Sin API key configurada”

### Requirement: Migración de la clave histórica

La clave guardada con la cuenta histórica compartida SHALL migrarse una sola vez a la cuenta del proveedor que la cargue primero, sin duplicarla a los demás proveedores.

#### Scenario: Primera lectura tras actualizar
- **WHEN** la cuenta del proveedor seleccionado no existe y la cuenta histórica `ai-api-key` contiene una clave
- **THEN** Candil mueve esa clave a la cuenta del proveedor, borra la histórica y la usa en la sesión

#### Scenario: Cuenta propia existente
- **WHEN** la cuenta del proveedor ya tiene una clave
- **THEN** la cuenta histórica no se toca ni se copia

#### Scenario: Migración incompleta
- **WHEN** la escritura en la cuenta nueva falla
- **THEN** la clave histórica sigue disponible para la sesión y se reintenta en el próximo arranque

