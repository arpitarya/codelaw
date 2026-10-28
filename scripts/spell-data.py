#!/usr/bin/env python3
"""The spell pack's word data: codelaw's own, generated from the ratified
sources only (SR-SPELL decision 6; L11 §3 D-42, D-44; W-103).

    spell-data.py fetch [--cache D]     download each pinned source into D and
                                        check its SHA-256 (the network, once;
                                        never in the engine, L5)
    spell-data.py write [--cache D]     write packs/spell/data/spell/ from D
    spell-data.py check [--cache D]     regenerate in memory; fail if any
                                        committed byte differs

What it writes:

- ``typos.json`` — the known-typo list (SR-SPELL decision 6, the decider's
  "Let's go with B"). It is *generated*, never copied: single-edit variants of
  SCOWL's most common words (size 20 and below, four letters or more) — two
  neighbouring letters swapped, a letter dropped, a consonant doubled, and,
  for the size-10 words only, a letter replaced by its neighbour on the same
  keyboard row (never the first letter) — kept only when the variant has
  four letters or more, is in no SCOWL list of size 80 or below (any
  spelling), in no SCOWL name, abbreviation or contraction list of any size
  and in no software term, and comes from exactly one source word. Entries
  added by hand (scripts/spell-typos.txt) are merged over the generated ones.

  The table is one JSON object whose ``entries`` string holds ``typo:fix``
  pairs (``typo:fix1|fix2`` for a hand entry with several corrections),
  space-separated and sorted by byte, so the engine binary-searches it as it
  is loaded: no per-entry JSON values, no index to build (SR-SPELL decision 9).

The same sources write the same bytes: every set is sorted, and nothing reads
the clock or the platform. The standard library only, Python 3.9.
"""
import argparse
import gzip
import hashlib
import io
import json
import os
import sys
import tarfile
import urllib.request
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "packs" / "spell" / "data" / "spell"
EXTRA = ROOT / "scripts" / "spell-typos.txt"

# The pins (L11 §3 records the same versions and digests).
SOURCES = {
    "scowl": {
        "version": "2020.12.07",
        "file": "scowl-2020.12.07.tar.gz",
        "url": "https://master.dl.sourceforge.net/project/wordlist/SCOWL/2020.12.07/scowl-2020.12.07.tar.gz?viasf=1",
        "sha256": "5587667caa20c4891390c2d42dbb4d5c4c3f41bee77af1457ece3ba23fb859cc",
    },
    "varcon": {
        "version": "2020.12.07",
        "file": "varcon-2020.12.07.tar.gz",
        "url": "https://downloads.sourceforge.net/project/wordlist/VarCon/2020.12.07/varcon-2020.12.07.tar.gz",
        "sha256": "3b0720c5718008f37c02658a83d51d5598dd86531f0a2206d410c854006cb184",
    },
    "software-terms": {
        "version": "5.4.8",
        "file": "dict-software-terms-5.4.8.tgz",
        "url": "https://registry.npmjs.org/@cspell/dict-software-terms/-/dict-software-terms-5.4.8.tgz",
        "sha256": "4ea4907a9d9a066d53efbd964b44a8712c34014b5e6c7a5cf605a6ff2d0f11a1",
    },
}

# The generation's parameters: changing one changes the list, so each is
# named in SR-SPELL decision 6.
SOURCE_MAX_SIZE = 20      # SCOWL size classes the source words come from
KEY_MAX_SIZE = 10         # the size classes whose words get keyboard-neighbour variants
EXCLUDE_MAX_SIZE = 80     # a variant in a SCOWL word list up to this size is a word
MIN_SOURCE_LEN = 4        # letters in a source word
MIN_TYPO_LEN = 4          # letters in a typo: the checker skips shorter words
DICT_MAX_SIZE = 60        # SCOWL size classes in the dictionary (aspell's default `en` size)
# The SCOWL lists the dictionary and the locale table read: English, US and
# British (with British -ize) words, names, upper-case words, abbreviations
# and contractions; Canadian and Australian lists and the variant lists are not.
DICT_REGIONS = ("english", "american", "british", "british_z")
DICT_KINDS = ("words", "proper-names", "upper", "abbreviations", "contractions")
SIZES = (10, 20, 35, 40, 50, 55, 60, 70, 80, 95)
ROWS = ("qwertyuiop", "asdfghjkl", "zxcvbnm")
VOWELS = set("aeiou")


def default_cache():
    return Path(os.environ.get("XDG_CACHE_HOME", Path.home() / ".cache")) / "codelaw" / "spell-data"


