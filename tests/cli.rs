use std::io::Write;
use std::process::{Command, Output, Stdio};

fn numerus(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_numerus"))
        .args(args)
        .output()
        .expect("failed to run numerus binary")
}

fn numerus_with_stdin(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_numerus"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn numerus binary");
    child
        .stdin
        .take()
        .expect("child stdin was not piped")
        .write_all(input.as_bytes())
        .expect("failed to write to child stdin");
    child.wait_with_output().expect("failed to wait on child")
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).expect("stdout was not utf8")
}

fn stderr(out: &Output) -> String {
    String::from_utf8(out.stderr.clone()).expect("stderr was not utf8")
}

#[test]
fn parse_accepts_canonical_numeral() {
    let out = numerus(&["parse", "XIV"]);
    assert!(out.status.success());
    assert_eq!(stdout(&out), "XIV = 14\n");
}

#[test]
fn parse_rejects_non_canonical_numeral() {
    let out = numerus(&["parse", "IIII"]);
    assert!(!out.status.success());
    assert_eq!(out.status.code(), Some(1));
    assert!(stderr(&out).contains("did you mean 'IV'?"));
}

#[test]
fn parse_json_success_matches_expected_shape() {
    let out = numerus(&["parse", "XIV", "--json"]);
    assert!(out.status.success());
    assert_eq!(stdout(&out), "{\"input\":\"XIV\",\"valid\":true,\"value\":14}\n");
}

#[test]
fn parse_json_failure_still_exits_nonzero() {
    let out = numerus(&["parse", "IIII", "--json"]);
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(text.contains("\"valid\":false"));
    assert!(text.contains("IIII"));
    assert!(stderr(&out).is_empty());
}

#[test]
fn format_accepts_number_in_range() {
    let out = numerus(&["format", "1994"]);
    assert!(out.status.success());
    assert_eq!(stdout(&out), "1994 = MCMXCIV\n");
}

#[test]
fn format_rejects_out_of_range_number() {
    let out = numerus(&["format", "4000"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("outside the representable range"));
}

#[test]
fn format_rejects_non_numeric_input() {
    let out = numerus(&["format", "not-a-number"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("is not an integer"));
}

#[test]
fn missing_arguments_reports_usage_and_fails() {
    let out = numerus(&["parse"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("missing arguments"));
    assert!(stderr(&out).contains("usage:"));
}

#[test]
fn unknown_command_reports_usage_and_fails() {
    let out = numerus(&["frobnicate", "XIV"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("unknown command 'frobnicate'"));
}

#[test]
fn help_flag_prints_usage_and_succeeds() {
    let out = numerus(&["--help"]);
    assert!(out.status.success());
    assert!(stdout(&out).contains("usage:"));
}

#[test]
fn parse_batch_reads_one_numeral_per_stdin_line() {
    let out = numerus_with_stdin(&["parse", "-"], "XIV\nIX\nMCMXCIV\n");
    assert!(out.status.success());
    assert_eq!(stdout(&out), "XIV = 14\nIX = 9\nMCMXCIV = 1994\n");
}

#[test]
fn parse_batch_skips_blank_lines_and_keeps_going_after_a_failure() {
    let out = numerus_with_stdin(&["parse", "-"], "XIV\n\nIIII\nIX\n");
    assert!(!out.status.success());
    assert_eq!(stdout(&out), "XIV = 14\nIX = 9\n");
    assert!(stderr(&out).contains("did you mean 'IV'?"));
}

#[test]
fn lenient_accepts_legacy_forms_that_strict_rejects() {
    let out = numerus(&["parse", "IIII", "--lenient"]);
    assert!(out.status.success());
    assert_eq!(stdout(&out), "IIII = 4\n");
}

#[test]
fn lenient_still_rejects_invalid_characters() {
    let out = numerus(&["parse", "IIXZ", "--lenient"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("invalid character 'Z'"));
}

#[test]
fn lenient_flag_is_rejected_for_format() {
    let out = numerus(&["format", "1994", "--lenient"]);
    assert!(!out.status.success());
    assert!(stderr(&out).contains("--lenient only applies to 'parse'"));
}

#[test]
fn format_batch_reads_one_number_per_stdin_line_as_json() {
    let out = numerus_with_stdin(&["format", "-", "--json"], "1994\n4000\n");
    assert!(!out.status.success());
    let text = stdout(&out);
    assert!(text.contains("\"input\":\"1994\",\"valid\":true,\"roman\":\"MCMXCIV\""));
    assert!(text.contains("\"input\":\"4000\",\"valid\":false"));
}
