# `codelaw-rules-security-python`

codelaw's Python security rules (Bandit's set, Django, Flask, FastAPI, ML loaders) — one of codelaw's first-party rule packs
([SR-RULE-PACKS](../../records/0101_rule-packs.md)). It ships on PyPI only, as `codelaw-rules-security-python` (L6 decision 11).

```toml
# codelaw.toml
extends = ["codelaw:py/recommended", "codelaw:sec-py/recommended"]
preview = true   # every rule is `preview` until a corpus run promotes it
```

`codelaw init` does not add it: a security pack is chosen, not detected.

## Presets

<!-- docs-sync:presets -->

| preset | rules | ported from |
|---|---|---|
| `sec-py/recommended` | 57 | high-confidence rules, at `error` |
| `sec-py/strict` | 61 | `sec-py/recommended` and the heuristic rules, at `warn` |

<!-- /docs-sync:presets -->

`recommended` holds only high-confidence rules, at `error`; `strict` adds the
heuristic ones, at `warn`, so a team can start without noise.

## Rules

61 rules, each a `match:` rule (no Rust). Every rule carries `security: { cwe, owasp, severity }`
([SR-RULE-FORMAT](../../records/0112_rule-format.md) decision 2a), so

- its finding names the CWE (`sec-py/<rule> (CWE-79) …`);
- `codelaw rules --tag cwe:79` and `--tag owasp:A05` find it, and
  `codelaw rules --security-coverage` puts it in the OWASP Top 10:2025 matrix;
- SARIF carries its `security` and `external/cwe/cwe-<n>` tags and a
  `security-severity` that GitHub code scanning ranks by
  ([SR-CLI-MCP](../../records/0115_cli-and-mcp.md) decision 4).

