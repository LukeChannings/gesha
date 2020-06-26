all: clean pkged.go linux-arm linux-arm64 linux-amd64 darwin compress

linux-arm:
	GOOS=linux GOARCH=arm GOARM=6 go build -ldflags="-s -w" -o build/linux-arm/gesha

linux-arm64:
	GOOS=linux GOARCH=arm64 go build -ldflags="-s -w" -o build/linux-arm64/gesha

linux-amd64:
	GOOS=linux GOARCH=amd64 go build -ldflags="-s -w" -o build/linux-amd64/gesha

darwin:
	GOOS=darwin GOARCH=amd64 go build -ldflags="-s -w" -o build/darwin/gesha

compress:
	upx --brute ./build/*/gesha

generate-api:
	openapi-generator generate -i ./api/openapi.yaml --git-user-id lukechannings --git-repo-id gesha -g go-server -c ./api/generator-config.yaml

pkged.go:
	pkger

clean:
	rm -rf build pkged.go

pi: clean pkged.go linux-arm
	scp ./build/linux-arm/gesha coffee-machine:~

.PHONY: clean linux-arm darwin all generate-api pi
