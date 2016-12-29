use std::process;

pub fn enable_ipv4_forwarding() -> Result<(), &'static str> {
    let sysctl_arg = if cfg!(target_os = "linux") {
        "net.ipv4.ip_forward=1"
    } else if cfg!(target_os = "macos") {
        "net.inet.ip.forwarding=1"
    } else {
        unimplemented!()
    };
    let status = process::Command::new("sysctl")
        .arg("-w")
        .arg(sysctl_arg)
        .status()
        .expect("sysctl command failed to start");
    if status.success() {
        Ok(())
    } else {
        Err("sysctl command failed to start")
    }
}
