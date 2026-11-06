---
id: spec-65
title: Centro de configuración
status: aprobada
version: 1.0.0
updated: 2026-11-06
milestone: M6
depends_on: [spec-20, spec-35, spec-60, adr-0005]
---

# Spec 65 — Centro de configuración

> Spec hermana de la 60: las consultas necesitan un lugar donde activar la red,
> elegir proveedores, idioma, modelo, límite de historial y ver el estado de la
> clave. Ese lugar es el centro de configuración.

## Objetivo

Un único lugar para configurar el programa sin depender de editar JSON a mano:

- tipografía/lectura y preferencia por libro;
- biblioteca (carpetas, refresco, portadas);
- anotaciones y exportación;
- consultas (diccionario, traducción, IA) y privacidad;
- historial y datos avanzados.

La ventana “Tipografía” (`Aa`) sigue existiendo como acceso rápido **por libro**.
El centro de configuración edita lo global y las preferencias nuevas; no
duplica estado.

## Acceso

- Botón de engranaje `⚙` en la barra del lector y en la biblioteca.
- `⌘,` / `Ctrl+,` abre/cierra.
- `S` y `Aa` siguen abriendo la tipografía rápida, para no romper la costumbre.
- `Esc` cierra la ventana; no cambia de pantalla.

## Estructura propuesta

```
┌ Configuración ─────────────────────────────────────────────┐
│ Lectura       │ Tema, tamaño, interlineado, ancho, párrafo │
│ Biblioteca    │ Carpetas, refrescar, portadas, orden       │
│ Anotaciones   │ Color por defecto, exportar, historial     │
│ Consultas     │ Online, diccionario, traducción, IA        │
│ Privacidad    │ Qué sale a la red, historial, claves       │
│ Avanzado      │ Rutas, reset, versión                      │
└────────────────────────────────────────────────────────────┘
```

- Columna izquierda con la categoría activa; contenido con scroll a la derecha.
- Cambios con guardado diferido (700 ms) y `Guardar`/`Restablecer` por sección.
- Cada sección muestra una línea de estado (“Guardado”, “Requiere reinicio” no
  aplica en M6, “Sin conexión”).

### Lectura

- Edita `Config.settings` global (tema, fuente, interlineado, ancho, párrafo).
- Si hay un libro abierto, un botón **“Tipografía de este libro…”** abre el
  panel `Aa` existente; el checkbox “Solo para este libro” sigue viviendo ahí.
- La precedencia no cambia: `BookEntry.settings` → `Config.settings` → default.

### Biblioteca

- Lista de carpetas vigiladas con “Añadir carpeta…” y `×`.
- “Refrescar” ejecuta el escaneo incremental.
- Estado: cantidad de libros, faltantes, portadas cacheadas.
- “Borrar caché de portadas” con confirmación.
- Orden por defecto de la estantería (reciente/título/autor).

### Anotaciones

- Color por defecto para nuevos destacados (acento/ámbar/verde).
- Cantidad de anotaciones y acceso al panel “Destacados y notas”.
- Exportar todas a Markdown (reutiliza `export_markdown`).
- El historial de **consultas** no se mezcla con las anotaciones.

### Consultas

- **Permitir consultas online** (`online_enabled`), apagado por defecto. Al
  activarlo se muestra el aviso de privacidad una vez.
- Diccionario: Wiktionary (fijo en M6; selector para futuros proveedores).
- Traducción: MyMemory (gratuito) o LibreTranslate (URL configurable).
- Idioma destino: `Auto` (en→es, es→en, resto→es) / `es` / `en` / otro.
- IA:
  - proveedor **OpenCode Go**;
  - `base_url` `https://opencode.ai/inference/go/openai/v1` (avanzado, editable);
  - modelo `deepseek-v4.1-flash` (editable; “Ver modelos” en fase posterior);
  - timeout y `max_tokens`.
- **Estado de la clave**: “`OPENCODE_API_KEY` detectada” o “no configurada”.
  Nunca se muestra el valor ni se escribe en `config.json`.
- “Probar conexión” (solo con online activo y clave): petición mínima, spinner y
  resultado; puede consumir una fracción de cuota.
- Historial: límite de entradas, ruta del archivo, cantidad actual y “Limpiar
  historial” con confirmación.

### Privacidad

- Explica qué se envía y a quién: texto seleccionado + contexto del párrafo para
  IA, texto para diccionario/traducción; nunca notas, rutas, historial ni EPUB.
