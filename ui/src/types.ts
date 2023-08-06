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

export type Mode = "idle" | "active" | "brew" | "steam"

export type ControlMethod = "None" | "Threshold" | "PID" | "MPC"
