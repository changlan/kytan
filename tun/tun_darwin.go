package tun

import "os"

func openDevice(name string) (*os.File, error) {
	return os.OpenFile("/dev/"+name, os.O_RDWR, 0)
}

func createInterface(file *os.File, name string) (string, error) {
	return name, nil
}

func setupInterface(name string, localip string) error {
	return nil
}
