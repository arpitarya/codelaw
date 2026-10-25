---
type: Handoff
name: W-57
description: "Arpit, 2026-10-25: the security-python pack: Bandit's checks ported (Apache-2.0), plus Django, Flask and FastAPI settings and sinks and ML model-loading risks, mapped to CWE and OWASP 2025."
item: W-57
filed: 2026-10-25
filed_by: "Arpit (Cowork, Opus 5.5)"
updated: 2026-10-25
updated_by: "Arpit (Cowork, Opus 5.5)"
ball: agent
---

# W-57 — `security-python` — Python security rules (Bandit's set, Django, Flask, FastAPI, ML loaders)

## At a glance

| step | what we do | what it gives us | state |
|---|---|---|---|
| 1 | Port Bandit's B1xx–B7xx checks (Apache-2.0, `ported_from: bandit Bnnn`) and write the framework rules below, with cases | the catalogue | ⏳ |
| 2 | Framework sources for `from_source:`: Django `request.GET/POST`, Flask `request.args/form/json`, FastAPI parameters | precise rules | ⏳ |
| 3 | `recommended` and `strict` presets; README with the not-ported table against Bandit and Ruff `S` | adoption | ⏳ |
| 4 | Corpus on three real Django/Flask/FastAPI repos | trustworthy defaults | ⏳ |

**Arpit, 2026-10-25:** *"I want to create a pack for security issues. Security issues like Veracode, Black Duck, X-Ray, OWASP. They would have some suggestions on how to write, what to write, what not to write in JavaScript and Python. So create … one for JavaScript, one for TypeScript, one for HTML, one for Python. Research and create work items for individual languages."*

**Model:** Sonnet — a written catalogue against Bandit's documented checks.

## Goal

| rule | flags | CWE | OWASP 2025 | fix |
|---|---|---|---|---|
| `sec-py/no-exec-eval` | `exec`, `eval`, `compile` of a non-literal (B102, B307) | 94 | A05 | manual |
| `sec-py/no-shell` | `subprocess.*(shell=True)`, `os.system`, `os.popen` with a non-literal (B602–B605) | 78 | A05 | suggest: list args, `shell=False` |
| `sec-py/no-sql-string` | `cursor.execute(f"…")`, `%`, `.format`, `+` (B608); Django `.extra`, `RawSQL` (B610, B611) | 89 | A05 | manual: parameters |
| `sec-py/no-unsafe-deserialize` | `pickle`, `marshal`, `shelve`, `dill`, `jsonpickle` loads (B301, B302, B403) | 502 | A08 | manual |
| `sec-py/yaml-safe-load` | `yaml.load` without `SafeLoader` (B506) | 502 | A08 | suggest: `yaml.safe_load` |
| `sec-py/torch-load-weights-only` | `torch.load` without `weights_only=True` (B614) | 502 | A08 | suggest |
| `sec-py/hf-pin-revision` | `from_pretrained`/`hf_hub_download` without `revision=` (B615) | 494 | A03 | manual |
| `sec-py/no-xml-unsafe-parser` | `xml.etree`, `minidom`, `sax`, `lxml` parse (B313–B320, B405–B411) | 611 | A05 | suggest: `defusedxml` |
| `sec-py/no-tls-bypass` | `requests(…, verify=False)` (B501); `ssl._create_unverified_context` (B323); old TLS versions (B502–B504) | 295 | A04 | suggest: remove |
| `sec-py/requests-timeout` | `requests.*` without `timeout=` (B113) | 400 | A06 | suggest: `timeout=` |
| `sec-py/no-weak-hash` | `hashlib.md5`/`sha1` without `usedforsecurity=False` (B324) | 327 | A04 | suggest: `usedforsecurity=False` when not security |
| `sec-py/no-insecure-random` | `random.*` for a token/secret/password (B311) | 338 | A04 | suggest: `secrets` |
| `sec-py/no-hardcoded-secret` | password/key literals (B105–B107), cloud key patterns | 798 | A07 | manual |
| `sec-py/no-insecure-tempfile` | `tempfile.mktemp` (B306); hard-coded `/tmp` (B108) | 377 | A01 | suggest: `mkstemp` |
| `sec-py/tarfile-filter` | `extractall` without `filter="data"` (B202) | 22 | A01 | suggest |
| `sec-py/no-path-traversal` | `open`/`send_file`/`FileResponse` with a request source | 22 | A01 | manual |
| `sec-py/no-ssrf` | `requests`/`httpx`/`urllib` URL from a request source | 918 | A01 | manual |
| `sec-py/no-open-redirect` | `redirect(request.args.get("next"))` and Django twins | 601 | A01 | manual |
| `sec-py/jinja-autoescape` | `Environment(autoescape=False)` (B701); `render_template_string` with a request source (SSTI) | 79, 1336 | A05 | suggest: `autoescape=True` |
| `sec-py/no-mark-safe` | Django `mark_safe`, `SafeString`, `Markup` of a non-literal (B703, B704, B308) | 79 | A05 | manual |
| `sec-py/flask-debug` | `app.run(debug=True)` (B201); bind to `0.0.0.0` (B104) | 489, 605 | A02 | suggest |
| `sec-py/django-settings` | `DEBUG = True`, `ALLOWED_HOSTS = ["*"]`, literal `SECRET_KEY`, `SESSION_COOKIE_SECURE = False`, `CSRF_COOKIE_SECURE = False` | 489, 798, 614 | A02 | manual |
| `sec-py/no-csrf-exempt` | `@csrf_exempt` on a view | 352 | A01 | manual |
| `sec-py/fastapi-cors` | `CORSMiddleware(allow_origins=["*"], allow_credentials=True)` | 942 | A02 | manual |
| `sec-py/no-assert-for-auth` | `assert` used as an access check (B101; stripped by `-O`) | 703 | A10 | manual: raise |
| `sec-py/no-silent-except` | `except: pass` / `except Exception: pass` (B110, B112) | 703 | A10 | manual |
| `sec-py/paramiko-host-key` | `AutoAddPolicy` / `WarningPolicy` (B507) | 295 | A07 | manual |
| `sec-py/no-world-writable` | `os.chmod(…, 0o777)` and friends (B103) | 732 | A01 | suggest |
| `sec-py/no-secret-in-log` | logging a password/token/secret binding | 532 | A09 | manual |
| `sec-py/no-llm-output-to-sink` | an OpenAI/Anthropic/LangChain response reaching `exec`, `eval`, `subprocess`, SQL, `mark_safe` | 94 | A05 | manual |

