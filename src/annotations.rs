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

pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Annotation {
    pub id: u64,
    /// Ruta canónica del EPUB.
    pub book: String,
    /// Índice del spine.
    pub chapter: usize,
    /// Índice en `Vec<Block>`.
    pub block: usize,
    /// Offset en caracteres dentro de `Block::text()`.
    pub start: usize,
    pub end: usize,
    /// Texto seleccionado, para mostrar y reanclar.
    pub quote: String,
    /// Vacío = solo destacado.
    pub note: String,
    /// Reservado para varios colores; 0 en el MVP.
    pub color: u8,
    pub created: i64,
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
        store::load_json_with_report(path, "annotations.json")
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
        if annotation.created == 0 {
            annotation.created = now();
        }
        self.items.push(annotation);
        id
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
        all.add(annotation("/old.epub", 2, "dos"));
        all.add(annotation("/otro.epub", 3, "tres"));

        assert_eq!(all.rekey("/old.epub", "/new.epub"), 2);
        assert_eq!(all.for_book("/new.epub").count(), 2);
        assert_eq!(all.for_book("/old.epub").count(), 0);
        assert_eq!(all.for_book("/otro.epub").count(), 1);
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
        let all: Annotations = serde_json::from_str("{}").unwrap();
        assert_eq!(all.schema_version, SCHEMA_VERSION);
        assert!(all.items.is_empty());
    }
}
