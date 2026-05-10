# Changelog

## [0.1.1] - 2026-05-10

### Added
- `Options::strict()` preset: every cleanup operation enabled. Use when
  feeding untrusted text into a downstream that's sensitive to unicode
  tricks (token smuggling, RTL embedding, etc.).
- `normalize_newlines(text)` standalone helper: collapses `\r\n` (CRLF)
  and lone `\r` (CR) to `\n`. Idempotent. Independent from `sanitize`.

## [0.1.0] - 2026-05-09

### Added
- NFKC unicode normalization, zero-width strip, control-char strip,
  whitespace collapse, trim — all individually configurable.
- Optional ASCII punctuation conversion (smart quotes, dashes).
- Optional emoji strip and ASCII-only modes.
- Bulk `sanitize_many` with rayon parallelism.
- abi3-py310 wheel: one wheel for CPython 3.10 through 3.13.
