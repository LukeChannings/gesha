package start

import (
	"fmt"
	"log"
	"net/http"
	"os"
	"os/signal"
	"syscall"

	"github.com/lukechannings/gesha/internal/api"
	"github.com/lukechannings/gesha/internal/config"
	"github.com/lukechannings/gesha/internal/i18n"
	"github.com/lukechannings/gesha/internal/pid"
	"github.com/lukechannings/gesha/internal/temp"
	"github.com/lukechannings/gesha/web"
	"github.com/markbates/pkger"
)

// Cmd - starts the main Gesha service
func Cmd(configPath string, verbose bool) {
	c := config.New(configPath)

	trErr := i18n.PopulateTranslations()

	if trErr != nil {
		fmt.Printf("An error occurred when loading translations: %s", trErr.Error())
	}

	t, err := temp.New(c.SpiPort)

	if err != nil {
		log.Fatal("Couldn't create a temperature stream! " + err.Error())
	}

	pid := pid.New(&c, t)

	apiService := api.NewAPIService(&c, t, &pid, configPath)
	apiController := api.NewDefaultAPIController(apiService)

	if c.PidAutostart {
		log.Println("Autostarting PID")
		pid.Start(&c)
	}

	r := api.NewRouter(apiController)
	r.Handle("/", web.Index(&c, t, &pid))
	r.PathPrefix("/").Handler(http.StripPrefix("/", http.FileServer(pkger.Dir("/web/static"))))

	done := make(chan os.Signal, 1)
	signal.Notify(done, os.Interrupt, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		log.Printf("Starting server on port %s\n", c.Port)
		if err := http.ListenAndServe(":"+c.Port, r); err != nil && err != http.ErrServerClosed {
			log.Fatalf("Error starting the server on port %s. Maybe try re-running as root?\nError message: %v\n", c.Port, err)
		}
	}()
	log.Print("Server Started")

	<-done
	log.Print("Server Stopped")

	defer pid.OverrideBoilerOff()
}
