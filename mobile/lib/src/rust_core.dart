import 'dart:convert';
import 'dart:ffi';
import 'dart:io';
import 'dart:typed_data';

import 'package:ffi/ffi.dart';

import 'models.dart';

final class _CandilBookC extends Struct {
  external Pointer<Void> handle;
  external Pointer<Utf8> error;
}

final class _CandilBufferC extends Struct {
  external Pointer<Uint8> data;

  @IntPtr()
  external int len;

  @IntPtr()
  external int cap;
}

typedef _OpenC = _CandilBookC Function(Pointer<Utf8>);
typedef _OpenDart = _CandilBookC Function(Pointer<Utf8>);
typedef _HandleStringC = Pointer<Utf8> Function(Pointer<Void>);
typedef _HandleStringDart = Pointer<Utf8> Function(Pointer<Void>);
typedef _ChapterC = Pointer<Utf8> Function(Pointer<Void>, IntPtr);
typedef _ChapterDart = Pointer<Utf8> Function(Pointer<Void>, int);
typedef _BufferC = Pointer<_CandilBufferC> Function(Pointer<Void>);
typedef _BufferDart = Pointer<_CandilBufferC> Function(Pointer<Void>);
typedef _ResourceC = Pointer<_CandilBufferC> Function(
  Pointer<Void>,
  Pointer<Utf8>,
);
typedef _ResourceDart = Pointer<_CandilBufferC> Function(
  Pointer<Void>,
  Pointer<Utf8>,
);
typedef _SearchC = Pointer<Utf8> Function(
  Pointer<Void>,
  Pointer<Utf8>,
  Uint8,
  IntPtr,
);
typedef _SearchDart = Pointer<Utf8> Function(
  Pointer<Void>,
  Pointer<Utf8>,
  int,
  int,
);
typedef _CloseC = Void Function(Pointer<Void>);
typedef _CloseDart = void Function(Pointer<Void>);
typedef _FreeStringC = Void Function(Pointer<Utf8>);
typedef _FreeStringDart = void Function(Pointer<Utf8>);
typedef _FreeBufferC = Void Function(Pointer<_CandilBufferC>);
typedef _FreeBufferDart = void Function(Pointer<_CandilBufferC>);

class RustException implements Exception {
  RustException(this.message);
  final String message;

  @override
  String toString() => message;
}

class RustCore {
  RustCore._() : _library = _load() {
    _open = _library.lookupFunction<_OpenC, _OpenDart>('candil_open');
    _metadata = _library.lookupFunction<_HandleStringC, _HandleStringDart>(
      'candil_metadata',
    );
    _chapter = _library.lookupFunction<_ChapterC, _ChapterDart>(
      'candil_chapter',
    );
    _cover = _library.lookupFunction<_BufferC, _BufferDart>('candil_cover');
    _resource = _library.lookupFunction<_ResourceC, _ResourceDart>(
      'candil_resource',
    );
    _search = _library.lookupFunction<_SearchC, _SearchDart>('candil_search');
    _close = _library.lookupFunction<_CloseC, _CloseDart>('candil_close');
    _freeString = _library.lookupFunction<_FreeStringC, _FreeStringDart>(
      'candil_free_string',
    );
    _freeBuffer = _library.lookupFunction<_FreeBufferC, _FreeBufferDart>(
      'candil_free_buffer',
    );
  }

  static final RustCore instance = RustCore._();

  final DynamicLibrary _library;
  late final _OpenDart _open;
  late final _HandleStringDart _metadata;
  late final _ChapterDart _chapter;
  late final _BufferDart _cover;
  late final _ResourceDart _resource;
  late final _SearchDart _search;
  late final _CloseDart _close;
  late final _FreeStringDart _freeString;
  late final _FreeBufferDart _freeBuffer;

  static DynamicLibrary _load() {
    final override = Platform.environment['CANDIL_CORE_LIB'];
    if (override != null && override.isNotEmpty) {
      return DynamicLibrary.open(override);
    }
    if (Platform.isAndroid) {
      return DynamicLibrary.open('libcandil_mobile_core.so');
    }
    if (Platform.isIOS || Platform.isMacOS) {
      try {
        return DynamicLibrary.process();
      } on ArgumentError {
        return DynamicLibrary.open('libcandil_mobile_core.dylib');
      }
    }
    if (Platform.isWindows) {
      return DynamicLibrary.open('candil_mobile_core.dll');
    }
    for (final path in <String>[
      '../target/debug/libcandil_mobile_core.so',
      '../target/release/libcandil_mobile_core.so',
      '../rust/target/debug/libcandil_mobile_core.so',
    ]) {
      if (File(path).existsSync()) return DynamicLibrary.open(path);
    }
    return DynamicLibrary.open('libcandil_mobile_core.so');
  }

