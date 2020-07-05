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

type pidProc struct {
	t    *time.Ticker
	done *chan bool
}

// Handle - a handle for the PID instance
type Handle struct {
	Running           bool
	Heating           bool
	Output            *chan float64
	temperatureStream *chan temp.Temp
	heatPin           gpio.PinIO
	pid               *pidctrl.PIDController
	pidProc           *pidProc
}

// New creates a Handle and gets the GPIO pin
func New(pinName string, temperatureStream *chan temp.Temp) Handle {
	h := Handle{}

	h.heatPin = gpioreg.ByName(pinName)
	h.temperatureStream = temperatureStream
	h.Heating = false
	h.Running = false

	return h
}

func convTempC(value float64, unit string) float64 {
	if unit == "C" {
		return value
	}

	return value - 32*5/9
}

// Start - starts a new PID
func (h *Handle) Start(c *config.Config) {
	if !h.Running {
		ticker := time.NewTicker(c.PidFrequency)
		output := make(chan float64)

		h.pidProc = &pidProc{
			t: ticker,
		}

		h.Output = &output

		go func() {
			pid := pidctrl.NewPIDController(c.PID[0], c.PID[1], c.PID[2])
			pid.Set(convTempC(c.TemperatureTarget, c.TemperatureUnit))
			pid.SetOutputLimits(-1.0, 1.0)

			h.pid = pid

			h.Running = true

			var a, b temp.Temp = <-*h.temperatureStream, <-*h.temperatureStream

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

// Stop - stops the running PID
func (h *Handle) Stop() {
	if h.Running && h.pidProc != nil {
		fmt.Println("Sending quit signal to PID")
		h.pidProc.t.Stop()
		h.Running = false
		h.pidProc = nil
		h.heatPin.Out(gpio.Low)
	} else {
		fmt.Println("PID not running")
	}
}

// SetTarget - sets a new target temperature
func (h *Handle) SetTarget(targetTemp float64, unit string) {
	if h.Running {
		h.pid.Set(convTempC(targetTemp, unit))
	}
}
