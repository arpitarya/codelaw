---
id: sec-py/no-prompt-injection
title: 'A request value is not written into a system prompt'
languages: [python]
fix: manual
fix_reason: "what goes in the system role and what goes in the user role is a design decision"
stability: preview
security: { cwe: [1427], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

A value from the request is not part of the system prompt of a language-model call.

**Why:** The model cannot tell your instructions from the user's text, so a user can rewrite the rules the system prompt sets (CWE-1427). This rule uses `from_source:`, which is not taint analysis: it sees a request value only in the function that reads it, never through a call or a parameter.

```codelaw
match:
  any:
    - all:
        - { kind: dictionary }
        - has:
            kind: pair
            all:
              - { has: { field: key, regex: '^[''"]role[''"]$' } }
              - { has: { field: value, regex: '^[''"]system[''"]$' } }
        - has:
            kind: pair
            all:
              - { has: { field: key, regex: '^[''"]content[''"]$' } }
              - { has: { field: value, pattern: $V } }
    - { pattern: 'SystemMessage($V)' }
    - { pattern: 'SystemMessage(content=$V)' }
    - all:
        - { kind: keyword_argument }
        - { has: { field: name, regex: '^(system|system_prompt|instructions)$' } }
        - { has: { field: value, pattern: $V } }
where: { $V: { from_source: { tables: [sources/django, sources/flask, sources/fastapi] } } }
message: '`$V` comes from the request and is part of the system prompt'
```

## How to fix

1. Keep the system prompt a constant.
2. Send the user's text as a separate `user` message, never concatenated into the system one.
3. Treat the model's answer as untrusted too (see `sec-py/no-llm-output-to-sink`).

## Examples

### Python

#### Don't

```py
def ask(request):
    topic = request.GET["topic"]
    return client.messages.create(system=f"You help with {topic}.", messages=msgs)
```

#### Do

```py
def ask(request):
    msgs = [{"role": "user", "content": request.GET["topic"]}]
    return client.messages.create(system=SYSTEM_PROMPT, messages=msgs)
```
