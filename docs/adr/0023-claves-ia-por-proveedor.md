# ADR-0023: Claves de IA por proveedor

- **Estado:** aceptado
- **Fecha:** 2026-11-10
- **Área:** configuración, IA, seguridad

## Contexto

Candil guardaba una sola clave en el llavero (`account = ai-api-key`) y la
reaplicaba al proveedor que estuviera seleccionado. Al cambiar de OpenAI a
Anthropic o Gemini, la clave anterior seguía cargada y podía enviarse al
proveedor equivocado. `OPENCODE_API_KEY` también se aplicaba a cualquier
proveedor, con el mismo riesgo.

## Decisión

Cada proveedor tiene su propia entrada en el llavero:

- Servicio: `com.quicio.candil`.
- Cuenta: `ai-api-key/<id-del-proveedor>` (`ai-api-key/openai`,
  `ai-api-key/anthropic`, etc.).
- `OPENCODE_API_KEY` solo aplica al proveedor `opencode`; para el resto se ignora.
- Al cambiar el proveedor en Configuración se recarga el estado de la clave y se
  limpia el campo de entrada y la confirmación de borrado.

### Migración

Si la cuenta del proveedor no existe y sí existe la cuenta histórica
`ai-api-key`, Candil mueve esa clave a la cuenta del proveedor actual, borra la
entrada vieja y la usa. Si la escritura falla, la clave sigue disponible para la
sesión y se reintenta en el próximo arranque; si el borrado falla, la entrada
vieja queda como respaldo. Solo la primera cuenta que se cargue recibe la clave
histórica, que era compartida y por lo tanto ambigua.

**Por qué migrar en vez de duplicar:** duplicar la clave histórica a todos los
proveedores volvería a exponer una clave en el proveedor equivocado.

## Alternativas descartadas

- **Una clave por sesión:** simple, pero sigue enviando credenciales cruzadas.
- **Clave única con validación por proveedor:** requiere llamadas de prueba y no
  evita el problema de fondo.
- **Guardar un mapa de claves en `config.json`:** viola la regla de no escribir
  secretos en archivos.
- **Cuentas con nombre del proveedor en el servicio:** separar por servicio
  complica la migración y no aporta seguridad.

## Consecuencias

- Cambiar de proveedor ya no reutiliza la clave anterior; la UI muestra el estado
  de la cuenta correspondiente.
- La primera lectura tras actualizar puede mover la clave histórica. Es una
  operación local, sin red y sin cambiar el valor.
- `OPENCODE_API_KEY` deja de funcionar como override universal; sigue sirviendo
  para OpenCode Go en CI y scripts.
- No se agregan dependencias ni campos a `config.json`; el nombre de la cuenta no
  es secreto.
- Tests cubren el formato de cuenta por proveedor; el llavero real no se toca en
  tests ni capturas (`CANDIL_NO_KEYRING=1`).
