import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:archive/archive.dart';
import 'package:crypto/crypto.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';

import 'models.dart';

const backupFormat = 1;
const backupManifest = 'candil-backup.json';
const backupFiles = <String>[
  'config.json',
  'library.json',
  'annotations.json',
  'consultas.json',
  'studies.json',
];
const maxBackupFile = 64 * 1024 * 1024;

class BackupResult {
  const BackupResult({required this.path, required this.files});
  final String path;
  final int files;
}

class BackupImportResult {
  const BackupImportResult({required this.files, this.previousBackup});
  final int files;
  final String? previousBackup;
}

class DataStore {
  DataStore({Directory? directory}) : _override = directory;

  final Directory? _override;
  Directory? _dir;

  Map<String, dynamic> configJson = <String, dynamic>{};
  Map<String, dynamic> libraryJson = <String, dynamic>{};
  Map<String, dynamic> annotationsJson = <String, dynamic>{};
  Map<String, dynamic> consultasJson = <String, dynamic>{};
  Map<String, dynamic> studiesJson = <String, dynamic>{};

  Directory get root {
    final dir = _dir;
    if (dir == null) throw StateError('DataStore.init() no se ejecutó.');
    return dir;
  }

  String get rootPath => root.path;

  Future<void> init() async {
    final base =
        _override ??
        Directory(
          p.join((await getApplicationDocumentsDirectory()).path, 'candil'),
        );
    _dir = base;
    await base.create(recursive: true);
    await _loadAll();
  }

  Future<void> _loadAll() async {
    configJson = await _readMap('config.json');
    libraryJson = await _readMap('library.json');
    annotationsJson = await _readMap('annotations.json');
    consultasJson = await _readMap('consultas.json');
    studiesJson = await _readMap('studies.json');
  }

  Future<Map<String, dynamic>> _readMap(String name) async {
    final file = File(p.join(root.path, name));
    if (!await file.exists()) return <String, dynamic>{};
    final text = await file.readAsString();
    if (text.trim().isEmpty) return <String, dynamic>{};
    final data = jsonDecode(text);
    if (data is! Map) {
      throw FormatException('$name no contiene un objeto JSON.');
    }
    return Map<String, dynamic>.from(data);
  }

  Settings get settings => Settings.fromJson(asMap(configJson['settings']));

  BookLibrary get library => BookLibrary.fromJson(libraryJson);

  AnnotationStore get annotations => AnnotationStore.fromJson(annotationsJson);

  Future<void> saveSettings(Settings settings) async {
    configJson['settings'] = settings.toJson();
    await _writeMap('config.json', configJson);
  }

  Future<void> saveLibrary(BookLibrary library) async {
    libraryJson = library.toJson();
    await _writeMap('library.json', libraryJson);
  }

  Future<void> saveAnnotations(AnnotationStore annotations) async {
    annotationsJson = annotations.toJson();
    await _writeMap('annotations.json', annotationsJson);
  }

  Future<void> _writeMap(String name, Map<String, dynamic> data) =>
      _writeText(name, const JsonEncoder.withIndent('  ').convert(data));

  Future<void> _writeText(String name, String text) async {
    final target = File(p.join(root.path, name));
    final temp = File('${target.path}.tmp');
    await temp.parent.create(recursive: true);
    await temp.writeAsString(text, flush: true);
    await _replace(temp, target);
  }

  Future<void> _writeBytes(String name, List<int> bytes) async {
    final target = File(p.join(root.path, name));
    final temp = File('${target.path}.tmp');
    await temp.parent.create(recursive: true);
    await temp.writeAsBytes(bytes, flush: true);
    await _replace(temp, target);
  }

  Future<void> _replace(File temp, File target) async {
    try {
      await temp.rename(target.path);
    } on FileSystemException {
      if (await target.exists()) await target.delete();
      await temp.rename(target.path);
    }
  }

  Future<BackupResult> exportBackup({String prefix = 'candil-datos'}) async {
    final exports = Directory(p.join(root.path, 'exports'));
    await exports.create(recursive: true);
    final stamp = DateTime.now().millisecondsSinceEpoch;
    final target = File(p.join(exports.path, '$prefix-$stamp.zip'));
    return _writeBackup(target);
  }

