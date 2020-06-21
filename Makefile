all: clean linux-arm darwin

linux-arm:
	GOOS=linux GOARCH=arm GOARM=6 go build -o build/arm/gesha

darwin:
	GOOS=darwin go build -o build/darwin/gesha

clean:
	rm -rf build pkged.go

.PHONY: clean linux-arm darwin all
