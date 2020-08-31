package temp

import (
	"encoding/json"
)

type wireTemp struct {
	D     int64   `json:"d"`
	TempC float64 `json:"tempC"`
	TempF float64 `json:"tempF"`
}

// MarshalJSON - implements encoding/json Marshaller interface
func (tc CurrentTemp) MarshalJSON() ([]byte, error) {
	wire := wireTemp{
		D:     tc.Time.UnixNano() / 1000000,
		TempC: tc.Temp.Celsius(),
		TempF: tc.Temp.Fahrenheit(),
	}
	return json.Marshal(wire)
}
