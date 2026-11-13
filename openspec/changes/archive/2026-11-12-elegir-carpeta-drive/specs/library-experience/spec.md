## MODIFIED Requirements

### Requirement: Google Drive local folder

The library SHALL offer to add a chosen subfolder of the local Google Drive for desktop as a watched folder without requiring OAuth or storing Google credentials.

#### Scenario: Drive detected

- **WHEN** a Google Drive for desktop folder exists in a known location
- **THEN** "Añadir Google Drive…" opens a folder picker at the Drive root so the user can choose a subfolder

#### Scenario: Choose a subfolder

- **WHEN** the user selects a subfolder inside Google Drive
- **THEN** that folder is indexed like any other watched folder and `My Drive` is not added wholesale

#### Scenario: Already indexed

- **WHEN** the chosen folder is already in the library
- **THEN** Candil does not duplicate it and says it is already indexed

#### Scenario: Cloud-only files

- **WHEN** the folder contains EPUB that Drive has not downloaded
- **THEN** the message explains that Drive downloads them when opened and recommends marking the folder as available offline

#### Scenario: Drive not detected

- **WHEN** no Google Drive folder is found
- **THEN** the action explains how to install Drive for desktop or use "Añadir carpeta…"

#### Scenario: No credentials

- **WHEN** the Google Drive folder is added
- **THEN** Candil does not call the Google API, does not ask for a Google account and does not write tokens

#### Scenario: Synced changes

- **WHEN** Drive updates files in the watched folder
- **THEN** the normal incremental rescan reflects new, changed or missing EPUB
