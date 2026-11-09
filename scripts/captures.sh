#!/usr/bin/env bash
# Genera las capturas del README en un sandbox aislado.
#
# No usa el HOME real ni ~/Libros. Los libros son de Project Gutenberg
# (dominio público) y quedan cacheados en target/captures-sandbox/Libros.
# Uso: scripts/captures.sh
set -euo pipefail

# Las capturas usan un HOME falso sin llavero de login.
export CANDIL_NO_KEYRING=1

cd "$(dirname "$0")/.."
ROOT="$PWD"
SANDBOX="${CANDIL_CAPTURE_SANDBOX:-$ROOT/target/captures-sandbox}"
BOOKS="$SANDBOX/Libros"
HOME_DIR="$SANDBOX/home"
SHOTS="$SANDBOX/shots"
BIN="$ROOT/target/fast/candil"
MAGS=(
  "1342:pride-and-prejudice.epub"
  "1952:the-yellow-wallpaper.epub"
  "84:frankenstein.epub"
  "345:dracula.epub"
  "2000:don-quijote.epub"
)

command -v python3 >/dev/null || { echo "falta python3"; exit 1; }
cargo build --profile fast >/dev/null

rm -rf "$HOME_DIR" "$SHOTS"
mkdir -p "$BOOKS" "$HOME_DIR" "$SHOTS" "$ROOT/docs/img"

for spec in "${MAGS[@]}"; do
  id="${spec%%:*}"
  name="${spec#*:}"
  if [ ! -s "$BOOKS/$name" ]; then
    echo "descargando ${name}..."
    curl -L --fail --silent --show-error --max-time 180 \
      -o "$BOOKS/$name" \
      "https://www.gutenberg.org/cache/epub/$id/pg$id.epub"
  fi
done

HOME="$HOME_DIR" "$BIN" --scan "$BOOKS" >/dev/null
DATA_DIR=$(dirname "$(find "$HOME_DIR" -type f -name library.json -print -quit)")
PP=$(python3 - "$DATA_DIR/library.json" <<'PY'
import json, sys
for b in json.load(open(sys.argv[1]))['books']:
    if b['title'].startswith('Pride and Prejudice'):
        print(b['path'])
        break
PY
)
QUIJOTE=$(python3 - "$DATA_DIR/library.json" <<'PY'
import json, sys
for b in json.load(open(sys.argv[1]))['books']:
    if b['title'].startswith('Don Quijote'):
        print(b['path'])
        break
PY
)
[ -n "$PP" ] || { echo "no encontré Pride and Prejudice"; exit 1; }
[ -n "$QUIJOTE" ] || { echo "no encontré Don Quijote"; exit 1; }

# El libro de lectura arranca en Chapter I, tema Noche. No toca datos reales.
python3 - "$DATA_DIR/library.json" "$PP" <<'PY'
import json, sys, time
path, pp = sys.argv[1], sys.argv[2]
now = int(time.time())
data = json.load(open(path))
for book in data['books']:
    if book['path'] == pp:
        book['progress'] = {'chapter': 1, 'ratio': 0.12, 'anchor': 0, 'anchor_ratio': 0.0}
        book['last_opened'] = now - 300
        book['settings'] = {
            'theme': 0, 'font_size': 19.0, 'line_height': 1.55,
            'column_width': 660.0, 'paragraph_space': 0.75,
        }
json.dump(data, open(path, 'w'), ensure_ascii=False, indent=2)
PY

# Destacados públicos: tres colores en P&P + uno de Don Quijote para "Todos".
python3 - "$DATA_DIR/library.json" "$PP" "$QUIJOTE" <<'PY'
import json, sys, time
lib, pp, quijote = sys.argv[1], sys.argv[2], sys.argv[3]
now = int(time.time())
items = [
    {
        'id': 1, 'book': pp, 'chapter': 1, 'block': 0, 'start': 0, 'end': 1,
        'quote': 'is a truth universally acknowledged',
        'note': 'La ironía abre la novela: una “verdad” que en realidad es un chisme de vecinas.',
        'color': 0, 'created': now - 300,
    },
    {
        'id': 2, 'book': pp, 'chapter': 1, 'block': 0, 'start': 0, 'end': 1,
        'quote': 'a single man in possession of a good fortune',
        'note': '',
        'color': 1, 'created': now - 200,
    },
    {
        'id': 3, 'book': pp, 'chapter': 1, 'block': 0, 'start': 0, 'end': 1,
        'quote': 'must be in want of a wife',
        'note': '',
        'color': 2, 'created': now - 100,
    },
    {
        'id': 4, 'book': quijote, 'chapter': 2, 'block': 0, 'start': 0, 'end': 1,
        'quote': 'En un lugar de la Mancha',
        'note': 'El comienzo más famoso de la literatura en español.',
        'color': 1, 'created': now - 50,
    },
]
data = {'schema_version': 1, 'items': items}
json.dump(data, open(f'{lib.rsplit("/", 1)[0]}/annotations.json', 'w'), ensure_ascii=False, indent=2)
PY

