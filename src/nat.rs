use utils;
use std::collections::HashMap;
use std::mem;
use packet::{Ipv4Header,UdpHeader,TcpHeader, udptcp_cksum};
use std::net::Ipv4Addr;

pub struct NAT {
    pub forward_table: HashMap<(u8, u32, u16), u16>, 
    pub backward_table: HashMap<(u8, u16), (u32, u16)>,
    pub change_port: u16,
}

impl NAT{
    pub fn new() -> NAT {
	NAT{
	    forward_table: HashMap::new(), 
	    backward_table: HashMap::new(), 
	    change_port: 0,
	}
    }

    pub fn handle_forward_packet(&mut self, data: &[u8], iph: &mut Ipv4Header, ex_address: u32){
	let len = ((iph.version_ihl & 0xf) * 5) as isize;
	let sc_port = match iph.protocol { //get source_port
	    1 => Err(String::from("The version is ICMP")),
	    17 => {
		let udphd = unsafe {
		    mem::transmute::<*const u8, &mut UdpHeader>(data.as_ptr().offset(len))
	        };
		Ok(udphd.source_port)
	    } 
	    6  => {
		let tcphd = unsafe {
		    mem::transmute::<*const u8, &mut TcpHeader>(data.as_ptr().offset(len))
		};
		Ok(tcphd.source_port)
	    }
    	    _ => Err(String::from("Invalid address!")),
	}.unwrap();
		
        let key = (iph.protocol, iph.source_address, sc_port);
        let value = self.forward_table.get(&key).cloned();
        let change_port = match value { //use source_port to get change_port
	    Some(p) => {
		Ok(p.clone())
	    }
	    None => {
 		if self.change_port < 0xffff {
	    	    self.change_port = self.change_port+1;
		    self.forward_table.insert((iph.protocol,iph.source_address,sc_port),(self.change_port));
      		    self.backward_table.insert((iph.protocol, self.change_port),(iph.source_address, sc_port));
		    Ok(self.change_port)
	  	} else {
	       	    Err(String::from("No port to distribute"))
	   	};		
		Err(String::from("no response"))
            }
	}.unwrap();
	iph.source_address = ex_address; //source_ip -> external_ip
	match iph.protocol {           //source_port -> change_port & checksum
    	    1   => Err(String::from("The version is ICMP")),
    	    17  => {
		let udphd = unsafe {
		    mem::transmute::<*const u8, &mut UdpHeader>(data.as_ptr().offset(len))
		};
		let cksum = udptcp_cksum(&iph, &udphd);
		udphd.source_port = change_port;
		Ok(udphd.checksum = cksum)
	    }
    	    6  => {
		let tcphd = unsafe {
		    mem::transmute::<*const u8, &mut TcpHeader>(data.as_ptr().offset(len))
		};
		let cksum = udptcp_cksum(&iph, &tcphd);
		tcphd.source_port = change_port;
		Ok(tcphd.checksum = cksum)				
	    }
    	    _ => Err(String::from("Invalid address!")),
	}.unwrap()
    }

    pub fn handle_backward_packet(&mut self, data:&[u8], iph:&mut Ipv4Header){
	let len = ((iph.version_ihl & 0xf) * 5) as isize;
	let change_port = match iph.protocol { //get change_port
	    1  => Err(String::from("The version is ICMP")),
	    17 => {
		let udphd = unsafe {
		    mem::transmute::<*const u8, &mut UdpHeader>(data.as_ptr().offset(len))
		};
		Ok(udphd.destination_port) //change_port ?= destination_port
	    } 
	    6  => {
		let tcphd = unsafe {
		    mem::transmute::<*const u8, &mut TcpHeader>(data.as_ptr().offset(len))
		};
		Ok(tcphd.destination_port)
	    }
    	    _  => Err(String::from("Invalid address!")),
	}.unwrap();
	let key = (iph.protocol, change_port);
	let value = self.backward_table.get(&key);
	let buf = match value {         //use key to get source_address & source_port
	    Some(sc) => {
		Ok(sc.clone())
	    }
	    None => {
		Err(String::from("Invalid message"))
	    }
	}.unwrap();
	let (sc_address, sc_port) = buf;      
	iph.destination_address = sc_address;   	//destination_address -> sc_address
	match iph.protocol {                        //destination_port -> sc_port & checksum
    	    1   => Err(String::from("The version is ICMP")),
    	    17  => {
		let udphd = unsafe {
		    mem::transmute::<*const u8, &mut UdpHeader>(data.as_ptr().offset(len))
		};
		let cksum = udptcp_cksum(&iph, &udphd);
		udphd.checksum = cksum;
		Ok(udphd.destination_port = sc_port)
	    }
    	    6  =>{
		let tcphd = unsafe {
		    mem::transmute::<*const u8, &mut TcpHeader>(data.as_ptr().offset(len))
		};
		let cksum = udptcp_cksum(&iph, &tcphd);
		tcphd.checksum = cksum;
	        Ok(tcphd.destination_port = sc_port)
	    }
    	    _  => Err(String::from("Invalid address!")),
	}.unwrap()
    }	
}

