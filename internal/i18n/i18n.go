package i18n

import (
	"fmt"
	"io/ioutil"
	"os"
	"path"
	"strings"

	"github.com/markbates/pkger"
	"golang.org/x/text/language"
	"gopkg.in/yaml.v2"
)

// Translations - the structure of the translations doocument
type Translations struct {
	Meta struct {
		Description string `yaml:"description"`
	} `yaml:"meta"`

	Brew struct {
		NavLabel string `yaml:"navLabel"`
		Legend   string `yaml:"legend"`
	} `yaml:"brew"`

	History struct {
		NavLabel string `yaml:"navLabel"`
	} `yaml:"history"`

	Settings struct {
		NavLabel string `yaml:"navLabel"`
	} `yaml:"settings"`
}

var availableTranslations map[language.Tag]*Translations = make(map[language.Tag]*Translations)

func PopulateTranslations() error {
	err := pkger.Walk("/i18n", func(filepath string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}

		extension := path.Ext(filepath)

		if extension == ".yaml" {
			lang := strings.Replace(path.Base(filepath), extension, "", 1)
			tag, langParseErr := language.Parse(lang)
			if langParseErr != nil {
				return langParseErr
			}

			translations := Translations{}

			handle, openErr := pkger.Open(filepath)

			if openErr != nil {
				fmt.Println("Error opening ", filepath, openErr.Error())
			}

			data, _ := ioutil.ReadAll(handle)

			unmarshalErr := yaml.Unmarshal(data, &translations)

			if unmarshalErr != nil {
				return err
			}

			availableTranslations[tag] = &translations
		}

		return nil
	})

	if err != nil {
		return err
	}

	return nil
}

func GetTranslations(langs []string) (string, *Translations) {

	keys := make([]language.Tag, 0, len(availableTranslations))
	for k := range availableTranslations {
		keys = append(keys, k)
	}

	var matcher = language.NewMatcher(keys)

	key, _ := language.MatchStrings(matcher, langs...)

	baseLang, _ := key.Base()
	langRegion, regionConfidence := key.Region()

	var htmlLang string = baseLang.String()

	if regionConfidence > language.High {
		htmlLang += "-" + langRegion.String()
	}

	fmt.Printf("base %v, region %v, conf %v", baseLang, langRegion, regionConfidence)

	return htmlLang, availableTranslations[key]
}
