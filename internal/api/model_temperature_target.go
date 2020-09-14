package api

// TemperatureTarget - The target temperature of the boiler
type TemperatureTarget struct {
	Target float64 `json:"target"`
	Unit   string  `json:"unit"`
}
