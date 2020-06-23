package web

import (
	"encoding/json"
	"io/ioutil"
	"log"
	"net/http"
	"text/template"

	"golang.org/x/text/language"

	"github.com/lukechannings/gesha/internal/config"
	"github.com/markbates/pkger"
)

var matcher = language.NewMatcher([]language.Tag{
	language.English,
})

func getIndexTemplate() (*template.Template, error) {
	tmpl, err := pkger.Open("/web/index.template")

	if err != nil {
		log.Fatal("Could not load index.template")
	}

	b, _ := ioutil.ReadAll(tmpl)

	return template.New("index").Parse(string(b))
}

func getTranslations(lang language.Tag) interface{} {
	baseLang := lang.Parent().String()

	if baseLang == "en" {
		file, err := pkger.Open("/i18n/en.json")

		if err != nil {
			log.Fatal("Failed to load translation file ", baseLang)
		}

		defer file.Close()

		var strings interface{}

		byteValue, _ := ioutil.ReadAll(file)

		json.Unmarshal(byteValue, &strings)

		return strings
	}

	return nil
}

// Index - serves the interpolated index page
func Index(c *config.Config) http.Handler {

	tmpl, err := getIndexTemplate()

	if err != nil {
		log.Fatal("Failed to read template", err.Error())
	}

	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		lang, _ := r.Cookie("lang")
		accept := r.Header.Get("Accept-Language")
		tag, _ := language.MatchStrings(matcher, lang.String(), accept)

		tr := getTranslations(tag)

		tmpl.Execute(w, tr)
	})
}
