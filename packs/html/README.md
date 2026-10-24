# @codelaw/rules-html · codelaw-rules-html

codelaw's rules for HTML — one of the first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). The same files ship on npm
as `@codelaw/rules-html` and on PyPI as `codelaw-rules-html`.

```toml
# codelaw.toml
extends = ["codelaw:html/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

## Presets

| preset | rules | ported from |
|---|---|---|
| `html/recommended` | 64 | html-validate 11.16.4 `html-validate:recommended`, without its layout rules and without the cosmetic rules its `prettier` and `browser` presets switch off (see below) |
| `html/a11y` | 26 | html-validate 11.16.4 `html-validate:a11y`: every rule it turns on |
| `html/document` | 5 | html-validate 11.16.4 `html-validate:document`: checks that only make sense for a complete page |
| `html/conventions` | 3 | opt-in: the starter rule `html/target-blank-rel`, and HTMLHint's `inline-style-disabled` and `inline-script-disabled` |

`html/a11y` is a subset of `html/recommended`, as upstream; turn it on alone
for an accessibility-only check. `html/document` adds to `html/recommended`.

Every rule is **reimplemented**, not vendored: the checks, messages and docs are
written fresh, and each rule's `ported_from:` names the source rule. Every rule
is a `native:` rule, sharing one model of the document
(`crates/codelaw-native/src/html_dom.rs`): tree-sitter-html knows only a few of
HTML's implied end tags, so the model rebuilds the element tree the way a
browser (and html-validate) does, from the element table in `data/`.

Fix classes: 10 `auto`, 11 `suggest`, 51 `manual`.

## Rules

| rule | fix | ported from | presets |
|---|---|---|---|
| `html/area-alt` | manual | html-validate/area-alt | recommended, a11y |
| `html/aria-hidden-body` | suggest | html-validate/aria-hidden-body | recommended, a11y |
| `html/aria-label-misuse` | manual | html-validate/aria-label-misuse | recommended, a11y |
| `html/attribute-allowed-values` | manual | html-validate/attribute-allowed-values, htmlhint/src-not-empty | recommended |
| `html/attribute-misuse` | manual | html-validate/attribute-misuse | recommended |
| `html/autocomplete-password` | manual | html-validate/autocomplete-password | recommended |
| `html/button-type` | suggest | html-validate/no-implicit-button-type, htmlhint/button-type-require | recommended, a11y |
| `html/close-attr` | auto | html-validate/close-attr | recommended |
| `html/close-order` | manual | html-validate/close-order, htmlhint/tag-pair | recommended |
| `html/deprecated` | manual | html-validate/deprecated, htmlhint/tag-no-obsolete | recommended |
| `html/doctype-html` | suggest | html-validate/doctype-html, htmlhint/doctype-html5 | recommended |
| `html/element-name` | manual | html-validate/element-name | recommended |
| `html/element-permitted-content` | manual | html-validate/element-permitted-content | recommended |
| `html/element-permitted-occurrences` | manual | html-validate/element-permitted-occurrences | recommended |
| `html/element-permitted-order` | manual | html-validate/element-permitted-order | recommended |
| `html/element-permitted-parent` | manual | html-validate/element-permitted-parent | recommended |
| `html/element-required-ancestor` | manual | html-validate/element-required-ancestor | recommended |
| `html/element-required-attributes` | manual | html-validate/element-required-attributes, htmlhint/html-lang-require | recommended |
| `html/element-required-content` | manual | html-validate/element-required-content, htmlhint/title-require | recommended |
| `html/empty-heading` | manual | html-validate/empty-heading | recommended, a11y |
| `html/empty-title` | manual | html-validate/empty-title | recommended, a11y |
| `html/fieldset-legend` | manual | html-validate/wcag/h71 | recommended, a11y |
| `html/form-dup-name` | manual | html-validate/form-dup-name | recommended |
| `html/form-submit` | manual | html-validate/wcag/h32 | recommended, a11y |
| `html/hidden-focusable` | manual | html-validate/hidden-focusable | recommended, a11y |
| `html/img-alt` | manual | html-validate/wcag/h37, htmlhint/alt-require | recommended, a11y |
| `html/img-title` | manual | html-validate/wcag/h67 | recommended, a11y |
| `html/input-attributes` | suggest | html-validate/input-attributes | recommended |
| `html/input-image-alt` | manual | html-validate/wcag/h36 | recommended, a11y |
| `html/link-text` | manual | html-validate/wcag/h30 | recommended, a11y |
| `html/long-title` | manual | html-validate/long-title | recommended |
| `html/map-dup-name` | manual | html-validate/map-dup-name | recommended |
| `html/map-id-name` | manual | html-validate/map-id-name | recommended |
| `html/meta-refresh` | manual | html-validate/meta-refresh | recommended, a11y |
| `html/multiple-labeled-controls` | manual | html-validate/multiple-labeled-controls | recommended, a11y |
| `html/no-abstract-role` | manual | html-validate/no-abstract-role | recommended, a11y |
| `html/no-autoplay` | suggest | html-validate/no-autoplay | recommended, a11y |
| `html/no-conditional-comment` | manual | html-validate/no-conditional-comment | recommended |
| `html/no-deprecated-attr` | manual | html-validate/no-deprecated-attr | recommended |
| `html/no-dup-attr` | auto | html-validate/no-dup-attr, htmlhint/attr-no-duplication | recommended |
| `html/no-dup-class` | auto | html-validate/no-dup-class | recommended |
| `html/no-duplicate-id` | manual | html-validate/no-dup-id, htmlhint/id-unique | recommended, a11y |
| `html/no-implicit-close` | suggest | html-validate/no-implicit-close | recommended |
| `html/no-implicit-input-type` | suggest | html-validate/no-implicit-input-type | recommended |
| `html/no-multiple-main` | manual | html-validate/no-multiple-main | recommended |
| `html/no-utf8-bom` | manual | html-validate/no-utf8-bom | recommended |
| `html/no-raw-characters` | auto | html-validate/no-raw-characters, htmlhint/spec-char-escape | recommended |
| `html/no-redundant-aria-label` | auto | html-validate/no-redundant-aria-label | recommended, a11y |
| `html/no-redundant-for` | auto | html-validate/no-redundant-for | recommended, a11y |
| `html/no-redundant-role` | auto | html-validate/no-redundant-role | recommended, a11y |
| `html/prefer-button` | manual | html-validate/prefer-button | recommended |
| `html/prefer-native-element` | manual | html-validate/prefer-native-element | recommended, a11y |
| `html/prefer-tbody` | auto | html-validate/prefer-tbody | recommended |
| `html/script-element` | suggest | html-validate/script-element | recommended |
| `html/script-type` | auto | html-validate/script-type | recommended |
| `html/tel-non-breaking` | suggest | html-validate/tel-non-breaking | recommended |
| `html/text-content` | manual | html-validate/text-content | recommended, a11y |
| `html/th-scope` | manual | html-validate/wcag/h63 | recommended, a11y |
| `html/unique-landmark` | manual | html-validate/unique-landmark | recommended, a11y |
| `html/unrecognized-char-ref` | manual | html-validate/unrecognized-char-ref | recommended |
| `html/valid-autocomplete` | manual | html-validate/valid-autocomplete | recommended, a11y |
| `html/valid-for` | manual | html-validate/valid-for | recommended |
| `html/valid-id` | manual | html-validate/valid-id | recommended |
| `html/void-content` | suggest | html-validate/void-content | recommended |
| `html/heading-level` | manual | html-validate/heading-level | document |
| `html/input-missing-label` | manual | html-validate/input-missing-label, htmlhint/input-requires-label | document |
| `html/missing-doctype` | suggest | html-validate/missing-doctype | document |
| `html/no-missing-references` | manual | html-validate/no-missing-references | document |
| `html/require-sri` | manual | html-validate/require-sri | document |
| `html/target-blank-rel` | auto | — (starter rule) | conventions |
| `html/no-inline-style` | manual | html-validate/no-inline-style, htmlhint/inline-style-disabled | conventions |
| `html/no-inline-script` | manual | htmlhint/inline-script-disabled | conventions |

The four starter-catalogue rules ([SR-RULE-PACKS](../../records/0101_rule-packs.md)
decision 8): `html/img-alt` (↔ `wcag/h37`), `html/no-duplicate-id` (↔ `no-dup-id`)
and `html/button-type` (↔ `no-implicit-button-type`) match html-validate rules
and sit in `html/recommended`; `html/target-blank-rel` matches none and forms
`html/conventions`. The other ported rules keep html-validate's names, except
its WCAG technique rules, which are named for what they check
(`wcag/h30` → `html/link-text`, `h32` → `form-submit`, `h36` → `input-image-alt`,
`h63` → `th-scope`, `h67` → `img-title`, `h71` → `fieldset-legend`).

## Not ported, and why

| upstream rule | why |
|---|---|
| html-validate `attr-case`, `element-case` | never: layout — case-only style belongs to the formatter |
| html-validate `attr-delimiter`, `attr-spacing`, `no-trailing-whitespace` | never: layout — whitespace belongs to the formatter |
| html-validate `attr-quotes`, `doctype-style` | never: layout — quoting and the doctype's case belong to the formatter (also switched off by its `prettier` preset) |
| html-validate `attribute-boolean-style`, `attribute-empty-style`, `void-style`, `no-self-closing` | never: cosmetic — how `disabled`, `alt=""` and void tags are spelled is the formatter's call (switched off by its `browser` and `prettier` presets) |
| html-validate `deprecated-rule`, `no-unused-disable` | about html-validate's own configuration and inline directives; codelaw reports its own |
| html-validate `svg-focusable` | switched off in every upstream preset (it only matters for Internet Explorer) |
| html-validate `no-inline-style` in `recommended` | moved to the opt-in `html/conventions`: its `browser` preset switches it off, and it is a style policy, not a correctness check |
| HTMLHint `attr-lowercase`, `tagname-lowercase`, `attr-value-double-quotes`, `attr-value-single-quotes`, `attr-whitespace`, `attr-no-unnecessary-whitespace`, `space-tab-mixed-disabled`, `tag-self-close`, `empty-tag-not-self-closed`, `attr-sorted` | never: layout |
| HTMLHint `head-script-disabled`, `script-disabled`, `style-disabled`, `id-class-ad-disabled`, `id-class-value`, `href-abs-or-rel`, `attr-unsafe-chars`, `attr-value-not-empty`, `attr-value-no-duplication`, `tagname-specialchars`, `tags-check` | house policy without a correctness reason, or covered by a ported rule (`tags-check` by the content-model rules, `tagname-specialchars` by `html/element-name`) |
| HTMLHint `h1-require`, `main-require`, `meta-charset-require`, `meta-description-require`, `meta-viewport-require`, `link-rel-canonical-require`, `frame-title-require`, `form-method-require` | page-level SEO or policy checks html-validate's presets don't have; candidates for a later `html/strict` |

## How the ports differ

| rule | difference |
|---|---|
| all | the document model is rebuilt from tree-sitter-html's tokens with html-validate's implied-end-tag and implied-element rules; markup tree-sitter can't parse at all (a raw `<` in text) is treated as text |
| all | attribute values holding template code (`{{ }}`, `<% %>`, `${ }`, `{% %}`) are treated as unknown and not checked, as html-validate does for its framework transforms |
| `html/element-permitted-content` | obsolete elements are left to `html/deprecated` instead of being reported again wherever they appear |
| `html/close-order` | reports unclosed elements and stray end tags; html-validate also reports the end tag that closed past them, which repeats the first finding |
| `html/missing-doctype` | only files with an `<html>` element are checked; fragments and partials are not documents |
| `html/no-implicit-close` | the `suggest` fix writes the end tag after the element's last content |
| `html/prefer-tbody` | the fix wraps the rows when they are next to each other; otherwise the rule only reports |
| `html/heading-level` | sectioning roots are matched by tag or `[attr="value"]` selectors only |
| `html/require-sri` | no `include`/`exclude` URL options yet |
| `html/no-utf8-bom` | `manual`: the mark is in the file's bytes, outside the text a fix edits; only the file itself is checked, not an HTML region of another file |

## Data

| table | source |
|---|---|
| `data/html/elements.json` | html-validate 11.16.4's `html5` element metadata (MIT): content categories, content models, permitted parents and order, required and deprecated attributes, attribute value enumerations, implied end tags, ARIA implicit roles. Its JavaScript callbacks are rewritten as JSON conditions (`{"attr": "href"}`, `{"ancestor": ["map"]}`, `{"if": …, "then": …, "else": …}`) that `html_dom.rs` evaluates. |
| `data/html/entities.json` | the named character references html-validate 11.16.4 ships (MIT), following the WHATWG table |
| `data/html/input-attributes.json` | which `<input>` types each attribute applies to, from html-validate 11.16.4's `input-attributes` rule (MIT) |

See `NOTICE`.

## Upstream versions

- html-validate **11.16.4** (MIT) — presets `recommended`, `a11y`, `document`, `prettier`, `browser`; rule docs at https://html-validate.org/rules/.
- HTMLHint **1.9.2** (MIT) — cross-checked for rules html-validate lacks.
