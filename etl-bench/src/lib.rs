//! Synthetic CSV fixture generator shared by criterion benches.
//!
//! `synth_csv(n)` returns `n` data rows plus the header, in the exact format
//! the [`etl_core::run`] pipeline accepts.

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Produce `n` rows of synthetic input CSV.
///
/// Output is `id,name,age` header followed by `n` rows. Ages cycle through
/// `[18, 78)` to keep rows non-degenerate without making the synthesizer
/// stateful.
#[must_use]
#[allow(clippy::cast_possible_truncation)] // `i % 60` is bounded to [0, 60)
pub fn synth_csv(n: usize) -> Vec<u8> {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(n * 16 + 16);
    out.push_str("id,name,age\n");
    for i in 0..n {
        let age: u32 = 18 + (i % 60) as u32;
        // Writing to a String never errors.
        let _ = writeln!(out, "{i},name{i},{age}");
    }
    out.into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synth_csv_has_header_plus_n_rows() {
        let bytes = synth_csv(5);
        let s = std::str::from_utf8(&bytes).unwrap();
        let lines: Vec<&str> = s.lines().collect();
        assert_eq!(lines.len(), 6);
        assert_eq!(lines[0], "id,name,age");
    }

    #[test]
    fn synth_csv_zero_rows_has_only_header() {
        let bytes = synth_csv(0);
        let s = std::str::from_utf8(&bytes).unwrap();
        assert_eq!(s.lines().count(), 1);
    }
}
