## Purpose

Define the library as a calm personal shelf: easy to scan, easy to search, and free of dashboard chrome that competes with the books.

## ADDED Requirements

### Requirement: Library header hierarchy

The library SHALL present the product identity, the library count and the search field as the primary header elements; add, refresh, settings and folder management SHALL be secondary.

#### Scenario: Default window
- **WHEN** the library is shown at 980×780
- **THEN** the header shows Candil, the book count, a prominently sized search field and a primary add action without overlapping or clipping any control

#### Scenario: Narrow window
- **WHEN** the library is shown at 720×600
- **THEN** secondary actions collapse into an overflow menu and no label, field or control overlaps another

### Requirement: Search, order and filters

Search SHALL be the most discoverable library action; order and filter controls SHALL remain available without occupying permanent visual weight equal to search.

#### Scenario: Search by title or author
- **WHEN** the user types a query
- **THEN** the grid filters immediately and the query remains visible and editable

#### Scenario: Change order or filter
- **WHEN** the user opens the order or filter control
- **THEN** the current value is visible, the available options are listed and the grid updates on selection

#### Scenario: No matches
- **WHEN** the query or filter returns no books
- **THEN** the library shows a specific empty-result message and a way to clear the query or filter

### Requirement: Book card

Each book card SHALL show a cover, title, author, reading progress and status; the whole card SHALL be the open target.

#### Scenario: Book with cover
- **WHEN** a book has a cover
- **THEN** the cover is shown without upscaling or distortion and the card has a visible hover and focus state

#### Scenario: Book without cover
- **WHEN** a book has no cover
- **THEN** the card shows a neutral placeholder that does not imitate a real cover

#### Scenario: Book missing or broken
- **WHEN** a book file is missing or cannot be parsed
- **THEN** the card shows the specific status and offers a recovery action instead of only a generic error label

### Requirement: Library empty state

The empty library SHALL explain what Candil needs and offer one primary action plus secondary alternatives.

#### Scenario: No folders configured
- **WHEN** the library has no books and no folders
- **THEN** the empty state shows a primary "Añadir carpeta" action, a secondary "Abrir un EPUB" action and the drag-and-drop hint

#### Scenario: Folders configured but empty
- **WHEN** folders are configured but contain no EPUB
- **THEN** the empty state explains that no EPUB were found and offers refresh plus add-folder actions

### Requirement: Library error and missing states

Library-level errors SHALL be recoverable and SHALL NOT require the user to guess what happened.

#### Scenario: One broken EPUB
- **WHEN** a scan finds an EPUB that cannot be opened
- **THEN** the header shows the error count, the card shows the affected book and the card exposes a detail or remove action

#### Scenario: Missing file
- **WHEN** a previously indexed file no longer exists
- **THEN** the card marks it as missing and offers to locate or remove the entry without deleting the underlying file

### Requirement: Folder management

Folder chips SHALL be visible only when folders exist and SHALL allow stopping watch without deleting books or files.

#### Scenario: Remove a folder
- **WHEN** the user removes a folder chip
- **THEN** Candil stops watching that folder, confirms the action and leaves the EPUB files untouched

### Requirement: Library scanning feedback

Scanning and refreshing SHALL provide timely feedback without blocking the window.

#### Scenario: Refresh a large library
- **WHEN** the user triggers a refresh
- **THEN** the library shows progress or a busy state and remains responsive until the scan finishes
