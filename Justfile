export RUST_BACKTRACE := "1"
export DATABASE_URL := "sqlite:gesha.db"

start thing="ui":
    #!/usr/bin/env bash

    if [[ "{{thing}}" == "app" ]]; then
        cargo tauri dev --target aarch64-apple-darwin
    elif [[ "{{thing}}" == "ui" ]]; then
        cd ui; npm start
    elif [[ "{{thing}}" == "gesha" ]]; then
        ssh silvia.iot -t "systemctl start gesha.service"
    fi

stop gesha:
    ssh silvia.iot -t "systemctl stop gesha.service"

restart gesha:
    ssh silvia.iot -t "systemctl restart gesha.service"

build thing="app":
    #!/usr/bin/env bash

    if [[ "{{thing}}" == "app" ]]; then
        cargo tauri build --target aarch64-apple-darwin
    elif [[ "{{thing}}" == "ui" ]]; then
        cd ui; npm run build
    elif [[ "{{thing}}" == "db" ]]; then
        cargo sqlx prepare --database-url sqlite:gesha.db
    elif [[ "{{thing}}" == "thermofilter" ]]; then
        cargo build --release --bin thermofilter
    elif [[ "{{thing}}" == "dissertation" ]]; then
        typst compile --root . ./docs/report/0-main.typ docs/dissertation-preprint-$(date '+%d-%m-%Y').pdf
    else
        cargo build --release --bin gesha
    fi

test thing="app":
    #!/usr/bin/env bash

    if [[ "{{thing}}" = "ui" ]]; then
        cd ui; npm test
    elif [[ "{{thing}}" = "models" ]]; then
        cd models;
        poetry run python -m unittest predictive.dataset_test
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
    elif [[ "{{service}}" == "app" ]]; then
        cp -R src-tauri/target/aarch64-apple-darwin/release/bundle/macos/Gesha.app /Applications/
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
    #!/usr/bin/env bash
    ssh silvia.iot "sudo systemctl stop gesha"
    scp silvia.iot:/opt/gesha/var/db/gesha.db "./gesha-$(date '+%d-%m-%Y').db"
    ssh silvia.iot "sudo systemctl start gesha"

export-diagrams:
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 0 -f svg -o docs/diagrams/silvia-e-electrical-diagram.svg docs/diagrams/diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 1 -f svg -o docs/diagrams/silvia-shelly-electrical-diagram.svg docs/diagrams/diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 2 -f svg -o docs/diagrams/silvia-shelly-pi-electrical-diagram.svg docs/diagrams/diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 3 -f svg -o docs/diagrams/pi-zero-pinout.svg docs/diagrams/diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 4 -f svg -o docs/diagrams/software-architecture.svg docs/diagrams/diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 5 -f svg -o docs/diagrams/software-event-example.svg docs/diagrams/diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 6 -f svg -o docs/diagrams/software-database-tables.svg docs/diagrams/diagrams.drawio
    sed -i '' -e 's|<text text-anchor="middle" font-size="10px" x="50%" y="100%">Text is not SVG - cannot display</text>||g' docs/diagrams/*.svg

model something *args:
    cd models; poetry run {{something}} -- {{args}}

api *args:
    cd models; poetry run api {{args}}
