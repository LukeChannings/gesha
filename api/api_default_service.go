package api

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"strconv"
	"time"

	"github.com/lukechannings/gesha/internal/config"
	"github.com/lukechannings/gesha/internal/pid"
	"github.com/lukechannings/gesha/internal/temp"
)

// DefaultAPIService is a service that implents the logic for the DefaultAPIServicer
// This service should implement the business logic for every endpoint for the DefaultApi API.
// Include any external packages or services that will be required by this service.
type DefaultAPIService struct {
	c  *config.Config
	t  *temp.Handle
	p  *pid.Handle
	ts *chan temp.Temp
}

// NewDefaultAPIService creates a default api service
func NewDefaultAPIService(c *config.Config, t *temp.Handle, p *pid.Handle, ts *chan temp.Temp) DefaultAPIServicer {
	return &DefaultAPIService{c: c, t: t, p: p, ts: ts}
}

// GetConfig - Returns the running config
func (s *DefaultAPIService) GetConfig() (interface{}, error) {
	return s.c, nil
}

// GetPidEnabled - Your GET endpoint
func (s *DefaultAPIService) GetPidEnabled() (interface{}, error) {
	return PIDEnabled{Enabled: s.p.Enabled, Heating: s.p.Heating}, nil
}

// GetPidOutput - Your GET endpoint
func (s *DefaultAPIService) GetPidOutput() (interface{}, error) {
	output := <-*s.p.Output
	return output, nil
}

// GetStreamPidOutput - Your GET endpoint
func (s *DefaultAPIService) GetStreamPidOutput(w http.ResponseWriter, r *http.Request) {
	http.Error(w, "Not Implemented", http.StatusInternalServerError)
}

// GetStreamTempCurrent - Your GET endpoint
func (s *DefaultAPIService) GetStreamTempCurrent(w http.ResponseWriter, r *http.Request) {
	query := r.URL.Query()
	var sampleRate time.Duration

	if sampleRateMsQuery, ok := query["sampleRateMs"]; ok {
		parsedSampleRate, err := strconv.ParseInt(sampleRateMsQuery[0], 10, 32)
		if err != nil {
			fmt.Println("Could not parse sampleRateMs from query string")
		}
		sampleRate = time.Duration(parsedSampleRate) * time.Millisecond
	}

	if sampleRate == 0 {
		sampleRate = 100 * time.Millisecond
	}

	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "Streaming unsupported!", http.StatusInternalServerError)
		return
	}
	h := w.Header()
	h.Set("connection", "keep-alive")
	h.Set("cache-control", "no-cache")
	h.Set("content-type", "text/event-stream")
	h.Set("access-control-allow-origin", "*")

	ticker := time.NewTicker(sampleRate)
	defer ticker.Stop()

	for {
		t := <-*s.ts
		tJSON, err := json.Marshal(t)
		if err != nil {
			http.Error(w, "Could not marshal temperature data to JSON", http.StatusInternalServerError)
		}
		fmt.Fprintf(w, "data: %s\n\n", string(tJSON))
		flusher.Flush()
		<-ticker.C
	}
}

// GetTemp - Your GET endpoint
func (s *DefaultAPIService) GetTemp(unit string) (interface{}, error) {
	return s.t.Get()
}

// GetTempTarget - Your GET endpoint
func (s *DefaultAPIService) GetTempTarget() (interface{}, error) {
	// TODO - update GetTempTarget with the required logic for this service method.
	// Add api_default_service.go to the .openapi-generator-ignore to avoid overwriting this service implementation when updating open api generation.
	return nil, errors.New("service method 'GetTempTarget' not implemented")
}

// PostConfig -
func (s *DefaultAPIService) PostConfig(config config.Config) (interface{}, error) {
	// TODO - update PostConfig with the required logic for this service method.
	// Add api_default_service.go to the .openapi-generator-ignore to avoid overwriting this service implementation when updating open api generation.
	return nil, errors.New("service method 'PostConfig' not implemented")
}

// PostPidEnabled -
func (s *DefaultAPIService) PostPidEnabled(pidEnabled PIDEnabled) (interface{}, error) {
	fmt.Println("Enabling the PID!")
	s.p.Start(s.c)
	return nil, nil
}

// PostTempTarget -
func (s *DefaultAPIService) PostTempTarget(temperatureTarget TemperatureTarget) (interface{}, error) {
	// TODO - update PostTempTarget with the required logic for this service method.
	// Add api_default_service.go to the .openapi-generator-ignore to avoid overwriting this service implementation when updating open api generation.
	return nil, errors.New("service method 'PostTempTarget' not implemented")
}
