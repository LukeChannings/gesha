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

export const computeRects = (
    series: Series<boolean>,
): [x1: number, x2: number][] => {
    let rects: [x1: number, x2: number][] = []

    let rectStart;

    for (const { x, y } of series) {
        if (y && !rectStart) {
            rectStart = x
        } else if (!y && rectStart) {
            rects.push([rectStart, x]);
            rectStart = null;
        }
    }

    if (rectStart) {
        rects.push([rectStart, series[series.length - 1].x])
    }

    return rects
}
