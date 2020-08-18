TAGGED_VERSION = $(shell git tag --points-at HEAD)
GIT_HASH = $(shell git rev-parse --short HEAD)
LD_FLAGS = -s -w -X main.gitHash=${GIT_HASH} -X main.taggedVersion=${TAGGED_VERSION}
WEB_SRC = web/app/src
WEB_DEST = web/static/dist

all: clean pkged.go linux-386 linux-arm linux-arm64 linux-amd64 darwin compress

linux-arm:
	mkdir -p build/linux-arm
	GOOS=linux GOARCH=arm GOARM=6 go build -ldflags="${LD_FLAGS}" -o build/linux-arm/gesha

linux-arm64:
	mkdir -p build/linux-arm64
	GOOS=linux GOARCH=arm64 go build -ldflags="${LD_FLAGS}" -o build/linux-arm64/gesha

linux-amd64:
	mkdir -p build/linux-amd64
	GOOS=linux GOARCH=amd64 go build -ldflags="${LD_FLAGS}" -o build/linux-amd64/gesha

linux-386:
	mkdir -p build/linux-386
	GOOS=linux GOARCH=386 go build -ldflags="${LD_FLAGS}" -o build/linux-386/gesha
	
darwin:
	mkdir -p build/darwin
	GOOS=darwin GOARCH=amd64 go build -ldflags="${LD_FLAGS}" -o build/darwin/gesha

compress:
	upx --brute ./build/*/gesha

generate-docs:
	openapi-generator generate -i ./api/openapi-spec/v1.openapi.yaml -g markdown -o docs/api

pkged.go:
	pkger

clean: clean-web
	rm -rf build pkged.go

clean-web:
	rm -rf ${WEB_DEST}
	mkdir -p ${WEB_DEST}

pi: clean web pkged.go linux-arm
	scp ./build/linux-arm/gesha coffee-machine:~

test:
	go test -v ./...
	cd web/app ;\
		npm ci ;\
		npm run lint ;\
		npm t

web: clean-web ${WEB_DEST}/main.js ${WEB_DEST}/main.css

${WEB_DEST}/main.js: ${WEB_SRC}/main.ts
	esbuild --bundle --sourcemap=inline $? --outfile=$@

${WEB_DEST}/main.css: ${WEB_SRC}/*.css ${WEB_SRC}/**/*.css
	cat $? | grep -v '@import ' > $@

.PHONY: clean linux-arm darwin all generate-api generate-api-docs pi test web clean-web
