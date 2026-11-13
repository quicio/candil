# library-experience Specification

## Purpose
Define the library as a calm personal shelf: easy to scan, easy to search, and free of dashboard chrome that competes with the books.
## Requirements
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

### Requirement: Series metadata

The library SHALL read series name and position from the EPUB OPF and SHALL persist them with each book.

#### Scenario: Calibre metadata

- **WHEN** the OPF contains `calibre:series` and `calibre:series_index`
- **THEN** the card shows the series name and position and the values survive a rescan

#### Scenario: EPUB3 collection

- **WHEN** the OPF contains `belongs-to-collection` with `collection-type=series` and `group-position`
- **THEN** the card shows that collection as the series and its position

#### Scenario: No series

- **WHEN** the OPF does not declare a series
- **THEN** the card adds no series line and the book remains valid

### Requirement: Series-aware ordering

The library order SHALL offer a series order that groups books by series name and position.

#### Scenario: Books in a series

- **WHEN** the user selects the series order
- **THEN** books with the same series appear together ordered by position, then title

#### Scenario: Series group header

- **WHEN** the series order is active and at least one book declares a series
- **THEN** each group starts with a header that shows the series name and the number of books in the group

#### Scenario: Books without series

- **WHEN** some books have no series
- **THEN** those books appear after the series groups under a "Sin serie" header, ordered by title

### Requirement: Metadata refresh

Existing entries without the current metadata version SHALL be re-read once on the next scan without losing progress, settings, cover or dates.

#### Scenario: Existing library

- **WHEN** an old `library.json` is scanned after the update
- **THEN** series fields are populated and `progress`, `settings`, `last_opened`, `added` and `cover` are preserved

### Requirement: Startup destination

The app SHALL let the user choose whether to open the library or the last valid book on startup, defaulting to the last book.

#### Scenario: Default startup

- **WHEN** the user opens Candil and a previous book exists
- **THEN** the reader opens that book at its saved progress

#### Scenario: Missing last book

- **WHEN** the last book path no longer exists
- **THEN** the app opens the library instead of failing

#### Scenario: Library preference

- **WHEN** the user selects "Biblioteca" in Configuración → Biblioteca
- **THEN** the next startup opens the library even if a last book exists

#### Scenario: Explicit path or resume

- **WHEN** the app is launched with an EPUB path, `--resume` or `--screen library`
- **THEN** that explicit option takes precedence over the stored preference

### Requirement: Duplicate detection

The library SHALL identify possible duplicates by comparing normalized title and author, SHALL let the user review them together, and SHALL NOT delete or move any file automatically.

#### Scenario: Same title and author

- **WHEN** two entries share the same title and author after ignoring case and accents
- **THEN** both are marked as possible duplicates with the number of copies

#### Scenario: Unique book

- **WHEN** an entry has no other entry with the same title and author
- **THEN** it is not marked and does not appear in the duplicate filter

#### Scenario: Review duplicates

- **WHEN** the user selects the "Duplicados" filter
- **THEN** only entries with at least one possible duplicate are shown and each card keeps its normal open, locate, reveal and remove actions

#### Scenario: Conservative copy

- **WHEN** a book is marked as a possible duplicate
- **THEN** Candil explains that the criterion is title and author and does not remove the entry or the file without an explicit action

#### Scenario: Empty title

- **WHEN** an entry has no title
- **THEN** it is never grouped as a duplicate

### Requirement: Large library virtualization

The library SHALL render only the rows near the viewport when the visible grid is large, SHALL preserve series headers and card actions, and SHALL keep the full rendering path for normal-sized libraries.

#### Scenario: Normal library

- **WHEN** the grid has fewer than 240 visible books
- **THEN** every card is rendered and remains focusable as before

#### Scenario: Large library

- **WHEN** the grid has 240 or more visible books
- **THEN** only rows intersecting the viewport, plus a margin, are built and drawn

#### Scenario: Series order

- **WHEN** the large grid uses the series order
- **THEN** each series header and its row positions are preserved

#### Scenario: Card actions

- **WHEN** a rendered card is clicked or opened from its context menu
- **THEN** the same open, locate, recover, edit and remove actions run

#### Scenario: Keyboard focus trade-off

- **WHEN** the large grid is virtualized
- **THEN** keyboard focus reaches rendered cards, and the documentation explains that off-screen cards are created when scrolled into view

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

### Requirement: Google Drive local folder

The library SHALL offer to add a chosen subfolder of the local Google Drive for desktop as a watched folder without requiring OAuth or storing Google credentials.

#### Scenario: Drive detected

- **WHEN** a Google Drive for desktop folder exists in a known location
- **THEN** "Añadir Google Drive…" opens a folder picker at the Drive root so the user can choose a subfolder

#### Scenario: Choose a subfolder

- **WHEN** the user selects a subfolder inside Google Drive
- **THEN** that folder is indexed like any other watched folder and `My Drive` is not added wholesale

#### Scenario: Already indexed

- **WHEN** the chosen folder is already in the library
- **THEN** Candil does not duplicate it and says it is already indexed

#### Scenario: Cloud-only files

- **WHEN** the folder contains EPUB that Drive has not downloaded
- **THEN** the message explains that Drive downloads them when opened and recommends marking the folder as available offline

#### Scenario: Drive not detected

- **WHEN** no Google Drive folder is found
- **THEN** the action explains how to install Drive for desktop or use "Añadir carpeta…"

#### Scenario: No credentials

- **WHEN** the Google Drive folder is added
- **THEN** Candil does not call the Google API, does not ask for a Google account and does not write tokens

#### Scenario: Synced changes

- **WHEN** Drive updates files in the watched folder
- **THEN** the normal incremental rescan reflects new, changed or missing EPUB

