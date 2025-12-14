use std::{
    io::{self, Write},
    process,
    sync::atomic::Ordering,
};

use clap::Parser;
use helpers::{list_signals, macros::QUIET, parse_signal, qprintln};
use nix::{
    sys::signal::{Signal, kill},
    unistd::Pid,
};

use faulx::{
    change_user,
    cli::{FaulxArgs, MAX_NAMES},
    processes::{OptionsPids, list_pids},
};

fn main() {
    let args = FaulxArgs::parse();

    if let Some(user) = &args.user {
        change_user(user);
    }

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
        parse_signal(name).unwrap_or_else(|| {
            qprintln!("{name}: unknown signal");
            process::exit(1);
        })
    });

    for process_name in &args.process_names {
        let opts = OptionsPids {
            use_group: args.process_group,
            younger_than: args.younger_than,
            older_than: args.older_than,
            ignore_case: args.ignore_case,
            #[cfg(feature = "regex")]
            regexp: args.regexp,
            namespace: args.namespace.as_deref(),
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

        if args.wait {
            let start = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(20);

            loop {
                let alive_pids: Vec<i32> = pids
                    .iter()
                    .filter_map(|&pid| {
                        kill_process(pid, sig, args.verbose, process_name, args.interactive)
                    })
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
            for pid in &pids {
                kill_process(*pid, sig, args.verbose, process_name, args.interactive);
            }
        }
    }
}

fn kill_process(
    pid: i32,
    sig: Signal,
    verbose: bool,
    process_name: &str,
    interactive: bool,
) -> Option<i32> {
    if interactive {
        loop {
            print!("Kill {process_name}({pid}) ? (y/N) ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                eprintln!("Failed to read input.");
                continue;
            }

            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => break,
                "n" | "no" => {
                    return None;
                }
                _ => {
                    println!("Please enter 'y' or 'n'.");
                }
            }
        }
    }

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
