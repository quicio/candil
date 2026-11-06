//! Búsqueda efímera dentro de un libro.
//!
//! El índice se construye la primera vez que se busca y vive solo en memoria:
//! no se persiste en `library.json` (crecería con el texto de cada libro). La
//! normalización es 1:1 en caracteres (minúsculas + sin acentos), así que los
//! offsets del texto normalizado sirven directamente para resaltar y saltar.
//! No hay stemming, sinónimos ni regex: substring, como pide la spec 40.

use crate::epub::Book;

/// Tope de resultados para que una palabra común no congele la UI.
pub const MAX_RESULTS: usize = 200;

#[derive(Debug, Clone, PartialEq)]
pub struct BlockText {
    /// Índice en el `Vec<Block>` original (no en la lista filtrada).
    pub block: usize,
    pub original: String,
    pub normalized: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChapterIndex {
    pub title: String,
    pub blocks: Vec<BlockText>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SearchIndex {
    pub chapters: Vec<ChapterIndex>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub chapter: usize,
    pub block: usize,
    /// Offsets en caracteres dentro de `Block::text()`.
    pub start: usize,
    pub end: usize,
    pub before: String,
    pub matched: String,
    pub after: String,
}

pub fn build(book: &mut Book) -> SearchIndex {
    let mut chapters = Vec::with_capacity(book.spine.len());
    for index in 0..book.spine.len() {
        let title = book.chapter_title(index);
        let blocks = book.chapter_blocks(index);
        let mut texts = Vec::new();
        for (block_index, block) in blocks.into_iter().enumerate() {
            if !matches!(
                block.kind,
                crate::xhtml::Kind::Para
                    | crate::xhtml::Kind::Heading(_)
                    | crate::xhtml::Kind::Quote
                    | crate::xhtml::Kind::Bullet
                    | crate::xhtml::Kind::Numbered(_)
                    | crate::xhtml::Kind::Pre
                    | crate::xhtml::Kind::Caption
            ) {
                continue;
            }
            let original = block.text();
            if original.trim().is_empty() {
                continue;
            }
            texts.push(BlockText {
                block: block_index,
                normalized: normalize(&original),
                original,
            });
        }
        chapters.push(ChapterIndex {
            title,
            blocks: texts,
        });
    }
    SearchIndex { chapters }
}

/// Minúsculas + sin acentos, un carácter por carácter. Mantener la longitud es
/// lo que permite reutilizar los índices del texto normalizado en el original.
pub fn normalize(text: &str) -> String {
    text.chars()
        .map(|c| {
            let lower = c.to_lowercase().next().unwrap_or(c);
            deaccent(lower)
        })
        .collect()
}

fn deaccent(c: char) -> char {
    match c {
        'á' | 'à' | 'ä' | 'â' | 'ã' | 'å' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        'ç' => 'c',
        'ý' | 'ÿ' => 'y',
        _ => c,
    }
}

/// Busca `query` en el índice y devuelve hasta `max` coincidencias.
/// El segundo valor indica si se cortó la lista porque había más.
pub fn search(index: &SearchIndex, query: &str, max: usize) -> (Vec<Hit>, bool) {
    let needle = normalize(query.trim());
    if needle.chars().count() < 2 {
        return (Vec::new(), false);
    }
    let needle_chars = needle.chars().count();
    let mut hits = Vec::new();
    let mut truncated = false;

    'chapters: for (chapter, chapter_index) in index.chapters.iter().enumerate() {
        for text in &chapter_index.blocks {
            let mut from = 0usize;
            while let Some(offset) = text.normalized[from..].find(&needle) {
                let byte = from + offset;
                let start = text.normalized[..byte].chars().count();
                let end = start + needle_chars;
                if hits.len() >= max {
                    truncated = true;
                    break 'chapters;
                }
                hits.push(make_hit(chapter, text.block, text, start, end));
                from = byte + needle.len();
            }
        }
    }
    (hits, truncated)
}

fn make_hit(chapter: usize, block: usize, text: &BlockText, start: usize, end: usize) -> Hit {
    const CONTEXT: usize = 60;
    let chars: Vec<char> = text.original.chars().collect();
    let start = start.min(chars.len());
    let end = end.min(chars.len());
    let from = start.saturating_sub(CONTEXT);
    let to = (end + CONTEXT).min(chars.len());

    let mut before: String = chars[from..start].iter().collect();
    let matched: String = chars[start..end].iter().collect();
    let mut after: String = chars[end..to].iter().collect();

    if from > 0 {
        before.insert(0, '…');
    }
    if to < chars.len() {
        after.push('…');
    }

    Hit {
        chapter,
        block,
        start,
        end,
        before: clean_preview(&before),
        matched: clean_preview(&matched),
        after: clean_preview(&after),
    }
}

fn clean_preview(text: &str) -> String {
    text.replace(['\n', '\r', '\t'], " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index(chapters: Vec<(&str, Vec<&str>)>) -> SearchIndex {
        SearchIndex {
            chapters: chapters
                .into_iter()
                .map(|(title, blocks)| ChapterIndex {
                    title: title.to_string(),
                    blocks: blocks
                        .into_iter()
                        .enumerate()
                        .map(|(block, text)| BlockText {
                            block,
                            original: text.to_string(),
                            normalized: normalize(text),
                        })
                        .collect(),
                })
                .collect(),
        }
    }

    #[test]
    fn normaliza_sin_cambiar_el_largo() {
        let original = "Árbol Ñandú, canción. ¡ÜBER!";
        let normalized = normalize(original);
        assert_eq!(normalized, "arbol nandu, cancion. ¡uber!");
        assert_eq!(
            original.chars().count(),
            normalized.chars().count(),
            "la normalización debe ser 1:1 para mapear offsets"
        );
    }

    #[test]
    fn busca_acentos_y_mayusculas_en_varios_bloques() {
        let idx = index(vec![
            ("Uno", vec!["El árbol de la ciencia", "otro árbol más"]),
            ("Dos", vec!["Sin coincidencias"]),
        ]);
        let (hits, truncated) = search(&idx, "ARBOL", 10);
        assert!(!truncated);
        assert_eq!(hits.len(), 2);
        assert_eq!((hits[0].chapter, hits[0].block), (0, 0));
        assert_eq!((hits[0].start, hits[0].end), (3, 8));
        assert_eq!(hits[0].matched, "árbol");
        assert_eq!(hits[0].before, "El ");
        assert_eq!((hits[1].chapter, hits[1].block), (0, 1));
        assert_eq!(hits[1].start, 5);
    }

    #[test]
    fn respeta_el_tope_y_marca_truncado() {
        let idx = index(vec![("Uno", vec!["eco eco eco eco"])]);
        let (hits, truncated) = search(&idx, "eco", 2);
        assert_eq!(hits.len(), 2);
        assert!(truncated);
        assert_eq!(hits[0].start, 0);
        assert_eq!(hits[1].start, 4);
    }

    #[test]
    fn consulta_corta_o_vacia_no_busca() {
        let idx = index(vec![("Uno", vec!["hola mundo"])]);
        assert!(search(&idx, "", 10).0.is_empty());
        assert!(search(&idx, "a", 10).0.is_empty());
        assert!(search(&idx, "   ", 10).0.is_empty());
        assert_eq!(search(&idx, "mundo", 10).0.len(), 1);
    }

    #[test]
    fn contextos_recortan_y_limpian_saltos() {
        let idx = index(vec![(
            "Uno",
            vec![&format!(
                "{}\nlínea con objetivo\n{}",
                "a".repeat(70),
                "b".repeat(70)
            )],
        )]);
        let (hits, _) = search(&idx, "objetivo", 10);
        let hit = &hits[0];
        assert!(hit.before.starts_with('…'));
        assert!(hit.after.ends_with('…'));
        assert!(!hit.before.contains('\n'));
        assert!(!hit.after.contains('\n'));
        assert_eq!(hit.matched, "objetivo");
    }
}
