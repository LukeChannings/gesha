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

export const TIME_WINDOW_1M = 60 * 1_000
export const TIME_WINDOW_5M = 5 * 60 * 1_000
export const TIME_WINDOW_10M = 10 * 60 * 1_000
export const TIME_WINDOW_30M = 30 * 60 * 1_000
export const TIME_WINDOW_60M = 60 * 60 * 1_000
