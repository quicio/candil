#!/usr/bin/env python3
"""Genera los fixtures EPUB propios y versionables.

`mini.epub` cubre el contrato de M1:
  1. XHTML válido con estilos inline, listas y blockquote.
  2. Capítulo con `text/html` mal formado -> parser tolerante.
  3. Imagen referenciada por ruta relativa desde un subdirectorio.

`edge.epub` fija los bordes de M1.1 (spec 15):
  4. Espacios en límites de runs inline (`<b>Hola</b> <i>mundo</i>`).
  5. Entidades Latin-1/griegas/matemáticas.
  6. `<pre>` con saltos e indentación.
  7. Tabla, `<dl>` y `<svg><image/></svg>`.

`notes.epub` cubre las notas al pie:
  8. Nota en el mismo capítulo y nota en otro documento del spine.

Las entradas usan una fecha fija para que regenerar no cambie el binario.

Uso:  python3 scripts/make_fixture.py
"""

import pathlib
import struct
import zipfile
import zlib

ROOT = pathlib.Path(__file__).resolve().parent.parent
OUT = ROOT / "tests" / "fixtures" / "mini.epub"
EDGE_OUT = ROOT / "tests" / "fixtures" / "edge.epub"

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
    <item id="img" href="Images/punto.png" media-type="image/png" properties="cover-image"/>
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

EDGE_OPF = """<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="id">urn:uuid:6d0f1f3e-fixture-0000-0000-000000000002</dc:identifier>
    <dc:title>Bordes</dc:title>
    <dc:creator>Autora Ficticia</dc:creator>
    <dc:language>es</dc:language>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="c1" href="Text/c1.xhtml" media-type="application/xhtml+xml"/>
    <item id="img" href="Images/punto.png" media-type="image/png"/>
  </manifest>
  <spine>
    <itemref idref="nav"/>
    <itemref idref="c1"/>
  </spine>
</package>
"""

EDGE_NAV = """<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
  <head><title>Índice</title></head>
  <body>
    <nav epub:type="toc">
      <ol>
        <li><a href="Text/c1.xhtml">Bordes</a></li>
      </ol>
    </nav>
  </body>
</html>
"""

EDGE_C1 = """<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:xlink="http://www.w3.org/1999/xlink">
  <head><title>Bordes</title></head>
  <body>
    <h1>Bordes</h1>
    <p><b>Hola</b> <i>mundo</i>.</p>
    <p>caf&eacute; y ni&ntilde;o, &le; &omega;.</p>
    <pre>def f(x):
    return x</pre>
    <table><tr><td>celda uno</td><td>celda dos</td></tr></table>
    <dl><dt>término</dt><dd>definición</dd></dl>
    <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 2 2">
      <image xlink:href="../Images/punto.png" width="2" height="2"/>
    </svg>
  </body>
</html>
"""

NOTES_OUT = ROOT / "tests" / "fixtures" / "notes.epub"

NOTES_OPF = """<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="id">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="id">urn:uuid:6d0f1f3e-fixture-0000-0000-000000000003</dc:identifier>
    <dc:title>Notas de prueba</dc:title>
    <dc:creator>Autora Ficticia</dc:creator>
    <dc:language>es</dc:language>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="c1" href="Text/cap1.xhtml" media-type="application/xhtml+xml"/>
    <item id="n1" href="Text/notas.xhtml" media-type="application/xhtml+xml"/>
    <item id="img" href="Images/punto.png" media-type="image/png" properties="cover-image"/>
  </manifest>
  <spine>
    <itemref idref="c1"/>
    <itemref idref="n1"/>
  </spine>
</package>
"""

NOTES_NAV = """<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
  <head><title>Índice</title></head>
  <body>
    <nav epub:type="toc">
      <ol>
        <li><a href="Text/cap1.xhtml">Con notas</a></li>
        <li><a href="Text/notas.xhtml">Notas</a></li>
      </ol>
    </nav>
  </body>
</html>
"""

