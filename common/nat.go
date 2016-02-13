package common

import (
	"encoding/binary"
	"errors"
	"fmt"
	"net"
)

type Addr struct {
	protocol    uint8
	source_ip   uint32
	source_port uint16
}

type Nat struct {
	addr      uint32
	forward   map[Addr]uint16
	reverse   map[uint16]*Addr
	next_port uint16
}

func NewNat(addr net.IP) *Nat {
	return &Nat{
		binary.BigEndian.Uint32(addr.To4()),
		make(map[Addr]uint16),
		make(map[uint16]*Addr),
		1,
	}
}

func (n *Nat) ForwardLookup(tuple *Addr) uint16 {
	port, ok := n.forward[*tuple]
	if !ok {
		port = n.next_port
		n.next_port++
		n.forward[*tuple] = port
		n.reverse[port] = tuple
	}
	return port
}

func (n *Nat) ForwardTranslate(pkt []byte) ([]byte, error) {
	ihl := pkt[1] & 0xf
	length := ihl << 2
	ip_src := binary.BigEndian.Uint32(pkt[12:16])
	protocol := pkt[9]
	switch protocol {
	case 0x06, 0x11:
		port_src := binary.BigEndian.Uint16(pkt[length : length+2])
		tuple := Addr{protocol, ip_src, port_src}

		ip_ext := n.addr
		port_ext := n.ForwardLookup(&tuple)

		binary.BigEndian.PutUint32(pkt[12:16], ip_ext)
		binary.BigEndian.PutUint16(pkt[length:length+2], port_ext)

		// TODO: Compute checksums
	case 0x01:
		// TODO: ICMP Handling
	default:
		// TODO: Other cases
	}
	return pkt, nil
}

func (n *Nat) ReverseLookup(port uint16) (*Addr, error) {
	tuple, ok := n.reverse[port]
	if !ok {
		msg := fmt.Sprintf("NAT record not found for external port %d", port)
		return nil, errors.New(msg)
	}
	return tuple, nil
}

func (n *Nat) ReverseTranslate(pkt []byte) ([]byte, error) {
	// TODO: Implement this
	return pkt, nil
}
