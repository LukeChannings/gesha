import { connect } from "mqtt"
import { TypedEventEmitter } from "mqtt/lib/TypedEmitter"
import { Accessor, createSignal, onCleanup } from "solid-js"
import { last } from "./util"

const {
    GESHA_MQTT_HOST,
    GESHA_MQTT_USER,
    GESHA_MQTT_PASS,
    GESHA_MQTT_WS_PORT,
} = import.meta.env

type GeshaClientEvents = {
    [key: `temperature/history/${string}`]: (
        measurements: Measurement[],
    ) => void
} & {} & {
    "status": (status: "connected" | "disconnected") => void
    "temperature/boiler": (valueChange: ValueChange[]) => void
    "temperature/boiler/history": (valueChange: ValueChange[]) => void
    "temperature/grouphead": (valueChange: ValueChange) => void
    "temperature/grouphead/history": (valueChange: ValueChange[]) => void
    "temperature/thermofilter": (valueChange: ValueChange) => void
    "temperature/thermofilter/history": (valueChange: ValueChange[]) => void
    "temperature/target": (temperatureTarget: number) => void
    boiler_level: (value: ValueChange) => void
    "boiler_level/history": (value: ValueChange[]) => void
    control_method: (controlMethod: ControlMethod) => void
    mode: (mode: Mode) => void
} & {
    [key: `shot/history/${string}`]: (shots: Shot[]) => void
} & {
    [key: `config/${string}`]: (value: string | number) => void
}

let singletonInstance: GeshaClient;

