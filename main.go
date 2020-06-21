package main

import (
	"encoding/json"
	"fmt"
	"log"

	"github.com/lukechannings/gesha/internal/config"
)

func main() {
	c := config.New()

	body, err := json.MarshalIndent(c, "", "  ")

	if err != nil {
		log.Fatal(err)
	}

	fmt.Printf(string(body))
}
