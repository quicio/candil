## ADDED Requirements

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