| rule | what it flags | CWE | OWASP 2025 | fix | preset |
|---|---|---|---|---|---|
| `sec-py/no-exec-eval` | `exec`, `eval` and `compile` are not given a value that is not a string literal | 94 | A05 | manual | recommended |
| `sec-py/no-shell` | A subprocess is not started with `shell=True`, and `os.system` / `os.popen` are not given a value that is not a literal | 78 | A05 | suggest | recommended |
| `sec-py/no-sql-string` | A SQL statement is not built with an f-string, `%`, `+` or `.format`; the values go in as parameters | 89 | A05 | manual | recommended |
| `sec-py/no-unsafe-deserialize` | Data is not loaded with `pickle`, `cPickle`, `dill`, `marshal`, `shelve` or `jsonpickle` | 502 | A08 | manual | recommended |
| `sec-py/yaml-safe-load` | YAML is read with `yaml.safe_load`, or with `yaml.load` and `SafeLoader` | 502 | A08 | suggest | recommended |
| `sec-py/torch-load-weights-only` | `torch.load` is called with `weights_only=True` | 502 | A08 | suggest | recommended |
| `sec-py/hf-pin-revision` | `from_pretrained`, `hf_hub_download`, `snapshot_download` and `load_dataset` are called with `revision=` | 494 | A03 | manual | recommended |
| `sec-py/no-xml-unsafe-parser` | XML from outside the program is parsed with `defusedxml`, not with `xml.etree`, `minidom`, `sax` or `lxml` | 611 | A05 | suggest | recommended |
| `sec-py/no-tls-bypass` | Certificate verification is not switched off (`verify=False`, `_create_unverified_context`), and an SSL context is not pinned to an old protocol | 295 | A04 | suggest | recommended |
| `sec-py/requests-timeout` | `requests` and `httpx` calls pass `timeout=` | 400 | A06 | suggest | recommended |
| `sec-py/no-weak-hash` | `hashlib.md5` and `hashlib.sha1` pass `usedforsecurity=False`, or are replaced by SHA-256 | 327 | A04 | suggest | recommended |
| `sec-py/no-insecure-random` | A password, token, key, nonce or code is not made with the `random` module | 338 | A04 | suggest | recommended |
| `sec-py/no-hardcoded-secret` | A password, key or token is not a string literal in the code, and a cloud access key is not pasted into it | 798 | A07 | manual | recommended |
| `sec-py/no-insecure-tempfile` | Temporary files are made with `tempfile.mkstemp`, `NamedTemporaryFile` or `TemporaryDirectory`, not `mktemp`, and not at a fixed path under `/tmp` | 377 | A01 | suggest | recommended |
| `sec-py/tarfile-filter` | `TarFile.extractall` and `shutil.unpack_archive` pass `filter="data"` | 22 | A01 | suggest | recommended |
| `sec-py/no-path-traversal` | A value from the request is not passed to `open`, `send_file`, `send_from_directory` or `FileResponse` as a path | 22 | A01 | manual | recommended |
| `sec-py/no-ssrf` | A value from the request is not the URL given to `requests`, `httpx` or `urllib` | 918 | A01 | manual | recommended |
| `sec-py/no-open-redirect` | A value from the request is not passed to `redirect`, `HttpResponseRedirect` or `RedirectResponse` | 601 | A01 | manual | recommended |
| `sec-py/jinja-autoescape` | `Environment` is not created with `autoescape=False` or without `autoescape`, and `render_template_string` / `Template` / `from_string` are not given a template that is not a literal | 79, 1336 | A05 | suggest | recommended |
| `sec-py/no-mark-safe` | `mark_safe`, `SafeString`, `Markup` and `format_html` are not given a value that is not a literal | 79 | A05 | manual | recommended |
| `sec-py/flask-debug` | `app.run(debug=True)` is not written in code | 489 | A02 | suggest | recommended |
| `sec-py/django-settings` | In a settings module, `DEBUG`, `ALLOWED_HOSTS`, `SECRET_KEY` and the cookie flags are not left in their development values | 489, 798, 614 | A02 | manual | recommended |
| `sec-py/no-csrf-exempt` | A view is not decorated with `@csrf_exempt` | 352 | A01 | manual | recommended |
| `sec-py/fastapi-cors` | `CORSMiddleware` is not given `allow_origins=["*"]` together with `allow_credentials=True` | 942 | A02 | manual | recommended |
| `sec-py/no-assert-for-auth` | A permission, role or authentication test is not written as an `assert` | 703 | A10 | manual | recommended |
| `sec-py/no-silent-except` | An `except` that catches everything (bare, `Exception` or `BaseException`) does not just `pass` or `continue` | 703 | A10 | manual | recommended |
| `sec-py/paramiko-host-key` | `paramiko` is not told to accept unknown host keys with `AutoAddPolicy` or `WarningPolicy` | 295 | A07 | manual | recommended |
| `sec-py/no-world-writable` | `os.chmod` and `os.umask` do not give write access to everyone | 732 | A01 | suggest | recommended |
| `sec-py/no-secret-in-log` | A value named `password`, `token` or `secret` is not written to a log | 532 | A09 | manual | recommended |
| `sec-py/no-llm-output-to-sink` | The reply of an OpenAI, Anthropic or LangChain call does not reach `exec`, `eval`, a subprocess, SQL or `mark_safe` | 94 | A05 | manual | strict |
| `sec-py/no-ldap-injection` | An LDAP search filter is not made with an f-string, `%`, `+` or `.format` | 90 | A05 | manual | recommended |
| `sec-py/no-xpath-injection` | An XPath expression is not made with an f-string, `%`, `+` or `.format`; values go in as XPath variables | 643 | A05 | manual | recommended |
| `sec-py/no-nosql-injection` | A value from the request is not used whole as a `pymongo` filter, and a `$where` expression is not built from a request value | 943 | A05 | manual | recommended |
| `sec-py/no-header-injection` | A value from the request is not written into a response header or a cookie value without its line breaks removed | 113 | A05 | manual | recommended |
| `sec-py/no-log-injection` | A value from the request is not passed to a logger without its line breaks removed | 117 | A09 | manual | strict |
| `sec-py/cookie-flags` | `set_cookie` passes `secure`, `httponly` and `samesite`, and none of them is `False` or `None` | 614, 1004 | A02 | suggest | recommended |
| `sec-py/no-insecure-protocol` | `telnetlib`, `ftplib` and SNMP community strings (v1/v2c) are not used | 319 | A04 | manual | recommended |
| `sec-py/urlopen-scheme` | `urllib.request.urlopen` and `urlretrieve` are not given a value that is not an `http(s)` literal | 22, 918 | A01 | manual | recommended |
| `sec-py/no-weak-cipher` | A weak cipher (DES, 3DES, ARC2, ARC4, Blowfish, CAST5, IDEA) or ECB mode is not used | 327 | A04 | manual | recommended |
| `sec-py/no-pycrypto` | `Crypto` is not imported from the unmaintained `pycrypto` package | 1104 | A03 | manual | recommended |
| `sec-py/weak-key-size` | An RSA or DSA key is not generated shorter than 2048 bits, and an elliptic curve is at least 224 bits | 326 | A04 | suggest | recommended |
| `sec-py/password-hash-effort` | A password is not hashed with a plain `hashlib` digest, `pbkdf2_hmac` runs at least 600 000 iterations, and `bcrypt` uses at least 10 rounds | 916 | A04 | manual | recommended |
| `sec-py/no-hardcoded-crypto-key` | A cipher key, HMAC key, IV or nonce is not a literal | 321, 329 | A04 | manual | recommended |
| `sec-py/no-mako` | `mako.template.Template` is not used | 79 | A05 | manual | recommended |
| `sec-py/logging-listen` | `logging.config.listen()` is not called | 94 | A08 | manual | recommended |
| `sec-py/no-wildcard-injection` | A command that runs `tar`, `chown`, `chmod` or `rsync` on `*` is not started through a shell | 155 | A05 | manual | recommended |
| `sec-py/no-partial-path` | A subprocess is not started with a bare program name such as `"git"` | 426 | A08 | manual | strict |
| `sec-py/no-xmlrpc` | An `xmlrpc` client or server is not created without `defusedxml.xmlrpc.monkey_patch()` in the program | 776 | A05 | manual | recommended |
| `sec-py/no-mass-assignment` | `Model(**request.POST)`, `Model.objects.create(**request.data)` and `update(**request.json)` are not used, and a `ModelForm` or serializer does not set `fields = "__all__"` | 915 | A01 | manual | recommended |
| `sec-py/no-reflected-xss` | A value from the request is not passed to `HttpResponse`, `make_response` or `HTMLResponse`, or built into a returned string, without escaping | 79 | A05 | manual | recommended |
| `sec-py/flask-settings` | `SECRET_KEY` is not a literal, and `WTF_CSRF_ENABLED` and `SESSION_COOKIE_SECURE` are not switched off in the app configuration | 798, 352, 614 | A02 | manual | recommended |
| `sec-py/no-dynamic-import` | A value from the request is not passed to `__import__`, `importlib.import_module` or `getattr` | 470 | A05 | manual | recommended |
| `sec-py/upload-secure-filename` | A file from a request is not saved under the name the client gave it | 434, 22 | A01 | suggest | recommended |
| `sec-py/ml-unsafe-load` | `numpy.load(allow_pickle=True)`, `joblib.load`, `pandas.read_pickle` and Keras `load_model(safe_mode=False)` are not used | 502 | A08 | manual | recommended |
| `sec-py/no-trust-remote-code` | `from_pretrained`, `load_dataset` and `pipeline` are not called with `trust_remote_code=True` | 94 | A03 | manual | recommended |
| `sec-py/no-prompt-injection` | A value from the request is not part of the system prompt of a language-model call | 1427 | A05 | manual | recommended |
| `sec-py/no-bind-all-interfaces` | A socket or server is not bound to `0.0.0.0` in the source | 605 | A02 | manual | strict |
| `sec-py/no-regex-injection` | A value from the request is not used as the pattern of `re.compile`, `re.search`, `re.match` or the other `re` functions | 730, 1333 | A06 | suggest | recommended |
| `sec-py/no-format-string-injection` | A value from the request is not the receiver of `.format` | 134 | A05 | manual | recommended |
| `sec-py/no-cleartext-http` | `requests` and `httpx` are not called with an `http://` URL, except for localhost | 319 | A04 | manual | recommended |
| `sec-py/no-stack-trace-response` | A response does not contain `traceback.format_exc()` or the text of the exception | 209 | A10 | manual | recommended |

