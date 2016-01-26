package main

import (
	"flag"
	"log"
	"github.com/changlan/mangi/common"
)

func main() {
	mode := flag.String("mode", "client", "Mode: client or server")
	address := flag.String("addr",
		"192.168.88.1",
		"client mode: server IP address / server mode: gateway virtual IP address",
	)
	port := flag.Int("port", 8964, "UDP port")

	flag.Parse()
	switch *mode {
	case "server":
		s, err := common.NewServer(*port, *address)
		if err != nil {
			log.Fatal(err)
			return
		}
		s.Run()
	case "client":
		c, err := common.NewClient(*address, *port)
		if err != nil {
			log.Fatal(err)
			return
		}
		c.Run()
	default:
		log.Fatalf("Invalid mode")
		return
	}
	return
}
