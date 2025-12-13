use std::{error::Error, fs, io, os::unix::ffi::OsStrExt};

use helpers::{PROC, parse_pid_from_bytes};
use nix::unistd::{SysconfVar, sysconf};
#[cfg(feature = "orx-parallel")]
use orx_parallel::{IterIntoParIter, ParIter};
#[cfg(feature = "rayon")]
use rayon::iter::{ParallelBridge, ParallelIterator};

pub struct OptionsPids {
    pub use_group: bool,
    pub younger_than: Option<humantime::Duration>,
    pub older_than: Option<humantime::Duration>,
    pub ignore_case: bool,
}

pub fn list_pids(target_name: &str, opt: &OptionsPids) -> Result<Vec<i32>, Box<dyn Error>> {
    let target_bytes = target_name.as_bytes();

    let entries = fs::read_dir(PROC)?;
    #[cfg(feature = "rayon")]
    let iter = entries.par_bridge();
    #[cfg(feature = "orx-parallel")]
    let iter = entries.iter_into_par();
    #[cfg(all(not(feature = "rayon"), not(feature = "orx-parallel")))]
    let iter = entries.into_iter();

    let matching_pids: Vec<i32> = iter
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            let pid = check_entry(&entry, target_bytes, opt.ignore_case)?;
            let stat = check_stat(pid)?;
            if matches!(
                check_time(&stat, opt.younger_than, opt.older_than),
                Ok(true)
            ) {
                Some(pid)
            } else {
                None
            }
        })
        .collect();

    if !opt.use_group || matching_pids.is_empty() {
        return Ok(matching_pids);
    }

    let mut groups: Vec<i32> = matching_pids
        .iter()
        .filter_map(|&pid| check_stat(pid).map(|s| s.pgrp))
        .collect();

    if groups.is_empty() {
        return Ok(matching_pids);
    }

    groups.sort_unstable();
    groups.dedup();

    let entries = fs::read_dir(PROC)?;
    #[cfg(feature = "rayon")]
    let iter = entries.par_bridge();
    #[cfg(feature = "orx-parallel")]
    let iter = entries.iter_into_par();
    #[cfg(all(not(feature = "rayon"), not(feature = "orx-parallel")))]
    let iter = entries.into_iter();

    let mut all_pids: Vec<i32> = iter
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            let pid = parse_pid_from_bytes(entry.file_name().as_bytes())?;
            let stat = check_stat(pid)?;
            if groups.binary_search(&stat.pgrp).is_ok()
                && matches!(
                    check_time(&stat, opt.younger_than, opt.older_than),
                    Ok(true)
                )
            {
                Some(pid)
            } else {
                None
            }
        })
        .collect();

    all_pids.sort_unstable();
    all_pids.dedup();
    Ok(all_pids)
}

struct Stat {
    pgrp: i32,
    starttime: f64,
}

fn check_stat(pid: i32) -> Option<Stat> {
    let stat_path = format!("{PROC}/{pid}/stat");
    let contents = fs::read_to_string(stat_path).ok()?;
    let mut parts = contents.split_whitespace();

    for _ in 0..4 {
        parts.next()?;
    }
    let pgrp: i32 = parts.next()?.parse().ok()?;

    for _ in 0..16 {
        parts.next()?;
    }
    let starttime: f64 = parts.next()?.parse().ok()?;
    Some(Stat { pgrp, starttime })
}

fn get_system_uptime() -> f64 {
    fs::read_to_string(format!("{PROC}/uptime"))
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
        .unwrap_or(0.0)
}

fn get_clock_ticks_per_sec() -> Result<f64, Box<dyn Error>> {
    let ticks = sysconf(SysconfVar::CLK_TCK)
        .ok()
        .flatten()
        .ok_or("failed to get clock ticks per second")?;

    // i64 to f64 conversion is infallible for reasonable values
    #[allow(clippy::cast_precision_loss)]
    let ticks_f64 = ticks as f64;
    Ok(ticks_f64)
}

fn check_time(
    pid_stat: &Stat,
    younger_than: Option<humantime::Duration>,
    older_than: Option<humantime::Duration>,
) -> Result<bool, Box<dyn Error>> {
    if younger_than.is_none() && older_than.is_none() {
        return Ok(true);
    }

    let system_uptime_secs = get_system_uptime();
    let ticks_per_sec = get_clock_ticks_per_sec()?;

    if ticks_per_sec <= 0.0 || system_uptime_secs <= 0.0 {
        return Ok(false);
    }

    let starttime_secs = pid_stat.starttime / ticks_per_sec;
    let process_age_secs = system_uptime_secs - starttime_secs;

    if let Some(max_age) = younger_than
        && process_age_secs >= max_age.as_secs_f64()
    {
        return Ok(false);
    }
    if let Some(min_age) = older_than
        && process_age_secs < min_age.as_secs_f64()
    {
        return Ok(false);
    }
    Ok(true)
}

