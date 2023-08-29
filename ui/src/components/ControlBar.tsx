import {
    Accessor,
    For,
    Match,
    Show,
    Switch,
    createEffect,
    createSignal,
} from "solid-js"
import styles from "./ControlBar.module.css"
import { TimeWindow } from "../types"
import { formatHeat } from "../util"
import { ControlMethod, Mode, ValueChange } from "../geshaClient"

export interface ControlBarProps {
    mode: Accessor<Mode | undefined>
    controlMethod: Accessor<ControlMethod | undefined>
    onControlMethodChange: (controlMethod: ControlMethod) => void
    currentBoilerLevel: Accessor<ValueChange | undefined>
    onHeatLevelChange: (heatLevel: number) => void
    onModeChange: (mode: Mode) => void
    currentTargetTemperature: Accessor<number | undefined>
    timeWindow: Accessor<number>
    onRetainedWindowSizeChange: (windowSize: TimeWindow) => void
    currentBoilerTemperature: Accessor<ValueChange | undefined>
    onTargetTempChange: (targetTemp: number) => void
    isLoadingHistory: Accessor<boolean>
    onShotToggle: () => void
    shotStartTime: Accessor<number | null>
}

export function ControlBar(_: ControlBarProps) {
    const [shotTimer, setShotTimer] = createSignal<null | string>(null)

    let shotTimerInterval: number | null = null

    createEffect(() => {
        const brewStartTime = _.shotStartTime()

        if (brewStartTime === null) {
            if (shotTimerInterval) {
                window.clearInterval(shotTimerInterval)
                shotTimerInterval = null
            }
            setShotTimer(null)
        } else {
            shotTimerInterval = window.setInterval(() => {
                const t = Date.now() - brewStartTime
                const s = Math.floor(t / 1000)
                const ms = Math.floor((t % 1000) / 10)
                setShotTimer(
                    `${s.toString().padStart(2, "0")}:${ms
                        .toString()
                        .padStart(2, "0")}`,
                )
            }, 50)
        }
    })

    return (
        <div class={styles.container}>
            <form
                class={styles.controlBar}
                onSubmit={(e) => e.preventDefault()}
            >
                <label class={styles.verticalLabel} data-value={_.mode() ?? "unknown"}>
                    <span class={styles.verticalLabelKey}>Mode</span>
                    <select
                        tabIndex={-1}
                        value={_.mode()}
                        onChange={(e) => {
                            _.onModeChange(e.target.value as Mode)
                            e.preventDefault()
                        }}
                    >
                        <option disabled value="offline">
                            Offline
                        </option>
                        <option value="idle">Idle</option>
                        <option value="active">Active</option>
                        <option
                            value="brew"
                            disabled={_.mode() !== "active"}
                        >
                            Brew
                        </option>
                        <option value="steam">Steam</option>
                    </select>
                </label>
                <label class={styles.verticalLabel}>
                    <span class={styles.verticalLabelKey}>Control</span>
                    <select
                        tabIndex={-1}
                        value={_.controlMethod()}
                        onChange={(e) => {
                            _.onControlMethodChange(
                                e.target.value as ControlMethod,
                            )
                            e.preventDefault()
                        }}
                    >
                        <option value="None">Manual</option>
                        <option value="Threshold">Threshold</option>
                        <option value="PID">PID</option>
                        <option value="MPC">MPC</option>
                    </select>
                </label>
                <label class={styles.verticalLabel}>
                    <span class={styles.verticalLabelKey}>Time window</span>
                    <select
                        tabIndex={-1}
                        value={_.timeWindow()}
                        onChange={(e) => {
                            _.onRetainedWindowSizeChange(+e.target.value)
                            e.preventDefault()
                        }}
                    >
                        <For
                            each={Object.values(TimeWindow).filter(
                                (value) => typeof value === "number",
                            )}
                        >
                            {(value) => (
                                <option value={value}>
                                    <Switch>
                                        <Match
                                            when={value === TimeWindow.OneHour}
                                        >
                                            1 hour
                                        </Match>
                                        <Match
                                            when={
                                                value ===
                                                TimeWindow.ThirtyMinutes
                                            }
                                        >
                                            30 minutes
                                        </Match>
                                        <Match
                                            when={
                                                value === TimeWindow.TenMinutes
                                            }
                                        >
                                            10 minutes
                                        </Match>
                                        <Match
                                            when={
                                                value === TimeWindow.FiveMinutes
                                            }
                                        >
                                            5 minutes
                                        </Match>
                                        <Match
                                            when={
                                                value === TimeWindow.OneMinute
                                            }
                                        >
                                            1 minute
                                        </Match>
                                    </Switch>
                                </option>
                            )}
                        </For>
                    </select>
                </label>
                <label class={styles.verticalLabel}>
                    <span class={styles.verticalLabelKey}>Target temp</span>
                    <span class={styles.verticalLabelValue}>
                        <input
                            tabIndex={-1}
                            type="number"
                            value={_.currentTargetTemperature()}
                            step={1}
                            style={{
                                width: "50px",
                                appearance: "none",
                                background: "transparent",
                                border: "none",
                                "font-weight": "bold",
                            }}
                            onChange={(e) => {
                                _.onTargetTempChange(+e.target.value)
                                e.preventDefault()
                            }}
                        />
                    </span>
                </label>
                <Show when={_.controlMethod() == "None"}>
                    <label class={styles.verticalLabel}>
                        <span class={styles.verticalLabelKey}>Heat level</span>
                        <input
                            tabIndex={-1}
                            disabled={_.mode() === "idle"}
                            type="range"
                            min="0"
                            max="1"
                            step="0.1"
                            onChange={(e) => {
                                _.onHeatLevelChange(+e.target.value)
                                e.preventDefault()
                            }}
                            value={_.currentBoilerLevel()?.value}
                        />
                    </label>
                </Show>
                <label class={styles.verticalLabel}>
                    <span class={styles.verticalLabelKey}>Heat</span>
                    <span class={styles.verticalLabelValue}>
                        {formatHeat(_.currentBoilerLevel()?.value)}
                    </span>
                </label>
                <button
                    class={styles.brewButton}
                    disabled={_.mode() !== "active" && _.mode() !== "brew"}
                    onClick={_.onShotToggle}
                    type="button"
                >
                    {shotTimer() ?? "Brew"}
                </button>
            </form>
            <div class={styles.progressBarContainer}>
                <Show when={_.isLoadingHistory()}>
                    <div class={styles.progressBar} />
                </Show>
            </div>
        </div>
    )
}
