export type Millis = number;

export const formatMillis = (millis: Millis) => {
    const secs = millis / 1_000;
    const mins = secs / 60;

    return Math.abs(mins) > 0 ? `${mins.toFixed(1)}m` : `${secs}s`
}

export type Datum = { x: number, y: number}
export type Series = Datum[];

export const updateSeries = (series: Series, d: Datum, isExpired: (d: Datum) => boolean) => {
    return [
        ...series.filter(isExpired),
        d,
    ]
}
