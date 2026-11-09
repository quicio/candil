# reading-workspace Specification

## Purpose
Define the reading workspace as the place where the book dominates and every supporting tool appears only when needed, then recedes.
## Requirements
### Requirement: Reading surface dominance

The reader SHALL dedicate the largest visual area to the book text and SHALL keep chrome visually subordinate.

#### Scenario: Default reading view
- **WHEN** a chapter is open
- **THEN** the top bar is quiet, the progress indicator is minimal, and the text column is the strongest element on screen

#### Scenario: Long reading session
- **WHEN** the user reads for an extended period
- **THEN** no element flashes, animates continuously or competes with the text for attention

### Requirement: Reader top bar

The reader top bar SHALL expose index, typography, notes, search, settings, chapter navigation and progress without crowding the book title.

#### Scenario: Book with long title
- **WHEN** the title and author are long
- **THEN** the title truncates deliberately and the navigation controls remain visible and usable

#### Scenario: Narrow window
- **WHEN** the reader is shown at 720×600
- **THEN** low-priority actions collapse or move to an overflow control instead of overlapping the title or navigation

#### Scenario: Current position
- **WHEN** the user scrolls or changes chapter
- **THEN** the chapter number, total chapters and percentage update without layout shift

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

### Requirement: Panel consistency

All reader panels SHALL share a common frame, header, close affordance, inner padding and width role.

#### Scenario: Open two panels
- **WHEN** the user opens typography, annotations or search
- **THEN** the title, close control and body padding occupy the same positions and the panel width matches its role

#### Scenario: Non-collapsible panel
- **WHEN** a panel cannot be collapsed
- **THEN** it does not show a collapse arrow

#### Scenario: Panel placement
- **WHEN** a panel opens
- **THEN** it does not permanently move the reading column or reset the reading position

### Requirement: Search and navigation panels

Search and go-to SHALL make the current target, result count and keyboard actions explicit.

#### Scenario: Search results
- **WHEN** the query returns results
- **THEN** each result shows context, chapter and a visible hover/focus state, and Enter or click jumps to the match

#### Scenario: Search truncated
- **WHEN** the result limit is reached
- **THEN** the panel says that only the first results are shown and asks the user to refine the query

#### Scenario: Go to percentage or chapter
- **WHEN** the user opens "Ir a"
- **THEN** percentage and chapter targets are clearly labeled, validated and confirmed by an explicit action

### Requirement: Settings and configuration

Configuration SHALL be organized by task, use the shared panel system and make destructive actions explicit.

#### Scenario: Reading settings
- **WHEN** the user opens typography
- **THEN** theme, size, line height, measure and paragraph spacing are grouped and the per-book override is clearly distinguished from the global setting

#### Scenario: Online consent
- **WHEN** the user views consultas settings
- **THEN** online access is off by default, the consequence of enabling it is explained and enabling requires an explicit action

#### Scenario: Clear history
- **WHEN** the user clears the consultation history
- **THEN** Candil asks for confirmation and reports the result

### Requirement: Consultation panel

The consultation panel SHALL support dictionary, translation, AI and history without turning the reader into a chat product.

#### Scenario: Dictionary result
- **WHEN** the user queries a word
- **THEN** the result shows provider, language pair and sections in a readable hierarchy

#### Scenario: History detail
- **WHEN** the user opens a saved consultation
- **THEN** the panel shows the query, response, book, chapter and an action to return to the source text

#### Scenario: Offline
- **WHEN** online access is disabled
- **THEN** the panel explains how to enable it and does not attempt a request

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

### Requirement: Internal link preview and return

Clicking an internal link SHALL show a preview of its destination without moving the reading position, and jumping to that destination SHALL offer a way back to the previous position.

#### Scenario: Preview a footnote in the same chapter

- **WHEN** the user clicks a reference whose target is a block in the current chapter
- **THEN** a floating preview shows the target text and the chapter, with "Ir al texto" and "Volver" actions, and the scroll and chapter do not change

#### Scenario: Preview a note in another document

- **WHEN** the link points to an anchor in another XHTML document of the spine
- **THEN** the target chapter is read to build the preview, the visible chapter stays the same, and the preview identifies the destination chapter

#### Scenario: Jump and return

- **WHEN** the user chooses "Ir al texto"
- **THEN** the reader jumps to the target and shows a "Volver al texto" control that restores the previous chapter, block and scroll fraction

#### Scenario: Dismiss without navigating

- **WHEN** the user chooses "Volver", presses Escape or scrolls while the preview is open
- **THEN** the preview closes and the reading position and chapter remain unchanged

#### Scenario: Destination without text

- **WHEN** the target anchor resolves to a block without readable text or to an empty chapter
- **THEN** the preview explains that there is no text and still allows closing or attempting the jump without crashing

#### Scenario: External or unresolved link

- **WHEN** the link is external or its target is not in the spine
- **THEN** no preview opens and the existing behavior is preserved

#### Scenario: Narrow window

- **WHEN** the reader is shown at 720×600
- **THEN** the preview fits on screen, stays anchored near the clicked link and does not cover the reference being read

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

