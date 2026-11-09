# ADR-0019: Respaldo local de datos en un ZIP validado

- **Estado:** aceptado
- **Fecha:** 2026-11-09
- **Área:** datos de usuario, portabilidad, seguridad

## Contexto

La biblioteca, el progreso, las anotaciones, el historial y los ajustes viven en
cuatro JSON dentro del directorio de datos. No hay forma de exportarlos ni de
moverlos a otra Mac; un fallo de disco los pierde. Los EPUB son externos y no
deben copiarse. La clave de IA vive en el llavero y nunca debe entrar en un
respaldo.

## Decisión

### Formato

Un ZIP con:

- `candil-backup.json`: manifiesto con `format: 1`, `app_version`, `created` y
  una entrada por archivo (`name`, `size`, `sha256`).
- `config.json`, `library.json`, `annotations.json`, `consultas.json`.

No incluye EPUB, `covers/`, `backups/` de EPUB ni claves.

### Exportación

Se leen los cuatro archivos existentes, se calculan sus checksums, se escribe el
manifiesto y luego los archivos con `zip` (deflate). Si no hay datos, falla. Si
el destino es uno de los archivos de datos, se rechaza para no sobrescribir la
fuente.

### Importación

1. Leer el manifiesto; rechazar formatos desconocidos.
2. Para cada entrada: nombre en lista blanca, tamaño máximo 64 MB, tamaño
   declarado, SHA-256 y parseo del JSON con el tipo correspondiente.
3. Si hay datos actuales, exportar un backup a
   `data_dir/backups/pre-import-<nanos>.zip`.
4. Escribir cada archivo con `store::write_atomic` (temporal + rename).
5. Recargar `Config`, `Library`, `Annotations` y `ConsultaHistory`; volver a la
   biblioteca y lanzar un escaneo incremental.

Cualquier fallo de validación ocurre antes de escribir. Un fallo de escritura
deja el backup previo como recuperación.

### UI

Configuración → Avanzado → Datos: “Exportar datos…” e “Importar datos…”, con la
aclaración de que no incluye EPUB/portadas/claves y que importar reemplaza los
datos actuales. Al terminar la importación se muestra un aviso con la ruta del
backup previo.

## Alternativas descartadas

- **Copiar el directorio de datos a mano:** no es portable ni verificable; el
  usuario tiene que saber qué archivos importan.
- **Incluir EPUB y portadas:** convierte el respaldo en gigabytes y duplica la
  biblioteca; los EPUB ya viven en las carpetas del usuario.
- **Nube o sincronización:** contradice el enfoque local-first y agrega cuentas,
  conflictos y privacidad.
- **Cifrar el ZIP ahora:** requeriría gestionar una contraseña o clave; el
  archivo no contiene secretos y se puede cifrar con herramientas del sistema si
  el usuario lo desea.
- **Confiar en el manifiesto sin validar:** un ZIP manipulado podría escribir
  rutas arbitrarias o datos corruptos.
- **Reiniciar la app tras importar:** más simple, pero peor experiencia; la
  recarga en caliente es acotada y verificable.

## Consecuencias

- El respaldo es chico (JSON), portable y auditable.
- Las rutas de biblioteca de otra Mac pueden quedar `missing`; el escaneo
  posterior marca lo que no existe y el usuario reindexa sus carpetas.
- Importar reemplaza ajustes, incluido el proveedor de IA; la clave sigue en el
  llavero local.
- `pre-import-*.zip` permite deshacer una importación incorrecta.
- No hay backup automático programado; el usuario decide cuándo exportar.
- Tests cubren export/import, backup previo, checksum y lista blanca.
