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
