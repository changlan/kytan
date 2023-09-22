use clap;
use clap::{App, Arg, SubCommand};
use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;


#[derive(Debug, Clone)]
pub struct Server {
    pub bind_addr: String,
    pub port: u16,
    pub key: String,
    pub dns: IpAddr,
}

#[derive(Debug, Clone)]
pub struct Client {
    pub remote_addr: String,
    pub port: u16,
    pub key: String,
    pub default_route: bool,
}

#[derive(Debug, Clone)]
pub enum Args {
    Client(Client),
    Server(Server),
}

pub fn get_args() -> Result<Args, String> {
    let matches = App::new("kytan: High Performance Peer-to-Peer VPN")
        .version("1.0")
        .subcommand(
            SubCommand::with_name("server")
                .help("client mode")
                .arg(
                    Arg::with_name("bind")
                        .short("l")
                        .long("listen")
                        .default_value("0.0.0.0")
                        .help("set the listen address")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .default_value("9527")
                        .help("set the listen port")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("key")
                        .short("k")
                        .long("key")
                        .help("set the key for encryption communication")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("dns")
                        .short("d")
                        .long("dns")
                        .default_value("8.8.8.8")
                        .help("set dns for client, default 8.8.8.8")
                        .takes_value(true),
                ),
        )
        .subcommand(
            SubCommand::with_name("client")
                .help("server mode")
                .arg(
                    Arg::with_name("server")
                        .short("s")
                        .long("server")
                        .help("set the remote server address")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("port")
                        .short("p")
                        .long("port")
                        .help("set the remote port")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("key")
                        .short("k")
                        .long("key")
                        .help("set the key for encryption communication")
                        .takes_value(true),
                )
                .arg(
                    Arg::with_name("no-default-route")
                        .short("n")
                        .long("no-default-route")
                        .help("do not set default route"),
                ),
        )
        .get_matches();
    if let Some(matches) = matches.subcommand_matches("client") {
        let ip_str = matches
            .value_of("server")
            .ok_or_else(|| "can not find client host value")
            .unwrap();
        let port_str = matches
            .value_of("port")
            .ok_or_else(|| "can not find client port value")
            .unwrap();
        let key_str = matches
            .value_of("key")
            .ok_or_else(|| "can not find client key value")
            .unwrap();
        // let remote_addr = IpAddr::V4(Ipv4Addr::from_str(ip_str).map_err(|e| e.to_string())?);
        let port = port_str.parse::<u16>().map_err(|e| e.to_string())?;
        let default_route = match matches.is_present("no-default-route") {
            false => true,
            true => false,
        };
        Ok(Args::Client(Client {
            remote_addr: ip_str.to_string(),
            port: port,
            key: key_str.to_string(),
            default_route: default_route,
        }))
    } else if let Some(matches) = matches.subcommand_matches("server") {
        let ip_str = matches
            .value_of("bind")
            .ok_or_else(|| "can not find server host value")
            .unwrap();
        let port_str = matches
            .value_of("port")
            .ok_or_else(|| "can not find server port value")
            .unwrap();
        let key_str = matches
            .value_of("key")
            .ok_or_else(|| "can not find server key value")
            .unwrap();
        let dns = matches
            .value_of("dns")
            .ok_or_else(|| "can not find dns value")?;
        // let bind_addr = IpAddr::V4(Ipv4Addr::from_str(ip_str).map_err(|e| e.to_string())?);
        let dns = IpAddr::V4(Ipv4Addr::from_str(dns).map_err(|e| e.to_string())?);
        let port = port_str.parse::<u16>().map_err(|e| e.to_string())?;
        Ok(Args::Server(Server {
            bind_addr: ip_str.to_string(),
            port: port,
            key: key_str.to_string(),
            dns: dns,
        }))
    } else {
        unimplemented!()
    }
}
