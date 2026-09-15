mod roman;

use roman::RomanError;
use std::env;
use std::io::{self, BufRead};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    match run(&args) {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("{}", msg);
            ExitCode::FAILURE
        }
    }
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let mut json = false;
    let mut lenient = false;
    let mut command: Option<&str> = None;
    let mut operand: Option<&str> = None;

    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
            "--lenient" => lenient = true,
            "-h" | "--help" => {
                println!("{}", usage_text());
                return Ok(ExitCode::SUCCESS);
            }
            other if command.is_none() => command = Some(other),
            other if operand.is_none() => operand = Some(other),
            other => return Err(format!("unexpected argument '{}'\n\n{}", other, usage_text())),
        }
    }

    let command = command.ok_or_else(usage_error)?;
    let operand = operand.ok_or_else(usage_error)?;

    match command {
        "parse" => Ok(run_parse(operand, json, lenient)),
        "format" if lenient => Err(format!(
            "--lenient only applies to 'parse'; 'format' always produces canonical output\n\n{}",
            usage_text()
        )),
        "format" => Ok(run_format(operand, json)),
        other => Err(format!("unknown command '{}'\n\n{}", other, usage_text())),
    }
}

fn run_parse(operand: &str, json: bool, lenient: bool) -> ExitCode {
    run_over_inputs(operand, |input| parse_one(input, json, lenient))
}

fn run_format(operand: &str, json: bool) -> ExitCode {
    run_over_inputs(operand, |input| format_one(input, json))
}

/// Runs `process` over a single operand, or, when the operand is `-`, over
/// every non-blank line read from stdin. Batch mode keeps going after a
/// failed line (so one bad row in a large input doesn't hide the rest of
/// the results) but still reports overall failure if any line failed.
fn run_over_inputs(operand: &str, mut process: impl FnMut(&str) -> bool) -> ExitCode {
    if operand != "-" {
        return if process(operand) {
            ExitCode::SUCCESS
        } else {
            ExitCode::FAILURE
        };
    }

    let stdin = io::stdin();
    let mut all_ok = true;
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(e) => {
                eprintln!("error reading stdin: {}", e);
                return ExitCode::FAILURE;
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !process(trimmed) {
            all_ok = false;
        }
    }
    if all_ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn parse_one(input: &str, json: bool, lenient: bool) -> bool {
    let result = if lenient {
        roman::parse_lenient(input)
    } else {
        roman::parse(input)
    };
    match result {
        Ok(value) => {
            if json {
                println!(
                    "{{\"input\":\"{}\",\"valid\":true,\"value\":{}}}",
                    json_escape(input),
                    value
                );
            } else {
                println!("{} = {}", input, value);
            }
            true
        }
        Err(e) => {
            if json {
                println!(
                    "{{\"input\":\"{}\",\"valid\":false,\"error\":\"{}\"}}",
                    json_escape(input),
                    json_escape(&e.to_string())
                );
            } else {
                eprintln!("error: {}", e);
            }
            false
        }
    }
}

fn format_one(input: &str, json: bool) -> bool {
    let result = parse_number(input).and_then(roman::to_roman);
    match result {
        Ok(numeral) => {
            if json {
                println!(
                    "{{\"input\":\"{}\",\"valid\":true,\"roman\":\"{}\"}}",
                    json_escape(input),
                    numeral
                );
            } else {
                println!("{} = {}", input, numeral);
            }
            true
        }
        Err(e) => {
            if json {
                println!(
                    "{{\"input\":\"{}\",\"valid\":false,\"error\":\"{}\"}}",
                    json_escape(input),
                    json_escape(&e.to_string())
                );
            } else {
                eprintln!("error: {}", e);
            }
            false
        }
    }
}

fn parse_number(input: &str) -> Result<u32, RomanError> {
    let n: i64 = input
        .trim()
        .parse()
        .map_err(|_| RomanError::NotANumber(input.to_string()))?;
    if n < roman::MIN_VALUE as i64 || n > roman::MAX_VALUE as i64 {
        return Err(RomanError::OutOfRange(n));
    }
    Ok(n as u32)
}

fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

fn usage_error() -> String {
    format!("missing arguments\n\n{}", usage_text())
}

fn usage_text() -> String {
    "usage:\n  numerus parse <NUMERAL|-> [--json] [--lenient]\n  numerus format <NUMBER|-> [--json]\n\n\
     examples:\n  numerus parse XIV\n  numerus format 1994 --json\n  numerus parse IIII --lenient\n\n\
     use '-' in place of the operand to read one numeral or number per line\n\
     from stdin; the exit code is nonzero if any line fails\n\n\
     --lenient relaxes 'parse' to accept legacy non-canonical forms like\n\
     'IIII' or 'VV', still rejecting invalid characters and out-of-range\n\
     values; it has no effect on 'format', which always produces canonical\n\
     output"
        .to_string()
}
