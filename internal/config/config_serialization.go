package config

import (
	"fmt"
	"strings"
	"time"

	"gopkg.in/yaml.v3"
	"periph.io/x/periph/conn/physic"
)

// configWire - a JSON / YAML friendly representation of the Config struct
type configWire struct {
	Port                  string    `json:"port" yaml:"port"`
	BoilerPin             string    `json:"boilerPin" yaml:"boilerPin"`
	SpiPort               string    `json:"spiPort" yaml:"spiPort"`
	TemperatureSampleRate string    `json:"temperatureSampleRate" yaml:"temperatureSampleRate"`
	TemperatureUnit       string    `json:"temperatureUnit" yaml:"temperatureUnit"`
	TemperatureTarget     string    `json:"temperatureTarget" yaml:"temperatureTarget"`
	TemperatureGHBR       float64   `json:"tempGHBR" yaml:"tempGHBR"`
	PID                   []float64 `json:"pid,flow" yaml:"pid"`
	PidFrequency          string    `json:"pidFrequency" yaml:"pidFrequency"`
	PidAutostart          bool      `json:"pidAutostart" yaml:"pidAutostart"`
	Verbose               bool      `json:"verbose" yaml:"verbose"`
	ThemeColorHue         string    `json:"themeColorHue" yaml:"themeColorHue"`
}

// MarshalYAML - Implements YAML Marshaller interface
func (c Config) MarshalYAML() (interface{}, error) {
	return c.toWire(), nil
}

// UnmarshalYAML - Implements YAML Unmarshaller interface
func (c *Config) UnmarshalYAML(value *yaml.Node) error {
	var wire configWire
	err := value.Decode(&wire)
	if err != nil {
		return err
	}
	config, err := wire.toConfig()
	if err != nil {
		return err
	}
	*c = config
	return nil
}

func (c Config) toWire() configWire {
	return configWire{
		Port:                  c.Port,
		BoilerPin:             c.BoilerPin,
		SpiPort:               c.SpiPort,
		TemperatureSampleRate: c.TemperatureSampleRate.String(),
		TemperatureUnit:       c.TemperatureUnit,
		TemperatureTarget:     c.TemperatureTarget.String(),
		PID:                   c.PID,
		PidFrequency:          c.PidFrequency.String(),
		PidAutostart:          c.PidAutostart,
		Verbose:               c.Verbose,
		ThemeColorHue:         c.ThemeColorHue,
	}
}

func (cw configWire) toConfig() (Config, error) {
	temperatureSampleRate, err := time.ParseDuration(cw.TemperatureSampleRate)
	if err != nil {
		temperatureSampleRate = 100 * time.Millisecond
	}

	var temperatureTarget physic.Temperature
	if !strings.HasSuffix(cw.TemperatureTarget, "C") || !strings.HasSuffix(cw.TemperatureTarget, "F") {
		cw.TemperatureTarget += cw.TemperatureUnit
	}
	err = temperatureTarget.Set(cw.TemperatureTarget)
	fmt.Printf("Marshalling ... %s - %v\n", cw.TemperatureTarget, temperatureTarget)
	if err != nil {
		temperatureTarget = 96 * physic.Celsius
	}

	pidFrequency, err := time.ParseDuration(cw.PidFrequency)
	if err != nil {
		pidFrequency = 1 * time.Second
	}

	if cw.TemperatureGHBR == 0 {
		cw.TemperatureGHBR = 1
	}

	return Config{
		Port:                  cw.Port,
		BoilerPin:             cw.BoilerPin,
		SpiPort:               cw.SpiPort,
		TemperatureSampleRate: temperatureSampleRate,
		TemperatureUnit:       cw.TemperatureUnit,
		TemperatureTarget:     temperatureTarget,
		TemperatureGHBR:       cw.TemperatureGHBR,
		PID:                   cw.PID,
		PidFrequency:          pidFrequency,
		PidAutostart:          cw.PidAutostart,
		Verbose:               cw.Verbose,
		ThemeColorHue:         cw.ThemeColorHue,
	}, nil
}
