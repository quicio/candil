## Context

See `proposal.md` for motivation. The current implementation concentrates UI in `src/app.rs` (≈5.000 líneas), colors in `src/theme.rs` and reading defaults in `src/config.rs`. egui 0.32 is pinned by ADR-0003; there is no CSS, no webview and no design-token layer. The five themes already have a coherent editorial palette, but the code mixes literal colors (`Color32::from_rgb(190, 92, 82)`), arbitrary spacing (2/4/6/8/12/14/18/22/28/34/90/100/140) and panel widths (340/410/420/440/460/800). Default egui visuals leak into sliders, checkboxes and window chrome.

The reading surface is the strongest existing asset: Literata, 19 px default, 1.55 line height, 660 px measure and a 38 px top bar. The design must protect it while giving the rest of the product the same level of care.

## Goals / Non-Goals

**Goals:**
- Make tokens and components explicit enough that a new panel or state is assembled, not improvised.
- Fix contrast, focus, overlap and color-swatch defects without changing feature behavior.
- Keep the reading column, shortcuts, persistence and parser untouched in behavior.
- Make every change verifiable through tests, contrast checks and `--shot` captures.

**Non-Goals:**
- No new product features, no data-model changes, no parser changes.
- No new fonts, icon packs, animation libraries or UI dependencies.
- No command palette, no side-sheet redesign and no mobile layout in this change.
- No screenshot-diff automation; captures remain a manual visual gate.

## Decisions

### 1. Token layer in `theme.rs` plus a small `ui.rs` helper module

Extend `Theme` with semantic roles (`bg`, `surface`, `surface_raised`, `fg`, `muted`, `accent`, `selection`, `error`, `border`) and add a `UiTokens` module with spacing, radius and UI type sizes. Add `src/ui.rs` for shared widgets: `button`, `icon_button`, `input`, `panel_frame`, `panel_header`, `chip`, `section_header`, `color_swatch`, `result_row`.

**Why:** `app.rs` is already too large for per-widget duplication. A helper module is additive, testable and keeps `app.rs` focused on composition. **Alternative considered:** restyle every call site inline; rejected because it repeats the same fixes and makes consistency regressions likely. **Alternative considered:** a full component framework; rejected as over-engineering.

### 2. Accessible palette adjustments that preserve identity

Keep the five themes and Literata. Adjust only the roles that fail AA:

| theme | muted target | error target | accent target |
|---|---|---|---|
| Papel | `#6E624A` (5.08 / 4.52) | `#9C4B43` (5.09 / 4.53) | `#8A582A` (5.08 / 4.52) |
| Noche | `#84878E` (5.03 / 4.59) | `#C56E65` (5.00 / 4.55) | `#7FA7C8` (unchanged) |
| Claro | `#6F6C66` (5.01 / 4.51) | `#A95249` (5.04 / 4.54) | `#3F6E9A` (unchanged) |
| Ámbar | `#93856B` (4.90 / 4.53) | `#C56E65` (4.89 / 4.52) | `#D9A441` (unchanged) |
| Contraste | `#9A9A9A` (unchanged) | `#BF5F55` (4.98 / 4.52) | `#E8C547` (unchanged) |

Ratios are against `bg` / `surface`. Selection fill moves from accent at 45% to 35% so dark themes keep text at ≥4.5:1. Highlight fills stay at 35% alpha; their text contrast is already ≥5.3:1 in every theme.

**Why:** the current muted text fails AA in Papel (3.77), Noche (3.65), Claro (3.43) and Ámbar (4.24); error text fails in all non-Contraste themes. **Alternative considered:** replace the palettes with neutral grays; rejected because it erases Candil's warmth. **Alternative considered:** raise font sizes instead of contrast; rejected because small metadata still needs AA.

### 3. Typography roles without adding a UI font

Keep Literata for reading and UI. Define:

- Reading body: user-controlled 19 px default, 1.55 line height, 660 px measure.
- Reading headings: 1.75× / 1.45× / 1.25× body, semibold.
- UI display: 24 px (empty state), 16 px (section title), 13.5 px (body/buttons), 12 px (labels), 11 px (non-essential metadata only).
- Captions and footnotes: 0.85× body.

**Why:** a second typeface would add weight and weaken the editorial identity; Literata has enough weights. **Alternative considered:** system sans for UI; rejected because it makes Candil look like a dashboard.

### 4. Library header becomes two-tier and additive

Row 1: Candil + count, search (flex), primary "Añadir carpeta", overflow (`Refrescar`, `Configuración`, `Carpetas`). Row 2 (only when folders exist): folder chips and compact order/filter controls. At <900 px, the search keeps priority and secondary actions collapse. The empty state keeps one primary action and one secondary text action.

