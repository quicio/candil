## ADDED Requirements

### Requirement: Continue reading shortcut

The library SHALL offer a direct shortcut to the most recently opened valid book without requiring the user to find its card.

#### Scenario: Recent book available

- **WHEN** the library has at least one non-missing book with a previous opening
- **THEN** the header shows a "Continuar leyendo" action that opens the book with the greatest last-opened time

#### Scenario: No recent book

- **WHEN** no book has been opened before
- **THEN** the shortcut is not shown

#### Scenario: Recent book missing or broken

- **WHEN** the most recent book is missing or has a parse error
- **THEN** the shortcut uses the next most recent valid book, or hides if none exists

#### Scenario: Narrow window

- **WHEN** the library is shown at 720×600 or the header runs out of room
- **THEN** the action uses the short label "Continuar" or hides without overlapping search, add or overflow controls

### Requirement: Refresh on window focus

The library SHALL refresh incrementally when the app regains focus, without blocking the UI and without scanning more often than once every 30 seconds.

#### Scenario: Return to the library

- **WHEN** the window regains focus while the library is visible
- **THEN** a background incremental scan runs and the existing "Escaneando…" indicator is shown

#### Scenario: Debounce

- **WHEN** the window regains focus again within 30 seconds
- **THEN** no new scan starts

#### Scenario: Busy or empty library

- **WHEN** a scan is already running, there are no watched folders or the app is taking a capture
- **THEN** focus does not start a new scan

#### Scenario: Scan completes

- **WHEN** the incremental scan finishes
- **THEN** new, updated, missing and error entries are reflected without losing the current search, order or filter
