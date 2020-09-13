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
			"GetStreamState",
			"GET",
			"/api/stream/state",
			c.GetStreamCurrentState,
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
			"GetCurrentState",
			"GET",
			"/api/state/current",
			c.GetCurrentState,
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

// GetConfig -
func (c *Controller) GetConfig(w http.ResponseWriter, r *http.Request) {
	result, err := c.service.GetConfig()
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// GetPidRunning -
func (c *Controller) GetPidRunning(w http.ResponseWriter, r *http.Request) {
	result, err := c.service.GetPidRunning()
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// GetPidOutput -
func (c *Controller) GetPidOutput(w http.ResponseWriter, r *http.Request) {
	result, err := c.service.GetPidOutput()
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// GetStreamPidOutput -
func (c *Controller) GetStreamPidOutput(w http.ResponseWriter, r *http.Request) {
	c.service.GetStreamPidOutput(w, r)
}

// GetStreamTempCurrent -
func (c *Controller) GetStreamTempCurrent(w http.ResponseWriter, r *http.Request) {
	c.service.GetStreamTempCurrent(w, r)
}

// GetStreamCurrentState -
func (c *Controller) GetStreamCurrentState(w http.ResponseWriter, r *http.Request) {
	c.service.GetStreamCurrentState(w, r)
}

// GetCurrentState -
func (c *Controller) GetCurrentState(w http.ResponseWriter, r *http.Request) {
	result, err := c.service.GetCurrentState()
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// GetTemp -
func (c *Controller) GetTemp(w http.ResponseWriter, r *http.Request) {
	result, err := c.service.GetTemp()
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// GetTempTarget -
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
	configWire := config.WireConfig{}
	if err := json.NewDecoder(r.Body).Decode(&configWire); err != nil {
		fmt.Println(err.Error())
		w.WriteHeader(500)
		return
	}

	parsedCfg, err := configWire.ToConfig()

	if err != nil {
		fmt.Print(err.Error())
		w.WriteHeader(500)
		return
	}

	result, err := c.service.PostConfig(parsedCfg)
	if err != nil {
		fmt.Print(err.Error())
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
