## ADDED Requirements

### Requirement: Markdown rendering for AI and notes

The consultation panel and the note editor SHALL render a small, dependency-free Markdown subset so that AI answers and notes show hierarchy and emphasis instead of raw markers.

#### Scenario: AI answer with headings and lists

- **WHEN** an AI response contains headings, bullet or numbered lists
- **THEN** the panel shows them with visual hierarchy and without the leading Markdown markers

#### Scenario: Inline emphasis

- **WHEN** the text contains `**bold**`, `*italic*` or `` `code` ``
- **THEN** the markers disappear and the text uses the corresponding style, with code in the monospace face and accent color

#### Scenario: Quote and code block

- **WHEN** the response contains a `>` quote or a fenced code block
- **THEN** it is rendered in a distinct, readable surface with the theme colors

#### Scenario: Note preview

- **WHEN** a note has Markdown
- **THEN** the annotation list shows inline formatting and the note editor shows a full preview below the editor

#### Scenario: Non-Markdown consultations

- **WHEN** the result is a dictionary entry or a translation
- **THEN** it remains plain text and is not interpreted as Markdown

#### Scenario: Incomplete marker

- **WHEN** an emphasis marker is not closed
- **THEN** no text is lost; the remaining text is shown with the active style
