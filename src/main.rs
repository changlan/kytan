extern crate libc;
extern crate getopts;
extern crate mio;
extern crate rustc_serialize;
extern crate bincode;
extern crate env_logger;
extern crate dns_lookup;
extern crate snap;

#[macro_use]
extern crate nix;
#[macro_use]
extern crate log;

use std::sync::atomic::Ordering;

mod tuntap;
mod utils;
mod connection;

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

extern "C" fn handle_signal(_: i32) {
    connection::INTERRUPTED.store(true, Ordering::Relaxed);
}

fn main() {
    env_logger::init().unwrap();

    if unsafe { libc::geteuid() != 0 } {
        panic!("Please run as root");
    }

    let mut opts = getopts::Options::new();
    opts.reqopt("m", "mode", "mode (server or client)", "[s|c]");
    opts.optopt("p", "port", "UDP port to listen/connect", "PORT");
    opts.optopt("h", "host", "remote host to connect (client mode)", "HOST");

    let args: Vec<String> = std::env::args().collect();
    let program = args[0].clone();

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            print_usage(&program, opts);
            return;
        }
    };

    let mode = matches.opt_str("m").unwrap();
    let port: u16 = matches.opt_str("p").unwrap_or(String::from("8964")).parse().unwrap();

    let sig_action =
        nix::sys::signal::SigAction::new(nix::sys::signal::SigHandler::Handler(handle_signal),
                                         nix::sys::signal::SaFlags::empty(),
                                         nix::sys::signal::SigSet::empty());
    unsafe {
        nix::sys::signal::sigaction(nix::sys::signal::SIGINT, &sig_action).unwrap();
        nix::sys::signal::sigaction(nix::sys::signal::SIGTERM, &sig_action).unwrap();
    }

    match mode.as_ref() {
        "s" => connection::serve(port),
        "c" => {
            let host = matches.opt_str("h").unwrap();
            connection::connect(&host, port)
        }
        _ => unreachable!(),
    };

    println!("SIGINT/SIGTERM captured. Exit.");
}
