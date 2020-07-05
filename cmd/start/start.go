package start

import (
	"fmt"
	"log"
	"net/http"
	"time"

	"github.com/lukechannings/gesha/api"
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

	DefaultAPIService := api.NewDefaultAPIService(&c, t, &pid, &ts)
	DefaultAPIController := api.NewDefaultAPIController(DefaultAPIService)

	r := api.NewRouter(DefaultAPIController)
	r.Handle("/", web.Index(&c, t, &pid))
	r.PathPrefix("/").Handler(http.StripPrefix("/", http.FileServer(pkger.Dir("/public"))))

	fmt.Printf("Gesha started on port %s\n", c.Port)
	log.Fatal(http.ListenAndServe(":"+c.Port, r))
}
