## ADDED Requirements

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
