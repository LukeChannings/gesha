TAGGED_VERSION = $(shell git tag --points-at HEAD)
GIT_HASH = $(shell git rev-parse --short HEAD)
LD_FLAGS = -s -w -X main.gitHash=${GIT_HASH} -X main.taggedVersion=${TAGGED_VERSION}

all: clean pkged.go linux-arm linux-arm64 linux-amd64 darwin compress

linux-arm:
	mkdir -p build/linux-arm
	GOOS=linux GOARCH=arm GOARM=6 go build -ldflags="${LD_FLAGS}" -o build/linux-arm/gesha

linux-arm64:
	mkdir -p linux-arm64
	GOOS=linux GOARCH=arm64 go build -ldflags="${LD_FLAGS}" -o build/linux-arm64/gesha

linux-amd64:
	mkdir -p build/linux-amd64
	GOOS=linux GOARCH=amd64 go build -ldflags="${LD_FLAGS}" -o build/linux-amd64/gesha

darwin:
	mkdir -p build/darwin
	GOOS=darwin GOARCH=amd64 go build -ldflags="${LD_FLAGS}" -o build/darwin/gesha

compress:
	upx --brute ./build/*/gesha

generate-api:
	openapi-generator generate -i ./api/openapi.yaml --git-user-id lukechannings --git-repo-id gesha -g go-server -c ./api/generator-config.yaml

pkged.go:
	pkger

clean:
	go clean -modcache
	rm -rf build pkged.go

pi: clean pkged.go linux-arm
	scp ./build/linux-arm/gesha coffee-machine:~

test:
	go test -v ./...

.PHONY: clean linux-arm darwin all generate-api pi test
