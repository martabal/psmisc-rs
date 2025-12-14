use clap::Parser;
use helpers::STYLES;

#[derive(Parser, Debug)]
#[command(styles = STYLES)]
#[command(author, version, about, long_about = None)]
pub struct FuserArgs {
    /// Files or directories to check
    #[arg(required = true)]
    pub names: Vec<String>,

    /// Kill processes accessing the file
    #[arg(short = 'k', long)]
    pub kill: bool,

    /// Signal to send (default: SIGKILL)
    #[arg(short = 's', long)]
    pub signal: Option<String>,

    /// Verbose output
    #[arg(short = 'v', long)]
    pub verbose: bool,

    /// Silent operation
    #[arg(short = 'q', long)]
    pub quiet: bool,
}
