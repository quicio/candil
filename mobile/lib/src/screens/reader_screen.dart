import 'dart:async';
import 'dart:typed_data';

import 'package:material_ui/material_ui.dart';

import '../app_store.dart';
import '../models.dart';
import '../rust_core.dart';
import '../theme.dart';
import '../widgets/focus_overlay.dart';
import 'annotations_screen.dart';

class ReaderScreen extends StatefulWidget {
  const ReaderScreen({super.key, required this.store, required this.bookPath});

  final AppStore store;
  final String bookPath;

  @override
  State<ReaderScreen> createState() => _ReaderScreenState();
}

class _ReaderScreenState extends State<ReaderScreen> {
  final ScrollController _scroll = ScrollController();
  final Map<String, Uint8List?> _resources = <String, Uint8List?>{};
  final Map<int, GlobalKey> _blockKeys = <int, GlobalKey>{};

  RustBook? _rust;
  BookMetadata? _metadata;
  ReaderChapter? _chapter;
  BookEntry? _entry;
  Timer? _saveTimer;
  int _index = 0;
  bool _loading = true;
  String? _error;
  double _ratio = 0;

  @override
  void initState() {
    super.initState();
    widget.store.addListener(_onStoreChanged);
    _scroll.addListener(_onScroll);
    _open();
  }

  void _onStoreChanged() {
    if (mounted) setState(() {});
  }

  Future<void> _open() async {
    try {
      _entry = widget.store.bookAt(widget.bookPath);
      final rust = RustCore.instance.open(widget.bookPath);
      final metadata = rust.metadata();
      final index = (_entry?.progress.chapter ?? 0).clamp(
        0,
        (metadata.chapters - 1).clamp(0, 1 << 30),
      );
      if (!mounted) {
        rust.close();
        return;
      }
      setState(() {
        _rust = rust;
        _metadata = metadata;
      });
      await _loadChapter(index, restoreRatio: _entry?.progress.ratio ?? 0);
    } catch (error) {
      if (mounted) {
        setState(() {
          _error = error.toString();
          _loading = false;
        });
      }
    }
  }

  Future<void> _loadChapter(int index, {double? restoreRatio}) async {
    final rust = _rust;
    if (rust == null) return;
    setState(() {
      _loading = true;
      _error = null;
      _index = index;
      _blockKeys.clear();
    });
    try {
      final chapter = rust.chapter(index);
      if (!mounted) return;
      _chapter = chapter;
      setState(() {
        _loading = false;
        _ratio = 0;
      });
      if (restoreRatio != null && restoreRatio > 0) {
        WidgetsBinding.instance.addPostFrameCallback(
          (_) => _jumpRatio(restoreRatio),
        );
      } else if (_scroll.hasClients) {
        _scroll.jumpTo(0);
      }
    } catch (error) {
      if (mounted) {
        setState(() {
          _loading = false;
          _error = error.toString();
        });
      }
    }
  }

  void _jumpRatio(double ratio) {
    if (!_scroll.hasClients) return;
    final max = _scroll.position.maxScrollExtent;
    _scroll.jumpTo((max * ratio).clamp(0, max));
  }

  void _onScroll() {
    if (!_scroll.hasClients) return;
    final max = _scroll.position.maxScrollExtent;
    _ratio = max <= 0 ? 0 : (_scroll.offset / max).clamp(0.0, 1.0);
    _saveTimer?.cancel();
    _saveTimer = Timer(const Duration(milliseconds: 650), _saveProgress);
  }

  Future<void> _saveProgress() async {
    final path = widget.bookPath;
    final entry = widget.store.bookAt(path);
    final progress = BookProgress(chapter: _index, ratio: _ratio);
    if (entry != null &&
        entry.progress.chapter == progress.chapter &&
        (entry.progress.ratio - progress.ratio).abs() < 0.005) {
      return;
    }
    await widget.store.saveProgress(path, progress);
  }

  Future<void> _goToChapter(int index) async {
    final metadata = _metadata;
    if (metadata == null) return;
    if (index < 0 || index >= metadata.chapters || index == _index) return;
    await _saveProgress();
    await _loadChapter(index);
  }

  @override
  void dispose() {
    widget.store.removeListener(_onStoreChanged);
    _saveTimer?.cancel();
    unawaited(_saveProgress());
    _scroll.dispose();
    _rust?.close();
    super.dispose();
  }

