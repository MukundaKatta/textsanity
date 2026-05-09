//! PyO3 bindings exposing `textsanity_core` as `textsanity._native`.

use pyo3::prelude::*;

use textsanity_core::{sanitize, sanitize_many, Options};

#[pyfunction(name = "sanitize")]
#[pyo3(signature = (
    text,
    *,
    nfkc=true,
    strip_zero_width=true,
    strip_control=true,
    collapse_whitespace=true,
    trim=true,
    ascii_punctuation=false,
    strip_emoji=false,
    ascii_only=false,
))]
#[allow(clippy::too_many_arguments)]
fn py_sanitize(
    py: Python<'_>,
    text: &str,
    nfkc: bool,
    strip_zero_width: bool,
    strip_control: bool,
    collapse_whitespace: bool,
    trim: bool,
    ascii_punctuation: bool,
    strip_emoji: bool,
    ascii_only: bool,
) -> String {
    let opts = Options {
        nfkc,
        strip_zero_width,
        strip_control,
        collapse_whitespace,
        trim,
        ascii_punctuation,
        strip_emoji,
        ascii_only,
    };
    let owned = text.to_owned();
    py.allow_threads(move || sanitize(&owned, &opts))
}

#[pyfunction(name = "sanitize_many")]
#[pyo3(signature = (
    texts,
    *,
    parallel=false,
    nfkc=true,
    strip_zero_width=true,
    strip_control=true,
    collapse_whitespace=true,
    trim=true,
    ascii_punctuation=false,
    strip_emoji=false,
    ascii_only=false,
))]
#[allow(clippy::too_many_arguments)]
fn py_sanitize_many(
    py: Python<'_>,
    texts: Vec<String>,
    parallel: bool,
    nfkc: bool,
    strip_zero_width: bool,
    strip_control: bool,
    collapse_whitespace: bool,
    trim: bool,
    ascii_punctuation: bool,
    strip_emoji: bool,
    ascii_only: bool,
) -> Vec<String> {
    let opts = Options {
        nfkc,
        strip_zero_width,
        strip_control,
        collapse_whitespace,
        trim,
        ascii_punctuation,
        strip_emoji,
        ascii_only,
    };
    py.allow_threads(move || {
        let refs: Vec<&str> = texts.iter().map(String::as_str).collect();
        sanitize_many(&refs, &opts, parallel)
    })
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(py_sanitize, m)?)?;
    m.add_function(wrap_pyfunction!(py_sanitize_many, m)?)?;
    Ok(())
}
