import 'package:material_ui/material_ui.dart';

import '../theme.dart';

class FocusOverlay extends StatelessWidget {
  const FocusOverlay({
    super.key,
    required this.controller,
    required this.theme,
  });

  final ScrollController controller;
  final CandilTheme theme;

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: controller,
      builder: (context, _) {
        if (!controller.hasClients) return const SizedBox.shrink();
        final position = controller.position;
        final top = (position.pixels / 180).clamp(0.0, 1.0);
        final bottom = ((position.maxScrollExtent - position.pixels) / 180)
            .clamp(0.0, 1.0);
        if (top == 0 && bottom == 0) return const SizedBox.shrink();
        final clear = theme.bg.withValues(alpha: 0);
        final strong = theme.bg.withValues(alpha: 0.92);
        final medium = theme.bg.withValues(alpha: 0.92 * 0.45);
        return IgnorePointer(
          child: Column(
            children: <Widget>[
              Expanded(
                child: DecoratedBox(
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      begin: Alignment.topCenter,
                      end: Alignment.center,
                      colors: <Color>[
                        strong.withValues(alpha: strong.a * top),
                        medium.withValues(alpha: medium.a * top),
                        clear,
                      ],
                      stops: const <double>[0, 0.55, 1],
                    ),
                  ),
                ),
              ),
              Expanded(
                child: DecoratedBox(
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      begin: Alignment.bottomCenter,
                      end: Alignment.center,
                      colors: <Color>[
                        strong.withValues(alpha: strong.a * bottom),
                        medium.withValues(alpha: medium.a * bottom),
                        clear,
                      ],
                      stops: const <double>[0, 0.55, 1],
                    ),
                  ),
                ),
              ),
            ],
          ),
        );
      },
    );
  }
}
