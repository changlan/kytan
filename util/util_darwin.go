package util

import (
	"fmt"
	"log"
	"os/exec"
)

func cmd(cmdline string) (string, error) {
	log.Printf(cmdline)
	out, err := exec.Command("bash", "-c", cmdline).CombinedOutput()
	return string(out), err
}

func DefaultGateway() (string, error) {
	return cmd("route -n get default | grep 'gateway' | awk '{print $2}'")
}

func ClearGateway() error {
	_, err := cmd("route -n delete -net default")
	return err
}

func SetDefaultGateway(gw string) error {
	command := fmt.Sprintf("route -n add -net default %s", gw)
	_, err := cmd(command)
	return err
}

func SetGatewayForHost(gw string, host string) error {
	command := fmt.Sprintf("route -n add -host %s %s", host, gw)
	_, err := cmd(command)
	return err
}

func ClearGatewayForHost(host string) error {
	command := fmt.Sprintf("route -n delete -host %s", host)
	_, err := cmd(command)
	return err
}
