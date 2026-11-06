---
id: spec-60
title: Consulta de selección — diccionario, traducción e IA
status: draft
version: 0.1.0
updated: 2026-11-06
milestone: M6
depends_on: [spec-30, spec-45, adr-0005]
---

# Spec 60 — Consulta de selección: diccionario, traducción e IA

> **Estado: borrador de diseño.** No se implementa hasta decidir proveedores,
> privacidad y la dependencia HTTP (ADR pendiente). La selección de texto ya
> existe desde M4: esta spec la reutiliza.

## Objetivo

Seleccionar una palabra o frase y consultarla sin salir del lector:

1. **Diccionario** para una palabra (definición, ejemplos, sinónimos).
2. **Traducción** para una palabra o frase, con detección del idioma del libro y
   destino configurable (por defecto inglés ↔ español).
3. **Análisis con IA** para una frase: explicación en contexto, gramática,
   referencias o traducción comentada.

La consulta es **explícita**: nada se envía a la red por seleccionar texto. El
usuario elige Diccionario, Traducir o Analizar con IA.

## Experiencia

```
selección en el lector
        │
        ▼
[●][●][●] Destacar  Nota  Consultar
                              │
                              ▼
                 ┌──────────────────────────────┐
                 │ Consulta                     │
                 │ “the sleep of reason…”       │
                 │ EN → ES          [×]         │
                 │ [Diccionario] [Traducción] [IA]│
                 │                              │
                 │ …resultado…                  │
                 │ Fuente: Wiktionary · MyMemory│
                 └──────────────────────────────┘
```

- `Consultar` se agrega a la barra flotante de selección (o atajo `K`).
- Una palabra sin espacios abre Diccionario por defecto; una frase abre
  Traducción. Las tres pestañas están siempre disponibles.
- El panel se puede dejar abierto mientras se lee; al seleccionar otra frase se
  actualiza el texto y se cancela/ignora la consulta anterior.
- El idioma de origen se toma de `Book.lang` (normalizando `en-US`/`en-GB` a
  `en` y `es-419` a `es`); es editable si falta o está mal.
- El destino por defecto: si el libro es `en`, `es`; si es `es`, `en`; si no,
  `es`. Se puede cambiar y se recuerda para la sesión.

## Alcance por fases

### Fase 1 — Diccionario y traducción (MVP)

- Acción `Consultar` + panel con pestañas Diccionario y Traducción.
- Diccionario online: **Wiktionary REST** (multilingüe, sin clave).
  Fallback inglés: **dictionaryapi.dev**.
- Traducción online: **MyMemory** (sin clave) por defecto; la interfaz queda
  preparada para LibreTranslate/DeepL.
- Cache en memoria por `(proveedor, idioma, texto, destino)`, para no repetir
  llamadas mientras el libro está abierto.
- Estados: cargando, sin red, rate limit, error del proveedor, sin resultados.
- Atribución visible y enlace a la fuente.
- Nada se envía hasta que el usuario pulsa una pestaña/botón.

### Fase 2 — Análisis con IA

- Proveedor **OpenAI-compatible** configurable (OpenAI, OpenRouter, Ollama,
  LM Studio): `base_url`, `model`, timeout.
- Clave desde variable de entorno (`LECTOR_AI_KEY`/`OPENAI_API_KEY`) o desde un
  archivo `ai-key` en el directorio de datos con permisos `0600`; nunca en
  `config.json` ni en logs. Las apps abiertas desde Finder no heredan el entorno
  del shell, así que el archivo es el camino práctico para uso diario.
- Botones de prompt: “Explicar en contexto”, “Gramática”, “Resumir”, “Traducir
  y explicar”. El texto seleccionado y el párrafo contenedor se envían como
  **datos delimitados** (el libro es contenido no confiable: no se ejecutan
  instrucciones que aparezcan dentro).
- Respuesta en texto plano seleccionable/copiable; sin render Markdown en MVP.
- Petición en un hilo de trabajo + `mpsc`; la UI muestra “Analizando…” y no se
  congela. Timeout y cancelación por cierre o nueva selección.
