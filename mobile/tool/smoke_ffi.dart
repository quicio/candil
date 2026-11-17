import 'dart:io';

import 'package:candil_mobile/src/rust_core.dart';

void main(List<String> args) {
  if (args.isEmpty) {
    stderr.writeln('Uso: dart run tool/smoke_ffi.dart <libro.epub>');
    exit(2);
  }
  final book = RustCore.instance.open(args.first);
  try {
    final metadata = book.metadata();
    stdout.writeln('${metadata.title} — ${metadata.author}');
    stdout.writeln('${metadata.chapters} capítulos');
    final chapter = book.chapter(0);
    stdout.writeln('Capítulo 1: ${chapter.title} (${chapter.blocks.length} bloques)');
    final cover = book.cover();
    stdout.writeln('Portada: ${cover?.length ?? 0} bytes');
    final found = book.search('texto', max: 5);
    stdout.writeln('Búsqueda "texto": ${found.hits.length} resultados');
  } finally {
    book.close();
  }
}
