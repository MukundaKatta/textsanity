"""Fast unicode/whitespace/encoding cleanup before LLM input.

Wraps `textsanity._native.sanitize` (Rust + unicode-normalization) with a
typed `Options` dataclass so callers can build a config once and reuse
it across batches.
"""

from __future__ import annotations

from collections.abc import Sequence
from dataclasses import dataclass
from importlib import metadata
from typing import Final

from textsanity._native import sanitize as _sanitize
from textsanity._native import sanitize_many as _sanitize_many


def _read_version() -> str:
    try:
        return metadata.version("textsanity")
    except metadata.PackageNotFoundError:
        return "0.0.0"


__version__: Final[str] = _read_version()

__all__ = ["Options", "__version__", "sanitize", "sanitize_many"]


@dataclass(frozen=True)
class Options:
    """Cleanup pipeline configuration. Defaults are the LLM-prep-shaped set."""

    nfkc: bool = True
    strip_zero_width: bool = True
    strip_control: bool = True
    collapse_whitespace: bool = True
    trim: bool = True
    ascii_punctuation: bool = False
    strip_emoji: bool = False
    ascii_only: bool = False


def sanitize(text: str, opts: Options | None = None) -> str:
    """Run the cleanup pipeline against `text`."""
    o = opts or Options()
    return str(
        _sanitize(
            text,
            nfkc=o.nfkc,
            strip_zero_width=o.strip_zero_width,
            strip_control=o.strip_control,
            collapse_whitespace=o.collapse_whitespace,
            trim=o.trim,
            ascii_punctuation=o.ascii_punctuation,
            strip_emoji=o.strip_emoji,
            ascii_only=o.ascii_only,
        )
    )


def sanitize_many(
    texts: Sequence[str], opts: Options | None = None, *, parallel: bool = False
) -> list[str]:
    """Bulk variant. With `parallel=True`, distributes across rayon."""
    o = opts or Options()
    return list(
        _sanitize_many(
            list(texts),
            parallel=parallel,
            nfkc=o.nfkc,
            strip_zero_width=o.strip_zero_width,
            strip_control=o.strip_control,
            collapse_whitespace=o.collapse_whitespace,
            trim=o.trim,
            ascii_punctuation=o.ascii_punctuation,
            strip_emoji=o.strip_emoji,
            ascii_only=o.ascii_only,
        )
    )
