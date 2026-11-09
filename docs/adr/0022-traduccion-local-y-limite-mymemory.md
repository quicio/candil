# ADR-0022: Traducción local con Ollama y límite de MyMemory

- **Estado:** aceptado
- **Fecha:** 2026-11-09
- **Área:** consultas, privacidad, configuración

## Contexto

`⇧K` traduce la selección directo, pero MyMemory rechaza consultas de más de
500 caracteres (`QUERY LENGTH LIMIT EXCEEDED`). Un párrafo largo fallaba justo
en el gesto pensado para leer sin interrupciones. Además, la traducción exigía
activar “Permitir consultas online”, aunque el usuario ya tuviera Ollama corriendo
en su Mac.

El usuario lee en inglés y quiere marcar una frase y ver la traducción sin pasos
extra. La traducción no debe convertirse en un flujo de varias etapas ni enviar
texto a un servicio externo cuando puede resolverse localmente.

## Decisión

### MyMemory: recorte en palabra, sin chunking

`translate()` recorta el texto a `MYMEMORY_MAX_CHARS` (500) con
`truncate_at_word`, que corta en el último espacio anterior al límite y nunca
parte una palabra. Si no hay espacios, corta exactamente en el límite. Cuenta
caracteres, no bytes.

Cuando el texto original supera el límite, la respuesta agrega una sección
“Aviso” que explica que se tradujo el fragmento hasta la última palabra
completa. El panel también avisa antes de enviar. No se hacen varias peticiones
ni se concatenan traducciones: el flujo sigue siendo una selección → una
respuesta.

**Por qué no chunking:** multiplica las peticiones (y el rate limit gratuito),
puede desordenar o duplicar fragmentos, exige manejar errores parciales y agrega
UI para explicar el resultado. Para frases y párrafos cortos, el recorte cubre
el caso real sin complejidad.

### Motor de traducción configurable

`ConsultaSettings.translation_provider` acepta `mymemory` (default) u `ollama`.
`translation_model` guarda el modelo local; si está vacío se usa el default del
proveedor (`llama3.2`). La configuración se edita en Configuración → Consultas.

Ollama reutiliza el proveedor OpenAI-compatible existente
(`http://localhost:11434/v1`, sin API key). La traducción local usa
`AiPrompt::SoloTraducir`, un system prompt estricto que pide devolver únicamente
la traducción, conservando párrafos y sin comentarios. La respuesta se marca como
`ConsultaKind::Traduccion` con título “Traducción”, así el panel y el historial
la tratan como cualquier traducción.

### Privacidad: local no exige online

La traducción y la IA con Ollama funcionan con `online_enabled=false`; MyMemory,
Wiktionary y los proveedores remotos siguen bloqueados hasta habilitar online.
La regla es por destino de red, no por tipo de consulta: si el texto no sale de
la máquina, no hace falta consentimiento de red.

### Errores accionables

Si Ollama no responde, el error sugiere abrirlo y verificar el modelo. Si el
modelo no existe (404), sugiere `ollama pull <modelo>`. El timeout y los tokens
máximos se reutilizan de la configuración de IA; la traducción fuerza al menos
1024 tokens para no cortar párrafos.

## Alternativas descartadas

- **Chunking de MyMemory:** más cobertura, pero varias peticiones, rate limit y
  una UI más pesada. Se descarta hasta que el uso real lo justifique.
- **Empaquetar un modelo local:** binario y distribución inviables; Ollama ya
  resuelve la inferencia y el usuario elige el modelo.
- **Usar la pestaña IA para traducir:** agrega un paso y mezcla traducción con
  explicación; `⇧K` debe seguir siendo directo.
- **Exigir online para Ollama:** no aporta privacidad ni seguridad; el tráfico es
  loopback.

## Consecuencias

- Una traducción de MyMemory puede ser parcial; se avisa explícitamente y el
  texto original no se modifica.
- Sin Ollama instalado, el motor local falla con instrucciones claras; el motor
  por defecto sigue siendo MyMemory.
- No se agregan dependencias ni modelos al binario. `config.json` gana
  `translation_model` con default vacío para archivos viejos.
- La traducción local no aparece en la red externa; el historial sigue siendo
  local y no guarda el párrafo de contexto.
- Tests cubren el recorte sin partir palabras, el mapeo del error de 500, el
  aviso de traducción parcial y el routing a Ollama sin API key.
