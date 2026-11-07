## Purpose

Define the reading workspace as the place where the book dominates and every supporting tool appears only when needed, then recedes.

## ADDED Requirements

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

Selecting text SHALL reveal a compact action bar that creates a highlight, opens a note or starts a query without moving the selection.

#### Scenario: Select a phrase
- **WHEN** the user selects text
- **THEN** the action bar appears near the selection and offers highlight, note and query actions

#### Scenario: Choose a highlight color
- **WHEN** the action bar or note editor shows the three colors
- **THEN** each color is a real swatch with a visible selected state and a text label or tooltip, and no swatch renders as a missing-glyph box

#### Scenario: Escape selection
- **WHEN** the user presses Escape or clicks outside
- **THEN** the action bar closes and the text selection is cleared

### Requirement: Annotations panel

The annotations panel SHALL make highlights and notes scannable and SHALL support returning to the quoted text.

#### Scenario: Book scope
- **WHEN** the user opens the panel for the current book
- **THEN** highlights are listed with quote, note if present, chapter and a clear "Ir al texto" action

#### Scenario: Global scope
- **WHEN** the user switches to "Todos"
- **THEN** each row shows the book title and chapter, and jumping opens the correct book and highlights the quote

#### Scenario: Edit or delete
- **WHEN** the user edits a note or deletes a highlight
- **THEN** the change is persisted and destructive deletion either asks for confirmation or offers undo

#### Scenario: Missing quote
- **WHEN** a saved quote cannot be found in its chapter
- **THEN** the row explains that the quote was not found and does not pretend the jump succeeded

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
