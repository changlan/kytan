use std::process::Command;

pub fn enable_ipv4_forwarding() -> Result<(), String> {
    let sysctl_arg = if cfg!(target_os = "linux") {
        "net.ipv4.ip_forward=1"
    } else if cfg!(target_os = "macos") {
        "net.inet.ip.forwarding=1"
    } else {
        unimplemented!()
    };
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

pub enum RouteType {
    Net,
    Host,
}

pub fn delete_route(route_type: RouteType, route: &str) -> Result<(), String> {
    let mode = match route_type {
        RouteType::Net => "-net",
        RouteType::Host => "-host",
    };
    let cmd = if cfg!(target_os = "linux") {
        format!("route -n del {} {}", mode, route)
    } else if cfg!(target_os = "macos") {
        format!("route -n delete {} {}", mode, route)
    } else {
        unimplemented!()
    };
    let status = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .status()
        .unwrap();
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
    let cmd = if cfg!(target_os = "linux") {
        format!("route -n add {} {} gw {}", mode, route, gateway)
    } else if cfg!(target_os = "macos") {
        format!("route -n add {} {} {}", mode, route, gateway)
    } else {
        unimplemented!()
    };
    let status = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .status()
        .unwrap();
    if status.success() {
        Ok(())
    } else {
        Err(format!("route: {}", status))
    }
}

pub fn set_default_gateway(gateway: &str) -> Result<(), String> {
    add_route(RouteType::Net, "default", gateway)
}

pub fn delete_default_gateway() -> Result<(), String> {
    delete_route(RouteType::Net, "default")
}

pub fn get_default_gateway() -> Result<String, String> {
    let cmd = if cfg!(target_os = "linux") {
        "ip -4 route list 0/0 | awk '{print $3}'"
    } else if cfg!(target_os = "macos") {
        "route -n get default | grep gateway | awk '{print $2}'"
    } else {
        unimplemented!()
    };
    let output = Command::new("bash")
        .arg("-c")
        .arg(cmd)
        .output()
        .unwrap();
    if output.status.success() {
        Ok(String::from_utf8(output.stdout).unwrap())
    } else {
        Err(String::from_utf8(output.stderr).unwrap())
    }
}
