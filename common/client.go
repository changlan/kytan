package common

import (
	"errors"
	"fmt"
	"github.com/changlan/mangi/crypto"
	"github.com/changlan/mangi/message"
	"github.com/changlan/mangi/tun"
	"github.com/changlan/mangi/util"
	"github.com/golang/protobuf/proto"
	"log"
	"net"
	"os"
	"os/signal"
	"strconv"
	"syscall"
	"time"
)

type Client struct {
	tun  *tun.TunDevice
	conn *net.UDPConn
	addr *net.UDPAddr
	gw   string
	key  []byte
}

func NewClient(server_name string, port int, key []byte) (*Client, error) {
	addr, err := net.ResolveUDPAddr("udp", server_name+":"+strconv.Itoa(port))
	if err != nil {
		return nil, err
	}

	log.Printf("Connecting to %s over UDP.", addr.String())
	conn, err := net.DialUDP("udp", nil, addr)

	return &Client{
		nil,
		conn,
		addr,
		"",
		key,
	}, nil
}

func (c *Client) handleTun(err_chan chan error) {
	defer c.tun.Close()
	for {
		pkt, err := c.tun.Read()

		log.Printf("%s -> %s", c.tun.String(), c.conn.RemoteAddr().String())

		if err != nil {
			err_chan <- err
			return
		}

		msg := &message.Message{
			Kind: message.Message_DATA.Enum(),
			Data: pkt,
		}
		if err != nil {
			err_chan <- err
			return
		}

		data, err := proto.Marshal(msg)
		if err != nil {
			err_chan <- err
			return
		}

		data, err = crypto.Encrypt(c.key, data)

		if err != nil {
			err_chan <- err
			return
		}

		_, err = c.conn.Write(data)

		if err != nil {
			err_chan <- err
			return
		}
	}
}

func (c *Client) handleUDP(err_chan chan error) {
	defer c.conn.Close()
	for {
		buf := make([]byte, 1600)
		n, err := c.conn.Read(buf)

		log.Printf("%s -> %s", c.conn.RemoteAddr().String(), c.tun.String())

		if err != nil {
			err_chan <- err
			return
		}

		buf = buf[:n]

		buf, err = crypto.Decrypt(c.key, buf)
		if err != nil {
			err_chan <- err
			return
		}

		msg := &message.Message{}
		err = proto.Unmarshal(buf, msg)

		if *msg.Kind != message.Message_DATA {
			err = errors.New("Unexpected message type.")
			err_chan <- err
			return
		}

		err = c.tun.Write(msg.Data)
		if err != nil {
			err_chan <- err
			return
		}
	}
}

func (c *Client) init() error {
	msg := &message.Message{
		Kind: message.Message_REQUEST.Enum(),
	}

	log.Printf("Sending request to %s.", c.conn.RemoteAddr().String())

	data, err := proto.Marshal(msg)
	if err != nil {
		return err
	}

	data, err = crypto.Encrypt(c.key, data)
	if err != nil {
		return err
	}

	handshaked := false
	buf := make([]byte, 1600)

	for !handshaked {
		_, err = c.conn.Write(data)
		if err != nil {
			return err
		}

		c.conn.SetReadDeadline(time.Now().Add(5 * time.Second))
		n, err := c.conn.Read(buf)
		c.conn.SetReadDeadline(time.Time{})

		if err != nil {
			log.Printf("Read error: %v. Reconnecting...", err)
			continue
		}

		buf = buf[:n]
		handshaked = true
	}

	buf, err = crypto.Decrypt(c.key, buf)
	if err != nil {
		return err
	}

	msg = &message.Message{}
	err = proto.Unmarshal(buf, msg)

	if *msg.Kind != message.Message_ACCEPT {
		return errors.New("Unexpected message type.")
	}

	var local_ip net.IP
	local_ip = msg.Data
	log.Printf("Client IP %s assigned.", local_ip.String())

	c.tun, err = tun.NewTun("tun0", local_ip.String())
	if err != nil {
		return err
	}

	local_ip[3] = 0x1
	c.gw, err = util.DefaultGateway()
	if err != nil {
		return err
	}
	err = util.SetGatewayForHost(c.gw, c.addr.IP.String())
	if err != nil {
		return err
	}
	err = util.ClearGateway()
	if err != nil {
		return err
	}
	err = util.SetDefaultGateway(local_ip.String())
	if err != nil {
		return err
	}

	return nil
}

func (c *Client) Run() {
	err := c.init()

	if err != nil {
		log.Fatal(err)
	}

	err_chan := make(chan error)

	go c.handleTun(err_chan)
	go c.handleUDP(err_chan)
	go c.handleSignal(err_chan)

	err = <-err_chan
	log.Print(err)

	c.cleanup()
}

func (c *Client) cleanup() {
	c.tun.Close()
	c.conn.Close()

	util.ClearGateway()
	util.SetDefaultGateway(c.gw)
	util.ClearGatewayForHost(c.addr.IP.String())
}

func (c *Client) handleSignal(err_chan chan error) {
	sigs := make(chan os.Signal, 1)
	signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)

	sig := <-sigs

	msg := fmt.Sprintf("%s received.", sig.String())
	log.Printf(msg)

	err_chan <- errors.New(msg)
}
