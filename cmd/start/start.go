package start

import (
	"log"
	"net/http"
	"time"

	"github.com/lukechannings/gesha/internal/api"
	"github.com/lukechannings/gesha/internal/config"
	"github.com/lukechannings/gesha/internal/pid"
	"github.com/lukechannings/gesha/internal/temp"
	"github.com/lukechannings/gesha/web"
	"github.com/markbates/pkger"
)

// Cmd - starts the main Gesha service
func Cmd(configPath string, verbose bool) {
	c := config.New(configPath)

	t, err := temp.New(c.SpiPort)

	if err != nil {
		log.Fatalf("Could not set up thermocouple. %v", err.Error())
	}

	ts, err := t.Stream(10*time.Millisecond, c.TemperatureUnit)

	if err != nil {
		log.Fatal("Couldn't create a temperature stream! " + err.Error())
	}

	pid := pid.New(c.BoilerPin, &ts)

	apiService := api.NewAPIService(&c, t, &pid, &ts)
	apiController := api.NewDefaultAPIController(apiService)

	if c.PidAutostart {
		pid.Start(c)
	}

	r := api.NewRouter(apiController)
	r.Handle("/", web.Index(&c, t, &pid))
	r.PathPrefix("/").Handler(http.StripPrefix("/", http.FileServer(pkger.Dir("/public"))))

	log.Printf("Started on port %s\n", c.Port)

	if err := http.ListenAndServe(":"+c.Port, r); err != nil {
		log.Fatalf("Error starting the server on port %s. Maybe try re-running as root?\nError message: %v\n", c.Port, err)
	}
}
