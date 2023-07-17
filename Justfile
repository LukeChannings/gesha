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

deploy: build
    ssh silvia.iot "sudo systemctl stop gesha"
    scp target/arm-unknown-linux-gnueabihf/release/gesha silvia.iot:/opt/gesha/bin/gesha
    ssh silvia.iot "sudo systemctl start gesha"

log service="gesha":
    ssh silvia.iot "journalctl -fu {{service}}.service"

install service:
    #!/usr/bin/env bash

    if [ "$service" == "gesha" ]; then
        ssh silvia.iot -t <<-EOF
        echo '$(cat ./config/systemd/gesha.service)' | sudo tee /etc/systemd/system/gesha.service > /dev/null
        sudo systemctl daemon-reload
        EOF
    fi

    if [ "$service" == "mosquitto" ]; then
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

sync db:
    scp silvia.iot:/opt/gesha/var/db/gesha.db .
