use std::{collections::HashMap, error::Error, fs, io};

#[derive(Debug, Clone)]
pub enum ProcessState {
    Running,
    Sleeping,
    Zombie,
    TracingStop,
    Dead,
    Idle,
}

#[derive(Debug, Clone)]
pub struct ProcessNode {
    pub pid: i32,
    pub ppid: i32,
    pub name: String,
    pub state: ProcessState,
    pub children: Option<Vec<i32>>,
}

impl Default for ProcessNode {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessNode {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            pid: 1,
            ppid: 0,
            name: String::new(),
            state: ProcessState::Idle,
            children: None,
        }
    }

    pub fn add_child(&mut self, child: i32) {
        self.children.get_or_insert_with(Vec::new).push(child);
    }
}

use helpers::parse_pid_from_bytes;
#[cfg(feature = "rayon")]
use rayon::prelude::*;

const PROC: &str = "/proc";

pub fn build_process_tree(
    process_list: &[ProcessNode],
) -> Result<HashMap<i32, ProcessNode>, Box<dyn Error>> {
    let mut tree = HashMap::new();

    for proc in process_list {
        tree.insert(proc.pid, proc.clone());
    }

    for proc in process_list {
        if proc.ppid != 0 {
            tree.get_mut(&proc.ppid)
                .ok_or("Failed to get parent process")?
                .add_child(proc.pid);
        }
    }
    Ok(tree)
}

pub fn read_process() -> io::Result<Vec<ProcessNode>> {
    let entries = fs::read_dir(PROC)?;

    #[cfg(feature = "rayon")]
    let iter = entries.par_bridge();

    #[cfg(not(feature = "rayon"))]
    let iter = entries.into_iter();

    let pids: Vec<ProcessNode> = iter
        .filter_map(|e| e.ok().and_then(|entry| check_entry(&entry)))
        .collect();

    Ok(pids)
}

fn check_entry(entry: &fs::DirEntry) -> Option<ProcessNode> {
    let pid = parse_pid_from_bytes(entry.file_name().as_encoded_bytes())?;
    parse_process(pid).ok()
}

fn parse_process(pid: i32) -> Result<ProcessNode, Box<dyn Error>> {
    let mut proc = ProcessNode::new();
    proc.pid = pid;

    let status_file = fs::read_to_string(format!("/proc/{pid}/status"))?;

    for line in status_file.lines() {
        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("Name:") => {
                if let Some(name) = parts.next() {
                    proc.name = name.to_string();
                }
            }
            Some("PPid:") => {
                if let Some(ppid_str) = parts.next() {
                    proc.ppid = ppid_str.parse::<i32>()?;
                    break;
                }
            }
            Some("State:") => {
                if let Some(state_str) = parts.next() {
                    proc.state = match state_str {
                        "R" => ProcessState::Running,
                        "S" => ProcessState::Sleeping,
                        "Z" => ProcessState::Zombie,
                        "T" => ProcessState::TracingStop,
                        "X" => ProcessState::Dead,
                        _ => ProcessState::Idle,
                    };
                }
            }
            _ => {}
        }
    }

    Ok(proc)
}
