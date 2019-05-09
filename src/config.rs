use clap::{App, Arg, Error};
use std::{ffi::OsString,path::PathBuf};

#[derive(Debug,Clone)]
pub struct Args {
    pub server: bool,
    pub client: bool,
    pub port: u16,
    pub host: String,
    pub key: String,
}

