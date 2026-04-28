//! # etl-core
//!
//! Typed CSV → JSON Lines ETL core for the shipping-rust reference workspace
//! (course c9: Shipping Rust — Cargo, CI, Benchmarks & Containers).
//!
//! The crate is intentionally small:
//!
//! - `RowSchema` (private) holds the required input columns (`id`, `fruit`,
//!   optional `weight_g`).
//! - [`Record`] is the typed output written as one JSON object per line.
//! - [`Report`] is the row-aligned summary every run produces.
//! - [`EtlError`] is the error enum (one variant per failure mode).
//! - [`run`] is the only public entry point.
//!
//! `run` is generic over `Read` and `Write` so the CLI, the benchmarks, and
//! the unit tests all share the exact same code path.

#![deny(unsafe_code)]
#![warn(missing_docs)]
// Tests legitimately use `unwrap()` / `expect()` / `panic!()` to fail fast on
// the unhappy path. Production code keeps those warnings active.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::collections::BTreeMap;
use std::io::{Read, Write};

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors emitted while running the pipeline.
///
/// Each variant maps to a single failure mode; the unit tests in this crate
/// exercise every variant so that `cargo llvm-cov --fail-under-lines 100`
/// stays satisfied.
#[derive(Debug, Error)]
pub enum EtlError {
    /// The CSV stream could not be parsed (malformed quotes, unterminated
    /// record, IO failure surfaced by the `csv` crate, etc.).
    #[error("csv parse error: {0}")]
    CsvParse(#[from] csv::Error),

    /// The header row was missing one of the required columns
    /// (`id`, `fruit`, `weight_g`).
    #[error("missing required column: {0}")]
    MissingColumn(String),

    /// The header row had no fields at all (empty input).
    #[error("empty header — expected id,fruit,weight_g")]
    EmptyHeader,

    /// Writing a record to the JSON Lines sink failed.
    #[error("io error writing output: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization of an output record failed. In practice this is
    /// unreachable for the [`Record`] shape, but the error path is part of the
    /// public surface so callers can match on it.
    #[error("json serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Categories used to describe rejected rows in the [`Report`].
///
/// `Display` of an [`ErrorKind`] is the stable string used as the key in
/// [`Report::errors_by_kind`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorKind {
    /// `id` could not be parsed as `u64`.
    InvalidId,
    /// `fruit` was empty after trimming.
    EmptyFruit,
    /// `weight_g` was present but not parseable as `u32`.
    InvalidWeight,
    /// The row had fewer columns than the header declared.
    ShortRow,
}

impl ErrorKind {
    /// Stable string label used as a map key in [`Report::errors_by_kind`].
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidId => "invalid_id",
            Self::EmptyFruit => "empty_fruit",
            Self::InvalidWeight => "invalid_weight",
            Self::ShortRow => "short_row",
        }
    }
}

impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Required-column locator inside the CSV header row.
///
/// Built once per `run` from the header record.
#[derive(Debug, Clone, Copy)]
struct RowSchema {
    id: usize,
    fruit: usize,
    weight_g: Option<usize>,
}

impl RowSchema {
    fn from_header(header: &csv::StringRecord) -> Result<Self, EtlError> {
        if header.is_empty() {
            return Err(EtlError::EmptyHeader);
        }

        let mut id: Option<usize> = None;
        let mut fruit: Option<usize> = None;
        let mut weight_g: Option<usize> = None;

        for (i, field) in header.iter().enumerate() {
            match field.trim().to_ascii_lowercase().as_str() {
                "id" => id = Some(i),
                "fruit" => fruit = Some(i),
                "weight_g" => weight_g = Some(i),
                _ => {}
            }
        }

        Ok(Self {
            id: id.ok_or_else(|| EtlError::MissingColumn("id".into()))?,
            fruit: fruit.ok_or_else(|| EtlError::MissingColumn("fruit".into()))?,
            weight_g,
        })
    }
}

/// Coarse size bucket attached to every output record, derived from
/// `weight_g`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum SizeBucket {
    /// `weight_g` was missing.
    Unknown,
    /// `weight_g < 100`.
    Small,
    /// `100 <= weight_g < 300`.
    Medium,
    /// `weight_g >= 300`.
    Large,
}

impl SizeBucket {
    /// Bucket for an optional weight in grams; `None` → [`SizeBucket::Unknown`].
    #[must_use]
    pub fn from_weight_g(weight_g: Option<u32>) -> Self {
        match weight_g {
            None => Self::Unknown,
            Some(w) if w < 100 => Self::Small,
            Some(w) if w < 300 => Self::Medium,
            Some(_) => Self::Large,
        }
    }
}

