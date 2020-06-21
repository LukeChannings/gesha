package temp

import (
	"strconv"
	"time"

	"github.com/LukeChannings/max31855"
	"periph.io/x/periph/conn/spi/spireg"
	"periph.io/x/periph/host"
)

// Handle keeps hold of the SPI device and port
type Handle struct {
	spiPort string
	dev     *max31855.Dev
}

type TempTime time.Time

func (t TempTime) MarshalText() ([]byte, error) {
	tf := time.Time(t).UnixNano() / 1e6
	return []byte(strconv.FormatInt(tf, 10)), nil
}

// Temp is a tuple of the current temperature and the time it was read
type Temp struct {
	Time TempTime `json:"time"`
	Temp float64  `json:"temp"`
}

// New connects to the SPI device and returns a handle
func New(spiPort string) (*Handle, error) {

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

	return &Handle{spiPort: spiPort, dev: dev}, nil
}

// Get - the current temperature
func (t *Handle) Get() (*Temp, error) {
	te, err := t.dev.GetTemp()

	if err != nil {
		return nil, err
	}

	return &Temp{TempTime(time.Now()), te}, nil
}

// Stream - sample the temperature on a timer
func (t *Handle) Stream(sampleRate time.Duration) (chan Temp, error) {
	c := make(chan Temp)

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
