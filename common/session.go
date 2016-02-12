package common

import (
	"net"
	"fmt"
	"errors"
	"encoding/binary"
)

type Session struct {
	addr uint32
	table map[uint32]*net.UDPAddr
	next_addr uint8
}

func NewSessions(addr net.IP) *Session {
	return &Session {
		binary.BigEndian.Uint32(addr.To4()),
		make(map[uint32]*net.UDPAddr),
		2,
	}
}

func (n *Session) NewClient(tuple *net.UDPAddr) uint32 {
	result := n.addr & 0xffffff00 | uint32(n.next_addr)
	n.next_addr++

	n.table[result] = tuple
	return result
}

func (n *Session) Lookup(addr uint32) (*net.UDPAddr, error) {
	tuple, ok := n.table[addr]
	if !ok {
		buf := make([]byte, 4)
		binary.BigEndian.PutUint32(buf, addr)
		var ip net.IP
		ip = buf
		msg := fmt.Sprintf("No session found for internal IP address: %s", ip.String())
		return nil, errors.New(msg)
	}
	return tuple, nil
}