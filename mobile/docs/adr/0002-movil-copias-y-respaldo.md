# ADR-0002: Copias locales y respaldo compatible en móvil

- **Estado:** aceptado
- **Fecha:** 2026-11-16
- **Área:** almacenamiento, biblioteca, respaldo

## Contexto

En escritorio, Candil indexa rutas sin copiar EPUB (ADR-0007). En Android e iOS
el acceso a archivos externos es temporal y puede revocarse; además no existen
carpetas vigiladas equivalentes a Linux/macOS.

Los respaldos de escritorio incluyen rutas absolutas en `library.json` y las
anotaciones usan esa ruta como identidad.

## Decisión

1. **Importar copia el EPUB al sandbox** en `books/<sha256>.epub`. El hash evita
   duplicados y el archivo queda disponible sin permisos externos.
2. **La identidad sigue siendo `path`**, pero ahora es la ruta del sandbox.
3. Al importar un respaldo, las entradas cuya ruta no existe en el móvil quedan
   `missing: true` y conservan la ruta de escritorio en `origin_path`.
4. Al importar el EPUB correspondiente, se busca por nombre de archivo original,
   título/autor o entrada faltante; se actualiza `path`, se conserva el progreso
   y se reasignan las anotaciones de la ruta vieja a la nueva.
5. Los cinco JSON mantienen el esquema de escritorio. `origin_path` es un campo
   adicional que escritorio ignora al deserializar.
6. El respaldo móvil reutiliza el manifiesto y los checksums de escritorio; no
   incluye EPUB, portadas ni claves.

## Alternativas descartadas

- **Guardar rutas externas y pedir permiso en cada apertura:** frágil en iOS y
  molesto en Android.
- **Renombrar las rutas al importar sin `origin_path`:** se pierden las
  anotaciones hasta que el usuario las busque a mano.
- **Formato de datos móvil distinto:** rompe la continuidad con escritorio y
  duplica migraciones.
