package config

import (
	"io/ioutil"
	"log"
	"time"

	"gopkg.in/yaml.v3"
)

// Config defines the runtime configuration for the application
type Config struct {
	Port                  string        `json:"port" yaml:"port"`
	BoilerPin             string        `json:"boilerPin,omitempty" yaml:"boilerPin"`
	SpiPort               string        `json:"spiPort,omitempty" yaml:"spiPort"`
	TemperatureSampleRate time.Duration `json:"temperatureSampleRate,omitempty" yaml:"temperatureSampleRateMs"`
	TemperatureUnit       string        `json:"temperatureUnit,omitempty" yaml:"temperatureUnit"`
	TemperatureTarget     float64       `json:"temperatureTarget,omitempty" yaml:"temperatureTarget"`
	PID                   []float64     `json:"pid,omitempty,flow" yaml:"pid"`
	PidFrequency          time.Duration `json:"pidFrequency,omitempty" yaml:"pidFrequencyMs"`
	PidAutostart          bool          `json:"pidAutostart,omitempty" yaml:"pidAutostart"`
}

// New creates a config with defaults and based on the environment file
func New(path string) Config {
	c := Config{}

	confData, confErr := ioutil.ReadFile(path)

	if confErr != nil {
		log.Fatalf("Error loading configuration from path '%v'.\nError Message: %v\n", path, confErr)
	}

	err := yaml.Unmarshal([]byte(confData), &c)

	if err != nil {
		log.Fatalf("error: %v", err)
	}

	return c
}
