//! `etl` — reference CLI for the `etl-core` crate.
//!
//! Reads CSV from `--input` (path or `-` for stdin), writes JSON Lines to
//! `--output` (path or `-` for stdout), and prints the report
//! (`etl_core::Report`) to stderr at end of run.
//!
//! ## Provable contracts
//!
//! - `ROWS_IN_EQUALS_ROWS_OUT` — `rows_in == rows_out + rows_rejected`.
//! - `REPORT_JSON_ROUNDTRIPS` — the report serializes and parses back as the
//!   same value.

#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use clap::Parser;
use etl_core::{run, Report};

/// CLI front-end for the typed CSV → JSON Lines ETL pipeline.
#[derive(Debug, Parser)]
#[command(
    name = "etl",
    version,
    about = "Typed CSV → JSON Lines ETL — course c9: Shipping Rust",
    long_about = None,
)]
struct Args {
    /// Input CSV file. Use `-` for stdin (the default).
    #[arg(short, long)]
    input: Option<String>,

    /// Output JSON Lines file. Use `-` for stdout (the default).
    #[arg(short, long)]
    output: Option<String>,
}

impl Args {
    fn input_spec(&self) -> &str {
        self.input.as_deref().unwrap_or("-")
    }
    fn output_spec(&self) -> &str {
        self.output.as_deref().unwrap_or("-")
    }
}

fn open_input(spec: &str) -> Result<Box<dyn Read>> {
    if spec == "-" {
        Ok(Box::new(BufReader::new(io::stdin())))
    } else {
        let path = PathBuf::from(spec);
        let file =
            File::open(&path).with_context(|| format!("opening input file {}", path.display()))?;
        Ok(Box::new(BufReader::new(file)))
    }
}

fn open_output(spec: &str) -> Result<Box<dyn Write>> {
    if spec == "-" {
        Ok(Box::new(BufWriter::new(io::stdout())))
    } else {
        let path = Path::new(spec);
        let file = File::create(path)
            .with_context(|| format!("creating output file {}", path.display()))?;
        Ok(Box::new(BufWriter::new(file)))
    }
}

/// Body of the program. Returns the [`Report`] so tests and the binary share
/// a single code path.
fn execute<R: Read, W: Write>(input: R, output: W) -> Result<Report> {
    let report = run(input, output).context("running ETL pipeline")?;
    Ok(report)
}

