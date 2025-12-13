use std::{collections::HashMap, error::Error};

use super::process::ProcessNode;

pub fn print_tree_with_pid<S>(
    tree: &HashMap<i32, ProcessNode, S>,
    pid: i32,
    depth: i32,
    prefix: &str,
    is_last: bool,
) -> Result<(), Box<dyn Error>>
where
    S: std::hash::BuildHasher,
{
    let Some(node) = tree.get(&pid) else {
        return Ok(());
    };

    let node_info = format!("{}({})", node.name, node.pid);

    match (depth, node.children.as_ref(), is_last) {
        (0, _, _) => println!("{prefix}{node_info}"),
        (_, Some(children), false) if !children.is_empty() => println!("{prefix}├─┬─{node_info}"),
        (_, _, false) => println!("{prefix}├───{node_info}"),
        _ => println!("{prefix}└───{node_info}"),
    }

    let Some(children) = &node.children else {
        return Ok(());
    };

    let new_prefix = match (depth, is_last) {
        (0, _) => format!("{prefix}  "),
        (_, true) => prefix.to_string(),
        _ => format!("{prefix}│ "),
    };

    for (i, &child_pid) in children.iter().enumerate() {
        let _ = print_tree_with_pid(
            tree,
            child_pid,
            depth + 1,
            &new_prefix,
            i == children.len() - 1,
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::{ProcessNode, ProcessState};

    fn create_test_node(pid: i32, ppid: i32, name: &str) -> ProcessNode {
        ProcessNode {
            pid,
            ppid,
            name: name.to_string(),
            state: ProcessState::Running,
            children: None,
        }
    }

    #[test]
    fn test_print_tree_single_node() {
        let mut tree = HashMap::new();
        tree.insert(1, create_test_node(1, 0, "init"));

        // Should not panic
        let result = print_tree_with_pid(&tree, 1, 0, "", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_tree_with_children() {
        let mut tree = HashMap::new();

        let mut parent = create_test_node(1, 0, "init");
        parent.add_child(2);
        parent.add_child(3);
        tree.insert(1, parent);

        tree.insert(2, create_test_node(2, 1, "child1"));
        tree.insert(3, create_test_node(3, 1, "child2"));

        // Should not panic
        let result = print_tree_with_pid(&tree, 1, 0, "", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_tree_nested() {
        let mut tree = HashMap::new();

        let mut parent = create_test_node(1, 0, "init");
        parent.add_child(2);
        tree.insert(1, parent);

        let mut child = create_test_node(2, 1, "child");
        child.add_child(3);
        tree.insert(2, child);

        tree.insert(3, create_test_node(3, 2, "grandchild"));

        // Should not panic with nested structure
        let result = print_tree_with_pid(&tree, 1, 0, "", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_tree_missing_pid() {
        let tree: HashMap<i32, ProcessNode> = HashMap::new();

        // Requesting non-existent PID should return Ok (gracefully handles missing node)
        let result = print_tree_with_pid(&tree, 999, 0, "", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_tree_multiple_children() {
        let mut tree = HashMap::new();

        let mut parent = create_test_node(1, 0, "parent");
        for i in 2..=5 {
            parent.add_child(i);
        }
        tree.insert(1, parent);

        for i in 2..=5 {
            tree.insert(i, create_test_node(i, 1, &format!("child{}", i)));
        }

        // Should handle multiple children
        let result = print_tree_with_pid(&tree, 1, 0, "", true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_tree_with_prefix() {
        let mut tree = HashMap::new();
        tree.insert(1, create_test_node(1, 0, "test"));

        // Test with various prefix strings
        let result = print_tree_with_pid(&tree, 1, 1, "  ", false);
        assert!(result.is_ok());

        let result = print_tree_with_pid(&tree, 1, 2, "│ ", true);
        assert!(result.is_ok());
    }
}
