package gesha

import (
	"errors"
	"fmt"
	"log"
	"os"

	"github.com/docopt/docopt-go"
	"github.com/lukechannings/gesha/cmd/install"
	"github.com/lukechannings/gesha/cmd/start"
)

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
	version, err := Version(taggedVersion, gitHash)

	if err != nil {
		log.Printf("Warning: This build was compiled without a version")
	}

	options, err := GetOptions(os.Args[1:], version)

	if err != nil {
		log.Fatalf("There was an error processing command line flags: %v", err)
	}

	if options.Install {
		install.Cmd()
	}

	if options.Start {
		start.Cmd(options.ConfigPath, options.Verbose)
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
func Version(tag string, gitHash string) (string, error) {
	var version string
	gitURL := "https://github.com/lukechannings/gesha"

	if tag != "" {
		version = fmt.Sprintf("%s (%s/releases/tag/%s)", tag, gitURL, tag)
	} else if gitHash != "" {
		version = fmt.Sprintf("%s (%s/tree/%s)", gitHash, gitURL, gitHash)
	} else {
		return "", errors.New("taggedVersion or gitHash must be set")
	}

	return version, nil
}
