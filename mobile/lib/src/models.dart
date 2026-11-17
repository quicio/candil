import 'dart:convert';

Map<String, dynamic> asMap(dynamic value) {
  if (value is Map<String, dynamic>) return value;
  if (value is Map) return Map<String, dynamic>.from(value);
  return <String, dynamic>{};
}

String asString(dynamic value, [String fallback = '']) =>
    value is String ? value : fallback;

int asInt(dynamic value, [int fallback = 0]) {
  if (value is int) return value;
  if (value is num) return value.toInt();
  return fallback;
}

int? asIntOrNull(dynamic value) {
  if (value == null) return null;
  if (value is int) return value;
  if (value is num) return value.toInt();
  return null;
}

double asDouble(dynamic value, [double fallback = 0]) {
  if (value is double) return value;
  if (value is num) return value.toDouble();
  return fallback;
}

double? asDoubleOrNull(dynamic value) {
  if (value == null) return null;
  if (value is double) return value;
  if (value is num) return value.toDouble();
  return null;
}

bool asBool(dynamic value, [bool fallback = false]) =>
    value is bool ? value : fallback;

List<String> asStringList(dynamic value) {
  if (value is! List) return <String>[];
  return value.whereType<String>().toList(growable: false);
}

class Settings {
  const Settings({
    this.theme = 0,
    this.fontSize = 19,
    this.lineHeight = 1.55,
    this.columnWidth = 660,
    this.paragraphSpace = 0.75,
    this.focusMode = false,
  });

  final int theme;
  final double fontSize;
  final double lineHeight;
  final double columnWidth;
  final double paragraphSpace;
  final bool focusMode;

  factory Settings.fromJson(Map<String, dynamic>? json) {
    final data = json ?? const <String, dynamic>{};
    return Settings(
      theme: asInt(data['theme']).clamp(0, 4),
      fontSize: asDouble(data['font_size'], 19).clamp(12, 34),
      lineHeight: asDouble(data['line_height'], 1.55).clamp(1.15, 2.3),
      columnWidth: asDouble(data['column_width'], 660).clamp(400, 1200),
      paragraphSpace: asDouble(data['paragraph_space'], 0.75).clamp(0, 1.6),
      focusMode: asBool(data['focus_mode']),
    );
  }

  Map<String, dynamic> toJson() => <String, dynamic>{
    'theme': theme,
    'font_size': fontSize,
    'line_height': lineHeight,
    'column_width': columnWidth,
    'paragraph_space': paragraphSpace,
    'focus_mode': focusMode,
  };

  Settings copyWith({
    int? theme,
    double? fontSize,
    double? lineHeight,
    double? columnWidth,
    double? paragraphSpace,
    bool? focusMode,
  }) {
    return Settings(
      theme: theme ?? this.theme,
      fontSize: fontSize ?? this.fontSize,
      lineHeight: lineHeight ?? this.lineHeight,
      columnWidth: columnWidth ?? this.columnWidth,
      paragraphSpace: paragraphSpace ?? this.paragraphSpace,
      focusMode: focusMode ?? this.focusMode,
    );
  }
}

class BookProgress {
  const BookProgress({
    this.chapter = 0,
    this.ratio = 0,
    this.anchor,
    this.anchorRatio,
  });

  final int chapter;
  final double ratio;
  final int? anchor;
  final double? anchorRatio;

  factory BookProgress.fromJson(Map<String, dynamic>? json) {
    final data = json ?? const <String, dynamic>{};
    return BookProgress(
      chapter: asInt(data['chapter']),
      ratio: asDouble(data['ratio']).clamp(0, 1),
      anchor: asIntOrNull(data['anchor']),
      anchorRatio: asDoubleOrNull(data['anchor_ratio']),
    );
  }

  Map<String, dynamic> toJson() => <String, dynamic>{
    'chapter': chapter,
    'ratio': ratio,
    if (anchor != null) 'anchor': anchor,
    if (anchorRatio != null) 'anchor_ratio': anchorRatio,
  };

  BookProgress copyWith({int? chapter, double? ratio}) => BookProgress(
    chapter: chapter ?? this.chapter,
    ratio: ratio ?? this.ratio,
    anchor: anchor,
    anchorRatio: anchorRatio,
  );
}

