## Context

`parse_openai` hacía una sola lectura:
`/choices/0/message/content` como `as_str()`. Si el proveedor devolvía un array
de partes (habitual en APIs OpenAI-compatible nuevas), `as_str()` era `None` y el
error era siempre “no devolvió texto”. Tampoco se miraba `finish_reason` ni los
campos de razonamiento.

## Goals / Non-Goals

**Goals:**

- Aceptar las formas reales de `content` sin cambiar el protocolo.
- Convertir un fallo opaco en un error con causa y acción.
- No mostrar razonamiento interno como si fuera la respuesta.

**Non-Goals:**

- Interpretar tool calls, streaming o contenido multimodal.
- Reintentar con más tokens automáticamente.

## Decisions

### 1. Normalizar `content`

`openai_content_text` acepta:

- string;
- array de strings u objetos con `text`;
- objeto con `text`.

Las partes se unen con salto de línea y se recortan. Si queda vacío, se sigue con
los diagnósticos.

### 2. Diagnóstico por `finish_reason`

- `length` → límite de tokens antes de texto; sugiere subir **máx. tokens**.
- `reasoning_content`, `reasoning` o `thinking` no vacíos → el modelo solo
  razonó; sugiere reintentar o subir tokens.
- otro `finish_reason` → se incluye en el mensaje genérico.

### 3. Formato legado

Si no hay `message.content`, se acepta `choices[0].text` para proveedores viejos.

## Risks / Trade-offs

- **Array con partes no textuales** → se ignoran las que no tienen `text`.
- **Razonamiento interno** → no se muestra para no confundirlo con la respuesta;
  se informa y se deja la decisión al usuario.
- **Error más técnico** → incluye `finish_reason`, pero sigue en español y con
  una acción concreta.

## Migration Plan

Sin migración. Rollback: volver a leer solo `content` string.
