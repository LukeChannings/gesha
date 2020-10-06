package history

import (
	"time"

	"periph.io/x/periph/conn/physic"
)

type Shot struct {
	Time   time.Time
	Temp   Temp
	Dose   float64
	Grind  float64
	Brewer string
	Coffee Coffee

	Yield    float64
	Duration time.Duration

	Rating       Rating
	TastingNotes string
}

type Rating struct {
	Bitterness int8
	Sourness   int8
	Sweetness  int8
	Body       int8
	Aftertaste int8
	Acidity    int8
	Overall    int8
}

type Temp struct {
	Target physic.Temperature
	Actual physic.Temperature
	Graph  []struct {
		Time time.Time
		Temp physic.Temperature
	}
}

type Coffee struct {
	Name    string
	Roaster string
}
