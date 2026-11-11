## MODIFIED Requirements

### Requirement: Position bookmarks

The reader SHALL let the user save the current reading position as a bookmark, optionally label it, review the bookmarks of the open book, jump back to one, rename it, and delete it, without modifying the EPUB.

#### Scenario: Save a bookmark

- **WHEN** the user presses B in the reader without a selection
- **THEN** Candil saves the current chapter, anchor and fraction without a label and confirms it

#### Scenario: Save with a label

- **WHEN** the user writes a label in the bookmark panel and presses "Añadir marcador aquí"
- **THEN** the bookmark is saved with that label and the field is cleared

#### Scenario: Rename a bookmark

- **WHEN** the user selects "Editar" on a bookmark and saves a new label
- **THEN** the row shows the new label and `library.json` is updated

#### Scenario: Empty label

- **WHEN** a bookmark has an empty label
- **THEN** the row falls back to the chapter title and percentage

#### Scenario: Avoid duplicates

- **WHEN** the user saves a bookmark at the same anchor and within 1% of an existing bookmark
- **THEN** Candil does not add a duplicate and explains why

#### Scenario: Open the bookmark panel

- **WHEN** the user presses Shift+B or uses the bookmark control in the reader bar
- **THEN** the panel lists the bookmarks of the current book with label (or chapter title) and percentage

#### Scenario: Jump to a bookmark

- **WHEN** the user selects a bookmark
- **THEN** the reader opens its chapter and restores the saved anchor and fraction

#### Scenario: Delete a bookmark

- **WHEN** the user removes a bookmark from the panel
- **THEN** it disappears from the list and from `library.json`

#### Scenario: Preserve bookmarks on rescan

- **WHEN** the library rescans or relinks the EPUB
- **THEN** the existing bookmarks and their labels are preserved

#### Scenario: No bookmarks

- **WHEN** the open book has no bookmarks
- **THEN** the panel explains how to add the first one
