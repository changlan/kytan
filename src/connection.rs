use std::net::{SocketAddr, UdpSocket};
use std::os::unix::io::AsRawFd;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering, ATOMIC_BOOL_INIT};
use std::io::{Write, Read};
use mio;
use dns_lookup;
use bincode::SizeLimit;
use bincode::rustc_serialize::{encode, decode};
use tuntap;
use utils;
use snap;

pub static INTERRUPTED: AtomicBool = ATOMIC_BOOL_INIT;

#[derive(RustcEncodable, RustcDecodable, PartialEq, Debug)]
enum Message {
    Request,
    Response { id: u8 },
    Data { id: u8, data: Vec<u8> },
}

const TUN: mio::Token = mio::Token(0);
const SOCK: mio::Token = mio::Token(1);

fn resolve(host: &str, port: u16) -> Result<SocketAddr, String> {
    let mut ip_list = try!(dns_lookup::lookup_host(host).map_err(|_| "dns_lookup::lookup_host"));
    let ip = ip_list.next().unwrap().unwrap();
    Ok(SocketAddr::new(ip, port))
}

fn initiate(socket: &UdpSocket, addr: &SocketAddr) -> Result<u8, String> {
    let req_msg = Message::Request;
    let encoded_req_msg: Vec<u8> = try!(encode(&req_msg, SizeLimit::Infinite)
        .map_err(|e| e.to_string()));

    let mut remaining_len = encoded_req_msg.len();
    while remaining_len > 0 {
        let sent_bytes = try!(socket.send_to(&encoded_req_msg, addr)
            .map_err(|e| e.to_string()));
        remaining_len -= sent_bytes;
    }
    info!("Request sent to {}.", addr);

    let mut buf = [0u8; 1600];
    let (len, recv_addr) = try!(socket.recv_from(&mut buf).map_err(|e| e.to_string()));
    assert_eq!(&recv_addr, addr);
    info!("Response received from {}.", addr);

    let resp_msg: Message = try!(decode(&buf[0..len]).map_err(|e| e.to_string()));
    match resp_msg {
        Message::Response { id } => Ok(id),
        _ => Err(format!("Invalid message {:?} from {}", resp_msg, addr)),
    }
}


pub fn connect(pass: &str, host: &str, port: u16) {
    info!("Working in client mode.");
    let remote_addr = resolve(host, port).unwrap();
    info!("Remote server: {}", remote_addr);

    let local_addr: SocketAddr = "0.0.0.0:0".parse::<SocketAddr>().unwrap();
    let socket = UdpSocket::bind(&local_addr).unwrap();

    let id = initiate(&socket, &remote_addr).unwrap();
    info!("Session established. Assigned IP address: 10.10.10.{}.", id);

    info!("Bringing up TUN device tun1.");
    let mut tun = tuntap::Tun::create(1);
    let tun_rawfd = tun.as_raw_fd();
    tun.up(id);
    let tunfd = mio::unix::EventedFd(&tun_rawfd);
    info!("TUN device tun1 initialized. Internal IP: 10.10.10.{}/24.",
          id);

    let poll = mio::Poll::new().unwrap();
    info!("Setting up TUN device for polling.");
    poll.register(&tunfd, TUN, mio::Ready::readable(), mio::PollOpt::level()).unwrap();

    info!("Setting up socket for polling.");
    let sockfd = mio::udp::UdpSocket::from_socket(socket).unwrap();
    poll.register(&sockfd, SOCK, mio::Ready::readable(), mio::PollOpt::level()).unwrap();

    let mut events = mio::Events::with_capacity(1024);
    let mut buf = [0u8; 1600];

    // RAII so ignore unused variable warning
    let _gw = utils::DefaultGateway::create("10.10.10.1", &format!("{}", remote_addr.ip()));
    let mut encoder = snap::Encoder::new();
    let mut decoder = snap::Decoder::new();

    info!("Ready for transmission.");

    loop {
        if INTERRUPTED.load(Ordering::Relaxed) {
            break;
        }

        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                SOCK => {
                    let (len, addr) = sockfd.recv_from(&mut buf).unwrap().unwrap();
                    let msg: Message = decode(&buf[0..len]).unwrap();
                    match msg {
                        Message::Request |
                        Message::Response { id: _ } => {
                            panic!("Invalid message {:?} from {}", msg, addr);
                        }
                        Message::Data { id: _, data } => {
                            let decompressed_data = decoder.decompress_vec(&data).unwrap();
                            let data_len = decompressed_data.len();
                            let sent_len = tun.write(&decompressed_data).unwrap();
                            assert_eq!(sent_len, data_len);
                        }
                    }
                }
                TUN => {
                    let len: usize = tun.read(&mut buf).unwrap();
                    let data = &buf[0..len];

                    let msg = Message::Data {
                        id: id,
                        data: encoder.compress_vec(data).unwrap(),
                    };
                    let encoded_msg = encode(&msg, SizeLimit::Infinite).unwrap();
                    sockfd.send_to(&encoded_msg, &remote_addr).unwrap().unwrap();
                }
                _ => unreachable!(),
            }
        }
    }
}

pub fn serve(pass: &str, port: u16) {
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
    let tunfd = mio::unix::EventedFd(&tun_rawfd);
    info!("TUN device tun0 initialized. Internal IP: 10.10.10.1/24.");

    let addr = format!("0.0.0.0:{}", port).parse().unwrap();
    let sockfd = mio::udp::UdpSocket::bind(&addr).unwrap();
    info!("Listening on: 0.0.0.0:{}.", port);

    let poll = mio::Poll::new().unwrap();
    poll.register(&sockfd, SOCK, mio::Ready::readable(), mio::PollOpt::level()).unwrap();
    poll.register(&tunfd, TUN, mio::Ready::readable(), mio::PollOpt::level()).unwrap();

    let mut events = mio::Events::with_capacity(1024);
    let mut available_ids: Vec<u8> = (2..254).collect();
    let mut client_map: HashMap<u8, SocketAddr> = HashMap::new();

    let mut buf = [0u8; 1600];
    let mut encoder = snap::Encoder::new();
    let mut decoder = snap::Decoder::new();
    info!("Ready for transmission.");

    loop {
        if INTERRUPTED.load(Ordering::Relaxed) {
            break;
        }

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
                            warn!("Invalid message {:?} from {}", msg, addr)
                        }
                        Message::Data { id: _, data } => {
                            let decompressed_data = decoder.decompress_vec(&data).unwrap();
                            let data_len = decompressed_data.len();
                            let sent_len = tun.write(&decompressed_data).unwrap();
                            assert_eq!(sent_len, data_len);
                        }
                    }
                }
                TUN => {
                    let len: usize = tun.read(&mut buf).unwrap();
                    let data = &buf[0..len];
                    let client_id: u8 = data[19];

                    match client_map.get(&client_id) {
                        None => warn!("Unknown IP packet from TUN for client {}.", client_id),
                        Some(addr) => {
                            let msg = Message::Data {
                                id: client_id,
                                data: encoder.compress_vec(data).unwrap(),
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
