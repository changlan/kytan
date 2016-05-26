package common

import (
	"errors"
	"fmt"
	"github.com/changlan/kytan/crypto"
	"github.com/changlan/kytan/message"
	"github.com/changlan/kytan/tun"
	"github.com/changlan/kytan/util"
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
	device           *tun.TunDevice
	serverConnection *net.UDPConn
	serverAddress    string
	gatewayAddress   string
	secretKey        []byte
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
		addr.IP.String(),
		"",
		key,
	}, nil
}

func (c *Client) handleTun(err_chan chan error) {
	defer c.device.Close()
	for {
		pkt, err := c.device.Read()

		log.Printf("%s -> %s", c.device.String(), c.serverConnection.RemoteAddr().String())

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

		data, err = crypto.Encrypt(c.secretKey, data)
		if err != nil {
			err_chan <- err
			return
		}

		_, err = c.serverConnection.Write(data)
		if err != nil {
			err_chan <- err
			return
		}
	}
}

func (c *Client) handleUDP(err_chan chan error) {
	defer c.serverConnection.Close()
	for {
		buf := make([]byte, 1600)

		n, err := c.serverConnection.Read(buf)
		log.Printf("%s -> %s", c.serverConnection.RemoteAddr().String(), c.device.String())
		if err != nil {
			err_chan <- err
			return
		}

		buf, err = crypto.Decrypt(c.secretKey, buf[:n])
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

		if *msg.Kind != message.Message_DATA {
			err = errors.New("Unexpected message type.")
			err_chan <- err
			return
		}

		err = c.device.Write(msg.Data)
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

	log.Printf("Sending request to %s.", c.serverConnection.RemoteAddr().String())

	data, err := proto.Marshal(msg)
	if err != nil {
		return err
	}

	data, err = crypto.Encrypt(c.secretKey, data)
	if err != nil {
		return err
	}

	success := false
	buf := make([]byte, 1600)

	for !success {
		_, err = c.serverConnection.Write(data)
		if err != nil {
			return err
		}

		c.serverConnection.SetReadDeadline(time.Now().Add(5 * time.Second))
		n, err := c.serverConnection.Read(buf)
		c.serverConnection.SetReadDeadline(time.Time{})

		if err != nil {
			log.Printf("Read error: %v. Reconnecting...", err)
			continue
		}

		buf = buf[:n]
		success = true
	}

	buf, err = crypto.Decrypt(c.secretKey, buf)
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

	c.device, err = tun.NewTun(DeviceNameClient, local_ip.String())
	if err != nil {
		return err
	}

	local_ip[3] = 0x1
	c.gatewayAddress, err = util.DefaultGateway()
	if err != nil {
		return err
	}
	err = util.SetGatewayForHost(c.gatewayAddress, c.serverAddress)
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
	c.device.Close()
	c.serverConnection.Close()

	util.ClearGateway()
	util.SetDefaultGateway(c.gatewayAddress)
	util.ClearGatewayForHost(c.serverAddress)
}

func (c *Client) handleSignal(err_chan chan error) {
	sigs := make(chan os.Signal, 1)
	signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)

	sig := <-sigs

	msg := fmt.Sprintf("%s received.", sig.String())
	log.Printf(msg)

	err_chan <- errors.New(msg)
}
