import { MqttClient } from "mqtt"
import { Measurement } from "./types"

export type Millis = number

export const formatMillis = (millis: Millis) => {
    const secs = millis / 1_000
    const mins = Math.floor(Math.abs(secs) / 60)
    const remainingSecs = Math.abs(secs % 60)

    return (
        (millis < 0 ? "-" : "") +
        (mins > 0
            ? `${mins}m ${
                  remainingSecs > 0 ? remainingSecs.toFixed(0) + "s" : ""
              }`
            : `${remainingSecs.toFixed(0)}s`)
    )
}

export type Datum<Value = number> = { x: number; y: Value }
export type Series<Value = number> = Array<Datum<Value>>

export class RingBuffer<T> {
    #values: Array<T>
    #size: number
    #length = 0
    #tail = 0
    #head = 0

    constructor(size: number) {
        this.#size = size
        this.#values = new Array(size)
    }

    get length() {
        return this.#length
    }

    get values(): T[] {
        const segment1 = this.#values.slice(0, this.#head)
        const segment2 = this.#values.slice(this.#head, this.#length)

        if (this.#head < this.#tail) {
            return [...segment1, ...segment2]
        } else {
            return [...segment2, ...segment1]
        }
    }

    get first(): T | undefined {
        if (this.#length < this.#size) {
            return this.#values[0]
        }

        return this.#values[this.#head]
    }

    get last(): T | undefined {
        return this.#values[this.#tail]
    }

    push(value: T): this {
        this.#values[this.#head] = value
        this.#tail = this.#head
        this.#head = (this.#head + 1) % this.#size

        if (this.#length < this.#size) {
            this.#length += 1
        }

        return this
    }

    load(values: T[]): this {
        this.#values =
            values.length < this.#size
                ? values
                : values.slice(values.length - this.#size)
        this.#head = this.#values.length % this.#size
        this.#length = this.#values.length

        return this
    }
}

export const computeLineSegments = (
    series: Series,
): Array<[number, number]> => {
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
        rects.set(x1, Date.now())
    }

    return [...rects.entries()]
}

export const formatHeat = (n?: number) =>
    n === 0 || !n ? "Off" : `${(n * 100).toFixed(0)}%`

export const getHistory = (
    client: MqttClient,
    {
        from,
        to,
        limit,
        bucketSize,
    }: { from: number; to: number; limit?: number; bucketSize?: number },
): Promise<Record<string, Series>> =>
    new Promise<Measurement[]>((resolve) => {
        const id = String(Math.round(Math.random() * 1_000_000))

        const callback = (topic: string, payload: Buffer) => {
            if (topic === `gesha/temperature/history/${id}`) {
                client.off("message", callback)

                resolve(JSON.parse(payload.toString()))
            }
        }

        client.on("message", callback)

        client.publish(
            "gesha/temperature/history/command",
            JSON.stringify({ id, from, to, limit, bucketSize }),
            { retain: false, qos: 2 },
        )
    }).then((measurements) => {
        const allSeries = measurements
            .sort((a, b) => a.time - b.time)
            .map(
                (measurement) =>
                    [
                        {
                            x: measurement.time,
                            y: measurement.boilerTempC,
                        },
                        {
                            x: measurement.time,
                            y: measurement.groupheadTempC,
                        },
                        {
                            x: measurement.time,
                            y: measurement.thermofilterTempC ?? 0,
                        },
                        {
                            x: measurement.time,
                            y: measurement.heatLevel,
                        },
                    ] as const,
            )

        const boilerTemp = allSeries.map(([v]) => v)
        const groupheadTemp = allSeries.map(([, v]) => v)
        const thermofilterTemp = allSeries.map(([, , v]) => v)
        const boilerLevel = allSeries.map(([, , , v]) => v)

        return { boilerTemp, groupheadTemp, thermofilterTemp, boilerLevel }
    })
