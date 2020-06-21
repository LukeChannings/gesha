package config

import (
	"log"
	"time"

	"github.com/joho/godotenv"
	"github.com/lukechannings/gesha/internal/util"
)

// Config defines the runtime configuration for the application
type Config struct {
	BoilerPin             string        `json:"boilerPin,omitempty"`
	SpiPort               string        `json:"spiPort,omitempty"`
	TemperatureSampleRate time.Duration `json:"temperatureSampleRate,omitempty"`
	TemperatureUnit       string        `json:"temperatureUnit,omitempty"`
	P                     float32       `json:"p,omitempty"`
	I                     float32       `json:"i,omitempty"`
	D                     float32       `json:"d,omitempty"`
}

// New creates a config with defaults and based on the environment file
func New() Config {
	godotenv.Load(".env", "/etc/gesha/config.env")

	c := Config{}

	c.BoilerPin = util.GetEnv("BOILER_PIN", "GPIO7")
	c.SpiPort = util.GetEnv("SPI_PORT", "")
	c.TemperatureUnit = util.GetEnv("TEMPERATURE_UNIT", "C")

	if c.TemperatureUnit != "C" && c.TemperatureUnit != "F" {
		log.Println("Invalid temperature unit preference. Must be 'C' or 'F'. Defaulting to C.")
		c.TemperatureUnit = "C"
	}

	c.P = 2.9
	c.I = 0.3
	c.D = 40.0

	c.TemperatureSampleRate = 100 * time.Millisecond

	return c
}
