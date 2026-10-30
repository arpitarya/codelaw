"""W-138: match every upstream rule against codelaw and class each one.
Inputs (all in this folder): up.jsonl, ruff-rules.json, biome.json, tslint-map.json, codelaw.json, biome-sources.json.
Outputs: ../<tool>.csv, ../summary.json, ../ports.json. Deterministic: same inputs, same bytes."""
import json, csv, re, os, sys, collections

D = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.dirname(D)  # results go beside method/
J = lambda f: json.load(open(os.path.join(D, f)))

up = [json.loads(l) for l in open(os.path.join(D, 'up.jsonl')) if l.strip()]
REC = {k: set(v) for k, v in J('recommended.json').items()}
ruff = J('ruff-rules.json'); biome = J('biome.json'); tsmap = J('tslint-map.json')
cl = J('codelaw.json'); bsrc = J('biome-sources.json')['sources']
cl_ids = {r[1] for r in cl}

# ---------- coverage: (tool, upstream id) -> [codelaw ids] ----------
ruff_by_name = {(r['linter'], r['name']): r['code'] for r in ruff}
cov = collections.defaultdict(set)
for pack, cid, fix, stab, pf in cl:
    for p in pf:
        if p.startswith('@tanstack/eslint-plugin-query/'):
            cov[('@tanstack/eslint-plugin-query', p.split('/', 2)[2])].add(cid); continue
        tool, _, rid = p.partition('/')
        if tool == 'bandit' and re.fullmatch(r'B\d+', rid):
            cov[('ruff', 'S' + rid[1:])].add(cid)
        elif tool == 'pylint':
            code = ruff_by_name.get(('Pylint', rid))
            if code: cov[('ruff', code)].add(cid)
        else:
            cov[(tool, rid)].add(cid)

# Equivalents a codelaw rule implements without naming them in ported_from (checked by hand, W-138).
# Counted as covered; W-139 adds the missing ported_from line.
UNLINKED = {
    ('eslint-plugin-import', 'no-default-export'): 'js/no-default-export', ('eslint-plugin-import', 'exports-last'): 'js/exports-at-end',
    ('eslint-plugin-import', 'order'): 'js/import-groups', ('eslint', 'no-warning-comments'): 'js/todo-marker',
    ('eslint', 'no-restricted-imports'): 'js/use-instead', ('eslint', 'no-restricted-globals'): 'js/use-instead',
    ('eslint', 'no-restricted-properties'): 'js/use-instead', ('ruff', 'TID251'): 'py/use-instead',
    ('ruff', 'T201'): 'py/no-print', ('ruff', 'FIX002'): 'py/todo-marker', ('ruff', 'FIX001'): 'py/todo-marker',
    ('ruff', 'FIX003'): 'py/todo-marker', ('ruff', 'FIX004'): 'py/todo-marker',
    ('typescript-eslint', 'consistent-type-imports'): 'ts/type-only-imports', ('stylelint', 'declaration-no-important'): 'css/no-important',
    ('eslint-plugin-react', 'no-danger'): 'sec-js/no-dangerously-set-inner-html', ('stylelint-scss', 'double-slash-comment-empty'): 'scss/double-slash-comment-empty',
    ('biome', 'suspicious/noFocusedTests'): 'js/no-focused-tests', ('biome', 'suspicious/noSkippedTests'): 'js/skipped-test',
    ('eslint-plugin-unicorn', 'expiring-todo-comments'): 'js/todo-marker',
    ('eslint-plugin-security', 'detect-no-csrf-before-method-override'): 'sec-js/no-csrf-before-method-override',
}
def covered(tool, rid):
    c = sorted(cov.get((tool, rid), ()))
    if not c and (tool, rid) in UNLINKED and UNLINKED[(tool, rid)] in cl_ids: return [UNLINKED[(tool, rid)] + '*']
    return c

