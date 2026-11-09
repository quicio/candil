## MODIFIED Requirements

### Requirement: Text selection and highlight creation

Selecting text SHALL reveal a compact action bar that creates a highlight, opens a note or starts a query without moving the selection. The selection SHALL work within one block and across consecutive blocks of the same chapter.

#### Scenario: Select a phrase

- **WHEN** the user selects text inside a paragraph
- **THEN** the action bar appears near the selection and offers highlight, note and query actions

#### Scenario: Extend selection across paragraphs

- **WHEN** the user drags the selection past the edge of a paragraph or presses Shift and clicks in another paragraph of the same chapter
- **THEN** the selection covers the text from the original anchor to the new point, visually spans every covered block and keeps the action bar anchored near the range

#### Scenario: Extend selection backwards

- **WHEN** the user extends the selection to a point before the original anchor
- **THEN** the stored range starts at the earlier point and ends at the later one, without duplicated or inverted text

#### Scenario: Choose a highlight color

- **WHEN** the action bar or note editor shows the three colors
- **THEN** each color is a real swatch with a visible selected state and a text label or tooltip, and no swatch renders as a missing-glyph box

#### Scenario: Escape selection

- **WHEN** the user presses Escape or clicks outside
- **THEN** the action bar closes and the text selection is cleared

#### Scenario: Empty or non-text selection

- **WHEN** the pointer only crosses images, rules or whitespace
- **THEN** no highlight, note or query action is offered

### Requirement: Annotations panel

The annotations panel SHALL make highlights and notes scannable and SHALL support returning to the quoted text, including annotations that span more than one block.

#### Scenario: Book scope

- **WHEN** the user opens the panel for the current book
- **THEN** highlights are listed with quote, note if present, chapter and a clear "Ir al texto" action

#### Scenario: Multi-paragraph row

- **WHEN** an annotation covers two or more blocks
- **THEN** its row shows a single-line preview with ellipsis, states how many paragraphs it covers and does not grow unbounded

#### Scenario: Global scope

- **WHEN** the user switches to "Todos"
- **THEN** each row shows the book title and chapter, and jumping opens the correct book and highlights the quote

#### Scenario: Edit or delete

- **WHEN** the user edits a note or deletes a highlight
- **THEN** the change is persisted and destructive deletion either asks for confirmation or offers undo

#### Scenario: Missing quote

- **WHEN** a saved quote cannot be found in its chapter
- **THEN** the row explains that the quote was not found and does not pretend the jump succeeded

#### Scenario: Jump to a multi-paragraph annotation

- **WHEN** the user jumps to an annotation that spans several blocks
- **THEN** the reader opens the chapter, scrolls to the first block of the range and shows the whole range highlighted

## ADDED Requirements

### Requirement: Multi-paragraph annotation ranges

An annotation SHALL be able to store a range that begins and ends in different blocks of the same chapter, SHALL persist enough information to re-anchor it, and SHALL remain compatible with previously saved single-block annotations.

#### Scenario: Persist a multi-paragraph range

- **WHEN** the user highlights or saves a note over several paragraphs
- **THEN** `annotations.json` records the start block and offset, the end block and offset, the full quote and the text of each covered block, and reloading the book restores the same range

#### Scenario: Migrate existing annotations

- **WHEN** Candil loads an `annotations.json` written by a previous version
- **THEN** every existing annotation is interpreted as a single-block range with its original offsets and quote, and no annotation is lost or duplicated

#### Scenario: Text changed in the book

- **WHEN** the stored offsets no longer match but the per-block quotes are still found in order
- **THEN** the annotation is re-anchored to the matching blocks and highlighted there

#### Scenario: Partial quote match

- **WHEN** only some of the per-block quotes can be found in the chapter
- **THEN** the annotation is reported as not found instead of highlighting a misleading partial range

#### Scenario: Rekey after moving a book

- **WHEN** the user relocates an EPUB to another path
- **THEN** all annotations of that book, including multi-paragraph ranges, move to the new path without changing their range or quote
