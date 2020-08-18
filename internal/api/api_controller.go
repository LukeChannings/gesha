package api

import (
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/lukechannings/gesha/internal/config"
)

// A Controller binds http requests to an api service and writes the service results to the http response
type Controller struct {
	service Servicer
}

// NewDefaultAPIController creates a default api controller
func NewDefaultAPIController(s Servicer) Router {
	return &Controller{service: s}
}

// Routes returns all of the api route for the DefaultAPIController
func (c *Controller) Routes() Routes {
	return Routes{
		{
			"GetConfig",
			"GET",
			"/api/config",
			c.GetConfig,
		},
		{
			"GetPidRunning",
			"GET",
			"/api/pid/running",
			c.GetPidRunning,
		},
		{
			"GetPidOutput",
			"GET",
			"/api/pid/output",
			c.GetPidOutput,
		},
		{
			"GetStreamPidOutput",
			"GET",
			"/api/stream/pid/output",
			c.GetStreamPidOutput,
		},
		{
			"GetStreamTempCurrent",
			"GET",
			"/api/stream/temp/current",
			c.GetStreamTempCurrent,
		},
		{
			"GetTemp",
			"GET",
			"/api/temp/current",
			c.GetTemp,
		},
		{
			"GetTempTarget",
			"GET",
			"/api/temp/target",
			c.GetTempTarget,
		},
		{
			"PostConfig",
			"POST",
			"/api/config",
			c.PostConfig,
		},
		{
			"PostPidRunning",
			"POST",
			"/api/pid/running",
			c.PostPidRunning,
		},
		{
			"PostTempTarget",
			"POST",
			"/api/temp/target",
			c.PostTempTarget,
		},
	}
}

// GetConfig - Your GET endpoint
func (c *Controller) GetConfig(w http.ResponseWriter, r *http.Request) {
	result, err := c.service.GetConfig()
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// GetPidRunning - Your GET endpoint
func (c *Controller) GetPidRunning(w http.ResponseWriter, r *http.Request) {
	result, err := c.service.GetPidRunning()
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// GetPidOutput - Your GET endpoint
func (c *Controller) GetPidOutput(w http.ResponseWriter, r *http.Request) {
	result, err := c.service.GetPidOutput()
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// GetStreamPidOutput - Your GET endpoint
func (c *Controller) GetStreamPidOutput(w http.ResponseWriter, r *http.Request) {
	c.service.GetStreamPidOutput(w, r)
}

// GetStreamTempCurrent - Your GET endpoint
func (c *Controller) GetStreamTempCurrent(w http.ResponseWriter, r *http.Request) {
	c.service.GetStreamTempCurrent(w, r)
}

// GetTemp - Your GET endpoint
func (c *Controller) GetTemp(w http.ResponseWriter, r *http.Request) {
	query := r.URL.Query()
	unit := query.Get("unit")
	result, err := c.service.GetTemp(unit)
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// GetTempTarget - Your GET endpoint
func (c *Controller) GetTempTarget(w http.ResponseWriter, r *http.Request) {
	result, err := c.service.GetTempTarget()
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// PostConfig -
func (c *Controller) PostConfig(w http.ResponseWriter, r *http.Request) {
	config := &config.Config{}
	if err := json.NewDecoder(r.Body).Decode(&config); err != nil {
		fmt.Println(err.Error())
		w.WriteHeader(500)
		return
	}

	result, err := c.service.PostConfig(*config)
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// PostPidRunning -
func (c *Controller) PostPidRunning(w http.ResponseWriter, r *http.Request) {
	pidEnabled := &PIDEnabled{}
	if err := json.NewDecoder(r.Body).Decode(&pidEnabled); err != nil {
		w.WriteHeader(500)
		return
	}

	result, err := c.service.PostPidRunning(*pidEnabled)
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// PostTempTarget -
func (c *Controller) PostTempTarget(w http.ResponseWriter, r *http.Request) {
	temperatureTarget := &TemperatureTarget{}
	if err := json.NewDecoder(r.Body).Decode(&temperatureTarget); err != nil {
		w.WriteHeader(500)
		return
	}

	result, err := c.service.PostTempTarget(*temperatureTarget)
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}
