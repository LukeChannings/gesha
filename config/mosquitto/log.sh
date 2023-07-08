#!/usr/bin/env bash

ssh silvia "journalctl -u mosquitto.service -f"
