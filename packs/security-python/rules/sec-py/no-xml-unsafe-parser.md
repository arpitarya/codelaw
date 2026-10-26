---
id: sec-py/no-xml-unsafe-parser
title: 'XML is parsed with `defusedxml`'
languages: [python]
fix: suggest
fix_reason: "`defusedxml` has to be installed and imported, and it refuses documents the standard parser accepts"
stability: preview
ported_from: [bandit/B313, bandit/B314, bandit/B315, bandit/B316, bandit/B317, bandit/B318, bandit/B319, bandit/B320, bandit/B405, bandit/B406, bandit/B407, bandit/B408, bandit/B409, bandit/B410]
security: { cwe: [611], owasp: ["A05:2025"], severity: high }
since: 0.2.0
---

XML from outside the program is parsed with `defusedxml`, not with `xml.etree`, `minidom`, `sax` or `lxml`.

**Why:** The standard parsers expand entities, so a crafted document can read local files, reach the network or exhaust memory (CWE-611).

```codelaw
match:
  any:
    - all:
        - { pattern: '$M.$F($$$A)' }
        - has:
            field: function
            has:
              field: object
              regex: '^(ET|ElementTree|cElementTree|xml\.etree\.(c)?ElementTree)$'
              pattern: $ET
    - { pattern: '$M.$F($$$A)' }
constraints:
  F:
    regex: '^(parse|parseString|iterparse|fromstring|fromstringlist|XML|XMLParser|make_parser)$'
where:
  $M:
    any:
      - imported-from:
          - xml.etree.ElementTree
          - xml.etree.cElementTree
          - xml.dom.minidom
          - xml.dom.pulldom
          - xml.sax
          - xml.dom.expatbuilder
          - lxml.etree
      - { imported-from: { module: xml.etree, name: ElementTree } }
      - { imported-from: { module: xml.etree, name: cElementTree } }
      - { imported-from: { module: xml.dom, name: minidom } }
      - { imported-from: { module: xml.dom, name: pulldom } }
      - { imported-from: { module: xml, name: sax } }
      - { imported-from: { module: lxml, name: etree } }
message: 'This parser expands XML entities; use `defusedxml`'
fix:
  - { replace: { target: $ET, with: defusedxml.ElementTree } }
```

## How to fix

1. Install `defusedxml` and import it: `import defusedxml.ElementTree as ET`.
2. Call the same function on it: `ET.fromstring(text)`.
3. For `minidom`, `sax` or `pulldom`, import the module of the same name from `defusedxml`; for `lxml`, build the parser with `resolve_entities=False, no_network=True`.

## Examples

### Python

#### Don't

```py
import xml.etree.ElementTree as ET

tree = ET.parse(path)
```

#### Do

```py
import defusedxml.ElementTree as ET

tree = ET.parse(path)
```

#### Fixed

```py
import xml.etree.ElementTree as ET

tree = defusedxml.ElementTree.parse(path)
```
