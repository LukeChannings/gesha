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
