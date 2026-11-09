# ai-providers Specification

## Purpose
Definir un catálogo de proveedores de IA y los protocolos HTTP necesarios para que Candil pueda analizar pasajes con distintos servicios sin cambiar el resto de la experiencia de lectura.
## Requirements
### Requirement: Catálogo de proveedores

El sistema SHALL ofrecer OpenCode Go, OpenAI, Anthropic, Google Gemini, OpenRouter, Groq, Mistral, DeepSeek y Ollama local, cada uno con id estable, etiqueta, protocolo, base URL, modelo por defecto y URL de origen.

#### Scenario: Abrir el selector
- **WHEN** el usuario abre Configuración → Consultas
- **THEN** el selector muestra los nueve proveedores y el proveedor activo

#### Scenario: Proveedor inválido
- **WHEN** `config.json` contiene un id de proveedor desconocido
- **THEN** Candil cae en OpenCode Go y completa base URL/modelo con sus defaults

### Requirement: Selección persistente y defaults

El proveedor elegido SHALL guardarse en `config.json`; al cambiarlo, la base URL y el modelo SHALL actualizarse a los defaults del proveedor, y luego SHALL poder editarse manualmente.

#### Scenario: Cambiar de proveedor
- **WHEN** el usuario elige Anthropic
- **THEN** base URL y modelo pasan a los de Anthropic y la elección se persiste

#### Scenario: Editar el modelo
- **WHEN** el usuario cambia el modelo a mano
- **THEN** la consulta usa ese modelo y no se pisa al reabrir la configuración

### Requirement: Protocolo OpenAI-compatible

Para OpenCode Go, OpenAI, OpenRouter, Groq, Mistral, DeepSeek y Ollama, el sistema SHALL usar `POST {base}/chat/completions`, `Authorization: Bearer` cuando hay clave y `choices[0].message.content` como respuesta.

#### Scenario: OpenCode Go
- **WHEN** el proveedor es OpenCode Go
- **THEN** además de Bearer se envía `x-opencode-session` y se parsea `choices[0].message.content`

#### Scenario: Error del proveedor
- **WHEN** la respuesta contiene `error.message`
- **THEN** el error se muestra sin incluir la clave

### Requirement: Protocolo Anthropic

Para Anthropic, el sistema SHALL usar `POST {base}/messages`, headers `x-api-key` y `anthropic-version`, body con `system`, `messages` y `max_tokens`, y concatenar los bloques `content[].text`.

#### Scenario: Respuesta de Anthropic
- **WHEN** el proveedor es Anthropic y la respuesta trae bloques de texto
- **THEN** el panel muestra el texto concatenado y el proveedor “Anthropic”

### Requirement: Protocolo Gemini

Para Google Gemini, el sistema SHALL usar `POST {base}/models/{model}:generateContent`, header `x-goog-api-key` (nunca la clave en la query), `system_instruction`, `contents` y concatenar `candidates[0].content.parts[].text`.

#### Scenario: Respuesta de Gemini
- **WHEN** el proveedor es Gemini y la respuesta trae `candidates`
- **THEN** el panel muestra el texto y el proveedor “Google Gemini”

#### Scenario: Respuesta bloqueada
- **WHEN** Gemini devuelve `promptFeedback.blockReason` sin texto
- **THEN** el error explica que la respuesta fue bloqueada

### Requirement: Requisito de clave por proveedor

Ollama local SHALL funcionar sin API key; el resto SHALL exigir una clave y deshabilitar la acción de IA si falta.

#### Scenario: Ollama
- **WHEN** el proveedor es Ollama y no hay clave
- **THEN** “Analizar con IA” está habilitado y no se envía `Authorization`

#### Scenario: OpenAI sin clave
- **WHEN** el proveedor es OpenAI y no hay clave
- **THEN** “Analizar con IA” está deshabilitado y se indica dónde configurarla

### Requirement: Identidad del proveedor en el historial

Cada respuesta SHALL guardar en el historial la etiqueta del proveedor usado y el modelo, sin guardar la clave.

#### Scenario: Consulta con Anthropic
- **WHEN** una consulta termina con Anthropic
- **THEN** la entrada del historial muestra “Anthropic” y el modelo, y no contiene la clave

