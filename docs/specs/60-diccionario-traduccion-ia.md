---
id: spec-60
title: Consulta de selección — diccionario, traducción e IA
status: implementada
version: 1.1.0
updated: 2026-11-06
milestone: M6
depends_on: [spec-30, spec-45, spec-65, adr-0005, adr-0011]
---

# Spec 60 — Consulta de selección: diccionario, traducción e IA

> **Estado: implementada el 2026-11-06.** Decisiones cerradas y código
> integrado (spec 60 + spec 65). La selección de texto ya existía desde M4.

## Objetivo

Seleccionar una palabra o frase y consultarla sin salir del lector:

1. **Diccionario** para una palabra (definición, ejemplos, sinónimos).
2. **Traducción** para una palabra o frase, con detección del idioma del libro y
   destino configurable (por defecto inglés ↔ español).
3. **Análisis con IA** para una frase: explicación en contexto, gramática,
   referencias o traducción comentada.

La consulta es **explícita**: nada se envía a la red por seleccionar texto. El
usuario elige Diccionario, Traducir o Analizar con IA. Las consultas online
están **apagadas por defecto** y se habilitan desde Configuración.

## Decisiones aprobadas (2026-11-06)

- **Diccionario:** Wiktionary REST; fallback inglés `dictionaryapi.dev`.
- **Traducción:** gratuita, **MyMemory** por defecto; LibreTranslate queda como
  alternativa configurable. Sin claves en el MVP.
- **IA:** **OpenCode Go** por suscripción, API OpenAI-compatible:
  - base `https://opencode.ai/inference/go/openai/v1`;
  - `POST /chat/completions` con `Authorization: Bearer …`;
  - clave en `OPENCODE_API_KEY` (solo entorno);
  - modelo por defecto `deepseek-v4.1-flash`, configurable;
  - header `x-opencode-session` estable por sesión y
    `User-Agent: lector/<versión>` (OpenCode Go pide identificarse como cliente
    propio, no como una librería HTTP genérica).
- **Claves:** solo variables de entorno; nunca en `config.json` ni en logs.
- **Online:** `online_enabled = false` por defecto; se activa explícitamente en
  Configuración y la primera consulta muestra un aviso.
- **IA:** solo prompts predefinidos (sin pregunta libre) en el MVP.
- **Historial:** persistente en disco (`consultas.json`), con límite, borrado y
  sin claves; ver sección Historial.
- **Traducción:** solo se muestra en el panel; **no reemplaza** el texto del
  libro. Un modo bilingüe que sustituya el texto queda fuera de alcance.

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
                 │ [Diccionario] [Traducción] [IA] [Historial]│
                 │                              │
                 │ …resultado…                  │
                 │ Fuente: Wiktionary · MyMemory│
                 └──────────────────────────────┘
