package test

import (
	"testing"

	"github.com/lukechannings/gesha/internal/util"
)

// TestGetEnv - checks that GetEnv returns the default
func TestGetEnv(t *testing.T) {
	defaultValue := "super cool default"
	notSetEnv := util.GetEnv("DEFINITELY_NOT_SET", defaultValue)

	if notSetEnv != defaultValue {
		t.Errorf("Expected %v to be %v", notSetEnv, defaultValue)
	}
}