- Etiqueta visible “Generado por IA” y proveedor/modelo usado.

### Fase 3 — Futuro

- Más proveedores (DeepL, LibreTranslate propio, Google).
- Historial de consultas y cache persistente opcional.
- Claves en Keychain/libsecret en vez de variables de entorno.
- Diccionario offline (sistema o paquete local) y “abrir en el Diccionario”.
- Streaming de la respuesta y markdown renderizado.

## Fuera de alcance

- Chat libre con el libro o con la biblioteca entera.
- Traducir el EPUB completo o generar una edición bilingüe.
- Enviar automáticamente notas, subrayados o el libro a un proveedor.
- Sincronización de consultas entre dispositivos.
- Cuentas propias, facturación o gestión de suscripciones.

## Modelo propuesto

```rust
// src/consulta.rs
pub enum ConsultaKind { Diccionario, Traduccion, Ia }

pub struct ConsultaRequest {
    pub id: u64,
    pub kind: ConsultaKind,
    pub text: String,          // selección, máx. ~2.000 caracteres
    pub context: String,       // párrafo contenedor para IA/traducción
    pub source_lang: String,    // de Book.lang
    pub target_lang: String,
    pub book: String,           // ruta canónica
    pub title: String,
    pub author: String,
}

pub struct ConsultaResult {
    pub kind: ConsultaKind,
    pub provider: String,
    pub source_url: Option<String>,
    pub sections: Vec<Section>, // definiciones / traducciones / texto IA
    pub raw: String,            // para copiar/depurar
}
```

- `Settings`/`Config` suman una sección `consulta` (sin secretos):
  `target_lang`, `translation_provider`, `dictionary_provider`,
  `ai_base_url`, `ai_model`, `ai_max_tokens`, `online_enabled`.
- El cache es un `HashMap` en memoria del `Lector`; no se escribe en disco en
  el MVP.
- La selección ya trae `chapter`, `block`, `start`, `end`, `quote`; la consulta
  no modifica `annotations.json`.

## Red, hilos y dependencias

- **HTTP**: se propone `ureq` + `rustls` (bloqueante, sin runtime async) y un
  ADR-0011 que compare `ureq`, `reqwest` y `curl` antes de agregarlo. Requisito:
  medir el aumento de `dist` (hoy 5,6 MB; presupuesto < 8 MB).
- **Hilos**: `std::thread` + `std::sync::mpsc`; un request id por consulta para
  descartar respuestas viejas. Nada de `block_on` en el hilo de UI.
- **JSON**: `serde_json` ya existe.
- **Sin webview** ni HTML del proveedor: se parsea JSON y se muestra texto.
- **Sin claves en `config.json`**: variable de entorno o archivo `ai-key`
  (`0600`) en el MVP. Si no hay clave, IA muestra cómo configurarla y no hace la
  petición.

### Endpoints candidatos (a validar en la implementación)

- Wiktionary: `GET https://{lang}.wiktionary.org/api/rest_v1/page/definition/{word}`
- Dictionary API (inglés): `GET https://api.dictionaryapi.dev/api/v2/entries/en/{word}`
- MyMemory: `GET https://api.mymemory.translated.net/get?q=...&langpair=en|es`
- IA: `POST {base_url}/chat/completions` con `Authorization: Bearer …`.

## Privacidad y seguridad

- La primera vez que se usa una consulta online, un aviso explica que el texto
  seleccionado y (para IA) el párrafo se envían al proveedor elegido.
- `online_enabled` en `config.json` permite desactivar toda salida a red.
- No se envían rutas, notas, historial ni el EPUB completo.
- Las claves no se muestran, no se escriben en logs y no viajan en la URL.
- Los errores HTTP no incluyen la clave; se trunca el cuerpo del error.
- Rate limit local: una consulta en vuelo por panel; botones deshabilitados
  mientras carga.
- La respuesta de IA se muestra como sugerencia, no como hecho; el libro manda.

## UI y atajos

