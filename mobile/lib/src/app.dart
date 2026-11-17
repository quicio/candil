import 'package:material_ui/material_ui.dart';

import 'app_store.dart';
import 'screens/library_screen.dart';
import 'theme.dart';

class CandilApp extends StatefulWidget {
  const CandilApp({super.key, this.store});

  /// Permite inyectar un almacén con un directorio temporal en tests.
  final AppStore? store;

  @override
  State<CandilApp> createState() => _CandilAppState();
}

class _CandilAppState extends State<CandilApp> {
  late final AppStore _store;
  late final bool _ownsStore;
  final GlobalKey<ScaffoldMessengerState> _messenger =
      GlobalKey<ScaffoldMessengerState>();

  @override
  void initState() {
    super.initState();
    _ownsStore = widget.store == null;
    _store = widget.store ?? AppStore();
    _store.addListener(_onStoreChanged);
    _store.init();
  }

  @override
  void dispose() {
    _store.removeListener(_onStoreChanged);
    if (_ownsStore) _store.dispose();
    super.dispose();
  }

  void _onStoreChanged() {
    final notice = _store.notice;
    if (notice == null) return;
    _store.clearNotice();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _messenger.currentState?.showSnackBar(SnackBar(content: Text(notice)));
    });
  }

  @override
  Widget build(BuildContext context) {
    return ListenableBuilder(
      listenable: _store,
      builder: (context, _) {
        final theme = themeFor(_store.settings.theme);
        return MaterialApp(
          title: 'Candil',
          debugShowCheckedModeBanner: false,
          scaffoldMessengerKey: _messenger,
          theme: theme.materialTheme(),
          home: _home(theme),
        );
      },
    );
  }

  Widget _home(CandilTheme theme) {
    if (_store.loading) {
      return Scaffold(
        body: Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: <Widget>[
              Text(
                'Candil',
                style: TextStyle(
                  fontFamily: 'Literata',
                  fontSize: 34,
                  fontWeight: FontWeight.w600,
                  color: theme.fg,
                ),
              ),
              const SizedBox(height: 20),
              CircularProgressIndicator(color: theme.accent),
            ],
          ),
        ),
      );
    }
    if (_store.fatalError != null) {
      return Scaffold(
        body: Center(
          child: Padding(
            padding: const EdgeInsets.all(28),
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: <Widget>[
                Icon(Icons.error_outline, size: 48, color: theme.error),
                const SizedBox(height: 16),
                Text(
                  'No pude iniciar Candil',
                  style: TextStyle(
                    color: theme.fg,
                    fontSize: 22,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  _store.fatalError!,
                  textAlign: TextAlign.center,
                  style: TextStyle(color: theme.dim, height: 1.45),
                ),
                const SizedBox(height: 12),
                TextButton(
                  onPressed: _store.init,
                  child: const Text('Reintentar'),
                ),
              ],
            ),
          ),
        ),
      );
    }
    return LibraryScreen(store: _store);
  }
}
