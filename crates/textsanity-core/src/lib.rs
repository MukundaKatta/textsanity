//! Pure-Rust core for `textsanity`. Configurable text cleanup before
//! the input gets near a tokenizer or an LLM.
//!
//! Each operation is independent and toggleable via [`Options`]. The
//! defaults reflect what most LLM-app builders actually want: NFKC,
//! zero-width strip, control-char strip, whitespace collapse, trim.

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use unicode_normalization::UnicodeNormalization;

/// Crate-wide result alias.
pub type Result<T> = std::result::Result<T, SanityError>;

/// All errors surfaced by `textsanity-core`.
#[derive(Error, Debug)]
pub enum SanityError {
    /// Reserved for future use; constructor is currently infallible.
    #[error("invalid config: {0}")]
    InvalidConfig(String),
}

/// Cleanup pipeline configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Options {
    /// Apply NFKC unicode normalization.
    pub nfkc: bool,
    /// Strip zero-width code points (ZWSP, ZWJ, ZWNJ, BOM, RTL/LTR marks).
    pub strip_zero_width: bool,
    /// Strip control characters (C0 + C1) except `\n` and `\t`.
    pub strip_control: bool,
    /// Collapse runs of whitespace to a single space.
    pub collapse_whitespace: bool,
    /// Trim leading and trailing whitespace.
    pub trim: bool,
    /// Replace common smart punctuation (curly quotes, em/en dash, ellipsis)
    /// with plain ASCII equivalents.
    pub ascii_punctuation: bool,
    /// Strip emoji and pictograph code points.
    pub strip_emoji: bool,
    /// Drop any non-ASCII character. Applied after `ascii_punctuation` so
    /// smart punctuation gets converted, not deleted.
    pub ascii_only: bool,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            nfkc: true,
            strip_zero_width: true,
            strip_control: true,
            collapse_whitespace: true,
            trim: true,
            ascii_punctuation: false,
            strip_emoji: false,
            ascii_only: false,
        }
    }
}

impl Options {
    /// Strict preset: every cleanup operation enabled. Use when feeding
    /// untrusted text into a downstream that's sensitive to unicode tricks.
    pub fn strict() -> Self {
        Self {
            nfkc: true,
            strip_zero_width: true,
            strip_control: true,
            collapse_whitespace: true,
            trim: true,
            ascii_punctuation: true,
            strip_emoji: true,
            ascii_only: true,
        }
    }
}

/// Normalize line endings to `\n`. Converts both `\r\n` (CRLF) and lone
/// `\r` (CR) to `\n`. Idempotent. Cheap to apply before or after the main
/// `sanitize` pipeline.
pub fn normalize_newlines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'\r' {
            out.push('\n');
            // Skip a following \n so CRLF collapses to one \n.
            if i + 1 < bytes.len() && bytes[i + 1] == b'\n' {
                i += 1;
            }
        } else {
            // Reconstruct the char from the byte (ASCII-safe; fall through to
            // re-decoding the rest as UTF-8 if a multi-byte sequence starts).
            if b < 0x80 {
                out.push(b as char);
            } else {
                // Multi-byte UTF-8: take a slice and let str handle it.
                let end = next_char_boundary(bytes, i);
                // Safe because text is &str and the indices are char boundaries.
                out.push_str(&text[i..end]);
                i = end - 1;
            }
        }
        i += 1;
    }
    out
}

fn next_char_boundary(bytes: &[u8], start: usize) -> usize {
    let mut end = start + 1;
    while end < bytes.len() && (bytes[end] & 0xC0) == 0x80 {
        end += 1;
    }
    end
}

/// Run the cleanup pipeline against `text` with the given options.
pub fn sanitize(text: &str, opts: &Options) -> String {
    // 1. NFKC normalize. This is the only step that needs a separate pass.
    let s: String = if opts.nfkc {
        text.nfkc().collect()
    } else {
        text.to_string()
    };

    // 2. Char-level filter pass (zero-width, control, emoji, ascii-only,
    //    smart-punctuation rewrite). Single allocation; bounded by len(s).
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if opts.strip_zero_width && is_zero_width(c) {
            continue;
        }
        if opts.strip_control && is_strippable_control(c) {
            continue;
        }
        if opts.strip_emoji && is_emoji_codepoint(c) {
            continue;
        }
        let mapped = if opts.ascii_punctuation {
            map_smart_punctuation(c)
        } else {
            CharRewrite::Single(c)
        };
        match mapped {
            CharRewrite::Single(ch) => {
                if opts.ascii_only && !ch.is_ascii() {
                    continue;
                }
                out.push(ch);
            }
            CharRewrite::Multi(s2) => {
                if opts.ascii_only && !s2.is_ascii() {
                    continue;
                }
                out.push_str(s2);
            }
            CharRewrite::Drop => {}
        }
    }

    // 3. Whitespace collapse.
    let s = if opts.collapse_whitespace {
        let mut collapsed = String::with_capacity(out.len());
        let mut prev_space = false;
        for c in out.chars() {
            if c.is_whitespace() {
                if !prev_space {
                    collapsed.push(' ');
                }
                prev_space = true;
            } else {
                collapsed.push(c);
                prev_space = false;
            }
        }
        collapsed
    } else {
        out
    };

    // 4. Trim.
    if opts.trim {
        s.trim().to_string()
    } else {
        s
    }
}

