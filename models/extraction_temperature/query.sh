#!/usr/bin/env bash

# sqlite3 ../gesha.db -header -csv <query.sql >measurements.csv
ssh silvia.iot "sqlite3 /opt/gesha/var/db/gesha.db -header -csv" <query.sql >measurements.csv
