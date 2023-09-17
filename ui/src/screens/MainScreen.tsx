import { createEffect, createSignal } from "solid-js"
import { Chart } from "../components/Chart"
import { ControlBar } from "../components/ControlBar"
import { GeshaClient } from "../geshaClient"
import { TimeWindow } from "../types"

import { last } from "../util"

export interface MainScreenProps {
    client: GeshaClient
}

export const MainScreen = (_: MainScreenProps) => {
    const boilerTemperatures = _.client.createValueChangeListSignal(
        "temperature/boiler",
        "temperature/boiler/history",
    )

    const groupheadTemperatures = _.client.createValueChangeListSignal(
        "temperature/grouphead",
        "temperature/grouphead/history",
    )

    const thermofilterTemperatures = _.client.createValueChangeListSignal(
        "temperature/thermofilter",
        "temperature/thermofilter/history",
    )

    const predictedThermofilterTemperatures = _.client.createValueChangeListSignal(
        "temperature/thermofilter_predicted"
    )

    const boilerLevels = _.client.createValueChangeListSignal(
        "boiler_level",
        "boiler_level/history",
    )
    const mode = _.client.createSignal("mode")
    const targetTemp = _.client.createSignal("temperature/target")
    const controlMethod = _.client.createSignal("control_method")

    const timeWindow = _.client.createSignalWithDefault(
        "config/ui_time_window",
        TimeWindow.TenMinutes,
    )

    const [isLoadingHistory, setIsLoadingHistory] = createSignal<boolean>(false)

    createEffect(async () => {
        const newTimeWindow = timeWindow()

        setIsLoadingHistory(true)

        const to = Date.now()
        const from = to - newTimeWindow

        await _.client.populateMeasurementHistory({
            from,
            to,
            bucketSize: 2_000,
        })

        setIsLoadingHistory(false)
    })

    const [shotStartTime, setShotStartTime] = createSignal<number | null>(null)

    const latestBoilerTemperature = () => last(boilerTemperatures())
    const latestBoilerLevel = () => last(boilerLevels())

    return (
        <>
            <ControlBar
                mode={mode}
                controlMethod={controlMethod}
                currentBoilerLevel={latestBoilerLevel}
                currentTargetTemperature={targetTemp}
                timeWindow={timeWindow}
                currentBoilerTemperature={latestBoilerTemperature}
                isLoadingHistory={isLoadingHistory}
                onControlMethodChange={_.client.setControlMethod}
                onHeatLevelChange={_.client.setBoilerLevel}
                onModeChange={_.client.setMode}
                onRetainedWindowSizeChange={async (newTimeWindow) => {
                    await _.client.setConfig(
                        "ui_time_window",
                        String(newTimeWindow),
                    )
                }}
                onTargetTempChange={_.client.setTargetTemperature}
                onShotToggle={async () => {
                    const newMode = mode() === "active" ? "brew" : "active"

                    await _.client.setMode(newMode)

                    if (newMode === "brew") {
                        setShotStartTime(Date.now())
                    } else {
                        setShotStartTime(null)
                    }
                }}
                shotStartTime={shotStartTime}
            />
            <Chart
                boilerTemperatures={boilerTemperatures}
                groupheadTemperatures={groupheadTemperatures}
                thermofilterTemperatures={thermofilterTemperatures}
                predictedThermofilterTemperatures={predictedThermofilterTemperatures}
                boilerLevels={boilerLevels}
                targetTemp={targetTemp}
                timeWindow={timeWindow}
            />
        </>
    )
}
