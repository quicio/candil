import 'dart:io';

import 'package:candil_mobile/src/app_store.dart';
import 'package:candil_mobile/src/data_store.dart';
import 'package:candil_mobile/src/models.dart';
import 'package:candil_mobile/src/screens/library_screen.dart';
import 'package:candil_mobile/src/theme.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';

void main() {
  testWidgets('la biblioteca vacía muestra la acción de importar', (
    tester,
  ) async {
    final store = AppStore(
      data: DataStore(directory: Directory('/tmp/candil-ui-test')),
    );
    store
      ..loading = false
      ..library = BookLibrary()
      ..annotations = AnnotationStore();
    await tester.pumpWidget(
      MaterialApp(
        theme: themeFor(0).materialTheme(),
        home: LibraryScreen(store: store),
      ),
    );
    await tester.pump();
    expect(find.text('Tu biblioteca está vacía'), findsOneWidget);
    expect(
      find.text(
        'Importa uno o varios EPUB para empezar. También puedes restaurar un '
        'respaldo de Candil desde el menú.',
      ),
      findsOneWidget,
    );
  });
}
