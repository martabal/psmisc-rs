use clap::Parser;
use helpers::STYLES;

pub const MAX_NAMES: usize = std::mem::size_of::<usize>() * 8;

#[derive(Parser, Debug)]
#[command(styles = STYLES)]
#[command(author, version, about, long_about = None)]
pub struct FaulxArgs {
    /// process name to kill
    #[arg(required_unless_present = "list")]
    pub process_names: Vec<String>,

    /// list all known signal names
    #[arg(short = 'l', long)]
    pub list: bool,

    /// Send this signal instead of SIGTERM
    #[arg(short = 's', long)]
    pub signal: Option<String>,

    /// Don't print complaints
    #[arg(short = 'q', long)]
    pub quiet: bool,

    /// kill process group instead of process
    #[arg(short = 'g', long)]
    pub process_group: bool,

    /// Kill process younger than TIME
    #[arg(short = 'y', long)]
    pub younger_than: Option<humantime::Duration>,

    /// Kill process older than TIME
    #[arg(short = 'o', long)]
    pub older_than: Option<humantime::Duration>,

    /// Wait for process to die
    #[arg(short = 'w', long)]
    pub wait: bool,

    /// Ask for confirmation before killing
    #[arg(short = 'i', long)]
    pub interactive: bool,

    /// Case insensitive process name match
    #[arg(short = 'I', long)]
    pub ignore_case: bool,

    /// Kill only process(es) running as USER
    #[arg(short = 'u', long)]
    pub user: Option<String>,

    /// Report if the signal was successfully sent
    #[arg(long)]
    pub verbose: bool,

    /// Interpret process_names as an extended regular expression
    #[cfg(feature = "regex")]
    #[arg(short = 'r', long)]
    pub regexp: bool,

    /// Match processes that belong to the same namespaces as PID
    #[arg(short = 'n', long)]
    pub namespace: Option<String>,
}
