## ADDED Requirements

### Requirement: Complete AI responses

The consultation panel SHALL use a token budget large enough for a passage explanation, SHALL detect when the provider stops because of that limit, and SHALL tell the user instead of presenting a truncated answer as complete.

#### Scenario: Default token budget

- **WHEN** a new or existing configuration uses the old 512-token default
- **THEN** Candil uses 1536 tokens and a custom value other than 512 is preserved

#### Scenario: Provider cuts the response

- **WHEN** the provider reports `length`, `max_tokens` or `MAX_TOKENS` as the stop reason
- **THEN** the response includes a visible notice with the current token limit and how to raise it

#### Scenario: Complete response

- **WHEN** the provider stops normally
- **THEN** no truncation notice is added

#### Scenario: Long selection

- **WHEN** the selected text exceeds 2000 characters
- **THEN** the panel warns that only the first 2000 characters will be sent
