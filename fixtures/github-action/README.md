# GitHub Action fixture

The project CI's `action` job runs the repository's own Action (`uses: ./`, [W-97](../../work/open/W-97-github-action.md)) on. `src/app.js` has one `debugger`; the local rule `.codelaw/rules/fixture/no-debugger.md` makes it an error, so no rule pack is needed. The job expects exit code `1`, one `::error` annotation at `fixtures/github-action/src/app.js` (the Action rewrites the path to the repository root) and a SARIF file whose one result has that path.
