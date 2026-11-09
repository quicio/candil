## ADDED Requirements

### Requirement: Return to library from the reader

The reader SHALL offer a discreet control to return to the library in addition to the Escape shortcut.

#### Scenario: Back control

- **WHEN** the reader is visible
- **THEN** a small ← control is available and returns to the library when clicked

#### Scenario: Keyboard

- **WHEN** the user presses Escape in the reader
- **THEN** the library opens as before

### Requirement: External link confirmation

External links in the book SHALL require explicit confirmation and SHALL only open allowlisted schemes in the system application.

#### Scenario: HTTPS link

- **WHEN** the user clicks an external `https` link
- **THEN** a confirmation shows the URL and only "Abrir enlace" launches the system application

#### Scenario: Cancel

- **WHEN** the user chooses "Cancelar" or presses Escape
- **THEN** nothing opens and the reader stays where it was

#### Scenario: Unsafe scheme

- **WHEN** the link uses `javascript:`, `data:` or `file:`
- **THEN** no confirmation or external application is offered

#### Scenario: No automatic navigation

- **WHEN** the user scrolls or reads past an external link
- **THEN** nothing opens without a click and confirmation
