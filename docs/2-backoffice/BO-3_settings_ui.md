# BO-3: Configuration Interface

## Description
As an administrator, I want an easy way to update LLM providers and API keys through a secure settings panel.

## Acceptance Criteria
- [ ] Implement a "Settings" page with fields for LLM Base URL, Model, and API Key.
- [ ] Show a "Success" toast notification upon saving configuration.
- [ ] Implement validation to ensure required fields are not empty.
- [ ] Mask the API Key field by default in the UI.

## Dependencies
- **BE-3**: Backend settings endpoints.
