package main

import "github.com/lukechannings/gesha/cmd/gesha"

// taggedVersion - the release version of Gesha, e.g. v1.0.0. This is set by LDFLAGS.
var taggedVersion string

// gitHash - the hash for the current git head. This is set by LDFLAGS.
var gitHash string

func main() {
	// Preferably I'd just make cmd/gesha the main package,
	// but that interferes with pkger's bundling.
	gesha.Run(taggedVersion, gitHash)
}
