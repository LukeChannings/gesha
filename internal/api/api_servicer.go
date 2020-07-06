package api

import (
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"net/http"
	"strconv"
	"time"

	"github.com/lukechannings/gesha/internal/config"
	"github.com/lukechannings/gesha/internal/pid"
	"github.com/lukechannings/gesha/internal/temp"
)

// Servicer defines the api actions for the DefaultApi service
// This interface intended to stay up to date with the openapi yaml used to generate it,
// while the service implementation can ignored with the .openapi-generator-ignore file
// and updated with the logic required for the API.
type Servicer interface {
	GetConfig() (interface{}, error)
	GetPidRunning() (interface{}, error)
	GetPidOutput() (interface{}, error)
	GetStreamPidOutput(w http.ResponseWriter, r *http.Request)
	GetStreamTempCurrent(w http.ResponseWriter, r *http.Request)
	GetTemp(string) (interface{}, error)
	GetTempTarget() (interface{}, error)
	PostConfig(config.Config) (interface{}, error)
	PostPidRunning(PIDEnabled) (interface{}, error)
	PostTempTarget(TemperatureTarget) (interface{}, error)
}

// Service is a service that implents the logic for the Servicer
// This service should implement the business logic for every endpoint for the API.
// Include any external packages or services that will be required by this service.
type Service struct {
	c  *config.Config
	t  *temp.Handle
	p  *pid.Handle
	ts *chan temp.Temp
}

// NewAPIService creates a default api service
func NewAPIService(c *config.Config, t *temp.Handle, p *pid.Handle, ts *chan temp.Temp) Servicer {
	return &Service{c: c, t: t, p: p, ts: ts}
}

// GetConfig - Returns the running config
func (s *Service) GetConfig() (interface{}, error) {
	return s.c, nil
}

// GetPidRunning - Your GET endpoint
func (s *Service) GetPidRunning() (interface{}, error) {
	return PIDEnabled{Running: s.p.Running, Heating: s.p.Heating}, nil
}

// GetPidOutput - Your GET endpoint
func (s *Service) GetPidOutput() (interface{}, error) {
	if s.p.Running {
		output := <-*s.p.Output
		return output, nil
	} else {
		return nil, errors.New("PID is not running")
	}
}

// GetStreamPidOutput - Your GET endpoint
func (s *Service) GetStreamPidOutput(w http.ResponseWriter, r *http.Request) {
	sampleRate := s.c.PidFrequency
	flusher, ok := w.(http.Flusher)
	if !ok {
		http.Error(w, "Streaming unsupported!", http.StatusInternalServerError)
		return
	}

	if !s.p.Running {
		http.Error(w, "PID not running", http.StatusInternalServerError)
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
		if !s.p.Running {
			ticker.Stop()
			return
		}
		t := <-*s.p.Output
		tJSON, err := json.Marshal(t)
		if err != nil {
			http.Error(w, "Could not marshal temperature data to JSON", http.StatusInternalServerError)
		}
		fmt.Fprintf(w, "data: %s\n\n", string(tJSON))
		flusher.Flush()

		select {
		case <-r.Context().Done():
			ticker.Stop()
			return
		default:
			<-ticker.C
		}
	}
}

// GetStreamTempCurrent - Your GET endpoint
func (s *Service) GetStreamTempCurrent(w http.ResponseWriter, r *http.Request) {
	query := r.URL.Query()
	var sampleRate time.Duration

	if sampleRateMsQuery, ok := query["sampleRateMs"]; ok {
		parsedSampleRate, err := strconv.ParseInt(sampleRateMsQuery[0], 10, 32)
		if err != nil {
			log.Println("Could not parse sampleRateMs from query string")
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

		select {
		case <-r.Context().Done():
			ticker.Stop()
			return
		default:
			<-ticker.C
		}
	}
}

// GetTemp - Your GET endpoint
func (s *Service) GetTemp(unit string) (interface{}, error) {
	return s.t.Get(s.c.TemperatureUnit)
}

// GetTempTarget - Your GET endpoint
func (s *Service) GetTempTarget() (interface{}, error) {
	return TemperatureTarget{Target: s.c.TemperatureTarget}, nil
}

// PostConfig -
func (s *Service) PostConfig(config config.Config) (interface{}, error) {
	// TODO - update PostConfig with the required logic for this service method.
	// Add api_default_service.go to the .openapi-generator-ignore to avoid overwriting this service implementation when updating open api generation.
	return nil, errors.New("service method 'PostConfig' not implemented")
}

// PostPidRunning -
func (s *Service) PostPidRunning(pidEnabled PIDEnabled) (interface{}, error) {
	if pidEnabled.Running {
		s.p.Start(s.c)
	} else {
		s.p.Stop()
	}

	return OperationResult{Ok: true}, nil
}

// PostTempTarget -
func (s *Service) PostTempTarget(temperatureTarget TemperatureTarget) (interface{}, error) {
	s.c.TemperatureTarget = temperatureTarget.Target

	if s.p.Running {
		s.p.SetTarget(temperatureTarget.Target, s.c.TemperatureUnit)
	}

	return OperationResult{Ok: true}, nil
}