# ---------- the plan for each tool ----------
PACK = {  # tool -> (pack, namespace)
    'eslint': ('js', 'js'), 'typescript-eslint': ('ts', 'ts'), 'eslint-plugin-react': ('react', 'react'),
    'eslint-plugin-react-hooks': ('react', 'react'), 'eslint-plugin-jsx-a11y': ('react', 'react'),
    'eslint-plugin-import': ('js', 'js'), 'eslint-plugin-n': ('node', 'node'), 'eslint-plugin-unicorn': ('js', 'js'),
    'eslint-plugin-security': ('security-js', 'sec-js'), '@tanstack/eslint-plugin-query': ('tanstack-query', 'tanstack-query'),
    'stylelint': ('css', 'css'), 'stylelint-scss': ('scss', 'scss'), 'html-validate': ('html', 'html'),
    'markdownlint': ('data', 'md'), 'ruff': ('python', 'py'),
}
PLANNED = {  # already filed elsewhere
    ('eslint', 'complexity'): 'W-132', ('eslint', 'max-depth'): 'W-132', 
    ('ruff', 'C901'): 'W-132', ('ruff', 'PLR1702'): 'W-132',
    **{('ruff', c): 'W-131' for c in ['TD001', 'TD002', 'TD003', 'TD004', 'TD005', 'TD006', 'TD007']},
}
TASTE = {  # 'skip': taste with no defect behind it; still portable on demand (W-140)
    'eslint': {'no-ternary', 'no-plusplus', 'no-undefined', 'id-length', 'capitalized-comments', 'sort-keys', 'sort-vars',
               'no-inline-comments', 'line-comment-position', 'multiline-comment-style', 'func-style', 'one-var', 'vars-on-top',
               'init-declarations', 'no-continue', 'func-names', 'id-match', 'id-denylist'},
}
PLUMBING = {  # skip: about the source tool's own config, directives or runtime, not about the code
    ('html-validate', 'deprecated-rule'): "html-validate's own config", ('html-validate', 'no-unused-disable'): "html-validate's own disable comments (codelaw: codelaw/stale-ignore)",
    ('eslint-plugin-react', 'jsx-uses-react'): "marks React used for ESLint's no-unused-vars; codelaw's semantic model reads JSX", ('eslint-plugin-react', 'jsx-uses-vars'): "marks JSX names used for ESLint's no-unused-vars; codelaw's semantic model reads JSX",
    ('eslint-plugin-react', 'react-in-jsx-scope'): 'only for the legacy JSX runtime (before React 17)',
    ('eslint-plugin-react-hooks', 'config'): "validates the React Compiler's own config", ('eslint-plugin-react-hooks', 'gating'): "validates the React Compiler's gating config",
    ('ruff', 'E902'): 'a file Ruff cannot read: a tool error, not a rule',
}
HV_LAYOUT = {'attr-case', 'attr-delimiter', 'attr-quotes', 'attr-spacing', 'attribute-boolean-style', 'attribute-empty-style', 'doctype-style', 'element-case', 'no-self-closing', 'no-trailing-whitespace', 'void-style'}
NEEDS_DATA = {('eslint-plugin-n', n) for n in ['no-unsupported-features/es-builtins', 'no-unsupported-features/es-syntax',
                                                  'no-unsupported-features/node-builtins', 'no-deprecated-api']}
MD_LAYOUT = {'MD003', 'MD004', 'MD005', 'MD007', 'MD009', 'MD010', 'MD012', 'MD013', 'MD018', 'MD019', 'MD020', 'MD021',
             'MD022', 'MD023', 'MD027', 'MD029', 'MD030', 'MD031', 'MD032', 'MD035', 'MD037', 'MD038', 'MD039', 'MD046',
             'MD047', 'MD048', 'MD049', 'MD050', 'MD055', 'MD058', 'MD060'}
RUFF_OUT = {'Airflow', 'flake8-django', 'FastAPI', 'NumPy-specific rules', 'pandas-vet'}

def kebab(s): return re.sub(r'(?<!^)(?=[A-Z])', '-', s).lower()
taken = set(cl_ids)
COLLIDE = {}
def pid(ns, name, tag):
    base = f'{ns}/{name}'
    i = base if base not in taken else f'{base}-{tag}'
    if base in cl_ids: COLLIDE[i] = base
    taken.add(i); return i

rows = collections.defaultdict(list)
def emit(tool, version, rid, cls, *, deprecated=False, ufix='', rec=False, cids=(), pack='', newid='', nfix='', prio='', note=''):
    rows[tool].append(dict(tool=tool, version=version, upstream_id=rid, class_=cls, deprecated='yes' if deprecated else '',
                           upstream_fix=ufix, upstream_recommended='yes' if rec else '', codelaw=' '.join(cids),
                           proposed_pack=pack, proposed_id=newid, proposed_fix=nfix, priority=prio, note=note))
FIX = {'auto': 'auto', 'suggest': 'suggest', 'none': 'manual', '': 'manual'}

