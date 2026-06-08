"""End-to-end tests."""

from __future__ import annotations

from textsanity import Options, __version__, sanitize, sanitize_many


def test_version_present() -> None:
    assert isinstance(__version__, str) and __version__ != ""


def test_default_collapses_and_trims() -> None:
    assert sanitize("  hello   world  ") == "hello world"


def test_nfkc_normalizes_ligature() -> None:
    assert sanitize("ﬁle") == "file"


def test_zero_width_stripped() -> None:
    assert sanitize("hi​there﻿") == "hithere"


def test_control_chars_stripped() -> None:
    # \x01 stripped, \n collapsed via whitespace.
    assert sanitize("a\x01b\nc") == "ab c"


def test_ascii_punctuation_off_by_default() -> None:
    # Default keeps smart quotes intact.
    assert sanitize("“hi”") == "“hi”"


def test_ascii_punctuation_replaces_smart_quotes() -> None:
    o = Options(ascii_punctuation=True)
    assert sanitize("“hello” — world…", o) == '"hello" - world...'


def test_ascii_only_drops_non_ascii() -> None:
    o = Options(ascii_only=True)
    assert sanitize("hello 你好 world", o) == "hello world"


def test_strip_emoji() -> None:
    o = Options(strip_emoji=True)
    assert sanitize("hi \U0001f30d world \U0001f680", o) == "hi world"


def test_strip_emoji_flags() -> None:
    o = Options(strip_emoji=True)
    # Flags are regional indicator pairs (US, then JP).
    assert sanitize("hi \U0001f1fa\U0001f1f8 there \U0001f1ef\U0001f1f5", o) == "hi there"


def test_strip_emoji_keycap() -> None:
    o = Options(strip_emoji=True)
    # "1" + variation selector + combining enclosing keycap.
    assert sanitize("press 1️⃣ now", o) == "press 1 now"


def test_options_immutable() -> None:
    import contextlib

    o = Options()
    with contextlib.suppress(AttributeError, Exception):
        o.nfkc = False  # type: ignore[misc]
    assert o.nfkc is True


def test_collapse_off_keeps_runs() -> None:
    o = Options(collapse_whitespace=False)
    assert sanitize("  hello   world  ", o) == "hello   world"


def test_trim_off_keeps_edges() -> None:
    o = Options(trim=False, collapse_whitespace=False)
    assert sanitize("  hi  ", o) == "  hi  "


def test_empty_input() -> None:
    assert sanitize("") == ""


def test_idempotent_on_clean_input() -> None:
    once = sanitize("hello world")
    twice = sanitize(once)
    assert once == twice


def test_sanitize_many_serial_and_parallel_match() -> None:
    inputs = ["  hi  ", "world﻿", "ﬁle"]
    s = sanitize_many(inputs)
    p = sanitize_many(inputs, parallel=True)
    assert s == p
    assert s == ["hi", "world", "file"]


def test_sanitize_many_empty_list() -> None:
    assert sanitize_many([]) == []


def test_options_dataclass_is_hashable() -> None:
    # Frozen dataclass → hashable.
    o1 = Options(nfkc=False)
    o2 = Options(nfkc=False)
    assert hash(o1) == hash(o2)
    assert o1 == o2
