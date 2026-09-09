use std::process::{Command, Output};

fn numerus(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_numerus"))
        .args(args)
        .output()
        .expect("failed to run numerus binary")
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