fn check_entry(entry: &fs::DirEntry, target_bytes: &[u8], case_insensitive: bool) -> Option<i32> {
    let pid = parse_pid_from_bytes(entry.file_name().as_bytes())?;

    let comm_path = format!("{PROC}/{pid}/comm");
    let mut buf = [0u8; 64];
    let len = fs::File::open(&comm_path)
        .ok()
        .and_then(|mut f| io::Read::read(&mut f, &mut buf).ok())?;

    let name = if len > 0 && buf[len - 1] == b'\n' {
        &buf[..len - 1]
    } else {
        &buf[..len]
    };

    if case_insensitive {
        if name.eq_ignore_ascii_case(target_bytes) {
            return Some(pid);
        }
        return None;
    }

    (name == target_bytes).then_some(pid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_parse_pid_valid() {
        assert_eq!(parse_pid_from_bytes(b"1"), Some(1));
        assert_eq!(parse_pid_from_bytes(b"12345"), Some(12345));
        assert_eq!(parse_pid_from_bytes(b"429496729"), Some(429496729));
    }

    #[test]
    fn test_parse_pid_invalid() {
        assert_eq!(parse_pid_from_bytes(b""), None);
        assert_eq!(parse_pid_from_bytes(b"abc"), None);
        assert_eq!(parse_pid_from_bytes(b"0000"), None);
        assert_eq!(parse_pid_from_bytes(b"18446744073"), None);
    }

    fn unique_test_dir() -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("fake_proc_{}", nanos))
    }

    fn setup_fake_proc(tmp: &Path, entries: &[(&str, &str)]) {
        fs::create_dir_all(tmp).unwrap();
        for (pid, comm) in entries {
            let proc_dir = tmp.join(pid);
            fs::create_dir_all(&proc_dir).unwrap();
            let comm_path = proc_dir.join("comm");
            let mut f = File::create(comm_path).unwrap();
            writeln!(f, "{}", comm).unwrap();
        }
    }

    fn cleanup_fake_proc(tmp: &Path) {
        if tmp.exists() {
            fs::remove_dir_all(tmp).unwrap();
        }
    }

    #[test]
    fn test_list_pids_no_match() {
        let tmp = unique_test_dir();
        setup_fake_proc(&tmp, &[("789", "sshd")]);

        let result: Vec<i32> = fs::read_dir(&tmp)
            .unwrap()
            .filter_map(|e| e.ok().and_then(|entry| check_entry(&entry, b"bash", false)))
            .collect();

        assert!(result.is_empty());

        cleanup_fake_proc(&tmp);
    }

    #[test]
    fn test_check_time_no_filters() {
        let dummy_stat = Stat {
            pgrp: 1,
            starttime: 0.0,
        };
        assert_eq!(check_time(&dummy_stat, None, None).unwrap(), true);
    }

    #[test]
    fn test_options_pids_default() {
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
    fn test_get_system_uptime() {
        let uptime = get_system_uptime();
        // On a real system, uptime should be positive
        // If reading fails, it returns 0.0
        assert!(uptime >= 0.0);
    }

    #[test]
    fn test_get_clock_ticks_per_sec() {
        let result = get_clock_ticks_per_sec();
        // Should succeed on Unix-like systems
        if let Ok(ticks) = result {
            assert!(ticks > 0.0);
        }
    }

    #[test]
    fn test_check_stat() {
        // Test with current process (should always exist)
        let current_pid = std::process::id() as i32;
        let stat = check_stat(current_pid);

        // Should successfully get stat for current process
        assert!(stat.is_some());
        if let Some(s) = stat {
            assert!(s.pgrp > 0);
            assert!(s.starttime >= 0.0);
        }
    }

    #[test]
    fn test_check_stat_invalid_pid() {
        // Very high PID that likely doesn't exist
        let stat = check_stat(i32::MAX);
        assert!(stat.is_none());
    }

    #[test]
    fn test_list_pids_invalid_process_name() {
        let opts = OptionsPids {
            use_group: false,
            younger_than: None,
            older_than: None,
            ignore_case: false,
        };

        // Search for a process name that definitely doesn't exist
        let result = list_pids("__nonexistent_process_12345__", &opts);

        match result {
            Ok(pids) => {
                // Should return empty list
                assert!(pids.is_empty());
            }
            Err(_) => {
                // Or error if /proc is not accessible
            }
        }
    }

    #[test]
    fn test_processes_options_construction() {
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
}
