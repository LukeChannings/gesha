export type Millis = number

export const formatMillis = (millis: Millis) => {
    const secs = millis / 1_000
    const mins = Math.floor(Math.abs(secs) / 60)
    const remainingSecs = Math.abs(secs % 60)

    return (
        (millis < 0 ? "-" : "") +
        (mins > 0
            ? `${mins}m ${remainingSecs > 0 ? remainingSecs + "s" : ""}`
            : `${remainingSecs}s`)
    )
}

export type Datum<Value = number> = { x: number; y: Value }
export type Series<Value = number> = Array<Datum<Value>>

export const updateSeries = <V = number>(
    series: Series<V>,
    d: Datum<V>,
    isExpired: (d: Datum<V>) => boolean,
) => {
    return [...series.filter(isExpired), d]
}

export const last = <T>(list: T[]): T | null =>
    list.length > 0 ? list[list.length - 1] : null

export const computeLineSegments = (series: Series): Map<number, number> => {
    let rects: Map<number, number> = new Map()

    let x1

    for (const { x, y } of series) {
        if (y > 0 && !x1) {
            x1 = x
        } else if (!y && x1) {
            rects.set(x1, x)
            x1 = null
        }
    }

    // If the rect is never closed
    if (x1) {
        rects.set(x1, series[series.length - 1].x)
    }

    return rects
}
