## MODIFIED Requirements

### Requirement: Search, order and filters

Search SHALL be the most discoverable library action; order and filter controls SHALL remain available without occupying permanent visual weight equal to search.

#### Scenario: Search by title or author
- **WHEN** the user types a query
- **THEN** the grid filters immediately and the query remains visible and editable

#### Scenario: Change order or filter
- **WHEN** the user opens the order or filter control
- **THEN** the current value is visible, the available options are listed and the grid updates on selection

#### Scenario: Filter by series
- **WHEN** the library has books with series metadata and the user opens the filter control
- **THEN** each detected series is listed once and selecting one shows only its books

#### Scenario: Filter books without series
- **WHEN** some books have no series and the user selects "Sin serie"
- **THEN** only books without series metadata are shown

#### Scenario: Content search toggle
- **WHEN** the library search is visible
- **THEN** a "Contenido" toggle is visible and indicates whether Enter will search inside the books

#### Scenario: No matches
- **WHEN** the query or filter returns no books
- **THEN** the library shows a specific empty-result message and a way to clear the query or filter

### Requirement: Full-text library search

The library SHALL search inside the text of its books on explicit request, run the search without blocking the UI, show progress and results with their source, and open the selected result at the matching passage without persisting an index.

#### Scenario: Start from the library search

- **WHEN** the "Contenido" toggle is active, the user types at least 3 characters and presses Enter
- **THEN** a background search starts across the available books

#### Scenario: Toggle starts the search

- **WHEN** the user activates "Contenido" with a query of at least 3 characters
- **THEN** the background search starts immediately

#### Scenario: Start from an empty metadata result

- **WHEN** the metadata filter has no matches and the query has at least 3 characters
- **THEN** the empty state offers a "Buscar en el contenido" action

#### Scenario: Progress and cancellation

- **WHEN** the content search is running
- **THEN** the UI shows how many books were scanned and offers a cancel action without freezing

#### Scenario: Result detail

- **WHEN** a match is found
- **THEN** the result shows the book, chapter and a snippet around the match

#### Scenario: Open a result

- **WHEN** the user selects a content result
- **THEN** the book opens at that chapter and the matching text is highlighted

#### Scenario: Wide results

- **WHEN** the results panel opens
- **THEN** its width and height scale with the window, up to 920×760, instead of using the small fixed panel

#### Scenario: Limits

- **WHEN** there are more matches than the configured limits
- **THEN** Candil stops at 100 results, 20 per book or 120 books and explains that the list is partial

#### Scenario: No index on disk

- **WHEN** the search finishes or is cancelled
- **THEN** no text index is written and `library.json` is unchanged

#### Scenario: Minimum query

- **WHEN** the query has fewer than 3 characters
- **THEN** the content search does not start and the panel explains the minimum

## ADDED Requirements

### Requirement: Google Drive local folder

The library SHALL offer to add the local Google Drive for desktop folder as a watched folder without requiring OAuth or storing Google credentials.

#### Scenario: Drive detected

- **WHEN** a Google Drive for desktop folder exists in a known location
- **THEN** the library offers "Añadir Google Drive…" and indexes that folder like any other

#### Scenario: Drive not detected

- **WHEN** no Google Drive folder is found
- **THEN** the action explains how to install Drive for desktop or use "Añadir carpeta…"

#### Scenario: No credentials

- **WHEN** the Google Drive folder is added
- **THEN** Candil does not call the Google API, does not ask for a Google account and does not write tokens

#### Scenario: Synced changes

- **WHEN** Drive updates files in the watched folder
- **THEN** the normal incremental rescan reflects new, changed or missing EPUB
