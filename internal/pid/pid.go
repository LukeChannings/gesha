package pid

import (
	"fmt"
	"time"

	"github.com/felixge/pidctrl"
	"github.com/lukechannings/gesha/internal/config"
	"github.com/lukechannings/gesha/internal/temp"
	"periph.io/x/periph/conn/gpio"
	"periph.io/x/periph/conn/gpio/gpioreg"
)

// Handle - a handle for the PID instance
type Handle struct {
	Enabled           bool
	Heating           bool
	Output            *chan float64
	temperatureStream *chan temp.Temp
	heatPin           gpio.PinIO
	pid               *pidctrl.PIDController
}

// New creates a Handle and gets the GPIO pin
func New(pinName string, temperatureStream *chan temp.Temp) Handle {
	h := Handle{}

	h.heatPin = gpioreg.ByName(pinName)
	h.temperatureStream = temperatureStream
	h.Heating = false
	h.Enabled = false

	return h
}

func (h *Handle) Start(c *config.Config) {
	fmt.Println("asdsds")
	if !h.Enabled {
		fmt.Println("Enabling PID...")
		go func() {
			output := make(chan float64)
			h.Output = &output
			fmt.Println("Starting PID")
			h.Enabled = true

			pid := pidctrl.NewPIDController(c.P, c.I, c.D)
			h.pid = pid

			pid.Set(c.TemperatureTarget)
			pid.SetOutputLimits(-1.0, 1.0)

			var a, b temp.Temp = <-*h.temperatureStream, <-*h.temperatureStream

			ticker := time.NewTicker(time.Second)
			defer ticker.Stop()

			for {
				a = b
				b = <-*h.temperatureStream
				pidOutput := pid.UpdateDuration(b.Temp, time.Time(b.Time).Sub(time.Time(a.Time)))
				output <- pidOutput

				if pidOutput <= 0.5 {
					h.heatPin.Out(gpio.Low)
					h.Heating = false
				} else {
					h.heatPin.Out(gpio.High)
					h.Heating = true
				}

				<-ticker.C
			}
		}()
	}
}

func (h *Handle) SetTarget(targetTemp float64) {
	if h.Enabled {
		h.pid.Set(targetTemp)
	}
}
