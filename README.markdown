# Gesha

Gesha is a PID app for your espresso machine that uses a [MAX31855](https://www.adafruit.com/product/3328) and a solid-state relay to monitor and control the temperature of your boiler.

> My specific use case is with a Rancilio Silvia, modified roughly according to [this excellent project](https://github.com/brycesub/silvia-pi), but Gesha will work with any espresso machine with similar modifications.

## Features

- [x] Fat binary with zero dependencies
- [ ] Builds for ARM64, ARM, and AMD64
- [x] Support for Internationalization
- [x] REST API, fully documented with the OpenAPI 3 standard
- [x] Real-time streaming of temperature and PID output using lightweight [Event Streams](https://html.spec.whatwg.org/multipage/iana.html#text/event-stream)
- [x] Fast and Accessible Web UI
- [x] Add to Home Screen

## Installation

1. Download the latest [release](https://github.com/LukeChannings/gesha/releases) for your architecture
2. Move the download to your desired server and run `sudo ./gesha install`
3. Use `systemctl enable gesha` to make sure Gesha runs on boot
4. `systemctl start gesha`

The install command will copy the binary into `/usr/local/`, install a systemd unit, and a default configuration file.

> If you do not have a distribution that uses systemd, you can run gesha directly with `./gesha start`.
