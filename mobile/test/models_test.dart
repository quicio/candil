import 'package:candil_mobile/src/models.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  test('settings conserva el formato de escritorio', () {
    final settings = Settings.fromJson(<String, dynamic>{
      'theme': 2,
      'font_size': 23.0,
      'line_height': 1.8,
      'paragraph_space': 0.4,
      'focus_mode': true,
    });
    expect(settings.theme, 2);
    expect(settings.fontSize, 23);
    expect(settings.focusMode, isTrue);
    final json = settings.toJson();
    expect(json['font_size'], 23);
    expect(json['focus_mode'], true);
  });

  test('la biblioteca busca sin acentos y por archivo', () {
    final library = BookLibrary.fromJson(<String, dynamic>{
      'books': <Map<String, dynamic>>[
        <String, dynamic>{
          'path': '/libros/Arbol.epub',
          'title': 'El árbol de la ciencia',
          'author': 'Pío Baroja',
        },
        <String, dynamic>{
          'path': '/libros/Niebla.epub',
          'title': 'Niebla',
          'author': 'Miguel de Unamuno',
        },
      ],
    });
    expect(library.search('arbol').length, 1);
    expect(library.search('baroja').single.title, 'El árbol de la ciencia');
    expect(library.search('niebla.epub').single.author, 'Miguel de Unamuno');
  });

  test('las anotaciones conservan campos desconocidos y reasignan ruta', () {
    final store = AnnotationStore.fromJson(<String, dynamic>{
      'schema_version': 3,
      'items': <Map<String, dynamic>>[
        <String, dynamic>{
          'id': 4,
          'book': '/viejo/Libro.epub',
          'chapter': 1,
          'block': 2,
          'start': 0,
          'end': 5,
          'parts': <String>['Hola'],
          'quote': 'Hola',
          'custom': 'se conserva',
        },
      ],
      'custom_top': 7,
    });
    expect(store.nextId(), 5);
    store.rekey('/viejo/Libro.epub', '/nuevo/Libro.epub');
    final json = store.toJson();
    expect(json['custom_top'], 7);
    final item = (json['items'] as List).single as Map<String, dynamic>;
    expect(item['book'], '/nuevo/Libro.epub');
    expect(item['custom'], 'se conserva');
  });

  test('el capítulo decodifica bloques y runs', () {
    final chapter = ReaderChapter.fromJson(<String, dynamic>{
      'index': 3,
      'title': 'Tres',
      'blocks': <Map<String, dynamic>>[
        <String, dynamic>{
          'kind': 'heading',
          'level': 2,
          'text': 'Título',
          'runs': <Map<String, dynamic>>[
            <String, dynamic>{'text': 'Título', 'bold': true},
          ],
        },
      ],
    });
    expect(chapter.index, 3);
    expect(chapter.blocks.single.kind, 'heading');
    expect(chapter.blocks.single.runs.single.bold, isTrue);
  });
}
