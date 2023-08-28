import { Accessor } from "solid-js"
import { ValueChange } from "./geshaClient"

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
export type Series = ValueChange[]

export const computeLineSegments = (
    series: Series,
): Array<[number, number]> => {
    let rects: Map<number, number> = new Map()

    let x1

    for (const { timestamp: x, value: y } of series) {
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

export const last = <T>(list: T[]): T | undefined => {
    return list.length > 0 ? list[list.length - 1] : undefined
}


type FN<Arguments extends unknown[], Return extends unknown = void> = (
    ...args: Arguments
) => Return

type MaybeAccessor<T = unknown> = Accessor<T> | T

const isFunction = (value: unknown): value is (...args: unknown[]) => unknown =>
    typeof value === "function"

const unwrap = <T,>(maybeValue: MaybeAccessor<T>): T =>
    isFunction(maybeValue) ? maybeValue() : maybeValue

export const createScheduler = <T, U>({
    loop,
    callback,
    cancel,
    schedule,
}: {
    loop?: MaybeAccessor<boolean>
    callback: MaybeAccessor<FN<[U]>>
    cancel: FN<[T]>
    schedule: (callback: FN<[U]>) => T
}): (() => void) => {
    let tickId: T
    const work = (): void => {
        if (unwrap(loop)) tick()
        unwrap(callback)
    }

    const tick = (): void => {
        tickId = schedule(work)
    }

    const dispose = (): void => {
        cancel(tickId)
    }

    tick()
    return dispose
}

export const createAnimationLoop = (callback: FrameRequestCallback) =>
    createScheduler({
        callback,
        loop: true,
        cancel: cancelAnimationFrame,
        schedule: requestAnimationFrame,
    })
