package gesha

import (
	"testing"
)

func TestEmptyVersion(t *testing.T) {
	version := Version("", "")

	if version != "development" {
		t.Fatalf("Expected development, got %s\n", version)
	}
}

func TestHashVersion(t *testing.T) {
	version := Version("", "abcdef")

	expectedVersion := "abcdef (https://github.com/lukechannings/gesha/tree/abcdef)"
	if version != expectedVersion {
		t.Fatalf("Expected %s, but got %s", expectedVersion, version)
	}
}

func TestReleaseVersion(t *testing.T) {
	version := Version("v1.0.0", "abcdef")

	expectedVersion := "v1.0.0 (https://github.com/lukechannings/gesha/releases/tag/v1.0.0)"
	if version != expectedVersion {
		t.Fatalf("Expected %s, but got %s", expectedVersion, version)
	}
}

func TestGetOptionsStart(t *testing.T) {
	opts, err := GetOptions([]string{"start"}, "")

	if err != nil {
		t.Fatalf("Error getting options: %v", err)
	}

	if !opts.Start {
		t.Fatalf("Expected Start to be true, but it was false")
	}
}

func TestGetOptionsStartWithConfigPath(t *testing.T) {
	configPath := "/foo/bar.yaml"
	opts, err := GetOptions([]string{"start", "-c", configPath}, "")

	if err != nil {
		t.Fatalf("Error getting options: %v", err)
	}

	if opts.ConfigPath == "" {
		t.Fatalf("Expected ConfigPath to be %v, but it was \"\"", configPath)
	}
}

func TestGetOptionsInstall(t *testing.T) {
	opts, err := GetOptions([]string{"install"}, "")

	if err != nil {
		t.Fatalf("Error getting options: %v", err)
	}

	if !opts.Install {
		t.Fatalf("Expected install to be true")
	}
}
