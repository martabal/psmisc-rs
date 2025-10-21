use std::{process, sync::atomic::Ordering};

use clap::Parser;
use nix::{
    sys::signal::{Signal, kill},
    unistd::Pid,
};
#[cfg(feature = "rayon")]
use rayon::prelude::*;

use faulx::{
    cli::{FaulxArgs, MAX_NAMES},
    macros::QUIET,
    processes::{OptionsPids, list_pids},
    qprintln,
    signals::{list_signals, parse_signal},
};

fn main() {
    let args = FaulxArgs::parse();

    QUIET.store(args.quiet, Ordering::Relaxed);

    if args.list {
        println!("{}", list_signals());
        return;
    }

    if args.process_names.len() > MAX_NAMES {
        qprintln!(
            "{}: Maximum number of names is {} and you gave {}",
            env!("CARGO_PKG_NAME"),
            MAX_NAMES,
            args.process_names.len(),
        );
        process::exit(1);
    }

    let sig = args.signal.as_deref().map_or(Signal::SIGTERM, |name| {
        parse_signal(name).map_or_else(
            || {
                qprintln!("{name}: unknown signal");
                process::exit(1);
            },
            |s| s,
        )
    });

    for process_name in &args.process_names {
        let opts = OptionsPids {
            use_group: args.process_group,
            younger_than: args.younger_than,
            older_than: args.older_than,
        };
        let pids = match list_pids(process_name, &opts) {
            Ok(pids) => pids,
            Err(e) => {
                qprintln!("Error: {e}");
                continue;
            }
        };

        if pids.is_empty() {
            qprintln!("{process_name}: no process found");
            process::exit(1);
        }

        #[cfg(feature = "rayon")]
        let pids_iter = pids.par_iter();
        #[cfg(not(feature = "rayon"))]
        let pids_iter = pids.iter();
        if args.wait {
            let start = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(20);

            loop {
                let alive_pids: Vec<i32> = pids_iter
                    .clone()
                    .filter_map(|&pid| kill_process(pid, sig, args.verbose, process_name))
                    .collect();

                if alive_pids.is_empty() {
                    break;
                }

                if start.elapsed() > timeout {
                    qprintln!(
                        "Timeout: {} process(es) still alive after {} seconds",
                        alive_pids.len(),
                        timeout.as_secs()
                    );
                    if args.verbose {
                        qprintln!("Still alive: {:?}", alive_pids);
                    }
                    process::exit(1);
                }

                std::thread::sleep(std::time::Duration::from_millis(1000));
            }
        } else {
            #[cfg(feature = "rayon")]
            let pids_iter = pids.par_iter();
            #[cfg(not(feature = "rayon"))]
            let pids_iter = pids.iter();

            pids_iter.for_each(|pid| {
                kill_process(*pid, sig, args.verbose, process_name);
            });
        }
    }
}

fn kill_process(pid: i32, sig: Signal, verbose: bool, process_name: &str) -> Option<i32> {
    match kill(Pid::from_raw(pid), sig) {
        Ok(()) => {
            if verbose {
                println!("Killed {process_name}({pid}) with signal {sig}");
            }
            Some(pid)
        }
        Err(nix::errno::Errno::ESRCH) => None,
        Err(err) => {
            qprintln!("Failed to send signal to {pid}: {err}");
            None
        }
    }
}
