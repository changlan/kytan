package tun

import (
	"log"
	"net"
	"os"
	"os/exec"
)

func openDevice(name string) (*os.File, error) {
	return os.OpenFile("/dev/"+name, os.O_RDWR, 0)
}

func createInterface(file *os.File, name string) (string, error) {
	return name, nil
}

func setupInterface(name string, localip string) error {
	local := net.ParseIP(localip).To4()
	remote := make(net.IP, len(local))
	copy(remote, local)
	remote[3] = 0x1

	log.Printf("ifconfig %s %s %s", name, local.String(), remote.String())
	err := exec.Command("ifconfig", name, local.String(), remote.String()).Run()
	if err != nil {
		return err
	}

	log.Printf("ifconfig %s mtu 1400 up", name)
	err = exec.Command("ifconfig", name, "mtu", "1400", "up").Run()
	if err != nil {
		return err
	}

	return nil
}
