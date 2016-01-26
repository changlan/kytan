package common
import (
	"net"
	"strconv"
	"encoding/binary"
	"bytes"
	"errors"
	"sync"
	"log"
	"github.com/changlan/mangi/tun"
)

type Server struct {
	tun *tun.TunDevice
	conn *net.UDPConn
	sessions map[uint32]*Session
	prefix []byte
	next_id byte
}

func NewServer (port int, local_ip string) (*Server, error) {
	ip := net.ParseIP(local_ip)
	mask := net.CIDRMask(24, 32)
	prefix := ip.Mask(mask)
	if (len(prefix) != 4) {
		return nil, errors.New("Incorrect IP address length")
	}

	tun, err := tun.NewTun("tun0", "192.168.88.1")

	if (err != nil) {
		return nil, err
	}
	addr, err := net.ResolveUDPAddr("udp", ":" + strconv.Itoa(port))
	if (err != nil) {
		tun.Close()
		return nil, err
	}
	conn, err := net.ListenUDP("udp", addr)
	if (err != nil) {
		tun.Close()
		return nil, err
	}

	return &Server {
		tun,
		conn,
		make(map[uint32]*Session),
		prefix,
		2,
	}, nil
}

func (s *Server) handleTun(wg *sync.WaitGroup) {
	defer s.tun.Close()
	defer wg.Done()

	for {
		pkt, err := s.tun.Read()
		if (err != nil) {
			log.Fatal(err)
			return
		}

		dst_ip := pkt[16:20]
		key := binary.BigEndian.Uint32(dst_ip)
		session := s.sessions[key]
		addr := session.addr

		buffer := new(bytes.Buffer)

		err = binary.Write(buffer, binary.BigEndian, Magic)
		if (err != nil) {
			log.Fatal(err)
			return
		}

		err = binary.Write(buffer, binary.BigEndian, Data)
		if (err != nil) {
			log.Fatal(err)
			return
		}

		buffer.Write(pkt)

		_, err = s.conn.WriteToUDP(buffer.Bytes(), addr)
		if (err != nil) {
			log.Fatal(err)
			return
		}
	}
}

func (s *Server) handleUDP(wg *sync.WaitGroup) {
	defer s.conn.Close()
	defer wg.Done()

	for {
		buf := make([]byte, 2000)
		n, addr, err := s.conn.ReadFromUDP(buf)
		if (err != nil) {
			log.Fatal(err)
			return
		}

		if (n < 5) {
			err = errors.New("Malformed UDP packet. Length less than 5.")
			log.Fatal(err)
			return
		}

		reader := bytes.NewReader(buf)
		var magic uint32
		err = binary.Read(reader, binary.BigEndian, &magic)

		if (err != nil) {
			log.Fatal(err)
			return
		}

		if (magic != Magic) {
			log.Fatal(errors.New("Malformed UDP packet. Invalid MAGIC."))
			return
		}

		var message_type uint8
		err = binary.Read(reader, binary.BigEndian, &message_type)

		if (err != nil) {
			log.Fatal(err)
			return
		}

		switch message_type {
		case Request:
			ip := s.prefix[:]
			ip[3] = s.next_id
			s.next_id ++

			key := binary.BigEndian.Uint32(ip)
			s.sessions[key] = &Session{addr}

			buffer := new(bytes.Buffer)
			err = binary.Write(buffer, binary.BigEndian, Magic)
			if (err != nil) {
				log.Fatal(err)
				return
			}

			err = binary.Write(buffer, binary.BigEndian, Accept)
			if (err != nil) {
				log.Fatal(err)
				return
			}

			buffer.Write(ip)
			_, err = s.conn.WriteToUDP(buffer.Bytes(), addr)

			if (err != nil) {
				log.Fatal(err)
				return
			}

		case Data:
			err = s.tun.Write(buf[5:n])
			if (err != nil) {
				log.Fatal(err)
				return
			}
		default:
			log.Fatal(errors.New("Unknown message type."))
			return
		}
	}
}

func (s *Server) Run() {
	var wg sync.WaitGroup
	wg.Add(2)

	go s.handleTun(&wg)
	go s.handleUDP(&wg)

	wg.Wait()
}