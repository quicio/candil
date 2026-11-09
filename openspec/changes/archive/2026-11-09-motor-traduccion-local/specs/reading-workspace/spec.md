## ADDED Requirements

### Requirement: Configurable translation engine

The consultation panel SHALL let the user choose between the free online
translation service and a local Ollama model, and SHALL apply that choice to
both the translate button and the direct translation shortcut.

#### Scenario: Choose online translation

- **WHEN** the translation provider is MyMemory
- **THEN** the panel shows the 500-character limit, the request requires online access and the response identifies MyMemory

#### Scenario: Choose local translation

- **WHEN** the user selects Ollama and sets a model
- **THEN** the panel shows the local model, the request goes to `localhost:11434`, no API key is sent and the response identifies Ollama

#### Scenario: Local translation without online access

- **WHEN** online consultations are disabled but the translation provider is Ollama
- **THEN** the translate button and `⇧K` still run locally and the panel explains that the text does not leave the Mac

#### Scenario: Ollama unavailable

- **WHEN** Ollama is not running or the configured model is not installed
- **THEN** the panel explains how to start Ollama or pull the model instead of showing a raw HTTP error

### Requirement: MyMemory length limit

The online translation SHALL send at most 500 characters, SHALL cut at a word
boundary and SHALL tell the user when the translation covers only part of the
selection.

#### Scenario: Long selection

- **WHEN** the selected text exceeds 500 characters
- **THEN** Candil translates the fragment up to the last complete word before the limit and adds a visible notice that the translation is partial

#### Scenario: No word boundary

- **WHEN** the text has no spaces within the limit
- **THEN** Candil cuts exactly at 500 characters and still never sends more than the limit

#### Scenario: Provider rejects the length

- **WHEN** MyMemory responds with a query-length error
- **THEN** Candil shows a friendly message about the 500-character limit instead of the raw provider detail

## MODIFIED Requirements

### Requirement: Direct translation shortcut

With text selected, the reader SHALL offer a shortcut that opens the translation panel and starts the translation request without an extra click, using the configured translation engine.

#### Scenario: Translate a selection

- **WHEN** the user has a text selection and presses Shift+K
- **THEN** the consultation panel opens on Traducción and the request starts immediately with the configured engine

#### Scenario: Online disabled

- **WHEN** the user presses Shift+K while online consultations are disabled and the engine is MyMemory
- **THEN** no request is sent and the panel explains how to enable online access or switch to Ollama

#### Scenario: Online disabled with local engine

- **WHEN** the user presses Shift+K while online consultations are disabled and the engine is Ollama
- **THEN** the request runs locally without enabling online access

#### Scenario: No selection

- **WHEN** the user presses Shift+K without a selection
- **THEN** nothing happens

#### Scenario: Existing shortcut

- **WHEN** the user presses K without Shift
- **THEN** the panel opens with the current behavior: dictionary for a word, translation for a phrase, without sending the request