fn real_main(args: &Args) -> Result<ExitCode> {
    let input = open_input(args.input_spec())?;
    let mut output = open_output(args.output_spec())?;
    let report = execute(input, &mut output)?;
    output.flush().context("flushing output")?;

    // Provable contract: ROWS_IN_EQUALS_ROWS_OUT
    assert_eq!(report.rows_in, report.rows_out + report.rows_rejected);

    // Provable contract: REPORT_JSON_ROUNDTRIPS
    let report_json = serde_json::to_string(&report).context("serializing report")?;
    let parsed: Report = serde_json::from_str(&report_json).context("parsing report roundtrip")?;
    assert_eq!(parsed, report);

    let mut stderr = io::stderr();
    let _ = writeln!(stderr, "{report_json}");

    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    let args = Args::parse();
    match real_main(&args) {
        Ok(code) => code,
        Err(err) => {
            let mut stderr = io::stderr();
            let _ = writeln!(stderr, "etl: {err:#}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::path::PathBuf;

    fn unique_tmp(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "shipping-rust-test-{}-{}-{}",
            name,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |d| d.as_nanos())
        ));
        p
    }

    #[test]
    fn execute_happy_path() {
        let csv_in = b"id,fruit,weight_g\n1,apple,150\n";
        let mut sink = Vec::<u8>::new();
        let report = execute(Cursor::new(&csv_in[..]), &mut sink).unwrap();
        assert_eq!(report.rows_in, 1);
        assert_eq!(report.rows_out, 1);
        assert_eq!(report.rows_rejected, 0);
        assert!(String::from_utf8(sink).unwrap().contains("\"id\":1"));
    }

    #[test]
    fn execute_propagates_pipeline_error() {
        let mut sink = Vec::<u8>::new();
        let err = execute(Cursor::new(&b""[..]), &mut sink).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("running ETL pipeline"));
    }

    #[test]
    fn rows_in_equals_rows_out_contract_holds() {
        // Provable contract: ROWS_IN_EQUALS_ROWS_OUT
        let csv_in =
            b"id,fruit,weight_g\n1,apple,150\nbad_id,banana,118\n3,,77\n4,watermelon,7800\n";
        let mut sink = Vec::<u8>::new();
        let report = execute(Cursor::new(&csv_in[..]), &mut sink).unwrap();
        assert_eq!(report.rows_in, report.rows_out + report.rows_rejected);
    }

    #[test]
    fn report_json_roundtrip_contract_holds() {
        // Provable contract: REPORT_JSON_ROUNDTRIPS
        let csv_in = b"id,fruit,weight_g\n1,apple,150\n2,watermelon,7800\n";
        let mut sink = Vec::<u8>::new();
        let report = execute(Cursor::new(&csv_in[..]), &mut sink).unwrap();
        let report_json = serde_json::to_string(&report).unwrap();
        let parsed: Report = serde_json::from_str(&report_json).unwrap();
        assert_eq!(parsed, report);
    }

    #[test]
    fn open_input_stdin_alias_returns_reader() {
        // We can't actually read from stdin in a test, but we can confirm
        // the `-` branch builds a Read-trait object without erroring.
        let _r = open_input("-").unwrap();
    }

    #[test]
    fn open_input_missing_file_errors() {
        let result = open_input("/no/such/file/at/all/please.csv");
        // Use `is_err()` first to avoid an unreachable `panic!` arm that
        // would only execute on test failure (and so never get covered).
        assert!(result.is_err(), "expected error");
        let err = result.err().unwrap();
        let msg = format!("{err:#}");
        assert!(msg.contains("opening input file"));
    }

    #[test]
    fn open_input_reads_real_file() {
        let path = unique_tmp("input");
        std::fs::write(&path, b"id,fruit,weight_g\n1,apple,150\n").unwrap();
        let mut r = open_input(path.to_str().unwrap()).unwrap();
        let mut buf = String::new();
        r.read_to_string(&mut buf).unwrap();
        assert!(buf.contains("apple"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn open_output_stdout_alias_returns_writer() {
        let _w = open_output("-").unwrap();
    }

    #[test]
    fn open_output_creates_file() {
        let path = unique_tmp("output");
        {
            let mut w = open_output(path.to_str().unwrap()).unwrap();
            w.write_all(b"hi").unwrap();
            w.flush().unwrap();
        }
        let bytes = std::fs::read(&path).unwrap();
        assert_eq!(bytes, b"hi");
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn open_output_unwritable_path_errors() {
        // A path under a non-existent directory cannot be created.
        let result = open_output("/no/such/dir/ever/output.jsonl");
        assert!(result.is_err(), "expected error");
        let err = result.err().unwrap();
        let msg = format!("{err:#}");
        assert!(msg.contains("creating output file"));
    }

    #[test]
    fn real_main_runs_with_files() {
        let in_path = unique_tmp("real-in");
        let out_path = unique_tmp("real-out");
        std::fs::write(
            &in_path,
            b"id,fruit,weight_g\n1,apple,150\n2,banana,118\nbad_id,cherry,8\n",
        )
        .unwrap();
        let args = Args {
            input: Some(in_path.to_string_lossy().into_owned()),
            output: Some(out_path.to_string_lossy().into_owned()),
        };
        let code = real_main(&args).unwrap();
        // ExitCode does not implement PartialEq; check via Debug formatting
        // (stable) which prints "ExitCode(unix_exit_status(0))".
        let dbg = format!("{code:?}");
        assert!(dbg.contains('0') || dbg.to_ascii_lowercase().contains("success"));
        let out = std::fs::read_to_string(&out_path).unwrap();
        assert_eq!(out.lines().count(), 2);
        let _ = std::fs::remove_file(&in_path);
        let _ = std::fs::remove_file(&out_path);
    }

    #[test]
    fn real_main_propagates_open_input_error() {
        let args = Args {
            input: Some("/no/such/file/please.csv".into()),
            output: Some("-".into()),
        };
        let err = real_main(&args).unwrap_err();
        let msg = format!("{err:#}");
        assert!(msg.contains("opening input file"));
    }

    #[test]
    fn args_default_specs_are_dash() {
        // Exercise the input_spec/output_spec defaults explicitly so coverage
        // sees both `Some` and `None` paths.
        let args = Args {
            input: None,
            output: None,
        };
        assert_eq!(args.input_spec(), "-");
        assert_eq!(args.output_spec(), "-");
    }
}
