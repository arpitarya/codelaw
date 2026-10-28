# The codelaw icon

A check between two code brackets: code, and the one verdict on it. One
colour only — there is no full-colour version.

| file | use it for |
|---|---|
| [`codelaw-mono.svg`](codelaw-mono.svg) | Ink on light backgrounds — README, docs, print, badges, avatars |
| [`codelaw-mono-white.svg`](codelaw-mono-white.svg) | white on dark backgrounds |
| [`png/`](png/) | both at 16, 32, 48, 128, 256 and 512 px |

Both SVGs draw in `currentColor`: inlined, they take the surrounding text
colour; loaded through `<img>`, they keep the colour set on their root.

## Colours

| name | hex |
|---|---|
| Ink | `#182133` — also the VS Code gallery banner |
| Paper | `#FFFFFF` |

## Regenerating the PNGs

```bash
for s in 16 32 48 128 256 512; do for v in mono mono-white; do rsvg-convert -w $s -h $s codelaw-$v.svg -o png/codelaw-$v-$s.png; done; done
```

Run from this folder; `rsvg-convert` is librsvg's (LGPL, a local tool, not a
dependency of codelaw).
