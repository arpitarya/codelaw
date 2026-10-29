---
id: css/todo-marker
title: "CSS TODO comments carry a ticket and, when asked, a date"
languages: [css]
severity: error
fix: manual
fix_reason: "a ticket and a date are facts about the work that only the author knows; the engine can't invent them"
stability: preview
since: 0.5.0
---

A comment line that opens with a marker tag (`TODO`, `FIXME`, `HACK` by default) in a `/* … */` comment carries a ticket, and a date when the project asks for one: `TODO(ABC-123, 2027-01-31): text`. The ticket and the date go in the parentheses, in either order, either one optional unless required; any other item in them (an owner's name) is ignored. The tag must start the comment line, so a sentence that merely mentions a TODO is not a marker.

A date that is written is always enforced, whether or not `require_date` is set: it is **expired** from the as-of date on (`--as-of`, `CODELAW_AS_OF`, else `HEAD`'s commit date, never the clock: [SR-BASELINE](../../../../records/0104_baseline-and-deprecations.md) decision 9) and is a **warning** for the `warn_days` before it. A date that is not a calendar day is reported too. With no as-of date, only the missing and the malformed are reported.

**Why:** A TODO with no ticket has no owner and no place to be tracked, and one with no date never comes due; both outlive the people who wrote them. Naming a ticket ties the comment to the work, and a date makes the comment fail loudly the day it is overdue instead of rotting quietly.

**Configuration** is the `[todo]` table of `codelaw.toml` ([SR-CONFIG](../../../../records/0110_config.md) decision 11a), one for every language:

```toml
[todo]
tags           = ["TODO", "FIXME", "HACK"]   # the words that open a marker
require_ticket = true                         # a ticket is required (default)
require_date   = false                        # a date is required (default: only a written date is enforced)
ticket_pattern = "[A-Z][A-Z0-9]+-\\d+"        # a whole ticket must match this regular expression
warn_days      = 14                           # warn this many days before the date
```

**A repo full of bare TODOs** gets every one of them at once. Take them into the baseline (`codelaw baseline`) and pay them down, or set `require_ticket = false` and let the rule enforce only the dates that are written.

```codelaw
native: css/todo-marker
```

## How to fix

1. Open or find the ticket for the work and write it in the parentheses: `TODO(ABC-123): …`.
2. When the project asks for a date (`require_date`), or the work has a deadline, add it: `TODO(ABC-123, 2027-01-31): …`.
3. For an expired marker: do the work and delete the comment; or, if the work really moves, change the date and say why in the text.
4. If the comment is not a to-do, rewrite it so it does not open with the tag.

## Examples

### CSS

#### Don't

```css
/* FIXME: remove after the redesign */
.legacy { margin: 0; }
```

#### Do

```css
/* FIXME(UI-12): remove after the redesign */
.legacy { margin: 0; }
```
