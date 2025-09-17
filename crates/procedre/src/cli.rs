use clap::{
    Parser,
    builder::{Styles, styling::AnsiColor},
    command,
};

pub const MAX_NAMES: usize = std::mem::size_of::<usize>() * 8;

const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Green.on_default().bold())
    .usage(AnsiColor::Green.on_default().bold())
    .literal(AnsiColor::Cyan.on_default().bold())
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Parser, Debug)]
#[command(styles = STYLES)]
#[command(author, version, about, long_about = None)]
pub struct ProcedreArgs {}
