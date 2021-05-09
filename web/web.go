package web

import (
	_ "embed"
	"fmt"
	"log"
	"net/http"
	"text/template"

	"github.com/dchest/uniuri"
	"github.com/lukechannings/gesha/internal/config"
	"github.com/lukechannings/gesha/internal/i18n"
	"github.com/lukechannings/gesha/internal/pid"
	"github.com/lukechannings/gesha/internal/temp"
)

//go:embed template/index.html
var templateText string

// Index - serves the interpolated index page
func Index(c *config.Config, t *temp.Handle, p *pid.Handle) http.Handler {

	tmpl, err := template.New("index").Parse(templateText)

	if err != nil {
		log.Fatal("Failed to read template", err.Error())
	}

	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		var userLangs []string

		if langCookie, err := r.Cookie("lang"); err == nil {
			userLangs = append(userLangs, langCookie.String())
		}

		userLangs = append(userLangs, r.Header.Get("Accept-Language"))

		chosenLang, tr := i18n.GetTranslations(userLangs)

		t, err := t.Get()

		if err != nil {
			http.Error(w, "Could not read the temperature", http.StatusInternalServerError)
		}

		scriptNonce := uniuri.New()

		currentTemp := t.Temp.Celsius()

		if c.TemperatureUnit == "F" {
			currentTemp = t.Temp.Fahrenheit()
		}

		targetTemp := c.TemperatureTarget.Celsius()

		if c.TemperatureUnit == "F" {
			targetTemp = c.TemperatureTarget.Fahrenheit()
		}

		ctx := struct {
			ScriptNonce string
			Lang        string
			C           *config.Config
			T           *i18n.Translations
			CurrentTemp string
			TargetTemp  string
			Heating     bool
			Running     bool
			IsTempF     bool
		}{
			ScriptNonce: scriptNonce,
			Lang:        chosenLang,
			T:           tr,
			C:           c,
			CurrentTemp: fmt.Sprintf("%.1f", currentTemp),
			TargetTemp:  fmt.Sprintf("%.0f", targetTemp),
			IsTempF:     c.TemperatureUnit == "F",
			Running:     p.Running,
			Heating:     p.Heating,
		}

		w.Header().Add("cache-control", "no-cache")
		w.Header().Add("content-security-policy", fmt.Sprintf("script-src 'strict-dynamic' 'nonce-%v' 'unsafe-inline' http: https:; object-src 'none'; base-uri 'none';", scriptNonce))

		tmpl.Execute(w, ctx)
	})
}
