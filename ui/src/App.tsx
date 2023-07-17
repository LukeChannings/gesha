import { connect } from "mqtt"
import { createEffect, createSignal } from "solid-js"

import { Datum, Series, last, updateSeries } from "./util"
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
    const [heatSeries, setHeatSeries] = createSignal<Series<boolean>>([])
    const [mode, setMode] = createSignal("")

    // We'll keep 30 minutes of data, and only show 5 minutes
    const retainedWindowMs = 30 * 60 * 1_000
    const timeWindowMs = 10 * 60 * 1_000

    const client = connect(
        "ws://luke:5s9zcBneIiIgETZ0FXLKw0frf6GrjrukPIZdYbQc@silvia.iot:8080",
    )
    client.subscribe("gesha/#")
    client.subscribe("ms-silvia-switch/switch/power/state")

    createEffect(() => {
        let lastT: number = Date.now() - 250
        let heat = false

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
                case "gesha/boiler_status": {
                    heat = value === "on"
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
        client.publish("gesha/mode/set", event.target.value)
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
            )
        })

    getHistory(Date.now() - 11 * 60 * 1000, Date.now()).then((measurements) => {
        const allSeries = measurements.sort((a, b) => a.time - b.time).map(
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
                        y: measurement.heat,
                    },
                ] as const,
        )

        setBoilerTempSeries(allSeries.map(([v]) => v))
        setGroupheadTempSeries(allSeries.map(([, v]) => v))
        setThermofilterTempSeries(allSeries.map(([, , v]) => v))
        setHeatSeries(allSeries.map(([, , , v]) => v))
    })

    return (
        <main class={styles.app}>
            <form class={styles.controls}>
                <select value={mode()} onChange={handleModeChange}>
                    <option value="idle">Idle</option>
                    <option value="heat">Heat</option>
                    <option value="brew">Brew</option>
                    <option value="steam">Steam</option>
                </select>
                <p>Heat: {last(heatSeries())?.y ? "on" : "off"}</p>
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
                        width={width}
                        height={height}
                        timeWindow={timeWindowMs}
                    />
                )}
            </ResizeContainer>
        </main>
    )
}

export default App
