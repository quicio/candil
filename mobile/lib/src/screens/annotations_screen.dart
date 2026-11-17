import 'package:material_ui/material_ui.dart';

import '../app_store.dart';
import '../models.dart';
import '../theme.dart';

class AnnotationsScreen extends StatelessWidget {
  const AnnotationsScreen({
    super.key,
    required this.store,
    required this.bookPath,
    required this.bookTitle,
  });

  final AppStore store;
  final String bookPath;
  final String bookTitle;

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: store,
      builder: (context, _) {
        final theme = themeFor(store.settings.theme);
        final items = store.annotations.forBook(bookPath);
        return Scaffold(
          appBar: AppBar(title: Text('Anotaciones · $bookTitle')),
          body: items.isEmpty
              ? Center(
                  child: Padding(
                    padding: const EdgeInsets.all(32),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: <Widget>[
                        Icon(
                          Icons.format_quote_outlined,
                          size: 48,
                          color: theme.dim,
                        ),
                        const SizedBox(height: 12),
                        Text(
                          'Todavía no hay anotaciones.',
                          style: TextStyle(color: theme.fg, fontSize: 18),
                        ),
                        const SizedBox(height: 6),
                        Text(
                          'Mantené pulsado un párrafo en el lector para '
                          'destacarlo o agregarle una nota.',
                          textAlign: TextAlign.center,
                          style: TextStyle(color: theme.dim, height: 1.4),
                        ),
                      ],
                    ),
                  ),
                )
              : ListView.separated(
                  padding: const EdgeInsets.all(16),
                  itemCount: items.length,
                  separatorBuilder: (_, _) => const SizedBox(height: 10),
                  itemBuilder: (context, index) {
                    final item = items[index];
                    return Container(
                      padding: const EdgeInsets.all(14),
                      decoration: BoxDecoration(
                        color: theme.panel,
                        borderRadius: BorderRadius.circular(12),
                        border: Border.all(color: theme.border),
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: <Widget>[
                          Text(
                            item.quote,
                            maxLines: 4,
                            overflow: TextOverflow.ellipsis,
                            style: TextStyle(
                              color: theme.fg,
                              fontStyle: FontStyle.italic,
                              height: 1.4,
                            ),
                          ),
                          if (item.note.trim().isNotEmpty) ...<Widget>[
                            const SizedBox(height: 10),
                            Text(item.note, style: TextStyle(color: theme.dim)),
                          ],
                          const SizedBox(height: 8),
                          Row(
                            children: <Widget>[
                              Text(
                                'Capítulo ${item.chapter + 1}',
                                style: TextStyle(
                                  color: theme.dim,
                                  fontSize: 12,
                                ),
                              ),
                              const Spacer(),
                              IconButton(
                                tooltip: 'Editar nota',
                                onPressed: () => _editNote(context, item),
                                icon: const Icon(Icons.edit_outlined),
                              ),
                              IconButton(
                                tooltip: 'Quitar',
                                onPressed: () => store.removeAnnotation(item),
                                icon: Icon(
                                  Icons.delete_outline,
                                  color: theme.error,
                                ),
                              ),
                            ],
                          ),
                        ],
                      ),
                    );
                  },
                ),
        );
      },
    );
  }

  Future<void> _editNote(BuildContext context, AnnotationItem item) async {
    final controller = TextEditingController(text: item.note);
    await showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: const Text('Nota'),
        content: TextField(
          controller: controller,
          autofocus: true,
          maxLines: 5,
          decoration: const InputDecoration(hintText: 'Escribí una nota'),
        ),
        actions: <Widget>[
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('Cancelar'),
          ),
          FilledButton(
            onPressed: () {
              store.updateAnnotationNote(item, controller.text.trim());
              Navigator.pop(context);
            },
            child: const Text('Guardar'),
          ),
        ],
      ),
    );
    controller.dispose();
  }
}
