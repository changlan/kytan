package common

import (
	"bytes"
	"encoding/binary"
	"errors"
	"github.com/changlan/mangi/tun"
	"log"
	"net"
	"strconv"
	"sync"
	"os"
	"os/signal"
	"syscall"
)

type Server struct {
	tun      *tun.TunDevice
	conn     *net.UDPConn
	sessions *Session
	nat		 *Nat
}

func NewServer(port int, local_ip string) (*Server, error) {
	ip := net.ParseIP(local_ip)

	log.Printf("Creating TUN device tun0.")
	tun, err := tun.NewTun("tun0", local_ip)
	if err != nil {
		return nil, err
	}

	addr, err := net.ResolveUDPAddr("udp", ":"+strconv.Itoa(port))
	if err != nil {
		tun.Close()
		return nil, err
	}

	log.Printf("Listening UDP connections on %s.", addr.String())
	conn, err := net.ListenUDP("udp", addr)
	if err != nil {
		tun.Close()
		return nil, err
	}

	return &Server{
		tun,
		conn,
		NewSessions(ip),
		NewNat(ip),
	}, nil
}

func (s *Server) handleTun(wg *sync.WaitGroup) {
	defer s.tun.Close()
	defer wg.Done()

	for {
		pkt, err := s.tun.Read()
		if err != nil {
			log.Fatal(err)
			return
		}

		// TODO: s.nat.ReverseTranslate(pkt)

		dst_ip := binary.BigEndian.Uint32(pkt[16:20])
		addr, err := s.sessions.Lookup(dst_ip)

		if err != nil {
			log.Fatal(err)
			return
		}

		buffer := new(bytes.Buffer)

		err = binary.Write(buffer, binary.BigEndian, Magic)
		if err != nil {
			log.Fatal(err)
			return
		}

		err = binary.Write(buffer, binary.BigEndian, Data)
		if err != nil {
			log.Fatal(err)
			return
		}

		buffer.Write(pkt)

		_, err = s.conn.WriteToUDP(buffer.Bytes(), addr)
		if err != nil {
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
		if err != nil {
			log.Fatal(err)
			return
		}

		if n < 5 {
			err = errors.New("Malformed UDP packet. Length less than 5.")
			log.Fatal(err)
			return
		}

		reader := bytes.NewReader(buf)
		var magic uint32
		err = binary.Read(reader, binary.BigEndian, &magic)

		if err != nil {
			log.Fatal(err)
			return
		}

		if magic != Magic {
			log.Fatal(errors.New("Malformed UDP packet. Invalid MAGIC."))
			return
		}

		var message_type uint8
		err = binary.Read(reader, binary.BigEndian, &message_type)

		if err != nil {
			log.Fatal(err)
			return
		}

		switch message_type {
		case Request:
			ip := s.sessions.NewClient(addr)

			buffer := new(bytes.Buffer)
			err = binary.Write(buffer, binary.BigEndian, Magic)
			if err != nil {
				log.Fatal(err)
				return
			}

			err = binary.Write(buffer, binary.BigEndian, Accept)
			if err != nil {
				log.Fatal(err)
				return
			}

			data := make([]byte, 4)
			binary.BigEndian.PutUint32(data, ip)
			buffer.Write(data)

			_, err = s.conn.WriteToUDP(buffer.Bytes(), addr)

			if err != nil {
				log.Fatal(err)
				return
			}

		case Data:
			pkt := buf[5:n]

			// TODO: s.nat.ForwardTranslate(pkt)

			err = s.tun.Write(pkt)
			if err != nil {
				log.Fatal(err)
				return
			}
		default:
			log.Fatal(errors.New("Unknown message type."))
			return
		}
	}
}

func (s *Server) cleanup() {
	s.tun.Close()
	s.conn.Close()
}

func (s *Server) handleSignal(wg *sync.WaitGroup) {
	sigs := make(chan os.Signal, 1)
	signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)

	sig := <-sigs

	log.Printf("%s received.", sig.String())

	wg.Done()
}

func (s *Server) Run() error {
	var wg sync.WaitGroup
	wg.Add(1)

	go s.handleTun(&wg)
	go s.handleUDP(&wg)
	go s.handleSignal(&wg)

	wg.Wait()
	s.cleanup()

	return nil
}
