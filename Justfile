build:
    cargo build --target arm-unknown-linux-gnueabihf --release

deploy: build
    ssh silvia.iot "sudo systemctl stop gesha"
    scp target/arm-unknown-linux-gnueabihf/release/gesha silvia.iot:/opt/gesha/bin/gesha
    ssh silvia.iot "sudo systemctl start gesha"

log:
    ssh silvia.iot "journalctl -fu gesha.service"

log-mosquitto:
    ssh silvia.iot "journalctl -fu mosquitto.service"

install-systemd:
    #!/usr/bin/env bash

    ssh silvia.iot -t <<-EOF
    echo '$(cat ./config/systemd/gesha.service)' | sudo tee /etc/systemd/system/gesha.service > /dev/null
    sudo systemctl daemon-reload
    EOF

initialise-silvia:
    #!/usr/bin/env bash
    ssh silvia.iot -t << EOF
    sudo mkdir -p /opt/gesha/{etc,bin}
    sudo chown -R luke:sudo /opt/gesha
    EOF

install-mosquitto:
    #!/usr/bin/env bash
    ssh silvia.iot -t << EOF
        sudo apt install -y mosquitto
        sudo chown -R luke:sudo /etc/mosquitto /var/lib/mosquitto
    EOF

    rsync -av --usermap=luke:sudo ./config/raspberry_pi/etc/mosquitto/ silvia.iot:/etc/mosquitto/

ui-dev:
    cd ui; npm start

ui-prod:
    cd ui; npm run build

tauri-dev:
    cargo tauri dev --target aarch64-apple-darwin

tauri-build:
    cargo tauri build --target aarch64-apple-darwin
