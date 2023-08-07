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
import { ControlMethod, Mode, TimeWindow } from "../types"
import { Datum, RingBuffer, formatHeat } from "../util"

export interface ControlBarProps {
    mode: Accessor<Mode>
    controlMethod: Accessor<ControlMethod>
    onControlMethodChange: (controlMethod: ControlMethod) => void
    boilerLevels: Accessor<RingBuffer<Datum>>
    onHeatLevelChange: (heatLevel: number) => void
    onModeChange: (mode: Mode) => void
    targetTemp: Accessor<number>
    timeWindow: Accessor<number>
    onRetainedWindowSizeChange: (windowSize: TimeWindow) => void
    boilerTemperatures: Accessor<RingBuffer<Datum>>
    onTargetTempChange: (targetTemp: number) => void
    isLoadingHistory: Accessor<boolean>
    onShotToggle: () => void
    shotStartTime: Accessor<number | null>
}

export function ControlBar(props: ControlBarProps) {
    const [shotTimer, setShotTimer] = createSignal<null | string>(null)

    let shotTimerInterval: NodeJS.Timer | null = null

    createEffect(() => {
        const brewStartTime = props.shotStartTime()

        console.log(brewStartTime)

        if (brewStartTime === null) {
            if (shotTimerInterval) {
                clearInterval(shotTimerInterval)
                shotTimerInterval = null
            }
            setShotTimer(null)
        } else {
            shotTimerInterval = setInterval(() => {
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
        <form class={styles.container} onSubmit={(e) => e.preventDefault()}>
            <label class={styles.verticalLabel}>
                <span class={styles.verticalLabelKey}>Mode</span>
                <select
                    value={props.mode()}
                    onChange={(e) => props.onModeChange(e.target.value as Mode)}
                >
                    <option disabled value="offline">
                        Offline
                    </option>
                    <option value="idle">Idle</option>
                    <option value="active">Active</option>
                    <option value="brew">Brew</option>
                    <option value="steam">Steam</option>
                </select>
            </label>
            <label class={styles.verticalLabel}>
                <span class={styles.verticalLabelKey}>Control</span>
                <select
                    value={props.controlMethod()}
                    onChange={(e) =>
                        props.onControlMethodChange(
                            e.target.value as ControlMethod,
                        )
                    }
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
                    value={props.timeWindow()}
                    onChange={(e) =>
                        props.onRetainedWindowSizeChange(+e.target.value)
                    }
                >
                    <For
                        each={Object.values(TimeWindow).filter(
                            (value) => typeof value === "number",
                        )}
                    >
                        {(value) => (
                            <option value={value}>
                                <Switch>
                                    <Match when={value === TimeWindow.OneHour}>
                                        1 hour
                                    </Match>
                                    <Match
                                        when={
                                            value === TimeWindow.ThirtyMinutes
                                        }
                                    >
                                        30 minutes
                                    </Match>
                                    <Match
                                        when={value === TimeWindow.TenMinutes}
                                    >
                                        10 minutes
                                    </Match>
                                    <Match
                                        when={value === TimeWindow.FiveMinutes}
                                    >
                                        5 minutes
                                    </Match>
                                    <Match
                                        when={value === TimeWindow.OneMinute}
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
                        type="number"
                        value={props.targetTemp()}
                        step={0.5}
                        style={{
                            width: "50px",
                            appearance: "none",
                            background: "transparent",
                            border: "none",
                            "font-weight": "bold",
                        }}
                        onChange={(e) => {
                            props.onTargetTempChange(+e.target.value)
                            e.preventDefault()
                        }}
                    />
                </span>
            </label>
            <Show when={props.controlMethod() == "None"}>
                <label class={styles.verticalLabel}>
                    <span class={styles.verticalLabelKey}>Heat level</span>
                    <input
                        disabled={props.mode() === "idle"}
                        type="range"
                        min="0"
                        max="1"
                        step="0.1"
                        onChange={(e) => {
                            props.onHeatLevelChange(+e.target.value)
                            e.preventDefault()
                        }}
                        value={props.boilerLevels().last?.y}
                    />
                </label>
            </Show>
            <label class={styles.verticalLabel}>
                <span class={styles.verticalLabelKey}>Heat</span>
                <span class={styles.verticalLabelValue}>
                    {formatHeat(props.boilerLevels().last?.y)}
                </span>
            </label>
            <Show when={props.isLoadingHistory()}>
                <label
                    class={styles.verticalLabel}
                    style={{ "margin-left": "auto" }}
                >
                    <span class={styles.verticalLabelValue}>
                        <progress />
                    </span>
                </label>
            </Show>
            <p
                class={
                    (props.boilerTemperatures().last?.x ?? 0) - Date.now() >
                    2_000
                        ? styles.streamStateOffline
                        : styles.streamStateOnline
                }
            >
                <span class={styles.verticalLabelKey}>Last measurement</span>
                <span class={styles.verticalLabelValue}>
                    {(() => {
                        let d = props.boilerTemperatures().last?.x
                        if (d) {
                            return new Date(d).toLocaleTimeString(
                                navigator.language,
                                {
                                    hour: "numeric",
                                    minute: "numeric",
                                    second: "numeric",
                                    fractionalSecondDigits: true,
                                } as Intl.DateTimeFormatOptions,
                            )
                        } else {
                            return "error"
                        }
                    })()}
                </span>
            </p>
            <button class={styles.brewButton} onClick={props.onShotToggle}>
                {shotTimer() ?? "Brew"}
            </button>
        </form>
    )
}
