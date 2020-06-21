package api

// TemperatureTarget - The target temperature of the boiler
type TemperatureTarget struct {
	Target float32 `json:"target,omitempty"`
}
