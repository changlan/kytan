package common

import (
	"bytes"
	"encoding/binary"
	"errors"
	"github.com/changlan/mangi/tun"
	"log"
	"net"
	"strconv"
	"os"
	"os/signal"
	"syscall"
	"fmt"
	"github.com/changlan/mangi/crypto"
)

type Server struct {
	tun      *tun.TunDevice
	conn     *net.UDPConn
	sessions *Session
	nat		 *Nat
	key 	 []byte
}

func NewServer(port int, local_ip string, key []byte) (*Server, error) {
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
		key,
	}, nil
}

func (s *Server) handleTun(err_chan chan error) {
	defer s.tun.Close()

	for {
		pkt, err := s.tun.Read()
		if err != nil {
			err_chan <- err
			return
		}

		// TODO: s.nat.ReverseTranslate(pkt)

		dst_ip := binary.BigEndian.Uint32(pkt[16:20])
		addr, err := s.sessions.Lookup(dst_ip)

		if err != nil {
			err_chan <- err
			return
		}

		buffer := new(bytes.Buffer)

		err = binary.Write(buffer, binary.BigEndian, Magic)
		if err != nil {
			err_chan <- err
			return
		}

		err = binary.Write(buffer, binary.BigEndian, Data)
		if err != nil {
			err_chan <- err
			return
		}

		buffer.Write(pkt)

		data, err := crypto.Encrypt(s.key, buffer.Bytes())
		if err != nil {
			err_chan <- err
			return
		}

		_, err = s.conn.WriteToUDP(data, addr)

		if err != nil {
			err_chan <- err
			return
		}
	}
}

func (s *Server) handleUDP(err_chan chan error) {
	defer s.conn.Close()

	for {
		buf := make([]byte, 2000)
		n, addr, err := s.conn.ReadFromUDP(buf)
		if err != nil {
			err_chan <- err
			return
		}
		if n < 5 {
			err = errors.New("Malformed UDP packet. Length less than 5.")
			err_chan <- err
			return
		}

		buf, err = crypto.Decrypt(s.key, buf)
		if err != nil {
			err_chan <- err
			return
		}

		reader := bytes.NewReader(buf)
		var magic uint32
		err = binary.Read(reader, binary.BigEndian, &magic)

		if err != nil {
			err_chan <- err
			return
		}

		if magic != Magic {
			err_chan <- errors.New("Malformed UDP packet. Invalid MAGIC.")
			return
		}

		var message_type uint8
		err = binary.Read(reader, binary.BigEndian, &message_type)

		if err != nil {
			err_chan <- err
			return
		}

		switch message_type {
		case Request:
			ip := s.sessions.NewClient(addr)

			buffer := new(bytes.Buffer)
			err = binary.Write(buffer, binary.BigEndian, Magic)
			if err != nil {
				err_chan <- err
				return
			}

			err = binary.Write(buffer, binary.BigEndian, Accept)
			if err != nil {
				err_chan <- err
				return
			}

			data := make([]byte, 4)
			binary.BigEndian.PutUint32(data, ip)
			buffer.Write(data)

			data, err = crypto.Encrypt(s.key, buffer.Bytes())
			if err != nil {
				err_chan <- err
				return
			}

			_, err = s.conn.WriteToUDP(data, addr)

			if err != nil {
				err_chan <- err
				return
			}

		case Data:
			pkt := buf[5:n]

			// TODO: s.nat.ForwardTranslate(pkt)

			err = s.tun.Write(pkt)

			if err != nil {
				err_chan <- err
				return
			}
		default:
			err_chan <- errors.New("Unknown message type.")
			return
		}
	}
}

func (s *Server) cleanup() {
	s.tun.Close()
	s.conn.Close()
}

func (s *Server) handleSignal(err_chan chan error) {
	sigs := make(chan os.Signal, 1)
	signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)

	sig := <-sigs

	msg := fmt.Sprintf("%s received.", sig.String())
	log.Printf(msg)

	err_chan <- errors.New(msg)
}

func (s *Server) Run() {
	err_chan := make(chan error)

	go s.handleTun(err_chan)
	go s.handleUDP(err_chan)
	go s.handleSignal(err_chan)

	err := <- err_chan
	log.Print(err)

	s.cleanup()
}
