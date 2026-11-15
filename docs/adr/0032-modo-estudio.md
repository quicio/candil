# ADR-0032: Modo estudio con sesiones e IA conversacional

- **Estado:** aceptado
- **Fecha:** 2026-11-14
- **Área:** lector, IA, datos de usuario

## Contexto

La consulta de IA actual hace una sola pregunta sobre una selección. Para estudiar
un libro hace falta algo distinto: un hilo que recuerde lo anterior, que reciba el
pasaje exacto y su contexto, y que conviva con destacados, notas y marcadores. En
la práctica el usuario termina con ChatGPT al lado del lector copiando texto y
perdiendo el hilo entre aplicaciones.

Un estudio no es una consulta puntual: es un cuaderno con sesiones, mensajes,
contextos y artefactos. Guardarlo dentro de `consultas.json` mezclaría dos ciclos
de vida y dos formas de privacidad.

## Decisión

1. **`studies.json` separado.** Un quinto JSON de usuario, con
   `schema_version`, y varias `StudySession` por libro. Cada sesión tiene título,
   libro/ruta, timestamps, mensajes y referencias a anotaciones/marcadores. El
   archivo se escribe atómico con `store::save_json_atomic`.
2. **Un mensaje guarda su contexto.** `StudyMessage` tiene rol (`user`/`assistant`),
   texto, `contexts: Vec<StudyContext>`, proveedor/modelo de la respuesta y fecha.
   `StudyContext` distingue selección, página actual, destacado o marcador y
   conserva capítulo, cita, partes, offsets, párrafos vecinos e IDs de referencia.
3. **Chat multi-turno real.** `consulta::run_chat` comparte transporte, claves y
   parsers con la consulta de un turno, pero envía un sistema más el historial.
   El historial se recorta por cantidad de mensajes y caracteres; Anthropic
   coalesce turnos consecutivos del mismo rol. El prompt de sistema indica que el
   libro es dato, no instrucción.
4. **El contexto se ve antes de salir.** La bandeja permite adjuntar la selección,
   la página actual, destacados y marcadores. Si no hay nada adjunto y hay una
   selección, se usa esa selección y se avisa en el compositor. Nada se envía sin
   una acción de enviar.
5. **La respuesta de IA no es una anotación automática.** Vive en la sesión como
   mensaje `assistant`. “Guardar nota” crea una anotación normal con
   `origin: ai`, para que el lector la muestre con una insignia y siga siendo
   editable/borrable. Las anotaciones del usuario conservan `origin: user`.
6. **Privacidad local.** Las sesiones se guardan siempre en disco y entran en el
   respaldo; se pueden borrar por sesión o todas desde Configuración. Online sigue
   apagado por defecto y el composer bloquea el envío remoto si no está habilitado
   o si falta la clave. Ollama local sigue funcionando sin online.
7. **UI de estudio.** En ventanas anchas, un `SidePanel` derecho con el lector a la
   izquierda. El panel tiene selector de sesiones, “Nueva”, borrar, hilo
   scrolleable, bandeja de contexto y compositor con `Ctrl+Enter`. En ventanas
   angostas el panel ocupa el ancho disponible y el texto queda detrás.
8. **Migración y relink.** `studies.json` es opcional: si falta, se arranca vacío.
   Al reubicar un EPUB se reasigna la ruta de las sesiones, como ya se hace con
   `annotations.json`. La importación de respaldos viejos no borra sesiones
   existentes.

## Alternativas descartadas

- **Meter todo en `consultas.json`:** mezcla historial descartable con un cuaderno
  que el usuario quiere conservar, y obliga a contaminar `ConsultaEntry` con roles,
  contextos y referencias.
- **Un archivo por libro:** complica el respaldo, la búsqueda y las referencias
  entre libros; además fragmenta la escritura atómica.
- **Solo memoria:** pierde el hilo al cerrar la app y no se puede exportar.
- **Enviar todo el libro o todo el hilo en cada request:** costo y latencia
  inaceptables. Se recorta el historial y se adjunta contexto explícito; la
  recuperación automática de pasajes queda para otra fase.
- **Guardar cada respuesta como anotación:** llena `annotations.json` de contenido
  que el usuario no eligió guardar y borra la distinción entre nota humana y
  respuesta de IA.
- **Embeddings/RAG en v1:** agrega proveedor, índice y decisiones de privacidad.
  La búsqueda local puede sumar pasajes en una fase posterior.

## Consecuencias

- El respaldo pasa de cuatro a cinco JSON de usuario; hay que actualizar
  `backup.rs`, la spec de respaldo y el AGENTS.md.
- `annotations.json` sube a schema 3: `origin` es aditivo y los archivos v1/v2 se
  leen como `user`.
- El hilo y los contextos no se envían a ningún lado salvo en el request que el
  usuario dispara.
- Tests: store, sesiones múltiples, historial, rekey, roundtrip; chat multi-turno
  por protocolo; respaldo de cinco archivos; origen de anotación.
- Quedan fuera: resumen rodante, búsqueda/recuperación automática, varias ventanas,
  compartir el hilo y exportarlo (se puede sumar después).