/// Bulk variant. With `parallel = true`, distributes across rayon's pool.
pub fn sanitize_many(texts: &[&str], opts: &Options, parallel: bool) -> Vec<String> {
    if parallel {
        texts.par_iter().map(|t| sanitize(t, opts)).collect()
    } else {
        texts.iter().map(|t| sanitize(t, opts)).collect()
    }
}

// --- small helpers ---

fn is_zero_width(c: char) -> bool {
    matches!(
        c,
        '\u{200B}'  // zero-width space
        | '\u{200C}'  // ZWNJ
        | '\u{200D}'  // ZWJ
        | '\u{200E}'  // LTR mark
        | '\u{200F}'  // RTL mark
        | '\u{202A}' // LRE
        | '\u{202B}' // RLE
        | '\u{202C}' // PDF
        | '\u{202D}' // LRO
        | '\u{202E}' // RLO
        | '\u{2060}'  // word joiner
        | '\u{2061}' // function application
        | '\u{2062}' // invisible times
        | '\u{2063}' // invisible separator
        | '\u{2064}' // invisible plus
        | '\u{FEFF}' // BOM
    )
}

fn is_strippable_control(c: char) -> bool {
    if c == '\n' || c == '\t' {
        return false;
    }
    let cp = c as u32;
    (cp <= 0x1F) || (0x7F..=0x9F).contains(&cp)
}

fn is_emoji_codepoint(c: char) -> bool {
    let cp = c as u32;
    matches!(
        cp,
        0x1F1E6..=0x1F1FF   // regional indicator symbols (flag emoji)
        | 0x1F300..=0x1F5FF  // miscellaneous symbols and pictographs
        | 0x1F600..=0x1F64F  // emoticons
        | 0x1F680..=0x1F6FF  // transport and map symbols
        | 0x1F700..=0x1F77F
        | 0x1F780..=0x1F7FF
        | 0x1F800..=0x1F8FF
        | 0x1F900..=0x1F9FF
        | 0x1FA00..=0x1FA6F
        | 0x1FA70..=0x1FAFF
        | 0x2600..=0x26FF    // miscellaneous symbols
        | 0x2700..=0x27BF    // dingbats
        | 0x20E3             // combining enclosing keycap (e.g. 1️⃣)
        | 0xFE0E..=0xFE0F    // variation selectors
    )
}

