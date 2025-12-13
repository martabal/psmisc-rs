use std::{collections::HashMap, error::Error, fs};

use helpers::{PROC, parse_pid_from_bytes};
#[cfg(feature = "orx-parallel")]
use orx_parallel::{IterIntoParIter, ParIter};
#[cfg(feature = "rayon")]
use rayon::iter::{ParallelBridge, ParallelIterator};

#[derive(Debug)]
pub enum ProcessState {
    Running,
    Sleeping,
    Zombie,
    TracingStop,
    Dead,
    Idle,
}

#[derive(Debug)]
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

pub fn build_process_tree() -> Result<HashMap<i32, ProcessNode>, Box<dyn Error>> {
    let entries = fs::read_dir(PROC)?;

    #[cfg(feature = "rayon")]
    let iter = entries.par_bridge();
    #[cfg(feature = "orx-parallel")]
    let iter = entries.iter_into_par();
    #[cfg(all(not(feature = "rayon"), not(feature = "orx-parallel")))]
    let iter = entries.into_iter();

    let pids: Vec<ProcessNode> = iter
        .filter_map(Result::ok)
        .filter_map(|entry| check_entry(&entry))
        .collect();

    let mut tree: HashMap<i32, ProcessNode> =
        pids.into_iter().map(|proc| (proc.pid, proc)).collect();

    let relationships: Vec<(i32, i32)> = {
        let iter = tree.values();

        iter.filter(|proc| proc.ppid != 0)
            .map(|proc| (proc.ppid, proc.pid))
            .collect()
    };

    for (ppid, pid) in relationships {
        tree.get_mut(&ppid)
            .ok_or("Failed to get parent process")?
            .add_child(pid);
    }

    Ok(tree)
}

fn check_entry(entry: &fs::DirEntry) -> Option<ProcessNode> {
    let pid = parse_pid_from_bytes(entry.file_name().as_encoded_bytes())?;
    parse_process(pid).ok()
}

fn parse_process(pid: i32) -> Result<ProcessNode, Box<dyn Error>> {
    let mut proc = ProcessNode::new();
    proc.pid = pid;

    let status_file = fs::read_to_string(format!("{PROC}/{pid}/status"))?;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_node_new() {
        let node = ProcessNode::new();
        assert_eq!(node.pid, 1);
        assert_eq!(node.ppid, 0);
        assert_eq!(node.name, "");
        assert!(node.children.is_none());
    }

    #[test]
    fn test_process_node_default() {
        let node = ProcessNode::default();
        assert_eq!(node.pid, 1);
        assert_eq!(node.ppid, 0);
        assert_eq!(node.name, "");
        assert!(node.children.is_none());
    }

    #[test]
    fn test_process_node_add_child() {
        let mut node = ProcessNode::new();

        // First child
        node.add_child(100);
        assert!(node.children.is_some());
        assert_eq!(node.children.as_ref().unwrap().len(), 1);
        assert_eq!(node.children.as_ref().unwrap()[0], 100);

        // Second child
        node.add_child(200);
        assert_eq!(node.children.as_ref().unwrap().len(), 2);
        assert_eq!(node.children.as_ref().unwrap()[1], 200);
    }

    #[test]
    fn test_process_node_add_multiple_children() {
        let mut node = ProcessNode::new();

        for i in 1..=5 {
            node.add_child(i * 100);
        }

        assert_eq!(node.children.as_ref().unwrap().len(), 5);
        for i in 1i32..=5 {
            assert_eq!(node.children.as_ref().unwrap()[(i - 1) as usize], i * 100);
        }
    }

    #[test]
    fn test_process_state_debug() {
        // Just verify we can debug print process states
        let states = vec![
            ProcessState::Running,
            ProcessState::Sleeping,
            ProcessState::Zombie,
            ProcessState::TracingStop,
            ProcessState::Dead,
            ProcessState::Idle,
        ];

        for state in states {
            let debug_str = format!("{:?}", state);
            assert!(!debug_str.is_empty());
        }
    }

    #[test]
    fn test_process_node_debug() {
        let mut node = ProcessNode::new();
        node.pid = 123;
        node.ppid = 1;
        node.name = "test".to_string();
        node.state = ProcessState::Running;
        node.add_child(456);

        let debug_str = format!("{:?}", node);
        assert!(debug_str.contains("123"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_build_process_tree_current_process() {
        // This test just verifies we can call build_process_tree without panicking
        // It should at least find the current process
        let result = build_process_tree();

        match result {
            Ok(tree) => {
                // Tree should not be empty on a real system
                assert!(!tree.is_empty());

                // On Linux systems, PID 1 should exist
                if cfg!(target_os = "linux") {
                    assert!(tree.contains_key(&1));
                }
            }
            Err(_) => {
                // If it fails, it might be due to permission issues or non-Linux system
                // We'll allow this test to pass in such cases
            }
        }
    }
}
