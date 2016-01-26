package common
import "net"

type Session struct {
	addr *net.UDPAddr
}