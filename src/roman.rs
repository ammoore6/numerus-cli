use std::fmt;

/// Valid roman numerals fall in this range; there is no zero and no
/// standard way to write anything past 3999 with the seven basic symbols.
pub const MIN_VALUE: u32 = 1;
pub const MAX_VALUE: u32 = 3999;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RomanError {
    Empty,
    InvalidChar(char, usize),
    NotCanonical { input: String, canonical: String },
    NotANumber(String),
    OutOfRange(i64),
}

impl fmt::Display for RomanError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RomanError::Empty => write!(f, "input is empty"),
            RomanError::InvalidChar(c, pos) => {
                write!(f, "invalid character '{}' at position {}", c, pos)
            }
            RomanError::NotCanonical { input, canonical } => write!(
                f,
                "'{}' is not a canonical roman numeral (did you mean '{}'?)",
                input, canonical
            ),
            RomanError::NotANumber(s) => write!(f, "'{}' is not an integer", s),
            RomanError::OutOfRange(n) => {
                write!(f, "{} is outside the representable range {}..={}", n, MIN_VALUE, MAX_VALUE)
            }
        }
    }
}

impl std::error::Error for RomanError {}

fn symbol_value(c: char) -> Option<u32> {
    match c {
        'I' => Some(1),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        'D' => Some(500),
        'M' => Some(1000),
        _ => None,
    }
}

/// Parses a roman numeral, accepting only its canonical form.
///
/// Rather than hand-coding every repetition and ordering rule, this sums the
/// numeral the standard way and then re-renders that sum with `to_roman`.
/// Any input that isn't already the canonical spelling of its own value
/// (e.g. "IIII", "VV", "IXIV") fails to round-trip and is rejected.
pub fn parse(input: &str) -> Result<u32, RomanError> {
    if input.is_empty() {
        return Err(RomanError::Empty);
    }

    let chars: Vec<char> = input.chars().collect();
    for (i, c) in chars.iter().enumerate() {
        if symbol_value(*c).is_none() {
            return Err(RomanError::InvalidChar(*c, i));
        }
    }

    let mut total: u32 = 0;
    let mut i = 0;
    while i < chars.len() {
        let value = symbol_value(chars[i]).unwrap();
        if i + 1 < chars.len() {
            let next = symbol_value(chars[i + 1]).unwrap();
            if next > value {
                total += next - value;
                i += 2;
                continue;
            }
        }
        total += value;
        i += 1;
    }

    if total < MIN_VALUE || total > MAX_VALUE {
        return Err(RomanError::OutOfRange(total as i64));
    }

    let canonical = to_roman(total).expect("total was already range-checked");
    if canonical == input {
        Ok(total)
    } else {
        Err(RomanError::NotCanonical {
            input: input.to_string(),
            canonical,
        })
    }
}

/// Renders an integer in the range 1..=3999 as a canonical roman numeral.
pub fn to_roman(mut n: u32) -> Result<String, RomanError> {
    if n < MIN_VALUE || n > MAX_VALUE {
        return Err(RomanError::OutOfRange(n as i64));
    }

    const TABLE: [(u32, &str); 13] = [
        (1000, "M"),
        (900, "CM"),
        (500, "D"),
        (400, "CD"),
        (100, "C"),
        (90, "XC"),
        (50, "L"),
        (40, "XL"),
        (10, "X"),
        (9, "IX"),
        (5, "V"),
        (4, "IV"),
        (1, "I"),
    ];

    let mut out = String::new();
    for &(value, symbol) in TABLE.iter() {
        while n >= value {
            out.push_str(symbol);
            n -= value;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_known_values() {
        let cases = [
            (1, "I"),
            (4, "IV"),
            (9, "IX"),
            (14, "XIV"),
            (40, "XL"),
            (90, "XC"),
            (444, "CDXLIV"),
            (1994, "MCMXCIV"),
            (3999, "MMMCMXCIX"),
        ];
        for (n, s) in cases {
            assert_eq!(to_roman(n).unwrap(), s);
            assert_eq!(parse(s).unwrap(), n);
        }
    }

    #[test]
    fn rejects_non_canonical_forms() {
        assert!(matches!(parse("IIII"), Err(RomanError::NotCanonical { .. })));
        assert!(matches!(parse("VV"), Err(RomanError::NotCanonical { .. })));
        assert!(matches!(parse("IXIV"), Err(RomanError::NotCanonical { .. })));
        assert!(matches!(parse("IC"), Err(RomanError::NotCanonical { .. })));
    }

    #[test]
    fn rejects_bad_characters_and_ranges() {
        assert!(matches!(parse(""), Err(RomanError::Empty)));
        assert!(matches!(parse("xiv"), Err(RomanError::InvalidChar('x', 0))));
        assert!(matches!(parse("MMMM"), Err(RomanError::OutOfRange(4000))));
        assert!(to_roman(0).is_err());
        assert!(to_roman(4000).is_err());
    }
}
