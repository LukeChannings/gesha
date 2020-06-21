package temp

import (
	"time"

	"github.com/LukeChannings/max31855"
	"periph.io/x/periph/conn/spi/spireg"
	"periph.io/x/periph/host"
)

type Temp struct {
	spiPort string
	dev     *max31855.Dev
}

type TempValue struct {
	Time int64   `json:"time"`
	Temp float64 `json:"temp"`
}

func New(spiPort string) (*Temp, error) {

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

	return &Temp{spiPort: spiPort, dev: dev}, nil
}

func (t *Temp) Get() (*TempValue, error) {
	te, err := t.dev.GetTemp()

	if err != nil {
		return nil, err
	}

	return &TempValue{time.Now().UnixNano() / 1e6, te}, nil
}

func (t *Temp) Stream() (chan TempValue, error) {
	c := make(chan TempValue)

	go func() {
		ticker := time.NewTicker(10 * time.Millisecond)
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
