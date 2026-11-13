# ADR-0026: Google Drive como carpeta local

- **Estado:** aceptado
- **Fecha:** 2026-11-12
- **Área:** biblioteca, integraciones

## Contexto

El usuario quiere leer EPUB que ya tiene en Google Drive. Candil indexa carpetas
locales y no copia archivos (ADR-0007). Existen dos caminos:

1. Usar la carpeta que Google Drive para escritorio mantiene sincronizada.
2. Integrar la API de Google Drive con OAuth, descarga y caché propia.

La segunda opción agrega credenciales, consentimiento, tokens, cuotas, manejo de
conflictos, caché y una dependencia HTTP nueva. El disco de la máquina está
limitado y la app no necesita otra cuenta para leer archivos que ya están en el
disco.

## Decisión

Candil ofrece **Añadir Google Drive…** cuando detecta una carpeta local de Google
Drive para escritorio:

- macOS: `~/Library/CloudStorage/GoogleDrive-*/My Drive`.
- Otros: `~/Google Drive` como candidato.
- El botón solo aparece si la carpeta existe; en el menú **Carpetas** siempre está
  la opción y, si no se detecta, explica cómo instalarlo o usar **Añadir
  carpeta…**.
- La acción abre un selector en la raíz de Drive para elegir una **subcarpeta**;
  no indexa `My Drive` completo por accidente.

La carpeta se indexa como cualquier otra: ruta canónica, escaneo incremental y
portadas en `covers/`. No se copian EPUB, no se guardan credenciales y no se
llama a la API de Google.

### Archivos solo en la nube

Drive para escritorio puede mantener archivos como placeholders. Al escanear o
abrir un EPUB, Drive puede descargarlo y consumir disco. Candil avisa al agregar
la carpeta y recomienda marcarla como **disponible sin conexión**. Detectar
placeholders y descargar solo lo necesario requiere OAuth/API y una caché propia;
queda fuera de esta decisión.

### Búsqueda

La búsqueda full-text deja de depender de Enter a ciegas: la barra tiene un
**toggle Contenido**. Con el toggle activo, Enter busca dentro del texto y el
panel de resultados se abre a un ancho relativo a la ventana (hasta 920 px), no
al ancho fijo pequeño anterior.

## Alternativas descartadas

- **OAuth + Drive API ahora:** funcional para cuentas sin cliente de escritorio,
  pero requiere un proyecto de Google Cloud, client ID, redirect URI, tokens,
  caché de descarga y un ADR de privacidad. Se pospone.
- **Descargar todo a la biblioteca:** duplica archivos y choca con ADR-0007 y con
  el poco disco libre.
- **Depender de `rclone`:** agrega un binario externo y credenciales fuera del
  llavero de Candil.
- **Solo Enter sin toggle:** el usuario no descubre la búsqueda en contenido.

## Consecuencias

- Google Drive funciona si el usuario tiene Drive para escritorio y la carpeta
  está sincronizada; si no, el mensaje lo explica.
- Los archivos se leen desde la ruta local; cambios de Drive se reflejan en el
  próximo escaneo.
- La integración OAuth queda como una decisión futura, no como deuda oculta.
- No se agregan dependencias ni secretos.
