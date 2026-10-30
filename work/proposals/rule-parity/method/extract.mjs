// Extract every rule each tool ships, from its own package. Output: up.jsonl
import { createRequire } from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
const ROOT = process.argv[2];
const require = createRequire(path.join(ROOT, 'x.js'));
const out = [];
const ver = p => JSON.parse(fs.readFileSync(path.join(ROOT,'node_modules',p,'package.json'),'utf8')).version;
const add = (tool, pkg, id, o = {}) => out.push({ tool, version: ver(pkg), id, ...o });
const fixOf = m => m?.fixable ? 'auto' : (m?.hasSuggestions ? 'suggest' : 'none');
async function load(p) { try { return require(p); } catch { const r = require.resolve(p); return await import(r); } }

function eslintLike(tool, pkg, rules, extra = () => ({})) {
  for (const [name, r] of Object.entries(rules)) {
    const m = r.meta || {};
    add(tool, pkg, name, { deprecated: !!m.deprecated, fix: fixOf(m), kind: m.type || '', recommended: !!(m.docs && m.docs.recommended), desc: (m.docs && m.docs.description) || '', ...extra(m) });
  }
}

// ESLint core
{ const { builtinRules } = require('eslint/use-at-your-own-risk'); eslintLike('eslint', 'eslint', Object.fromEntries(builtinRules)); }
// typescript-eslint
{ const p = await load('@typescript-eslint/eslint-plugin'); const rules = (p.default || p).rules; eslintLike('typescript-eslint', '@typescript-eslint/eslint-plugin', rules, m => ({ types: !!(m.docs && m.docs.requiresTypeChecking), extends: (m.docs && m.docs.extendsBaseRule) ? String(m.docs.extendsBaseRule === true ? 'base' : m.docs.extendsBaseRule) : '' })); }
// ESLint plugins
for (const [tool, pkg] of [['eslint-plugin-react', 'eslint-plugin-react'], ['eslint-plugin-react-hooks', 'eslint-plugin-react-hooks'], ['eslint-plugin-jsx-a11y', 'eslint-plugin-jsx-a11y'], ['eslint-plugin-import', 'eslint-plugin-import'], ['eslint-plugin-import-x', 'eslint-plugin-import-x'], ['eslint-plugin-n', 'eslint-plugin-n'], ['eslint-plugin-unicorn', 'eslint-plugin-unicorn'], ['eslint-plugin-security', 'eslint-plugin-security'], ['@tanstack/eslint-plugin-query', '@tanstack/eslint-plugin-query'], ['@stylistic/eslint-plugin', '@stylistic/eslint-plugin'],
  // round 2 (W-144)
  ['eslint-plugin-sonarjs', 'eslint-plugin-sonarjs'], ['eslint-plugin-jest', 'eslint-plugin-jest'], ['@vitest/eslint-plugin', '@vitest/eslint-plugin'], ['eslint-plugin-testing-library', 'eslint-plugin-testing-library'], ['eslint-plugin-playwright', 'eslint-plugin-playwright'], ['eslint-plugin-promise', 'eslint-plugin-promise'], ['eslint-plugin-jsdoc', 'eslint-plugin-jsdoc'], ['eslint-plugin-regexp', 'eslint-plugin-regexp'], ['@next/eslint-plugin-next', '@next/eslint-plugin-next'], ['eslint-plugin-tailwindcss', 'eslint-plugin-tailwindcss'], ['eslint-plugin-better-tailwindcss', 'eslint-plugin-better-tailwindcss']]) {
  const p = await load(pkg); const rules = (p.rules || (p.default && p.default.rules)); eslintLike(tool, pkg, rules, m => ({ types: !!(m.docs && (m.docs.requiresTypeChecking || (m.docs.requiresTypeChecking === undefined && m.docs.type === 'type-aware'))) }));
}
// TSLint
{ const dir = path.join(ROOT, 'node_modules/tslint/lib/rules');
  for (const f of fs.readdirSync(dir).filter(f => f.endsWith('Rule.js'))) {
    const md = require(path.join(dir, f)).Rule?.metadata; if (!md) continue;
    add('tslint', 'tslint', md.ruleName, { deprecated: !!md.deprecationMessage, fix: md.hasFix ? 'auto' : 'none', kind: md.type || '', types: !!md.requiresTypeInfo, desc: md.description || '' });
  } }
// stylelint
{ const s = (await load('stylelint')); const rules = (s.default || s).rules;
  for (const [name, r] of Object.entries(rules)) { const rule = await r; const m = rule.meta || {};
    add('stylelint', 'stylelint', name, { deprecated: !!m.deprecated, fix: m.fixable ? 'auto' : 'none', desc: '' }); } }
// stylelint-scss
{ const s = await load('stylelint-scss'); const list = s.default || s;
  for (const pl of list) { const m = pl.rule?.meta || {}; add('stylelint-scss', 'stylelint-scss', pl.ruleName, { deprecated: !!m.deprecated, fix: m.fixable ? 'auto' : 'none' }); } }
// html-validate
{ const h = require('html-validate'); for (const [name, R] of Object.entries(h.builtinRules)) {
    let deprecated = false; try { deprecated = !!R.prototype?.deprecated; } catch {}
    add('html-validate', 'html-validate', name, { deprecated, fix: 'none' }); } }
// markdownlint
{ const lib = await import(path.join(ROOT, 'node_modules/markdownlint/lib/exports.mjs')).catch(() => null);
  let rules = null;
  try { rules = (await import(path.join(ROOT, 'node_modules/markdownlint/lib/rules.mjs'))).default; } catch {}
  for (const r of rules || []) add('markdownlint', 'markdownlint', r.names[0], { alias: r.names.slice(1).join(' '), fix: 'none', desc: r.description, tags: r.tags.join(' ') }); }
// Prettier options
{ const p = await load('prettier'); const info = await (p.default || p).getSupportInfo();
  for (const o of info.options) add('prettier', 'prettier', o.name, { kind: 'option', desc: o.description || '', category: o.category || '' }); }
fs.writeFileSync(process.argv[3], out.map(o => JSON.stringify(o)).join('\n') + '\n');
const c = {}; for (const o of out) c[o.tool] = (c[o.tool] || 0) + 1; console.log(c);
