package web

import (
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
	"text/template"

	"github.com/lukechannings/gesha/internal/config"
	"github.com/lukechannings/gesha/internal/i18n"
	"github.com/lukechannings/gesha/internal/pid"
	"github.com/lukechannings/gesha/internal/temp"
	"github.com/markbates/pkger"
)

type templateContext struct {
	Lang        string
	C           *config.Config
	T           *i18n.Translations
	CurrentTemp string
	TargetTemp  float64
	Heating     bool
	Running     bool
	IsTempF     bool
}

func getIndexTemplate() (*template.Template, error) {
	tmpl, err := pkger.Open("/web/template/index.html")

	if err != nil {
		log.Fatal("Could not load index.tmpl")
	}

	b, _ := ioutil.ReadAll(tmpl)

	return template.New("index").Parse(string(b))
}

// Index - serves the interpolated index page
func Index(c *config.Config, t *temp.Handle, p *pid.Handle) http.Handler {

	tmpl, err := getIndexTemplate()

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

		fmt.Println(chosenLang)

		t, err := t.Get(c.TemperatureUnit)

		if err != nil {
			http.Error(w, "Could not read the temperature", http.StatusInternalServerError)
		}

		ctx := templateContext{
			Lang:        chosenLang,
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
