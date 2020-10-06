package pid

import (
	"log"
	"time"

	"periph.io/x/periph/conn/physic"

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

// Start - starts a new PID
func (h *Handle) Start(c *config.Config) {
	if !h.Running {
		ticker := time.NewTicker(c.PidFrequency)

		h.pidProc = &pidProc{
			t: ticker,
		}

		go func() {
			pid := pidctrl.NewPIDController(c.PID[0], c.PID[1], c.PID[2])
			pid.Set(c.TemperatureTarget.Celsius())
			pid.SetOutputLimits(0.0, 1.0)

			h.pid = pid

			h.Running = true

			var b *temp.CurrentTemp

			a, err := h.t.Get()
			b = a

			if err != nil {
				log.Fatalf("Error reading temperature! %v\n", err)
			}

			for {
				temp, err := h.t.Get()
				a = b
				b = temp

				if err != nil {
					log.Fatalf("Error reading temperature! %v\n", err)
					break
				}

				pidOutput := pid.UpdateDuration(b.Temp.Celsius(), time.Time(b.Time).Sub(time.Time(a.Time)))

				h.pidOutput = pidOutput

				if pidOutput == 1.0 && temp.Temp < c.TemperatureTarget+5.0*physic.Celsius {
					h.heatPin.Out(gpio.High)
					h.Heating = true
				} else {
					h.heatPin.Out(gpio.Low)
					h.Heating = false
				}

				if h.c.Verbose {
					log.Printf("PID | Output %v | Target %v | Temp %v | Heating %v\n", pidOutput, pid.Get(), b.Temp, h.Heating)
				}

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
	} else {
		h.heatPin.Out(gpio.Low)
		log.Println("PID not running")
	}
}

// OverrideBoilerOn disables the PID and manually sets the boiler on or off
func (h *Handle) OverrideBoilerOn() {
	h.Stop()
	h.heatPin.Out(gpio.High)
}

// OverrideBoilerOff disables the PID and manually sets the boiler off
func (h *Handle) OverrideBoilerOff() {
	h.Stop()
	h.heatPin.Out(gpio.Low)
}

// SetTarget - sets a new target temperature
func (h *Handle) SetTarget(targetTemp physic.Temperature) {
	if h.Running {
		h.pid.Set(targetTemp.Celsius())
	}
}
