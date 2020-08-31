package temp

import (
	"time"

	"periph.io/x/periph/conn/physic"

	"github.com/lukechannings/max31855"
	"periph.io/x/periph/conn/spi/spireg"
	"periph.io/x/periph/host"
)

// Handle keeps hold of the SPI device and port
type Handle struct {
	spiPort  string
	dev      *max31855.Dev
	tempGHBR float64
}

// CurrentTemp is a tuple of the current temperature and the time it was read
type CurrentTemp struct {
	Time time.Time          `json:"time"`
	Temp physic.Temperature `json:"temp"`
}

// New connects to the SPI device and returns a handle
func New(spiPort string, tempGHBR float64) (*Handle, error) {

	if _, err := host.Init(); err != nil {
		return nil, err
	}

	port, err := spireg.Open(spiPort)

	if err != nil {
		return nil, err
	}

	dev, err := max31855.New(port)

	if err != nil {
		return nil, err
	}

	return &Handle{spiPort: spiPort, dev: dev, tempGHBR: tempGHBR}, nil
}

// Get - the current temperature
func (t *Handle) Get() (*CurrentTemp, error) {
	te, err := t.dev.GetTemp()

	if err != nil {
		return nil, err
	}

	return &CurrentTemp{time.Now(), physic.Temperature(float64(te.Thermocouple) * t.tempGHBR)}, nil
}

// Stream - sample the temperature on a timer
func (t *Handle) Stream(sampleRate time.Duration) (chan CurrentTemp, error) {
	c := make(chan CurrentTemp)

	go func() {
		ticker := time.NewTicker(sampleRate)
		defer ticker.Stop()

		for {
			<-ticker.C
			t, err := t.Get()

			if err == nil {
				c <- *t
			}
		}
	}()

	return c, nil
}
