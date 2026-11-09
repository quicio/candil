# ADR-0012: API key de IA en el llavero del sistema

- **Estado:** aceptado
- **Fecha:** 2026-11-08
- **Área:** configuración, IA, seguridad

## Contexto

La clave de IA vivía solo en `OPENCODE_API_KEY`. El usuario quiere configurarla
desde la app y que quede cifrada en reposo. Guardarla en `config.json` en texto
plano es inaceptable; implementar criptografía propia o un archivo con contraseña
maestra agrega fricción y superficie de error.

## Decisión

Usar el llavero nativo del sistema a través de `keyring` 4.x:

- macOS: Keychain Services.
- Linux: Secret Service (D-Bus).
- Windows: Credential Manager.

La entrada es `service = com.quicio.candil`, `account = ai-api-key`. La UI de
Configuración permite guardar, reemplazar y borrar la clave; el valor nunca se
muestra, no se escribe en `config.json`, `consultas.json`, logs ni URLs.

`OPENCODE_API_KEY` sigue teniendo prioridad sobre el llavero para CI, scripts y
pruebas. Si el llavero no está disponible (Linux sin Secret Service), la UI
muestra el error y la variable de entorno sigue funcionando.

## Alternativas descartadas

- **Texto plano en `config.json`:** incumple el requisito de cifrado.
- **Archivo cifrado con contraseña maestra:** portable, pero obliga a introducir
  una contraseña al abrir y a gestionar su recuperación.
- **Cifrado con clave derivada de la máquina:** no pide contraseña, pero la
  protección real es limitada y da una falsa sensación de seguridad.
- **CLI `security` / `secret-tool`:** requiere código distinto por plataforma y
  manejo de secretos en argv/stdin.

## Consecuencias

- Nueva dependencia `keyring` (MIT OR Apache-2.0), multiplataforma y mantenida.
- El binario `dist` pasa de 6.763.856 B a 6.818.144 B (+54.288 B); sigue por
  debajo del presupuesto de 8 MB.
- macOS puede pedir permiso de Keychain la primera vez que se lee o guarda.
- Los tests no tocan el llavero real; las capturas usan
  `CANDIL_NO_KEYRING=1` porque su `HOME` es falso.
- Gemini envía la clave en el header `x-goog-api-key`, nunca en la query.
