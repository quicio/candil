## ADDED Requirements

### Requirement: Send EPUB to a device

The library SHALL let the user copy an EPUB to a mounted e-reader or folder without modifying the original file, without overwriting existing files and without converting formats.

#### Scenario: Kobo connected

- **WHEN** a Kobo volume with a `.kobo` directory is mounted
- **THEN** "Enviar a dispositivo…" offers "Enviar a Kobo" and copies the EPUB to `Books/` if present, otherwise to the root

#### Scenario: Generic device or folder

- **WHEN** the user chooses "Elegir carpeta o dispositivo…"
- **THEN** Candil copies the EPUB to the selected folder

#### Scenario: Existing name

- **WHEN** the destination already has a file with the same name
- **THEN** Candil creates a unique name such as `libro (2).epub` instead of overwriting

#### Scenario: Original untouched

- **WHEN** the copy finishes or fails
- **THEN** the library entry and the original EPUB remain unchanged

#### Scenario: Missing source

- **WHEN** the EPUB no longer exists at its library path
- **THEN** Candil reports it and does not create a file

#### Scenario: Kindle warning

- **WHEN** the device dialog is shown
- **THEN** it explains that Kindle normally does not read EPUB over USB and suggests Send to Kindle or a converted file

#### Scenario: No device detected

- **WHEN** no Kobo is mounted
- **THEN** the dialog still allows choosing a folder or canceling
