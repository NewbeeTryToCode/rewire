"""Query-side text handling for recall: stopwords and light stemming (EN + ID).

Index text is stored as written; the query is widened instead: every content
word is searched as itself OR as a prefix of its stem(s), so "deployed" finds
"deploy" and "menyimpan" finds "simpan". Deliberately crude; measured in
experiments/retrieval.
"""

import re

STOPWORDS = set("""
a an and are as at be been but by can could do does did for from has have how i if in into is it its
me my no not of on or our should so that the their them then there these they this to us was we were
what when where which who why will with would you your still any some about must need use used using
apa ada adalah agar akan aku anda apakah atau bagaimana bahwa banyak belum bila boleh bisa dalam dan
dari dengan di dia harus hanya ini itu jika juga kalau kami kamu kan ke kenapa ketika kita lagi lain
mana masih mau mereka nya oleh pada para perlu saat saja sama sampai saya sebelum sedang sesudah setelah
siapa sudah supaya tapi tetapi tidak untuk yang
""".split())

_VOWEL = "aeiou"


def _en(term: str) -> set[str]:
    out = set()
    for suffix, repl in (("ies", "y"), ("ing", ""), ("ed", ""), ("es", ""), ("s", ""), ("ly", "")):
        if term.endswith(suffix) and len(term) - len(suffix) >= 3:
            out.add(term[: -len(suffix)] + repl)
            break
    return out


def _id(term: str) -> set[str]:
    out = set()
    base = term
    for suffix in ("nya", "lah", "kah", "kan", "an", "i"):
        if base.endswith(suffix) and len(base) - len(suffix) >= 4:
            base = base[: -len(suffix)]
            out.add(base)
            break
    for word in {term, base}:
        for prefix, vowel_repl in (("meny", "s"), ("peny", "s"), ("meng", "k"), ("peng", "k"), ("mem", "p"),
                                   ("pem", "p"), ("men", "t"), ("pen", "t"), ("ber", ""), ("ter", ""),
                                   ("me", ""), ("pe", ""), ("di", ""), ("ke", ""), ("se", "")):
            if word.startswith(prefix) and len(word) - len(prefix) >= 3:
                rest = word[len(prefix):]
                out.add(rest)
                if vowel_repl and rest[0] in _VOWEL:
                    out.add(vowel_repl + rest)
                break
    return out


def stems(term: str) -> set[str]:
    return {s for s in _en(term) | _id(term) if s != term and len(s) >= 3}


def match_expression(query: str) -> str | None:
    """FTS5 MATCH expression: (term OR stem*) for each content word, OR-ed."""
    words = [w for w in re.findall(r"\w+", query.lower()) if len(w) > 1]
    content = [w for w in words if w not in STOPWORDS] or words
    parts = []
    for w in dict.fromkeys(content):
        alts = [f'"{w}"'] + [f'"{s}"*' if len(s) >= 4 else f'"{s}"' for s in sorted(stems(w))]
        parts.append("(" + " OR ".join(alts) + ")" if len(alts) > 1 else alts[0])
    return " OR ".join(parts) if parts else None