def plan(tool, version, rid, *, name=None, deprecated=False, ufix='none', rec=False, layout=False, types=False,
         out=False, extends='', preview=False, pack=None, note=''):
    c = covered(tool, rid)
    pk, ns = pack or PACK.get(tool, ('', ''))
    if c: return emit(tool, version, rid, 'covered', deprecated=deprecated, ufix=ufix, rec=rec, cids=c)
    if deprecated: return emit(tool, version, rid, 'retired', deprecated=True, ufix=ufix, rec=rec, note=note or 'deprecated upstream')
    if (tool, rid) in PLANNED: return emit(tool, version, rid, 'planned', ufix=ufix, rec=rec, note=f'filed as {PLANNED[(tool, rid)]}')
    if extends:
        b = covered('eslint', rid if extends == 'base' else extends)
        if b: return emit(tool, version, rid, 'covered', ufix=ufix, rec=rec, cids=b, note=f'through its base, eslint/{rid if extends == "base" else extends}')
        base = rid if extends == 'base' else extends
        if not types and any(r['tool'] == 'eslint' and r['id'] == base for r in up):
            return emit(tool, version, rid, 'same-as', ufix=ufix, rec=rec, note=f'extends eslint/{base}: counted there (codelaw js rules cover TypeScript)')
    if layout: return emit(tool, version, rid, 'layout', ufix=ufix, rec=rec, note='the formatter owns it (SR-FORMATTER decision 3)')
    if types: return emit(tool, version, rid, 'types', ufix=ufix, rec=rec, note='needs type information: W-105')
    if out: return emit(tool, version, rid, 'out-of-scope', ufix=ufix, rec=rec, note=note)
    if (tool, rid) in PLUMBING: return emit(tool, version, rid, 'skip', ufix=ufix, rec=rec, note=PLUMBING[(tool, rid)])
    if (tool, rid) in NEEDS_DATA: return emit(tool, version, rid, 'needs-data', ufix=ufix, rec=rec, note='needs runtime support data codelaw does not ship')
    if rid in TASTE.get(tool, ()): return emit(tool, version, rid, 'skip', ufix=ufix, rec=rec, note='taste, no defect behind it; on demand through W-140')
    prio = 'P1' if rec else ('P3' if preview else 'P2')
    nid = pid(ns, name or rid, tool.split('-')[-1])
    if nid in COLLIDE: note = (note + '; ' if note else '') + f'codelaw already has {COLLIDE[nid]}: check it is not the same rule'
    return emit(tool, version, rid, 'port', ufix=ufix, rec=rec, pack=pk, newid=nid, nfix=FIX[ufix], prio=prio, note=note)

# ---------- ESLint-shaped tools ----------
for r in up:
    t, v, rid = r['tool'], r['version'], r['id']
    if t in ('prettier', 'tslint'): continue
    kw = dict(deprecated=r.get('deprecated', False), ufix=r.get('fix', 'none'), rec=(rid in REC[t]) if t in REC else (t == 'markdownlint'))
    if t == '@stylistic/eslint-plugin':
        emit(t, v, rid, 'layout', **{k: kw[k] for k in ('ufix', 'rec')}, deprecated=kw['deprecated'], note='formatting; the formatter owns it'); continue
    if t == 'typescript-eslint':
        plan(t, v, rid, types=r.get('types', False), extends=r.get('extends', ''), **kw); continue
    if t == 'eslint-plugin-jsx-a11y':
        plan(t, v, rid, name='a11y-' + rid, **kw); continue
    if t == 'eslint-plugin-import':
        graph = rid in {'no-unresolved', 'named', 'default', 'namespace', 'no-cycle', 'no-unused-modules', 'export', 'no-deprecated',
                        'no-named-as-default', 'no-named-as-default-member', 'no-extraneous-dependencies', 'no-internal-modules',
                        'no-relative-packages', 'no-useless-path-segments', 'extensions', 'no-restricted-paths', 'no-self-import'}
        plan(t, v, rid, name='import-' + rid, note='needs the project graph (SR-PROJECT-GRAPH)' if graph else '', **kw); continue
    if t == 'eslint-plugin-import-x':
        if any(x['tool'] == 'eslint-plugin-import' and x['id'] == rid for x in up):
            emit(t, v, rid, 'same-as', ufix=kw['ufix'], rec=kw['rec'], deprecated=kw['deprecated'], note=f'fork of eslint-plugin-import/{rid}: counted there'); continue
        plan(t, v, rid, name='import-' + rid, pack=('js', 'js'), **kw); continue
    layout = r.get('kind') == 'layout'
    name = rid.replace('/', '-')
    if t == 'stylelint-scss':
        name = rid.split('/', 1)[1]
        layout = bool(re.search(r'(newline|space|empty-line|indentation|single-line|-line-before|-line-after)', name))
        c = covered(t, name)
        if c: emit(t, r['version'], rid, 'covered', deprecated=kw['deprecated'], ufix=kw['ufix'], rec=kw['rec'], cids=c); continue
    if t == 'markdownlint':
        layout = rid in MD_LAYOUT; name = (r.get('alias') or rid).split(' ')[0]
    if t == 'html-validate' and rid in HV_LAYOUT: layout = True
    if t == 'eslint-plugin-unicorn' and not name.startswith('unicorn'): name = name
    plan(t, v, rid, name=name, layout=layout, **kw)

