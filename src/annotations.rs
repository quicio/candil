//! Destacados y notas por libro (`annotations.json`).
//!
//! Archivo aparte de `library.json` para no hinchar el índice de la estantería.
//! Los offsets son de caracteres dentro de `Block::text()`, no píxeles: cambiar
//! tipografía o ancho no mueve el destacado. Si el bloque cambió, el lector
//! reancla por la cita guardada.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::library::now;
use crate::store;

pub const SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Annotation {
    pub id: u64,
    /// Ruta canónica del EPUB.
    pub book: String,
    /// Índice del spine.
    pub chapter: usize,
    /// Índice en `Vec<Block>` del bloque inicial.
    pub block: usize,
    /// Offset en caracteres dentro de `Block::text()` del bloque inicial.
    pub start: usize,
    /// Offset final dentro del bloque final.
    pub end: usize,
    /// Bloque final cuando el rango cruza párrafos. `None` = mismo bloque
    /// (compatible con el formato v1).
    pub end_block: Option<usize>,
    /// Texto seleccionado en cada bloque cubierto, en orden. Permite reanclar
    /// aunque los offsets hayan cambiado.
    pub parts: Vec<String>,
    /// Texto seleccionado completo, para mostrar y exportar. Las partes se unen
    /// con `\n`; la fuente de verdad para reanclar es `parts`.
    pub quote: String,
    /// Vacío = solo destacado.
    pub note: String,
    /// Reservado para varios colores; 0 en el MVP.
    pub color: u8,
    pub created: i64,
}

impl Annotation {
    /// Bloque final efectivo.
    pub fn end_block(&self) -> usize {
        self.end_block.unwrap_or(self.block)
    }

    /// `true` si el rango empieza y termina en bloques distintos.
    pub fn is_multi_block(&self) -> bool {
        self.end_block() > self.block
    }

    /// Cantidad de párrafos con texto que cubre el rango.
    pub fn paragraph_count(&self) -> usize {
        self.parts
            .iter()
            .filter(|part| !part.trim().is_empty())
            .count()
            .max(1)
    }

    /// Primera parte de la cita, para saltar al inicio del rango.
    pub fn first_part(&self) -> String {
        self.parts
            .iter()
            .find(|part| !part.is_empty())
            .cloned()
            .or_else(|| self.quote.split('\n').next().map(str::to_string))
            .unwrap_or_default()
    }

    /// Normaliza un rango recién cargado o creado: completa `parts` desde la
    /// cita y descarta un `end_block` redundante.
    pub fn normalize(&mut self) {
        if self.parts.is_empty() && !self.quote.is_empty() {
            self.parts = vec![self.quote.clone()];
        }
        if self.end_block == Some(self.block) {
            self.end_block = None;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Annotations {
    pub schema_version: u32,
    pub items: Vec<Annotation>,
}

impl Default for Annotations {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            items: Vec::new(),
        }
    }
}

impl Annotations {
    pub fn file() -> PathBuf {
        store::data_dir().join("annotations.json")
    }

    pub fn load() -> (Self, Option<String>) {
        Self::load_from(&Self::file())
    }

    pub fn load_from(path: &Path) -> (Self, Option<String>) {
        let (mut loaded, warning) = store::load_json_with_report::<Self>(path, "annotations.json");
        loaded.migrate();
        (loaded, warning)
    }

    pub fn save(&self) -> Result<(), String> {
        self.save_to(&Self::file())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), String> {
        store::save_json_atomic(path, self)
    }

    pub fn for_book<'a>(&'a self, book: &'a str) -> impl Iterator<Item = &'a Annotation> + 'a {
        self.items.iter().filter(move |a| a.book == book)
    }

    pub fn get(&self, id: u64) -> Option<&Annotation> {
        self.items.iter().find(|a| a.id == id)
    }

    pub fn get_mut(&mut self, id: u64) -> Option<&mut Annotation> {
        self.items.iter_mut().find(|a| a.id == id)
    }

    /// Agrega y devuelve el id asignado (max+1, sin colisiones tras borrar).
    pub fn add(&mut self, mut annotation: Annotation) -> u64 {
        let id = self.items.iter().map(|a| a.id).max().unwrap_or(0) + 1;
        annotation.id = id;
        annotation.normalize();
        if annotation.created == 0 {
            annotation.created = now();
        }
        self.items.push(annotation);
        id
    }

    /// Migra en memoria un archivo de una versión anterior. No escribe nada:
    /// el archivo se reescribe recién en la próxima mutación.
    pub fn migrate(&mut self) {
        // Normalizar es idempotente y también cubre archivos sin
        // `schema_version` o escritos por versiones intermedias.
        for annotation in &mut self.items {
            annotation.normalize();
        }
        if self.schema_version < SCHEMA_VERSION {
            self.schema_version = SCHEMA_VERSION;
        }
    }

