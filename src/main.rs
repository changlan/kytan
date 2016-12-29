extern crate libc;
extern crate getopts;
extern crate mio;
extern crate rustc_serialize;
extern crate bincode;
extern crate resolve;
extern crate byteorder;
extern crate pnet;

#[macro_use]
extern crate log;
extern crate env_logger;

mod tuntap;
mod utils;

use std::env;
use std::os::unix::io::AsRawFd;
use std::collections::HashMap;
use std::net::SocketAddr;
use getopts::Options;
use mio::*;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};
use pnet::packet::ipv4::Ipv4Packet;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
enum Message {
    Request,
    Response { id: u8 },
    Data { id: u8, data: Vec<u8> },
}

const TUN: Token = Token(0);
const SOCK: Token = Token(1);

fn connect(pass: &str, host: &str, port: u16) {
    info!("Working in client mode.");
    let remote_ip = resolve::resolve_host(host).unwrap().next().unwrap();
    let remote_addr = SocketAddr::new(remote_ip, port);
    info!("Remote server: {}", remote_addr);

    let local_addr: SocketAddr = "0.0.0.0:0".parse().unwrap();
    let socket = std::net::UdpSocket::bind(&local_addr).unwrap();

    let req_msg = Message::Request;
    let encoded_req_msg = encode(&req_msg, SizeLimit::Infinite).unwrap();
    socket.send_to(&encoded_req_msg, &remote_addr).unwrap();
    info!("Request sent to {}.", remote_addr);

    let mut buf = [0u8; 1600];
    let (len, addr) = socket.recv_from(&mut buf).unwrap();
    assert_eq!(addr, remote_addr);
    info!("Response received from {}.", remote_addr);

    let resp_msg: Message = decode(&buf[0..len]).unwrap();
    let id = match resp_msg {
        Message::Response { id } => id,
        _ => panic!("Invalid message {:?} from {}", resp_msg, remote_addr),
    };
    info!("Session established. Assigned IP address: 10.10.10.{}.", id);

    info!("Bringing up TUN device tun1.");
    let mut tun = tuntap::Tun::create(1);
    let tun_rawfd = tun.as_raw_fd();
    tun.up(id);
    let tunfd = unix::EventedFd(&tun_rawfd);
    info!("TUN device tun1 initialized. Internal IP: 10.10.10.{}/24.",
          id);

    let poll = Poll::new().unwrap();
    info!("Setting up TUN device for polling.");
    poll.register(&tunfd, TUN, Ready::readable(), PollOpt::edge()).unwrap();

    info!("Setting up socket for polling.");
    let sockfd = udp::UdpSocket::from_socket(socket).unwrap();
    poll.register(&sockfd, SOCK, Ready::readable(), PollOpt::edge()).unwrap();

    let mut events = Events::with_capacity(1024);
    info!("Ready for transmission.");

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                SOCK => {
                    let (len, addr) = sockfd.recv_from(&mut buf).unwrap().unwrap();
                    let msg: Message = decode(&buf[0..len]).unwrap();
                    match msg {
                        Message::Request |
                        Message::Response { id: _ } => {
                            panic!("Invalid message {:?} from {}", msg, addr)
                        }
                        Message::Data { id: _, data } => {
                            let data_len = data.len();
                            let sent_len = tun.write(&data).unwrap();
                            assert_eq!(sent_len, data_len);
                        }
                    }
                }
                TUN => {
                    let len: usize = tun.read(&mut buf).unwrap();
                    let data = &buf[0..len];
                    debug!("Data to {}. Len: {}", remote_addr, len);
                    assert_eq!(0x45, data[0]);

                    let msg = Message::Data {
                        id: id,
                        data: data.to_vec(),
                    };
                    let encoded_msg = encode(&msg, SizeLimit::Infinite).unwrap();
                    sockfd.send_to(&encoded_msg, &remote_addr).unwrap().unwrap();
                }
                _ => unreachable!(),
            }
        }
    }
}