- `K` abre/cierra el panel de consulta con la selección actual.
- `Esc` cierra el panel; si hay una consulta en vuelo, se ignora la respuesta.
- El panel es una ventana lateral no modal, con scroll y botón “Copiar”.
- Si no hay selección, el panel muestra “Seleccioná una palabra o frase”.
- Si el texto supera ~2.000 caracteres, se recorta para la petición y se avisa
  (la selección completa sigue visible).
- Diccionario y traducción comparten el mismo panel; no se abren ventanas
  separadas por proveedor.

## Tests

- Parseo de respuestas de Wiktionary/dictionaryapi/MyMemory con fixtures JSON
  (sin red).
- Codificación de palabras con acentos, espacios y apóstrofos en la URL.
- Selección de idioma destino y `Book.lang` faltante.
- Cache: misma consulta no repite request; distinta selección sí.
- Request id: una respuesta vieja no pisa la consulta nueva.
- Prompt de IA: delimitación del texto del libro y longitud máxima.
- Parseo de la respuesta OpenAI-compatible y errores (`401`, `429`, timeout).
- UI con proveedor mock y `--shot` (sin red real).
- `cargo test`, clippy y `scripts/check.sh` verdes.

## Criterio de salida (MVP)

1. Seleccionar una palabra en un libro en inglés y ver definición + traducción
   al español en el panel.
2. Cambiar el idioma destino y traducir una frase completa.
3. Sin red o con rate limit, el panel muestra un error claro y el lector sigue
   usable.
4. Ninguna petición sale sin click explícito; la clave no está en `config.json`.
5. Seleccionar una frase y pedir análisis a la IA no congela la UI, se puede
   cerrar el panel y la respuesta vieja no aparece.
6. Tests con fixtures/mocks y `dist` sigue por debajo de 8 MB.

## Riesgos y mitigaciones

| riesgo | mitigación |
|---|---|
| Dependencia HTTP engorda el binario | ADR con medición de `dist` antes de integrar; si no entra, evaluar `curl` o postergar |
| APIs gratuitas con rate limit o cambios | cache, timeout, errores claros, proveedor intercambiable |
| Privacidad: texto a terceros | acción explícita, aviso, `online_enabled`, no enviar notas/historial |
| Clave de IA filtrada | env vars, no logs, no URL; Keychain en fase 3 |
| Respuestas de IA inventadas | etiqueta “Generado por IA”, prompt con contexto, el texto original siempre visible |
| Inyección de prompt desde el libro | delimitar el pasaje y decir explícitamente que es dato, no instrucción |
| Libro en idioma desconocido | permitir elegir origen/destino manualmente |
| Selección gigante | tope de ~2.000 caracteres para la petición |

## Decisiones abiertas para Hugo

1. ¿Proveedor de diccionario preferido: Wiktionary + dictionaryapi.dev, o el
   Diccionario del sistema en macOS?
2. ¿Traducción gratuita (MyMemory/LibreTranslate) o con clave (DeepL/Google)?
3. ¿Proveedor de IA: OpenAI, Anthropic, OpenRouter, Ollama local? ¿La app debe
   asumir `base_url` + `model` configurables?
4. ¿Claves por variable de entorno, archivo `ai-key` con permisos `0600` o
   Keychain desde el inicio? (Finder no hereda el entorno del shell.)
5. ¿El análisis de IA debe incluir una pregunta libre del usuario o solo los
   prompts predefinidos?
6. ¿Guardar historial de consultas en disco (privacidad) o solo en memoria?
7. ¿La traducción debe poder reemplazar el texto del libro o solo mostrarse en
   el panel? (Recomendado: solo panel.)
8. ¿`online_enabled` arranca en `true` con aviso en la primera consulta, o en
   `false` hasta que el usuario lo active explícitamente?

## Orden de implementación propuesto

1. `ADR-0011` HTTP + medición de `dist`.
2. T1: acción `Consultar`, panel y modelo de consulta (sin red).
3. T2: proveedor de diccionario + parseo + fixtures.
4. T3: traducción + idioma destino + cache.
5. T4: hilo de trabajo, request id, errores y estados.
6. T5: IA OpenAI-compatible + prompts + configuración sin secretos.
7. T6: privacidad/UX, capturas y docs.
