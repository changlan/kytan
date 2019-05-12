// Copyright 2016-2017 Chang Lan
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod device;
mod utils;
mod network;
mod packet;
mod cli;


use std::sync::atomic::Ordering;
use getopts;
use env_logger;
use libc;

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

extern "C" fn handle_signal(_: libc::c_int) {
    network::INTERRUPTED.store(true, Ordering::Relaxed);
}

fn main() {
    env_logger::init();

    if !utils::is_root() {
        panic!("Please run as root");
    }

    // let mut opts = getopts::Options::new();
    // opts.reqopt("m", "mode", "mode (server or client)", "[s|c]");
    // opts.optopt("p", "port", "UDP port to listen/connect", "PORT");
    // opts.optopt("h", "host", "remote host to connect (client mode)", "HOST");
    // opts.optopt("s", "secret", "shared secret", "PASSWORD");

    // let args: Vec<String> = std::env::args().collect();
    // let program = args[0].clone();

    // let matches = match opts.parse(&args[1..]) {
    //     Ok(m) => m,
    //     Err(_) => {
    //         print_usage(&program, opts);
    //         return;
    //     }
    // };
    // let args = cli::get_args().unwrap();

    // let mode = matches.opt_str("m").unwrap();
    // let port: u16 = matches.opt_str("p").unwrap_or(String::from("8964")).parse().unwrap();
    // let secret = matches.opt_str("s").unwrap();

    unsafe {
        libc::signal(libc::SIGINT, handle_signal as libc::sighandler_t);
        libc::signal(libc::SIGTERM, handle_signal as libc::sighandler_t);
    }

    match cli::get_args().unwrap() {
        cli::Args::Client(client) => network::connect(&client.remote_addr, client.port, client.default_route, &client.key),
        cli::Args::Server(server) => network::serve(server.port, &server.key, server.dns),
    }

    // match args.mode.as_ref() {
    //     "server" => network::serve(args.port, &args.key,&args.dns),
    //     "client" => {
    //         network::connect(&args.host, args.port, true, &args.key)
    //     }
    //     _ => unreachable!(),
    // };

    println!("SIGINT/SIGTERM captured. Exit.");
}
