## ADDED Requirements

### Requirement: Startup destination

The app SHALL let the user choose whether to open the library or the last valid book on startup, defaulting to the last book.

#### Scenario: Default startup

- **WHEN** the user opens Candil and a previous book exists
- **THEN** the reader opens that book at its saved progress

#### Scenario: Missing last book

- **WHEN** the last book path no longer exists
- **THEN** the app opens the library instead of failing

#### Scenario: Library preference

- **WHEN** the user selects "Biblioteca" in Configuración → Biblioteca
- **THEN** the next startup opens the library even if a last book exists

#### Scenario: Explicit path or resume

- **WHEN** the app is launched with an EPUB path, `--resume` or `--screen library`
- **THEN** that explicit option takes precedence over the stored preference
