pub mod common;
pub mod cmd;
pub mod device;
pub mod networks;
pub mod packet;

use std::sync::atomic::Ordering;
use env_logger;
use libc;
use crate::common::utils;
use crate::cmd::cli;
use crate::networks::network;

extern "C" fn handle_signal(_: libc::c_int) {
    network::INTERRUPTED.store(true, Ordering::Relaxed);
}

pub fn run_app() {
    env_logger::init();

    if !utils::is_root() {
        panic!("Please run as root");
    }


    unsafe {
        libc::signal(libc::SIGINT, handle_signal as libc::sighandler_t);
        libc::signal(libc::SIGTERM, handle_signal as libc::sighandler_t);
    }

    match cli::get_args().unwrap() {
        cli::Args::Client(client) => network::connect(&client.remote_addr, client.port, client.default_route, &client.key),
        cli::Args::Server(server) => network::serve(server.port, &server.key, server.dns),
    }

    println!("SIGINT/SIGTERM captured. Exit.");
}
