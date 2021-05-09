package main

import (
	"embed"

	"github.com/lukechannings/gesha/cmd/gesha"
	"github.com/lukechannings/gesha/internal/i18n"
)

// taggedVersion - the release version of Gesha, e.g. v1.0.0. This is set by LDFLAGS.
var taggedVersion string

// gitHash - the hash for the current git head. This is set by LDFLAGS.
var gitHash string

// embedded files

//go:embed web/static init/gesha.service configs/rancilio-silvia.yaml
var embeddedStaticFiles embed.FS

//go:embed i18n
var embeddedTranslations embed.FS

func main() {
	gesha.EmbeddedStaticFiles = embeddedStaticFiles
	i18n.EmbeddedTranslations = embeddedTranslations

	gesha.Run(taggedVersion, gitHash)
}
