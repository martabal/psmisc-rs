use super::process::ProcessNode;
use std::{collections::HashMap, error::Error};

pub fn print_tree_with_pid<S: ::std::hash::BuildHasher>(
    tree: &HashMap<i32, ProcessNode, S>,
    pid: i32,
    depth: i32,
    prefix: &str,
    is_last: bool,
) -> Result<(), Box<dyn Error>> {
    if let Some(node) = tree.get(&pid) {
        let node_info = format!("{}({})", node.name, node.pid);
        if depth == 0 {
            println!("{prefix}{node_info}");
        } else if let Some(v) = &node.children
            && !v.is_empty()
            && !is_last
        {
            println!("{prefix}├─┬─{node_info}");
        } else if !is_last {
            println!("{prefix}├───{node_info}");
        } else {
            println!("{prefix}└───{node_info}");
        }

        if node.children.is_none() {
            return Ok(());
        }

        let new_prefix = if depth == 0 {
            format!("{prefix}  ")
        } else if is_last {
            prefix.to_string()
        } else {
            format!("{prefix}│ ")
        };

        if let Some(children) = &node.children {
            for (child_index, child_pid) in children.iter().enumerate() {
                let _ = print_tree_with_pid(
                    tree,
                    *child_pid,
                    depth + 1,
                    &new_prefix,
                    child_index == children.len() - 1,
                );
            }
        }
    }
    Ok(())
}
