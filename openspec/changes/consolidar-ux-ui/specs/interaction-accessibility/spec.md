## Purpose

Define the interaction and accessibility contract that makes every Candil surface predictable with keyboard, mouse or trackpad, including users who need higher contrast or visible focus.

## ADDED Requirements

### Requirement: Keyboard operability

Every primary action SHALL be reachable and operable with the keyboard, and existing shortcuts SHALL keep working.

#### Scenario: Existing shortcut
- **WHEN** the user presses Space, arrows, T, S, D, G, K, H, N, Command+F, Command+O or Command+,
- **THEN** the action behaves as documented in README and in-app help

#### Scenario: Panel dismissal
- **WHEN** a panel is open and the user presses Escape
- **THEN** the panel closes without losing the reading position or saved data

#### Scenario: Focus traversal
- **WHEN** the user presses Tab through a panel or toolbar
- **THEN** focus moves in a logical order and every focused control is visible and actionable

### Requirement: Visible focus

Keyboard focus SHALL be visually obvious on every interactive element and SHALL NOT rely on the browser or framework default alone.

#### Scenario: Focused button
- **WHEN** a button receives focus
- **THEN** it shows a 2 px accent ring or equivalent high-contrast treatment

#### Scenario: Focused input
- **WHEN** a text field receives focus
- **THEN** its border or ring changes in a way that is visible in every theme

### Requirement: Contrast and non-color signals

Status, errors, selection and highlight colors SHALL meet contrast requirements and SHALL be accompanied by text, icon or shape when color alone would carry meaning.

#### Scenario: Error state
- **WHEN** an error is shown
- **THEN** it uses the accessible error color plus a text message or icon

#### Scenario: Highlight color
- **WHEN** a highlight color is shown in a list
- **THEN** the color is accompanied by a label or tooltip such as "acento", "ámbar" or "verde"

#### Scenario: Selected tab
- **WHEN** a tab or segmented option is selected
- **THEN** selection is visible through more than color, such as weight, underline or a border

### Requirement: Target size and spacing

Interactive controls SHALL have a target of at least 32×32 logical pixels and sufficient separation to avoid accidental activation.

#### Scenario: Toolbar buttons
- **WHEN** the library or reader toolbar is rendered
- **THEN** each button meets the minimum target and adjacent buttons do not overlap

#### Scenario: Small icon button
- **WHEN** an icon-only control is used
- **THEN** it has a tooltip and a target that meets the minimum size

### Requirement: Feedback and system status

Actions that take time or change state SHALL provide visible feedback.

#### Scenario: Scan or refresh
- **WHEN** the user starts a scan or refresh
- **THEN** the interface shows a busy state and, when finished, a summary of found, new, updated and failed items

#### Scenario: Save note
- **WHEN** the user saves or edits a note
- **THEN** the change is reflected immediately and persisted without a silent failure

#### Scenario: Offline request
- **WHEN** the user attempts an online query while online access is disabled
- **THEN** the interface explains why nothing was sent and how to enable it

### Requirement: Error prevention and recovery

Destructive or irreversible actions SHALL require confirmation or provide undo.

#### Scenario: Delete highlight
- **WHEN** the user deletes a highlight
- **THEN** Candil asks for confirmation or offers an immediate undo

#### Scenario: Remove library entry
- **WHEN** the user removes a book from the library
- **THEN** Candil explains that the EPUB file is not deleted and asks for confirmation

#### Scenario: Clear history
- **WHEN** the user clears consultation history
- **THEN** Candil asks for confirmation before deleting the entries

### Requirement: In-app help

The app SHALL make keyboard shortcuts and core reading actions discoverable without leaving the current context.

#### Scenario: Shortcut help
- **WHEN** the user opens help or presses the help shortcut
- **THEN** a concise list of reading, navigation, annotation and panel shortcuts is shown

#### Scenario: Tooltip
- **WHEN** the user hovers an icon or an unfamiliar action
- **THEN** a tooltip names the action and, when applicable, its keyboard shortcut

### Requirement: Motion restraint

Motion SHALL be limited to short state transitions that clarify feedback; continuous or decorative animation SHALL NOT be used.

#### Scenario: Hover transition
- **WHEN** a button or row changes hover state
- **THEN** the change is brief and does not move surrounding layout

#### Scenario: Reduced motion
- **WHEN** the user has reduced motion enabled at the operating-system level
- **THEN** non-essential transitions are removed or shortened
