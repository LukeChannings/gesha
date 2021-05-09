TAGGED_VERSION = $(shell git tag --points-at HEAD)
GIT_HASH = $(shell git rev-parse --short HEAD)
LD_FLAGS = -s -w -X main.gitHash=${GIT_HASH} -X main.taggedVersion=${TAGGED_VERSION}
ESBUILD_FLAGS = --format=esm --bundle --sourcemap
WEB_APP = web/app
WEB_SRC = web/app/src
WEB_DEST = web/static/dist

all: build/linux-arm/gesha build/linux-arm64/gesha build/linux-amd64/gesha build/linux-i386/gesha build/darwin/gesha

build/linux-arm/gesha: cmd/**/*.go internal/**/*.go ${WEB_DEST}/*
	GOOS=linux GOARCH=arm GOARM=6 go build -ldflags="${LD_FLAGS}" -o $@

build/linux-arm64/gesha: cmd/**/*.go internal/**/*.go ${WEB_DEST}/*
	GOOS=linux GOARCH=arm64 go build -ldflags="${LD_FLAGS}" -o $@

build/linux-amd64/gesha: cmd/**/*.go internal/**/*.go ${WEB_DEST}/*
	GOOS=linux GOARCH=amd64 go build -ldflags="${LD_FLAGS}" -o $@

build/linux-i386/gesha: cmd/**/*.go internal/**/*.go ${WEB_DEST}/*
	GOOS=linux GOARCH=386 go build -ldflags="${LD_FLAGS}" -o $@
	
build/darwin/gesha: cmd/**/*.go internal/**/*.go ${WEB_DEST}/*
	GOOS=darwin GOARCH=amd64 go build -ldflags="${LD_FLAGS}" -o $@

docs/api: api/openapi-spec/v1.openapi.yaml
	openapi-generator generate -i $? -g markdown -o $@

${WEB_DEST}/*: ${WEB_SRC}/*.ts ${WEB_SRC}/**/*.ts web/app/node_modules
	cd web/app && npx esbuild ${ESBUILD_FLAGS} --minify ./src/main.ts --outdir=../static/dist

clean:
	rm -rf build web/static/dist

test: web/app/node_modules
	go test -v ./...
	cd web/app && npm run lint && npm t

web/app/node_modules: web/app/package.json
	cd web/app && npm ci

pi: build/linux-arm/gesha
	scp build/linux-arm/gesha Silvia:~

dev:
	cd web/app && npx esbuild ./src/main.ts --servedir=../static --outdir=../static/dist --define:window.__API_BASE__='"http://silvia.local"' ${ESBUILD_FLAGS}

format:
	cd web/app && npm run format

.PHONY: all clean test pi
