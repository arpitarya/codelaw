# acme-shop

A small shop: a TypeScript web client in `web/` and a Python API in `api/`.

## Layout

- `web/src/` — the client. `web/src/lib/http.ts` is the HTTP client.
- `api/acme/` — the API package. `api/acme/http.py` is the outbound HTTP client,
  `api/acme/db.py` the database helpers, `api/acme/clock.py` the clock.
- `api/migrations/` — one-off data migrations.

## Checks

The repo is linted and formatted by codelaw (`codelaw.toml`; the project's own
rules are in `.codelaw/rules/`). CI runs `codelaw check`.
