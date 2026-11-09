## Why

`⇧K` fallaba con párrafos largos porque MyMemory rechaza más de 500 caracteres, y
la traducción exigía habilitar consultas online aunque el usuario tuviera Ollama
corriendo en su Mac. El gesto rápido debe funcionar con frases y párrafos sin
obligar a un flujo de varias etapas ni a enviar texto afuera.

## What Changes

- MyMemory recorta a 500 caracteres en el último espacio antes del límite, sin
  partir palabras y sin chunking. Si hubo recorte, la respuesta agrega un aviso.
- Configuración → Consultas permite elegir **MyMemory (gratuita)** u
  **Ollama (local)** como motor de traducción, con modelo local configurable.
- La traducción con Ollama usa el endpoint OpenAI-compatible local
  (`http://localhost:11434/v1`), sin API key y con un prompt de solo traducción.
- La traducción local y la IA con Ollama funcionan con `online_enabled=false`;
  MyMemory/Wiktionary/proveedores remotos siguen requiriendo online.
- Errores de Ollama con instrucciones (`abrirlo`, `ollama pull <modelo>`).

## Capabilities

### New Capabilities

- Ninguna.

### Modified Capabilities

- `reading-workspace`: motor de traducción configurable, traducción local sin
  online y comportamiento explícito del recorte de MyMemory.

## Scope

**Incluido:** recorte en palabra, aviso de traducción parcial, selector de motor,
modelo local, routing a Ollama, habilitación local sin online, mensajes de error,
tests y documentación.

**No incluido:** chunking o traducción por partes, modelos empaquetados,
traducción automática sin selección, override manual del idioma de origen,
streaming de respuestas.

## Acceptance Criteria

- [x] MyMemory nunca recibe más de 500 caracteres ni una palabra partida.
- [x] Una traducción recortada muestra un aviso claro.
- [x] Se puede elegir MyMemory u Ollama en Configuración → Consultas.
- [x] Ollama traduce sin API key y sin activar consultas online.
- [x] `⇧K` usa el motor configurado y sigue siendo un solo gesto.
- [x] Si Ollama no está disponible, el error explica cómo resolverlo.
- [x] `cargo test`, `fmt`, `clippy -D warnings` y `scripts/check.sh` pasan.

## Impact

- `src/consulta.rs`, `src/config.rs`, `src/app.rs`.
- `config.json` gana `translation_model` (default vacío).
- Sin dependencias nuevas; Ollama es externo y opcional.
- Docs: README, HANDOFF, ROADMAP, ADR-0022.
