package tun

import (
	"os"
	"io"
)

type TunDevice struct {
	file *os.File
	name string
}

func NewTun(name string, localip string) (*TunDevice, error) {
	file, err := openDevice(name)
	if (err != nil) {
		return nil, err
	}
	interface_name, err := createInterface(file, name)
	if (err != nil) {
		file.Close()
		return nil, err
	}
	err = setupInterface(interface_name, localip)
	if (err != nil) {
		file.Close()
		return nil, err
	}
	return &TunDevice{file, interface_name}, nil
}

func (tun *TunDevice) Read() ([]byte, error) {
	buf := make([]byte, 2000)
	n, err := tun.file.Read(buf)
	if err != nil {
		return nil, err
	}
	pkt := buf[0:n]
	return pkt, err
}

func (tun *TunDevice) Write(packet []byte) error {
	n, err := tun.file.Write(packet)
	if err != nil {
		return err
	}
	if n != len(packet) {
		return io.ErrShortWrite
	}
	return nil
}

func (tun *TunDevice) Close() error {
	return tun.file.Close()
}