class BookEntry {
  BookEntry({
    required this.path,
    required this.title,
    required this.author,
    required this.lang,
    this.series,
    this.seriesIndex,
    this.chapters = 0,
    this.progress = const BookProgress(),
    this.cover,
    this.size = 0,
    this.mtime = 0,
    this.added = 0,
    this.lastOpened,
    this.missing = false,
    this.error,
    this.originPath,
    Map<String, dynamic>? raw,
  }) : _raw = raw ?? <String, dynamic>{};

  String path;
  String title;
  String author;
  String lang;
  String? series;
  double? seriesIndex;
  int chapters;
  BookProgress progress;
  String? cover;
  int size;
  int mtime;
  int added;
  int? lastOpened;
  bool missing;
  String? error;

  /// Ruta que tenía el libro en el escritorio cuando se importó el respaldo.
  /// Sirve para reencontrarlo cuando el usuario importa el EPUB en el móvil.
  String? originPath;

  final Map<String, dynamic> _raw;

  factory BookEntry.fromJson(Map<String, dynamic> json) {
    final data = asMap(json);
    return BookEntry(
      path: asString(data['path']),
      title: asString(data['title'], 'Sin título'),
      author: asString(data['author']),
      lang: asString(data['lang']),
      series: data['series'] is String ? data['series'] as String : null,
      seriesIndex: asDoubleOrNull(data['series_index']),
      chapters: asInt(data['chapters']),
      progress: BookProgress.fromJson(asMap(data['progress'])),
      cover: data['cover'] is String ? data['cover'] as String : null,
      size: asInt(data['size']),
      mtime: asInt(data['mtime']),
      added: asInt(data['added']),
      lastOpened: asIntOrNull(data['last_opened']),
      missing: asBool(data['missing']),
      error: data['error'] is String ? data['error'] as String : null,
      originPath: data['origin_path'] is String
          ? data['origin_path'] as String
          : null,
      raw: data,
    );
  }

  Map<String, dynamic> toJson() => <String, dynamic>{
    ..._raw,
    'path': path,
    'title': title,
    'author': author,
    'lang': lang,
    if (series != null) 'series': series,
    if (seriesIndex != null) 'series_index': seriesIndex,
    'chapters': chapters,
    'progress': progress.toJson(),
    if (cover != null) 'cover': cover,
    'size': size,
    'mtime': mtime,
    'added': added,
    if (lastOpened != null) 'last_opened': lastOpened,
    'missing': missing,
    if (error != null) 'error': error,
    if (originPath != null) 'origin_path': originPath,
  };

  String get fileName => path.split(RegExp(r'[\\/]')).last;
}

class BookLibrary {
  BookLibrary({
    this.schemaVersion = 1,
    List<String>? folders,
    List<BookEntry>? books,
    Map<String, dynamic>? raw,
  }) : folders = folders ?? <String>[],
       books = books ?? <BookEntry>[],
       _raw = raw ?? <String, dynamic>{};

  int schemaVersion;
  final List<String> folders;
  final List<BookEntry> books;
  final Map<String, dynamic> _raw;

  factory BookLibrary.fromJson(Map<String, dynamic>? json) {
    final data = json ?? const <String, dynamic>{};
    final books = (data['books'] is List)
        ? (data['books'] as List)
              .whereType<Map>()
              .map((item) => BookEntry.fromJson(asMap(item)))
              .toList()
        : <BookEntry>[];
    return BookLibrary(
      schemaVersion: asInt(data['schema_version'], 1),
      folders: asStringList(data['folders']),
      books: books,
      raw: data,
    );
  }

  Map<String, dynamic> toJson() => <String, dynamic>{
    ..._raw,
    'schema_version': schemaVersion,
    'folders': folders,
    'books': books.map((book) => book.toJson()).toList(),
  };

  List<BookEntry> search(String query, {int order = 0}) {
    final needle = fold(query.trim());
    final result = books.where((book) {
      if (needle.isEmpty) return true;
      return fold(book.title).contains(needle) ||
          fold(book.author).contains(needle) ||
          fold(book.series ?? '').contains(needle) ||
          fold(book.lang).contains(needle) ||
          fold(book.fileName).contains(needle);
    }).toList();

    switch (order) {
      case 1:
        result.sort((a, b) => fold(a.title).compareTo(fold(b.title)));
      case 2:
        result.sort((a, b) {
          final byAuthor = fold(a.author).compareTo(fold(b.author));
          return byAuthor != 0
              ? byAuthor
              : fold(a.title).compareTo(fold(b.title));
        });
      default:
        result.sort((a, b) {
          final left = a.lastOpened ?? a.added;
          final right = b.lastOpened ?? b.added;
          return right.compareTo(left);
        });
    }
    return result;
  }
}

