package util
import (
	"log"
	"os/exec"
	"fmt"
)

func cmd(cmdline string) (string, error) {
	log.Printf(cmdline)
	out, err := exec.Command("bash", "-c", cmdline).CombinedOutput()
	return string(out), err
}

func DefaultGateway() (string, error) {
	return cmd("route -n | awk '{if($4==\"UG\")print $2}'")
}

func ClearGateway() error {
	_, err := cmd("route -n del -net default")
	return err
}

func SetDefaultGateway(gw string) error {
	command := fmt.Sprintf("route -n add -net default gw %s", gw)
	_, err := cmd(command)
	return err
}

func SetGatewayForHost(gw string, host string) error {
	command := fmt.Sprintf("route -n add -host %s gw %s", host, gw)
	_, err := cmd(command)
	return err
}

func ClearGatewayForHost(host string) error {
	command := fmt.Sprintf("route -n del -host %s", host)
	_, err := cmd(command)
	return err
}