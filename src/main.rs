extern crate libc;
extern crate getopts;
extern crate mio;
extern crate rustc_serialize;
extern crate bincode;
extern crate resolve;
extern crate byteorder;
extern crate pnet;
extern crate env_logger;

#[macro_use]
extern crate nix;
#[macro_use]
extern crate log;

mod tuntap;
mod utils;
mod connection;

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn main() {
    env_logger::init().unwrap();

    if unsafe { libc::geteuid() != 0 } {
        panic!("Please run as root");
    }

    let mut opts = getopts::Options::new();
    opts.reqopt("m", "mode", "mode (server or client)", "[s|c]");
    opts.reqopt("s", "secret", "shared secret", "PASS");
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
    let pass = matches.opt_str("s").unwrap();
    let port: u16 = matches.opt_str("p").unwrap_or(String::from("8964")).parse().unwrap();

    match mode.as_ref() {
        "s" => connection::serve(&pass, port),
        "c" => {
            let host = matches.opt_str("h").unwrap();
            connection::connect(&pass, &host, port)
        }
        _ => unreachable!(),
    };
}
