package gesha

import (
	"embed"
	"fmt"
	"io/fs"
	"io/ioutil"
	"log"
	"net/http"
	"os"
	"os/exec"
	"os/signal"
	"path"
	"syscall"

	"github.com/gorilla/handlers"

	"github.com/docopt/docopt-go"
	"github.com/lukechannings/gesha/internal/api"
	"github.com/lukechannings/gesha/internal/config"
	"github.com/lukechannings/gesha/internal/i18n"
	"github.com/lukechannings/gesha/internal/pid"
	"github.com/lukechannings/gesha/internal/temp"
	"github.com/lukechannings/gesha/web"
)

var EmbeddedStaticFiles embed.FS

// Usage - the CLI definition, used as a DSL by docopt.
const Usage string = `
Gesha

  Usage:
    gesha start [-c=<path> | --config-path=<path>]
    gesha install
    gesha -h | --help
    gesha -v | --version

  Options:
    -c=<path> --config-path=<path>     path to a configuration file. see docs for config options
    -h --help                          show this help message and exit
    -v --version                       show version and exit
    --verbose                          print debug information to the console
`

// Options - CLI Options parsed from the usage
type Options struct {
	Start      bool
	Install    bool
	ShowConfig bool
	ConfigPath string
	Verbose    bool
}

// Run - starts the Gesha CLI
func Run(taggedVersion string, gitHash string) {

	version := Version(taggedVersion, gitHash)

	options, err := GetOptions(os.Args[1:], version)

	if err != nil {
		log.Fatalf("There was an error processing command line flags: %v", err)
	}

	if options.Install {
		install()
	}

	if options.Start {
		start(options.ConfigPath, options.Verbose)
	}
}

// GetOptions - parses args into an Options struct
func GetOptions(args []string, version string) (*Options, error) {
	opts, err := docopt.ParseArgs(Usage, args, version)

	if err != nil {
		return nil, err
	}

	options := Options{}
	opts.Bind(&options)
	return &options, nil
}

// Version - Formats the version and adds a github link
func Version(tag string, gitHash string) string {
	var version string
	gitURL := "https://github.com/lukechannings/gesha"

	if tag != "" {
		version = fmt.Sprintf("%s (%s/releases/tag/%s)", tag, gitURL, tag)
	} else if gitHash != "" {
		version = fmt.Sprintf("%s (%s/tree/%s)", gitHash, gitURL, gitHash)
	} else {
		return "development"
	}

	return version
}

func start(configPath string, verbose bool) {
	c := config.Load(configPath)

	trErr := i18n.PopulateTranslations()

	if trErr != nil {
		log.Fatalf("An error occurred when loading translations: %v", trErr.Error())
	}

	t, err := temp.New(c.SpiPort, c.TemperatureGHBR)

	if err != nil {
		log.Fatalf("Couldn't create a temperature stream! %v", err.Error())
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
	webStatic, err := fs.Sub(EmbeddedStaticFiles, "web/static")
	if err != nil {
		log.Fatalf("Failed to create a subfilesystem for web/static.")
	}
	r.PathPrefix("/").Handler(http.StripPrefix("/", http.FileServer(http.FS(webStatic))))

	done := make(chan os.Signal, 1)
	signal.Notify(done, os.Interrupt, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		log.Printf("Starting server on port %s\n", c.Port)
		headersOk := handlers.AllowedHeaders([]string{"X-Requested-With", "Content-Type"})
		originsOk := handlers.AllowedOrigins([]string{"*"})
		methodsOk := handlers.AllowedMethods([]string{"GET", "HEAD", "POST", "PUT", "OPTIONS"})
		if err := http.ListenAndServe(":"+c.Port, handlers.CORS(originsOk, headersOk, methodsOk)(r)); err != nil && err != http.ErrServerClosed {
			log.Fatalf("Error starting the server on port %s. Maybe try re-running as root?\nError message: %v\n", c.Port, err)
		}
	}()
	log.Print("Server Started")

	<-done
	log.Print("Server Stopped")

	defer pid.OverrideBoilerOff()
}

func install() {
	if hasSystemd() {
		fmt.Println("Installing...")

		installFile("init/gesha.service", "/etc/systemd/system/gesha.service", true, true)
		installFile("configs/rancilio-silvia.yaml", "/etc/gesha/config.yaml", false, true)

		geshaPath, _ := os.Executable()
		installFile(geshaPath, "/usr/local/bin/gesha", true, false)

		fmt.Println("Installation complete.")
		fmt.Println("To start on boot, run: sudo systemctl enable gesha")
		fmt.Println("To start now: sudo systemctl start gesha")
		fmt.Println("Configuration file can be edited at /etc/gesha/config.yaml")
	} else {
		fmt.Println("This OS does not use systemd. Please install manually.")
	}
}

func installFile(fromPath string, toPath string, overwrite bool, bundle bool) {
	if _, err := os.Stat(toPath); os.IsNotExist(err) || overwrite {
		var data []byte

		if !bundle {
			if d, err := EmbeddedStaticFiles.ReadFile(fromPath); err == nil {
				data = d
			}
		} else {
			sysData, err := ioutil.ReadFile(fromPath)
			if err != nil {
				log.Fatalf("Coudn't read %v\n", fromPath)
			}
			data = sysData
		}

		mkdirError := os.MkdirAll(path.Dir(toPath), os.ModePerm)
		if mkdirError != nil {
			log.Fatalf("Failed to create %v\n", path.Dir(toPath))
		}

		writeErr := ioutil.WriteFile(toPath, data, 0644)

		fmt.Printf("Writing %v to %v\n", fromPath, toPath)

		if writeErr != nil {
			log.Fatalf("Failed to write %v! %v\n", toPath, writeErr)
		}
	}
}

func hasSystemd() bool {
	_, err := exec.LookPath("systemd")
	return err == nil
}
