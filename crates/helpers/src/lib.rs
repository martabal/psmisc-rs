use clap::builder::{Styles, styling::AnsiColor};
use nix::sys::signal::Signal;

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

static SIGNALS: &[(&str, Signal)] = &[
    ("INT", Signal::SIGINT),
    ("TERM", Signal::SIGTERM),
    ("KILL", Signal::SIGKILL),
    ("HUP", Signal::SIGHUP),
    ("QUIT", Signal::SIGQUIT),
    ("USR1", Signal::SIGUSR1),
    ("USR2", Signal::SIGUSR2),
    ("ALRM", Signal::SIGALRM),
    ("CONT", Signal::SIGCONT),
    ("STOP", Signal::SIGSTOP),
    ("TSTP", Signal::SIGTSTP),
    ("CHLD", Signal::SIGCHLD),
    ("PIPE", Signal::SIGPIPE),
    ("SEGV", Signal::SIGSEGV),
    ("ABRT", Signal::SIGABRT),
    ("ILL", Signal::SIGILL),
    ("TRAP", Signal::SIGTRAP),
    ("BUS", Signal::SIGBUS),
    ("FPE", Signal::SIGFPE),
    ("TTIN", Signal::SIGTTIN),
    ("TTOU", Signal::SIGTTOU),
    ("URG", Signal::SIGURG),
    ("XCPU", Signal::SIGXCPU),
    ("XFSZ", Signal::SIGXFSZ),
    ("VTALRM", Signal::SIGVTALRM),
    ("PROF", Signal::SIGPROF),
    ("WINCH", Signal::SIGWINCH),
    ("IO", Signal::SIGIO),
    ("PWR", Signal::SIGPWR),
    ("SYS", Signal::SIGSYS),
];

#[must_use]
pub fn parse_signal(name: &str) -> Option<Signal> {
    let upper = name.to_uppercase();

    SIGNALS
        .iter()
        .find(|(sig_name, _)| *sig_name == upper.as_str())
        .map(|(_, signal)| *signal)
}

#[must_use]
pub fn list_signals() -> String {
    SIGNALS
        .iter()
        .map(|(name, _)| *name)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_signal_valid() {
        assert_eq!(parse_signal("INT"), Some(Signal::SIGINT));
        assert_eq!(parse_signal("TERM"), Some(Signal::SIGTERM));
        assert_eq!(parse_signal("KILL"), Some(Signal::SIGKILL));
        assert_eq!(parse_signal("HUP"), Some(Signal::SIGHUP));
        assert_eq!(parse_signal("QUIT"), Some(Signal::SIGQUIT));
        assert_eq!(parse_signal("USR1"), Some(Signal::SIGUSR1));
        assert_eq!(parse_signal("USR2"), Some(Signal::SIGUSR2));
    }

    #[test]
    fn test_parse_signal_case_insensitive() {
        // Lowercase
        assert_eq!(parse_signal("int"), Some(Signal::SIGINT));
        assert_eq!(parse_signal("term"), Some(Signal::SIGTERM));
        assert_eq!(parse_signal("kill"), Some(Signal::SIGKILL));

        // Mixed case
        assert_eq!(parse_signal("InT"), Some(Signal::SIGINT));
        assert_eq!(parse_signal("TeRm"), Some(Signal::SIGTERM));
        assert_eq!(parse_signal("KiLl"), Some(Signal::SIGKILL));
    }

    #[test]
    fn test_parse_signal_invalid() {
        assert_eq!(parse_signal("INVALID"), None);
        assert_eq!(parse_signal(""), None);
        assert_eq!(parse_signal("123"), None);
        assert_eq!(parse_signal("SIG"), None);
        assert_eq!(parse_signal("SIGINT"), None); // Function expects short signal names without 'SIG' prefix
    }

    #[test]
    fn test_parse_signal_all_signals() {
        // Verify all signals in our list can be parsed
        for (name, expected_signal) in SIGNALS {
            assert_eq!(parse_signal(name), Some(*expected_signal));
        }
    }

    #[test]
    fn test_list_signals_contains_expected() {
        let signals = list_signals();

        // Check that common signals are in the list
        assert!(signals.contains("INT"));
        assert!(signals.contains("TERM"));
        assert!(signals.contains("KILL"));
        assert!(signals.contains("HUP"));
        assert!(signals.contains("QUIT"));
    }

    #[test]
    fn test_list_signals_format() {
        let signals = list_signals();

        // Should be space-separated
        assert!(signals.contains(' '));

        // Should not have trailing space
        assert!(!signals.ends_with(' '));

        // Should not be empty
        assert!(!signals.is_empty());
    }

    #[test]
    fn test_list_signals_count() {
        let signals = list_signals();
        let count = signals.split(' ').count();

        // Should match the number of signals we defined
        assert_eq!(count, SIGNALS.len());
    }

    #[test]
    fn test_common_signals() {
        // Test the most common signals used in killall
        assert_eq!(parse_signal("TERM"), Some(Signal::SIGTERM));
        assert_eq!(parse_signal("KILL"), Some(Signal::SIGKILL));
        assert_eq!(parse_signal("INT"), Some(Signal::SIGINT));
        assert_eq!(parse_signal("HUP"), Some(Signal::SIGHUP));
    }

    #[test]
    fn test_parse_all_listed_signals() {
        let signal_list = list_signals();
        let signal_names: Vec<&str> = signal_list.split(' ').collect();

        // Every signal in the list should be parseable
        for name in signal_names {
            assert!(
                parse_signal(name).is_some(),
                "Signal {} should be parseable",
                name
            );
        }
    }

    #[test]
    fn test_signal_parsing_roundtrip() {
        // Parse a signal and verify it returns the expected value
        let common_signals = vec!["INT", "TERM", "KILL", "HUP", "QUIT"];

        for signal_name in common_signals {
            let parsed = parse_signal(signal_name);
            assert!(parsed.is_some(), "Failed to parse {}", signal_name);
        }
    }

    #[test]
    fn test_list_signals_not_empty() {
        let signals = list_signals();
        assert!(!signals.is_empty());
        assert!(signals.len() > 10); // Should have many signals
    }
}
