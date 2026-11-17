import 'package:material_ui/material_ui.dart';

import 'models.dart';

class CandilTheme {
  const CandilTheme({
    required this.name,
    required this.bg,
    required this.fg,
    required this.dim,
    required this.accent,
    required this.panel,
    required this.error,
    required this.selection,
    required this.border,
  });

  final String name;
  final Color bg;
  final Color fg;
  final Color dim;
  final Color accent;
  final Color panel;
  final Color error;
  final Color selection;
  final Color border;

  bool get isDark {
    final luminance = 0.299 * bg.r + 0.587 * bg.g + 0.114 * bg.b;
    return luminance < 0.5;
  }

  ThemeData materialTheme() {
    final scheme =
        ColorScheme.fromSeed(
          seedColor: accent,
          brightness: isDark ? Brightness.dark : Brightness.light,
        ).copyWith(
          primary: accent,
          onPrimary: isDark ? bg : Colors.white,
          secondary: accent,
          surface: bg,
          onSurface: fg,
          error: error,
          outline: border,
        );
    return ThemeData(
      useMaterial3: true,
      colorScheme: scheme,
      brightness: scheme.brightness,
      scaffoldBackgroundColor: bg,
      canvasColor: panel,
      fontFamily: 'Literata',
      appBarTheme: AppBarTheme(
        backgroundColor: bg,
        foregroundColor: fg,
        surfaceTintColor: Colors.transparent,
        elevation: 0,
        centerTitle: false,
      ),
      dividerColor: border,
      inputDecorationTheme: InputDecorationTheme(
        filled: true,
        fillColor: panel,
        hintStyle: TextStyle(color: dim),
        border: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
          borderSide: BorderSide(color: border),
        ),
        enabledBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
          borderSide: BorderSide(color: border),
        ),
        focusedBorder: OutlineInputBorder(
          borderRadius: BorderRadius.circular(12),
          borderSide: BorderSide(color: accent, width: 1.5),
        ),
      ),
      sliderTheme: SliderThemeData(
        activeTrackColor: accent,
        thumbColor: accent,
        inactiveTrackColor: border,
      ),
      snackBarTheme: SnackBarThemeData(
        backgroundColor: panel,
        contentTextStyle: TextStyle(color: fg),
        actionTextColor: accent,
      ),
    );
  }
}

const candilThemes = <CandilTheme>[
  CandilTheme(
    name: 'Papel',
    bg: Color(0xFFF4ECD8),
    fg: Color(0xFF3B3226),
    dim: Color(0xFF6E624A),
    accent: Color(0xFF8A582A),
    panel: Color(0xFFEADFC6),
    error: Color(0xFF9C4B43),
    selection: Color(0xFFCFB89B),
    border: Color(0xFFC5BCA6),
  ),
  CandilTheme(
    name: 'Noche',
    bg: Color(0xFF14161A),
    fg: Color(0xFFC9C7C1),
    dim: Color(0xFF84878E),
    accent: Color(0xFF7FA7C8),
    panel: Color(0xFF1C1F25),
    error: Color(0xFFC56E65),
    selection: Color(0xFF394957),
    border: Color(0xFF3B3E43),
  ),
  CandilTheme(
    name: 'Claro',
    bg: Color(0xFFFBFAF7),
    fg: Color(0xFF24221F),
    dim: Color(0xFF6F6C66),
    accent: Color(0xFF3F6E9A),
    panel: Color(0xFFF0EEE8),
    error: Color(0xFFA95249),
    selection: Color(0xFFB9C9D6),
    border: Color(0xFFCAC8C4),
  ),
  CandilTheme(
    name: 'Ámbar',
    bg: Color(0xFF1E1710),
    fg: Color(0xFFDACBA9),
    dim: Color(0xFF93856B),
    accent: Color(0xFFD9A441),
    panel: Color(0xFF271E15),
    error: Color(0xFFC56E65),
    selection: Color(0xFF5F4821),
    border: Color(0xFF473E30),
  ),
  CandilTheme(
    name: 'Contraste',
    bg: Color(0xFF000000),
    fg: Color(0xFFFFFFFF),
    dim: Color(0xFF9A9A9A),
    accent: Color(0xFFE8C547),
    panel: Color(0xFF101010),
    error: Color(0xFFBF5F55),
    selection: Color(0xFF514519),
    border: Color(0xFF363636),
  ),
];

CandilTheme themeFor(int index) =>
    candilThemes[index.clamp(0, candilThemes.length - 1)];

TextStyle readingStyle(Settings settings, CandilTheme theme) => TextStyle(
  fontFamily: 'Literata',
  fontSize: settings.fontSize,
  height: settings.lineHeight,
  color: theme.fg,
);
