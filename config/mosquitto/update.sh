#!/usr/bin/env bash

ssh silvia "sudo chown -R luke:sudo /etc/mosquitto /var/lib/mosquitto"
scp ./default.conf silvia:/etc/mosquitto/conf.d/
scp ./{acl,password} silvia:/etc/mosquitto/
ssh silvia "sudo systemctl restart mosquitto"
