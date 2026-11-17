import 'dart:io';

import 'package:crypto/crypto.dart';
import 'package:flutter/foundation.dart';
import 'package:path/path.dart' as p;

import 'data_store.dart';
import 'models.dart';
import 'rust_core.dart';

class AppStore extends ChangeNotifier {
  AppStore({DataStore? data}) : data = data ?? DataStore();

  final DataStore data;

  bool loading = true;
  String? fatalError;
  String? notice;

  Settings settings = const Settings();
  BookLibrary library = BookLibrary();
  AnnotationStore annotations = AnnotationStore();
  Future<void>? _initializing;

  Future<void> init() {
    final running = _initializing;
    if (running != null) return running;
    final future = _init();
    _initializing = future;
    future.whenComplete(() {
      if (fatalError != null) _initializing = null;
    });
    return future;
  }

  Future<void> _init() async {
    try {
      await data.init();
      if (data.configJson.isEmpty) {
        await data.saveSettings(settings);
      }
      if (data.libraryJson.isEmpty) {
        await data.saveLibrary(library);
      }
      if (data.annotationsJson.isEmpty) {
        await data.saveAnnotations(annotations);
      }
      _reloadModels();
    } catch (error) {
      fatalError = _message(error);
    } finally {
      loading = false;
      notifyListeners();
    }
  }

  void _reloadModels() {
    settings = data.settings;
    library = data.library;
    annotations = data.annotations;
  }

  Future<void> reload() async {
    await data.init();
    _reloadModels();
    notifyListeners();
  }

  Future<void> saveSettings(Settings next) async {
    settings = next;
    await data.saveSettings(next);
    notifyListeners();
  }

  Future<int> importBooks(List<String> sources) async {
    final now = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    final booksDir = Directory(p.join(data.rootPath, 'books'));
    final coversDir = Directory(p.join(data.rootPath, 'covers'));
    await booksDir.create(recursive: true);
    await coversDir.create(recursive: true);

    var imported = 0;
    final failures = <String>[];
    for (final source in sources) {
      try {
        final sourceFile = File(source);
        if (!await sourceFile.exists()) {
          throw const FormatException('El archivo ya no existe.');
        }
        final digest = (await sha256.bind(sourceFile.openRead()).first)
            .toString();
        final target = File(p.join(booksDir.path, '$digest.epub'));
        if (!await target.exists()) {
          final temp = File('${target.path}.tmp');
          await sourceFile.copy(temp.path);
          await temp.rename(target.path);
        }

        final rust = RustCore.instance.open(target.path);
        try {
          final metadata = rust.metadata();
          String? coverPath;
          final cover = rust.cover();
          if (cover != null && cover.isNotEmpty) {
            final coverFile = File(p.join(coversDir.path, '$digest.png'));
            await coverFile.writeAsBytes(cover, flush: true);
            coverPath = 'covers/$digest.png';
          }
          final stat = await target.stat();
          final existing = _findExisting(target.path, source, metadata);
          if (existing != null) {
            final oldPath = existing.path;
            existing
              ..path = target.path
              ..title = metadata.title
              ..author = metadata.author
              ..lang = metadata.lang
              ..series = metadata.series
              ..seriesIndex = metadata.seriesIndex
              ..chapters = metadata.chapters
              ..size = stat.size
              ..mtime = stat.modified.millisecondsSinceEpoch ~/ 1000
              ..missing = false
              ..error = null
              ..lastOpened = now;
            if (coverPath != null) existing.cover = coverPath;
            if (oldPath.isNotEmpty && oldPath != target.path) {
              existing.originPath ??= oldPath;
              annotations.rekey(oldPath, target.path);
            }
          } else {
            library.books.add(
              BookEntry(
                path: target.path,
                title: metadata.title,
                author: metadata.author,
                lang: metadata.lang,
                series: metadata.series,
                seriesIndex: metadata.seriesIndex,
                chapters: metadata.chapters,
                cover: coverPath,
                size: stat.size,
                mtime: stat.modified.millisecondsSinceEpoch ~/ 1000,
                added: now,
                lastOpened: now,
              ),
            );
          }
          imported += 1;
        } finally {
          rust.close();
        }
      } catch (error) {
        failures.add('${p.basename(source)}: ${_message(error)}');
      }
    }

    await data.saveLibrary(library);
    await data.saveAnnotations(annotations);
    notice = imported == 0
        ? failures.join('\n')
        : imported == 1
        ? 'Libro importado.'
        : '$imported libros importados.'
              '${failures.isEmpty ? '' : '\n${failures.join('\n')}'}';
    notifyListeners();
    return imported;
  }

  BookEntry? _findExisting(
    String targetPath,
    String sourcePath,
    BookMetadata metadata,
  ) {
    final originName = p.basename(sourcePath).toLowerCase();
    for (final book in library.books) {
      if (book.path == targetPath) return book;
    }
    for (final book in library.books) {
      final origin = book.originPath;
      if (origin != null && p.basename(origin).toLowerCase() == originName) {
        return book;
      }
    }
    for (final book in library.books) {
      if (book.missing &&
          book.title.toLowerCase() == metadata.title.toLowerCase() &&
          (book.author.isEmpty ||
              metadata.author.isEmpty ||
              book.author.toLowerCase() == metadata.author.toLowerCase())) {
        return book;
      }
    }
    return null;
  }

  Future<void> saveProgress(String path, BookProgress progress) async {
    final book = _bookFor(path);
    if (book == null) return;
    book.progress = progress;
    book.lastOpened = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    await data.saveLibrary(library);
    notifyListeners();
  }

  BookEntry? _bookFor(String path) {
    for (final book in library.books) {
      if (book.path == path) return book;
    }
    return null;
  }

  BookEntry? bookAt(String path) => _bookFor(path);

  Future<void> addAnnotation(AnnotationItem annotation) async {
    annotation.id = annotations.nextId();
    if (annotation.created == 0) {
      annotation.created = DateTime.now().millisecondsSinceEpoch ~/ 1000;
    }
    annotations.items.add(annotation);
    await data.saveAnnotations(annotations);
    notifyListeners();
  }

  Future<void> updateAnnotationNote(
    AnnotationItem annotation,
    String note,
  ) async {
    annotation.note = note;
    await data.saveAnnotations(annotations);
    notifyListeners();
  }

  Future<void> removeAnnotation(AnnotationItem annotation) async {
    annotations.items.removeWhere((item) => item.id == annotation.id);
    await data.saveAnnotations(annotations);
    notifyListeners();
  }

  Future<void> removeBook(BookEntry book) async {
    library.books.remove(book);
    annotations.items.removeWhere((item) => item.book == book.path);
    await data.saveLibrary(library);
    await data.saveAnnotations(annotations);
    final file = File(book.path);
    if (await file.exists()) await file.delete();
    notifyListeners();
  }

  Future<BackupResult> exportBackup() async {
    final result = await data.exportBackup();
    notice = 'Respaldo guardado en ${result.path}';
    notifyListeners();
    return result;
  }

  Future<BackupImportResult> importBackup(String path) async {
    final result = await data.importBackup(path);
    _reloadModels();
    notice = result.previousBackup == null
        ? 'Respaldo importado (${result.files} archivos).'
        : 'Respaldo importado (${result.files} archivos). '
              'Copia previa: ${result.previousBackup}';
    notifyListeners();
    return result;
  }

  void clearNotice() {
    notice = null;
  }

  static String _message(Object error) {
    final text = error.toString();
    if (text.startsWith('Exception: ')) return text.substring(11);
    if (text.startsWith('FormatException: ')) return text.substring(17);
    return text;
  }
}
