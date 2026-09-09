# numerus

A command-line roman numeral parser and pretty printer.

Most "roman numeral" code you find online is loose about what counts as
valid input. It'll happily accept `IIII` or `VV` or `IXIV` because it just
sums symbol values left to right without checking that the result is the
*canonical* spelling of that number. `numerus` treats a numeral as invalid
unless it's the one and only correct way to write its value: `IV` is
accepted, `IIII` is not.

## Usage

Parse a numeral into its integer value:

```
$ numerus parse XIV
XIV = 14

$ numerus parse IIII
error: 'IIII' is not a canonical roman numeral (did you mean 'IV'?)
```

Format an integer as a numeral:

```
$ numerus format 1994
1994 = MCMXCIV

$ numerus format 4000
error: 4000 is outside the representable range 1..=3999
```

Both commands accept `--json` for machine-readable output, on both the
success and failure paths:

```
$ numerus parse XIV --json
{"input":"XIV","valid":true,"value":14}

$ numerus parse IIII --json
{"input":"IIII","valid":false,"error":"'IIII' is not a canonical roman numeral (did you mean 'IV'?)"}

$ numerus format 1994 --json
{"input":"1994","valid":true,"roman":"MCMXCIV"}
```

The exit code is 0 when the input is valid and 1 otherwise, regardless of
output mode, so `numerus parse "$X" --json` is safe to use in a pipeline
that also wants to `jq` the result.

Only values 1 through 3999 have a standard roman numeral form (there's no
symbol for zero, and repeating `M` more than three times isn't canonical),
so both commands reject anything outside that range.

## Building

Requires a stable Rust toolchain. No external dependencies.

```
cargo build --release
./target/release/numerus parse MCMXCIV
```

`cargo test` runs both the unit tests in `src/roman.rs` and the integration
tests in `tests/cli.rs`, which build the binary and shell out to it to check
the actual command-line behavior (exit codes, stdout/stderr, `--json`).

## How validation works

Rather than hand-coding every repetition and ordering rule (no more than
three of the same symbol in a row, only specific subtractive pairs, etc.),
the parser sums the numeral the usual way and then re-renders that sum with
the same logic the pretty printer uses. If the input doesn't match its own
canonical re-rendering, it's rejected. This means the parser and printer
can never disagree with each other about what "valid" means.
