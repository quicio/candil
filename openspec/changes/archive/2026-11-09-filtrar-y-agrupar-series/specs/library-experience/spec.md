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

#### Scenario: No matches
- **WHEN** the query or filter returns no books
- **THEN** the library shows a specific empty-result message and a way to clear the query or filter

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
