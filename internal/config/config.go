package config

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"time"

	"gopkg.in/yaml.v3"
)

// Config defines the runtime configuration for the application
type Config struct {
	Port                  string        `json:"port" yaml:"port"`
	BoilerPin             string        `json:"boilerPin" yaml:"boilerPin"`
	SpiPort               string        `json:"spiPort" yaml:"spiPort"`
	TemperatureSampleRate time.Duration `json:"temperatureSampleRate" yaml:"temperatureSampleRate"`
	TemperatureUnit       string        `json:"temperatureUnit" yaml:"temperatureUnit"`
	TemperatureTarget     float64       `json:"temperatureTarget" yaml:"temperatureTarget"`
	PID                   []float64     `json:"pid,flow" yaml:"pid"`
	PidFrequency          time.Duration `json:"pidFrequency" yaml:"pidFrequency"`
	PidAutostart          bool          `json:"pidAutostart" yaml:"pidAutostart"`
	Verbose               bool          `json:"verbose" yaml:"verbose"`
	ThemeColor            struct {
		Hex string `json:"hex" yaml:"hex"`
		Hue string `json:"hue" yaml:"hue"`
	} `json:"themeColor" yaml:"themeColor"`
}

// New creates a config with defaults and based on the environment file
func New(path string) Config {
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

// Update - updates the running config with a newConfig and writes it to disk
func (c *Config) Update(newConfig *Config, path string) error {
	c.Port = newConfig.Port
	c.BoilerPin = newConfig.BoilerPin
	c.SpiPort = newConfig.SpiPort
	c.TemperatureSampleRate = newConfig.TemperatureSampleRate
	c.TemperatureUnit = newConfig.TemperatureUnit
	c.TemperatureTarget = newConfig.TemperatureTarget
	c.PID = newConfig.PID
	c.PidFrequency = newConfig.PidFrequency
	c.PidAutostart = newConfig.PidAutostart
	c.ThemeColor = newConfig.ThemeColor
	c.Verbose = newConfig.Verbose

	data, err := yaml.Marshal(c)

	if err != nil {
		return err
	}

	fmt.Println(string(data))

	ioutil.WriteFile(path, data, 0700)

	return nil
}

type ConfigDto struct {
	Port              string    `json:"port" yaml:"port"`
	BoilerPin         string    `json:"boilerPin" yaml:"boilerPin"`
	SpiPort           string    `json:"spiPort" yaml:"spiPort"`
	TemperatureUnit   string    `json:"temperatureUnit" yaml:"temperatureUnit"`
	TemperatureTarget float64   `json:"temperatureTarget" yaml:"temperatureTarget"`
	PID               []float64 `json:"pid,flow" yaml:"pid"`
	PidAutostart      bool      `json:"pidAutostart" yaml:"pidAutostart"`
	Verbose           bool      `json:"verbose" yaml:"verbose"`
	ThemeColor        struct {
		Hex string `json:"hex" yaml:"hex"`
		Hue string `json:"hue" yaml:"hue"`
	} `json:"themeColor" yaml:"themeColor"`
	TemperatureSampleRate string `json:"temperatureSampleRate,omitempty"`
	PidFrequency          string `json:"pidFrequency,omitempty"`
}

func (c *Config) MarshalJSON() ([]byte, error) {
	var config ConfigDto

	config.Port = c.Port
	config.BoilerPin = c.BoilerPin
	config.SpiPort = c.SpiPort
	config.TemperatureSampleRate = c.TemperatureSampleRate.String()
	config.TemperatureUnit = c.TemperatureUnit
	config.TemperatureTarget = c.TemperatureTarget
	config.PID = c.PID
	config.PidFrequency = c.PidFrequency.String()
	config.PidAutostart = c.PidAutostart
	config.Verbose = c.Verbose
	config.ThemeColor = c.ThemeColor

	return json.Marshal(config)
}

func (c *Config) UnmarshalJSON(b []byte) error {
	var config ConfigDto

	err := json.Unmarshal(b, &config)

	if err != nil {
		return err
	}

	temperatureSampleRate, err := time.ParseDuration(config.TemperatureSampleRate)
	if err != nil {
		return err
	}
	pidFrequency, err := time.ParseDuration(config.PidFrequency)
	if err != nil {
		return err
	}

	c.Port = config.Port
	c.BoilerPin = config.BoilerPin
	c.SpiPort = config.SpiPort
	c.TemperatureSampleRate = temperatureSampleRate
	c.TemperatureUnit = config.TemperatureUnit
	c.TemperatureTarget = config.TemperatureTarget
	c.PID = config.PID
	c.PidFrequency = pidFrequency
	c.PidAutostart = config.PidAutostart
	c.Verbose = config.Verbose
	c.ThemeColor = config.ThemeColor

	return nil
}
