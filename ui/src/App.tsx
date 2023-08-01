import { OnMessageCallback, connect } from "mqtt"
import { Show, createEffect, createSignal, onCleanup } from "solid-js"

import { Datum, Millis, RingBuffer, formatHeat, getHistory } from "./util"
import { Chart } from "./Chart"

import styles from "./App.module.css"
import { ResizeContainer } from "./ResizeContainer"
import {
    TIME_WINDOW_60M,
    TIME_WINDOW_30M,
    TIME_WINDOW_10M,
    TIME_WINDOW_5M,
    TIME_WINDOW_1M,
} from "./types"

const RING_BUFFER_SIZE = 1_000

function App() {
    const [boilerTemperatures, setBoilerTemperatures] = createSignal<
        RingBuffer<Datum>
    >(new RingBuffer(RING_BUFFER_SIZE), { equals: false })
    const [groupheadTemperatures, setGroupheadTemperatures] = createSignal<
        RingBuffer<Datum>
    >(new RingBuffer(RING_BUFFER_SIZE), { equals: false })
    const [thermofilterTemperatures, setThermofilterTemperatures] =
        createSignal<RingBuffer<Datum>>(new RingBuffer(RING_BUFFER_SIZE), {
            equals: false,
        })
    const [boilerLevels, setBoilerLevels] = createSignal<RingBuffer<Datum>>(
        new RingBuffer(RING_BUFFER_SIZE),
        { equals: false },
    )
    const [mode, setMode] = createSignal("")
    const [targetTemp, setTargetTemp] = createSignal<number>(-1000)
    const [timeWindow, setTimeWindow] = createSignal<Millis>(TIME_WINDOW_30M)
    const [controlMethod, setControlMethod] = createSignal("")

    const client = connect(
        "ws://luke:5s9zcBneIiIgETZ0FXLKw0frf6GrjrukPIZdYbQc@silvia.iot:8080",
    )
    client.subscribe("gesha/#")
    client.subscribe("ms-silvia-switch/switch/power/state")

    createEffect(() => {
        let lastT: number = boilerLevels().last?.x ?? Date.now() - 5_000

        const callback: OnMessageCallback = (topic, msg) => {
            let value = msg.toString()

            try {
                value = JSON.parse(value)
            } catch {}

            switch (topic) {
                case "gesha/temperature/last_updated": {
                    lastT = +value
                    break
                }
                case "gesha/temperature/boiler": {
                    setBoilerTemperatures(
                        boilerTemperatures().push({ x: lastT, y: +value }),
                    )
                    break
                }
                case "gesha/temperature/grouphead": {
                    setGroupheadTemperatures(
                        groupheadTemperatures().push({ x: lastT, y: +value }),
                    )
                    break
                }
                case "gesha/temperature/thermofilter": {
                    setThermofilterTemperatures(
                        thermofilterTemperatures().push({
                            x: lastT,
                            y: +value,
                        }),
                    )
                    break
                }
                case "gesha/temperature/target": {
                    setTargetTemp(+value)
                    break
                }
                case "gesha/boiler_level": {
                    setBoilerLevels(
                        boilerLevels().push({ x: lastT, y: +value }),
                    )
                    break
                }
                case "gesha/mode": {
                    setMode(value)
                    break
                }
                case "gesha/control_method": {
                    setControlMethod(value)
                    break
                }
            }
        }

        client.on("message", callback)

        onCleanup(() => client.off("message", callback))
    })

    const handleModeChange = (
        event: Event & {
            currentTarget: HTMLSelectElement
            target: HTMLSelectElement
        },
    ) => {
        client.publish("gesha/mode/set", event.target.value, { retain: false })
        event.preventDefault()
    }

    const handleTargetTempChange = (
        event: Event & {
            currentTarget: HTMLInputElement
            target: HTMLInputElement
        },
    ) => {
        client.publish("gesha/temperature/target/set", event.target.value, {
            retain: false,
        })
        event.preventDefault()
    }

    const handleControlMethodChange = (
        event: Event & {
            currentTarget: HTMLSelectElement
            target: HTMLSelectElement
        },
    ) => {
        client.publish("gesha/control_method/set", event.target.value, {
            retain: false,
        })
        event.preventDefault()
    }

    const handleHeatLevelChange = (
        event: Event & { target: HTMLInputElement },
    ) => {
        client.publish("gesha/boiler_level/set", event.target.value, {
            retain: false,
        })
    }

    const handleRetainedWindowSizeChange = async (newTimeWindow: number) => {
        setTimeWindow(newTimeWindow)
        const to = Date.now()
        const from = to - newTimeWindow

        const history = await getHistory(client, {
            from,
            to,
            bucketSize: 5_000,
        })

        setBoilerTemperatures(boilerTemperatures().load(history.boilerTemp))
        setGroupheadTemperatures(
            groupheadTemperatures().load(history.groupheadTemp),
        )
        setThermofilterTemperatures(
            thermofilterTemperatures().load(history.thermofilterTemp),
        )
        setBoilerLevels(boilerLevels().load(history.boilerLevel))
    }

    handleRetainedWindowSizeChange(timeWindow())

    return (
        <main class={styles.app}>
            <form class={styles.controls} onSubmit={(e) => e.preventDefault()}>
                <select value={mode()} onChange={handleModeChange}>
                    <option value="idle">Idle</option>
                    <option value="active">Active</option>
                    <option value="brew">Brew</option>
                    <option value="steam">Steam</option>
                </select>
                <p>Heat: {formatHeat(boilerLevels().last?.y)}</p>|
                <label>
                    Target temp:{" "}
                    <input
                        type="number"
                        value={targetTemp()}
                        step={0.5}
                        style={{
                            width: "50px",
                            appearance: "none",
                            background: "transparent",
                            border: "none",
                            "font-weight": "bold",
                        }}
                        onChange={handleTargetTempChange}
                    />{" "}
                    &deg;C
                </label>
                |
                <label>
                    Control Method
                    <select
                        value={controlMethod()}
                        onChange={handleControlMethodChange}
                    >
                        <option value="None">Manual</option>
                        <option value="Threshold">Threshold</option>
                        <option value="PID">PID</option>
                        <option value="MPC">MPC</option>
                    </select>
                </label>
                <Show when={controlMethod() == "None"}>
                    <>
                        |
                        <label>
                            Heat level{" "}
                            <input
                                type="range"
                                min="0"
                                max="1"
                                step="0.1"
                                onChange={handleHeatLevelChange}
                                value={boilerLevels().last?.y}
                            />
                            <span>{boilerLevels().last?.y}</span>
                        </label>
                    </>
                </Show>
                |
                <label>
                    Time window
                    <select
                        value={timeWindow()}
                        onChange={(e) =>
                            handleRetainedWindowSizeChange(+e.target.value)
                        }
                    >
                        <option value={TIME_WINDOW_60M}>1h</option>
                        <option value={TIME_WINDOW_30M}>30m</option>
                        <option value={TIME_WINDOW_10M}>10m</option>
                        <option value={TIME_WINDOW_5M}>5m</option>
                        <option value={TIME_WINDOW_1M}>1m</option>
                    </select>
                </label>
                |
                <p>
                    {"Last measurement: "}
                    {(() => {
                        let d = boilerTemperatures().last?.x
                        if (d) {
                            return new Date(d).toLocaleTimeString()
                        } else {
                            return "error"
                        }
                    })()}
                </p>
            </form>
            <ResizeContainer class={styles.chart}>
                {(width, height) => (
                    <Chart
                        boilerTempSeries={boilerTemperatures}
                        groupheadTempSeries={groupheadTemperatures}
                        thermofilterTempSeries={thermofilterTemperatures}
                        heatSeries={boilerLevels}
                        targetTemp={targetTemp}
                        width={width}
                        height={height}
                        timeWindow={timeWindow}
                    />
                )}
            </ResizeContainer>
        </main>
    )
}

export default App
