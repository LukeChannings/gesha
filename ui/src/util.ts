export type Millis = number;

export const formatMillis = (millis: Millis) => {
    const secs = millis / 1_000;
    const mins = Math.floor(Math.abs(secs) / 60);
    const remainingSecs = Math.abs(secs % 60);

    return (
        (millis < 0 ? "-" : "") +
        (mins > 0 ? `${mins}m ${remainingSecs > 0 ? remainingSecs + "s" : ""}` : `${remainingSecs}s`)
    );
};

export type Datum = { x: number; y: number };
export type Series = Datum[];

export const updateSeries = (series: Series, d: Datum, isExpired: (d: Datum) => boolean) => {
    return [...series.filter(isExpired), d];
};

export const last = <T>(list: T[]): T | null => (list.length > 0 ? list[list.length - 1] : null);
