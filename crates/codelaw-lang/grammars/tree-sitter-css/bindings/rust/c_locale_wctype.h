/*
 * codelaw: the external scanner's character classes, the same on every
 * platform. Force-included before src/ by bindings/rust/build.rs, so src/
 * stays upstream's byte for byte.
 *
 * The scanner asks <wctype.h> whether a character is a space or a letter.
 * The answer depends on the C library: macOS and glibc in the "C" locale
 * (a Rust program never calls setlocale) say ASCII only, but MSVC's
 * iswspace says yes to U+00A0 and musl's to U+2028 and U+3000. On Windows
 * `.menu<U+00A0>.item` then read as a descendant selector, with the same
 * tokens as `.menu .item`, and css/no-irregular-whitespace's SCSS example
 * failed only on the win32 legs (W-44, 2026-10-27). These are the "C"
 * locale's classes, which is what the grammar is tested with upstream.
 */
#ifndef CODELAW_C_LOCALE_WCTYPE_H
#define CODELAW_C_LOCALE_WCTYPE_H

#include <wctype.h>

static inline int codelaw_iswspace(wint_t c) { return c == ' ' || (c >= '\t' && c <= '\r'); }
static inline int codelaw_iswdigit(wint_t c) { return c >= '0' && c <= '9'; }
static inline int codelaw_iswalpha(wint_t c) { return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z'); }
static inline int codelaw_iswalnum(wint_t c) { return codelaw_iswalpha(c) || codelaw_iswdigit(c); }

#undef iswspace
#undef iswdigit
#undef iswalpha
#undef iswalnum
#define iswspace codelaw_iswspace
#define iswdigit codelaw_iswdigit
#define iswalpha codelaw_iswalpha
#define iswalnum codelaw_iswalnum

#endif
