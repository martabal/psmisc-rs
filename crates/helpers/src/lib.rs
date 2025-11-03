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
