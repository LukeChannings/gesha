#!/usr/bin/env bash
ssh silvia.iot "sqlite3 /opt/gesha/var/db/gesha.db -header -csv" <query.sql >measurements.csv