# Historial público para la pestaña Historial y el detalle.
python3 - "$DATA_DIR/library.json" "$PP" <<'PY'
import json, sys, time
lib, pp = sys.argv[1], sys.argv[2]
now = int(time.time())
data = {
    'schema_version': 1,
    'items': [
        {
            'id': 1, 'created': now - 120, 'kind': 'diccionario',
            'provider': 'Wiktionary', 'model': None, 'prompt': None,
            'source_lang': 'en', 'target_lang': 'es',
            'book': pp, 'book_title': 'Pride and Prejudice',
            'chapter': 1, 'block': 0, 'quote': 'truth',
            'response': 'Noun\nA fact or belief accepted as true.\n\nVerbo\nTo tell the truth.',
            'error': None,
        },
        {
            'id': 2, 'created': now - 60, 'kind': 'traduccion',
            'provider': 'MyMemory', 'model': None, 'prompt': None,
            'source_lang': 'en', 'target_lang': 'es',
            'book': pp, 'book_title': 'Pride and Prejudice',
            'chapter': 1, 'block': 0, 'quote': 'is a truth universally acknowledged',
            'response': 'Es una verdad universalmente reconocida.',
            'error': None,
        },
    ],
}
json.dump(data, open(f'{lib.rsplit("/", 1)[0]}/consultas.json', 'w'), ensure_ascii=False, indent=2)
PY

shot() {
  local out="$1"
  shift
  HOME="$HOME_DIR" "$BIN" --shot "$SHOTS/$out" "$@" >/dev/null 2>&1
}

# Biblioteca (tema Papel por defecto)
shot library-full.png --screen library
shot library-search.png --screen library --search austen
shot library-edit.png --screen library --edit
shot config-data.png --screen library --panel advanced

# Lector: Chapter I de Pride and Prejudice (dominio público).
ANCHOR=pgepubid00022
shot reader-settings.png --anchor "$ANCHOR" --panel settings "$PP"
shot reader-highlight.png --anchor "$ANCHOR" "$PP"
shot reader-selection.png --anchor "$ANCHOR" --selection "$PP"
shot reader-notes.png --anchor "$ANCHOR" --panel notes "$PP"
shot reader-note.png --anchor "$ANCHOR" --panel note "$PP"
shot reader-notes-global.png --anchor "$ANCHOR" --panel notes --annotations-all "$PP"
shot reader-search.png --anchor "$ANCHOR" --panel search --book-search truth --search-jump "$PP"
shot reader-goto.png --anchor "$ANCHOR" --panel goto "$PP"
shot reader-config.png --anchor "$ANCHOR" --panel config "$PP"
shot reader-consulta.png --anchor "$ANCHOR" --consulta sleep --consulta-kind diccionario --consulta-demo "$PP"
shot reader-history.png --anchor "$ANCHOR" --consulta truth --consulta-kind historial --consulta-demo "$PP"

# Biblioteca vacía y biblioteca con un EPUB roto (sin contenido de usuario).
EMPTY=$(mktemp -d)
HOME="$EMPTY" "$BIN" --shot "$SHOTS/library-empty.png" --screen library >/dev/null 2>&1
rm -rf "$EMPTY"
BROKEN="$SANDBOX/broken"
rm -rf "$BROKEN"
mkdir -p "$BROKEN/Libros"
cp "$PP" "$BROKEN/Libros/pride-and-prejudice.epub"
printf 'esto no es un epub' > "$BROKEN/Libros/roto.epub"
HOME="$BROKEN" "$BIN" --scan "$BROKEN/Libros" >/dev/null 2>&1 || true
HOME="$BROKEN" "$BIN" --shot "$SHOTS/library-error.png" --screen library --recover >/dev/null 2>&1 || true

# Copia a docs/img. sips en macOS; magick/convert en Linux.
jpg() {
  local src="$1" dst="$ROOT/docs/img/$2"
  if command -v sips >/dev/null; then
    sips -s format jpeg -s formatOptions 88 -Z 1400 "$src" --out "$dst" >/dev/null
  elif command -v magick >/dev/null; then
    magick "$src" -resize '1400x>' -quality 88 "$dst"
  elif command -v convert >/dev/null; then
    convert "$src" -resize '1400x>' -quality 88 "$dst"
  else
    echo "falta sips/magick/convert para convertir $src"; exit 1
  fi
}
png() {
  local src="$1" dst="$ROOT/docs/img/$2"
  if command -v sips >/dev/null; then
    sips -Z 1400 "$src" --out "$dst" >/dev/null
  elif command -v magick >/dev/null; then
    magick "$src" -resize '1400x>' "$dst"
  elif command -v convert >/dev/null; then
    convert "$src" -resize '1400x>' "$dst"
  else
    cp "$src" "$dst"
  fi
}

png "$SHOTS/library-empty.png" library-empty.png
png "$SHOTS/library-error.png" library-error.png
png "$SHOTS/library-full.png" library-full.png
png "$SHOTS/library-search.png" library-search.png
png "$SHOTS/library-edit.png" library-edit.png
png "$SHOTS/config-data.png" config-data.png
png "$SHOTS/reader-settings.png" reader-settings.png
jpg "$SHOTS/reader-highlight.png" reader-highlight.jpg
jpg "$SHOTS/reader-selection.png" reader-selection.jpg
jpg "$SHOTS/reader-notes.png" reader-notes.jpg
jpg "$SHOTS/reader-note.png" reader-note.jpg
jpg "$SHOTS/reader-notes-global.png" reader-notes-global.jpg
jpg "$SHOTS/reader-search.png" reader-search.jpg
jpg "$SHOTS/reader-goto.png" reader-goto.jpg
jpg "$SHOTS/reader-config.png" reader-config.jpg
jpg "$SHOTS/reader-consulta.png" reader-consulta.jpg
jpg "$SHOTS/reader-history.png" reader-history.jpg

echo "capturas -> docs/img (sandbox: $SANDBOX)"
