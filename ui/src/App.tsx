import { connect } from "mqtt"
import { createEffect, createSignal } from "solid-js"

import { Datum, Millis, Series, last, updateSeries } from "./util"
import { Chart } from "./Chart"

import styles from "./App.module.css"
import { ResizeContainer } from "./ResizeContainer"

function App() {
    const [boilerTempSeries, setBoilerTempSeries] = createSignal<Series>([])
    const [groupheadTempSeries, setGroupheadTempSeries] = createSignal<Series>(
        [],
    )
    const [thermofilterTempSeries, setThermofilterTempSeries] =
        createSignal<Series>([])
    const [heatSeries, setHeatSeries] = createSignal<Series>([])
    const [mode, setMode] = createSignal("")
    const [targetTemp, setTargetTemp] = createSignal<number>(-1000)
    const [timeWindow, setTimeWindow] = createSignal<Millis>(10 * 60 * 1_000);

    const retainedWindowMs = 30 * 60 * 1_000

    const client = connect(
        "ws://luke:5s9zcBneIiIgETZ0FXLKw0frf6GrjrukPIZdYbQc@silvia.iot:8080",
    )
    client.subscribe("gesha/#")
    client.subscribe("ms-silvia-switch/switch/power/state")

    createEffect(() => {
        let lastT: number = Date.now() - 250
        let heat: number = 0

        client.on("message", (topic, msg) => {
            let value = msg.toString()

            try {
                value = JSON.parse(value)
            } catch {}

            // expire any data older than the time window.
            const isExpired = (d: Datum<any>) =>
                d.x > Date.now() - retainedWindowMs

            switch (topic) {
                case "gesha/temperature/last_updated": {
                    lastT = +value
                    setHeatSeries(
                        updateSeries(
                            heatSeries(),
                            { x: lastT, y: heat },
                            isExpired,
                        ),
                    )
                    break
                }
                case "gesha/temperature/boiler": {
                    setBoilerTempSeries(
                        updateSeries(
                            boilerTempSeries(),
                            { x: lastT, y: +value },
                            isExpired,
                        ),
                    )
                    break
                }
                case "gesha/temperature/grouphead": {
                    setGroupheadTempSeries(
                        updateSeries(
                            groupheadTempSeries(),
                            { x: lastT, y: +value },
                            isExpired,
                        ),
                    )
                    break
                }
                case "gesha/temperature/thermofilter": {
                    setThermofilterTempSeries(
                        updateSeries(
                            thermofilterTempSeries(),
                            { x: lastT, y: +value },
                            isExpired,
                        ),
                    )
                    break
                }
                case "gesha/temperature/target": {
                    setTargetTemp(+value)
                    break
                }
                case "gesha/boiler_level": {
                    console.log(value)
                    heat = +value
                    break
                }
                case "gesha/mode": {
                    setMode(value)
                    break
                }
            }
        })
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

    const handleTargetTempChange = (event: Event & {
        currentTarget: HTMLInputElement
        target: HTMLInputElement
    }) => {
        client.publish("gesha/temperature/target", event.target.value);
        event.preventDefault()
    }

    const getHistory = (from: number, to: number): Promise<Measurement[]> =>
        new Promise((resolve) => {
            const callback = (topic: string, payload: Buffer) => {
                if (topic === "gesha/temperature/history") {
                    client.off("message", callback)

                    resolve(JSON.parse(payload.toString()))
                }
            }

            client.on("message", callback)

            client.publish(
                "gesha/temperature/history/command",
                JSON.stringify({ from, to }),
                { retain: false, qos: 2 }
            )
        })

    getHistory(Date.now() - retainedWindowMs, Date.now()).then((measurements) => {
        const allSeries = measurements
            .sort((a, b) => a.time - b.time)
            .map(
                (measurement) =>
                    [
                        {
                            x: measurement.time,
                            y: measurement.boilerTempC,
                        },
                        {
                            x: measurement.time,
                            y: measurement.groupheadTempC,
                        },
                        {
                            x: measurement.time,
                            y: measurement.thermofilterTempC ?? 0,
                        },
                        {
                            x: measurement.time,
                            y: measurement.heatLevel,
                        },
                    ] as const,
            )

        setBoilerTempSeries(allSeries.map(([v]) => v))
        setGroupheadTempSeries(allSeries.map(([, v]) => v))
        setThermofilterTempSeries(allSeries.map(([, , v]) => v))
        setHeatSeries(allSeries.map(([, , , v]) => v))
    })

    const formatHeat = (n?: number) => n === 0 || !n ? 'Off' : `${(n * 100).toFixed(0)}%`

    return (
        <main class={styles.app}>
            <form class={styles.controls} onSubmit={e => e.preventDefault()}>
                <select value={mode()} onChange={handleModeChange}>
                    <option value="idle">Idle</option>
                    <option value="active">Active</option>
                    <option value="brew">Brew</option>
                    <option value="steam">Steam</option>
                </select>
                <p>Heat: {formatHeat(last(heatSeries())?.y)}</p>
                |
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
                            "font-weight": "bold"
                        }}
                        onChange={handleTargetTempChange}
                    />{" "}
                    &deg;C
                </label>
                |
                <label>
                    Time window
                    <select value={timeWindow()} onChange={e => setTimeWindow(+e.target.value)}>
                        <option value={30 * 60 * 1_000}>30m</option>
                        <option value={10 * 60 * 1_000}>10m</option>
                        <option value={5 * 60 * 1_000}>5m</option>
                    </select>
                </label>
                |
                <p>
                    Lag:{" "}
                    {Math.max(
                        0,
                        Date.now() - (last(boilerTempSeries())?.x ?? 0),
                    )}
                    ms
                </p>
            </form>
            <ResizeContainer class={styles.chart}>
                {(width, height) => (
                    <Chart
                        boilerTempSeries={boilerTempSeries}
                        groupheadTempSeries={groupheadTempSeries}
                        thermofilterTempSeries={thermofilterTempSeries}
                        heatSeries={heatSeries}
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
