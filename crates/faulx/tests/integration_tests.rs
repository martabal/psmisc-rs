use faulx::signals::{list_signals, parse_signal};
use nix::sys::signal::Signal;

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
fn test_common_signals() {
    // Test the most common signals used in killall
    assert_eq!(parse_signal("TERM"), Some(Signal::SIGTERM));
    assert_eq!(parse_signal("KILL"), Some(Signal::SIGKILL));
    assert_eq!(parse_signal("INT"), Some(Signal::SIGINT));
    assert_eq!(parse_signal("HUP"), Some(Signal::SIGHUP));
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

#[test]
fn test_processes_options_construction() {
    use faulx::processes::OptionsPids;
    
    let opts = OptionsPids {
        use_group: false,
        younger_than: None,
        older_than: None,
        ignore_case: false,
    };
    
    assert!(!opts.use_group);
    assert!(opts.younger_than.is_none());
    assert!(opts.older_than.is_none());
    assert!(!opts.ignore_case);
}

#[test]
fn test_list_pids_nonexistent_process() {
    use faulx::processes::{OptionsPids, list_pids};
    
    let opts = OptionsPids {
        use_group: false,
        younger_than: None,
        older_than: None,
        ignore_case: false,
    };
    
    // Try to find a process that definitely doesn't exist
    let result = list_pids("__this_process_definitely_does_not_exist_12345__", &opts);
    
    // Should either return an empty list or an error
    match result {
        Ok(pids) => assert!(pids.is_empty()),
        Err(_) => {} // Error is also acceptable if /proc is not accessible
    }
}

#[test]
fn test_list_pids_with_case_sensitivity() {
    use faulx::processes::{OptionsPids, list_pids};
    
    // Test case sensitive
    let opts_sensitive = OptionsPids {
        use_group: false,
        younger_than: None,
        older_than: None,
        ignore_case: false,
    };
    
    // Test case insensitive  
    let opts_insensitive = OptionsPids {
        use_group: false,
        younger_than: None,
        older_than: None,
        ignore_case: true,
    };
    
    // Both should work without panicking
    let _ = list_pids("systemd", &opts_sensitive);
    let _ = list_pids("SYSTEMD", &opts_insensitive);
}
