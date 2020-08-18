package web

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"text/template"

	"golang.org/x/text/language"

	"github.com/lukechannings/gesha/internal/config"
	"github.com/lukechannings/gesha/internal/pid"
	"github.com/lukechannings/gesha/internal/temp"
	"github.com/markbates/pkger"
)

type templateContext struct {
	C           *config.Config
	T           *Translations
	CurrentTemp string
	TargetTemp  float64
	Heating     bool
	Running     bool
	IsTempF     bool
}

var matcher = language.NewMatcher([]language.Tag{
	language.English,
})

func getIndexTemplate() (*template.Template, error) {
	tmpl, err := pkger.Open("/web/template/index.html")

	if err != nil {
		log.Fatal("Could not load index.tmpl")
	}

	b, _ := ioutil.ReadAll(tmpl)

	return template.New("index").Parse(string(b))
}

type Translations struct {
	TemperatureCurrentTitle     string `json:"temperatureCurrentTitle"`
	TemperatureSubtextLag       string `json:"temperatureSubtextLag"`
	TemperatureTargetTitle      string `json:"temperatureTargetTitle"`
	TemperatureTargetButton     string `json:"temperatureTargetButton"`
	PidTitle                    string `json:"pidTitle"`
	PidButton                   string `json:"pidButton"`
	HeatTitle                   string `json:"heatTitle"`
	GlobalOn                    string `json:"globalOn"`
	GlobalOff                   string `json:"globalOff"`
	MessageTargetTempSetSuccess string `json:"messageTargetTempSetSuccess"`
	MessageTargetTempSetFailure string `json:"messageTargetTempSetFailure"`
	MessageStartPidSuccess      string `json:"messageStartPidSuccess"`
	MessageStartPidFailure      string `json:"messageStartPidFailure"`
	MessageStopPidSuccess       string `json:"messageStopPidSuccess"`
	MessageStopPidFailure       string `json:"messageStopPidFailure"`
}

func getTranslations(lang language.Tag) *Translations {
	baseLang := lang.Parent().String()

	if baseLang == "en" {
		file, err := pkger.Open("/i18n/en.json")

		if err != nil {
			log.Fatal("Failed to load translation file ", baseLang)
		}

		defer file.Close()

		tr := new(Translations)

		byteValue, _ := ioutil.ReadAll(file)

		json.Unmarshal(byteValue, &tr)

		return tr
	}

	return nil
}

// Index - serves the interpolated index page
func Index(c *config.Config, t *temp.Handle, p *pid.Handle) http.Handler {

	tmpl, err := getIndexTemplate()

	if err != nil {
		log.Fatal("Failed to read template", err.Error())
	}

	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		lang, _ := r.Cookie("lang")
		accept := r.Header.Get("Accept-Language")
		tag, _ := language.MatchStrings(matcher, lang.String(), accept)

		tr := getTranslations(tag)

		t, err := t.Get(c.TemperatureUnit)

		if err != nil {
			http.Error(w, "Could not read the temperature", http.StatusInternalServerError)
		}

		ctx := templateContext{
			T:           tr,
			C:           c,
			CurrentTemp: fmt.Sprintf("%.1f", t.Temp),
			TargetTemp:  c.TemperatureTarget,
			IsTempF:     c.TemperatureUnit == "F",
			Running:     p.Running,
			Heating:     p.Heating,
		}

		w.Header().Add("cache-control", "no-cache")

		tmpl.Execute(w, ctx)
	})
}
