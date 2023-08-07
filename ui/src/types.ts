export interface Measurement {
    time: number
    targetTempC: number
    boilerTempC: number
    groupheadTempC: number
    thermofilterTempC?: number
    power: boolean
    heatLevel: number
    pull: boolean
    steam: boolean
}

export enum TimeWindow {
    OneMinute = 60 * 1_000,
    FiveMinutes = 5 * 60 * 1_000,
    TenMinutes = 10 * 60 * 1_000,
    ThirtyMinutes = 30 * 60 * 1_000,
    OneHour = 60 * 60 * 1_000,
}

export function assertTimeWindow(value: unknown): asserts value is TimeWindow {
    if (!(typeof value === "number" && value in TimeWindow)) {
        throw new Error(`${value} is not a TimeWindow value!`)
    }
}

export type Mode = "offline" | "idle" | "active" | "brew" | "steam"

export type ControlMethod = "None" | "Threshold" | "PID" | "MPC"
