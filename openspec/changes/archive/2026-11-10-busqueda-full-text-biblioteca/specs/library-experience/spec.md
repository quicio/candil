## ADDED Requirements

### Requirement: Full-text library search

The library SHALL search inside the text of its books on explicit request, run the search without blocking the UI, show progress and results with their source, and open the selected result at the matching passage without persisting an index.

#### Scenario: Start from the library search

- **WHEN** the user types at least 3 characters and presses Enter in the library search field
- **THEN** a background search starts across the available books

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

#### Scenario: Limits

- **WHEN** there are more matches than the configured limits
- **THEN** Candil stops at 100 results, 20 per book or 120 books and explains that the list is partial

#### Scenario: No index on disk

- **WHEN** the search finishes or is cancelled
- **THEN** no text index is written and `library.json` is unchanged

#### Scenario: Minimum query

- **WHEN** the query has fewer than 3 characters
- **THEN** the content search does not start and the panel explains the minimum
