use copy::cli::move_args::MoveArgs;
use copy::core::move_op::{move_multiple, move_path};
use copy::error::CliError;
use signal_hook::consts::signal::*;
use signal_hook::iterator::Signals;
use std::process;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn main() {
    let args = MoveArgs::parse();

    let (sources, destination, mut options) = match args.validate() {
        Ok(validated) => validated,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };

    let abort = Arc::new(AtomicBool::new(false));
    options.abort = abort.clone();

    let mut signals = Signals::new([SIGINT, SIGTERM])
        .map_err(CliError::Io)
        .unwrap_or_else(|e| {
            eprintln!("Failed to setup signal handler: {}", e);
            process::exit(1);
        });

    std::thread::spawn({
        let abort = abort.clone();
        move || {
            for sig in signals.forever() {
                match sig {
                    SIGINT | SIGTERM => {
                        abort.store(true, Ordering::Relaxed);
                    }
                    _ => unreachable!(),
                }
            }
        }
    });

    let result = if sources.len() == 1 {
        move_path(&sources[0], &destination, &options)
    } else {
        move_multiple(sources, destination, &options)
    };

    if let Err(e) = result {
        if abort.load(Ordering::Relaxed) {
            eprintln!("\nOperation interrupted");
            eprintln!(
                "Already-moved files remain at their destination; sources of in-progress moves are left in place"
            );
            process::exit(130); // SIGINT
        } else {
            eprintln!("Error moving file: {}", e);
            process::exit(1);
        }
    }
}
