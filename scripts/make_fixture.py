#!/usr/bin/env python3
"""Genera tests/fixtures/mini.epub: corpus mínimo, propio y versionable.

Cubre a propósito los tres casos que rompieron lectores reales:
  1. XHTML válido con estilos inline, listas y blockquote.
  2. Capítulo con `text/html` mal formado (atributo sin comillas) -> parser tolerante.
  3. Imagen referenciada por ruta relativa desde un subdirectorio.

Uso:  python3 scripts/make_fixture.py
"""

import pathlib
import zipfile

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "tests" / "fixtures" / "mini.epub"

CONTAINER = """<?xml version="1.0" encoding="utf-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>
"""

OPF = """<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="id">urn:uuid:6d0f1f3e-fixture-0000-0000-000000000001</dc:identifier>
    <dc:title>Libro de prueba</dc:title>
    <dc:creator>Autora Ficticia</dc:creator>
    <dc:language>es</dc:language>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="c1" href="Text/cap1.xhtml" media-type="application/xhtml+xml"/>
    <item id="c2" href="Text/cap2.xhtml" media-type="text/html"/>
    <item id="c3" href="Text/cap3.xhtml" media-type="application/xhtml+xml"/>
    <item id="img" href="Images/punto.png" media-type="image/png"/>
  </manifest>
  <spine>
    <itemref idref="c1"/>
    <itemref idref="c2"/>
    <itemref idref="c3"/>
  </spine>
</package>
"""

NAV = """<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
  <head><title>Índice</title></head>
  <body>
    <nav epub:type="toc">
      <ol>
        <li><a href="Text/cap1.xhtml">Uno</a></li>
        <li><a href="Text/cap2.xhtml">Dos</a></li>
        <li><a href="Text/cap3.xhtml#final">Tres</a></li>
      </ol>
    </nav>
  </body>
</html>
"""

CAP1 = """<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
  <head><title>Uno</title></head>
  <body>
    <h1>Uno</h1>
    <p>Un párrafo con <em>énfasis</em> y <strong>negrita</strong>.</p>
    <p>Segundo párrafo con una entidad: a&nbsp;b &mdash; c &amp; d.</p>
    <blockquote>Una cita breve.</blockquote>
    <ul><li>primer ítem</li><li>segundo ítem</li></ul>
    <ol><li>uno</li><li>dos</li></ol>
    <hr/>
    <img src="../Images/punto.png" alt="un punto"/>
  </body>
</html>
"""

# A propósito NO es XML válido: atributo sin comillas y entidades sueltas.
CAP2 = """<!DOCTYPE html>
<html>
  <head><title>Dos</title></head>
  <body>
    <h2>Dos</h2>
    <p class=roto>Texto de un capítulo que no es XML válido: efectos: raros.</p>
    <p>Otro párrafo <b>en negrita</b> que debe sobrevivir.</p>
  </body>
</html>
"""

CAP3 = """<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
  <head><title>Tres</title></head>
  <body>
    <h1 id="final">Tres</h1>
    <p>Último capítulo.</p>
  </body>
</html>
"""

# PNG 2x2 negro, mínimo y sin dependencias.
PNG = bytes.fromhex(
    "89504e470d0a1a0a0000000d49484452000000020000000208060000"
    "00f478d4fa0000001549444154789c63fcffff3f03030303031c1818"
    "1800e0e00a0b8f4b3d900000000049454e44ae426082"
)


def main() -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    with zipfile.ZipFile(OUT, "w") as z:
        # El mimetype debe ir primero y sin comprimir.
        z.writestr(
            zipfile.ZipInfo("mimetype"),
            "application/epub+zip",
            compress_type=zipfile.ZIP_STORED,
        )
        z.writestr("META-INF/container.xml", CONTAINER, zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/content.opf", OPF, zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/nav.xhtml", NAV, zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/Text/cap1.xhtml", CAP1, zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/Text/cap2.xhtml", CAP2, zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/Text/cap3.xhtml", CAP3, zipfile.ZIP_DEFLATED)
        z.writestr("OEBPS/Images/punto.png", PNG, zipfile.ZIP_DEFLATED)
    print(f"escrito {OUT} ({OUT.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
