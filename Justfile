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
    else
        cargo build --release
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

export-diagrams:
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 0 -f png --width 1500 -o docs/diagrams/silvia-e-electrical-diagram.png docs/diagrams/silvia-e-diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 1 -f png --width 1500 -o docs/diagrams/silvia-shelly-electrical-diagram.png docs/diagrams/silvia-e-diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 2 -f png --width 1500 -o docs/diagrams/silvia-shelly-pi-electrical-diagram.png docs/diagrams/silvia-e-diagrams.drawio
    /Applications/draw.io.app/Contents/MacOS/draw.io --export -p 3 -f png --width 1500 -o docs/diagrams/pi-zero-pinout.png docs/diagrams/silvia-e-diagrams.drawio
