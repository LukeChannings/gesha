package main

import (
	"fmt"
	"log"
	"net/http"
	"time"

	"github.com/lukechannings/gesha/api"
	"github.com/lukechannings/gesha/internal/config"
	"github.com/lukechannings/gesha/internal/pid"
	"github.com/lukechannings/gesha/internal/temp"
	"github.com/lukechannings/gesha/internal/util"
	"github.com/lukechannings/gesha/web"
	"github.com/markbates/pkger"
)

func main() {
	c := config.New()

	t, err := temp.New(c.SpiPort)

	if err != nil {
		log.Fatalf("Could not set up thermocouple. %v", err.Error())
	}

	port := util.GetEnv("PORT", "3000")
	fmt.Printf("Running on port %v\n\nConfig: %+v\n", port, c)

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

	log.Fatal(http.ListenAndServe(":"+port, r))
}
