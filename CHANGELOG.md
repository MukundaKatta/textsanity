# Changelog

## [0.1.0] - 2026-05-09

### Added
- NFKC unicode normalization, zero-width strip, control-char strip,
  whitespace collapse, trim — all individually configurable.
- Optional ASCII punctuation conversion (smart quotes, dashes).
- Optional emoji strip and ASCII-only modes.
- Bulk `sanitize_many` with rayon parallelism.
- abi3-py310 wheel: one wheel for CPython 3.10 through 3.13.
