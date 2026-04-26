# etl-core

Pure-ish ETL core: read typed CSV rows, validate against a row schema, write
JSON Lines records to a sink, accumulate a row-aligned `Report`.

This crate has no IO of its own — `run` takes `Read` and `Write` so the binary
in `etl-cli` (or a benchmark in `etl-bench`) can drive it from anything that
implements those traits.

## Public API

```rust
use etl_core::{run, Report, EtlError};
use std::io::Cursor;

let input = b"id,name,age\n1,Ada,42\n";
let mut output = Vec::<u8>::new();
let report: Report = run(Cursor::new(&input[..]), &mut output).unwrap();
assert_eq!(report.rows_in, 1);
assert_eq!(report.rows_out, 1);
```