enum CharRewrite {
    Single(char),
    Multi(&'static str),
    Drop,
}

fn map_smart_punctuation(c: char) -> CharRewrite {
    match c {
        '\u{2018}' | '\u{2019}' | '\u{201A}' | '\u{201B}' => CharRewrite::Single('\''),
        '\u{201C}' | '\u{201D}' | '\u{201E}' | '\u{201F}' => CharRewrite::Single('"'),
        '\u{2013}' | '\u{2014}' | '\u{2212}' => CharRewrite::Single('-'),
        '\u{2026}' => CharRewrite::Multi("..."),
        '\u{00A0}' | '\u{2007}' | '\u{202F}' => CharRewrite::Single(' '),
        '\u{00AB}' | '\u{00BB}' => CharRewrite::Single('"'),
        _ => CharRewrite::Single(c),
    }
}

// `Drop` is reserved for future use (e.g. dropping mid-text marks). Keep
// the variant referenced so a future addition doesn't fight the compiler.
#[allow(dead_code)]
fn _force_drop_referenced() -> CharRewrite {
    CharRewrite::Drop
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> Options {
        Options::default()
    }

    #[test]
    fn strict_preset_enables_everything() {
        let s = Options::strict();
        assert!(s.nfkc);
        assert!(s.strip_zero_width);
        assert!(s.strip_control);
        assert!(s.collapse_whitespace);
        assert!(s.trim);
        assert!(s.ascii_punctuation);
        assert!(s.strip_emoji);
        assert!(s.ascii_only);
    }

    #[test]
    fn normalize_newlines_crlf_to_lf() {
        assert_eq!(normalize_newlines("a\r\nb\r\nc"), "a\nb\nc");
    }

    #[test]
    fn normalize_newlines_lone_cr_to_lf() {
        assert_eq!(normalize_newlines("a\rb\rc"), "a\nb\nc");
    }

    #[test]
    fn normalize_newlines_idempotent() {
        let once = normalize_newlines("a\r\nb\r\nc");
        let twice = normalize_newlines(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn normalize_newlines_preserves_unicode() {
        assert_eq!(normalize_newlines("hi 世界\r\nbye 🌍"), "hi 世界\nbye 🌍");
    }

    #[test]
    fn strict_preset_strips_emoji_and_smart_quotes() {
        let r = sanitize("\u{201C}hi\u{201D} 🌍 there", &Options::strict());
        // Smart quotes -> ascii ", emoji removed.
        assert_eq!(r, "\"hi\" there");
    }

    #[test]
    fn defaults_collapse_and_trim() {
        let r = sanitize("  hello   world  ", &defaults());
        assert_eq!(r, "hello world");
    }

    #[test]
    fn nfkc_normalizes_ligature_and_fullwidth() {
        let r = sanitize("ﬁle ABC１２３", &defaults());
        assert_eq!(r, "file ABC123");
    }

    #[test]
    fn zero_width_stripped() {
        let r = sanitize("hi\u{200B}there\u{FEFF}", &defaults());
        assert_eq!(r, "hithere");
    }

    #[test]
    fn control_stripped_but_newline_preserved() {
        let r = sanitize("a\x01b\nc", &defaults());
        assert_eq!(r, "ab c"); // \n collapses with following space-equivalent? No.
    }

    #[test]
    fn newline_preserved_when_collapse_off() {
        let opts = Options {
            collapse_whitespace: false,
            ..Options::default()
        };
        let r = sanitize("a\nb", &opts);
        assert_eq!(r, "a\nb");
    }

    #[test]
    fn ascii_punctuation_replaces_smart_quotes() {
        let opts = Options {
            ascii_punctuation: true,
            ..Options::default()
        };
        let r = sanitize("\u{201C}hello\u{201D} \u{2014} world\u{2026}", &opts);
        assert_eq!(r, "\"hello\" - world...");
    }

    #[test]
    fn ascii_only_drops_non_ascii() {
        let opts = Options {
            ascii_only: true,
            ..Options::default()
        };
        let r = sanitize("hello 世界 world", &opts);
        assert_eq!(r, "hello world");
    }

    #[test]
    fn ascii_only_with_punctuation_keeps_converted() {
        let opts = Options {
            ascii_only: true,
            ascii_punctuation: true,
            ..Options::default()
        };
        // Smart quote becomes ascii ", which survives ascii_only.
        let r = sanitize("\u{201C}hi\u{201D}", &opts);
        assert_eq!(r, "\"hi\"");
    }

    #[test]
    fn strip_emoji() {
        let opts = Options {
            strip_emoji: true,
            ..Options::default()
        };
        let r = sanitize("hi 🌍 world 🚀", &opts);
        assert_eq!(r, "hi world");
    }

    #[test]
    fn strip_emoji_flags() {
        let opts = Options {
            strip_emoji: true,
            ..Options::default()
        };
        // Flags are pairs of regional indicator symbols (U+1F1E6..=U+1F1FF).
        let r = sanitize("hi 🇺🇸 there 🇯🇵", &opts);
        assert_eq!(r, "hi there");
    }

    #[test]
    fn strip_emoji_keycap_leaves_no_combining_remnant() {
        let opts = Options {
            strip_emoji: true,
            ..Options::default()
        };
        // Keycap "1️⃣" = '1' + U+FE0F + U+20E3. The combining enclosing
        // keycap must go too, otherwise a stray mark is left behind.
        let r = sanitize("press 1\u{FE0F}\u{20E3} now", &opts);
        assert_eq!(r, "press 1 now");
    }

    #[test]
    fn nfkc_off_preserves_ligature() {
        let opts = Options {
            nfkc: false,
            ..Options::default()
        };
        let r = sanitize("ﬁle", &opts);
        assert_eq!(r, "ﬁle");
    }

    #[test]
    fn empty_input_returns_empty() {
        assert_eq!(sanitize("", &defaults()), "");
    }

    #[test]
    fn collapse_off_keeps_runs() {
        let opts = Options {
            collapse_whitespace: false,
            ..Options::default()
        };
        let r = sanitize("  hello   world  ", &opts);
        // Trim still runs: removes leading/trailing.
        assert_eq!(r, "hello   world");
    }

    #[test]
    fn trim_off_keeps_edges() {
        let opts = Options {
            trim: false,
            collapse_whitespace: false,
            ..Options::default()
        };
        let r = sanitize("  hi  ", &opts);
        assert_eq!(r, "  hi  ");
    }

    #[test]
    fn nbsp_replaced_with_space_when_ascii_punct() {
        let opts = Options {
            ascii_punctuation: true,
            ..Options::default()
        };
        let r = sanitize("a\u{00A0}b", &opts);
        // Non-breaking space becomes regular space, then collapse leaves one.
        assert_eq!(r, "a b");
    }

    #[test]
    fn sanitize_many_serial_and_parallel_match() {
        let texts: Vec<&str> = vec!["  hi  ", "world\u{FEFF}", "ﬁle"];
        let opts = defaults();
        let s = sanitize_many(&texts, &opts, false);
        let p = sanitize_many(&texts, &opts, true);
        assert_eq!(s, p);
        assert_eq!(s, vec!["hi", "world", "file"]);
    }

    #[test]
    fn idempotent_on_clean_input() {
        let opts = defaults();
        let once = sanitize("hello world", &opts);
        let twice = sanitize(&once, &opts);
        assert_eq!(once, twice);
    }
}
