# Gesha

![Tests](https://github.com/LukeChannings/gesha/workflows/Tests/badge.svg?branch=main)

Gesha is an app for your modded espresso machine. It integrates with a [MAX31855](https://www.adafruit.com/product/3328) and a relay to control brew temperature, and lets you track variables like dose, grind, ratio, etc. to make perfecting your espresso easier.

![Gesha in light and dark modes](web/app/screenshots/composed.png)

## Features

- [x] Finely control brew temperature with PID control
- [ ] Easily override the PID by sliding all the way to the bottom (off) or the top (on)
- [x] Keep a history of past shots to help you dial in
- [x] Nothing to install or configure with a zero dependency binary
- [x] Configurable with [a simple YAML file](https://github.com/LukeChannings/gesha/blob/main/configs/rancilio-silvia.yaml), or through the app's settings screen.
- [x] Written Go and TypeScript for a fast and finely tuned experience

## Installation

1. Download the latest [release](https://github.com/LukeChannings/gesha/releases) for your architecture
2. Move the download to your desired server and run `sudo ./gesha install`
3. Use `sudo systemctl enable gesha` to make sure Gesha runs on boot
4. `sudo systemctl daemon-reload` `sudo systemctl start gesha`

The install command will copy the binary into `/usr/local/`, install a systemd unit, and a default configuration file.

> If you do not have a distribution that uses systemd, you can run gesha directly with `./gesha start`.

## Developing the UI

The UI is written in HTML, CSS, and TypeScript. It doesn't use a web framework and relies on modern CSS and JS APIs.

### Browser support

- Safari latest
- FireFox latest
- Chrome latest

### Compiling

I wanted to try something new with the development toolchain, inspired by [Snowpack](https://www.snowpack.dev).

The development environment does not require Node.js or NPM to be installed, instead the UI is compiled with [Hammer](https://github.com/LukeChannings/hammer), an extremely simple HTTP server / compiler combo that uses [esbuild](https://github.com/evanw/esbuild).

Install hammer with `go get github.com/lukechannings/hammer/cmd/hammer`

To start developing, run `make dev` and load up [localhost:4321](http://localhost:4321).

The app should load instantly. As you edit, reload the browser to update.

## Supporting documents & prior works

- [silvia-pi](https://github.com/brycesub/silvia-pi)
- [Silvia PID manual](https://www.seattlecoffeegear.com/assets/files/silvia-pid-operation-manual.pdf)
- [PID without a PhD](https://m.eet.com/media/1112634/f-wescot.pdf)
- [PID for Dummies](https://www.csimn.com/CSI_pages/PIDforDummies.html)
- [Rancilio Silvia User Manual](https://www.ranciliogroupna.com/filebin/images/Downloadables/User_Manuals/Homeline/Silvia_User_Manual_2017.PDF) (vector electrical diagrams from page 32)
