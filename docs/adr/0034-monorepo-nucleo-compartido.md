# ADR-0034: Monorepo con núcleo portable compartido

- **Estado:** aceptado
- **Fecha:** 2026-11-16
- **Área:** arquitectura, escritorio, móvil

## Contexto

Candil escritorio (egui) y Candil Mobile (Flutter) compartían por copia el
parser EPUB, el modelo de bloques y la búsqueda. Cada corrección había que
replicarla en dos lugares y los tests no cubrían ambos consumidores.

El primer intento de extracción creó un repositorio `candil-core` separado y un
`candil-mobile` separado. Eso evita duplicar código, pero obliga a coordinar
tres repos y deja el núcleo fuera del repo que le da sentido.

## Decisión

Un **monorepo** con tres paquetes y una regla de dependencia explícita:

```
candil/
  Cargo.toml                 workspace + paquete de escritorio
  src/                       UI egui y adaptadores de escritorio
  crates/candil-core/        núcleo portable
  mobile/                    app Flutter + FFI
    lib/                     UI Flutter
    rust/                    adaptador C/JSON (cdylib/staticlib)
  openspec/                  specs compartidas
  docs/                      arquitectura, handoff y ADR
```

Dependencias:

```
candil (egui) ──────────────┐
                            ├──► candil-core (sin UI ni plataforma)
candil-mobile-core (FFI) ───┘
        ▲
        │ C/JSON
    Flutter
```

Reglas:

1. `candil-core` no conoce egui, Flutter, `dirs`, `keyring`, `rfd` ni rutas de
   usuario. Solo recibe bytes/rutas y devuelve dominio.
2. La UI de escritorio y la de móvil son carriles independientes. No se intenta
   compartir egui con Flutter.
3. El acceso a archivos y almacenamiento es distinto por plataforma: escritorio
   indexa rutas; móvil copia al sandbox. Los formatos JSON siguen siendo el
   contrato común.
4. Un cambio de parser o búsqueda se prueba con `cargo test --workspace` y se
   verifica en ambos consumidores antes de cerrar la tarea.
5. El FFI es un adaptador de borde. La app Flutter no importa detalles de Rust;
   consume JSON y punteros opacos.

## Consecuencias

- Se elimina la copia de `epub.rs`, `xhtml.rs` y `search.rs`.
- Un solo `Cargo.lock` y un solo `target/` para escritorio, núcleo y FFI.
- El build Android sigue siendo separado (Flutter/Gradle) y se documenta en
  `mobile/README.md`.
- Publicar la app móvil requiere incluir `crates/candil-core` en el mismo repo;
  ya no es un proyecto que se pueda clonar solo.
- iOS sigue requiriendo macOS/Xcode; el monorepo no cambia esa restricción.
