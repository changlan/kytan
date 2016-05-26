package main

import (
	"flag"
	"fmt"
	"github.com/changlan/kytan/common"
	"github.com/changlan/kytan/crypto"
	"log"
	"math/rand"
	"time"
	"os"
)

type Config struct {
	// Common
	Mode   string
	Secret string

	// Server
	BindPort int

	// Client
	RemoteIP   string
	RemotePort int
}

func parseFlags() *Config {
	mode := flag.String("mode", "client", "Mode: client or server")
	secret := flag.String("secret", "default", "Secret Key")

	bindPort := flag.Int("bind", 8964, "UDP port for incoming connections")

	remoteIP := flag.String("addr", "8.8.8.8", "Server IP")
	remotePort := flag.Int("port", 8964, "Server UDP port")

	flag.Parse()

	return &Config{
		*mode,
		*secret,
		*bindPort,
		*remoteIP,
		*remotePort,
	}
}

func isRoot() bool {
	return os.Geteuid() == 0
}

func main() {
	if !isRoot() {
		log.Fatalf("mangi must be run as root.")
	}

	config := parseFlags()
	key := crypto.GenerateKey(config.Secret)

	switch config.Mode {
	case "server":
		rand.Seed(time.Now().UTC().UnixNano())
		localIP := fmt.Sprintf("10.%d.%d.1", rand.Intn(256), rand.Intn(256))

		log.Printf("Starting as a server. Port %d", config.BindPort)
		log.Printf("Local LAN IP address: %s", localIP)

		s, err := common.NewServer(config.BindPort, localIP, key, "tun0")
		if err != nil {
			log.Fatal(err)
		}

		s.Run()

	case "client":
		log.Printf("Starting as a client. Connect to %s:%d", config.RemoteIP, config.RemotePort)

		c, err := common.NewClient(config.RemoteIP, config.RemotePort, key, "tun0")
		if err != nil {
			log.Fatal(err)
		}

		c.Run()
	default:
		log.Fatalf("Invalid mode")
	}
	return
}
