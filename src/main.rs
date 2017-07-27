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

extern crate getopts;
extern crate mio;
extern crate libc;

#[macro_use]
extern crate serde_derive;
extern crate bincode;

extern crate env_logger;
extern crate dns_lookup;
extern crate snap;
extern crate rand;
extern crate transient_hashmap;

extern crate nix;
#[macro_use]
extern crate log;
extern crate ring;
use std::sync::atomic::Ordering;
use ring::rand::{SystemRandom, SecureRandom};

mod device;
mod utils;
mod network;
mod packet;

fn print_usage(program: &str, opts: getopts::Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

extern "C" fn handle_signal(_: i32) {
    network::INTERRUPTED.store(true, Ordering::Relaxed);
}

fn main() {
    env_logger::init().unwrap();

    if !utils::is_root() {
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
	
	let (sealing_key, opening_key) = network::key_derivation();
	let mut nonce = vec![0; 12];
	let rand = SystemRandom::new();
	rand.fill(&mut nonce).unwrap();
	
    match mode.as_ref() {
        "s" => network::serve(port, &sealing_key, &opening_key, &nonce),
        "c" => {
            let host = matches.opt_str("h").unwrap();
            network::connect(&host, port, true, &sealing_key, &opening_key, &nonce)
        }
        _ => unreachable!(),
    };

    println!("SIGINT/SIGTERM captured. Exit.");
}