NOTES_CAP1 = """<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
  <head><title>Con notas</title></head>
  <body>
    <h1>Con notas</h1>
    <p>Una idea con una nota al pie.<a epub:type="noteref" href="#fn1">1</a> Y el texto sigue.</p>
    <p>Otra idea con una nota en otro documento.<a epub:type="noteref" href="notas.xhtml#fn2">2</a></p>
    <hr/>
    <p id="fn1">1. Nota al pie en el mismo capítulo.</p>
  </body>
</html>
"""

NOTES_NOTES = """<?xml version="1.0" encoding="utf-8"?>
<html xmlns="http://www.w3.org/1999/xhtml">
  <head><title>Notas</title></head>
  <body>
    <h1>Notas</h1>
    <p id="fn2">2. Nota dos, en un documento aparte.</p>
  </body>
</html>
"""

def png_2x2() -> bytes:
    """PNG RGBA 2x2 negro, generado con zlib para no depender de Pillow."""

    def chunk(kind: bytes, data: bytes) -> bytes:
        return (
            struct.pack(">I", len(data))
            + kind
            + data
            + struct.pack(">I", zlib.crc32(kind + data) & 0xFFFFFFFF)
        )

    ihdr = struct.pack(">IIBBBBB", 2, 2, 8, 6, 0, 0, 0)
    raw = b"".join(b"\x00" + b"\x00\x00\x00\xff" * 2 for _ in range(2))
    return (
        b"\x89PNG\r\n\x1a\n"
        + chunk(b"IHDR", ihdr)
        + chunk(b"IDAT", zlib.compress(raw))
        + chunk(b"IEND", b"")
    )


PNG = png_2x2()

# Fecha fija: regenerar el fixture no debe cambiar el binario.
ZIP_DATE = (2026, 1, 1, 0, 0, 0)


def add(z: zipfile.ZipFile, name: str, data, *, stored: bool = False) -> None:
    info = zipfile.ZipInfo(name, date_time=ZIP_DATE)
    info.compress_type = zipfile.ZIP_STORED if stored else zipfile.ZIP_DEFLATED
    z.writestr(info, data)


def write_mini() -> None:
    with zipfile.ZipFile(OUT, "w") as z:
        add(z, "mimetype", "application/epub+zip", stored=True)
        add(z, "META-INF/container.xml", CONTAINER)
        add(z, "OEBPS/content.opf", OPF)
        add(z, "OEBPS/nav.xhtml", NAV)
        add(z, "OEBPS/Text/cap1.xhtml", CAP1)
        add(z, "OEBPS/Text/cap2.xhtml", CAP2)
        add(z, "OEBPS/Text/cap3.xhtml", CAP3)
        add(z, "OEBPS/Images/punto.png", PNG)


def write_edge() -> None:
    with zipfile.ZipFile(EDGE_OUT, "w") as z:
        add(z, "mimetype", "application/epub+zip", stored=True)
        add(z, "META-INF/container.xml", CONTAINER)
        add(z, "OEBPS/content.opf", EDGE_OPF)
        add(z, "OEBPS/nav.xhtml", EDGE_NAV)
        add(z, "OEBPS/Text/c1.xhtml", EDGE_C1)
        add(z, "OEBPS/Images/punto.png", PNG)


def write_notes() -> None:
    with zipfile.ZipFile(NOTES_OUT, "w") as z:
        add(z, "mimetype", "application/epub+zip", stored=True)
        add(z, "META-INF/container.xml", CONTAINER)
        add(z, "OEBPS/content.opf", NOTES_OPF)
        add(z, "OEBPS/nav.xhtml", NOTES_NAV)
        add(z, "OEBPS/Text/cap1.xhtml", NOTES_CAP1)
        add(z, "OEBPS/Text/notas.xhtml", NOTES_NOTES)
        add(z, "OEBPS/Images/punto.png", PNG)


def main() -> None:
    OUT.parent.mkdir(parents=True, exist_ok=True)
    write_mini()
    write_edge()
    write_notes()
    print(f"escrito {OUT} ({OUT.stat().st_size} bytes)")
    print(f"escrito {EDGE_OUT} ({EDGE_OUT.stat().st_size} bytes)")
    print(f"escrito {NOTES_OUT} ({NOTES_OUT.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
