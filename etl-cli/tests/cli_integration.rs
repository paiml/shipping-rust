//! Integration tests that spawn the `etl` binary so coverage covers `main()`,
//! the clap-derived `Args` parser, and the contract assertions.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

fn etl_binary() -> PathBuf {
    // CARGO_BIN_EXE_<name> is provided by cargo for integration tests of bin
    // crates (since 1.43). Falls back to a sibling lookup for older toolchains.
    if let Some(p) = std::env::var_os("CARGO_BIN_EXE_etl") {
        return PathBuf::from(p);
    }
    let mut p = std::env::current_exe().unwrap();
    p.pop();
    p.pop();
    p.push("etl");
    p
}

fn unique_tmp(name: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!(
        "shipping-rust-itest-{}-{}-{}",
        name,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    p
}

#[test]
fn binary_runs_with_stdin_and_files() {
    let in_path = unique_tmp("stdin-in");
    let out_path = unique_tmp("stdin-out");
    std::fs::write(&in_path, b"id,name,age\n1,Ada,42\n2,Grace,80\n").unwrap();

    let status = Command::new(etl_binary())
        .args([
            "--input",
            in_path.to_str().unwrap(),
            "--output",
            out_path.to_str().unwrap(),
        ])
        .status()
        .expect("spawning etl");
    assert!(status.success());
    let out = std::fs::read_to_string(&out_path).unwrap();
    assert_eq!(out.lines().count(), 2);
    let _ = std::fs::remove_file(&in_path);
    let _ = std::fs::remove_file(&out_path);
}

#[test]
fn binary_propagates_open_error() {
    let status = Command::new(etl_binary())
        .args(["--input", "/no/such/file/please.csv"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("spawning etl");
    assert!(!status.success());
}

#[test]
fn binary_runs_with_stdin_pipe() {
    let mut child = Command::new(etl_binary())
        .args(["--input", "-", "--output", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawning etl");
    {
        let stdin = child.stdin.as_mut().expect("stdin pipe");
        stdin
            .write_all(b"id,name,age\n1,Ada,42\n")
            .expect("writing stdin");
    }
    let output = child.wait_with_output().expect("waiting for etl");
    assert!(output.status.success(), "etl exited non-zero");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"id\":1"));
}

#[test]
fn binary_runs_with_default_args() {
    // Spawn with no `--input` / `--output` so the clap default values fire
    // (which feeds coverage of the derive expansion sites).
    let mut child = Command::new(etl_binary())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawning etl");
    {
        let stdin = child.stdin.as_mut().expect("stdin pipe");
        stdin
            .write_all(b"id,name,age\n42,Ada,28\n")
            .expect("writing stdin");
    }
    let output = child.wait_with_output().expect("waiting for etl");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("\"id\":42"));
}

#[test]
fn binary_help_prints_and_succeeds() {
    let output = Command::new(etl_binary())
        .arg("--help")
        .output()
        .expect("spawning etl --help");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("--input"));
    assert!(stdout.contains("--output"));
}
