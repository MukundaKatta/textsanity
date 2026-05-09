"""Type stubs for `textsanity._native`."""

from __future__ import annotations

__version__: str

def sanitize(
    text: str,
    *,
    nfkc: bool = True,
    strip_zero_width: bool = True,
    strip_control: bool = True,
    collapse_whitespace: bool = True,
    trim: bool = True,
    ascii_punctuation: bool = False,
    strip_emoji: bool = False,
    ascii_only: bool = False,
) -> str: ...
def sanitize_many(
    texts: list[str],
    *,
    parallel: bool = False,
    nfkc: bool = True,
    strip_zero_width: bool = True,
    strip_control: bool = True,
    collapse_whitespace: bool = True,
    trim: bool = True,
    ascii_punctuation: bool = False,
    strip_emoji: bool = False,
    ascii_only: bool = False,
) -> list[str]: ...
