# acme-shop

A small shop: a TypeScript web client in `web/` and a Python API in `api/`. The
client talks only to the API; the API talks to the database and to three outside
services (rates, the ledger and the warehouse).

## Layout

- `web/src/` — the client. `web/src/lib/http.ts` is the HTTP client;
  `web/src/components/` holds the React components, one per file.
- `api/acme/` — the API package. `api/acme/http.py` is the outbound HTTP client,
  `api/acme/db.py` the database helpers, `api/acme/clock.py` the clock.
- `api/migrations/` — one-off data migrations, each a module with `run(conn)`.

## Build and run

- Client: `cd web && npm install && npm run dev` serves it on port 5173 and proxies
  `/api` to the API.
- API: `cd api && python -m acme` on port 8000; it reads `ACME_DB` for the database
  path and `ACME_ENV` (`dev`, `staging`, `prod`).
- Python 3.12 and Node 22. No other runtime is assumed.

## Working in this repo

- Keep a change to what the task asks; do not reformat files you did not touch.
- Small modules: one concern per file, exported names at the bottom of a TS file.
- New Python modules get a one-line docstring saying what the module is for.
- Commit messages are imperative and name the area: `web: …`, `api: …`.
