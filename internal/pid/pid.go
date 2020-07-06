package pid

import (
	"log"
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
	Running   bool
	Heating   bool
	t         *temp.Handle
	c         *config.Config
	heatPin   gpio.PinIO
	pid       *pidctrl.PIDController
	pidProc   *pidProc
	pidOutput float64
}

// New creates a Handle and gets the GPIO pin
func New(c *config.Config, t *temp.Handle) Handle {
	h := Handle{}

	h.c = c

	h.heatPin = gpioreg.ByName(h.c.BoilerPin)
	h.t = t
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

		h.pidProc = &pidProc{
			t: ticker,
		}

		go func() {
			pid := pidctrl.NewPIDController(c.PID[0], c.PID[1], c.PID[2])
			pid.Set(convTempC(c.TemperatureTarget, c.TemperatureUnit))
			pid.SetOutputLimits(-1.0, 1.0)

			h.pid = pid

			h.Running = true

			var b *temp.Temp

			a, _ := h.t.Get(h.c.TemperatureUnit)
			b = a

			for {
				temp, _ := h.t.Get(h.c.TemperatureUnit)
				a = b
				b = temp

				pidOutput := pid.UpdateDuration(b.Temp, time.Time(b.Time).Sub(time.Time(a.Time)))

				h.pidOutput = pidOutput

				if pidOutput <= 0.5 {
					h.heatPin.Out(gpio.Low)
					h.Heating = false
				} else {
					h.heatPin.Out(gpio.High)
					h.Heating = true
				}

				log.Printf("PID | Output %v | Heating %v\n", pidOutput, h.Heating)
				<-ticker.C
			}
		}()
	}
}

// Output returns the current PID output
func (h *Handle) Output() float64 {
	return h.pidOutput
}

// Stop - stops the running PID
func (h *Handle) Stop() {
	if h.Running && h.pidProc != nil {
		log.Println("Sending quit signal to PID")
		h.pidProc.t.Stop()
		h.Running = false
		h.pidProc = nil
		h.heatPin.Out(gpio.Low)
	} else {
		log.Println("PID not running")
	}
}

// SetTarget - sets a new target temperature
func (h *Handle) SetTarget(targetTemp float64, unit string) {
	if h.Running {
		h.pid.Set(convTempC(targetTemp, unit))
	}
}