def sha256(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fetch(cache):
    cache.mkdir(parents=True, exist_ok=True)
    for name, s in SOURCES.items():
        dest = cache / s["file"]
        if dest.exists() and sha256(dest) == s["sha256"]:
            print(f"{name} {s['version']}: cached")
            continue
        req = urllib.request.Request(s["url"], headers={"User-Agent": "codelaw-spell-data"})
        with urllib.request.urlopen(req, timeout=300) as r:
            data = r.read()
        got = hashlib.sha256(data).hexdigest()
        if got != s["sha256"]:
            sys.exit(f"{name}: SHA-256 {got} is not the pinned {s['sha256']}")
        dest.write_bytes(data)
        print(f"{name} {s['version']}: fetched, {len(data)} bytes")


def archive(cache, name):
    path = cache / SOURCES[name]["file"]
    if not path.exists():
        sys.exit(f"{path} is missing: run `spell-data.py fetch` first")
    if sha256(path) != SOURCES[name]["sha256"]:
        sys.exit(f"{path}: not the pinned SHA-256; run `spell-data.py fetch` again")
    # Members are read in memory, never extracted.
    return tarfile.open(path, "r:gz")


def scowl_lists(cache):
    """{(category, size): [words]} of SCOWL's final/ lists (ISO-8859-1)."""
    out = {}
    with archive(cache, "scowl") as tar:
        for m in tar.getmembers():
            parts = m.name.split("/")
            if len(parts) != 3 or parts[1] != "final" or not m.isfile():
                continue
            category, _, size = parts[2].rpartition(".")
            out[(category, int(size))] = tar.extractfile(m).read().decode("latin-1").split()
    return out


def software_terms(cache):
    with archive(cache, "software-terms") as tar:
        raw = tar.extractfile("package/dict/softwareTerms.txt.gz").read()
    lines = gzip.decompress(raw).decode("utf-8").splitlines()
    return sorted({ln.strip() for ln in lines if ln.strip() and not ln.lstrip().startswith("#")})


def neighbours():
    pos = {c: (r, i) for r, row in enumerate(ROWS) for i, c in enumerate(row)}
    return {c: sorted(d for d in pos if pos[d][0] == pos[c][0] and abs(pos[d][1] - pos[c][1]) == 1) for c in pos}


def variants(word, keys, nb):
    """Single-edit variants of `word`; the first letter is never changed."""
    out = set()
    n = len(word)
    for i in range(1, n - 1):  # two neighbouring letters swapped
        if word[i] != word[i + 1]:
            out.add(word[:i] + word[i + 1] + word[i] + word[i + 2:])
    for i in range(1, n):  # a letter dropped
        out.add(word[:i] + word[i + 1:])
    for i in range(1, n):  # a consonant doubled
        if word[i] not in VOWELS and word[i] != word[i - 1] and (i + 1 == n or word[i] != word[i + 1]):
            out.add(word[:i] + word[i] + word[i:])
    if keys:  # a letter replaced by its neighbour on the same row
        for i in range(1, n):
            for d in nb.get(word[i], ()):
                out.add(word[:i] + d + word[i + 1:])
    out.discard(word)
    return out


def read_extra():
    """scripts/spell-typos.txt: `typo fix[,fix…]` per line, `#` comments."""
    extra = {}
    if not EXTRA.exists():
        return extra
    for n, line in enumerate(EXTRA.read_text(encoding="utf-8").splitlines(), 1):
        line = line.split("#", 1)[0].strip()
        if not line:
            continue
        typo, _, fixes = line.partition(" ")
        fixes = [f.strip() for f in fixes.split(",") if f.strip()]
        if not typo.isascii() or not typo.isalpha() or not typo.islower() or not fixes:
            sys.exit(f"{EXTRA}:{n}: expected `typo fix[,fix…]` in lowercase ASCII letters")
        extra[typo] = fixes
    return extra


def dictionary_words(lists, terms):
    """{word: size class} — lower case, ASCII letters only; a contraction or
    possessive gives the letters before its apostrophe (`doesn't` → `doesn`),
    because the checker splits words there. A word keeps its smallest class;
    a software term absent from SCOWL gets class 100."""
    out = {}

    def add(word, size):
        w = word.lower()
        if w.isascii() and w.isalpha() and (w not in out or size < out[w]):
            out[w] = size

    for (category, size), words in lists.items():
        region, _, kind = category.partition("-")
        if size > DICT_MAX_SIZE or region not in DICT_REGIONS or kind not in DICT_KINDS:
            continue
        for w in words:
            for part in w.replace("'", " ").split()[:1]:
                add(part, size)
    for t in terms:
        for part in "".join(c if c.isascii() and c.isalpha() else " " for c in t).split():
            add(part, 100)
    return out


def varcon_pairs(cache, lists):
    """VarCon's one-to-one US↔British pairs: a line with exactly one plain
    American (`A`) and one plain British (`B`) spelling, both lower-case
    ASCII letters, that differ; a pair is kept only when each spelling is
    in no SCOWL list of the other locale (size 80 or below), so a word that
    is right in both under another meaning (`check`/`cheque`,
    `program`/`programme`) is never reported."""
    with archive(cache, "varcon") as tar:
        text = tar.extractfile("varcon-2020.12.07/varcon.txt").read().decode("latin-1")

    def words_of(regions):
        return {w.lower() for (c, size), ws in lists.items() if size <= EXCLUDE_MAX_SIZE and c.partition("-")[0] in regions and c.endswith("-words") for w in ws}

    us_words = words_of(("english", "american"))
    gb_words = words_of(("english", "british"))
    gb_z_words = words_of(("english", "british", "british_z"))
    pairs = []
    for line in text.splitlines():
        line = line.split("#", 1)[0].split("|", 1)[0].strip()
        if not line:
            continue
        a = b = None
        ok = True
        for entry in line.split("/"):
            tags, _, word = entry.partition(":")
            word, tags = word.strip(), tags.split()
            if "A" in tags:
                ok = ok and a is None
                a = word
            if "B" in tags:
                ok = ok and b is None
                b = word
        if ok and a and b and a != b and all(w.isascii() and w.isalpha() and w.islower() for w in (a, b)):
            pairs.append((a, b))

    def one_to_one(kept):
        us_count, gb_count = defaultdict(int), defaultdict(int)
        for a, b in kept:
            us_count[a] += 1
            gb_count[b] += 1
        return sorted({(a, b) for a, b in kept if us_count[a] == 1 and gb_count[b] == 1})

    # Under en-US a British spelling is reported: it is no American word and
    # its American twin is no British word. Under en-GB an American spelling
    # is reported: it is no British word (Oxford -ize spellings included,
    # so `organize` is never reported) and its British twin no American word.
    us = one_to_one([(a, b) for a, b in set(pairs) if b not in us_words and a not in gb_words])
    gb = one_to_one([(a, b) for a, b in set(pairs) if a not in gb_z_words and b not in us_words])
    return us, gb


def build(cache):
    lists = scowl_lists(cache)
    terms = software_terms(cache)
    # A variant that is a word is no typo. SCOWL's size-95 word lists are
    # left out of that test: they hold the commonest misspellings
    # (`untill`, `wierd`, `accomodate`) among their rare words.
    known = {w.lower() for (category, size), words in lists.items() if size <= EXCLUDE_MAX_SIZE or not category.endswith("-words") for w in words}
    known.update(t.lower() for t in terms)

    def common(max_size):
        out = set()
        for (category, size), words in lists.items():
            if size <= max_size and category in ("english-words", "american-words", "british-words"):
                out.update(w for w in words if w.isascii() and w.isalpha() and w.islower() and len(w) >= MIN_SOURCE_LEN)
        return out

    keyed = common(KEY_MAX_SIZE)
    nb = neighbours()
    sources = defaultdict(set)
    for word in sorted(common(SOURCE_MAX_SIZE)):
        for v in variants(word, word in keyed, nb):
            if len(v) >= MIN_TYPO_LEN and v not in known:
                sources[v].add(word)
    table = {typo: [next(iter(src))] for typo, src in sources.items() if len(src) == 1}
    generated = len(table)
    table.update(read_extra())
    entries = " ".join(f"{t}:{'|'.join(table[t])}" for t in sorted(table))
    doc = {
        "count": len(table),
        "entries": entries,
        "format": 1,
        "generated": generated,
        "generated_by": "scripts/spell-data.py",
        "sources": {k: v["version"] for k, v in sorted(SOURCES.items())},
    }
    words = dictionary_words(lists, terms)
    classes = defaultdict(list)
    for w, size in words.items():
        classes[str(size)].append(w)
    dictionary = {
        "classes": {k: " ".join(sorted(v)) for k, v in classes.items()},
        "count": len(words),
        "format": 1,
        "generated_by": "scripts/spell-data.py",
        "max_size": DICT_MAX_SIZE,
        "sources": {k: v["version"] for k, v in sorted(SOURCES.items()) if k != "varcon"},
    }
    us, gb = varcon_pairs(cache, lists)
    locale = {
        "count": len(us) + len(gb),
        "en-GB": " ".join(f"{a}:{b}" for a, b in gb),
        "en-US": " ".join(f"{b}:{a}" for a, b in sorted(us, key=lambda p: p[1])),
        "format": 1,
        "generated_by": "scripts/spell-data.py",
        "sources": {k: SOURCES[k]["version"] for k in ("scowl", "varcon")},
    }
    doc["sources"] = {k: v["version"] for k, v in sorted(SOURCES.items()) if k != "varcon"}
    dump = lambda d: json.dumps(d, sort_keys=True, ensure_ascii=True, separators=(",", ":")) + "\n"  # noqa: E731
    return {"typos.json": dump(doc), "words.json": dump(dictionary), "locale.json": dump(locale)}


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("command", choices=("fetch", "write", "check"))
    ap.add_argument("--cache", type=Path, default=default_cache())
    a = ap.parse_args()
    if a.command == "fetch":
        fetch(a.cache)
        return 0
    files = build(a.cache)
    if a.command == "write":
        OUT.mkdir(parents=True, exist_ok=True)
        for name, text in files.items():
            (OUT / name).write_text(text, encoding="utf-8", newline="\n")
            doc = json.loads(text)
            print(f"{name}: {doc['count']} entries, {len(text.encode())} bytes")
        return 0
    bad = [n for n, t in files.items() if not (OUT / n).exists() or (OUT / n).read_text(encoding="utf-8") != t]
    for n in bad:
        print(f"packs/spell/data/spell/{n} differs from what scripts/spell-data.py writes", file=sys.stderr)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
