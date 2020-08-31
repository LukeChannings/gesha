package config

import (
	"io/ioutil"
	"log"
	"time"

	"gopkg.in/yaml.v3"
	"periph.io/x/periph/conn/physic"
)

// Config - defines the runtime configuration for the application
type Config struct {
	Port                  string
	BoilerPin             string
	SpiPort               string
	TemperatureSampleRate time.Duration
	TemperatureUnit       string
	TemperatureTarget     physic.Temperature
	// Group Head to Boiler temperature ratio
	TemperatureGHBR float64
	PID             []float64
	PidFrequency    time.Duration
	PidAutostart    bool
	Verbose         bool
	ThemeColorHue   string
}

// Load creates a config with defaults and based on the environment file
func Load(path string) Config {
	c := Config{}

	confData, confErr := ioutil.ReadFile(path)

	if confErr != nil {
		log.Fatalf("Error loading configuration from path '%v'.\nError Message: %v\n", path, confErr)
	}

	err := yaml.Unmarshal(confData, &c)

	if err != nil {
		log.Fatalf("error: %v", err)
	}

	return c
}

// Update - updates the running configuration and writes the update to disk
func (c *Config) Update(nc *Config, path string) error {
	*c = *nc
	confData, err := yaml.Marshal(c)
	if err != nil {
		return err
	}
	err = ioutil.WriteFile(path, confData, 0x740)
	if err != nil {
		return err
	}

	return nil
}
