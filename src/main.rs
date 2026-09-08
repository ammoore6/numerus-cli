mod roman;

use roman::RomanError;
use std::env;
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
    let mut command: Option<&str> = None;
    let mut operand: Option<&str> = None;

    for arg in args {
        match arg.as_str() {
            "--json" => json = true,
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
        "parse" => Ok(run_parse(operand, json)),
        "format" => Ok(run_format(operand, json)),
        other => Err(format!("unknown command '{}'\n\n{}", other, usage_text())),
    }
}

fn run_parse(input: &str, json: bool) -> ExitCode {
    match roman::parse(input) {
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
            ExitCode::SUCCESS
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
            ExitCode::FAILURE
        }
    }
}

fn run_format(input: &str, json: bool) -> ExitCode {
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
            ExitCode::SUCCESS
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
            ExitCode::FAILURE
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
    "usage:\n  numerus parse <NUMERAL> [--json]\n  numerus format <NUMBER> [--json]\n\nexamples:\n  numerus parse XIV\n  numerus format 1994 --json"
        .to_string()
}
