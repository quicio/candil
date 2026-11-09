## ADDED Requirements

### Requirement: Annotation filter

The annotations panel SHALL allow filtering saved highlights and notes by text without changing their stored data or the selected scope.

#### Scenario: Filter by quote or note

- **WHEN** the user types in the annotations filter
- **THEN** only rows whose quote or note contains the normalized text remain visible

#### Scenario: Filter by book or chapter

- **WHEN** the panel is in "Todos" and the query matches a book title or chapter
- **THEN** matching rows from those books remain visible

#### Scenario: No matches

- **WHEN** the filter excludes every row
- **THEN** the panel explains that there are no matches and offers a clear-filter action

#### Scenario: Clear filter

- **WHEN** the user clears the filter or presses its clear button
- **THEN** all rows for the selected scope are visible again

#### Scenario: Export with filter

- **WHEN** the user exports highlights while a filter is active
- **THEN** the Markdown includes only the visible matching rows
