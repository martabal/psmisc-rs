use clap::Parser;
use procedre::{
    cli::ProcedreArgs,
    output,
    process::{build_process_tree, read_process},
};

fn main() {
    ProcedreArgs::parse();

    let process_list = match read_process() {
        Ok(list) => list,
        Err(e) => {
            eprintln!("Failed to read processes: {e}");
            std::process::exit(1);
        }
    };

    let process_tree = match build_process_tree(&process_list) {
        Ok(tree) => tree,
        Err(e) => {
            eprintln!("Failed to build process tree: {e}");
            std::process::exit(1);
        }
    };

    if let Err(e) = output::print_tree_with_pid(&process_tree, 1, 0, "", true) {
        eprintln!("Failed to print tree: {e}");
        std::process::exit(1);
    }
}
