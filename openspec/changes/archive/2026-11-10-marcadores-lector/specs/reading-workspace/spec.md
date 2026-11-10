## ADDED Requirements

### Requirement: Position bookmarks

The reader SHALL let the user save the current reading position as a bookmark, review the bookmarks of the open book, jump back to one, and delete it, without modifying the EPUB.

#### Scenario: Save a bookmark

- **WHEN** the user presses B in the reader without a selection
- **THEN** Candil saves the current chapter, anchor and fraction and confirms it

#### Scenario: Avoid duplicates

- **WHEN** the user saves a bookmark at the same anchor and within 1% of an existing bookmark
- **THEN** Candil does not add a duplicate and explains why

#### Scenario: Open the bookmark panel

- **WHEN** the user presses Shift+B or uses the bookmark control in the reader bar
- **THEN** the panel lists the bookmarks of the current book with chapter title and percentage

#### Scenario: Jump to a bookmark

- **WHEN** the user selects a bookmark
- **THEN** the reader opens its chapter and restores the saved anchor and fraction

#### Scenario: Delete a bookmark

- **WHEN** the user removes a bookmark from the panel
- **THEN** it disappears from the list and from `library.json`

#### Scenario: Preserve bookmarks on rescan

- **WHEN** the library rescans or relinks the EPUB
- **THEN** the existing bookmarks are preserved with progress and settings

#### Scenario: No bookmarks

- **WHEN** the open book has no bookmarks
- **THEN** the panel explains how to add the first one
