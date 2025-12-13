use std::{collections::HashMap, error::Error, fs, path::Path};

use helpers::{PROC, parse_pid_from_bytes};

#[derive(Debug)]
pub struct ProcessInfo {
    pub pid: i32,
    pub comm: String,
    pub accesses: Vec<AccessType>,
}

#[derive(Debug)]
pub enum AccessType {
    Cwd,       // Current working directory
    Root,      // Root directory
    Exe,       // Executable
    Fd,        // File descriptor
    MemoryMap, // Memory mapped file
}

impl AccessType {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Cwd => "cwd",
            Self::Root => "root",
            Self::Exe => "exe",
            Self::Fd => "fd",
            Self::MemoryMap => "mmap",
        }
    }
}

pub fn find_processes_using_file(target_path: &str) -> Result<Vec<ProcessInfo>, Box<dyn Error>> {
    let target_path = fs::canonicalize(target_path)?;
    let mut results = Vec::new();

    for entry in fs::read_dir(PROC)? {
        let Ok(entry) = entry else { continue };

        let Some(pid) = parse_pid_from_bytes(entry.file_name().as_encoded_bytes()) else {
            continue;
        };

        if let Some(info) = check_process(pid, &target_path) {
            results.push(info);
        }
    }

    Ok(results)
}

fn check_process(pid: i32, target_path: &Path) -> Option<ProcessInfo> {
    let comm = get_process_comm(pid)?;
    let mut accesses = Vec::new();

    if let Ok(cwd) = fs::read_link(format!("{PROC}/{pid}/cwd"))
        && cwd == target_path
    {
        accesses.push(AccessType::Cwd);
    }

    if let Ok(root) = fs::read_link(format!("{PROC}/{pid}/root"))
        && root == target_path
    {
        accesses.push(AccessType::Root);
    }

    if let Ok(exe) = fs::read_link(format!("{PROC}/{pid}/exe"))
        && exe == target_path
    {
        accesses.push(AccessType::Exe);
    }

    if let Ok(fd_entries) = fs::read_dir(format!("{PROC}/{pid}/fd")) {
        for fd in fd_entries.filter_map(Result::ok) {
            if let Ok(path) = fs::read_link(fd.path())
                && path == target_path
            {
                accesses.push(AccessType::Fd);
                break;
            }
        }
    }

    if let Ok(maps) = fs::read_to_string(format!("{PROC}/{pid}/maps")) {
        for line in maps.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 6 && Path::new(parts[5]) == target_path {
                accesses.push(AccessType::MemoryMap);
                break;
            }
        }
    }

    if accesses.is_empty() {
        None
    } else {
        Some(ProcessInfo {
            pid,
            comm,
            accesses,
        })
    }
}

fn get_process_comm(pid: i32) -> Option<String> {
    let comm_path = format!("{PROC}/{pid}/comm");
    fs::read_to_string(comm_path)
        .ok()
        .map(|s| s.trim().to_string())
}

#[must_use]
pub fn group_by_file(infos: Vec<ProcessInfo>) -> HashMap<String, Vec<ProcessInfo>> {
    let mut grouped: HashMap<String, Vec<ProcessInfo>> = HashMap::new();

    for info in infos {
        grouped
            .entry(format!("{}({})", info.comm, info.pid))
            .or_default()
            .push(info);
    }

    grouped
}
