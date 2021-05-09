package i18n

import (
	"embed"
	"errors"
	"fmt"
	"path"
	"strings"

	"golang.org/x/text/language"
	"gopkg.in/yaml.v2"
)

var EmbeddedTranslations embed.FS

// Translations - the structure of the translations doocument
type Translations struct {
	Meta struct {
		Description string `yaml:"description"`
	} `yaml:"meta"`
	Global struct {
		Units struct {
			Celcius struct {
				Label string `yaml:"label"`
				Short string `yaml:"short"`
				Title string `yaml:"title"`
			} `yaml:"celcius"`
			Fahrenheit struct {
				Label string `yaml:"label"`
				Short string `yaml:"short"`
				Title string `yaml:"title"`
			} `yaml:"fahrenheit"`
			Grams struct {
				Label string `yaml:"label"`
				Short string `yaml:"short"`
			} `yaml:"grams"`
			Seconds struct {
				Label string `yaml:"label"`
				Short string `yaml:"short"`
			} `yaml:"seconds"`
		} `yaml:"units"`
	} `yaml:"global"`
	Brew struct {
		NavLabel     string `yaml:"navLabel"`
		Legend       string `yaml:"legend"`
		ActionButton struct {
			Label string `yaml:"label"`
		} `yaml:"actionButton"`
		Temp struct {
			Description string `yaml:"description"`
			Label       string `yaml:"label"`
			Title       string `yaml:"title"`
		} `yaml:"temp"`
		Grind struct {
			Description string `yaml:"description"`
			Label       string `yaml:"label"`
		} `yaml:"grind"`
		Dose struct {
			Description string `yaml:"description"`
			Label       string `yaml:"label"`
		} `yaml:"dose"`
	} `yaml:"brew"`
	Timer struct {
		DoneButton struct {
			Label string `yaml:"label"`
		} `yaml:"doneButton"`
		CancelButton struct {
			Label string `yaml:"label"`
		} `yaml:"cancelButton"`
	} `yaml:"timer"`
	History struct {
		NavLabel         string `yaml:"navLabel"`
		EmptyMessage     string `yaml:"emptyMessage"`
		ItemDeleteButton struct {
			Label string `yaml:"label"`
		} `yaml:"itemDeleteButton"`
	} `yaml:"history"`
	Settings struct {
		NavLabel   string `yaml:"navLabel"`
		SaveButton struct {
			Label string `yaml:"label"`
		} `yaml:"saveButton"`
		ServerPort struct {
			Label string `yaml:"label"`
		} `yaml:"serverPort"`
		BoilerPin struct {
			Label       string `yaml:"label"`
			Description string `yaml:"description"`
		} `yaml:"boilerPin"`
		SpiPort struct {
			Label       string `yaml:"label"`
			Description string `yaml:"description"`
		} `yaml:"spiPort"`
		TemperatureSampleRate struct {
			Label       string `yaml:"label"`
			Description string `yaml:"description"`
		} `yaml:"temperatureSampleRate"`
		TemperatureUnit struct {
			Label string `yaml:"label"`
		} `yaml:"temperatureUnit"`
		TemperatureTarget struct {
			Label string `yaml:"label"`
		} `yaml:"temperatureTarget"`
		Pid struct {
			Label  string `yaml:"label"`
			Attr   string `yaml:"attr"`
			PLabel string `yaml:"pLabel"`
			ILabel string `yaml:"iLabel"`
			DLabel string `yaml:"dLabel"`
		} `yaml:"pid"`
		ThemeColor struct {
			Label string `yaml:"label"`
		} `yaml:"themeColor"`
		PidFrequency struct {
			Label string `yaml:"label"`
		} `yaml:"pidFrequency"`
		PidAutostart struct {
			Label       string `yaml:"label"`
			Description string `yaml:"description"`
		} `yaml:"pidAutostart"`
		Verbose struct {
			Label       string `yaml:"label"`
			Description string `yaml:"description"`
		} `yaml:"verbose"`
		TemperatureGHBR struct {
			Label       string `yaml:"label"`
			Description string `yaml:"description"`
		} `yaml:"temperatureGHBR"`
	} `yaml:"settings"`
}

var availableTranslations map[language.Tag]*Translations = make(map[language.Tag]*Translations)

// PopulateTranslations - populates available translations with translations files from /i18n/*.yaml
func PopulateTranslations() error {
	entries, err := EmbeddedTranslations.ReadDir("i18n")
	for _, entry := range entries {
		filepath := "i18n/" + entry.Name()
		extension := path.Ext(filepath)
		if entry.Type().IsRegular() && extension == ".yaml" {
			lang := strings.Replace(entry.Name(), extension, "", 1)
			tag, langParseErr := language.Parse(lang)
			if langParseErr != nil {
				return langParseErr
			}
			translations := Translations{}

			data, err := EmbeddedTranslations.ReadFile(filepath)

			if err != nil {
				fmt.Println("Error opening ", filepath, err.Error())
				continue
			}

			unmarshalErr := yaml.Unmarshal(data, &translations)

			if unmarshalErr != nil {
				fmt.Printf("Error loading translation file %v. %v.\n", filepath, unmarshalErr.Error())
				return err
			}

			availableTranslations[tag] = &translations
		}
	}

	if err != nil {
		return err
	}

	if langs := GetAvailableLanguages(); len(langs) == 0 {
		return errors.New("there are no translations available")
	}

	return nil
}

// GetAvailableLanguages - gets a list of the loaded translations
func GetAvailableLanguages() []language.Tag {
	keys := make([]language.Tag, 0, len(availableTranslations))
	for k := range availableTranslations {
		keys = append(keys, k)
	}
	return keys
}

// GetTranslations - Gets a translations set based on a list of languages by preference
func GetTranslations(langs []string) (string, *Translations) {

	availableLangs := GetAvailableLanguages()

	var matcher = language.NewMatcher(availableLangs)

	key, _ := language.MatchStrings(matcher, "en-GB")

	baseLang, _ := key.Base()
	langRegion, regionConfidence := key.Region()

	var htmlLang string = baseLang.String()

	if regionConfidence > language.High {
		htmlLang += "-" + langRegion.String()
	}

	return htmlLang, availableTranslations[key]
}
