import 'dart:io';

import 'package:file_picker/file_picker.dart';
import 'package:material_ui/material_ui.dart';
import 'package:path/path.dart' as p;

import '../app_store.dart';
import '../models.dart';
import '../theme.dart';
import '../widgets/book_card.dart';
import 'annotations_screen.dart';
import 'reader_screen.dart';
import 'settings_screen.dart';

class LibraryScreen extends StatefulWidget {
  const LibraryScreen({super.key, required this.store});

  final AppStore store;

  @override
  State<LibraryScreen> createState() => _LibraryScreenState();
}

class _LibraryScreenState extends State<LibraryScreen> {
  final _search = TextEditingController();
  int _order = 0;
  bool _busy = false;

  @override
  void initState() {
    super.initState();
    widget.store.addListener(_onStoreChanged);
  }

  void _onStoreChanged() {
    if (mounted) setState(() {});
  }

  @override
  void dispose() {
    widget.store.removeListener(_onStoreChanged);
    _search.dispose();
    super.dispose();
  }

  Future<void> _importBooks() async {
    final files = await FilePicker.pickFiles(
      type: FileType.custom,
      allowedExtensions: const <String>['epub'],
    );
    final paths = files.map((file) => file.path).whereType<String>().toList();
    if (paths.isEmpty) return;
    if (!mounted) return;
    setState(() => _busy = true);
    try {
      await widget.store.importBooks(paths);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _importSpecificBook(BookEntry book) async {
    final file = await FilePicker.pickFile(
      type: FileType.custom,
      allowedExtensions: const <String>['epub'],
    );
    final path = file?.path;
    if (path == null) return;
    setState(() => _busy = true);
    try {
      await widget.store.importBooks(<String>[path]);
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _importBackup() async {
    final file = await FilePicker.pickFile(
      type: FileType.custom,
      allowedExtensions: const <String>['zip'],
    );
    final path = file?.path;
    if (path == null) return;
    setState(() => _busy = true);
    try {
      await widget.store.importBackup(path);
    } catch (error) {
      _showMessage(error.toString());
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _exportBackup() async {
    setState(() => _busy = true);
    try {
      await widget.store.exportBackup();
    } catch (error) {
      _showMessage(error.toString());
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  void _showMessage(String text) {
    if (!mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(text)));
  }

  void _openReader(BookEntry book) {
    if (book.missing) {
      showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('Este libro todavía no está en el móvil'),
          content: const Text(
            'Elige el mismo EPUB para completar la entrada importada del '
            'escritorio. Candil conservará el progreso y las anotaciones.',
          ),
          actions: <Widget>[
            TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('Cancelar'),
            ),
            FilledButton(
              onPressed: () {
                Navigator.pop(context);
                _importSpecificBook(book);
              },
              child: const Text('Elegir EPUB'),
            ),
          ],
        ),
      );
      return;
    }
    Navigator.of(context)
        .push(
          MaterialPageRoute<void>(
            builder: (_) =>
                ReaderScreen(store: widget.store, bookPath: book.path),
          ),
        )
        .then((_) => setState(() {}));
  }

  void _openAnnotations(BookEntry book) {
    Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => AnnotationsScreen(
          store: widget.store,
          bookPath: book.path,
          bookTitle: book.title,
        ),
      ),
    );
  }

  void _showBookMenu(BookEntry book) {
    showModalBottomSheet<void>(
      context: context,
      showDragHandle: true,
      builder: (context) {
        final theme = themeFor(widget.store.settings.theme);
        return SafeArea(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              ListTile(
                leading: const Icon(Icons.menu_book_outlined),
                title: const Text('Abrir'),
                onTap: () {
                  Navigator.pop(context);
                  _openReader(book);
                },
              ),
              ListTile(
                leading: const Icon(Icons.format_quote_outlined),
                title: const Text('Anotaciones'),
                onTap: () {
                  Navigator.pop(context);
                  _openAnnotations(book);
                },
              ),
              if (book.missing)
                ListTile(
                  leading: const Icon(Icons.file_open_outlined),
                  title: const Text('Elegir EPUB para completar'),
                  onTap: () {
                    Navigator.pop(context);
                    _importSpecificBook(book);
                  },
                ),
              ListTile(
                leading: Icon(Icons.delete_outline, color: theme.error),
                title: Text(
                  'Quitar del móvil',
                  style: TextStyle(color: theme.error),
                ),
                onTap: () {
                  Navigator.pop(context);
                  _confirmRemove(book);
                },
              ),
            ],
          ),
        );
      },
    );
  }

  void _confirmRemove(BookEntry book) {
    showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('¿Quitar este libro?'),
        content: Text(
          'Se quitará «${book.title}» de la biblioteca del móvil y se borrará '
          'la copia local. El EPUB original no se toca.',
        ),
        actions: <Widget>[
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () {
              Navigator.pop(context);
              widget.store.removeBook(book);
            },
            child: const Text('Quitar'),
          ),
        ],
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    final store = widget.store;
    final theme = themeFor(store.settings.theme);
    final books = store.library.search(_search.text, order: _order);

