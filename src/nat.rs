use std::collections::HashMap;
use std::mem;
use packet::{Ipv4Header, UdpHeader, TcpHeader, udptcp_cksum};

pub struct NAT {
    pub forward_table: HashMap<(u8, u32, u16), u16>,
    pub backward_table: HashMap<(u8, u16), (u32, u16)>,
    pub next_port: u16,
}

impl NAT {
    pub fn new() -> NAT {
        NAT {
            forward_table: HashMap::new(),
            backward_table: HashMap::new(),
            next_port: 0,
        }
    }

    pub fn handle_forward_packet(&mut self, data: &[u8], iph: &mut Ipv4Header, ex_address: u32) {
        let ihl = ((iph.version_ihl & 0xf) * 5) as isize;

        let sc_port = match iph.protocol {
                1 => Err(String::from("The version is ICMP")),
                6 | 17 => {
                    let udphd = unsafe {
                        mem::transmute::<*const u8, &mut UdpHeader>(data.as_ptr().offset(ihl))
                    };
                    Ok(u16::from_be(udphd.source_port))
                }
                _ => Err(String::from("Invalid address!")),
            }
            .unwrap();

        let key = (iph.protocol, iph.source_address, sc_port);
        let value = self.forward_table.get(&key).cloned();

        let virtual_port = match value {
                Some(p) => Ok(p.clone()),
                None => {
                    if self.next_port < 0xffff {
                        self.next_port = self.next_port + 1;
                        self.forward_table.insert(key, self.next_port);
                        self.backward_table.insert((iph.protocol, self.next_port),
                                                   (iph.source_address, sc_port));
                        Ok(self.next_port)
                    } else {
                        Err(String::from("No port to distribute"))
                    }
                }
            }
            .unwrap();

        iph.source_address = ex_address;
        match iph.protocol {
            17 => {
                let udphd = unsafe {
                    mem::transmute::<*const u8, &mut UdpHeader>(data.as_ptr().offset(ihl))
                };
                udphd.source_port = virtual_port.to_be();
                udphd.checksum = udptcp_cksum(&iph, &udphd);
            }
            6 => {
                let tcphd = unsafe {
                    mem::transmute::<*const u8, &mut TcpHeader>(data.as_ptr().offset(ihl))
                };
                tcphd.source_port = virtual_port.to_be();
                tcphd.checksum = udptcp_cksum(&iph, &tcphd);
            }
            x @ _ => panic!("Unsupported protocol: {}", x),
        };
    }

    pub fn handle_backward_packet(&mut self, data: &[u8], iph: &mut Ipv4Header) {
        let ihl = ((iph.version_ihl & 0xf) * 5) as isize;
        let virtual_port = match iph.protocol {
                1 => Err(String::from("The version is ICMP")),
                6 | 17 => {
                    let udphd = unsafe {
                        mem::transmute::<*const u8, &mut UdpHeader>(data.as_ptr().offset(ihl))
                    };
                    Ok(u16::from_be(udphd.destination_port))
                }
                _ => Err(String::from("Invalid address!")),
            }
            .unwrap();

        let key = (iph.protocol, virtual_port);
        let value = self.backward_table.get(&key);

        let buf = value.unwrap();
        let (sc_address, sc_port) = buf.clone();

        iph.destination_address = sc_address;
        match iph.protocol {
            17 => {
                let udphd = unsafe {
                    mem::transmute::<*const u8, &mut UdpHeader>(data.as_ptr().offset(ihl))
                };
                udphd.destination_port = sc_port;
                udphd.checksum = udptcp_cksum(&iph, &udphd);
            }
            6 => {
                let tcphd = unsafe {
                    mem::transmute::<*const u8, &mut TcpHeader>(data.as_ptr().offset(ihl))
                };
                tcphd.destination_port = sc_port;
                tcphd.checksum = udptcp_cksum(&iph, &tcphd);
            }
            x @ _ => panic!("Unsupported protocol: {}", x),
        };
    }
}
