# ADR-0027: Enviar EPUB a un dispositivo por copia

- **Estado:** aceptado
- **Fecha:** 2026-11-12
- **Área:** biblioteca, dispositivos

## Contexto

El usuario quiere pasar un EPUB del lector a un Kobo, un Kindle u otro lector.
Kobo lee EPUB nativo por USB; Kindle normalmente no lee EPUB por USB (necesita
conversión o Send to Kindle). No hay una API local uniforme para “enviar a
dispositivo”.

## Decisión

Agregar **Enviar a dispositivo…** al menú contextual de cada tarjeta:

- Detecta un Kobo montado buscando una carpeta `.kobo` en volúmenes comunes
  (`/Volumes` en macOS, `/media`/`/run/media` en Linux, letras de unidad en
  Windows) y ofrece **Enviar a Kobo**.
- En Kobo copia a `Books/` si existe; si no, a la raíz. El Kobo indexa EPUB.
- **Elegir carpeta o dispositivo…** abre un selector para cualquier lector o
  tarjeta montada.
- Nunca sobrescribe: si el nombre existe, usa `libro (2).epub`, etc.
- Es una **copia**, no un movimiento: el EPUB de la biblioteca queda intacto.
- No se borra nada del dispositivo.

### Kindle

No se copia EPUB a un Kindle como si fuera compatible. El diálogo muestra una
nota: Kindle normalmente no lee EPUB por USB; hay que usar Send to Kindle o un
archivo convertido. La carpeta genérica permite copiar un archivo ya convertido,
pero Candil no convierte formatos.

## Alternativas descartadas

- **Enviar por email/SMTP a Kindle:** requiere credenciales de correo, la
  dirección `@kindle.com` y manejo de rebotes; mala experiencia y secretos extra.
- **API de Send to Kindle:** no hay una API pública estable para escritorio.
- **Conversión integrada a AZW3/MOBI:** exige un conversor pesado y cambia el
  alcance; se puede hacer externamente y copiar el resultado.
- **Copiar automáticamente al conectar:** sorpresivo y puede llenar el
  dispositivo; se prefiere una acción explícita.

## Consecuencias

- Funciona con Kobo y con cualquier lector que se monte como carpeta.
- Kindle queda explícitamente fuera del copiado directo de EPUB.
- No se agregan dependencias ni credenciales.
- El espacio del dispositivo puede llenarse; el error de copia se muestra y no se
  toca el original.
- Tests cubren detección de `.kobo` y nombres únicos.