**Why:** the current single row has seven controls plus a 200 px search field; at 980 px the search collides with "Orden" (confirmed in `library-error.png`). **Alternative considered:** hide search behind an icon on narrow widths; rejected because search is the main library action. **Alternative considered:** a permanent left sidebar; rejected because it adds chrome and reduces the book grid.

### 5. Reader top bar keeps text labels but reduces competition

Left: index, typography, notes, search. Center-left: title + author (truncated). Right: chapter navigation + percentage. Settings moves to the far right as a gear with the same tooltip/shortcut, or into the overflow if the window is narrow. The bar remains 38–40 px and uses the panel surface.

**Why:** five text buttons plus title, author and three navigation elements overflow at 980 px. **Alternative considered:** icon-only navigation; rejected for first-run discoverability. **Alternative considered:** auto-hiding bar; rejected because it hides system status and complicates keyboard focus.

### 6. Panels share one frame and three width roles

- `panel-sm` 340 px: typography, go-to.
- `panel-md` 420 px: annotations, search, note, consultation.
- `panel-lg` 760 px: configuration.

Every panel uses a 44 px header with title left and close right, 16 px body padding, 12 px section gaps and no collapse arrow when not collapsible. Default placement is top-right with a 16 px margin; egui may remember a user-dragged position. Panels do not resize the reading column.

**Why:** six widths and three paddings make the product feel assembled from parts. **Alternative considered:** one width for all panels; rejected because configuration needs more room and compact panels would feel empty.

### 7. Component states use explicit visuals, not framework defaults

Define button variants (primary, secondary, ghost, destructive), input, chip, card, tab, result row and swatch. Set `Visuals` for `widgets.inactive/hovered/active/open` and `selection`; add a 2 px accent focus ring. Replace the `●` text glyph with a painted filled circle plus ring for selected state.

**Why:** default egui blue sliders/checkboxes and the missing `●` glyph are visible defects. **Alternative considered:** hide the color control behind a dropdown; rejected because the three colors are a frequent action and should stay one click away.

### 8. Accessibility is enforced by a unit test

Add a `theme` unit test that computes WCAG contrast for every `(fg, bg)`, `(muted, bg/surface)`, `(error, bg/surface)` and `(fg, selection)` pair and asserts ≥4.5:1. Keep manual checks for focus, target size and non-color signals.

**Why:** contrast regressions are cheap to prevent and expensive to notice. **Alternative considered:** a one-off audit script; rejected because it would not run in CI.

### 9. No motion system, no new dependencies

Use egui's existing hover/active interpolation only for color; no layout animation, no parallax, no decorative transitions. If the OS reduced-motion setting is unavailable to egui, document the limitation and keep motion effectively static.

**Why:** the product is a reading tool; motion is noise. **Alternative considered:** add `egui_animation` or similar; rejected as unnecessary weight.

### 10. Phased delivery with a hard visual gate

Implement in five phases: (1) tokens and helpers, (2) library, (3) reader bar and selection, (4) panels and settings, (5) accessibility/captures/docs. Each phase ends with `cargo test`, `cargo clippy`, a contrast check and `--shot` captures at 980×780 and 720×600. A phase that degrades the reading view is reverted before the next one starts.

**Why:** `app.rs` is large and the reading experience is the product. **Alternative considered:** one large visual rewrite; rejected because regressions would be hard to isolate.

## Risks / Trade-offs

- **`app.rs` regression risk** → change one surface per task, keep a capture before/after and run the existing reader tests after each phase.
- **egui 0.32 widget limits** → prefer `Frame` + `Painter` for custom visuals and keep native controls where they already work; do not fork egui.
- **Contrast changes flatten the palette** → adjust only muted/error/accent roles and verify the five themes visually; keep the paper warmth.
- **Top bar simplification hides actions** → keep text labels on wide windows, tooltips and shortcuts on narrow ones, and verify keyboard discoverability.
- **Screenshot drift** → regenerate `docs/img/` only after the visual gate; keep public-domain books in the sandbox.
- **Scope creep into new features** → any new behavior requires a new OpenSpec change, not a task in this one.

## Migration Plan

No data migration. The change is visual and interactional; `config.json`, `library.json`, `annotations.json` and `consultas.json` keep their schemas. Rollback is a normal revert of the UI commits; theme tokens are additive and do not affect persisted data. If a phase fails the visual gate, revert that phase before continuing.

## Open Questions

None that would change the specs, approach or task breakdown. Command palette, side sheets and mobile layouts are explicitly deferred to future changes.