A rule that says `suggest` has a `fix:` that `codelaw fix --unsafe` applies to
the common shape of the code; a match of another shape stays `manual`, with
the steps in the rule's *How to fix*. Three rules the catalogue called
`suggest` are `manual` here, because no edit of the code alone is right:
`sec-py/no-xpath-injection` (the expression has to be rewritten around
variables), `sec-py/no-xmlrpc` (the patch must run at start-up) and
`sec-py/no-cleartext-http` (the server must serve HTTPS first).

## What the rules see

The rules match the code **as written**. They follow an alias only where a
rule says so (`sec-py/no-xml-unsafe-parser` and `sec-py/no-mako` use the
`imported-from` predicate); `import numpy as np` is matched by its common
name `np`, and `from subprocess import run` is not matched at all. A value
that a function receives as a parameter is not a request value.

### `from_source:` is not taint analysis

A rule here that asks whether a sink's argument comes from a request
uses the `from_source:` predicate ([SR-RULE-FORMAT](../../records/0112_rule-format.md)
decision 3a). It holds when the argument **is** a listed request source
(or a method called on one), is built from one by concatenation,
interpolation or `?:`, or is a local name bound to one **in the same
function** — one or more assignments or destructurings away. It never
follows a call, a return value, a parameter, a closure's outer variable or
another file, so a value that crosses a function boundary is not seen
(cross-function flow waits on a deeper analysis, as B-049's type-aware
rules do). Every rule page that uses it says so. The sources are listed per
framework in this pack's `data/sources/<framework>.json`: Django
(`request.GET`, `request.POST`, `request.COOKIES`, `request.body`,
`request.FILES`, `request.META`, `request.headers`, DRF's `request.data` and
`request.query_params`), Flask (`request.args`, `form`, `values`, `json`,
`get_json`, `data`, `cookies`, `files`, `headers`, `view_args`) and
FastAPI/Starlette's `Request` (`query_params`, `path_params`, `headers`,
`cookies`, `form`, `json`, `body`).

