package tun

import (
	"io"
	"log"
	"os"
)

type TunDevice struct {
	file *os.File
	name string
}

func NewTun(name string, localip string) (*TunDevice, error) {
	log.Printf("Opening device %s", name)
	file, err := openDevice(name)
	if err != nil {
		return nil, err
	}

	log.Printf("Creating TUN interface %s", name)
	interface_name, err := createInterface(file, name)
	if err != nil {
		file.Close()
		return nil, err
	}

	log.Printf("Interface %s created", interface_name)
	log.Printf("Setting up interface %s with %s", interface_name, localip)

	err = setupInterface(interface_name, localip)
	if err != nil {
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

func (tun *TunDevice) String() string {
	return tun.name
}
