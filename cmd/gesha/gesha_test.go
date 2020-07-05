package gesha

import (
	"testing"
)

func TestEmptyVersion(t *testing.T) {
	_, err := Version("", "")

	if err.Error() != "taggedVersion or gitHash must be set" {
		t.Fatal("Passing an empty taggedVersion and gitHash to Version() should error")
	}
}

func TestHashVersion(t *testing.T) {
	version, err := Version("", "abcdef")

	if err != nil {
		t.Fatalf("Version should not error, but did with %v", err)
	}

	expectedVersion := "abcdef (https://github.com/lukechannings/gesha/tree/abcdef)"
	if version != expectedVersion {
		t.Fatalf("Expected %s, but got %s", expectedVersion, version)
	}
}

func TestReleaseVersion(t *testing.T) {
	version, err := Version("v1.0.0", "abcdef")

	if err != nil {
		t.Fatalf("Version should not error, but did with %v", err)
	}

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
