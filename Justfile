export RUST_BACKTRACE := "1"
export DATABASE_URL := "sqlite:gesha.db"

start thing="ui":
    #!/usr/bin/env bash

    if [[ "{{thing}}" == "tauri" ]]; then
        cargo tauri dev --target aarch64-apple-darwin
    elif [[ "{{thing}}" == "ui" ]]; then
        cd ui; npm start
    fi

build thing="app":
    #!/usr/bin/env bash

    if [[ "{{thing}}" == "tauri" ]]; then
        cargo tauri build --target aarch64-apple-darwin
    elif [[ "{{thing}}" == "ui" ]]; then
        cd ui; npm run build
    elif [[ "{{thing}}" == "db" ]]; then
        cargo sqlx prepare --database-url sqlite:gesha.db
    elif [[ "{{thing}}" == "thermofilter" ]]; then
        cargo build --release --bin thermofilter
    elif [[ "{{thing}}" == "report" ]]; then
        typst compile --root . ./docs/report/0-main.typ docs/Dissertation.pdf
    else
        cargo build --release --bin gesha
    fi

test thing="app":
    #!/usr/bin/env bash

    if [[ "{{thing}}" = "ui" ]]; then
        cd ui; npm test
    else
        cargo test --target aarch64-apple-darwin
    fi

format thing="ui":
    #!/usr/bin/env bash

    if [[ "{{thing}}" == "ui" ]]; then
        cd ui; npm run format
    else
        cargo fmt
    fi

deploy service="gesha": (build service)
    #!/usr/bin/env bash

    if [[ "{{service}}" == "gesha" ]]; then
        ssh silvia.iot "sudo systemctl stop gesha"
        scp target/arm-unknown-linux-gnueabihf/release/gesha silvia.iot:/opt/gesha/bin/gesha
        ssh silvia.iot "sudo systemctl start gesha"
    elif [[ "{{service}}" == "thermofilter" ]]; then
        scp target/arm-unknown-linux-gnueabihf/release/thermofilter aux-silvia.iot:~
    elif [[ "{{service}}" == "ui" ]]; then
        scp -r ui/dist/* silvia.iot:/opt/gesha/web/
    fi

log service="gesha":
    ssh silvia.iot "journalctl -fu {{service}}.service"

install service:
    #!/usr/bin/env bash

    if [[ "{{service}}" == "gesha" ]]; then
        ssh silvia.iot -t <<-EOF
        echo '$(cat ./config/systemd/gesha.service)' | sudo tee /etc/systemd/system/gesha.service > /dev/null
        sudo systemctl daemon-reload
        EOF
    fi

    if [[ "{{service}}" == "mosquitto" ]]; then
        #!/usr/bin/env bash
        ssh silvia.iot -t << EOF
            sudo apt install -y mosquitto
            sudo chown -R luke:sudo /etc/mosquitto /var/lib/mosquitto
        EOF

        rsync -av --usermap=luke:sudo ./config/raspberry_pi/etc/mosquitto/ silvia.iot:/etc/mosquitto/
    fi

init silvia:
    #!/usr/bin/env bash
    ssh silvia.iot -t << EOF
    sudo mkdir -p /opt/gesha/{etc,bin}
    sudo chown -R luke:sudo /opt/gesha
    EOF

download-db:
    scp silvia.iot:/opt/gesha/var/db/gesha.db .

export-diagrams:
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 0 -f svg -o docs/diagrams/silvia-e-electrical-diagram.svg docs/diagrams/diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 1 -f svg -o docs/diagrams/silvia-shelly-electrical-diagram.svg docs/diagrams/diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 2 -f svg -o docs/diagrams/silvia-shelly-pi-electrical-diagram.svg docs/diagrams/diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 3 -f svg -o docs/diagrams/pi-zero-pinout.svg docs/diagrams/diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 4 -f svg -o docs/diagrams/software-architecture.svg docs/diagrams/diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 5 -f svg -o docs/diagrams/software-event-example.svg docs/diagrams/diagrams.drawio
    sed -i '' -e 's|<text text-anchor="middle" font-size="10px" x="50%" y="100%">Text is not SVG - cannot display</text>||g' docs/diagrams/*.svg

model something *args:
    cd models; poetry run {{something}} -- {{args}}

api *args:
    cd models; poetry run api {{args}}
