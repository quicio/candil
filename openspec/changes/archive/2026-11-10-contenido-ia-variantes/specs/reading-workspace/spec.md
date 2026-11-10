## ADDED Requirements

### Requirement: Tolerant AI content parsing

The AI consultation SHALL accept the common OpenAI-compatible shapes of `message.content` and SHALL explain why a response has no text instead of showing a generic failure.

#### Scenario: Content as parts

- **WHEN** the provider returns `message.content` as an array of text parts
- **THEN** Candil joins the parts and shows the response

#### Scenario: Content as object

- **WHEN** the provider returns `content` as an object with a `text` field
- **THEN** Candil uses that text

#### Scenario: Legacy completion

- **WHEN** the provider returns `choices[0].text` instead of `message.content`
- **THEN** Candil uses the legacy text

#### Scenario: No text because of token limit

- **WHEN** `content` is empty and `finish_reason` is `length`
- **THEN** the error explains that the token limit was reached and suggests raising **máx. tokens**

#### Scenario: Only reasoning

- **WHEN** `content` is empty but the provider returned reasoning fields
- **THEN** Candil reports that there is no final answer and suggests retrying or raising the limit

#### Scenario: Other stop reason

- **WHEN** `content` is empty and `finish_reason` has another value
- **THEN** the generic error includes that value for diagnosis
