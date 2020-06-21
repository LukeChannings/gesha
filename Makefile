all: clean pkged.go linux-arm darwin

linux-arm:
	GOOS=linux GOARCH=arm GOARM=6 go build -o build/arm/gesha

darwin:
	GOOS=darwin go build -o build/darwin/gesha

generate-api:
	openapi-generator generate -i ./api/openapi.yaml --git-user-id lukechannings --git-repo-id gesha -g go-server -c ./api/generator-config.yaml

pkged.go:
	pkger

clean:
	rm -rf build pkged.go

.PHONY: clean linux-arm darwin all generate-api
