//! API pública del lector. `main.rs` es solo un cascarón: toda la lógica vive
//! acá para que los tests de integración puedan ejercitarla sin abrir ventana.

pub mod ai;
pub mod annotations;
pub mod app;
pub mod backup;
pub mod config;
pub mod consulta;
pub mod covers;
pub mod epub;
pub mod epub_edit;
pub mod fonts;
pub mod library;
#[cfg(target_os = "macos")]
pub mod macos;
pub mod markdown;
pub mod search;
pub mod secrets;
pub mod store;
pub mod theme;
pub mod ui;
pub mod update;
pub mod xhtml;