# ---------- TSLint: retired; record the successor ----------
for r in up:
    if r['tool'] != 'tslint': continue
    succ = tsmap.get(r['id'], [])
    ids = []
    for s in succ:
        if s.startswith('@typescript-eslint/'): ids += covered('typescript-eslint', s.split('/', 1)[1])
        elif '/' in s and s.split('/')[0] in ('react', 'import', 'jsdoc', 'prefer-arrow', 'unicorn'):
            tool = {'react': 'eslint-plugin-react', 'import': 'eslint-plugin-import', 'unicorn': 'eslint-plugin-unicorn'}.get(s.split('/')[0], s.split('/')[0])
            ids += covered(tool, s.split('/', 1)[1])
        elif s and s != '?': ids += covered('eslint', s)
    note = ('successor: ' + ', '.join(succ)) if succ and succ != ['?'] else 'no ESLint successor'
    emit('tslint', r['version'], r['id'], 'retired', deprecated=True, ufix=r.get('fix', ''), cids=sorted(set(ids)),
         note=note + ('; successor covered' if ids else ''))

# ---------- Ruff ----------
for r in ruff:
    code, lin = r['code'] or ('(no code) ' + r['name']), r['linter'] or ''
    st = list(r['status'].keys())[0] if isinstance(r['status'], dict) else r['status']
    ufix = {'Always': 'auto', 'Sometimes': 'suggest'}.get(r.get('fix_availability'), 'none')
    layout = (lin == 'pycodestyle' and (code[:2] in ('E1', 'E2', 'E3', 'W1', 'W2', 'W3') or code in ('E501', 'E701', 'E702', 'E703'))) \
        or lin == 'flake8-quotes' or code in ('COM812', 'COM819', 'ISC001', 'ISC002')
    pack = ('security-python', 'sec-py') if lin == 'flake8-bandit' else None
    plan('ruff', '0.16.9', code, name=r['name'], deprecated=(st == 'Removed'), ufix=ufix, rec=(lin in ('Pyflakes',) or code[:2] in ('E4', 'E7', 'E9')),
         layout=layout, out=(lin in RUFF_OUT), preview=(st == 'Preview'), pack=pack,
         note=(f'{lin}' + (', preview upstream' if st == 'Preview' else '') + (', a framework with no codelaw pack' if lin in RUFF_OUT else '')))

# ---------- Biome ----------
EXTRACTED = {'@tanstack/eslint-plugin-query', 'stylelint-scss', 'eslint', 'typescript-eslint', 'eslint-plugin-react', 'eslint-plugin-react-hooks', 'eslint-plugin-jsx-a11y',
             'eslint-plugin-import', 'eslint-plugin-n', 'eslint-plugin-unicorn', 'eslint-plugin-security', 'stylelint'}
def src_tool(s):
    t, _, rid = s.partition('/')
    t = {'typescript-eslint': 'typescript-eslint'}.get(t, t)
    return t, rid
for r in biome:
    n, v = r['name'], r['version']
    langs = set(r['langs'].split()) - {'json', 'jsonc'} or set(r['langs'].split())
    srcs = bsrc.get(n, [])
    cids = sorted({c for s in srcs for c in covered(*src_tool(s))})
    if 'css' in langs: pk = ('css', 'css')
    elif 'html' in langs and not langs & {'js', 'jsx', 'ts', 'tsx'}: pk = ('html', 'html')
    elif r['group'] == 'a11y' or 'jsx' in langs or 'tsx' in langs: pk = ('react', 'react')
    elif langs <= {'json', 'jsonc'} and langs: pk = ('data', 'json')
    elif langs == {'ts'}: pk = ('ts', 'ts')
    else: pk = ('js', 'js')
    out = bool(langs) and langs <= {'vue', 'svelte', 'astro', 'graphql', 'grit'}
    if cids: emit('biome', v, r['id'], 'covered', ufix=r['fix'], rec=r['recommended'], cids=cids, note='through its source: ' + ', '.join(srcs)); continue
    if 'Biome' in n: emit('biome', v, r['id'], 'skip', ufix=r['fix'], rec=r['recommended'], note="about Biome's own config or comments"); continue
    if r['deprecated']: emit('biome', v, r['id'], 'retired', deprecated=True, ufix=r['fix'], rec=r['recommended']); continue
    if out: emit('biome', v, r['id'], 'out-of-scope', ufix=r['fix'], rec=r['recommended'], note='language outside L3: ' + ' '.join(sorted(langs))); continue
    same = [s for s in srcs if src_tool(s)[0] in EXTRACTED]
    if same: emit('biome', v, r['id'], 'same-as', ufix=r['fix'], rec=r['recommended'], note='counted under ' + ', '.join(same)); continue
    emit('biome', v, r['id'], 'port', ufix=r['fix'], rec=r['recommended'], pack=pk[0], newid=pid(pk[1], kebab(n), 'biome'),
         nfix=FIX[r['fix']], prio=('P3' if r['group'] == 'nursery' else ('P1' if r['recommended'] else 'P2')),
         note=('nursery upstream; ' if r['group'] == 'nursery' else '') + (('source: ' + ', '.join(srcs)) if srcs else 'Biome only'))

