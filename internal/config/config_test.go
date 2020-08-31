package config

import (
	"testing"
	"time"

	"periph.io/x/periph/conn/physic"

	"gopkg.in/yaml.v3"
)

func TestDurationUnmarshaling(t *testing.T) {
	configYaml := "temperatureSampleRate: 100ms\npidFrequency: 1s"
	var config Config
	err := yaml.Unmarshal([]byte(configYaml), &config)
	if err != nil {
		t.Fatal(err)
	}

	if config.TemperatureSampleRate != 100*time.Millisecond {
		t.Fatalf("TemperatureSampleRate, want: %v. Got: %v", 100*time.Millisecond, config.TemperatureSampleRate)
	}
	if config.PidFrequency != 1*time.Second {
		t.Fatalf("PidFrequency, want: %v. Got: %v", 1*time.Second, config.PidFrequency)
	}
}

func TestTemperatureUnmarshaling(t *testing.T) {
	configYaml := "temperatureTarget: 200C"
	var config Config
	err := yaml.Unmarshal([]byte(configYaml), &config)
	if err != nil {
		t.Fatal(err)
	}

	if config.TemperatureTarget != 200*physic.Celsius+physic.ZeroCelsius {
		t.Fatalf("TemperatureTarget, want: %v. Got: %v", 200*physic.Celsius+physic.ZeroCelsius, config.TemperatureTarget)
	}
}
