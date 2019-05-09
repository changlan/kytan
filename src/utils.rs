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

use std::process::Command;
use std::net::Ipv4Addr;
use libc;

const REMOTE: &str = "123.206.66.49";

pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

pub fn enable_ipv4_forwarding() -> Result<(), String> {
    let sysctl_arg = if cfg!(target_os = "linux") {
        "net.ipv4.ip_forward=1"
    } else if cfg!(target_os = "macos") {
        "net.inet.ip.forwarding=1"
    } else {
        unimplemented!()
    };
    info!("Enabling IPv4 Forwarding.");
    let status = Command::new("sysctl")
        .arg("-w")
        .arg(sysctl_arg)
        .status()
        .unwrap();
    if status.success() {
        Ok(())
    } else {
        Err(format!("sysctl: {}", status))
    }
}

#[test]
fn enable_ipv4_forwarding_test() {
    enable_ipv4_forwarding().unwrap();
}

pub enum RouteType {
    Net,
    Host,
}

pub struct DefaultGateway {
    origin: String,
    remote: String,
}

impl DefaultGateway {
    pub fn create(gateway: &str, remote: &str) -> DefaultGateway {
        let origin = get_default_gateway().unwrap_or(String::new());
        info!("Original default gateway: {}.", origin);
        if let Ok(_) = remote.parse::<Ipv4Addr>() {
            add_route(RouteType::Host, remote, &origin).unwrap();
        }
        set_default_gateway(gateway).unwrap();
        DefaultGateway {
            origin: origin,
            remote: String::from(remote),
        }
    }
}

impl Drop for DefaultGateway {
    fn drop(&mut self) {
    }
}

pub fn delete_route(route_type: RouteType, route: &str) -> Result<(), String> {
    let mode = match route_type {
        RouteType::Net => "-net",
        RouteType::Host => "-host",
    };
    info!("Deleting route: {} {}.", mode, route);
    let status = if cfg!(target_os = "linux") {
        Command::new("ip")
            .arg("r")
            .arg("d")
            .arg(route)
            .status()
            .unwrap()
    } else if cfg!(target_os = "macos") {
        Command::new("route")
            .arg("-n")
            .arg("delete")
            .arg(mode)
            .arg(route)
            .status()
            .unwrap()
    } else {
        unimplemented!()
    };
    if status.success() {
        Ok(())
    } else {
        Err(format!("route: {}", status))
    }
}

pub fn add_route(route_type: RouteType, route: &str, gateway: &str) -> Result<(), String> {
    let mode = match route_type {
        RouteType::Net => "-net",
        RouteType::Host => "-host",
    };
    info!("Adding route: {} {} gateway {}.", mode, route, gateway);
    let status = if cfg!(target_os = "linux") {
        Command::new("ip")
            .arg("r")
            .arg("a")
            .arg(route)
            .arg("via")
            .arg(gateway)
            .arg("metric")
            .arg("1")
            .status()
    } else if cfg!(target_os = "macos") {
        Command::new("route")
            .arg("-n")
            .arg("add")
            .arg(mode)
            .arg(route)
            .arg(gateway)
            .status()
    } else {
        unimplemented!()
    };
    Ok(())
}

pub fn set_default_gateway(gateway: &str) -> Result<(), String> {
    add_route(RouteType::Net, "default", gateway)
}

pub fn delete_default_gateway() -> Result<(), String> {
    delete_route(RouteType::Net, "default")
}

pub fn get_default_gateway() -> Result<String, String> {
    let cmd = if cfg!(target_os = "linux") {
        format!("ip -o r g {} | awk '{{print $3}}'", REMOTE)
    } else {
        unimplemented!()
    };
    let output = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .output()
        .unwrap();
    if output.status.success() {
        Ok(String::from_utf8(output.stdout).unwrap().trim_right().to_string())
    } else {
        Err(String::from_utf8(output.stderr).unwrap())
    }
}

pub fn get_public_ip() -> Result<String, String> {
    let output = Command::new("curl")
        .arg("ipecho.net/plain")
        .output()
        .unwrap();
    if output.status.success() {
        Ok(String::from_utf8(output.stdout).unwrap())
    } else {
        Err(String::from_utf8(output.stderr).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use utils::*;

    #[test]
    fn route_test() {
        assert!(is_root());

        let gw = get_default_gateway().unwrap();
        add_route(RouteType::Host, "1.1.1.1", &gw).unwrap();
        delete_route(RouteType::Host, "1.1.1.1").unwrap();
    }
}
