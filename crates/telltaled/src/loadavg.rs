//! Load-average collection — the cheapest first telemetry signal on Linux.
//!
//! The kernel exposes the 1/5/15-minute load averages as the first three
//! whitespace-separated fields of `/proc/loadavg`, e.g.
//!
//! ```text
//! 0.00 0.01 0.05 1/234 5678
//! ```
//!
//! Reading that file is I/O and lives in the binary shim; this module is the
//! pure, side-effect-free core: [`parse_loadavg`] turns the file's *contents*
//! into a [`LoadAvg`], so it is unit- and property-testable without touching the
//! real system — keeping with the **low host overhead** constraint (see
//! `AGENTS.md`). The trailing process/PID fields are deliberately ignored, so a
//! future kernel reshaping them does not break us.

use std::fmt;

/// The system load averages over the last 1, 5, and 15 minutes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LoadAvg {
    /// Load average over the last minute.
    pub one: f64,
    /// Load average over the last five minutes.
    pub five: f64,
    /// Load average over the last fifteen minutes.
    pub fifteen: f64,
}

impl fmt::Display for LoadAvg {
    /// One line, matching the source field order: `load 1m 5m 15m`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "load {:.2} {:.2} {:.2}",
            self.one, self.five, self.fifteen
        )
    }
}

/// The three load-average fields, in order, for error reporting.
const FIELD_NAMES: [&str; 3] = ["1-minute", "5-minute", "15-minute"];

/// Why a `/proc/loadavg` string could not be parsed into a [`LoadAvg`].
///
/// Malformed-input is read structurally: too few fields, or a field that is not
/// a number. (Real load averages are always finite and non-negative; rejecting
/// NaN/infinite/negative values is left as future hardening — those never appear
/// in `/proc/loadavg`.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Fewer than three whitespace-separated fields were present.
    /// `found` is how many were available.
    MissingField { found: usize },
    /// The field at `index` (0/1/2) was present but not a valid number.
    InvalidNumber { index: usize },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::MissingField { found } => {
                write!(f, "expected at least 3 load-average fields, found {found}")
            }
            Self::InvalidNumber { index } => {
                let name = FIELD_NAMES.get(index).copied().unwrap_or("load-average");
                write!(f, "{name} load average is not a valid number")
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse the contents of `/proc/loadavg` into the 1/5/15-minute load averages.
///
/// Pure: the input is the file's contents and the output is the three averages,
/// with no system access. Only the first three whitespace-separated fields are
/// read; any trailing fields (running/total processes, last PID) and surrounding
/// whitespace — including the trailing newline `read_to_string` returns — are
/// ignored.
///
/// # Errors
///
/// Returns [`ParseError::MissingField`] if fewer than three fields are present,
/// or [`ParseError::InvalidNumber`] if one of the first three is not a number.
pub fn parse_loadavg(contents: &str) -> Result<LoadAvg, ParseError> {
    let mut fields = contents.split_whitespace();
    let one = next_average(&mut fields, 0)?;
    let five = next_average(&mut fields, 1)?;
    let fifteen = next_average(&mut fields, 2)?;
    Ok(LoadAvg { one, five, fifteen })
}

/// Pull the next whitespace-separated field and parse it as a load average.
/// `index` (0/1/2) identifies the field for error reporting; if the field is
/// absent, `found` is the count of fields that *were* present before it.
fn next_average<'a>(
    fields: &mut impl Iterator<Item = &'a str>,
    index: usize,
) -> Result<f64, ParseError> {
    let token = fields
        .next()
        .ok_or(ParseError::MissingField { found: index })?;
    token
        .parse::<f64>()
        .map_err(|_| ParseError::InvalidNumber { index })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parses_a_known_good_line() {
        // The real shape the binary feeds the parser: trailing fields + newline.
        let got = parse_loadavg("0.00 0.01 0.05 1/234 5678\n").expect("valid input parses");
        assert_eq!(
            got,
            LoadAvg {
                one: 0.00,
                five: 0.01,
                fifteen: 0.05,
            }
        );
    }

    #[test]
    fn ignores_trailing_fields_and_extra_whitespace() {
        let got = parse_loadavg("  1.50   2.50  3.50  garbage here  ").expect("valid input parses");
        assert_eq!(got.one, 1.50);
        assert_eq!(got.five, 2.50);
        assert_eq!(got.fifteen, 3.50);
    }

    #[test]
    fn rejects_too_few_fields() {
        assert_eq!(
            parse_loadavg("0.00 0.01"),
            Err(ParseError::MissingField { found: 2 })
        );
        assert_eq!(
            parse_loadavg(""),
            Err(ParseError::MissingField { found: 0 })
        );
    }

    #[test]
    fn rejects_non_numeric_fields() {
        assert_eq!(
            parse_loadavg("x 0.01 0.05"),
            Err(ParseError::InvalidNumber { index: 0 })
        );
        assert_eq!(
            parse_loadavg("0.00 y 0.05"),
            Err(ParseError::InvalidNumber { index: 1 })
        );
        assert_eq!(
            parse_loadavg("0.00 0.01 z"),
            Err(ParseError::InvalidNumber { index: 2 })
        );
    }

    #[test]
    fn display_renders_one_line() {
        let la = LoadAvg {
            one: 0.0,
            five: 0.01,
            fifteen: 0.05,
        };
        assert_eq!(la.to_string(), "load 0.00 0.01 0.05");
    }

    proptest! {
        /// Robustness invariant: the parser never panics on arbitrary input.
        #[test]
        fn never_panics_on_arbitrary_input(s in any::<String>()) {
            let _ = parse_loadavg(&s);
        }

        /// Liveness: any well-formed line round-trips. This fails a vacuous
        /// "always error" parser, so the never-panic test can't be satisfied
        /// trivially. Values are limited to two decimals to mirror the kernel's
        /// own formatting; the comparison uses an epsilon to dodge f64 noise.
        #[test]
        fn round_trips_well_formed_lines(
            a in 0u32..100_000,
            b in 0u32..100_000,
            c in 0u32..100_000,
        ) {
            let (one, five, fifteen) = (f64::from(a) / 100.0, f64::from(b) / 100.0, f64::from(c) / 100.0);
            let line = format!("{one:.2} {five:.2} {fifteen:.2} 1/1 1\n");
            let got = parse_loadavg(&line).expect("well-formed line parses");
            prop_assert!((got.one - one).abs() < 1e-9);
            prop_assert!((got.five - five).abs() < 1e-9);
            prop_assert!((got.fifteen - fifteen).abs() < 1e-9);
        }
    }
}