export class GeshaClient extends TypedEventEmitter<GeshaClientEvents> {
    #mqttClient = connect(
        `ws://${GESHA_MQTT_USER}:${GESHA_MQTT_PASS}@${GESHA_MQTT_HOST}:${GESHA_MQTT_WS_PORT}`,
    )

    constructor() {
        // There should only ever be one instance of this class
        if (singletonInstance) {
            return singletonInstance
        }

        super()

        this.#mqttClient.on("message", this.#handleEvent)

        this.#mqttClient.on("connect", () => {
            this.emit("status", "connected")
        })

        this.#mqttClient.on("disconnect", () => {
            this.emit("status", "disconnected")
        })

        singletonInstance = this
    }

    destroy = () => {
        this.#mqttClient.off("message", this.#handleEvent)
    }

    #handleEvent = (topic: string, payload: Buffer) => {
        topic = topic.replace("gesha/", "")

        const matchTopic = topic
            .replace(/(temperature\/)(?!target\b)(?!history\/).*/, "$1*")
            .replace(/(temperature\/history\/)(.+)/, "$1*")
            .replace(/(shot\/history\/)(.+)/, "$1*")
            .replace(/(config\/)(.+)/, "$1*")

        switch (matchTopic) {
            case "control_method":
            case "temperature/history/*":
            case "shot/history/*":
            case "config/*":
            case "temperature/target":
            case "mode":
            case "temperature/*":
            case "boiler_level": {
                let data = payload.toString()

                try {
                    data = JSON.parse(data)
                } catch (err) {
                    // Ignore
                }

                this.emit(
                    topic as keyof GeshaClientEvents,
                    data,
                )

                break
            }
        }
    }

    on = <TEvent extends keyof GeshaClientEvents>(
        event: TEvent,
        callback: GeshaClientEvents[TEvent],
    ): this => {

        if (event === "status") {
            this.emit("status", this.#mqttClient.connected ? "connected" : "disconnected")
        } else {
            this.#mqttClient.subscribe(`gesha/${event}`)
        }

        return super.on(event, callback)
    }

    off = <TEvent extends keyof GeshaClientEvents>(
        event: TEvent,
        callback: GeshaClientEvents[TEvent],
    ): this => {
        this.#mqttClient.unsubscribe(`gesha/${event}`)
        return super.off(event, callback)
    }

    once = <TEvent extends keyof GeshaClientEvents>(
        event: TEvent,
        callback: GeshaClientEvents[TEvent],
    ): this => {
        this.#mqttClient.subscribe(`gesha/${event}`)

        return super.once(event, (value: any) => {
            this.#mqttClient.unsubscribe(`gesha/${event}`)
            ;(callback as any)(value)
        })
    }

    createSignal = <
        Topic extends keyof GeshaClientEvents,
        T extends Parameters<GeshaClientEvents[Topic]>[0],
    >(
        topic: Topic,
    ): Accessor<T | undefined> => {
        const [accessor, setter] = createSignal<T>()

        const handler = (data: any): void => {
            setter(data)
        }

        this.on(topic, handler)

        onCleanup(() => this.off(topic, handler))

        return accessor
    }

    createSignalWithDefault = <
        Topic extends keyof GeshaClientEvents,
        T extends Parameters<GeshaClientEvents[Topic]>[0],
    >(
        topic: Topic,
        defaultValue: T,
    ): Accessor<T> => {
        const [accessor, setter] = createSignal<T>(defaultValue)

        const handler = (data: any): void => {
            setter(data)
        }

        this.on(topic, handler)

        onCleanup(() => this.off(topic, handler))

        return accessor
    }

    createValueChangeListSignal = <
        Topic extends Extract<
            keyof GeshaClientEvents,
            | "temperature/boiler"
            | "temperature/grouphead"
            | "temperature/thermofilter"
            | "boiler_level"
        >,
    >(
        topic: Topic,
        historyTopic?: Extract<keyof GeshaClientEvents, `${string}/history`>,
    ): Accessor<ValueChange[]> => {
        const [get, set] = createSignal<ValueChange[]>([])

        const handler = (data: ValueChange) => {
            set((current) => current.concat(data))
        }

        this.on(topic, handler as GeshaClientEvents[Topic])

        onCleanup(() => this.off(topic, handler as GeshaClientEvents[Topic]))

        if (historyTopic) {
            const historyHandler = (data: ValueChange[]) => {
                const values: Record<number, number> = {}

                for (const { timestamp, value } of data) {
                    values[timestamp] = value
                }

                for (const { timestamp, value } of get()) {
                    values[timestamp] = value
                }

                set(
                    Object.entries(values)
                        .sort(([a], [b]) => Number(a) - Number(b))
                        .map(([timestamp, value]) => ({
                            timestamp: Number(timestamp),
                            value,
                        })),
                )
            }

            this.on(historyTopic, historyHandler)

            onCleanup(() =>
                this.off(historyTopic, historyHandler),
            )
        }

        return get
    }

    getMeasurementHistory = ({
        from,
        to,
        limit,
        bucketSize,
        timeoutMs = 60_000,
    }: {
        from: number
        to: number
        limit?: number
        bucketSize?: number
        timeoutMs?: number
    }) => {
        return new Promise<Measurement[]>((resolve, reject) => {
            const id = Date.now()

            const timeout = setTimeout(() => {
                reject(
                    new Error(
                        "A history request took longer than the timeout limit",
                    ),
                )
            }, timeoutMs)

            this.once(`temperature/history/${id}`, (measurements) => {
                clearTimeout(timeout)
                resolve(measurements)
            })

            this.#mqttClient.publish(
                `gesha/temperature/history/command`,
                JSON.stringify({ id: String(id), from, to, limit, bucketSize }),
            )
        })
    }

    populateMeasurementHistory = async ({
        from,
        to,
        limit,
        bucketSize,
        timeoutMs = 60_000,
    }: {
        from: number
        to: number
        limit?: number
        bucketSize?: number
        timeoutMs?: number
    }) => {
        const measurements = await this.getMeasurementHistory({
            from,
            to,
            limit,
            bucketSize,
            timeoutMs,
        })

        const [
            boilerMeasurements,
            groupheadMeasurements,
            thermofilterMeasurements,
            boilerLevelMeasurements,
        ] = measurements.reduce(
            (acc, measurement) => {
                if (last(acc[0])?.value !== measurement.boilerTempC) {
                    acc[0].push({
                        timestamp: measurement.time,
                        value: measurement.boilerTempC,
                    })
                }

                if (last(acc[1])?.value !== measurement.groupheadTempC) {
                    acc[1].push({
                        timestamp: measurement.time,
                        value: measurement.groupheadTempC,
                    })
                }

                if (
                    measurement.thermofilterTempC !== undefined &&
                    last(acc[2])?.value !== measurement.thermofilterTempC
                ) {
                    acc[2].push({
                        timestamp: measurement.time,
                        value: measurement.thermofilterTempC,
                    })
                }

                if (last(acc[3])?.value !== measurement.heatLevel) {
                    acc[3].push({
                        timestamp: measurement.time,
                        value: measurement.heatLevel,
                    })
                }

                return acc
            },
            [[], [], [], []] as ValueChange[][],
        )

        if (boilerMeasurements.length > 0) {
            this.emit("temperature/boiler/history", boilerMeasurements)
        }

        if (groupheadMeasurements.length > 0) {
            this.emit("temperature/grouphead/history", groupheadMeasurements)
        }

        if (thermofilterMeasurements.length > 0) {
            this.emit(
                "temperature/thermofilter/history",
                thermofilterMeasurements,
            )
        }

        if (boilerLevelMeasurements.length > 0) {
            this.emit("boiler_level/history", boilerLevelMeasurements)
        }
    }

    getShotHistory = async ({
        from,
        to,
        limit,
        bucketSize,
        timeoutMs = 60_000,
    }: {
        from: number
        to: number
        limit?: number
        bucketSize?: number
        timeoutMs?: number
    }) => {
        return new Promise<Shot[]>((resolve, reject) => {
            const id = String(Date.now())

            const timeout = setTimeout(() => {
                reject(new Error("Shot history request timed out"))
            }, timeoutMs)

            this.once(`shot/history/${id}`, (shots) => {
                clearTimeout(timeout)
                resolve(shots)
            })
            this.#mqttClient.publish(
                "gesha/shot/history/command",
                JSON.stringify({ id, from, to, limit, bucketSize }),
                {
                    retain: false,
                    qos: 2,
                },
            )
        })
    }

    setMode = async (mode: Mode) => {
        await this.#mqttClient.publishAsync(`gesha/mode/set`, mode, {
            retain: false,
            qos: 2,
        })
    }

    setControlMethod = async (controlMethod: ControlMethod) => {
        await this.#mqttClient.publishAsync(
            `gesha/control_method/set`,
            controlMethod,
            { retain: false, qos: 2 },
        )
    }

    setTargetTemperature = async (targetTemperature: number) => {
        await this.#mqttClient.publishAsync(
            `gesha/temperature/target/set`,
            targetTemperature.toString(),
            { retain: false, qos: 2 },
        )
    }

    setConfig = async (key: string, value: string) => {
        await this.#mqttClient.publishAsync(
            `gesha/config/set`,
            JSON.stringify({
                key,
                value,
            }),
            {
                retain: false,
                qos: 2,
            },
        )
    }

    setBoilerLevel = async (boilerLevel: number) => {
        await this.#mqttClient.publishAsync(
            `gesha/boiler_level/set`,
            boilerLevel.toString(),
            { retain: false, qos: 2 },
        )
    }
}

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

export type ValueChange = {
    timestamp: number
    value: number
}

export function assertValueChange(
    value: unknown,
): asserts value is ValueChange {
    if (typeof value !== "object" || value === null) {
        throw new Error(`Value is not an object: ${JSON.stringify(value)}`)
    }

    if (
        !("value" in value && typeof value.value === "number") ||
        !("timestamp" in value && typeof value.timestamp === "number")
    ) {
        throw new Error(`value is not a ValueChange: ${JSON.stringify(value)}`)
    }
}

export type Mode = "offline" | "idle" | "active" | "brew" | "steam"

export type ControlMethod = "None" | "Threshold" | "PID" | "MPC"

export type Shot = {
    startTime: number
    endTime: number
    totalTime: number
    brewTempAverageC: number
    groupheadTempAvgC: number
}
