# The editor's look — design of record

How the VS Code extension shows codelaw's findings, ruled by Arpit on
2026-10-30 (Cowork) and built by
[W-137](../../../archive/open/W-137-editor-findings-look.md). **Build what this
folder shows, and nothing it does not.** Where this README and an image
disagree, this README wins; ask before resolving anything else.

The working canvas, with the options that were not chosen, is
<https://claude.ai/artifact/Ufrc5yN8MKfGqpAwaBi7qy> (private to Arpit).

## The rule in one table

Three categories, each with its own setting. Every category supports both
views; only the defaults differ.

| setting | what it covers | default | other value |
|---|---|---|---|
| `codelaw.view.code` | findings in JS, TS and Python code | **`squiggles`** | `badges` |
| `codelaw.view.manifests` | `package.json`, `pyproject.toml`, their lockfiles, `.nvmrc`, `.python-version`, `.tool-versions` | **`badges`** | `squiggles` |
| `codelaw.view.markers` | TODO, FIXME, HACK and SKIP comments, and skipped-test calls without a reason | **`badges`** | `squiggles` |

Also: `codelaw.sidebar` (default on) and `codelaw.badges.clean` (show
"latest" on clean dependencies, default on).

## The boards

| image | what it fixes |
|---|---|
| [D · manifests, badges](png/D-manifests-badges.png) | `codelaw.view.manifests = badges`, the sidebar, the status bar, coloured tab names, the hover card on a dependency |
| [H · code, squiggles](png/H-code-squiggles-default.png) | `codelaw.view.code = squiggles`: only a wavy underline; the card appears **only while the pointer is over it** |
| [J · every code use case](png/J-code-every-use-case.png) | which colour each code finding gets |
| [K · hover cards](png/K-hover-cards.png) | the card for each of twelve use cases — every one is drawn open only so they can be compared |
| [L · the defaults together](png/L-defaults-together.png) | code squiggles beside TODO/FIXME/SKIP badges in the same files |
| [E · code, badges](png/E-code-badges-view.png) | `codelaw.view.code = badges` only — **not** the default |

`src/` holds each board as one standalone HTML file — open it in a browser
for the exact colours, sizes and spacing.

## Squiggle colours

| colour | meaning |
|---|---|
| red `#f14c4c` | error |
| yellow `#cca700` | warning |
| blue `#3794ff` | info |
| green `#73c991` | fixes itself on save — nothing for the user to do |
| dotted grey `#8a8a8a` | an old finding held in the baseline |

These are VS Code Dark+'s values. In the extension, every colour is a
`contributes.colors` entry so light and high-contrast themes get their own;
never hard-code a hex.

## The hover card

The same anatomy in every case (board K):

1. icon, a plain-language title, a severity chip, a fix chip (`one-click fix`,
   `needs your call`, `fixes itself on save`, `baseline`);
2. one sentence on why it matters;
3. an example — Don't/Do, or the fix as before → after, or a few key/value
   rows (ticket, due date, owner, safe version);
4. a meta line: the rule id, then CWE, `new in this branch` or the baseline
   date, the snapshot day;
5. a footer with **one** primary button, then plain links (*Rule page*,
   *Ignore with expiry…*).

## Not to build

- **A, B, C** (the first three manifest directions), **F** (inline fix
  diffs with Accept/Reject) and **G** (thin bars and a peek panel): options
  shown to Arpit and not chosen.
- **The pop-up notification** ("2 high vulnerabilities… Fix both"): not chosen; ask before adding.
- **Gutter icons, row tints, badges or CodeLens on ordinary code** while
  `codelaw.view.code` is `squiggles`.

## Notes for the builder

- The images use Inter and JetBrains Mono (ligatures off). The extension uses
  the user's editor font; nothing here asks for a font.
- Version numbers, advisory counts and dates are illustrative.
- Rule ids not yet built (`js/todo`, `js/flag-expiry`, `js/no-focused-test`,
  `js/skip-needs-reason`, `js/complexity`, `deps/vulnerable`) take their final
  names from W-128, W-130, W-131 and W-132.
