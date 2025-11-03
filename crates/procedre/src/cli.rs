use clap::Parser;
use helpers::STYLES;

pub const MAX_NAMES: usize = std::mem::size_of::<usize>() * 8;

#[derive(Parser, Debug)]
#[command(styles = STYLES)]
#[command(author, version, about, long_about = None)]
pub struct ProcedreArgs {}
