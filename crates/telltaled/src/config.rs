//! Runtime configuration from environment variables (ADR 0004, #3).
//!
//! The only entry point for production use is [`parse_config`], a pure function
//! that turns a string map (typically from [`std::env::vars`]) into a validated
//! [`Config`]. All I/O stays in the binary shim; this module is unit-testable
//! without touching the real environment.

use std::collections::HashMap;
use std::fmt;

/// The default sampling interval when `TELLTALED_INTERVAL_SECS` is absent.
pub const DEFAULT_INTERVAL_SECS: u64 = 60;

/// Validated runtime configuration for the daemon.
#[derive(PartialEq)]
pub struct Config {
    /// Full push URL including `?identifier=secret` (from `TELLTALED_YGGIO_URL`).
    pub yggio_url: String,
    /// Per-host device secret (from `TELLTALED_YGGIO_SECRET`).
    pub yggio_secret: String,
    /// Sampling interval in seconds (from `TELLTALED_INTERVAL_SECS`; default 60).
    pub interval_secs: u64,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("yggio_url", &self.yggio_url)
            .field("yggio_secret", &"<redacted>")
            .field("interval_secs", &self.interval_secs)
            .finish()
    }
}

/// Why configuration could not be built from the supplied variables.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// A required variable is absent or empty.
    Missing {
        /// The variable name, e.g. `"TELLTALED_YGGIO_URL"`.
        var: &'static str,
    },
    /// `TELLTALED_INTERVAL_SECS` was present but not a valid positive integer.
    InvalidInterval {
        /// The raw value that could not be parsed.
        value: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { var } => write!(f, "required environment variable {var} is not set"),
            Self::InvalidInterval { value } => {
                write!(
                    f,
                    "TELLTALED_INTERVAL_SECS must be a positive integer, got {value:?}"
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// Parse and validate daemon configuration from a string-to-string variable map.
///
/// On success returns a fully-validated [`Config`]. On failure returns the first
/// validation error encountered. The caller is responsible for sourcing the map
/// (e.g. `std::env::vars().collect()`); this function is pure and side-effect-free.
///
/// # Errors
///
/// - [`ConfigError::Missing`] if `TELLTALED_YGGIO_URL` or `TELLTALED_YGGIO_SECRET`
///   is absent or empty.
/// - [`ConfigError::InvalidInterval`] if `TELLTALED_INTERVAL_SECS` is present but
///   not a valid `u64`.
pub fn parse_config(vars: &HashMap<String, String>) -> Result<Config, ConfigError> {
    let yggio_url = require(vars, "TELLTALED_YGGIO_URL")?;
    let yggio_secret = require(vars, "TELLTALED_YGGIO_SECRET")?;
    let interval_secs = optional_interval(vars)?;
    Ok(Config {
        yggio_url,
        yggio_secret,
        interval_secs,
    })
}

fn require(vars: &HashMap<String, String>, var: &'static str) -> Result<String, ConfigError> {
    vars.get(var)
        .filter(|v| !v.is_empty())
        .cloned()
        .ok_or(ConfigError::Missing { var })
}

fn optional_interval(vars: &HashMap<String, String>) -> Result<u64, ConfigError> {
    match vars.get("TELLTALED_INTERVAL_SECS") {
        None => Ok(DEFAULT_INTERVAL_SECS),
        Some(v) => v
            .parse::<u64>()
            .map_err(|_| ConfigError::InvalidInterval { value: v.clone() }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn full_map() -> HashMap<String, String> {
        map(&[
            (
                "TELLTALED_YGGIO_URL",
                "https://example.com/push?identifier=abc",
            ),
            ("TELLTALED_YGGIO_SECRET", "s3cr3t"),
            ("TELLTALED_INTERVAL_SECS", "30"),
        ])
    }

    #[test]
    fn parses_all_vars_present() {
        let cfg = parse_config(&full_map()).expect("valid input parses");
        assert_eq!(cfg.yggio_url, "https://example.com/push?identifier=abc");
        assert_eq!(cfg.yggio_secret, "s3cr3t");
        assert_eq!(cfg.interval_secs, 30);
    }

    #[test]
    fn defaults_interval_to_60_when_absent() {
        let vars = map(&[
            ("TELLTALED_YGGIO_URL", "https://example.com"),
            ("TELLTALED_YGGIO_SECRET", "s3cr3t"),
        ]);
        let cfg = parse_config(&vars).expect("valid without interval");
        assert_eq!(cfg.interval_secs, DEFAULT_INTERVAL_SECS);
    }

    #[test]
    fn missing_url_is_an_error() {
        let vars = map(&[("TELLTALED_YGGIO_SECRET", "s3cr3t")]);
        assert_eq!(
            parse_config(&vars),
            Err(ConfigError::Missing {
                var: "TELLTALED_YGGIO_URL"
            })
        );
    }

    #[test]
    fn missing_secret_is_an_error() {
        let vars = map(&[("TELLTALED_YGGIO_URL", "https://example.com")]);
        assert_eq!(
            parse_config(&vars),
            Err(ConfigError::Missing {
                var: "TELLTALED_YGGIO_SECRET"
            })
        );
    }

    #[test]
    fn empty_url_is_treated_as_missing() {
        let vars = map(&[
            ("TELLTALED_YGGIO_URL", ""),
            ("TELLTALED_YGGIO_SECRET", "s3cr3t"),
        ]);
        assert_eq!(
            parse_config(&vars),
            Err(ConfigError::Missing {
                var: "TELLTALED_YGGIO_URL"
            })
        );
    }

    #[test]
    fn empty_secret_is_treated_as_missing() {
        let vars = map(&[
            ("TELLTALED_YGGIO_URL", "https://example.com"),
            ("TELLTALED_YGGIO_SECRET", ""),
        ]);
        assert_eq!(
            parse_config(&vars),
            Err(ConfigError::Missing {
                var: "TELLTALED_YGGIO_SECRET"
            })
        );
    }

    #[test]
    fn invalid_interval_is_an_error() {
        let mut vars = full_map();
        vars.insert(
            "TELLTALED_INTERVAL_SECS".to_string(),
            "not-a-number".to_string(),
        );
        assert_eq!(
            parse_config(&vars),
            Err(ConfigError::InvalidInterval {
                value: "not-a-number".to_string()
            })
        );
    }

    #[test]
    fn negative_interval_string_is_an_error() {
        let mut vars = full_map();
        vars.insert("TELLTALED_INTERVAL_SECS".to_string(), "-5".to_string());
        assert_eq!(
            parse_config(&vars),
            Err(ConfigError::InvalidInterval {
                value: "-5".to_string()
            })
        );
    }

    #[test]
    fn error_messages_are_human_readable() {
        let missing = ConfigError::Missing {
            var: "TELLTALED_YGGIO_URL",
        };
        assert!(
            missing.to_string().contains("TELLTALED_YGGIO_URL"),
            "error message names the variable"
        );

        let invalid = ConfigError::InvalidInterval {
            value: "abc".to_string(),
        };
        assert!(
            invalid.to_string().contains("abc"),
            "error message includes the bad value"
        );
    }

    #[test]
    fn debug_redacts_secret() {
        let cfg = parse_config(&full_map()).expect("valid");
        let debug = format!("{cfg:?}");
        assert!(
            debug.contains("redacted"),
            "secret must be redacted in Debug"
        );
        assert!(!debug.contains("s3cr3t"), "raw secret must not appear");
    }
}