    pub fn remove(&mut self, id: u64) -> bool {
        let before = self.items.len();
        self.items.retain(|a| a.id != id);
        self.items.len() != before
    }

    pub fn update_note(&mut self, id: u64, note: &str) -> bool {
        match self.get_mut(id) {
            Some(a) => {
                a.note = note.to_string();
                true
            }
            None => false,
        }
    }

    pub fn update_color(&mut self, id: u64, color: u8) -> bool {
        match self.get_mut(id) {
            Some(a) => {
                a.color = color;
                true
            }
            None => false,
        }
    }

    /// Migra las anotaciones de una ruta a otra (por ejemplo, al reubicar un
    /// EPUB). Devuelve cuántas entradas cambiaron.
    pub fn rekey(&mut self, old: &str, new: &str) -> usize {
        let mut changed = 0;
        for annotation in &mut self.items {
            if annotation.book == old {
                annotation.book = new.to_string();
                changed += 1;
            }
        }
        changed
    }

    /// Anotaciones del libro ordenadas por capítulo, bloque y offset.
    pub fn sorted_for_book<'a>(&'a self, book: &'a str) -> Vec<&'a Annotation> {
        let mut items: Vec<&Annotation> = self.for_book(book).collect();
        items.sort_by_key(|a| (a.chapter, a.block, a.start, a.id));
        items
    }

