import { createRequire } from 'node:module'; import path from 'node:path'; import fs from 'node:fs';
const ROOT = process.argv[2]; const require = createRequire(path.join(ROOT, 'x.js'));
const load = async p => { try { return require(p); } catch { return await import(require.resolve(p)); } };
const on = rules => Object.entries(rules || {}).filter(([, v]) => !(v === 'off' || v === 0 || (Array.isArray(v) && (v[0] === 'off' || v[0] === 0)) || v === null || v === false)).map(([k]) => k);
const out = {};
const strip = (ids, pre) => ids.filter(i => i.startsWith(pre)).map(i => i.slice(pre.length));
{ const js = await load('@eslint/js'); out['eslint'] = on((js.default || js).configs.recommended.rules); }
{ const ts = await load('@typescript-eslint/eslint-plugin'); const p = ts.default || ts; const c = p.configs['flat/recommended'] || p.configs.recommended; const rules = Array.isArray(c) ? Object.assign({}, ...c.map(x => x.rules || {})) : c.rules; out['typescript-eslint'] = strip(on(rules), '@typescript-eslint/'); }
for (const [t, pre, key] of [['eslint-plugin-react', 'react/', 'recommended'], ['eslint-plugin-react-hooks', 'react-hooks/', 'recommended'], ['eslint-plugin-jsx-a11y', 'jsx-a11y/', 'recommended'], ['eslint-plugin-import', 'import/', 'recommended'], ['eslint-plugin-import-x', 'import-x/', 'recommended'], ['eslint-plugin-n', 'n/', 'flat/recommended'], ['eslint-plugin-unicorn', 'unicorn/', 'recommended'], ['eslint-plugin-security', 'security/', 'recommended'], ['@tanstack/eslint-plugin-query', '@tanstack/query/', 'flat/recommended'],
  // round 2 (W-144)
  ['eslint-plugin-sonarjs', 'sonarjs/', 'recommended'], ['eslint-plugin-jest', 'jest/', 'flat/recommended'], ['@vitest/eslint-plugin', 'vitest/', 'recommended'], ['eslint-plugin-testing-library', 'testing-library/', 'flat/react'], ['eslint-plugin-playwright', 'playwright/', 'flat/recommended'], ['eslint-plugin-promise', 'promise/', 'flat/recommended'], ['eslint-plugin-jsdoc', 'jsdoc/', 'flat/recommended'], ['eslint-plugin-regexp', 'regexp/', 'flat/recommended'], ['@next/eslint-plugin-next', '@next/next/', 'recommended'], ['eslint-plugin-tailwindcss', 'tailwindcss/', 'flat/recommended'], ['eslint-plugin-better-tailwindcss', 'better-tailwindcss/', 'recommended']]) {
  const m = await load(t); const p = m.default || m; let c = (p.configs || {})[key] || (p.configs || {})['flat/recommended'] || (p.configs || {})['recommended'] || (p.configs || {})['recommended-latest'];
  if (Array.isArray(c) && c.length && typeof c[0] === 'string') c = null;
  const rules = Array.isArray(c) ? Object.assign({}, ...c.map(x => x.rules || {})) : (c && c.rules);
  out[t] = strip(on(rules), pre);
}
{ const c = await import(path.join(ROOT,'node_modules/stylelint-config-recommended/index.js')); out['stylelint'] = on((c.default || c).rules); }
{ const c = await import(path.join(ROOT,'node_modules/stylelint-config-recommended-scss/index.js')); out['stylelint-scss'] = on((c.default || c).rules).filter(i => i.startsWith('scss/')); }
{ const h = require('html-validate'); const pre = h.configPresets['html-validate:recommended']; out['html-validate'] = on(pre.rules); }
fs.writeFileSync(process.argv[3], JSON.stringify(out, null, 1));
console.log(Object.fromEntries(Object.entries(out).map(([k, v]) => [k, v.length])));
