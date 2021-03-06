#[cfg(target_os = "linux")]
use std::os::raw::c_uint;
use std::{
    os::{raw::c_int, unix::process::CommandExt},
    process::{self, Command},
};

use coreutils_core::{
    libc::ENOENT,
    os::process::priority::{get_priority, set_priority, PRIO_PROCESS},
};

mod cli;

#[cfg(target_os = "linux")]
const P_PROCESS: c_uint = PRIO_PROCESS as c_uint;
#[cfg(not(target_os = "linux"))]
const P_PROCESS: c_int = PRIO_PROCESS;

fn main() {
    let matches = cli::create_app().get_matches();

    let adjustment: c_int = {
        // Ok to unwrap because it's set with default value, so it will always have a value.
        let str_n = matches.value_of("adjustment").unwrap();
        match str_n.parse() {
            Ok(n) => n,
            Err(err) => {
                eprintln!("nice: {} is not a valid number: {}", str_n, err);
                process::exit(125);
            },
        }
    };

    // Ok to unwrap: COMMAND is required
    let mut cmd = matches.values_of("COMMAND").unwrap();
    // Ok to unwrap: Since COMMAND is required, there must be the first value
    let command = cmd.next().unwrap();
    let args: Vec<_> = cmd.collect();

    let mut niceness = match get_priority(P_PROCESS, 0) {
        Ok(nice) => nice,
        Err(err) => {
            eprintln!("nice: failed to get priority: {}", err);
            drop(args);
            process::exit(125);
        },
    };

    niceness += adjustment;

    if let Err(err) = set_priority(P_PROCESS, 0, niceness) {
        eprintln!("nice: failed to set priority: {}", err);
        drop(args);
        process::exit(125);
    }

    let err = Command::new(command).args(args).exec();

    if err.raw_os_error().unwrap() as c_int == ENOENT {
        eprintln!("nice: '{}': {}", command, err);
        process::exit(127);
    } else {
        eprintln!("nice: {}", err);
        process::exit(126);
    }
}
