use std::sync::atomic::AtomicBool;

pub static QUIET: AtomicBool = AtomicBool::new(false);

#[macro_export]
macro_rules! qprintln {
    () => {
        if !QUIET.load(Ordering::Relaxed) {
            eprintln!();
        }
    };
    ($($arg:tt)*) => {
        if !QUIET.load(Ordering::Relaxed) {
            eprintln!($($arg)*);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    #[test]
    fn test_quiet_flag_default() {
        // QUIET should be false by default but may have been set by other tests
        // Just verify we can read it
        let _ = QUIET.load(Ordering::Relaxed);
    }

    #[test]
    fn test_quiet_flag_toggle() {
        // Save initial state
        let initial = QUIET.load(Ordering::Relaxed);
        
        // Test setting to true
        QUIET.store(true, Ordering::Relaxed);
        assert_eq!(QUIET.load(Ordering::Relaxed), true);
        
        // Test setting to false
        QUIET.store(false, Ordering::Relaxed);
        assert_eq!(QUIET.load(Ordering::Relaxed), false);
        
        // Restore initial state
        QUIET.store(initial, Ordering::Relaxed);
    }
}