- Recordatorio de que el historial es local y se puede borrar.
- Enlace a los términos de OpenCode Go y a las fuentes (Wiktionary/MyMemory).
- `online_enabled=false` corta toda salida; los botones de consulta quedan
  deshabilitados con un aviso.

### Avanzado

- Rutas de `config.json`, `library.json`, `annotations.json`, `consultas.json` y
  `covers/`; botón “Abrir carpeta de datos”.
- Versión (`CARGO_PKG_VERSION`) y cantidad de datos.
- “Restablecer configuración” (global) con confirmación; no borra biblioteca,
  anotaciones ni historial salvo que se elija explícitamente.
- “Borrar historial de consultas” y “Borrar caché de portadas”.

## Modelo de configuración

`Config` conserva `settings`, `last_book`, `recent` y `progress` legado, y suma
una sección sin secretos:

```rust
#[derive(Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct ConsultaSettings {
    pub online_enabled: bool,          // false
    pub dictionary_provider: String,   // "wiktionary"
    pub translation_provider: String,  // "mymemory"
    pub target_lang: Option<String>,   // None = auto
    pub ai_base_url: String,           // https://opencode.ai/inference/go/openai/v1
    pub ai_model: String,              // deepseek-v4.1-flash
    pub ai_timeout_secs: u64,          // 30
    pub ai_max_tokens: u32,            // 512
    pub history_limit: usize,          // 200; 0 = no guardar
}
```

- `Config` se escribe atómico y con debounce, como hoy.
- `consultas.json` es propiedad de spec 60; la configuración solo guarda su
  límite y muestra su estado.
- Migración: archivos viejos cargan con `serde(default)`; no hay que tocar
  `library.json` ni `annotations.json`.

## UI y persistencia

- No se agregan dependencias: `egui` ya alcanza para la ventana y el layout.
- El centro de configuración no bloquea el hilo de UI; las pruebas de conexión
  usan el mismo patrón de hilo + `mpsc` de spec 60.
- La ventana se recuerda solo por sesión (posición/tamaño si egui lo permite);
  no se persiste estado de UI en M6.
- Los cambios de tipografía se ven en el acto; los de red se aplican a la
  próxima consulta.
- Un `status` discreto confirma guardados y errores.

## Tests

- Serde: `consulta` ausente → defaults; `online_enabled` default **false**;
  límites clampados.
- `Config` roundtrip sin secretos; la clave no aparece en el JSON.
- Precedencia de tipografía: global vs. `BookEntry.settings`.
- Historial: límite 0 no guarda, límite N recorta, limpiar borra.
- Aviso de privacidad: primera activación de online; no se repite.
- Botones de consulta deshabilitados con online apagado.
- UI con `--panel config` (`--shot`) y estados vacío/con datos.

## Criterio de salida

1. `⌘,`/engranaje abre la configuración y `Esc` la cierra.
2. Se puede activar/desactivar online y la clave se muestra solo como
   “detectada/no configurada”.
3. Se eligen idioma destino, proveedor de traducción y modelo de IA; persisten
   al reiniciar.
4. El historial se limita y se puede limpiar desde la UI.
5. La tipografía global y la de un libro siguen funcionando sin duplicar estado.
6. `cargo test` verde, clippy limpio y captura revisada.

## Riesgos

| riesgo | mitigación |
|---|---|
| Duplicar controles entre `Aa` y Configuración | `Aa` = por libro; Configuración = global; misma fuente de datos |
| Mostrar una clave por accidente | solo booleano “detectada”; nunca el valor ni logs |
| Activar red sin querer | `online_enabled=false` por defecto, aviso la primera vez, botones deshabilitados |
| Demasiadas opciones y UI pesada | secciones con scroll y valores por defecto; lo avanzado queda plegado |
| Historial sensible | límite, borrado, archivo local; no se envía a proveedores |
| Cambios de configuración a medio guardar | escritura atómica existente y debounce |

## Decisiones

- La pestaña **Biblioteca** replica los controles de la estantería; la pantalla
  de biblioteca sigue siendo el lugar natural para agregar/quitar carpetas.
- **Restablecer configuración** afecta solo `config.json`; borrar biblioteca,
  anotaciones o historial son acciones separadas y confirmadas.
- La ventana usa **barra lateral** de categorías para poder crecer.
- `base_url` y modelo de IA son editables en “Avanzado”; un selector con la
  lista de `/models` queda para una fase posterior.
