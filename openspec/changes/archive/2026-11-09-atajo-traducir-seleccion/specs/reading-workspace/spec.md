## ADDED Requirements

### Requirement: Direct translation shortcut

With text selected, the reader SHALL offer a shortcut that opens the translation panel and starts the translation request without an extra click.

#### Scenario: Translate a selection

- **WHEN** the user has a text selection and presses Shift+K
- **THEN** the consultation panel opens on Traducción and the request starts immediately

#### Scenario: Online disabled

- **WHEN** the user presses Shift+K while online consultations are disabled
- **THEN** no request is sent and the panel explains how to enable them

#### Scenario: No selection

- **WHEN** the user presses Shift+K without a selection
- **THEN** nothing happens

#### Scenario: Existing shortcut

- **WHEN** the user presses K without Shift
- **THEN** the panel opens with the current behavior: dictionary for a word, translation for a phrase, without sending the request