  Future<BackupResult> _writeBackup(File target) async {
    final files = <Map<String, dynamic>>[];
    final archive = Archive();
    for (final name in backupFiles) {
      final file = File(p.join(root.path, name));
      if (!await file.exists()) continue;
      final bytes = await file.readAsBytes();
      files.add(<String, dynamic>{
        'name': name,
        'size': bytes.length,
        'sha256': sha256.convert(bytes).toString(),
      });
      archive.add(ArchiveFile.bytes(name, bytes));
    }
    if (files.isEmpty) {
      throw const FormatException('No hay datos para exportar.');
    }
    final manifest = <String, dynamic>{
      'format': backupFormat,
      'app_version': '0.1.0-mobile',
      'created': DateTime.now().millisecondsSinceEpoch ~/ 1000,
      'files': files,
    };
    archive.add(
      ArchiveFile.string(
        backupManifest,
        const JsonEncoder.withIndent('  ').convert(manifest),
      ),
    );
    final bytes = ZipEncoder().encodeBytes(archive);
    await target.parent.create(recursive: true);
    final temp = File('${target.path}.tmp');
    await temp.writeAsBytes(bytes, flush: true);
    await _replace(temp, target);
    return BackupResult(path: target.path, files: files.length);
  }

  Future<BackupImportResult> importBackup(String archivePath) async {
    final bytes = await File(archivePath).readAsBytes();
    final archive = ZipDecoder().decodeBytes(bytes);
    final entries = <String, Uint8List>{};
    for (final file in archive.files) {
      if (!file.isFile) continue;
      final content = file.readBytes();
      if (content != null) entries[file.name] = content;
    }
    final manifestBytes = entries[backupManifest];
    if (manifestBytes == null) {
      throw const FormatException('Falta el manifiesto de Candil.');
    }
    final manifest = asMap(jsonDecode(utf8.decode(manifestBytes)));
    final format = asInt(manifest['format']);
    if (format == 0 || format > backupFormat) {
      throw FormatException('Formato de respaldo no soportado ($format).');
    }
    final list = manifest['files'];
    if (list is! List || list.isEmpty) {
      throw const FormatException('El respaldo no contiene datos.');
    }

    final imported = <String, Uint8List>{};
    for (final raw in list) {
      final entry = asMap(raw);
      final name = asString(entry['name']);
      if (!backupFiles.contains(name)) {
        throw FormatException('Archivo no permitido en el respaldo: $name');
      }
      final content = entries[name];
      if (content == null) throw FormatException('Falta $name en el respaldo.');
      if (content.length > maxBackupFile) {
        throw FormatException('$name es demasiado grande.');
      }
      if (content.length != asInt(entry['size'])) {
        throw FormatException('El tamaño de $name no coincide.');
      }
      final digest = sha256.convert(content).toString();
      if (digest.toLowerCase() != asString(entry['sha256']).toLowerCase()) {
        throw FormatException('El checksum de $name no coincide.');
      }
      final decoded = jsonDecode(utf8.decode(content));
      if (decoded is! Map) {
        throw FormatException('$name no contiene un objeto JSON.');
      }
      imported[name] = content;
    }

    String? previous;
    final hasData = backupFiles.any(
      (name) => File(p.join(root.path, name)).existsSync(),
    );
    if (hasData) {
      final backups = Directory(p.join(root.path, 'backups'));
      await backups.create(recursive: true);
      final target = File(
        p.join(
          backups.path,
          'pre-import-${DateTime.now().millisecondsSinceEpoch}.zip',
        ),
      );
      previous = (await _writeBackup(target)).path;
    }

    for (final entry in imported.entries) {
      await _writeBytes(entry.key, entry.value);
    }
    await _loadAll();
    await markImportedBooksMissing();
    return BackupImportResult(files: imported.length, previousBackup: previous);
  }

  /// Un respaldo de escritorio apunta a rutas que en el móvil no existen.
  /// Se conservan como `origin_path` para poder reencontrarlas al importar el
  /// EPUB y reasignar sus anotaciones.
  Future<void> markImportedBooksMissing() async {
    final lib = library;
    var changed = false;
    for (final book in lib.books) {
      if (book.missing) continue;
      if (!File(book.path).existsSync()) {
        book.missing = true;
        book.originPath ??= book.path;
        changed = true;
      }
    }
    if (changed) await saveLibrary(lib);
  }
}
