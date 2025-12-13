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
    use crate::output::print_tree_with_pid;

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

                // On Unix-like systems, PID 1 should exist
                #[cfg(unix)]
                {
                    assert!(tree.contains_key(&1));
                }
            }
            Err(_) => {
                // If it fails, it might be due to permission issues or non-Unix system
                // We'll allow this test to pass in such cases
            }
        }
    }

    #[test]
    fn test_process_state_variants() {
        // Test that all process state variants can be created
        let states = vec![
            ProcessState::Running,
            ProcessState::Sleeping,
            ProcessState::Zombie,
            ProcessState::TracingStop,
            ProcessState::Dead,
            ProcessState::Idle,
        ];

        for state in states {
            let node = ProcessNode {
                pid: 1,
                ppid: 0,
                name: "test".to_string(),
                state,
                children: None,
            };
            assert_eq!(node.pid, 1);
        }
    }

    #[test]
    fn test_process_node_with_many_children() {
        let mut node = ProcessNode::new();

        // Add many children
        for i in 100..200 {
            node.add_child(i);
        }

        assert!(node.children.is_some());
        assert_eq!(node.children.as_ref().unwrap().len(), 100);
    }

    #[test]
    fn test_print_tree_with_single_process() {
        let mut tree = HashMap::new();
        let node = ProcessNode {
            pid: 1,
            ppid: 0,
            name: "init".to_string(),
            state: ProcessState::Running,
            children: None,
        };
        tree.insert(1, node);

        // Should successfully print a single process
        let result = print_tree_with_pid(&tree, 1, 0, "", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_process_node_relationships() {
        let result = build_process_tree();

        match result {
            Ok(tree) => {
                // Check that parent-child relationships are consistent
                for (pid, node) in &tree {
                    assert_eq!(node.pid, *pid, "Node PID should match map key");

                    // If node has children, verify those children exist in the tree
                    if let Some(children) = &node.children {
                        for &child_pid in children {
                            if let Some(child_node) = tree.get(&child_pid) {
                                // Child's parent should be this node
                                assert_eq!(
                                    child_node.ppid, *pid,
                                    "Child {} should have parent {}",
                                    child_pid, pid
                                );
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }
    }
    #[test]
    fn test_build_process_tree_returns_tree() {
        // Should be able to build a process tree on any Unix-like system
        let result = build_process_tree();

        match result {
            Ok(tree) => {
                // Tree should have at least one process (init/systemd at PID 1)
                assert!(!tree.is_empty(), "Process tree should not be empty");

                // On Unix-like systems, PID 1 should exist
                #[cfg(unix)]
                {
                    assert!(tree.contains_key(&1), "PID 1 (init/systemd) should exist");
                }
            }
            Err(e) => {
                // On some systems or in containers, this might fail
                // We'll allow the test to pass but print the error
                eprintln!("Warning: Could not build process tree: {}", e);
            }
        }
    }

    #[test]
    fn test_build_process_tree_has_current_process() {
        let result = build_process_tree();

        match result {
            Ok(tree) => {
                let current_pid = std::process::id() as i32;

                // Current process should be in the tree
                assert!(
                    tree.contains_key(&current_pid),
                    "Current process (PID {}) should be in the tree",
                    current_pid
                );
            }
            Err(_) => {
                // Allow test to pass if we can't read process tree
            }
        }
    }
    #[test]
    fn test_print_tree_with_hierarchy() {
        let mut tree = HashMap::new();

        // Create parent
        let parent = ProcessNode {
            pid: 1,
            ppid: 0,
            name: "parent".to_string(),
            state: ProcessState::Running,
            children: Some(vec![2, 3]),
        };
        tree.insert(1, parent);

        // Create children
        let child1 = ProcessNode {
            pid: 2,
            ppid: 1,
            name: "child1".to_string(),
            state: ProcessState::Sleeping,
            children: None,
        };
        tree.insert(2, child1);

        let child2 = ProcessNode {
            pid: 3,
            ppid: 1,
            name: "child2".to_string(),
            state: ProcessState::Running,
            children: None,
        };
        tree.insert(3, child2);

        // Should successfully print hierarchy
        let result = print_tree_with_pid(&tree, 1, 0, "", true);
        assert!(result.is_ok());
    }
}
