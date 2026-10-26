---
id: sec-py/no-llm-output-to-sink
title: 'Model output is not run or rendered'
languages: [python]
severity: warn
fix: manual
fix_reason: "what to do with the output instead (parse it against a schema, pick from a fixed list of actions) is a design choice"
stability: preview
security: { cwe: [94], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

The reply of an OpenAI, Anthropic or LangChain call does not reach `exec`, `eval`, a subprocess, SQL or `mark_safe`.

**Why:** A model's output is shaped by its input, so a prompt written by an attacker becomes code, a command, a query or a script (CWE-94).

```codelaw
match:
  all:
    - { kind: call }
    - has:
        field: function
        regex: '^(exec|eval|os\.system|os\.popen|subprocess\.(run|call|check_call|check_output|Popen)|mark_safe|Markup|(.*\.)?(execute|executescript))$'
    - has:
        field: arguments
        has:
          regex: '(?i)(\.choices\[0\]\.message\.content|\.content\[0\]\.text|\.content\b|\.text\b|llm_output|completion|model_output|llm_response|ai_response)'
message: 'The output of a language model reaches a code, command, query or HTML sink'
```

## How to fix

1. Ask the model for structured output and validate it against a schema.
2. Map the answer to a fixed list of actions the program already has; never pass it to a sink as text.
3. Escape it before it goes into HTML; use parameters for SQL.

## Examples

### Python

#### Don't

```py
reply = client.chat.completions.create(model=m, messages=msgs)
exec(reply.choices[0].message.content)
```

#### Do

```py
reply = client.chat.completions.create(model=m, messages=msgs)
action = reply.choices[0].message.content.strip()
if action in ACTIONS:
    ACTIONS[action]()
```