```

- `Consultar` se agrega a la barra flotante de selección (o atajo `K`).
- Una palabra sin espacios abre Diccionario por defecto; una frase abre
  Traducción. Las cuatro pestañas (Diccionario, Traducción, IA e Historial)
  están disponibles según el estado.
- El panel se puede dejar abierto mientras se lee; al seleccionar otra frase se
  actualiza el texto y se cancela/ignora la consulta anterior.
- El idioma de origen se toma de `Book.lang` (normalizando `en-US`/`en-GB` a
  `en` y `es-419` a `es`). Si falta, se asume `en`. El override manual de origen
  queda para fase 3.
- El destino por defecto: si el libro es `en`, `es`; si es `es`, `en`; si no,
  `es`. Se puede cambiar y se recuerda para la sesión.

## Alcance por fases

### Fase 1 — Diccionario y traducción (MVP)

- Acción `Consultar` + panel con pestañas Diccionario y Traducción.
- Diccionario online: **Wiktionary REST** (multilingüe, sin clave).
  Fallback inglés: **dictionaryapi.dev**.
- Traducción online: **MyMemory** (sin clave) por defecto; LibreTranslate
  opcional en Configuración. DeepL/Google quedan para fase 3.
- Cache en memoria por `(proveedor, idioma, texto, destino)`, para no repetir
  llamadas mientras el libro está abierto.
- Historial persistente en `consultas.json` (ver sección Historial), visible en
  la pestaña Historial del panel.
- Estados: cargando, sin red, rate limit, error del proveedor, sin resultados.
- Atribución visible y enlace a la fuente.
- Nada se envía hasta que el usuario pulsa una pestaña/botón.

### Fase 2 — Análisis con IA

- **OpenCode Go** por defecto (`base_url`, `model` y timeout configurables en
  Configuración). La app no gestiona la suscripción: usa la clave del entorno.
- Clave **solo** desde `OPENCODE_API_KEY`; nunca en `config.json`, logs ni URL.
  Si falta, la pestaña IA explica cómo definirla y no hace la petición.
- Header `x-opencode-session`: un id aleatorio/estable generado al arrancar la
  app (timestamp + pid + contador; sin dependencia nueva) para cumplir el
  requisito del proveedor.
- Botones de prompt: “Explicar en contexto”, “Gramática”, “Resumir”, “Traducir
  y explicar”. El texto seleccionado y el párrafo contenedor se envían como
  **datos delimitados** (el libro es contenido no confiable: no se ejecutan
  instrucciones que aparezcan dentro).
- Respuesta en texto plano seleccionable/copiable; sin render Markdown en MVP.
- Petición en un hilo de trabajo + `mpsc`; la UI muestra “Analizando…” y no se
  congela. Timeout, cancelación por cierre o nueva selección y request id para
  descartar respuestas viejas.
- Etiqueta visible “Generado por IA” y proveedor/modelo usado. Si el proveedor
  devuelve error de cuota/429, se muestra el mensaje sin reintentar en loop.

### Fase 3 — Futuro

- Más proveedores (DeepL, LibreTranslate propio, Google).
- Cache persistente opcional (el historial ya es persistente).
- Claves en Keychain/libsecret además de variables de entorno (Finder no hereda
  el entorno del shell).
- Diccionario offline (sistema o paquete local) y “abrir en el Diccionario”.
- Streaming de la respuesta y markdown renderizado.

## Fuera de alcance

- Chat libre con el libro o con la biblioteca entera.
- Traducir el EPUB completo, reemplazar el texto del libro o generar una edición
  bilingüe.
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

- `Config` suma una sección `consulta` (sin secretos), editable desde el centro
  de configuración de la spec 65: `online_enabled`, `target_lang`,
  `translation_provider`, `dictionary_provider`, `ai_base_url`, `ai_model`,
  `ai_max_tokens`, `history_limit`.
- El cache de resultados es un `HashMap` en memoria del `Lector`.
- El historial es persistente en `consultas.json` (ver abajo).
- La selección ya trae `chapter`, `block`, `start`, `end`, `quote`; la consulta
  no modifica `annotations.json`.

## Historial en disco

Archivo separado `consultas.json`, escritura atómica con `src/store.rs` y
`schema_version: 1`. No se mezcla con `annotations.json` ni `library.json`.

```rust
pub struct ConsultaEntry {
    pub id: u64,
    pub created: i64,
    pub kind: ConsultaKind,
    pub provider: String,      // "Wiktionary", "MyMemory", "OpenCode Go"
    pub model: Option<String>, // solo IA
    pub prompt: Option<String>, // prompt predefinido, solo IA
    pub source_lang: String,
    pub target_lang: String,
    pub book: String,          // ruta canónica
    pub book_title: String,
    pub chapter: usize,
    pub block: usize,
    pub quote: String,         // selección, no el párrafo completo
    pub response: String,      // texto plano recortado a ~8 KB
    pub error: Option<String>,
}

