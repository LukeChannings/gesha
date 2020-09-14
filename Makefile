TAGGED_VERSION = $(shell git tag --points-at HEAD)
GIT_HASH = $(shell git rev-parse --short HEAD)
LD_FLAGS = -s -w -X main.gitHash=${GIT_HASH} -X main.taggedVersion=${TAGGED_VERSION}
WEB_SRC = web/app/src
WEB_DEST = web/static/dist

all: build/linux-arm/gesha build/linux-arm64/gesha build/linux-amd64/gesha build/linux-i386/gesha build/darwin/gesha

build/linux-arm/gesha: cmd/**/*.go ${WEB_DEST}/main.js ${WEB_DEST}/main.css pkged.go
	GOOS=linux GOARCH=arm GOARM=6 go build -ldflags="${LD_FLAGS}" -o $@

build/linux-arm64/gesha: cmd/**/*.go ${WEB_DEST}/main.js ${WEB_DEST}/main.css pkged.go
	GOOS=linux GOARCH=arm64 go build -ldflags="${LD_FLAGS}" -o $@

build/linux-amd64/gesha: cmd/**/*.go ${WEB_DEST}/main.js ${WEB_DEST}/main.css pkged.go
	GOOS=linux GOARCH=amd64 go build -ldflags="${LD_FLAGS}" -o $@

build/linux-i386/gesha: cmd/**/*.go ${WEB_DEST}/main.js ${WEB_DEST}/main.css pkged.go
	GOOS=linux GOARCH=386 go build -ldflags="${LD_FLAGS}" -o $@
	
build/darwin/gesha: cmd/**/*.go ${WEB_DEST}/main.js ${WEB_DEST}/main.css pkged.go
	GOOS=darwin GOARCH=amd64 go build -ldflags="${LD_FLAGS}" -o $@

docs/api: api/openapi-spec/v1.openapi.yaml
	openapi-generator generate -i $? -g markdown -o $@

pkged.go: internal/**/*.go web/template/index.html web/web.go i18n/**.yaml ${WEB_DEST}/main.js ${WEB_DEST}/main.css
	pkger

${WEB_DEST}/main.js: ${WEB_SRC}/*.ts ${WEB_SRC}/**/*.ts web/app/node_modules
	hammer bundle --sourcemap=external --extract-css ${WEB_SRC}/main.ts $@

clean:
	rm -rf build pkged.go web/static/dist

test: web/app/node_modules
	go test -v ./...
	cd web/app && npm run lint && npm t

web/app/node_modules: web/app/package.json
	cd web/app && npm ci

pi: build/linux-arm/gesha
	scp build/linux-arm/gesha coffee-machine:~

dev:
	hammer serve --define=window.__API_BASE__='"http://192.168.20.24"' -w web/app/src web/static

format:
	cd web/app && npm run format

.PHONY: all clean test pi
