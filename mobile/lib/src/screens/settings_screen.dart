import 'package:material_ui/material_ui.dart';

import '../app_store.dart';
import '../models.dart';
import '../theme.dart';

class SettingsScreen extends StatefulWidget {
  const SettingsScreen({super.key, required this.store});

  final AppStore store;

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  late Settings _settings = widget.store.settings;

  void _update(Settings next, {bool save = true}) {
    setState(() => _settings = next);
    if (save) widget.store.saveSettings(next);
  }

  @override
  Widget build(BuildContext context) {
    final theme = themeFor(_settings.theme);
    final store = widget.store;
    return Scaffold(
      appBar: AppBar(title: const Text('Ajustes')),
      body: ListView(
        padding: const EdgeInsets.fromLTRB(16, 8, 16, 32),
        children: <Widget>[
          _section('Tema', theme),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: <Widget>[
              for (var index = 0; index < candilThemes.length; index++)
                ChoiceChip(
                  label: Text(candilThemes[index].name),
                  selected: _settings.theme == index,
                  onSelected: (_) => _update(_settings.copyWith(theme: index)),
                ),
            ],
          ),
          const SizedBox(height: 24),
          _section('Tipografía', theme),
          _slider(
            theme,
            label: 'Tamaño ${_settings.fontSize.round()}',
            value: _settings.fontSize,
            min: 14,
            max: 32,
            divisions: 18,
            onChanged: (value) =>
                _update(_settings.copyWith(fontSize: value), save: false),
            onChangeEnd: (value) =>
                _update(_settings.copyWith(fontSize: value)),
          ),
          _slider(
            theme,
            label: 'Interlínea ${_settings.lineHeight.toStringAsFixed(2)}',
            value: _settings.lineHeight,
            min: 1.2,
            max: 2.2,
            divisions: 10,
            onChanged: (value) =>
                _update(_settings.copyWith(lineHeight: value), save: false),
            onChangeEnd: (value) =>
                _update(_settings.copyWith(lineHeight: value)),
          ),
          _slider(
            theme,
            label:
                'Espacio entre párrafos '
                '${_settings.paragraphSpace.toStringAsFixed(2)}',
            value: _settings.paragraphSpace,
            min: 0,
            max: 1.6,
            divisions: 8,
            onChanged: (value) =>
                _update(_settings.copyWith(paragraphSpace: value), save: false),
            onChangeEnd: (value) =>
                _update(_settings.copyWith(paragraphSpace: value)),
          ),
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: const Text('Modo enfoque'),
            subtitle: const Text(
              'Atenúa el texto lejos de la línea central. Al empezar o '
              'terminar un capítulo, esas líneas quedan legibles.',
            ),
            value: _settings.focusMode,
            onChanged: (value) => _update(_settings.copyWith(focusMode: value)),
          ),
          const SizedBox(height: 24),
          _section('Datos', theme),
          Text(
            'Los datos de Candil Mobile viven en:\n${store.data.rootPath}',
            style: TextStyle(color: theme.dim, height: 1.5),
          ),
          const SizedBox(height: 10),
          Text(
            'El formato de config.json, library.json y annotations.json es '
            'compatible con el respaldo de Candil de escritorio. Las carpetas '
            'vigiladas no se escanean en el móvil: los EPUB se importan de a uno.',
            style: TextStyle(color: theme.dim, height: 1.5),
          ),
          const SizedBox(height: 12),
          Text(
            'La caché de EPUB remotos, la actualización automática y el envío a '
            'dispositivo son funciones de escritorio y no se incluyen en esta '
            'primera base móvil.',
            style: TextStyle(color: theme.dim, height: 1.5),
          ),
          const SizedBox(height: 24),
          _section('Acerca de', theme),
          Text(
            'Candil Mobile 0.1.0 · Flutter + núcleo Rust compartido',
            style: TextStyle(color: theme.dim),
          ),
        ],
      ),
    );
  }

  Widget _section(String title, CandilTheme theme) => Padding(
    padding: const EdgeInsets.only(bottom: 12),
    child: Text(
      title,
      style: TextStyle(
        color: theme.fg,
        fontSize: 18,
        fontWeight: FontWeight.w600,
      ),
    ),
  );

  Widget _slider(
    CandilTheme theme, {
    required String label,
    required double value,
    required double min,
    required double max,
    required int divisions,
    required ValueChanged<double> onChanged,
    required ValueChanged<double> onChangeEnd,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: <Widget>[
        Text(label, style: TextStyle(color: theme.dim)),
        Slider(
          value: value.clamp(min, max),
          min: min,
          max: max,
          divisions: divisions,
          onChanged: onChanged,
          onChangeEnd: onChangeEnd,
        ),
      ],
    );
  }
}