pub struct ConsultaHistory {
    pub schema_version: u32, // 1
    pub items: Vec<ConsultaEntry>,
}
```

- Se guarda una entrada por consulta completada (también los errores, sin
  respuesta) para poder reintentar.
- **No se guarda** el párrafo de contexto, las claves ni el EPUB completo.
- Retención por defecto: 200 entradas o ~2 MB; al superar, se eliminan las más
  viejas. El límite es configurable y se puede poner en 0 para no guardar.
- La pestaña **Historial** del panel muestra fecha, tipo, cita y proveedor;
  click reabre el resultado. “Limpiar historial” pide confirmación y borra el
  archivo (o lo vacía).
- El historial es local; no se sincroniza ni se envía a los proveedores.

## Red, hilos y dependencias

- **HTTP**: se propone `ureq` + `rustls` (bloqueante, sin runtime async) y un
  ADR-0011 que compare `ureq`, `reqwest` y `curl` antes de agregarlo. Requisito:
  medir el aumento de `dist` (hoy 5,6 MB; presupuesto < 8 MB).
- **Hilos**: `std::thread` + `std::sync::mpsc`; un request id por consulta para
  descartar respuestas viejas. Nada de `block_on` en el hilo de UI.
- **JSON**: `serde_json` ya existe.
- **Sin webview** ni HTML del proveedor: se parsea JSON y se muestra texto.
- **Sin claves en `config.json`**: solo `OPENCODE_API_KEY` en el entorno. Si no
  hay clave, IA muestra cómo configurarla y no hace la petición. Las apps
  abiertas desde Finder pueden no heredar el entorno; se documenta lanzar desde
  terminal y se deja Keychain para fase 3.

### Endpoints candidatos (a validar en la implementación)

- Wiktionary: `GET https://{lang}.wiktionary.org/api/rest_v1/page/definition/{word}`
- Dictionary API (inglés): `GET https://api.dictionaryapi.dev/api/v2/entries/en/{word}`
- MyMemory: `GET https://api.mymemory.translated.net/get?q=...&langpair=en|es`
- IA (OpenCode Go): `POST https://opencode.ai/inference/go/openai/v1/chat/completions`
  con `Authorization: Bearer $OPENCODE_API_KEY`, `x-opencode-session` estable por
  sesión, `User-Agent: lector/<versión>` y
  `model: "deepseek-v4.1-flash"` por defecto.
- Lista de modelos: `GET https://opencode.ai/inference/go/v1/models` (fase
  posterior; en el MVP el modelo es un campo de texto con default).

## Privacidad y seguridad

- La primera vez que se usa una consulta online, un aviso explica que el texto
  seleccionado y (para IA) el párrafo se envían al proveedor elegido.
- `online_enabled` arranca en **false**; se activa desde Configuración y la
  primera consulta muestra un aviso. Desactivarlo bloquea toda salida a red
  (diccionario, traducción e IA).
- No se envían rutas, notas, historial ni el EPUB completo. El historial local
  no se manda a ningún proveedor; solo se envía el texto de la consulta actual.
- Las claves no se muestran, no se escriben en logs y no viajan en la URL.
- Los errores HTTP no incluyen la clave; se trunca el cuerpo del error.
- Rate limit local: una consulta en vuelo por panel; botones deshabilitados
  mientras carga.
- La respuesta de IA se muestra como sugerencia, no como hecho; el libro manda.
- La traducción se muestra **solo en el panel**: el texto del libro, los
  offsets de anotaciones y el progreso no se tocan. Un modo bilingüe que
  reemplace el texto queda fuera de alcance.

## UI y atajos

- `K` abre/cierra el panel de consulta con la selección actual.
- `Esc` cierra el panel; si hay una consulta en vuelo, se ignora la respuesta.
- El panel es una ventana lateral no modal, con scroll y botón “Copiar”.
- Si no hay selección, el panel muestra “Seleccioná una palabra o frase”.
- Si el texto supera ~2.000 caracteres, se recorta para la petición y se avisa
  (la selección completa sigue visible).
- Diccionario, traducción, IA e historial comparten el mismo panel; no se abren
  ventanas separadas por proveedor.
- La pestaña Historial muestra las entradas existentes; con `history_limit = 0` no se guardan nuevas y se puede limpiar desde Configuración.

## Tests

- Parseo de respuestas de Wiktionary/dictionaryapi/MyMemory con fixtures JSON
  (sin red).
- Codificación de palabras con acentos, espacios y apóstrofos en la URL.
- Selección de idioma destino y `Book.lang` faltante.
- Cache: misma consulta no repite request; distinta selección sí.
- Request id: una respuesta vieja no pisa la consulta nueva.
- Prompt de IA: delimitación del texto del libro y longitud máxima.
- Parseo de la respuesta OpenAI-compatible y errores (`401`, `429`, timeout).
- Historial: roundtrip, retención por límite, limpieza y archivo corrupto.
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
6. El historial sobrevive un reinicio, se puede limpiar y no contiene claves ni
   el párrafo completo.
