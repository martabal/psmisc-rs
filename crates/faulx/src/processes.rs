use std::{error::Error, fs, io, os::unix::ffi::OsStrExt};

use helpers::parse_pid_from_bytes;
use nix::unistd::{SysconfVar, sysconf};
#[cfg(feature = "rayon")]
use rayon::prelude::*;

const PROC: &str = "/proc";

pub struct OptionsPids {
    pub use_group: bool,
    pub younger_than: Option<humantime::Duration>,
    pub older_than: Option<humantime::Duration>,
}

pub fn list_pids(target_name: &str, opt: &OptionsPids) -> Result<Vec<i32>, Box<dyn Error>> {
    let target_bytes = target_name.as_bytes();
    let uptime = get_system_uptime();
    let ticks = get_clock_ticks_per_sec()?;

    let entries = fs::read_dir(PROC)?;
    #[cfg(feature = "rayon")]
    let iter = entries.par_bridge();
    #[cfg(not(feature = "rayon"))]
    let iter = entries.into_iter();

    let matching_pids: Vec<i32> = iter
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            let pid = check_entry(&entry, target_bytes)?;
            let stat = check_stat(pid)?;
            if check_time(&stat, opt.younger_than, opt.older_than, uptime, ticks) {
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

    groups.sort_unstable();
    groups.dedup();

    if groups.is_empty() {
        return Ok(matching_pids);
    }

    let entries = fs::read_dir(PROC)?;
    #[cfg(feature = "rayon")]
    let iter = entries.par_bridge();
    #[cfg(not(feature = "rayon"))]
    let iter = entries.into_iter();

    let mut all_pids: Vec<i32> = iter
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            let pid = parse_pid_from_bytes(entry.file_name().as_bytes())?;
            let stat = check_stat(pid)?;
            if groups.binary_search(&stat.pgrp).is_ok()
                && check_time(&stat, opt.younger_than, opt.older_than, uptime, ticks)
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
    fs::read_to_string("/proc/uptime")
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
    system_uptime_secs: f64,
    ticks_per_sec: f64,
) -> bool {
    if younger_than.is_none() && older_than.is_none() {
        return true;
    }

    if ticks_per_sec <= 0.0 || system_uptime_secs <= 0.0 {
        return false;
    }

    let starttime_secs = pid_stat.starttime / ticks_per_sec;
    let process_age_secs = system_uptime_secs - starttime_secs;

    if let Some(max_age) = younger_than
        && process_age_secs >= max_age.as_secs_f64()
    {
        return false;
    }
    if let Some(min_age) = older_than
        && process_age_secs < min_age.as_secs_f64()
    {
        return false;
    }
    true
}

fn check_entry(entry: &fs::DirEntry, target_bytes: &[u8]) -> Option<i32> {
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
            .filter_map(|e| e.ok().and_then(|entry| check_entry(&entry, b"bash")))
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
        assert!(check_time(&dummy_stat, None, None, 100.0, 100.0));
    }
}