### Added after the second research pass (2026-10-25) — 31 more

Sources: CodeQL's query list (read for ideas; MIT, nothing copied), Ruff's `S` rules and Bandit (Apache-2.0/MIT, portable with `ported_from:`), the OWASP cheat sheets. `preset` says where each rule starts: `recommended` (`error`) or `strict` (`warn`, heuristic).

| rule | flags | CWE | OWASP 2025 | fix | preset |
|---|---|---|---|---|---|
| `sec-py/no-ldap-injection` | `ldap3`/`python-ldap` search filter from an f-string or `%` | 90 | A05 | manual: escape filter | recommended |
| `sec-py/no-xpath-injection` | `lxml` `.xpath(f"…")` | 643 | A05 | suggest: XPath variables | recommended |
| `sec-py/no-nosql-injection` | `pymongo` `find(request.json)`, `$where` strings | 943 | A05 | manual | recommended |
| `sec-py/no-header-injection` | response header value from a request source (CRLF) | 113 | A05 | manual | recommended |
| `sec-py/no-log-injection` | logging a request source without stripping newlines | 117 | A09 | manual | strict |
| `sec-py/cookie-flags` | `set_cookie` without `secure`, `httponly`, `samesite`; `samesite=None` | 614, 1004 | A02 | suggest: add flags | recommended |
| `sec-py/no-insecure-protocol` | `telnetlib`, `ftplib` (S312, S321, S401, S402); SNMP v1/v2 (S508, S509) | 319 | A04 | manual: SSH/SFTP/SNMPv3 | recommended |
| `sec-py/urlopen-scheme` | `urllib.request.urlopen` of a non-literal (`file://` reads) (S310) | 22, 918 | A01 | manual | recommended |
| `sec-py/no-weak-cipher` | DES, 3DES, ARC4, Blowfish; ECB mode (S304, S305) | 327 | A04 | manual: AES-GCM | recommended |
| `sec-py/no-pycrypto` | `import Crypto` from unmaintained pycrypto (S413) | 1104 | A03 | manual: pycryptodome/cryptography | recommended |
| `sec-py/weak-key-size` | RSA/DSA < 2048, EC < 224 bits (S505) | 326 | A04 | suggest | recommended |
| `sec-py/password-hash-effort` | `hashlib.sha*` of a password binding; `pbkdf2_hmac` < 600 000 iterations; bcrypt rounds < 10 | 916 | A04 | manual: argon2/bcrypt | recommended |
| `sec-py/no-hardcoded-crypto-key` | `AES.new(b"…")`, `Fernet(b"…")`, `hmac.new(b"…")`, a literal IV/nonce | 321, 329 | A04 | manual | recommended |
| `sec-py/no-mako` | Mako templates (no autoescape by default) (S702) | 79 | A05 | manual | recommended |
| `sec-py/logging-listen` | `logging.config.listen()` (S612) | 94 | A08 | manual | recommended |
| `sec-py/no-wildcard-injection` | `os.system`/`subprocess` with `*` in `tar`/`chown`/`chmod`/`rsync` (S609) | 155 | A05 | manual | recommended |
| `sec-py/no-partial-path` | starting a process with a bare executable name (S607) | 426 | A08 | manual: absolute path | strict |
| `sec-py/no-xmlrpc` | `xmlrpc` client/server without `defusedxml.xmlrpc` (S411) | 776 | A05 | suggest | recommended |
| `sec-py/no-mass-assignment` | `Model.objects.create(**request.POST)`; ModelForm/DRF serializer `fields = "__all__"` | 915 | A01 | manual: list fields | recommended |
| `sec-py/no-reflected-xss` | `HttpResponse`/`make_response` of a request source, unescaped | 79 | A05 | manual: `escape()`/templates | recommended |
| `sec-py/flask-settings` | literal `SECRET_KEY`; `WTF_CSRF_ENABLED = False`; `SESSION_COOKIE_SECURE = False` | 798, 352, 614 | A02 | manual | recommended |
| `sec-py/no-dynamic-import` | `__import__`, `importlib.import_module`, `getattr(obj, …)` with a request source | 470 | A05 | manual: allow-list | recommended |
| `sec-py/upload-secure-filename` | `request.files[…].save(f.filename)` without `secure_filename` | 434, 22 | A01 | suggest: `secure_filename` | recommended |
| `sec-py/ml-unsafe-load` | `numpy.load(allow_pickle=True)`, `joblib.load`, `pandas.read_pickle`, Keras `safe_mode=False` | 502 | A08 | manual: safetensors | recommended |
| `sec-py/no-trust-remote-code` | `trust_remote_code=True` (transformers, datasets) | 94 | A03 | manual | recommended |
| `sec-py/no-prompt-injection` | a request source concatenated into an LLM system prompt | 1427 | A05 | manual: separate roles | recommended |
| `sec-py/no-bind-all-interfaces` | `socket.bind(("0.0.0.0", …))`, uvicorn/gunicorn `host="0.0.0.0"` in code (S104) | 605 | A02 | manual | strict |
| `sec-py/no-regex-injection` | `re.compile`/`re.search` pattern from a request source; nested quantifiers (ReDoS) | 730, 1333 | A06 | suggest: `re.escape` | recommended |
| `sec-py/no-format-string-injection` | `<request source>.format(obj)` (attribute access through format fields) | 134 | A05 | manual | recommended |
| `sec-py/no-cleartext-http` | `requests`/`httpx` to `http://` other than localhost | 319 | A04 | suggest: `https://` | recommended |
| `sec-py/no-stack-trace-response` | `traceback.format_exc()`/`str(e)` returned in a response | 209 | A10 | manual | recommended |

