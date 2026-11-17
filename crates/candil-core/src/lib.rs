//! Núcleo portable de Candil.
//!
//! Regla de dependencia: este crate no conoce egui, Flutter, rutas de
//! plataforma, llaveros, actualizaciones ni almacenamiento de usuario. Solo
//! transforma bytes de EPUB en bloques y búsquedas.
//!
//! - [`xhtml`] es el modelo de dominio/lectura (bloques y runs).
//! - [`epub`] es el adaptador de entrada: ZIP → OPF → capítulos.
//! - [`search`] es la lógica de aplicación sobre los bloques.
//!
//! Los bordes viven afuera: Candil escritorio lo consume desde egui y
//! `candil-mobile/rust` lo envuelve en una API C para Flutter.

#![forbid(unsafe_code)]

pub mod epub;
pub mod search;
pub mod xhtml;