  RustBook open(String path) {
    final nativePath = path.toNativeUtf8();
    try {
      final response = _open(nativePath);
      if (response.handle == nullptr) {
        final message = _readAndFree(response.error);
        throw RustException(
          message.isEmpty ? 'No pude abrir el EPUB.' : message,
        );
      }
      if (response.error != nullptr) _freeString(response.error);
      return RustBook._(this, response.handle);
    } finally {
      malloc.free(nativePath);
    }
  }

  String _readAndFree(Pointer<Utf8> pointer) {
    if (pointer == nullptr) return '';
    try {
      return pointer.toDartString();
    } finally {
      _freeString(pointer);
    }
  }

  String _handleCall(Pointer<Void> handle, _HandleStringDart function) {
    final pointer = function(handle);
    final text = _readAndFree(pointer);
    final data = jsonDecode(text);
    if (data is Map && data['ok'] == false) {
      throw RustException(asString(data['error'], 'Error del núcleo.'));
    }
    return text;
  }

  String chapterJson(Pointer<Void> handle, int index) {
    final pointer = _chapter(handle, index);
    final text = _readAndFree(pointer);
    final data = jsonDecode(text);
    if (data is Map && data['ok'] == false) {
      throw RustException(asString(data['error'], 'Error del núcleo.'));
    }
    return text;
  }

  String metadataJson(Pointer<Void> handle) => _handleCall(handle, _metadata);

  Uint8List? cover(Pointer<Void> handle) => _readBuffer(_cover(handle));

  Uint8List? resource(Pointer<Void> handle, String path) {
    final nativePath = path.toNativeUtf8();
    try {
      return _readBuffer(_resource(handle, nativePath));
    } finally {
      malloc.free(nativePath);
    }
  }

  SearchResult search(
    Pointer<Void> handle,
    String query, {
    int mode = 0,
    int max = 200,
  }) {
    final nativeQuery = query.toNativeUtf8();
    try {
      final pointer = _search(handle, nativeQuery, mode, max);
      final text = _readAndFree(pointer);
      final data = asMap(jsonDecode(text));
      if (data['ok'] == false) {
        throw RustException(asString(data['error'], 'Error de búsqueda.'));
      }
      return SearchResult(
        hits: (data['hits'] is List)
            ? (data['hits'] as List)
                  .whereType<Map>()
                  .map((item) => SearchHit.fromJson(asMap(item)))
                  .toList()
            : <SearchHit>[],
        truncated: asBool(data['truncated']),
      );
    } finally {
      malloc.free(nativeQuery);
    }
  }

  Uint8List? _readBuffer(Pointer<_CandilBufferC> pointer) {
    if (pointer == nullptr) return null;
    try {
      final buffer = pointer.ref;
      if (buffer.data == nullptr || buffer.len <= 0) return null;
      return Uint8List.fromList(buffer.data.asTypedList(buffer.len));
    } finally {
      _freeBuffer(pointer);
    }
  }

  void close(Pointer<Void> handle) => _close(handle);
}

class SearchResult {
  const SearchResult({required this.hits, required this.truncated});
  final List<SearchHit> hits;
  final bool truncated;
}

class RustBook {
  RustBook._(this._core, this._handle);

  final RustCore _core;
  final Pointer<Void> _handle;
  bool _closed = false;

  BookMetadata metadata() =>
      BookMetadata.fromJson(asMap(jsonDecode(_core.metadataJson(_handle))));

  ReaderChapter chapter(int index) => ReaderChapter.fromJson(
    asMap(jsonDecode(_core.chapterJson(_handle, index))),
  );

  Uint8List? cover() => _core.cover(_handle);

  Uint8List? resource(String path) => _core.resource(_handle, path);

  SearchResult search(String query, {int mode = 0, int max = 200}) =>
      _core.search(_handle, query, mode: mode, max: max);

  void close() {
    if (_closed) return;
    _closed = true;
    _core.close(_handle);
  }
}
