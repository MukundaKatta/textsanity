# textsanity

Fast unicode/whitespace/encoding cleanup before LLM input.
Rust core, Python frontend.

## What it does

A bag of small text-cleanup operations every LLM-app builder reaches for
and rewrites poorly:

- **NFKC normalize** so `ﬁ` (fi-ligature) becomes `fi`, full-width digits
  become regular digits, etc.
- **Strip zero-width** code points (ZWSP, ZWJ, ZWNJ, BOM, RTL/LTR marks)
  that often sneak in from copy-paste and break tokenization.
- **Strip control characters** (C0/C1) except `\n` and `\t`.
- **Collapse whitespace** so `"hello   \n\n  world"` becomes `"hello world"`.
- **Trim** leading/trailing whitespace.
- **ASCII punctuation** mode replaces `“”‘’–—…` with `""''-/-...`.
- **Strip emoji** and **ASCII-only** modes for stricter sanitization.

Defaults are LLM-prep-shaped: NFKC + zero-width + control + collapse + trim
all on; emoji/ASCII-only off.

## Install

```bash
pip install textsanity
```

## Quickstart

```python
from textsanity import sanitize, Options

dirty = "Hello​   World  —  with  ﬁ   and  smart “quotes” "
clean = sanitize(dirty)
# "Hello World — with fi and smart "quotes""

# Lock down further if you want pure ASCII:
clean = sanitize(dirty, Options(ascii_punctuation=True, ascii_only=True))
# "Hello World - with fi and smart "quotes""
```

## License

Dual-licensed under MIT or Apache-2.0.