fn serve(pass: &str, port: u16) {
    if cfg!(not(target_os = "linux")) {
        panic!("Server mode is only available in Linux!");
    }
    info!("Working in server mode.");

    info!("Enabling kernel's IPv4 forwarding.");
    utils::enable_ipv4_forwarding().unwrap();

    info!("Bringing up TUN device tun0.");
    let mut tun = tuntap::Tun::create(0);
    tun.up(1);

    let tun_rawfd = tun.as_raw_fd();
    let tunfd = unix::EventedFd(&tun_rawfd);
    info!("TUN device tun0 initialized. Internal IP: 10.10.10.1/24.");

    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
    let sockfd = udp::UdpSocket::bind(&addr).unwrap();
    info!("Listening on: 0.0.0.0:{}.", port);

    let poll = Poll::new().unwrap();
    poll.register(&sockfd, SOCK, Ready::readable(), PollOpt::edge()).unwrap();
    poll.register(&tunfd, TUN, Ready::readable(), PollOpt::edge()).unwrap();

    let mut events = Events::with_capacity(1024);
    let mut available_ids: Vec<u8> = (2..254).collect();
    let mut client_map: HashMap<u8, SocketAddr> = HashMap::new();

    let mut buf = [0u8; 1600];
    info!("Ready for transmission.");

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                SOCK => {
                    let (len, addr) = sockfd.recv_from(&mut buf).unwrap().unwrap();
                    let msg: Message = decode(&buf[0..len]).unwrap();
                    match msg {
                        Message::Request => {
                            let client_id: u8 = available_ids.pop().unwrap();
                            client_map.insert(client_id, addr);

                            info!("Got request from {}. Assigning IP address: 10.10.10.{}.",
                                  addr,
                                  client_id);

                            let reply = Message::Response { id: client_id };
                            let encoded_reply = encode(&reply, SizeLimit::Infinite).unwrap();
                            let sent_len = sockfd.send_to(&encoded_reply, &addr).unwrap().unwrap();
                            assert_eq!(sent_len, encoded_reply.len());
                        }
                        Message::Response { id: _ } => {
                            panic!("Invalid message {:?} from {}", msg, addr)
                        }
                        Message::Data { id: _, data } => {
                            let data_len = data.len();
                            let pkt = Ipv4Packet::new(&data).unwrap();
                            debug!("Data from {}. Len: {}. Source: {}. \
                                   Destination: {}",
                                   addr,
                                   data_len,
                                   pkt.get_source(),
                                   pkt.get_destination());
                            assert_eq!(0x45, data[0]);

                            let sent_len = tun.write(&data).unwrap();
                            assert_eq!(sent_len, data_len);
                        }
                    }
                }
                TUN => {
                    let len: usize = tun.read(&mut buf).unwrap();
                    let data = &buf[0..len];
                    let client_id: u8 = data[19];

                    match client_map.get(&client_id) {
                        None => {
                            let pkt = Ipv4Packet::new(data).unwrap();
                            warn!("Unknown IP packet from TUN for client {}. Source: {}. \
                                   Destination: {}",
                                  client_id,
                                  pkt.get_source(),
                                  pkt.get_destination())
                        }
                        Some(addr) => {
                            let msg = Message::Data {
                                id: client_id,
                                data: data.to_vec(),
                            };
                            let encoded_msg = encode(&msg, SizeLimit::Infinite).unwrap();
                            sockfd.send_to(&encoded_msg, addr).unwrap().unwrap();
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
    }
}

fn main() {
    env_logger::init().unwrap();

    if unsafe { libc::geteuid() != 0 } {
        panic!("Please run as root");
    }

    let mut opts = Options::new();
    opts.reqopt("m", "mode", "mode (server or client)", "[s|c]");
    opts.reqopt("s", "secret", "shared secret", "PASS");
    opts.optopt("p", "port", "UDP port to listen/connect", "PORT");
    opts.optopt("r", "remote IP", "remote IP to connect", "IP");

    let args: Vec<String> = env::args().collect();
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
        "s" => serve(&pass, port),
        "c" => {
            let host = matches.opt_str("r").unwrap();
            connect(&pass, &host, port);
        }
        _ => unreachable!(),
    }
}
