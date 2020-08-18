package install

import (
	"fmt"
	"io/ioutil"
	"log"
	"os"
	"os/exec"
	"path"

	"github.com/markbates/pkger"
)

const (
	servicePath = "/etc/systemd/system/gesha.service"
	configPath  = "/etc/gesha/config.yaml"
	exePath     = "/usr/local/bin/gesha"
)

// Cmd - installs a systemd service
func Cmd() {
	if hasSystemd() {
		pkger.Include("/init/gesha.service")
		pkger.Include("/configs/rancilio-silvia.yaml")

		fmt.Println("Installing...")

		installFile("/init/gesha.service", servicePath, true, true)
		installFile("/configs/rancilio-silvia.yaml", configPath, false, true)

		geshaPath, _ := os.Executable()
		installFile(geshaPath, exePath, true, false)

		fmt.Println("Installation complete.")
		fmt.Println("To start on boot, run: sudo systemctl enable gesha")
		fmt.Println("To start now: sudo systemctl start gesha")
		fmt.Println("Configuration file can be edited at /etc/gesha/config.yaml")
	} else {
		fmt.Println("This OS does not use systemd. Please install manually.")
	}
}

func installFile(fromPath string, toPath string, overwrite bool, bundle bool) {
	if _, err := os.Stat(toPath); os.IsNotExist(err) || overwrite {
		var data []byte
		if bundle {
			handle, openErr := pkger.Open(fromPath)

			if openErr != nil {
				log.Fatalf("Failed to load file %v: %v\n", fromPath, openErr)
			}

			data, _ = ioutil.ReadAll(handle)
		} else {
			sysData, err := ioutil.ReadFile(fromPath)
			if err != nil {
				log.Fatalf("Coudn't read %v\n", fromPath)
			}
			data = sysData
		}

		mkdirError := os.MkdirAll(path.Dir(toPath), os.ModePerm)
		if mkdirError != nil {
			log.Fatalf("Failed to create %v\n", path.Dir(toPath))
		}

		writeErr := ioutil.WriteFile(toPath, data, 0644)

		fmt.Printf("Writing %v to %v\n", fromPath, toPath)

		if writeErr != nil {
			log.Fatalf("Failed to write %v! %v\n", toPath, writeErr)
		}
	}
}

func hasSystemd() bool {
	_, err := exec.LookPath("systemd")
	return err == nil
}
