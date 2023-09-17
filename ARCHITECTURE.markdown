# Architecture

## MQTT API

```
gesha
  /mode=idle|heating|manual
    /set
  /power="on"|"off"
    /set
  /target_temperature=double
    /set
  /controller="threshold" | "mpc" | "none" | { p: int, i: int, d: int}
    /set
  /temperature/
    /boiler
    /grouphead
    /basket
    /basket_predicted
  /preheated=bool
  /shot_history={ start_time, end_time, duration, temperatures }[]
```

## Modes

### Idle mode

- The machine is powered off
- boiler is off
- Temperature is reported every 5 seconds
- Temperature buffer for the last 5 seconds is maintained

### Heating mode

- The machine is powered on
- The boiler is toggled based on the target temperature and the controller
- The temperature is reported every 0.5 seconds

### Manual mode

- The machine power state is toggleable
- The heating state is toggleable
- The temperature is reported every 80ms (median)

## Experiments / Models

- Temperature loss when pre-heated vs not, as measured from the basket
- Predict when a shot is being pulled based on temperature drop
- Predict basket temperature from boiler and grouphead temps

