package api

import (
	"encoding/json"
	"net/http"
	"strings"

	"github.com/lukechannings/gesha/internal/config"
)

// A DefaultAPIController binds http requests to an api service and writes the service results to the http response
type DefaultAPIController struct {
	service DefaultAPIServicer
}

// NewDefaultAPIController creates a default api controller
func NewDefaultAPIController(s DefaultAPIServicer) Router {
	return &DefaultAPIController{service: s}
}

// Routes returns all of the api route for the DefaultAPIController
func (c *DefaultAPIController) Routes() Routes {
	return Routes{
		{
			"GetConfig",
			strings.ToUpper("Get"),
			"/api/config",
			c.GetConfig,
		},
		{
			"GetPidEnabled",
			strings.ToUpper("Get"),
			"/api/pid/enabled",
			c.GetPidEnabled,
		},
		{
			"GetPidOutput",
			strings.ToUpper("Get"),
			"/api/pid/output",
			c.GetPidOutput,
		},
		{
			"GetStreamPidOutput",
			strings.ToUpper("Get"),
			"/api/stream/pid/output",
			c.GetStreamPidOutput,
		},
		{
			"GetStreamTempCurrent",
			strings.ToUpper("Get"),
			"/api/stream/temp/current",
			c.GetStreamTempCurrent,
		},
		{
			"GetTemp",
			strings.ToUpper("Get"),
			"/api/temp/current",
			c.GetTemp,
		},
		{
			"GetTempTarget",
			strings.ToUpper("Get"),
			"/api/temp/target",
			c.GetTempTarget,
		},
		{
			"PostConfig",
			strings.ToUpper("Post"),
			"/api/config",
			c.PostConfig,
		},
		{
			"PostPidEnabled",
			strings.ToUpper("Post"),
			"/api/pid/enabled",
			c.PostPidEnabled,
		},
		{
			"PostTempTarget",
			strings.ToUpper("Post"),
			"/api/temp/target",
			c.PostTempTarget,
		},
	}
}

// GetConfig - Your GET endpoint
func (c *DefaultAPIController) GetConfig(w http.ResponseWriter, r *http.Request) {
	result, err := c.service.GetConfig()
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// GetPidEnabled - Your GET endpoint
func (c *DefaultAPIController) GetPidEnabled(w http.ResponseWriter, r *http.Request) {
	result, err := c.service.GetPidEnabled()
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// GetPidOutput - Your GET endpoint
func (c *DefaultAPIController) GetPidOutput(w http.ResponseWriter, r *http.Request) {
	result, err := c.service.GetPidOutput()
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// GetStreamPidOutput - Your GET endpoint
func (c *DefaultAPIController) GetStreamPidOutput(w http.ResponseWriter, r *http.Request) {
	c.service.GetStreamPidOutput(w, r)
}

// GetStreamTempCurrent - Your GET endpoint
func (c *DefaultAPIController) GetStreamTempCurrent(w http.ResponseWriter, r *http.Request) {
	c.service.GetStreamTempCurrent(w, r)
}

// GetTemp - Your GET endpoint
func (c *DefaultAPIController) GetTemp(w http.ResponseWriter, r *http.Request) {
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
func (c *DefaultAPIController) GetTempTarget(w http.ResponseWriter, r *http.Request) {
	result, err := c.service.GetTempTarget()
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// PostConfig -
func (c *DefaultAPIController) PostConfig(w http.ResponseWriter, r *http.Request) {
	config := &config.Config{}
	if err := json.NewDecoder(r.Body).Decode(&config); err != nil {
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

// PostPidEnabled -
func (c *DefaultAPIController) PostPidEnabled(w http.ResponseWriter, r *http.Request) {
	pidEnabled := &PIDEnabled{}
	if err := json.NewDecoder(r.Body).Decode(&pidEnabled); err != nil {
		w.WriteHeader(500)
		return
	}

	result, err := c.service.PostPidEnabled(*pidEnabled)
	if err != nil {
		w.WriteHeader(500)
		return
	}

	EncodeJSONResponse(result, nil, w)
}

// PostTempTarget -
func (c *DefaultAPIController) PostTempTarget(w http.ResponseWriter, r *http.Request) {
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
