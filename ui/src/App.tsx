import { OnMessageCallback, connect } from "mqtt"
import { createEffect, createSignal, onCleanup } from "solid-js"

import { Datum, Millis, RingBuffer, getHistory } from "./util"
import { Chart } from "./components/Chart"

import styles from "./App.module.css"
import { ResizeContainer } from "./components/ResizeContainer"
import { ControlMethod, Mode, TimeWindow } from "./types"
import { ControlBar } from "./components/ControlBar"
import { ShotTimer } from "./components/ShotTimer"
import { Dialog } from "./components/Dialog"

const BUFFER_SIZE = 3_000

const {
    GESHA_MQTT_HOST,
    GESHA_MQTT_USER,
    GESHA_MQTT_PASS,
    GESHA_MQTT_WS_PORT,
} = import.meta.env

function App() {
    const [boilerTemperatures, setBoilerTemperatures] = createSignal<
        RingBuffer<Datum>
    >(new RingBuffer(BUFFER_SIZE), { equals: false })
    const [groupheadTemperatures, setGroupheadTemperatures] = createSignal<
        RingBuffer<Datum>
    >(new RingBuffer(BUFFER_SIZE), { equals: false })
    const [thermofilterTemperatures, setThermofilterTemperatures] =
        createSignal<RingBuffer<Datum>>(new RingBuffer(BUFFER_SIZE), {
            equals: false,
        })
    const [boilerLevels, setBoilerLevels] = createSignal<RingBuffer<Datum>>(
        new RingBuffer(BUFFER_SIZE),
        { equals: false },
    )
    const [mode, setMode] = createSignal<Mode>("idle")
    const [targetTemp, setTargetTemp] = createSignal<number>(-1000)
    const [timeWindow, setTimeWindow] = createSignal<Millis>(
        TimeWindow.TenMinutes,
    )
    const [controlMethod, setControlMethod] =
        createSignal<ControlMethod>("None")

    const [isLoadingHistory, setIsLoadingHistory] = createSignal<boolean>(false)

    const client = connect(
        `ws://${GESHA_MQTT_USER}:${GESHA_MQTT_PASS}@${GESHA_MQTT_HOST}:${GESHA_MQTT_WS_PORT}`,
    )

    client.subscribe("gesha/#")
    client.subscribe("ms-silvia-switch/switch/power/state")

    createEffect(() => {
        let lastT: number

        const callback: OnMessageCallback = (topic, msg) => {
            let value = msg.toString()

            try {
                value = JSON.parse(value)
            } catch {}

            if (topic === "gesha/temperature/last_updated") {
                lastT = +value
                return
            }

            switch (topic) {
                case "gesha/mode": {
                    setMode(value as Mode)
                    break
                }
                case "gesha/control_method": {
                    setControlMethod(value as ControlMethod)
                    break
                }
                case "gesha/temperature/target": {
                    setTargetTemp(+value)
                    break
                }
            }

            if (typeof lastT !== "number") {
                return
            }

            switch (topic) {
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
                case "gesha/boiler_level": {
                    setBoilerLevels(
                        boilerLevels().push({ x: lastT, y: +value }),
                    )
                    break
                }
            }
        }

        client.on("message", callback)

        onCleanup(() => client.off("message", callback))
    })

    const handleRetainedWindowSizeChange = async (newTimeWindow: number) => {
        setTimeWindow(newTimeWindow)
        setIsLoadingHistory(true)
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

        setIsLoadingHistory(false)
    }

    handleRetainedWindowSizeChange(TimeWindow.TenMinutes)

    const [shotStartTime, setShotStartTime] = createSignal<number | null>(null)

    return (
        <main class={styles.app}>
            <ControlBar
                mode={mode}
                controlMethod={controlMethod}
                boilerLevels={boilerLevels}
                targetTemp={targetTemp}
                timeWindow={timeWindow}
                boilerTemperatures={boilerTemperatures}
                isLoadingHistory={isLoadingHistory}
                onControlMethodChange={(controlMethod: ControlMethod) => {
                    client.publish("gesha/control_method/set", controlMethod, {
                        retain: false,
                    })
                }}
                onHeatLevelChange={(heatLevel: number) => {
                    client.publish(
                        "gesha/boiler_level/set",
                        heatLevel.toString(),
                        {
                            retain: false,
                        },
                    )
                }}
                onModeChange={(mode: Mode) => {
                    client.publish("gesha/mode/set", mode, { retain: false })
                }}
                onRetainedWindowSizeChange={handleRetainedWindowSizeChange}
                onTargetTempChange={(targetTemp: number) => {
                    client.publish(
                        "gesha/temperature/target/set",
                        targetTemp.toString(),
                        {
                            retain: false,
                        },
                    )
                }}
                onShotToggle={async () => {
                    const newMode = mode() === "active" ? "brew" : "active"

                    console.log(newMode)

                    await client.publishAsync("gesha/mode/set", newMode, {
                        retain: false,
                    })
                    if (newMode === "brew") {
                        setShotStartTime(Date.now())
                    } else {
                        setShotStartTime(null)
                    }
                }}
                shotStartTime={shotStartTime}
            />
            <ResizeContainer class={styles.chart}>
                {(width, height) => (
                    <Chart
                        boilerTemperatures={boilerTemperatures}
                        groupheadTemperatures={groupheadTemperatures}
                        thermofilterTemperatures={thermofilterTemperatures}
                        boilerLevels={boilerLevels}
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
