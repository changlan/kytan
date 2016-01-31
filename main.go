package main

import (
	"flag"
	"github.com/changlan/mangi/common"
	"log"
	"github.com/changlan/mangi/crypto"
)

func main() {
	mode := flag.String("mode", "client", "Mode: client or server")
	address := flag.String("addr",
		"192.168.88.1",
		"Client mode: server IP address / Server mode: gateway virtual IP address",
	)
	port := flag.Int("port", 8964, "UDP port")
	secret := flag.String("secret", "default", "Secret Key")

	flag.Parse()

	key := crypto.GenerateKey(*secret)

	switch *mode {
	case "server":
		log.Printf("Starting as a server. Port %d", *port)
		log.Printf("Local LAN IP address: %s", *address)
		s, err := common.NewServer(*port, *address, key)
		if err != nil {
			log.Fatal(err)
		}
		s.Run()

	case "client":
		log.Printf("Starting as a client. Connect to %s:%d", *address, *port)
		c, err := common.NewClient(*address, *port, key)
		if err != nil {
			log.Fatal(err)
		}
		c.Run()
	default:
		log.Fatalf("Invalid mode")
	}
	return
}
