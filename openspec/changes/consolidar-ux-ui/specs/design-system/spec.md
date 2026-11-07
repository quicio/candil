## Purpose

Define the visual and interaction tokens that keep Candil coherent across library, reader, panels and settings, so new UI reads as one editorial product instead of a collection of default widgets.

## ADDED Requirements

### Requirement: Centralized visual tokens

The system SHALL expose a single set of semantic tokens for background, surface, primary text, secondary text, accent, selection, error and border, and SHALL apply them to every themed surface.

#### Scenario: Theme switch
- **WHEN** the user changes between Papel, Noche, Claro, Ámbar and Contraste
- **THEN** library, reader, panels, inputs and controls use that theme's tokens without leaving default framework colors visible

#### Scenario: New surface
- **WHEN** a new panel or control is added
- **THEN** it uses existing tokens and component patterns instead of introducing new literal colors or sizes

### Requirement: Accessible text contrast

All UI text SHALL meet a 4.5:1 contrast ratio against its immediate background; large text and non-text indicators SHALL meet at least 3:1.

#### Scenario: Secondary labels
- **WHEN** a label, metadata line, hint or timestamp is rendered in any theme
- **THEN** its contrast against the surface is at least 4.5:1

#### Scenario: Error message
- **WHEN** an error, missing-file warning or failed-query message is shown
- **THEN** its contrast against the surface is at least 4.5:1 and it is not communicated by color alone

#### Scenario: Text selection
- **WHEN** the user selects text in any theme
- **THEN** the selected text keeps at least 4.5:1 contrast against the selection background

### Requirement: Typography hierarchy

The system SHALL provide distinct roles for reading text, headings, UI labels, metadata and display text, with reading typography remaining user-adjustable.

#### Scenario: Default reading view
- **WHEN** a book opens with default settings
- **THEN** body text uses Literata at 19 px, line height 1.55 and a measure of approximately 60–75 characters per line

#### Scenario: UI hierarchy
- **WHEN** a panel shows a title, section heading, label and metadata
- **THEN** each role is visually distinct through size and weight, not through arbitrary one-off values

### Requirement: Spacing and shape scale

UI spacing and corner radii SHALL follow a small shared scale; reading rhythm MAY scale from the user's font size.

#### Scenario: Panel padding
- **WHEN** two panels of different widths are open
- **THEN** their inner padding, header height and control spacing follow the same scale

#### Scenario: Controls
- **WHEN** buttons, inputs, chips and cards are rendered
- **THEN** their corner radii come from the shared radius scale and remain visually related

### Requirement: Flat editorial surfaces

Depth SHALL be conveyed primarily by tonal surface changes and hairline borders; decorative shadows, gradients and glass effects SHALL NOT be used.

#### Scenario: Floating panel
- **WHEN** a panel opens over the reader
- **THEN** it separates from the page through surface tone and a border, without a heavy drop shadow

#### Scenario: Toolbar
- **WHEN** the library or reader toolbar is visible
- **THEN** it uses the panel surface and a single separation treatment, not stacked borders or shadows

### Requirement: Component state conventions

Buttons, inputs, chips, cards, tabs and rows SHALL define consistent default, hover, focus, active, selected and disabled states.

#### Scenario: Keyboard focus
- **WHEN** a control receives keyboard focus
- **THEN** it shows a visible focus indicator of at least 2 px in the theme accent color

#### Scenario: Disabled control
- **WHEN** a control is unavailable
- **THEN** it is visually muted, does not show hover feedback and is not focusable

#### Scenario: Selected color
- **WHEN** a highlight color is selected
- **THEN** the selected state is visible through a ring, check or label in addition to the color itself

### Requirement: Identity preservation

The system SHALL preserve Candil's editorial identity: warm paper and ink tones, Literata for reading, restrained amber accent and a quiet, text-first hierarchy.

#### Scenario: Theme redesign
- **WHEN** a palette or typography value is changed
- **THEN** the result still reads as Candil and does not adopt generic dashboard, SaaS or AI-generated aesthetics
