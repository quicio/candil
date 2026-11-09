## Purpose

Permitir que el usuario guarde y restaure localmente su biblioteca, progreso, anotaciones, historial y ajustes en un archivo portable, sin depender de la nube ni incluir secretos.

## ADDED Requirements

### Requirement: Exportación local

La app SHALL exportar los datos de usuario a un ZIP elegido por el usuario, con un manifiesto que describa formato, versión, fecha y checksums.

#### Scenario: Exportar con datos

- **WHEN** el usuario elige “Exportar datos…” y un destino
- **THEN** se crea un ZIP con `config.json`, `library.json`, `annotations.json`, `consultas.json` y el manifiesto, y se informa la ruta

#### Scenario: Sin datos

- **WHEN** no existe ningún archivo de datos
- **THEN** la exportación falla con un mensaje claro y no crea un ZIP vacío

#### Scenario: Destino peligroso

- **WHEN** el usuario elige como destino uno de los archivos de datos de Candil
- **THEN** la exportación se rechaza para no sobrescribir la fuente

### Requirement: Importación validada

La importación SHALL validar el manifiesto, la lista blanca de archivos, los checksums y el JSON antes de reemplazar cualquier dato.

#### Scenario: Respaldo válido

- **WHEN** el usuario importa un ZIP generado por Candil
- **THEN** los datos se reemplazan, la app recarga biblioteca, ajustes, anotaciones e historial, vuelve a la biblioteca y lanza un escaneo incremental

#### Scenario: Checksum incorrecto

- **WHEN** el contenido de un archivo no coincide con el checksum del manifiesto
- **THEN** la importación se aborta y los datos actuales no se tocan

#### Scenario: JSON inválido

- **WHEN** uno de los archivos no se puede parsear
- **THEN** la importación se aborta antes de escribir y se explica cuál archivo falló

#### Scenario: Archivo no permitido

- **WHEN** el manifiesto menciona un archivo fuera de la lista de datos de Candil
- **THEN** la importación se rechaza para evitar rutas arbitrarias

### Requirement: Backup previo e integridad

Antes de reemplazar datos existentes, la importación SHALL crear un respaldo del estado actual; los archivos SHALL escribirse de forma atómica.

#### Scenario: Datos existentes

- **WHEN** la carpeta de datos ya contiene archivos y se importa un respaldo
- **THEN** se crea `backups/pre-import-<nanos>.zip` antes de escribir y se informa su ruta

#### Scenario: Carpeta vacía

- **WHEN** no hay datos previos
- **THEN** la importación no exige un backup y lo indica

#### Scenario: Fallo de escritura

- **WHEN** la escritura de un archivo falla
- **THEN** el archivo previo no queda a medias y el respaldo previo permite recuperarlo

### Requirement: Contenido y privacidad

El respaldo SHALL incluir solo los cuatro JSON de usuario y SHALL NOT incluir EPUB, portadas cacheadas, backups de EPUB ni claves de IA.

#### Scenario: Inspeccionar el ZIP

- **WHEN** el usuario abre el ZIP exportado
- **THEN** solo encuentra el manifiesto y los cuatro JSON permitidos

#### Scenario: Clave de IA

- **WHEN** el usuario exporta sus datos
- **THEN** la clave del llavero no aparece en ningún archivo del ZIP

### Requirement: UI de respaldo

La sección Avanzado SHALL ofrecer exportar e importar, explicar el alcance y advertir que la importación reemplaza los datos actuales.

#### Scenario: Importación correcta

- **WHEN** termina la importación
- **THEN** se muestra un aviso con el resultado y la ruta del backup previo

#### Scenario: Ventana angosta

- **WHEN** la configuración se muestra a 720×600
- **THEN** las acciones de datos se leen y se operan sin desbordar
