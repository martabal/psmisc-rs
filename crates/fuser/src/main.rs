use std::{collections::HashSet, process};

use clap::Parser;
use fuser::{cli::FuserArgs, finder};
use helpers::parse_signal;
use nix::{
    sys::signal::{Signal, kill},
    unistd::Pid,
};

fn main() {
    let args = FuserArgs::parse();

    let mut all_pids = HashSet::new();

    for name in &args.names {
        match finder::find_processes_using_file(name) {
            Ok(infos) => {
                if infos.is_empty() && !args.quiet {
                    eprintln!("{name}: No process using this file");
                    continue;
                }

                if !args.quiet {
                    print!("{name}:");
                }

                for info in &infos {
                    all_pids.insert(info.pid);
                    if !args.quiet {
                        if args.verbose {
                            print!(" {}({})[{:?}]", info.comm, info.pid, info.accesses);
                        } else {
                            print!(" {}", info.pid);
                        }
                    }
                }

                if !args.quiet {
                    println!();
                }
            }
            Err(e) => {
                if !args.quiet {
                    eprintln!("{name}: {e}");
                }
                process::exit(1);
            }
        }
    }

    if args.kill {
        let signal = args.signal.as_ref().map_or(Signal::SIGKILL, |sig_name| {
            parse_signal(sig_name).unwrap_or_else(|| {
                eprintln!("Unknown signal: {sig_name}");
                process::exit(1);
            })
        });

        for pid in all_pids {
            if let Err(e) = kill(Pid::from_raw(pid), signal) {
                if !args.quiet {
                    eprintln!("Failed to kill process {pid}: {e}");
                }
            } else if args.verbose && !args.quiet {
                println!("Killed process {pid}");
            }
        }
    }
}
