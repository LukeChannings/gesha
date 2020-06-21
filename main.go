package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"

	"github.com/lukechannings/gesha/api"
	"github.com/lukechannings/gesha/internal/config"
	"github.com/lukechannings/gesha/internal/temp"
	"github.com/lukechannings/gesha/internal/util"
	"github.com/markbates/pkger"
)

func main() {
	c := config.New()

	config, _ := json.MarshalIndent(c, "", "  ")

	t, err := temp.New(c.SpiPort)

	if err != nil {
		// log.Fatalf("Could not set up thermocouple. %s", err.Error())
		fmt.Println("No thermocouple connected!")
	}

	port := util.GetEnv("PORT", "3000")
	fmt.Printf("Running on port %s\n\nRunning Config: %s\n", port, string(config))

	temp, err := t.Get()

	fmt.Printf("Current Temperature: %f", temp.Temp)

	DefaultAPIService := api.NewDefaultAPIService(&c, t)
	DefaultAPIController := api.NewDefaultAPIController(DefaultAPIService)

	r := api.NewRouter(DefaultAPIController)
	r.PathPrefix("/").Handler(http.StripPrefix("/", http.FileServer(pkger.Dir("/web"))))

	log.Fatal(http.ListenAndServe(":"+port, r))
}