7. Tests con fixtures/mocks y `dist` sigue por debajo de 8 MB.

Estado: 1–4 y 6–7 cumplidos y verificados; el 5 se validó con un 401 real de
OpenCode Go (sin congelar la UI) y con mocks, pero la respuesta exitosa de IA
requiere que el usuario configure `OPENCODE_API_KEY`.

## Riesgos y mitigaciones

| riesgo | mitigación |
|---|---|
| Dependencia HTTP engorda el binario | ADR con medición de `dist` antes de integrar; si no entra, evaluar `curl` o postergar |
| APIs gratuitas con rate limit o cambios | cache, timeout, errores claros, proveedor intercambiable |
| Privacidad: texto a terceros | acción explícita, aviso, `online_enabled`, no enviar notas/historial |
| Clave de IA filtrada | env vars, no logs, no URL; Keychain en fase 3 |
| Respuestas de IA inventadas | etiqueta “Generado por IA”, prompt con contexto, el texto original siempre visible |
| Inyección de prompt desde el libro | delimitar el pasaje y decir explícitamente que es dato, no instrucción |
| Libro en idioma desconocido | se usa `en`; el override manual de origen queda para fase 3 |
| Selección gigante | tope de ~2.000 caracteres para la petición |
| Historial con texto sensible | archivo local, límite, borrado y aviso; no se envía a proveedores |
| OpenCode Go cambia modelos/headers | base/model configurables; `x-opencode-session` estable; errores claros |
| Uso de la suscripción para lectura | el usuario verifica los términos/límites de OpenCode Go; la app solo envía fragmentos explícitos |

## Implementación (2026-11-06)

- `src/consulta.rs`: `ConsultaKind`, `ConsultaRequest/Response`, `Transport`,
  `HttpTransport` (ureq+rustls, timeout), `run`, `AiPrompt`, parsers de
  Wiktionary/dictionaryapi.dev/MyMemory/OpenAI-compatible y `ConsultaHistory`.
- UI: acción `Consultar` en la barra de selección y atajo `K`; panel con
  Diccionario, Traducción, IA e Historial. Petición en `std::thread` + `mpsc`,
  request id, spinner y descarte de respuestas viejas. Las fuentes se muestran
  con enlace y “Copiar”.
- OpenCode Go: `https://opencode.ai/inference/go/openai/v1/chat/completions`,
  `OPENCODE_API_KEY`, `x-opencode-session` estable y `User-Agent: lector/…`.
  Prompt con `<pasaje>`/`<contexto>` delimitados y prompts predefinidos.
- Historial: `consultas.json` atómico, límite configurable (200 por defecto),
  borrado con confirmación, sin contexto ni claves; se guardan también errores.
  El detalle permite **Ir al texto** (abre el libro/capítulo y resalta la cita)
  y copiar la respuesta.
- Configuración: el centro de la spec 65 edita online, proveedores, idioma,
  modelo, timeout, tokens e historial. `online_enabled=false` por defecto.
- **Medición**: `dist` pasó de 5.599.328 B a **6.664.160 B** (+1.064.832 B por
  ureq/rustls), dentro del presupuesto < 8 MB.
- **Verificación**: 119 unit (1 live ignorado) + 10 integración. Se probaron en
  vivo Wiktionary y MyMemory; con `wiktionary` real el panel muestra
  definiciones. OpenCode Go se probó con una clave inválida y mostró el 401 sin
  congelar la UI; falta la prueba de una respuesta exitosa con la clave real del
  usuario. El historial guardó las tres consultas (incluido el error).
- Capturas: `docs/img/reader-consulta.jpg`, `reader-history.jpg` y
  `reader-config.jpg`.

## Orden de implementación propuesto

1. `ADR-0011` HTTP + medición de `dist`.
2. Spec 65: centro de configuración (puede ir antes o en paralelo a T1).
3. T1: acción `Consultar`, panel y modelo de consulta (sin red).
4. T2: proveedor de diccionario + parseo + fixtures.
5. T3: traducción + idioma destino + cache + historial.
6. T4: hilo de trabajo, request id, errores y estados.
7. T5: OpenCode Go + prompts + estado de la clave.
8. T6: privacidad/UX, capturas y docs.
