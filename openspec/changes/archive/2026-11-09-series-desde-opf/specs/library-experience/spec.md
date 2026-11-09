## ADDED Requirements

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

#### Scenario: Books without series

- **WHEN** some books have no series
- **THEN** those books appear after the series groups, ordered by title

### Requirement: Metadata refresh

Existing entries without the current metadata version SHALL be re-read once on the next scan without losing progress, settings, cover or dates.

#### Scenario: Existing library

- **WHEN** an old `library.json` is scanned after the update
- **THEN** series fields are populated and `progress`, `settings`, `last_opened`, `added` and `cover` are preserved