    return Scaffold(
      appBar: AppBar(
        title: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: <Widget>[
            const Text('Candil', style: TextStyle(fontWeight: FontWeight.w600)),
            Text(
              '${store.library.books.length} '
              '${store.library.books.length == 1 ? 'libro' : 'libros'}',
              style: TextStyle(fontSize: 12, color: theme.dim),
            ),
          ],
        ),
        actions: <Widget>[
          IconButton(
            tooltip: 'Importar EPUB',
            onPressed: _busy ? null : _importBooks,
            icon: const Icon(Icons.add),
          ),
          PopupMenuButton<String>(
            enabled: !_busy,
            onSelected: (value) {
              switch (value) {
                case 'backup-import':
                  _importBackup();
                case 'backup-export':
                  _exportBackup();
                case 'settings':
                  Navigator.of(context).push(
                    MaterialPageRoute<void>(
                      builder: (_) => SettingsScreen(store: store),
                    ),
                  );
              }
            },
            itemBuilder: (context) => const <PopupMenuEntry<String>>[
              PopupMenuItem<String>(
                value: 'backup-import',
                child: Text('Importar respaldo de Candil…'),
              ),
              PopupMenuItem<String>(
                value: 'backup-export',
                child: Text('Exportar respaldo…'),
              ),
              PopupMenuDivider(),
              PopupMenuItem<String>(value: 'settings', child: Text('Ajustes')),
            ],
          ),
        ],
      ),
      body: SafeArea(
        child: Column(
          children: <Widget>[
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 8, 16, 8),
              child: Row(
                children: <Widget>[
                  Expanded(
                    child: TextField(
                      controller: _search,
                      onChanged: (_) => setState(() {}),
                      decoration: InputDecoration(
                        hintText: 'Buscar por título, autor, serie o archivo',
                        prefixIcon: const Icon(Icons.search),
                        suffixIcon: _search.text.isEmpty
                            ? null
                            : IconButton(
                                tooltip: 'Limpiar',
                                onPressed: () {
                                  _search.clear();
                                  setState(() {});
                                },
                                icon: const Icon(Icons.close),
                              ),
                      ),
                    ),
                  ),
                  const SizedBox(width: 8),
                  PopupMenuButton<int>(
                    tooltip: 'Ordenar',
                    initialValue: _order,
                    onSelected: (value) => setState(() => _order = value),
                    icon: const Icon(Icons.sort),
                    itemBuilder: (context) => const <PopupMenuEntry<int>>[
                      PopupMenuItem<int>(value: 0, child: Text('Recientes')),
                      PopupMenuItem<int>(value: 1, child: Text('Título')),
                      PopupMenuItem<int>(value: 2, child: Text('Autor')),
                    ],
                  ),
                ],
              ),
            ),
            if (_busy) const LinearProgressIndicator(minHeight: 2),
            Expanded(
              child: books.isEmpty
                  ? _EmptyLibrary(store: store, search: _search.text)
                  : LayoutBuilder(
                      builder: (context, constraints) {
                        final width = constraints.maxWidth;
                        final columns = width >= 1100
                            ? 6
                            : width >= 840
                            ? 5
                            : width >= 620
                            ? 4
                            : width >= 420
                            ? 3
                            : 2;
                        return GridView.builder(
                          padding: const EdgeInsets.fromLTRB(16, 8, 16, 24),
                          gridDelegate:
                              SliverGridDelegateWithFixedCrossAxisCount(
                                crossAxisCount: columns,
                                mainAxisSpacing: 12,
                                crossAxisSpacing: 12,
                                childAspectRatio: 0.56,
                              ),
                          itemCount: books.length,
                          itemBuilder: (context, index) {
                            final book = books[index];
                            final cover = book.cover == null
                                ? null
                                : File(
                                    p.join(store.data.rootPath, book.cover!),
                                  );
                            return BookCard(
                              book: book,
                              theme: theme,
                              coverFile: cover,
                              onTap: () => _openReader(book),
                              onMenu: () => _showBookMenu(book),
                            );
                          },
                        );
                      },
                    ),
            ),
          ],
        ),
      ),
    );
  }
}

class _EmptyLibrary extends StatelessWidget {
  const _EmptyLibrary({required this.store, required this.search});

  final AppStore store;
  final String search;

  @override
  Widget build(BuildContext context) {
    final theme = themeFor(store.settings.theme);
    final filtered = search.trim().isNotEmpty;
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: <Widget>[
            Icon(
              filtered ? Icons.search_off : Icons.auto_stories_outlined,
              size: 56,
              color: theme.dim,
            ),
            const SizedBox(height: 16),
            Text(
              filtered ? 'Sin resultados' : 'Tu biblioteca está vacía',
              style: TextStyle(
                color: theme.fg,
                fontSize: 22,
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              filtered
                  ? 'Prueba con otro título, autor o nombre de archivo.'
                  : 'Importa uno o varios EPUB para empezar. También puedes '
                        'restaurar un respaldo de Candil desde el menú.',
              textAlign: TextAlign.center,
              style: TextStyle(color: theme.dim, height: 1.45),
            ),
          ],
        ),
      ),
    );
  }
}
