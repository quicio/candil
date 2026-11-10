## ADDED Requirements

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
