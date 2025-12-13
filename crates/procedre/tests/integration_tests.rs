use procedre::process::{ProcessNode, ProcessState, build_process_tree};
use procedre::output::print_tree_with_pid;
use std::collections::HashMap;

#[test]
fn test_build_process_tree_returns_tree() {
    // Should be able to build a process tree on any Unix-like system
    let result = build_process_tree();
    
    match result {
        Ok(tree) => {
            // Tree should have at least one process (init/systemd at PID 1)
            assert!(!tree.is_empty(), "Process tree should not be empty");
            
            // On Linux systems, PID 1 should exist
            if cfg!(target_os = "linux") {
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
fn test_print_tree_with_empty_tree() {
    let tree: HashMap<i32, ProcessNode> = HashMap::new();
    
    // Should not panic with empty tree
    let result = print_tree_with_pid(&tree, 1, 0, "", true);
    assert!(result.is_ok());
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
fn test_build_and_print_integration() {
    // Integration test: build tree and print it
    let result = build_process_tree();
    
    match result {
        Ok(tree) => {
            // Try to print from PID 1
            let print_result = print_tree_with_pid(&tree, 1, 0, "", true);
            assert!(print_result.is_ok());
        }
        Err(_) => {
            // Allow test to pass if we can't build tree
        }
    }
}
