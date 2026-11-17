import 'dart:io';

import 'package:material_ui/material_ui.dart';

import '../models.dart';
import '../theme.dart';

class BookCard extends StatelessWidget {
  const BookCard({
    super.key,
    required this.book,
    required this.theme,
    required this.coverFile,
    required this.onTap,
    required this.onMenu,
  });

  final BookEntry book;
  final CandilTheme theme;
  final File? coverFile;
  final VoidCallback onTap;
  final VoidCallback onMenu;

  @override
  Widget build(BuildContext context) {
    final progress = book.progress.ratio.clamp(0.0, 1.0);
    return Material(
      color: theme.panel,
      borderRadius: BorderRadius.circular(14),
      child: InkWell(
        borderRadius: BorderRadius.circular(14),
        onTap: onTap,
        onLongPress: onMenu,
        child: Padding(
          padding: const EdgeInsets.all(10),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: <Widget>[
              Expanded(
                child: Stack(
                  fit: StackFit.expand,
                  children: <Widget>[
                    ClipRRect(
                      borderRadius: BorderRadius.circular(8),
                      child: coverFile != null && coverFile!.existsSync()
                          ? Image.file(coverFile!, fit: BoxFit.cover)
                          : _placeholder(),
                    ),
                    if (book.missing)
                      Positioned(
                        left: 6,
                        top: 6,
                        child: _badge('Falta', theme.error),
                      ),
                    Positioned(
                      right: 0,
                      top: 0,
                      child: Material(
                        color: theme.bg.withValues(alpha: 0.82),
                        shape: const CircleBorder(),
                        child: InkWell(
                          customBorder: const CircleBorder(),
                          onTap: onMenu,
                          child: Padding(
                            padding: const EdgeInsets.all(4),
                            child: Icon(
                              Icons.more_horiz,
                              size: 18,
                              color: theme.fg,
                            ),
                          ),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 10),
              Text(
                book.title,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(
                  color: theme.fg,
                  fontWeight: FontWeight.w600,
                  height: 1.2,
                ),
              ),
              const SizedBox(height: 3),
              Text(
                book.author.isEmpty ? 'Autor desconocido' : book.author,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: TextStyle(color: theme.dim, fontSize: 12.5),
              ),
              const SizedBox(height: 8),
              ClipRRect(
                borderRadius: BorderRadius.circular(8),
                child: LinearProgressIndicator(
                  value: progress,
                  minHeight: 3,
                  backgroundColor: theme.border,
                  color: theme.accent,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _placeholder() {
    final letter = book.title.trim().isEmpty
        ? '?'
        : book.title.trim()[0].toUpperCase();
    return ColoredBox(
      color: theme.bg,
      child: Center(
        child: Text(
          letter,
          style: TextStyle(
            fontFamily: 'Literata',
            fontSize: 54,
            color: theme.dim.withValues(alpha: 0.45),
          ),
        ),
      ),
    );
  }

  Widget _badge(String text, Color color) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 7, vertical: 3),
    decoration: BoxDecoration(
      color: theme.bg.withValues(alpha: 0.9),
      borderRadius: BorderRadius.circular(999),
      border: Border.all(color: color),
    ),
    child: Text(text, style: TextStyle(color: color, fontSize: 11)),
  );
}