/// Typed output record. One [`Record`] becomes one JSON object on its own line.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    /// Stable numeric identifier, parsed from the CSV `id` column.
    pub id: u64,
    /// Trimmed fruit name from the `fruit` column.
    pub fruit: String,
    /// Categorized size, derived from the optional `weight_g` column.
    pub size_bucket: SizeBucket,
}

/// Row-aligned summary of one [`run`] invocation.
///
/// `rows_in == rows_out + rows_rejected` is the binary's primary
/// `Provable contract: ROWS_IN_EQUALS_ROWS_OUT` invariant.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Report {
    /// Number of CSV data rows observed (excludes header).
    pub rows_in: u64,
    /// Number of records successfully written to the sink.
    pub rows_out: u64,
    /// Number of rows the schema rejected and which were skipped.
    pub rows_rejected: u64,
    /// Per-error-kind reject counts. Keys come from [`ErrorKind::as_str`].
    pub errors_by_kind: BTreeMap<String, u64>,
}

impl Report {
    fn note_reject(&mut self, kind: ErrorKind) {
        self.rows_rejected += 1;
        *self.errors_by_kind.entry(kind.to_string()).or_insert(0) += 1;
    }
}

/// Run the ETL pipeline: read CSV from `input`, write JSON Lines to `output`,
/// return the [`Report`].
///
/// # Errors
///
/// - [`EtlError::EmptyHeader`] — the input has no header row at all.
/// - [`EtlError::MissingColumn`] — required column (`id` or `fruit`) is absent.
/// - [`EtlError::CsvParse`] — the CSV stream is malformed.
/// - [`EtlError::Io`] — writing to the sink fails.
/// - [`EtlError::Json`] — record serialization fails (in practice unreachable).
pub fn run<R: Read, W: Write>(input: R, mut output: W) -> Result<Report, EtlError> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(true)
        .flexible(true)
        .from_reader(input);

    let header = reader.headers()?.clone();
    let schema = RowSchema::from_header(&header)?;

    let mut report = Report::default();
    let mut record_buf = csv::StringRecord::new();

    while reader.read_record(&mut record_buf)? {
        report.rows_in += 1;
        process_row(&schema, &record_buf, &mut output, &mut report)?;
    }

    Ok(report)
}

/// Validate, transform, and emit a single CSV record.
///
/// On a structural reject (short row, bad id, empty fruit, bad weight) the
/// helper notes the rejection on `report` and returns `Ok(())`. On a successful
/// transform it writes one JSON Lines record to `output` and bumps
/// `report.rows_out`. Only true I/O or serializer failures bubble up.
fn process_row<W: Write>(
    schema: &RowSchema,
    record_buf: &csv::StringRecord,
    output: &mut W,
    report: &mut Report,
) -> Result<(), EtlError> {
    let max_idx = schema.weight_g.map_or(schema.fruit.max(schema.id), |w| {
        schema.id.max(schema.fruit).max(w)
    });
    if record_buf.len() <= max_idx {
        report.note_reject(ErrorKind::ShortRow);
        return Ok(());
    }

    let Some(id) = parse_id(record_buf.get(schema.id).unwrap_or(""), report) else {
        return Ok(());
    };
    let Some(fruit) = parse_fruit(record_buf.get(schema.fruit).unwrap_or(""), report) else {
        return Ok(());
    };
    let weight_str = schema.weight_g.and_then(|i| record_buf.get(i));
    let weight_g = match parse_weight_g(weight_str, report) {
        WeightOutcome::Reject => return Ok(()),
        WeightOutcome::Absent => None,
        WeightOutcome::Parsed(n) => Some(n),
    };

    let record = Record {
        id,
        fruit,
        size_bucket: SizeBucket::from_weight_g(weight_g),
    };
    let line = serde_json::to_string(&record)?;
    output.write_all(line.as_bytes())?;
    output.write_all(b"\n")?;
    report.rows_out += 1;
    Ok(())
}

/// Parse the `id` cell. Returns `None` and notes the rejection on bad input.
fn parse_id(cell: &str, report: &mut Report) -> Option<u64> {
    if let Ok(id) = cell.trim().parse::<u64>() {
        Some(id)
    } else {
        report.note_reject(ErrorKind::InvalidId);
        None
    }
}

/// Parse the `fruit` cell. Returns `None` and notes the rejection on empty input.
fn parse_fruit(cell: &str, report: &mut Report) -> Option<String> {
    let trimmed = cell.trim();
    if trimmed.is_empty() {
        report.note_reject(ErrorKind::EmptyFruit);
        None
    } else {
        Some(trimmed.to_string())
    }
}

/// Outcome of parsing the optional `weight_g` cell. Distinguishes "no value
/// provided" from "value provided and parsed" from "value provided but
/// malformed (rejected)".
enum WeightOutcome {
    Absent,
    Parsed(u32),
    Reject,
}