# ---------- write ----------
COLS = ['upstream_id', 'class_', 'codelaw', 'proposed_pack', 'proposed_id', 'proposed_fix', 'priority', 'upstream_fix', 'upstream_recommended', 'deprecated', 'note']
summary = {}
for tool in sorted(rows):
    rs = sorted(rows[tool], key=lambda x: x['upstream_id'])
    fn = tool.replace('@', '').replace('/', '-') + '.csv'
    with open(os.path.join(OUT, fn), 'w', newline='') as f:
        w = csv.writer(f, lineterminator='\n'); w.writerow([c.rstrip('_') for c in COLS])
        for x in rs: w.writerow([x[c] for c in COLS])
    summary[tool] = dict(version=rs[0]['version'], total=len(rs), file=fn, **collections.Counter(x['class_'] for x in rs))
json.dump(summary, open(os.path.join(OUT, 'summary.json'), 'w'), indent=1, sort_keys=True)
ports = [x for t in rows for x in rows[t] if x['class_'] == 'port']
byp = collections.defaultdict(collections.Counter)
for x in ports: byp[x['proposed_pack']][x['priority']] += 1
json.dump({'ports_by_pack': {k: dict(v) for k, v in sorted(byp.items())}}, open(os.path.join(OUT, 'ports.json'), 'w'), indent=1, sort_keys=True)
for t, s in summary.items(): print(f"{t:32} {s['total']:5}  " + '  '.join(f"{k}={s[k]}" for k in ['covered', 'port', 'types', 'layout', 'planned', 'out-of-scope', 'needs-data', 'retired', 'skip', 'same-as'] if k in s))
print('ports', len(ports), {k: dict(v) for k, v in sorted(byp.items())})

# ---------- Prettier: options, not rules ----------
pr = [r for r in up if r['tool'] == 'prettier']
PMAP = {'printWidth': 'line_width', 'tabWidth': 'indent_width', 'useTabs': 'indent_style', 'endOfLine': 'end_of_line',
        'singleQuote': 'quote_style', 'semi': 'semicolons', 'trailingComma': 'trailing_commas',
        'embeddedLanguageFormatting': '[format.markdown] format_code_blocks (Markdown only)'}
prow = []
for r in sorted(pr, key=lambda x: x['id']):
    n = r['id']
    if n in PMAP: prow.append([n, 'covered', PMAP[n], r.get('category', ''), ''])
    elif n in ('requirePragma', 'insertPragma', 'checkIgnorePragma'): prow.append([n, 'skip', '', r.get('category', ''), 'codelaw has its own ignore markers (SR-FORMATTER decision 6)'])
    elif n in ('cursorOffset', 'rangeStart', 'rangeEnd', 'parser', 'filepath', 'plugins'): prow.append([n, 'skip', '', r.get('category', ''), 'an API or CLI argument, not a style'])
    elif n == 'vueIndentScriptAndStyle': prow.append([n, 'out-of-scope', '', r.get('category', ''), 'Vue is outside L3'])
    else: prow.append([n, 'port', '', r.get('category', ''), 'a style option codelaw has no key for: add a [format] key, or rule that it stays at Prettier\'s default'])
with open(os.path.join(OUT, 'prettier.csv'), 'w', newline='') as f:
    w = csv.writer(f, lineterminator='\n'); w.writerow(['option', 'class', 'codelaw', 'category', 'note']); w.writerows(prow)
summary['prettier'] = dict(version=pr[0]['version'], total=len(prow), file='prettier.csv', **collections.Counter(x[1] for x in prow))
json.dump(summary, open(os.path.join(OUT, 'summary.json'), 'w'), indent=1, sort_keys=True)