    /// Todas las anotaciones, ordenadas por libro, capítulo, bloque y offset.
    pub fn sorted_all(&self) -> Vec<&Annotation> {
        let mut items: Vec<&Annotation> = self.items.iter().collect();
        items.sort_by_key(|a| (a.book.as_str(), a.chapter, a.block, a.start, a.id));
        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir =
            std::env::temp_dir().join(format!("lector-ann-{tag}-{}-{stamp}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn annotation(book: &str, block: usize, quote: &str) -> Annotation {
        Annotation {
            book: book.into(),
            chapter: 2,
            block,
            start: 4,
            end: 4 + quote.chars().count(),
            quote: quote.into(),
            ..Annotation::default()
        }
    }

    #[test]
    fn roundtrip_y_archivo_atomico() {
        let dir = temp_dir("roundtrip");
        let path = dir.join("annotations.json");
        let mut all = Annotations::default();
        let mut a = annotation("/libro.epub", 3, "frase");
        a.color = 2;
        let id = all.add(a);
        all.update_note(id, "mi nota");
        all.save_to(&path).expect("guardar");

        assert!(!dir.join("annotations.json.tmp").exists());
        let (loaded, warning) = Annotations::load_from(&path);
        assert!(warning.is_none());
        assert_eq!(loaded.schema_version, SCHEMA_VERSION);
        assert_eq!(loaded.items.len(), 1);
        let a = &loaded.items[0];
        assert_eq!(a.id, id);
        assert_eq!(a.quote, "frase");
        assert_eq!(a.note, "mi nota");
        assert_eq!(a.color, 2);
        assert_eq!(a.book, "/libro.epub");

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn ids_unicos_y_filtro_por_libro() {
        let mut all = Annotations::default();
        let a = all.add(annotation("/a.epub", 1, "uno"));
        let b = all.add(annotation("/b.epub", 2, "dos"));
        let c = all.add(annotation("/a.epub", 3, "tres"));
        assert!(a < b && b < c);

        let de_a: Vec<u64> = all.for_book("/a.epub").map(|x| x.id).collect();
        assert_eq!(de_a, vec![a, c]);
        let ordenadas = all.sorted_for_book("/a.epub");
        assert_eq!(ordenadas.len(), 2);
        assert_eq!(ordenadas[0].id, a);
    }

    #[test]
    fn rekey_mueve_anotaciones_al_nuevo_path() {
        let mut all = Annotations::default();
        all.add(annotation("/old.epub", 1, "uno"));
        let mut multi = annotation("/old.epub", 2, "dos");
        multi.end_block = Some(4);
        multi.parts = vec!["dos".into(), "tres".into(), "cuatro".into()];
        all.add(multi);
        all.add(annotation("/otro.epub", 3, "tres"));

        assert_eq!(all.rekey("/old.epub", "/new.epub"), 2);
        assert_eq!(all.for_book("/new.epub").count(), 2);
        assert_eq!(all.for_book("/old.epub").count(), 0);
        assert_eq!(all.for_book("/otro.epub").count(), 1);
        let moved = all
            .for_book("/new.epub")
            .find(|a| a.block == 2)
            .expect("rango multi-bloque");
        assert_eq!(moved.end_block(), 4);
        assert_eq!(moved.parts.len(), 3);
    }

    #[test]
    fn borrar_actualizar_y_reusar_id() {
        let mut all = Annotations::default();
        let id = all.add(annotation("/libro.epub", 1, "uno"));
        assert!(all.remove(id));
        assert!(!all.remove(id));
        assert!(all.items.is_empty());

        let nuevo = all.add(annotation("/libro.epub", 2, "dos"));
        assert_eq!(nuevo, 1, "el id vuelve a empezar tras borrar todo");
        assert!(all.update_note(nuevo, "nota"));
        assert_eq!(all.get(nuevo).unwrap().note, "nota");
        assert!(all.update_color(nuevo, 2));
        assert_eq!(all.get(nuevo).unwrap().color, 2);
        assert!(!all.update_color(999, 1));
        assert!(!all.update_note(999, "x"));
    }

    #[test]
    fn orden_global_por_libro_y_posicion() {
        let mut all = Annotations::default();
        let b2 = all.add(annotation("/b.epub", 1, "b"));
        let a2 = all.add(annotation("/a.epub", 2, "a2"));
        let a1 = all.add(annotation("/a.epub", 1, "a1"));
        let orden: Vec<u64> = all.sorted_all().iter().map(|a| a.id).collect();
        assert_eq!(orden, vec![a1, a2, b2]);
        assert_eq!(all.sorted_for_book("/a.epub").len(), 2);
    }

    #[test]
    fn json_corrupto_hace_backup_y_arranca_vacio() {
        let dir = temp_dir("corrupt");
        let path = dir.join("annotations.json");
        std::fs::write(&path, "{ esto no es json").unwrap();

        let (all, warning) = Annotations::load_from(&path);
        assert_eq!(all, Annotations::default());
        assert!(warning.is_some());
        assert!(!path.exists());
        let backups = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains("corrupt"))
            .count();
        assert_eq!(backups, 1);

        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn json_sin_campos_usa_defaults() {
        let a: Annotation = serde_json::from_str(r#"{"book":"/x.epub","quote":"hola"}"#).unwrap();
        assert_eq!(a.id, 0);
        assert_eq!(a.color, 0);
        assert_eq!(a.note, "");
        assert!(a.end_block.is_none());
        assert!(a.parts.is_empty());
        let all: Annotations = serde_json::from_str("{}").unwrap();
        assert_eq!(all.schema_version, SCHEMA_VERSION);
        assert!(all.items.is_empty());
    }

    #[test]
    fn migra_v1_a_v2_sin_perder_anotaciones() {
        let raw = r#"{
            "schema_version": 1,
            "items": [
                {"id": 1, "book": "/a.epub", "chapter": 2, "block": 3,
                 "start": 4, "end": 9, "quote": "frase", "note": "", "color": 1}
            ]
        }"#;
        let mut all: Annotations = serde_json::from_str(raw).unwrap();
        all.migrate();
        assert_eq!(all.schema_version, SCHEMA_VERSION);
        assert_eq!(all.items.len(), 1);
        assert_eq!(all.items[0].end_block, None);
        assert_eq!(all.items[0].parts, vec!["frase"]);
        assert_eq!(all.items[0].first_part(), "frase");
        assert_eq!(all.items[0].paragraph_count(), 1);

        // Migrar dos veces no duplica ni cambia nada.
        all.migrate();
        assert_eq!(all.items[0].parts, vec!["frase"]);
    }

    #[test]
    fn roundtrip_multi_bloque_conserva_partes_y_extremos() {
        let dir = temp_dir("multi");
        let path = dir.join("annotations.json");
        let mut all = Annotations::default();
        let id = all.add(Annotation {
            book: "/libro.epub".into(),
            chapter: 1,
            block: 2,
            start: 4,
            end: 6,
            end_block: Some(4),
            parts: vec!["uno".into(), "dos".into(), "tres".into()],
            quote: "uno\ndos\ntres".into(),
            ..Annotation::default()
        });
        all.save_to(&path).unwrap();

        let (loaded, warning) = Annotations::load_from(&path);
        assert!(warning.is_none());
        let a = loaded.get(id).unwrap();
        assert_eq!(a.block, 2);
        assert_eq!(a.start, 4);
        assert_eq!(a.end_block(), 4);
        assert_eq!(a.end, 6);
        assert!(a.is_multi_block());
        assert_eq!(a.paragraph_count(), 3);
        assert_eq!(a.parts, vec!["uno", "dos", "tres"]);
        assert_eq!(a.quote, "uno\ndos\ntres");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn normaliza_end_block_redundante_y_completa_partes() {
        let mut a = Annotation {
            book: "/x.epub".into(),
            block: 7,
            end_block: Some(7),
            quote: "hola".into(),
            ..Annotation::default()
        };
        a.normalize();
        assert_eq!(a.end_block, None);
        assert!(!a.is_multi_block());
        assert_eq!(a.parts, vec!["hola"]);
    }
}
