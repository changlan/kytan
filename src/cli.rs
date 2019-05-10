use clap::{App, Arg, Error,SubCommand};
use clap;
use std::{ffi::OsString,path::PathBuf};

#[derive(Debug,Clone)]
pub struct Args {
    pub mode: String,
    pub port: u16,
    pub host: String,
    pub key: String,
    pub dns: String,
}


pub fn get_args() -> Result<Args,String> {
    let matches = App::new("My Super Program")
                        .version("1.0")
                        .author("Kevin K. <kbknapp@gmail.com>")
                        .about("Does awesome things")
                        .arg(Arg::with_name("config")
                            .long("config")
                            .value_name("FILE")
                            .help("Sets a custom config file")
                            .takes_value(true))
                        .arg(Arg::with_name("client")
                            .short("c")
                            .long("client")
                            .help("Sets the mode as client"))
                        .arg(Arg::with_name("server")
                            .short("s")
                            .long("server")
                            .help("Sets the mode as server"))
                        .arg(Arg::with_name("host")
                            .short("s")
                            .long("server")
                            .help("set the target address or bind address")
                            .takes_value(true))
                        .arg(Arg::with_name("port")
                            .short("p")
                            .long("port")
                            .help("set the target port")
                            .takes_value(true))
                        .arg(Arg::with_name("key")
                            .short("k")
                            .long("key")
                            .help("password of your remote server")
                            .takes_value(true))
                        .arg(Arg::with_name("dns")
                            .short("d")
                            .long("dns")
                            .default_value("8.8.8.8")
                            .help("set the dns, default value 8.8.8.8")
                            .takes_value(true)
                        )
                        .get_matches();
    // if matches.is_present("config") {
    //     let matches = load_yaml!(matches.value_of("config").map_err(|e| e.to_string())?);
    // }
    
    let mut mode = "";
    if matches.is_present("server") {
        mode = "server";
    } else {
        if matches.is_present("client") {
            mode = "client";
        } else {
            panic!("please select work mode");
        }
    }

    let mut host = "";
    if matches.is_present("host") {
        host = matches.value_of("host").unwrap();
    }

    let mut port: u16 = 0;
    if matches.is_present("port") {
        let port_str = matches.value_of("port").unwrap();
        port = port_str.parse::<u16>().map_err(|e| e.to_string())?;
    }

    let mut key = "";
    if matches.is_present("key") {
        key = matches.value_of("key").unwrap();
    }
    Ok(Args {
        mode: mode.to_string(),
        port: port,
        host: host.to_string(),
        key: key.to_string(),
        dns: matches.value_of("dns").unwrap().to_string(),
    })
}