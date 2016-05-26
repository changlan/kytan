package common

import (
	"encoding/binary"
	"errors"
	"fmt"
	"github.com/changlan/kytan/crypto"
	"github.com/changlan/kytan/message"
	"github.com/changlan/kytan/tun"
	"github.com/golang/protobuf/proto"
	"log"
	"net"
	"os"
	"os/signal"
	"strconv"
	"syscall"
)

type Server struct {
	device             *tun.TunDevice
	listenerConnection *net.UDPConn
	sessionTable       *SessionTable
	secretKey          []byte
}

func NewServer(port int, local_ip string, key []byte) (*Server, error) {
	ip := net.ParseIP(local_ip)

	log.Printf("Creating TUN device")
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
		NewSessionTable(ip),
		key,
	}, nil
}

func (s *Server) handleTun(err_chan chan error) {
	defer s.device.Close()

	for {
		pkt, err := s.device.Read()
		if err != nil {
			err_chan <- err
			return
		}

		dst_ip := binary.BigEndian.Uint32(pkt[16:20])
		addr, err := s.sessionTable.Lookup(dst_ip)

		if err != nil {
			err_chan <- err
			return
		}

		msg := &message.Message{
			Kind: message.Message_DATA.Enum(),
			Data: pkt,
		}

		data, err := proto.Marshal(msg)

		if err != nil {
			err_chan <- err
			return
		}

		data, err = crypto.Encrypt(s.secretKey, data)
		if err != nil {
			err_chan <- err
			return
		}

		_, err = s.listenerConnection.WriteToUDP(data, addr)

		if err != nil {
			err_chan <- err
			return
		}
	}
}

func (s *Server) handleUDP(err_chan chan error) {
	defer s.listenerConnection.Close()

	for {
		buf := make([]byte, 2000)
		n, addr, err := s.listenerConnection.ReadFromUDP(buf)
		if err != nil {
			err_chan <- err
			return
		}
		buf = buf[:n]

		buf, err = crypto.Decrypt(s.secretKey, buf)
		if err != nil {
			err_chan <- err
			return
		}

		msg := &message.Message{}
		err = proto.Unmarshal(buf, msg)
		if err != nil {
			err_chan <- err
			return
		}

		switch *msg.Kind {
		case message.Message_REQUEST:
			ip := s.sessionTable.NewClient(addr)

			data := make([]byte, 4)
			binary.BigEndian.PutUint32(data, ip)

			msg = &message.Message{
				Kind: message.Message_ACCEPT.Enum(),
				Data: data,
			}

			data, err = proto.Marshal(msg)
			if err != nil {
				err_chan <- err
				return
			}

			data, err = crypto.Encrypt(s.secretKey, data)
			if err != nil {
				err_chan <- err
				return
			}

			_, err = s.listenerConnection.WriteToUDP(data, addr)
			if err != nil {
				err_chan <- err
				return
			}

		case message.Message_DATA:
			pkt := msg.Data

			err = s.device.Write(pkt)

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
	s.device.Close()
	s.listenerConnection.Close()
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

	err := <-err_chan
	log.Print(err)

	s.cleanup()
}
