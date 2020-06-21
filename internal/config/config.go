package config

import (
	"log"
	"os"
	"strconv"
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
	TemperatureTarget     float64       `json:"temperatureTarget,omitempty"`
	P                     float64       `json:"p,omitempty"`
	I                     float64       `json:"i,omitempty"`
	D                     float64       `json:"d,omitempty"`
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

	P, err := strconv.ParseFloat(util.GetEnv("P", "2.9"), 64)

	if err != nil {
		log.Fatalf("P (%s) could not be parsed as a float.", util.GetEnv("P", "2.9"))
	}

	c.P = P

	I, err := strconv.ParseFloat(util.GetEnv("I", "0.3"), 64)

	if err != nil {
		log.Fatalf("I (%s) could not be parsed as a float.", util.GetEnv("I", "0.3"))
	}

	c.I = I

	D, err := strconv.ParseFloat(util.GetEnv("D", "40.0"), 64)

	if err != nil {
		log.Fatalf("D (%s) could not be parsed as a float.", util.GetEnv("D", "40.0"))
	}

	c.D = D

	tempSampleRate, hasTempSampleRate := os.LookupEnv("TEMPERATURE_SAMPLE_RATE_MS")

	if hasTempSampleRate {
		tempSampleRateMs, err := strconv.ParseUint(tempSampleRate, 10, 64)
		if err != nil {
			log.Fatalf("TEMPERATURE_SAMPLE_RATE_MS (%s) could not be parsed as a number", tempSampleRate)
		}

		c.TemperatureSampleRate = time.Duration(tempSampleRateMs) * time.Millisecond
	} else {
		c.TemperatureSampleRate = 100 * time.Millisecond
	}

	targetTemp, hasTargetTemp := os.LookupEnv("TEMPERATURE_TARGET")

	if hasTargetTemp {
		parsedTargetTemp, err := strconv.ParseFloat(targetTemp, 64)
		if err != nil {
			log.Fatalf("TEMPERATURE_TARGET (%s) could not be parsed", targetTemp)
		}

		c.TemperatureTarget = parsedTargetTemp
	} else {
		c.TemperatureTarget = 90.0
	}

	return c
}
