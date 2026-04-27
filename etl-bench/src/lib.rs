//! Synthetic CSV fixture generator shared by criterion benches.
//!
//! `synth_csv(n)` returns `n` data rows plus the header, in the exact format
//! the [`etl_core::run`] pipeline accepts.

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

/// Rotating pool of fruit names used by [`synth_csv`]. Cycled with
/// `i % POOL.len()` so the synthetic CSV reads as realistic, varied
/// produce data.
const FRUIT_POOL: &[&str] = &[
    "apple",
    "banana",
    "cherry",
    "date",
    "elderberry",
    "fig",
    "grape",
    "honeydew",
    "kiwi",
    "lemon",
    "mango",
    "nectarine",
    "orange",
    "papaya",
    "quince",
    "raspberry",
    "strawberry",
    "tangerine",
    "watermelon",
];

/// Produce `n` rows of synthetic input CSV.
///
/// Output is `id,fruit,weight_g` header followed by `n` rows. Fruit names
/// rotate through an internal 19-item pool (`i % POOL.len()`); weights cycle
/// through `[30, 900)` grams via `30 + (i * 17) % 870` so we exercise every
/// `SizeBucket` variant without making the synthesizer stateful.
#[must_use]
#[allow(clippy::cast_possible_truncation)] // bounded to [0, 870)
pub fn synth_csv(n: usize) -> Vec<u8> {
    use std::fmt::Write as _;
    let mut out = String::with_capacity(n * 24 + 24);
    out.push_str("id,fruit,weight_g\n");
    for i in 0..n {
        let fruit = FRUIT_POOL[i % FRUIT_POOL.len()];
        let weight_g: u32 = 30 + ((i.wrapping_mul(17)) % 870) as u32;
        // Writing to a String never errors.
        let _ = writeln!(out, "{i},{fruit},{weight_g}");
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
        assert_eq!(lines[0], "id,fruit,weight_g");
    }

    #[test]
    fn synth_csv_zero_rows_has_only_header() {
        let bytes = synth_csv(0);
        let s = std::str::from_utf8(&bytes).unwrap();
        assert_eq!(s.lines().count(), 1);
    }

    #[test]
    fn synth_csv_cycles_fruit_pool() {
        // The first row uses the first pool entry; row at `POOL.len()` wraps
        // back to the first entry. This exercises the modulo path explicitly.
        let n = FRUIT_POOL.len() + 1;
        let bytes = synth_csv(n);
        let s = std::str::from_utf8(&bytes).unwrap();
        let lines: Vec<&str> = s.lines().collect();
        // Line 0 is the header, line 1 is row 0 (apple), line `n` is row n-1.
        assert!(lines[1].contains(",apple,"));
        assert!(lines[FRUIT_POOL.len() + 1].contains(",apple,"));
    }
}
