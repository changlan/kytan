package tun

import (
	"os"
	"unsafe"
	"syscall"
	"os/exec"
	"net"
)

const (
	cIFF_TUN   = 0x0001
	cIFF_TAP   = 0x0002
	cIFF_NO_PI = 0x1000
)

func openDevice(name string) (*os.File, error) {
	return os.OpenFile("/dev/net/tun", os.O_RDWR, 0)
}

func createInterface(file *os.File, name string) (string, error) {
	var req ifReq
	req.Flags = 0
	copy(req.Name[:15], name)
	req.Flags |= cIFF_TUN
	req.Flags |= cIFF_NO_PI
	_, _, err := syscall.Syscall(syscall.SYS_IOCTL, file.Fd(), uintptr(syscall.TUNSETIFF), uintptr(unsafe.Pointer(&req)))
	if err != 0 {
		return "", err
	}
	return string(req.Name[:]), nil
}

func setupInterface(name string, local_ip string) error {
	err := exec.Command("ip", "link", "set", name, "up").Run()
	if err != nil {
		return err
	}

	ip_net := net.IPNet{net.ParseIP(local_ip), net.CIDRMask(24, 32)}
	err = exec.Command("ip", "link", "add", ip_net.String(), "dev", name).Run()
	if err != nil {
		return err
	}

	return nil
}