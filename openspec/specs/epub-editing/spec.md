# epub-editing Specification

## Purpose
Definir cómo Candil lee, modifica y guarda metadatos y portada de un EPUB sin corromper el archivo ni perder el progreso, los ajustes o las anotaciones asociadas.
## Requirements
### Requirement: Lectura del estado actual

El editor SHALL mostrar el título, autor, idioma y portada actuales leídos del OPF y del ZIP.

#### Scenario: EPUB con portada
- **WHEN** el usuario abre el editor de un libro con `properties="cover-image"`
- **THEN** se muestran los metadatos y la preview de la portada

#### Scenario: EPUB sin portada
- **WHEN** el libro no declara portada
- **THEN** el editor muestra “Sin portada” y permite elegir una

### Requirement: Edición de metadatos

El sistema SHALL actualizar `dc:title`, `dc:creator` y `dc:language` del OPF, preservando el resto del documento y las entradas del ZIP.

#### Scenario: Corregir título y autor
- **WHEN** el usuario cambia título/autor e guarda
- **THEN** al reabrir el EPUB `Book::open` devuelve los nuevos valores

#### Scenario: Caracteres especiales
- **WHEN** el título o el autor contienen `&`, `<` o `>`
- **THEN** el OPF queda con entidades XML válidas y el EPUB abre

### Requirement: Agregar o reemplazar portada

El sistema SHALL aceptar JPG, PNG o WebP, ajustar la imagen a un máximo de 1600×2400 sin agrandar y agregar/reemplazar el ítem `cover-image` y el `meta name="cover"`.

#### Scenario: Agregar portada
- **WHEN** el EPUB no tiene portada y el usuario elige una imagen
- **THEN** el nuevo EPUB declara un ítem `cover-image`, incluye el archivo y `Book::cover_bytes()` lo devuelve

#### Scenario: Reemplazar portada
- **WHEN** el EPUB ya tiene portada y el usuario elige otra
- **THEN** el archivo de portada se reemplaza y la preview del editor se actualiza

#### Scenario: Imagen inválida
- **WHEN** el archivo elegido no es una imagen soportada
- **THEN** el editor muestra un error y no modifica el EPUB

### Requirement: Backup y reemplazo seguro

Antes de reemplazar el original, el sistema SHALL copiarlo a `data_dir/backups/` y SHALL validar el EPUB nuevo; si algo falla, el original SHALL quedar intacto.

#### Scenario: Guardado correcto
- **WHEN** la edición termina correctamente
- **THEN** existe un backup y el original apunta al EPUB nuevo validado

#### Scenario: Fallo de validación
- **WHEN** el EPUB temporal no abre o la portada no queda accesible
- **THEN** se borra el temporal, se muestra el error y el original no se toca

#### Scenario: Fallo de backup
- **WHEN** no se puede crear el backup
- **THEN** no se reemplaza el original

### Requirement: Integridad del ZIP

El EPUB resultante SHALL conservar todas las entradas, SHALL escribir `mimetype` primero y sin compresión, y SHALL mantener el OPF y la portada en las rutas declaradas.

#### Scenario: Abrir el EPUB editado
- **WHEN** se inspecciona el ZIP resultante
- **THEN** la primera entrada es `mimetype` con compresión `Stored` y las demás entradas siguen presentes

### Requirement: Actualización de la biblioteca

Después de guardar, la entrada de biblioteca SHALL actualizar título, autor, idioma, tamaño, mtime y portada cacheada, preservando progreso, ajustes, fechas y la ruta canónica.

#### Scenario: Libro con progreso y notas
- **WHEN** se edita un libro que tenía progreso, ajustes y anotaciones
- **THEN** esos datos siguen asociados al mismo libro después de la edición

### Requirement: UI de edición

El editor SHALL ser un modal con campos, preview, elección de imagen, error visible, advertencia de reescritura y botones Guardar/Cancelar; SHALL funcionar a 980×780 y 720×600.

#### Scenario: Cancelar
- **WHEN** el usuario cancela o cierra el editor
- **THEN** no se escribe ningún archivo

#### Scenario: Error de guardado
- **WHEN** el guardado falla
- **THEN** el modal queda abierto con el error y el original no cambia