/// Parse the optional `weight_g` cell. On a malformed value the rejection is
/// noted on `report` and [`WeightOutcome::Reject`] is returned.
fn parse_weight_g(cell: Option<&str>, report: &mut Report) -> WeightOutcome {
    match cell.map(str::trim) {
        None | Some("") => WeightOutcome::Absent,
        Some(s) => {
            if let Ok(parsed) = s.parse::<u32>() {
                WeightOutcome::Parsed(parsed)
            } else {
                report.note_reject(ErrorKind::InvalidWeight);
                WeightOutcome::Reject
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor};

    fn run_str(csv_in: &str) -> (Report, String) {
        let mut sink = Vec::<u8>::new();
        let report = run(Cursor::new(csv_in.as_bytes()), &mut sink).expect("run failed");
        let out = String::from_utf8(sink).expect("non-utf8 output");
        (report, out)
    }

    #[test]
    fn happy_path_three_rows() {
        // apple → Medium (150 g), watermelon → Large (7800 g), grape → Small (5 g).
        let csv_in = "id,fruit,weight_g\n1,apple,150\n2,watermelon,7800\n3,grape,5\n";
        let (report, out) = run_str(csv_in);

        assert_eq!(report.rows_in, 3);
        assert_eq!(report.rows_out, 3);
        assert_eq!(report.rows_rejected, 0);
        assert!(report.errors_by_kind.is_empty());

        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(lines.len(), 3);
        assert!(lines[0].contains("\"size_bucket\":\"Medium\""));
        assert!(lines[1].contains("\"size_bucket\":\"Large\""));
        assert!(lines[2].contains("\"size_bucket\":\"Small\""));
    }

    #[test]
    fn missing_weight_column_yields_unknown_bucket() {
        let (report, out) = run_str("id,fruit\n7,banana\n");
        assert_eq!(report.rows_in, 1);
        assert_eq!(report.rows_out, 1);
        assert!(out.contains("\"size_bucket\":\"Unknown\""));
    }

    #[test]
    fn blank_weight_field_yields_unknown_bucket() {
        let (report, out) = run_str("id,fruit,weight_g\n7,banana,\n");
        assert_eq!(report.rows_out, 1);
        assert!(out.contains("\"size_bucket\":\"Unknown\""));
    }

    #[test]
    fn invalid_id_is_rejected() {
        let (report, out) = run_str("id,fruit,weight_g\nbad_id,apple,150\n");
        assert_eq!(report.rows_in, 1);
        assert_eq!(report.rows_out, 0);
        assert_eq!(report.rows_rejected, 1);
        assert_eq!(report.errors_by_kind.get("invalid_id").copied(), Some(1));
        assert!(out.is_empty());
    }

    #[test]
    fn empty_fruit_is_rejected() {
        let (report, _) = run_str("id,fruit,weight_g\n1,   ,150\n");
        assert_eq!(report.rows_rejected, 1);
        assert_eq!(report.errors_by_kind.get("empty_fruit").copied(), Some(1));
    }

    #[test]
    fn invalid_weight_is_rejected() {
        let (report, _) = run_str("id,fruit,weight_g\n1,apple,not_a_number\n");
        assert_eq!(report.rows_rejected, 1);
        assert_eq!(
            report.errors_by_kind.get("invalid_weight").copied(),
            Some(1)
        );
    }

    #[test]
    fn short_row_is_rejected() {
        // Only one column on the data row, but the header declares three.
        // `flexible(true)` lets the parser ignore the column-count mismatch
        // so we can flag it as ShortRow rather than failing the whole stream.
        let (report, _) = run_str("id,fruit,weight_g\n1\n");
        assert_eq!(report.rows_rejected, 1);
        assert_eq!(report.errors_by_kind.get("short_row").copied(), Some(1));
    }

    #[test]
    fn missing_required_column_errors() {
        let mut sink = Vec::<u8>::new();
        let err = run(Cursor::new(b"id,weight_g\n1,150\n"), &mut sink).unwrap_err();
        assert!(matches!(&err, EtlError::MissingColumn(c) if c == "fruit"));
    }

    #[test]
    fn missing_id_column_errors() {
        let mut sink = Vec::<u8>::new();
        let err = run(Cursor::new(b"fruit,weight_g\napple,150\n"), &mut sink).unwrap_err();
        assert!(matches!(&err, EtlError::MissingColumn(c) if c == "id"));
    }

    #[test]
    fn empty_header_errors() {
        // Truly empty input — no header at all.
        let mut sink = Vec::<u8>::new();
        let err = run(Cursor::new(b""), &mut sink).unwrap_err();
        assert!(matches!(err, EtlError::EmptyHeader));
    }

    #[test]
    fn malformed_csv_errors() {
        // Force a `csv::Error` by having the underlying `Read` itself fail.
        // (The csv crate is forgiving about quoted-field oddities when
        // `flexible(true)`, so the cleanest reproduction is an IO error.)
        let mut sink = Vec::<u8>::new();
        let err = run(FailingReader, &mut sink).unwrap_err();
        assert!(matches!(err, EtlError::CsvParse(_)));
    }

    #[test]
    fn unknown_header_column_is_ignored() {
        // Exercise the `_ => {}` arm of the header-column matcher.
        let (report, out) = run_str("id,fruit,weight_g,extra\n1,apple,150,trailing\n");
        assert_eq!(report.rows_in, 1);
        assert_eq!(report.rows_out, 1);
        assert!(out.contains("\"id\":1"));
    }

    /// A `Write` sink that fails on the first call, so we can exercise
    /// `EtlError::Io` without touching the filesystem.
    struct FailingWriter;
    impl Write for FailingWriter {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::BrokenPipe, "write closed"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn failing_writer_flush_is_noop() {
        // Coverage: ensure the flush method body is exercised.
        let mut w = FailingWriter;
        w.flush().unwrap();
    }

    /// A `Read` source that fails on the first call, so we can exercise
    /// `EtlError::CsvParse` (the csv crate wraps the io error).
    struct FailingReader;
    impl Read for FailingReader {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::ConnectionAborted,
                "read aborted",
            ))
        }
    }

    /// A `Serialize` impl that always returns a custom serde error. Used to
    /// build a real `serde_json::Error` value for the Display test.
    struct AlwaysFails;
    impl Serialize for AlwaysFails {
        fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("synthetic"))
        }
    }

    #[test]
    fn write_error_surfaces_as_io() {
        let err = run(
            Cursor::new(b"id,fruit,weight_g\n1,apple,150\n"),
            FailingWriter,
        )
        .unwrap_err();
        assert!(matches!(err, EtlError::Io(_)));
    }

    #[test]
    fn read_error_surfaces_as_csv_parse() {
        // The csv crate wraps io errors as csv::Error::Io; either way the
        // public-facing variant is CsvParse.
        let err = run(FailingReader, Vec::<u8>::new()).unwrap_err();
        assert!(matches!(err, EtlError::CsvParse(_)));
    }

    #[test]
    fn rows_in_equals_rows_out_plus_rejected() {
        // The binary-level provable contract — exercise it from the lib too.
        let csv_in =
            "id,fruit,weight_g\n1,apple,150\nbad_id,banana,118\n3,,77\n4,watermelon,7800\n";
        let (report, _) = run_str(csv_in);
        assert_eq!(report.rows_in, report.rows_out + report.rows_rejected);
    }

    #[test]
    fn error_kind_display_and_str_match() {
        for k in [
            ErrorKind::InvalidId,
            ErrorKind::EmptyFruit,
            ErrorKind::InvalidWeight,
            ErrorKind::ShortRow,
        ] {
            assert_eq!(format!("{k}"), k.as_str());
        }
    }

    #[test]
    fn size_bucket_boundaries() {
        // None → Unknown; <100 → Small; 100..299 → Medium; >=300 → Large.
        assert_eq!(SizeBucket::from_weight_g(None), SizeBucket::Unknown);
        assert_eq!(SizeBucket::from_weight_g(Some(0)), SizeBucket::Small);
        assert_eq!(SizeBucket::from_weight_g(Some(99)), SizeBucket::Small);
        assert_eq!(SizeBucket::from_weight_g(Some(100)), SizeBucket::Medium);
        assert_eq!(SizeBucket::from_weight_g(Some(299)), SizeBucket::Medium);
        assert_eq!(SizeBucket::from_weight_g(Some(300)), SizeBucket::Large);
        assert_eq!(SizeBucket::from_weight_g(Some(7800)), SizeBucket::Large);
    }

    #[test]
    fn etl_error_display_strings_are_stable() {
        // Construct each EtlError variant directly and exercise its Display.
        // CsvParse: build a synthetic csv::Error from an io::Error so we have
        // a stable, deterministic value (no need to drive the parser).
        let csv_err: csv::Error = csv::Error::from(io::Error::other("synthetic"));
        let csv_msg = format!("{}", EtlError::CsvParse(csv_err));
        assert!(csv_msg.contains("csv parse error"));

        assert_eq!(
            format!("{}", EtlError::MissingColumn("fruit".into())),
            "missing required column: fruit"
        );
        assert_eq!(
            format!("{}", EtlError::EmptyHeader),
            "empty header — expected id,fruit,weight_g"
        );
        let io_msg = format!("{}", EtlError::Io(io::Error::other("boom")));
        assert!(io_msg.contains("io error"));

        // Trigger a json error via the AlwaysFails Serialize defined above.
        let json_err = serde_json::to_string(&AlwaysFails).unwrap_err();
        let json_msg = format!("{}", EtlError::Json(json_err));
        assert!(json_msg.contains("json serialization"));
    }
}
