//! Núcleo de Candil Mobile: parser EPUB, bloques y búsqueda.
//!
//! La UI Flutter consume este crate por una API C mínima basada en JSON. Así
//! Android e iOS comparten exactamente el mismo parser que la app de escritorio.

pub use candil_core::{epub, search, xhtml};

pub mod ffi;
