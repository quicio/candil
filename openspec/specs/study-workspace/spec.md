# study-workspace Specification

## Purpose
Convertir la lectura en un espacio de estudio con sesiones persistentes por libro, contexto explícito y una conversación de IA que mantiene el hilo, sin mezclar estudio con consultas puntuales ni enviar datos sin una acción del usuario.
## Requirements
### Requirement: Sesiones de estudio separadas

La app SHALL guardar las sesiones de estudio en `studies.json`, separadas de `consultas.json`, con varias sesiones por libro.

#### Scenario: Nueva sesión sobre el mismo libro

- **WHEN** el usuario crea una sesión nueva mientras ya existe otra para el mismo libro
- **THEN** ambas sesiones se conservan con su título, mensajes y contexto propios

#### Scenario: Reabrir un libro

- **WHEN** el usuario vuelve a abrir un libro con sesiones guardadas
- **THEN** el panel muestra la sesión actualizada más recientemente y permite cambiar a otra

#### Scenario: Archivo ausente

- **WHEN** no existe `studies.json`
- **THEN** la app arranca sin sesiones y no falla

### Requirement: Panel de estudio dividido

En ventanas anchas la app SHALL mostrar el lector a la izquierda y el estudio a la derecha sin reemplazar el texto; en ventanas angostas el estudio SHALL ocupar el ancho disponible.

#### Scenario: Abrir el modo estudio

- **WHEN** el usuario pulsa “Estudio” o la tecla `E`
- **THEN** se abre el panel con el libro actual, la sesión elegida y el compositor

#### Scenario: Cerrar el modo estudio

- **WHEN** el usuario pulsa la cruz o `Esc`
- **THEN** el texto vuelve a ocupar el ancho normal y la sesión queda guardada

### Requirement: Contexto explícito y visible

El compositor SHALL permitir adjuntar selección actual, página actual, destacados y marcadores, y SHALL mostrar los contextos adjuntos antes de enviar.

#### Scenario: Adjuntar una selección

- **WHEN** el usuario elige “Selección actual”
- **THEN** la cita, el capítulo y los párrafos vecinos quedan como contexto visible y removible

#### Scenario: Enviar sin contexto adjunto

- **WHEN** hay una selección activa y el usuario envía una pregunta sin contextos adjuntos
- **THEN** la selección se adjunta automáticamente y el compositor lo anuncia antes de enviar

#### Scenario: Demasiados contextos

- **WHEN** el usuario intenta adjuntar más de ocho contextos
- **THEN** la app no agrega más y lo explica

### Requirement: Conversación multi-turno

El envío SHALL usar el proveedor de IA configurado, mantener el historial de la sesión y respetar el estado de online y de la clave.

#### Scenario: Pregunta con historial

- **WHEN** el usuario envía una pregunta después de una respuesta anterior
- **THEN** el request incluye el sistema, los mensajes anteriores y el contexto nuevo, recortados a un presupuesto de tamaño

#### Scenario: Online apagado

- **WHEN** el proveedor es remoto y las consultas online están desactivadas
- **THEN** el envío se bloquea con un mensaje y no sale ninguna petición

#### Scenario: Error del proveedor

- **WHEN** la petición falla
- **THEN** el mensaje del usuario queda en la sesión, se muestra el error y se ofrece “Reintentar” sin duplicar la pregunta

### Requirement: Respuestas de IA diferenciadas

Las respuestas SHALL guardarse como mensajes de la sesión con proveedor y modelo, y “Guardar nota” SHALL crear una anotación con origen IA.

#### Scenario: Guardar una respuesta

- **WHEN** el usuario pulsa “Guardar nota” en una respuesta con un pasaje adjunto
- **THEN** se crea una anotación con la cita del pasaje y la respuesta como nota, con origen IA

#### Scenario: Respuesta sin pasaje

- **WHEN** la respuesta no tiene un pasaje adjunto con texto
- **THEN** la acción de guardar nota no se ofrece o se explica por qué no se puede

#### Scenario: Panel de anotaciones

- **WHEN** una anotación tiene origen IA
- **THEN** el panel la muestra con una insignia “IA” y sigue siendo editable y borrable

### Requirement: Privacidad y persistencia

Las sesiones SHALL guardarse solo en disco, entrar en el respaldo de datos y poder borrarse por sesión o todas; el envío remoto SHALL seguir siendo explícito.

#### Scenario: Respaldo

- **WHEN** el usuario exporta sus datos
- **THEN** `studies.json` se incluye con checksum y la importación lo valida

#### Scenario: Borrar sesiones

- **WHEN** el usuario confirma “Borrar sesiones de estudio”
- **THEN** se eliminan todas las sesiones y el panel queda vacío

#### Scenario: Sin red automática

- **WHEN** el usuario abre el modo estudio o cambia de sesión
- **THEN** no se hace ninguna petición de red
