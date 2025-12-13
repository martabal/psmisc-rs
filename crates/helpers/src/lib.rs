use clap::builder::{Styles, styling::AnsiColor};

pub mod macros;

pub fn parse_pid_from_bytes(bytes: &[u8]) -> Option<i32> {
    if bytes.is_empty() || bytes.len() > 10 {
        return None;
    }

    let mut result: i32 = 0;
    for &b in bytes {
        if !b.is_ascii_digit() {
            return None;
        }
        result = result.checked_mul(10)?.checked_add((b - b'0').into())?;
    }
    if result == 0 { None } else { Some(result) }
}

pub const PROC: &str = "/proc";

pub const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().bold())
    .usage(AnsiColor::Green.on_default().bold())
    .literal(AnsiColor::Cyan.on_default().bold())
    .placeholder(AnsiColor::Cyan.on_default());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pid_from_bytes_valid() {
        assert_eq!(parse_pid_from_bytes(b"1"), Some(1));
        assert_eq!(parse_pid_from_bytes(b"123"), Some(123));
        assert_eq!(parse_pid_from_bytes(b"12345"), Some(12345));
        assert_eq!(parse_pid_from_bytes(b"999999"), Some(999999));
        assert_eq!(parse_pid_from_bytes(b"2147483647"), Some(2147483647)); // i32::MAX
    }

    #[test]
    fn test_parse_pid_from_bytes_invalid() {
        // Empty input
        assert_eq!(parse_pid_from_bytes(b""), None);

        // Zero should return None
        assert_eq!(parse_pid_from_bytes(b"0"), None);
        assert_eq!(parse_pid_from_bytes(b"00"), None);
        assert_eq!(parse_pid_from_bytes(b"000"), None);

        // Non-digit characters
        assert_eq!(parse_pid_from_bytes(b"abc"), None);
        assert_eq!(parse_pid_from_bytes(b"12a"), None);
        assert_eq!(parse_pid_from_bytes(b"a12"), None);
        assert_eq!(parse_pid_from_bytes(b"1 2"), None);
        assert_eq!(parse_pid_from_bytes(b"-1"), None);
        assert_eq!(parse_pid_from_bytes(b"+1"), None);

        // Too long (more than 10 digits)
        assert_eq!(parse_pid_from_bytes(b"12345678901"), None);

        // Overflow i32::MAX
        assert_eq!(parse_pid_from_bytes(b"2147483648"), None);
        assert_eq!(parse_pid_from_bytes(b"9999999999"), None);
    }

    #[test]
    fn test_parse_pid_from_bytes_edge_cases() {
        // Single digit
        assert_eq!(parse_pid_from_bytes(b"1"), Some(1));
        assert_eq!(parse_pid_from_bytes(b"9"), Some(9));

        // Leading zeros don't affect the final parsed value
        assert_eq!(parse_pid_from_bytes(b"01"), Some(1));

        // Maximum valid PID
        assert_eq!(parse_pid_from_bytes(b"2147483647"), Some(2147483647));
    }
}
