import 'dart:io';

import 'package:candil_mobile/src/data_store.dart';
import 'package:candil_mobile/src/models.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:path/path.dart' as p;

void main() {
  test('exporta e importa un respaldo compatible', () async {
    final sourceDir = await Directory.systemTemp.createTemp('candil-src-');
    final targetDir = await Directory.systemTemp.createTemp('candil-dst-');
    addTearDown(() async {
      if (await sourceDir.exists()) await sourceDir.delete(recursive: true);
      if (await targetDir.exists()) await targetDir.delete(recursive: true);
    });

    const desktopPath = '/desktop/Candil/Libro.epub';

    final source = DataStore(directory: sourceDir);
    await source.init();
    await source.saveSettings(
      const Settings(theme: 3, fontSize: 21, focusMode: true),
    );
    await source.saveLibrary(
      BookLibrary(
        books: <BookEntry>[
          BookEntry(
            path: desktopPath,
            title: 'Libro',
            author: 'Autora',
            lang: 'es',
            chapters: 4,
            progress: const BookProgress(chapter: 2, ratio: 0.35),
          ),
        ],
      ),
    );
    await source.saveAnnotations(
      AnnotationStore(
        items: <AnnotationItem>[
          AnnotationItem(
            id: 1,
            book: desktopPath,
            chapter: 2,
            block: 1,
            end: 4,
            parts: const <String>['Hola'],
            quote: 'Hola',
            note: 'Una nota',
          ),
        ],
      ),
    );
    await File(p.join(sourceDir.path, 'studies.json'))
        .writeAsString('{"schema_version":1,"sessions":[],"mensajes":[]}');

    final exported = await source.exportBackup();
    expect(await File(exported.path).exists(), isTrue);
    expect(exported.files, 4);

    final target = DataStore(directory: targetDir);
    await target.init();
    final imported = await target.importBackup(exported.path);
    expect(imported.files, 4);
    expect(target.settings.theme, 3);
    expect(target.settings.focusMode, isTrue);
    final book = target.library.books.single;
    expect(book.title, 'Libro');
    expect(book.progress.chapter, 2);
    expect(book.missing, isTrue);
    expect(book.originPath, desktopPath);
    expect(target.annotations.items.single.quote, 'Hola');
    expect(target.studiesJson['schema_version'], 1);
  });

  test('rechaza un respaldo sin manifiesto', () async {
    final dir = await Directory.systemTemp.createTemp('candil-bad-');
    addTearDown(() async {
      if (await dir.exists()) await dir.delete(recursive: true);
    });
    final store = DataStore(directory: dir);
    await store.init();
    final fake = File(p.join(dir.path, 'fake.zip'));
    await fake.writeAsBytes(<int>[0, 1, 2, 3]);
    expect(() => store.importBackup(fake.path), throwsA(anything));
  });
}