String fold(String text) => text
    .toLowerCase()
    .replaceAll(RegExp('[áàäâã]'), 'a')
    .replaceAll(RegExp('[éèëê]'), 'e')
    .replaceAll(RegExp('[íìïî]'), 'i')
    .replaceAll(RegExp('[óòöôõ]'), 'o')
    .replaceAll(RegExp('[úùüû]'), 'u')
    .replaceAll('ñ', 'n')
    .replaceAll('ç', 'c');

class AnnotationItem {
  AnnotationItem({
    required this.id,
    required this.book,
    required this.chapter,
    required this.block,
    this.start = 0,
    required this.end,
    this.endBlock,
    List<String>? parts,
    required this.quote,
    this.note = '',
    this.color = 0,
    this.origin = 'user',
    this.created = 0,
    Map<String, dynamic>? raw,
  }) : parts = parts ?? <String>[],
       _raw = raw ?? <String, dynamic>{};

  int id;
  String book;
  int chapter;
  int block;
  int start;
  int end;
  int? endBlock;
  List<String> parts;
  String quote;
  String note;
  int color;
  String origin;
  int created;
  final Map<String, dynamic> _raw;

  factory AnnotationItem.fromJson(Map<String, dynamic> json) {
    final data = asMap(json);
    return AnnotationItem(
      id: asInt(data['id']),
      book: asString(data['book']),
      chapter: asInt(data['chapter']),
      block: asInt(data['block']),
      start: asInt(data['start']),
      end: asInt(data['end']),
      endBlock: asIntOrNull(data['end_block']),
      parts: asStringList(data['parts']),
      quote: asString(data['quote']),
      note: asString(data['note']),
      color: asInt(data['color']),
      origin: asString(data['origin'], 'user'),
      created: asInt(data['created']),
      raw: data,
    );
  }

  Map<String, dynamic> toJson() => <String, dynamic>{
    ..._raw,
    'id': id,
    'book': book,
    'chapter': chapter,
    'block': block,
    'start': start,
    'end': end,
    if (endBlock != null) 'end_block': endBlock,
    'parts': parts,
    'quote': quote,
    'note': note,
    'color': color,
    'origin': origin,
    'created': created,
  };
}

class AnnotationStore {
  AnnotationStore({
    this.schemaVersion = 3,
    List<AnnotationItem>? items,
    Map<String, dynamic>? raw,
  }) : items = items ?? <AnnotationItem>[],
       _raw = raw ?? <String, dynamic>{};

  int schemaVersion;
  final List<AnnotationItem> items;
  final Map<String, dynamic> _raw;

  factory AnnotationStore.fromJson(Map<String, dynamic>? json) {
    final data = json ?? const <String, dynamic>{};
    final items = (data['items'] is List)
        ? (data['items'] as List)
              .whereType<Map>()
              .map((item) => AnnotationItem.fromJson(asMap(item)))
              .toList()
        : <AnnotationItem>[];
    return AnnotationStore(
      schemaVersion: asInt(data['schema_version'], 3),
      items: items,
      raw: data,
    );
  }

  Map<String, dynamic> toJson() => <String, dynamic>{
    ..._raw,
    'schema_version': schemaVersion,
    'items': items.map((item) => item.toJson()).toList(),
  };

  List<AnnotationItem> forBook(String path) {
    final result = items.where((item) => item.book == path).toList();
    result.sort((a, b) {
      final byChapter = a.chapter.compareTo(b.chapter);
      if (byChapter != 0) return byChapter;
      final byBlock = a.block.compareTo(b.block);
      if (byBlock != 0) return byBlock;
      return a.start.compareTo(b.start);
    });
    return result;
  }

  int nextId() =>
      items.fold<int>(0, (max, item) => item.id > max ? item.id : max) + 1;

  void rekey(String oldPath, String newPath) {
    for (final item in items) {
      if (item.book == oldPath) item.book = newPath;
    }
  }
}

class TocEntry {
  const TocEntry({required this.title, required this.spine, this.anchor});

