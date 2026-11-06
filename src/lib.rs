//! API pública del lector. `main.rs` es solo un cascarón: toda la lógica vive
//! acá para que los tests de integración puedan ejercitarla sin abrir ventana.

pub mod app;
pub mod config;
pub mod covers;
pub mod epub;
pub mod fonts;
pub mod library;
pub mod store;
pub mod theme;
pub mod xhtml;
