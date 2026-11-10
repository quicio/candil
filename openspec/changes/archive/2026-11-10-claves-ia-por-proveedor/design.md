## Context

`secrets.rs` usaba `Entry::new(SERVICE, "ai-api-key")` para una única clave.
`load()` no recibía proveedor y `app.rs` no recargaba la clave al cambiar
`ai_provider`. `OPENCODE_API_KEY` se consultaba antes que el llavero sin mirar el
proveedor.

## Goals / Non-Goals

**Goals:**

- Aislar credenciales por proveedor sin cambiar el servicio del llavero.
- Mantener la clave existente mediante una migración única y local.
- Evitar que la variable histórica se filtre a otros proveedores.

**Non-Goals:**

- Validar la clave haciendo una llamada al proveedor.
- Soportar varias claves para un mismo proveedor o rotación automática.
- Exportar claves en el respaldo (siguen excluidas).

## Decisions

### 1. Cuenta derivada del id

`account_for(provider)` devuelve `ai-api-key/<provider>`. El id ya está
normalizado por el catálogo de `ai.rs`; no se usa la etiqueta ni la URL.

### 2. Migración perezosa y única

Al leer una cuenta sin entrada, `migrate_legacy` busca `ai-api-key`, escribe la
clave en la cuenta del proveedor actual y borra la histórica. Solo el primer
proveedor que se cargue la recibe. Si la escritura falla, la sesión igual usa la
clave; si el borrado falla, la entrada vieja queda como respaldo.

### 3. Entorno solo para OpenCode Go

`env_key(provider)` devuelve `None` si el id no es `opencode`. La UI de otro
proveedor muestra su estado real y no la variable de entorno.

### 4. Recarga al cambiar proveedor

El combo de proveedor ya actualiza base URL y modelo; ahora también recarga
`KeyState`, limpia el campo de entrada y cancela una confirmación de borrado
pendiente.

## Risks / Trade-offs

- **Lectura de llavero en el hilo de UI** al cambiar proveedor → operación
  infrecuente y local; el llavero ya se leía al arrancar.
- **Migración ambigua** → la clave histórica era compartida; asignarla al primer
  proveedor cargado es la opción menos mala y no la duplica.
- **Borrado fallido de la entrada vieja** → puede reintentarse; no bloquea el uso.
- **Tests sin llavero real** → se prueba el nombre de cuenta; la integración real
  queda para uso manual.

## Migration Plan

Sin migración de `config.json`. La migración de llavero ocurre en la primera
lectura. Rollback: volver a la cuenta única y dejar las cuentas por proveedor
como entradas huérfanas.