  final String title;
  final int spine;
  final String? anchor;

  factory TocEntry.fromJson(Map<String, dynamic> json) => TocEntry(
    title: asString(json['title']),
    spine: asInt(json['spine']),
    anchor: json['anchor'] is String ? json['anchor'] as String : null,
  );
}

class BookMetadata {
  const BookMetadata({
    required this.path,
    required this.title,
    required this.author,
    required this.lang,
    required this.chapters,
    required this.toc,
    this.series,
    this.seriesIndex,
  });

  final String path;
  final String title;
  final String author;
  final String lang;
  final int chapters;
  final List<TocEntry> toc;
  final String? series;
  final double? seriesIndex;

  factory BookMetadata.fromJson(Map<String, dynamic> json) => BookMetadata(
    path: asString(json['path']),
    title: asString(json['title'], 'Sin título'),
    author: asString(json['author']),
    lang: asString(json['lang']),
    chapters: asInt(json['chapters']),
    series: json['series'] is String ? json['series'] as String : null,
    seriesIndex: asDoubleOrNull(json['series_index']),
    toc: (json['toc'] is List)
        ? (json['toc'] as List)
              .whereType<Map>()
              .map((item) => TocEntry.fromJson(asMap(item)))
              .toList()
        : <TocEntry>[],
  );
}

class RunSpan {
  const RunSpan({
    required this.text,
    this.bold = false,
    this.italic = false,
    this.small = false,
    this.link = false,
    this.href,
  });

  final String text;
  final bool bold;
  final bool italic;
  final bool small;
  final bool link;
  final String? href;

  factory RunSpan.fromJson(Map<String, dynamic> json) => RunSpan(
    text: asString(json['text']),
    bold: asBool(json['bold']),
    italic: asBool(json['italic']),
    small: asBool(json['small']),
    link: asBool(json['link']),
    href: json['href'] is String ? json['href'] as String : null,
  );
}

class ReaderBlock {
  const ReaderBlock({
    required this.kind,
    required this.text,
    required this.runs,
    this.level,
    this.number,
    this.src,
    this.alt = '',
    this.anchor,
  });

  final String kind;
  final int? level;
  final int? number;
  final String text;
  final String? src;
  final String alt;
  final String? anchor;
  final List<RunSpan> runs;

  bool get isImage => kind == 'image';

  factory ReaderBlock.fromJson(Map<String, dynamic> json) => ReaderBlock(
    kind: asString(json['kind'], 'para'),
    level: asIntOrNull(json['level']),
    number: asIntOrNull(json['number']),
    text: asString(json['text']),
    src: json['src'] is String ? json['src'] as String : null,
    alt: asString(json['alt']),
    anchor: json['anchor'] is String ? json['anchor'] as String : null,
    runs: (json['runs'] is List)
        ? (json['runs'] as List)
              .whereType<Map>()
              .map((item) => RunSpan.fromJson(asMap(item)))
              .toList()
        : <RunSpan>[],
  );
}

class ReaderChapter {
  const ReaderChapter({
    required this.index,
    required this.title,
    required this.blocks,
  });

  final int index;
  final String title;
  final List<ReaderBlock> blocks;

  factory ReaderChapter.fromJson(Map<String, dynamic> json) => ReaderChapter(
    index: asInt(json['index']),
    title: asString(json['title']),
    blocks: (json['blocks'] is List)
        ? (json['blocks'] as List)
              .whereType<Map>()
              .map((item) => ReaderBlock.fromJson(asMap(item)))
              .toList()
        : <ReaderBlock>[],
  );
}

class SearchHit {
  const SearchHit({
    required this.chapter,
    required this.block,
    required this.start,
    required this.end,
    required this.before,
    required this.matched,
    required this.after,
  });

  final int chapter;
  final int block;
  final int start;
  final int end;
  final String before;
  final String matched;
  final String after;

  factory SearchHit.fromJson(Map<String, dynamic> json) => SearchHit(
    chapter: asInt(json['chapter']),
    block: asInt(json['block']),
    start: asInt(json['start']),
    end: asInt(json['end']),
    before: asString(json['before']),
    matched: asString(json['matched']),
    after: asString(json['after']),
  );

  String get preview => '$before$matched$after';
}

Map<String, dynamic> decodeJsonMap(List<int> bytes) =>
    asMap(jsonDecode(utf8.decode(bytes)));