  void _showToc() {
    final metadata = _metadata;
    if (metadata == null) return;
    showModalBottomSheet<void>(
      context: context,
      showDragHandle: true,
      isScrollControlled: true,
      builder: (context) {
        final theme = themeFor(widget.store.settings.theme);
        return DraggableScrollableSheet(
          expand: false,
          initialChildSize: 0.75,
          maxChildSize: 0.95,
          minChildSize: 0.35,
          builder: (context, controller) => ListView.builder(
            controller: controller,
            itemCount: metadata.toc.length,
            itemBuilder: (context, index) {
              final item = metadata.toc[index];
              final selected = item.spine == _index;
              return ListTile(
                selected: selected,
                selectedColor: theme.accent,
                title: Text(item.title),
                subtitle: Text('Capítulo ${item.spine + 1}'),
                onTap: () {
                  Navigator.pop(context);
                  _goToChapter(item.spine);
                },
              );
            },
          ),
        );
      },
    );
  }

  void _showAnnotations() {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => AnnotationsScreen(
          store: widget.store,
          bookPath: widget.bookPath,
          bookTitle: _metadata?.title ?? _entry?.title ?? 'Libro',
        ),
      ),
    );
  }

  Future<void> _showSearch() async {
    final rust = _rust;
    if (rust == null) return;
    final controller = TextEditingController();
    var results = <SearchHit>[];
    var truncated = false;
    var busy = false;
    await showModalBottomSheet<void>(
      context: context,
      showDragHandle: true,
      isScrollControlled: true,
      builder: (context) {
        final theme = themeFor(widget.store.settings.theme);
        return StatefulBuilder(
          builder: (context, setModalState) {
            Future<void> run() async {
              final query = controller.text.trim();
              if (query.length < 2) return;
              setModalState(() => busy = true);
              try {
                final found = rust.search(query, max: 80);
                setModalState(() {
                  results = found.hits;
                  truncated = found.truncated;
                });
              } catch (error) {
                setModalState(() => results = <SearchHit>[]);
                if (context.mounted) {
                  ScaffoldMessenger.of(context)
                      .showSnackBar(SnackBar(content: Text(error.toString())));
                }
              } finally {
                setModalState(() => busy = false);
              }
            }

            return Padding(
              padding: EdgeInsets.only(
                left: 16,
                right: 16,
                bottom: MediaQuery.viewInsetsOf(context).bottom + 16,
              ),
              child: SizedBox(
                height: MediaQuery.sizeOf(context).height * 0.7,
                child: Column(
                  children: <Widget>[
                    TextField(
                      controller: controller,
                      autofocus: true,
                      textInputAction: TextInputAction.search,
                      onSubmitted: (_) => run(),
                      decoration: InputDecoration(
                        hintText: 'Buscar dentro del libro',
                        prefixIcon: const Icon(Icons.search),
                        suffixIcon: IconButton(
                          tooltip: 'Buscar',
                          onPressed: busy ? null : run,
                          icon: const Icon(Icons.arrow_forward),
                        ),
                      ),
                    ),
                    if (busy) const LinearProgressIndicator(minHeight: 2),
                    if (truncated)
                      Padding(
                        padding: const EdgeInsets.only(top: 8),
                        child: Text(
                          'Mostrando los primeros resultados.',
                          style: TextStyle(color: theme.dim, fontSize: 12),
                        ),
                      ),
                    Expanded(
                      child: results.isEmpty
                          ? Center(
                              child: Text(
                                busy
                                    ? 'Buscando…'
                                    : 'Escribí al menos 2 caracteres.',
                                style: TextStyle(color: theme.dim),
                              ),
                            )
                          : ListView.builder(
                              itemCount: results.length,
                              itemBuilder: (context, index) {
                                final hit = results[index];
                                return ListTile(
                                  title: Text(
                                    hit.preview.replaceAll(RegExp(r'\s+'), ' '),
                                    maxLines: 3,
                                    overflow: TextOverflow.ellipsis,
                                  ),
                                  subtitle: Text('Capítulo ${hit.chapter + 1}'),
                                  onTap: () {
                                    Navigator.pop(context);
                                    _jumpToHit(hit);
                                  },
                                );
                              },
                            ),
                    ),
                  ],
                ),
              ),
            );
          },
        );
      },
    );
    controller.dispose();
  }

  Future<void> _jumpToHit(SearchHit hit) async {
    await _loadChapter(hit.chapter);
    WidgetsBinding.instance.addPostFrameCallback((_) {
      final context = _blockKeys[hit.block]?.currentContext;
      if (context != null) {
        Scrollable.ensureVisible(
          context,
          duration: const Duration(milliseconds: 250),
          alignment: 0.25,
        );
      }
    });
  }

  Future<void> _showReadingSettings() async {
    final store = widget.store;
    var settings = store.settings;
    await showModalBottomSheet<void>(
      context: context,
      showDragHandle: true,
      isScrollControlled: true,
      builder: (context) {
        final theme = themeFor(settings.theme);
        return StatefulBuilder(
          builder: (context, setModalState) {
            void update(Settings next) {
              setModalState(() => settings = next);
              store.saveSettings(next);
            }

            return SafeArea(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(20, 8, 20, 24),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: <Widget>[
                    Text(
                      'Lectura',
                      style: TextStyle(color: theme.fg, fontSize: 20),
                    ),
                    const SizedBox(height: 16),
                    Text(
                      'Tamaño ${settings.fontSize.round()}',
                      style: TextStyle(color: theme.dim),
                    ),
                    Slider(
                      value: settings.fontSize,
                      min: 14,
                      max: 32,
                      divisions: 18,
                      onChanged: (value) =>
                          update(settings.copyWith(fontSize: value)),
                    ),
                    Text(
                      'Interlínea ${settings.lineHeight.toStringAsFixed(2)}',
                      style: TextStyle(color: theme.dim),
                    ),
                    Slider(
                      value: settings.lineHeight,
                      min: 1.2,
                      max: 2.2,
                      divisions: 10,
                      onChanged: (value) =>
                          update(settings.copyWith(lineHeight: value)),
                    ),
                    SwitchListTile(
                      contentPadding: EdgeInsets.zero,
                      title: const Text('Modo enfoque'),
                      subtitle: const Text(
                        'Deja clara la línea central y atenúa el resto. '
                        'El inicio y el final del capítulo no se velan.',
                      ),
                      value: settings.focusMode,
                      onChanged: (value) =>
                          update(settings.copyWith(focusMode: value)),
                    ),
                  ],
                ),
              ),
            );
          },
        );
      },
    );
  }

  Future<void> _showBlockActions(int blockIndex, ReaderBlock block) async {
    final quote = block.text.trim();
    if (quote.isEmpty || block.isImage) return;
    final existing = _annotationFor(blockIndex);
    final theme = themeFor(widget.store.settings.theme);
    final note = TextEditingController(text: existing?.note ?? '');
    await showModalBottomSheet<void>(
      context: context,
      showDragHandle: true,
      isScrollControlled: true,
      builder: (context) {
        return Padding(
          padding: EdgeInsets.only(
            left: 20,
            right: 20,
            bottom: MediaQuery.viewInsetsOf(context).bottom + 24,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              Text(
                quote.length > 180 ? '${quote.substring(0, 180)}…' : quote,
                style: TextStyle(
                  color: theme.fg,
                  fontStyle: FontStyle.italic,
                  height: 1.4,
                ),
              ),
              const SizedBox(height: 12),
              TextField(
                controller: note,
                maxLines: 3,
                decoration: const InputDecoration(hintText: 'Nota (opcional)'),
              ),
              const SizedBox(height: 12),
              Row(
                children: <Widget>[
                  Expanded(
                    child: FilledButton.icon(
                      onPressed: () async {
                        Navigator.pop(context);
                        if (existing == null) {
                          await widget.store.addAnnotation(
                            AnnotationItem(
                              id: 0,
                              book: widget.bookPath,
                              chapter: _index,
                              block: blockIndex,
                              start: 0,
                              end: block.text.length,
                              parts: <String>[block.text],
                              quote: quote,
                              note: note.text.trim(),
                            ),
                          );
                        } else {
                          await widget.store.updateAnnotationNote(
                            existing,
                            note.text.trim(),
                          );
                        }
                      },
                      icon: const Icon(Icons.highlight_outlined),
                      label: Text(
                        existing == null ? 'Destacar' : 'Guardar nota',
                      ),
                    ),
                  ),
                  if (existing != null) ...<Widget>[
                    const SizedBox(width: 8),
                    IconButton(
                      tooltip: 'Quitar destacado',
                      onPressed: () async {
                        Navigator.pop(context);
                        await widget.store.removeAnnotation(existing);
                      },
                      icon: Icon(Icons.delete_outline, color: theme.error),
                    ),
                  ],
                ],
              ),
            ],
          ),
        );
      },
    );
    note.dispose();
  }

  AnnotationItem? _annotationFor(int blockIndex) {
    for (final item in widget.store.annotations.items) {
      if (item.book == widget.bookPath &&
          item.chapter == _index &&
          item.block == blockIndex) {
        return item;
      }
    }
    return null;
  }

  @override
  Widget build(BuildContext context) {
    final store = widget.store;
    final settings = store.settings;
    final theme = themeFor(settings.theme);
    final metadata = _metadata;
    final chapter = _chapter;

    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            Text(
              metadata?.title ?? _entry?.title ?? 'Candil',
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(fontSize: 16, fontWeight: FontWeight.w600),
            ),
            if (chapter != null)
              Text(
                chapter.title,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(fontSize: 12, color: theme.dim),
              ),
          ],
        ),
        actions: <Widget>[
          IconButton(
            tooltip: 'Buscar en el libro',
            onPressed: _rust == null ? null : _showSearch,
            icon: const Icon(Icons.search),
          ),
          IconButton(
            tooltip: 'Anotaciones',
            onPressed: _showAnnotations,
            icon: const Icon(Icons.format_quote_outlined),
          ),
          IconButton(
            tooltip: 'Índice',
            onPressed: metadata == null ? null : _showToc,
            icon: const Icon(Icons.list),
          ),
          IconButton(
            tooltip: 'Lectura',
            onPressed: _showReadingSettings,
            icon: const Icon(Icons.text_fields),
          ),
        ],
      ),
      body: SafeArea(
        child: Stack(
          children: <Widget>[
            if (_error != null)
              Center(
                child: Padding(
                  padding: const EdgeInsets.all(24),
                  child: Text(
                    _error!,
                    textAlign: TextAlign.center,
                    style: TextStyle(color: theme.error),
                  ),
                ),
              )
            else if (_loading || chapter == null)
              const Center(child: CircularProgressIndicator())
            else
              SingleChildScrollView(
                controller: _scroll,
                padding: EdgeInsets.fromLTRB(
                  MediaQuery.sizeOf(context).width > 760 ? 32 : 22,
                  20,
                  MediaQuery.sizeOf(context).width > 760 ? 32 : 22,
                  80,
                ),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: <Widget>[
                    Text(
                      chapter.title,
                      style: readingStyle(settings, theme).copyWith(
                        fontSize: settings.fontSize * 1.45,
                        fontWeight: FontWeight.w700,
                        height: 1.2,
                      ),
                    ),
                    SizedBox(height: settings.fontSize * 1.1),
                    for (var index = 0; index < chapter.blocks.length; index++)
                      _blockWidget(
                        index,
                        chapter.blocks[index],
                        settings,
                        theme,
                      ),
                  ],
                ),
              ),
            if (settings.focusMode && _error == null && !_loading)
              Positioned.fill(
                child: FocusOverlay(controller: _scroll, theme: theme),
              ),
          ],
        ),
      ),
      bottomNavigationBar: metadata == null
          ? null
          : SafeArea(
              child: Padding(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                child: Row(
                  children: <Widget>[
                    IconButton(
                      tooltip: 'Capítulo anterior',
                      onPressed: _index > 0
                          ? () => _goToChapter(_index - 1)
                          : null,
                      icon: const Icon(Icons.chevron_left),
                    ),
                    Expanded(
                      child: Text(
                        '${_index + 1} / ${metadata.chapters}'
                        '${_ratio > 0 ? ' · ${(_ratio * 100).round()}%' : ''}',
                        textAlign: TextAlign.center,
                        style: TextStyle(color: theme.dim, fontSize: 12),
                      ),
                    ),
                    IconButton(
                      tooltip: 'Capítulo siguiente',
                      onPressed: _index < metadata.chapters - 1
                          ? () => _goToChapter(_index + 1)
                          : null,
                      icon: const Icon(Icons.chevron_right),
                    ),
                  ],
                ),
              ),
            ),
    );
  }

  Widget _blockWidget(
    int index,
    ReaderBlock block,
    Settings settings,
    CandilTheme theme,
  ) {
    final key = _blockKeys.putIfAbsent(index, GlobalKey.new);
    final annotation = _annotationFor(index);
    final base = readingStyle(settings, theme);
    final spacing = settings.fontSize * settings.paragraphSpace;
    Widget child;

    if (block.isImage) {
      child = Padding(
        padding: EdgeInsets.symmetric(vertical: spacing),
        child: _image(block),
      );
    } else {
      final text = _richText(block, base, theme, settings);
      switch (block.kind) {
        case 'heading':
          final factor = switch (block.level ?? 2) {
            1 => 1.5,
            2 => 1.28,
            _ => 1.12,
          };
          child = Padding(
            padding: EdgeInsets.only(top: spacing * 1.4, bottom: spacing * 0.6),
            child: DefaultTextStyle.merge(
              style: base.copyWith(
                fontSize: settings.fontSize * factor,
                fontWeight: FontWeight.w700,
                height: 1.22,
              ),
              child: text,
            ),
          );
        case 'quote':
          child = Container(
            margin: EdgeInsets.symmetric(vertical: spacing),
            padding: const EdgeInsets.only(left: 14),
            decoration: BoxDecoration(
              border: Border(left: BorderSide(color: theme.accent, width: 3)),
            ),
            child: DefaultTextStyle.merge(
              style: base.copyWith(
                color: theme.dim,
                fontStyle: FontStyle.italic,
              ),
              child: text,
            ),
          );
        case 'bullet':
        case 'numbered':
          final marker = block.kind == 'numbered'
              ? '${block.number ?? 1}.'
              : '•';
          child = Padding(
            padding: EdgeInsets.symmetric(vertical: spacing * 0.35),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: <Widget>[
                SizedBox(width: 26, child: Text(marker, style: base)),
                Expanded(child: text),
              ],
            ),
          );
        case 'rule':
          child = Padding(
            padding: EdgeInsets.symmetric(vertical: spacing * 1.2),
            child: Divider(color: theme.border, height: 1),
          );
        case 'pre':
          child = Container(
            margin: EdgeInsets.symmetric(vertical: spacing),
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: theme.panel,
              borderRadius: BorderRadius.circular(8),
            ),
            child: DefaultTextStyle.merge(
              style: base.copyWith(
                fontFamily: 'monospace',
                fontSize: settings.fontSize * 0.88,
              ),
              child: SingleChildScrollView(
                scrollDirection: Axis.horizontal,
                child: text,
              ),
            ),
          );
        case 'caption':
          child = Padding(
            padding: EdgeInsets.symmetric(vertical: spacing * 0.5),
            child: DefaultTextStyle.merge(
              style: base.copyWith(
                fontSize: settings.fontSize * 0.86,
                color: theme.dim,
                fontStyle: FontStyle.italic,
              ),
              child: text,
            ),
          );
        default:
          child = Padding(
            padding: EdgeInsets.symmetric(vertical: spacing),
            child: text,
          );
      }
    }

    return Container(
      key: key,
      decoration: annotation == null
          ? null
          : BoxDecoration(
              color: theme.selection.withValues(alpha: 0.55),
              borderRadius: BorderRadius.circular(4),
            ),
      padding: annotation == null
          ? null
          : const EdgeInsets.symmetric(horizontal: 4),
      child: GestureDetector(
        behavior: HitTestBehavior.opaque,
        onLongPress: () => _showBlockActions(index, block),
        child: child,
      ),
    );
  }

  Widget _richText(
    ReaderBlock block,
    TextStyle base,
    CandilTheme theme,
    Settings settings,
  ) {
    final runs = block.runs.isEmpty
        ? <TextSpan>[TextSpan(text: block.text)]
        : block.runs
              .map(
                (run) => TextSpan(
                  text: run.text,
                  style: base.copyWith(
                    fontWeight: run.bold ? FontWeight.w700 : null,
                    fontStyle: run.italic ? FontStyle.italic : null,
                    fontSize: run.small ? settings.fontSize * 0.85 : null,
                    color: run.link ? theme.accent : null,
                    decoration: run.link ? TextDecoration.underline : null,
                  ),
                ),
              )
              .toList();
    return Text.rich(TextSpan(style: base, children: runs));
  }

  Widget _image(ReaderBlock block) {
    final src = block.src;
    if (src == null) return const SizedBox.shrink();
    final bytes = _resources.containsKey(src)
        ? _resources[src]
        : _resources.putIfAbsent(src, () => _rust?.resource(src));
    if (bytes == null) {
      return Text(block.alt.isEmpty ? '[Imagen no disponible]' : block.alt);
    }
    return ClipRRect(
      borderRadius: BorderRadius.circular(8),
      child: Image.memory(bytes, fit: BoxFit.contain),
    );
  }
}
