## Context

`translate()` enviaba el texto completo a MyMemory y el proveedor respondía 403
con `QUERY LENGTH LIMIT EXCEEDED. MAX ALLOWED QUERY: 500 CHARS`. El panel de
consulta solo ofrecía MyMemory y exigía `online_enabled` para cualquier
traducción, incluso cuando el usuario tenía Ollama local.

## Goals / Non-Goals

**Goals:**

- Que `⇧K` y el botón Traducir funcionen con frases y párrafos sin pasos extra.
- Ofrecer traducción local opcional sin sumar dependencias ni modelos al binario.
- Mantener la privacidad: lo local no exige habilitar online; lo remoto sí.

**Non-Goals:**

- Chunking, traducción incremental o concatenación de fragmentos.
- Empaquetar modelos o reemplazar Ollama.
- Traducir sin selección, streaming o preguntas libres.

## Decisions

### 1. Recorte en palabra antes de enviar

`truncate_at_word` corta en el último espacio anterior a 500 caracteres y
garantiza el límite incluso sin espacios. El original no se modifica. Si hubo
recorte, `translate()` agrega una sección “Aviso”.

**Por qué:** una petición, una respuesta, sin rate limit extra ni UI de partes.
**Alternativa descartada:** chunking, por errores parciales y complejidad.

### 2. Ollama reutiliza el proveedor existente

`translation_provider = "ollama"` arma una copia local de `ConsultaSettings` con
`ai_provider = "ollama"`, base `http://localhost:11434/v1`, el modelo de
`translation_model` (o el default) y sin clave. Llama a `ai()` con
`AiPrompt::SoloTraducir`.

**Por qué:** no duplica cliente HTTP, parser ni manejo de timeout.
**Alternativa descartada:** endpoint dedicado, por mantenimiento innecesario.

### 3. Habilitación por destino de red

`run_consulta` permite la petición si `online_enabled` o si el motor/proveedor es
Ollama. El panel muestra “Ollama local · modelo” y el aviso de que no sale de la
Mac. El resto de consultas conserva el bloqueo.

### 4. Configuración y migración

`translation_provider` default `mymemory`; `translation_model` con
`#[serde(default)]`. `clamp()` normaliza valores desconocidos y completa el
modelo cuando el motor es Ollama. No hay migración explícita.

## Risks / Trade-offs

- **Traducción parcial silenciosa** → se avisa en la respuesta y antes de enviar.
- **Ollama apagado o sin modelo** → mensajes accionables (`abrir Ollama`,
  `ollama pull`).
- **Modelo local lento** → corre en un hilo; la UI muestra spinner y permite
  cancelar el flujo como cualquier consulta.
- **Calidad variable del modelo** → el usuario elige el modelo; MyMemory sigue
  siendo el default.

## Migration Plan

Sin migración de datos. Rollback: volver a `translation_provider = "mymemory"`
y quitar el routing local; el campo extra se ignora sin romper `config.json`.
