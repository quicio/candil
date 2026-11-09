## Context

`K` con selección llama a `open_consulta`: arma `ConsultaTarget`, elige el tab
(Diccionario si hay una palabra, Traducción si hay varias) y muestra el panel.
El botón “Traducir” llama a `run_consulta`, que respeta `online_enabled` y corre
la petición en un hilo.

## Goals / Non-Goals

**Goals:**

- Un gesto para traducir la selección sin un clic extra.
- Reutilizar `open_consulta`/`run_consulta` y el aviso de online apagado.
- Mantener `K` sin cambios.

**Non-Goals:**

- Atajos configurables, ventana aparte, cambio de proveedor, consulta sin
  selección.

## Decisions

### 1. Shift+K sobre el mismo camino

Al detectar `K` con `shift` y selección, se llama a `open_consulta`, se fuerza
`ConsultaTab::Traduccion` y se llama a `run_consulta(ConsultaKind::Traduccion)`.
La petición es explícita y usa el proveedor/targetlang actuales.

**Por qué Shift+K:** `K` ya es “consultar”; Shift lo convierte en la variante
directa sin agregar otra letra ni chocar con `T`/`S`/`D`/`G`.

**Alternativa descartada:** hacer que `K` ejecute siempre. Cambiaría el
comportamiento actual y podría enviar una consulta cuando el usuario solo quería
ver el panel.

### 2. Online apagado

`run_consulta` ya corta con `consulta_error` si `online_enabled` es falso. El
panel se abre en Traducción y muestra el aviso; no hay petición.

### 3. Documentación

La ayuda de atajos y el tooltip de “Consultar” mencionan ⇧K; el README lo lista.

## Risks / Trade-offs

- **Doble acción accidental** → solo se dispara con selección y Shift; `K` sigue
  siendo seguro.
- **Proveedor sin configurar** → el panel muestra el error existente; no se
  agrega manejo nuevo.
- **Sin tests de teclado** → la lógica reutiliza funciones ya probadas; se
  verifica en la app.

## Migration Plan

Sin datos ni dependencias. Rollback: quitar la rama de `shift` y la entrada de
ayuda.