Two limits follow. FastAPI's typed handler parameters (`def f(q: str)`) are
function parameters, so they are not sources; only the `request` object is.
And a method called on a source stays a source: `request.args.get("x").strip()`
is still the request value, so the way out of a finding is to pass the value
through a function of your own.

## Not ported, and why

This pack accounts for every check of
[Bandit](https://bandit.readthedocs.io/en/latest/plugins/index.html)
(Apache-2.0) and the Ruff `S` rules (MIT) that mirror it. The rules are
written for codelaw; `ported_from:` names the Bandit and Ruff ids they cover
([NOTICE](NOTICE)).

| Bandit | Ruff | status | where |
|---|---|---|---|
| B101 `assert_used` | S101 | partly | `sec-py/no-assert-for-auth`: only an `assert` that is an access check; an `assert` in general is not a security flaw |
| B102 `exec_used` | S102 | ported | `sec-py/no-exec-eval` |
| B103 `set_bad_file_permissions` | S103 | ported | `sec-py/no-world-writable` |
| B104 `hardcoded_bind_all_interfaces` | S104 | ported | `sec-py/no-bind-all-interfaces` (`strict`) |
| B105, B106, B107 hard-coded passwords | S105, S106, S107 | ported | `sec-py/no-hardcoded-secret`; Django and Flask settings in `sec-py/django-settings` and `sec-py/flask-settings` |
| B108 `hardcoded_tmp_directory` | S108 | ported | `sec-py/no-insecure-tempfile` |
| B110, B112 `try_except_pass`, `try_except_continue` | S110, S112 | ported | `sec-py/no-silent-except` |
| B113 `request_without_timeout` | S113 | ported | `sec-py/requests-timeout` |
| B201 `flask_debug_true` | S201 | ported | `sec-py/flask-debug` |
| B202 `tarfile_unsafe_members` | S202 | ported | `sec-py/tarfile-filter` |
| B301, B302 `pickle`, `marshal` | S301, S302 | ported | `sec-py/no-unsafe-deserialize` |
| B303 `md5`, `sha1` | S303 | ported | `sec-py/no-weak-hash` |
| B304, B305 ciphers and modes | S304, S305 | ported | `sec-py/no-weak-cipher` |
| B306 `mktemp_q` | S306 | ported | `sec-py/no-insecure-tempfile` |
| B307 `eval` | S307 | ported | `sec-py/no-exec-eval` |
| B308 `mark_safe` | S308 | ported | `sec-py/no-mark-safe` |
| B309 `httpsconnection` | - | not ported | Python 2.7 and early 3.x only: `http.client.HTTPSConnection` verifies certificates by default since 3.4.3 |
| B310 `urllib_urlopen` | S310 | ported | `sec-py/urlopen-scheme` |
| B311 `random` | S311 | ported | `sec-py/no-insecure-random`: only when the value is bound to a secret-like name; `random` for a dice roll is fine |
| B312 `telnetlib` | S312 | ported | `sec-py/no-insecure-protocol` (at the import) |
| B313 to B320 XML calls | S313 to S320 | ported | `sec-py/no-xml-unsafe-parser` |
| B321 `ftplib` | S321 | ported | `sec-py/no-insecure-protocol` (at the import) |
| B322 `input` | - | not ported | Python 2's `input` evaluates its input; Python 3's does not |
| B323 `unverified_context` | S323 | ported | `sec-py/no-tls-bypass` |
| B324 `hashlib` | S324 | ported | `sec-py/no-weak-hash` |
| B325 `tempnam` | S325 | ported | `sec-py/no-insecure-tempfile` (`os.tempnam`, `os.tmpnam`) |
| B401, B402 `telnetlib`, `ftplib` imports | S401, S402 | ported | `sec-py/no-insecure-protocol` |
| B403 `pickle` import | S403 | covered elsewhere | the load call is flagged by `sec-py/no-unsafe-deserialize`; importing `pickle` to write it is not a flaw |
| B404 `subprocess` import | S404 | not ported | an import alone flags every program that runs a command; the calls are `sec-py/no-shell` and `sec-py/no-partial-path` |
| B405 to B410 XML imports | S405 to S410 | covered elsewhere | the parse calls are flagged by `sec-py/no-xml-unsafe-parser`; the import of a parser is not |
| B411 `xmlrpc` import | S411 | ported | `sec-py/no-xmlrpc` (at the client or server call) |
| B412 `httpoxy` | S412 | not ported | CGI handlers (`wsgiref.handlers.CGIHandler`) are rare, and web servers now drop the `Proxy` header |
| B413 `pycrypto` | S413 | ported | `sec-py/no-pycrypto` |
| B414 `pycryptodome` | - | not ported | Bandit dropped it: `pycryptodome` is maintained |
| B415 `pyghmi` | S415 | not ported | plain-text IPMI, a niche library; `sec-py/no-insecure-protocol` covers the common ones |
| B501 to B504 TLS | S501 to S504 | ported | `sec-py/no-tls-bypass` |
| B505 `weak_cryptographic_key` | S505 | ported | `sec-py/weak-key-size` |
| B506 `yaml_load` | S506 | ported | `sec-py/yaml-safe-load` |
| B507 `ssh_no_host_key_verification` | S507 | ported | `sec-py/paramiko-host-key` |
| B508 `snmp_insecure_version` | S508 | ported | `sec-py/no-insecure-protocol` |
| B509 `snmp_weak_cryptography` | S509 | not ported | needs the `UsmUserData` protocol arguments checked; only the v1/v2c community string is flagged |
| B601 `paramiko_calls` | S601 | not ported | `exec_command` runs on the remote host and is flagged by Bandit for any call; no safe rewrite to name |
| B602, B604, B605 shell | S602, S604, S605 | ported | `sec-py/no-shell` |
| B603 `subprocess_without_shell_equals_true` | S603 | not ported | informational in Bandit and flags every `subprocess` call; the dangerous cases are `sec-py/no-shell` and `sec-py/no-wildcard-injection` |
| B606 `start_process_with_no_shell` | S606 | not ported | `os.exec*` and `os.spawn*` are rare; `sec-py/no-partial-path` covers `subprocess` and `os.system` |
| B607 `start_process_with_partial_path` | S607 | ported | `sec-py/no-partial-path` (`strict`) |
| B608 `hardcoded_sql_expressions` | S608 | ported | `sec-py/no-sql-string`: from the call that runs the statement, not from a string that looks like SQL |
| B609 `linux_commands_wildcard_injection` | S609 | ported | `sec-py/no-wildcard-injection` |
| B610, B611 Django `extra`, `RawSQL` | S610, S611 | ported | `sec-py/no-sql-string` |
| B612 `logging_config_insecure_listen` | S612 | ported | `sec-py/logging-listen` |
| B613 `trojansource` | - | not ported | bidirectional control characters in the source bytes; a rule reads the syntax tree and does not see them |
| B614 `pytorch_load` | - | ported | `sec-py/torch-load-weights-only` |
| B615 `huggingface_unsafe_download` | - | ported | `sec-py/hf-pin-revision` |
| B701 `jinja2_autoescape_false` | S701 | ported | `sec-py/jinja-autoescape` |
| B702 `use_of_mako_templates` | S702 | ported | `sec-py/no-mako` |
| B703 `django_mark_safe` | S703 | ported | `sec-py/no-mark-safe` |
| B704 `markupsafe_markup_xss` | S704 | ported | `sec-py/no-mark-safe` |

Beyond Bandit and Ruff the pack adds the framework and library rules of the
catalogue: Django, Flask and FastAPI settings and sinks, mass assignment,
uploads, header, LDAP, XPath and NoSQL injection, ML model loading
(`torch.load`, `joblib`, `numpy`, Hugging Face), `trust_remote_code`, prompt
injection and language-model output reaching a sink. Two parts of the catalogue are not
built: the `strict` ReDoS check for a literal pattern with nested quantifiers
(`sec-py/no-regex-injection` handles only a request-supplied pattern), and a
`from_source` check on every argument of a call (it reads the first, and for a
logger the second).

## Corpus

`packs/security-python/corpus.toml` holds the finding count per rule over the
pinned repositories (`scripts/corpus.json`): Flask, Starlette,
full-stack-fastapi-template and djangoproject.com, 946 files. On 2026-10-26
the rules gave **34 findings, all of them read by hand: one false positive**
(`sec-py/no-open-redirect` on a redirect to a trusted URL that had `?q=…`
appended from the request, which `from_source:` cannot tell from a hostile
target), about 3%. The other 33 are what the rules say: a `mark_safe` of a
non-literal, an MD5 or SHA-1 digest, an `exec` of a config file, a request
without a `timeout`, a `subprocess` started by a bare name (`strict`), and
the like. Two limits of that number: four repositories are a small sample,
and the noisy rules skip test files (`exclude:` in their frontmatter) because
the first run, before they did, was mostly asserts and `password="password"`
in tests. Every rule is `preview` until its count has been reviewed again
over a larger corpus.
