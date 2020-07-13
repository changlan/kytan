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

use std::mem;
use std::num::Wrapping;

#[repr(packed)]
pub struct Ipv4Header {
    pub version_ihl: u8,            // IP version (= 4) + Internet header length
    pub type_of_service: u8,        // Type of service
    pub total_length: u16,          // Total length in octets
    pub identification: u16,        // Identification
    pub flags_fragment_offset: u16, // 3-bits Flags + Fragment Offset
    pub time_to_live: u8,           // Time To Live
    pub protocol: u8,               // Protocol
    pub header_checksum: u16,       // Checksum
    pub source_address: u32,        // Source Address
    pub destination_address: u32,   // Destination Address
}

#[repr(packed)]
pub struct UdpHeader {
    pub source_port: u16,
    pub destination_port: u16,
    pub total_length: u16,
    pub checksum: u16,
}

#[repr(packed)]
pub struct TcpHeader {
    pub source_port: u16,
    pub destination_port: u16,
    pub seq_num: u32,
    pub ack_sum: u32,
    pub data_offset: u8,
    pub flags: u8,
    pub receive_window: u16,
    pub checksum: u16,
    pub urg_ptr: u16,
}

#[repr(packed)]
pub struct IcmpHeader {
    pub icmp_type: u8,
    pub icmp_code: u8,
    pub icmp_checksum: u16,
    pub icmp_ident: u16,
    pub icmp_seq_num: u16,
}

fn raw_cksum<T>(buf: *const T, len: usize) -> u16 {
    let mut sum = Wrapping(0);
    let mut remaining_len = len;
    let mut ptr = buf as *const u16;
    while remaining_len >= 2 {
        unsafe {
            sum += Wrapping(*ptr);
            ptr = ptr.offset(1);
        }
        remaining_len -= 2;
    }
    if remaining_len == 1 {
        unsafe {
            sum += Wrapping(*(ptr as *const u8) as u16);
        }
    }
    sum.0
}

pub fn ipv4_cksum(buf: &Ipv4Header) -> u16 {
    let cksum = raw_cksum(buf as *const Ipv4Header, mem::size_of::<Ipv4Header>());
    if cksum == 0xffff {
        cksum
    } else {
        !cksum
    }
}

#[repr(packed)]
struct Ipv4PseudoHeader {
    pub source_address: u32,      // Source Address
    pub destination_address: u32, // Destination Address
    pub zero: u8,
    pub protocol: u8,
    pub length: u16,
}

pub fn ipv4_phdr_cksum(ip: &Ipv4Header) -> u16 {
    let psd_hdr = Ipv4PseudoHeader {
        source_address: ip.source_address,
        destination_address: ip.destination_address,
        zero: 0,
        protocol: ip.protocol,
        length: (u16::from_be(ip.total_length) - (mem::size_of::<Ipv4Header>() as u16)).to_be(),
    };
    raw_cksum(&psd_hdr, mem::size_of::<Ipv4PseudoHeader>())
}

pub fn udptcp_cksum<T>(ip: &Ipv4Header, l4: &T) -> u16 {
    let l4_len = (u16::from_be(ip.total_length) as usize) - mem::size_of::<Ipv4Header>();
    let mut cksum = raw_cksum(l4 as *const T, l4_len) as u32;
    cksum += ipv4_phdr_cksum(ip) as u32;
    cksum = ((cksum & 0xffff0000) >> 16) + (cksum & 0xffff);
    cksum = (!cksum) & 0xffff;
    if cksum == 0 {
        cksum = 0xffff;
    }
    cksum as u16
}

#[cfg(test)]
mod tests {
    use crate::packet::*;

    #[test]
    fn raw_cksum_test() {
        assert_eq!(raw_cksum(&[] as *const u8, 0), 0);
        assert_eq!(raw_cksum(&[1u8] as *const u8, 1), 1);
        assert_eq!(raw_cksum(&[1u8, 2u8] as *const u8, 2), 2 * 256 + 1);
        assert_eq!(raw_cksum(&[1u8, 2u8, 3u8] as *const u8, 3), 2 * 256 + 1 + 3);
    }

    #[test]
    fn ipv4_cksum_test() {
        let ip = Ipv4Header {
            version_ihl: 0,
            type_of_service: 0,
            total_length: 0,
            identification: 0,
            flags_fragment_offset: 0,
            time_to_live: 0,
            protocol: 0,
            header_checksum: 0,
            source_address: 0,
            destination_address: 0,
        };
        assert_eq!(ipv4_cksum(&ip), !0);
    }

    #[test]
    fn udptcp_cksum_test() {
        let ip = Ipv4Header {
            version_ihl: 0,
            type_of_service: 0,
            total_length: ((mem::size_of::<Ipv4Header>() + mem::size_of::<UdpHeader>()) as u16)
                .to_be(),
            identification: 0,
            flags_fragment_offset: 0,
            time_to_live: 0,
            protocol: 0,
            header_checksum: 0,
            source_address: 0,
            destination_address: 0,
        };
        let udp = UdpHeader {
            source_port: 0,
            destination_port: 0,
            total_length: (mem::size_of::<UdpHeader>() as u16).to_be(),
            checksum: 0,
        };
        assert_eq!(udptcp_cksum(&ip, &udp), 0xefff);
    }
}