`sec-py/tarfile-filter` also covers `shutil.unpack_archive` of tar formats; `sec-py/no-tls-bypass` also covers `ssl.wrap_socket` without a version (S504).

## Definition of done

| DoD | what | state |
|---|---|---|
| 1 · | every rule has examples, cases, `security:` metadata and a fix | ⏳ |
| 2 · | `codelaw test packs/security-python` green | ⏳ |
| 3 · | README's table accounts for every Bandit check (ported / covered elsewhere / not ported and why) | ⏳ |
| 4 · | `recommended` under 5% false positives on the corpus | ⏳ |

## Blockers

waits on **W-53**.

## Record affected

SR-RULE-PACKS (the pack's row; PyPI only).

## Hazards

- **No copyleft copying (L2):** Bandit, eslint-plugin-security and Ruff's `S` rules (Apache-2.0 / MIT) may be ported with `ported_from:` and NOTICE; eslint-plugin-no-unsanitized (MPL-2.0), njsscan (LGPL) and Semgrep's rules (Semgrep Rules License) are **reference only** — read, never copied.
- **Pattern-level, not taint analysis:** every rule fires on a dangerous sink with a non-literal argument, or with a known request source in the same function (W-53's predicate). False positives go to `security-*/strict` at `warn`; `recommended` holds only high-confidence rules at `error`.
- Every rule carries `security:` metadata (CWE, OWASP, severity) from W-53; L4: every `manual` fix names the safe API in its steps.
- Bandit's `B` ids belong in `ported_from:` only — codelaw's ids stay `sec-py/*`.

## Reference

[CodeQL Python queries](https://codeql.github.com/codeql-query-help/python/) · [Ruff flake8-bandit rules](https://docs.astral.sh/ruff/rules/#flake8-bandit-s) · [OWASP Password Storage](https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html) · [Hugging Face security](https://huggingface.co/docs/hub/security) · [Bandit plugins](https://bandit.readthedocs.io/en/latest/plugins/index.html) · [OWASP Django cheat sheet](https://cheatsheetseries.owasp.org/cheatsheets/Django_Security_Cheat_Sheet.html) · [JFrog SAST rule list](https://docs.jfrog.com/security/docs/list-of-sast-rules)
