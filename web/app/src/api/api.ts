import { assert, isRecord } from "../util/assert"

declare const window: Window & typeof globalThis & { __API_BASE__: string }

const API_URI = (window.__API_BASE__ || "") + "/api"

export interface State {
  currentTemp: Temperature
  isHeating: boolean
}

export interface Temperature {
  tempC: number
  tempF: number
  time: string
}

export interface Config {
  port: string
  boilerPin: string
  temperatureSampleRate: string
  temperatureUnit: "C" | "F"
  temperatureTarget: string
  pid: [number, number, number]
  pidFrequency: string
  pidAutostart: boolean
  verbose: boolean
  themeColorHue: string
}

export interface TemperatureEvent extends MessageEvent {
  detail: Temperature
}

export interface StateEvent extends MessageEvent {
  detail: State
}

export class ParseResultError extends Error {}
export class RequestNotOkError extends Error {}

export const getTempStream = (
  apiUrl = API_URI,
  sampleRateMs = 100,
): EventSource => {
  const es = new EventSource(
    `${apiUrl}/stream/temp/current?sampleRateMs=${sampleRateMs}`,
  )

  es.addEventListener("message", e => {
    const detail = JSON.parse(e.data)

    ;(e as TemperatureEvent).detail = detail
  })

  return es
}

export const getStateStream = (
  apiUrl = API_URI,
  sampleRateMs = 100,
): EventSource => {
  const es = new EventSource(
    `${apiUrl}/stream/state?sampleRateMs=${sampleRateMs}`,
  )

  es.addEventListener("message", e => {
    const detail = JSON.parse(e.data)

    ;(e as StateEvent).detail = detail
  })

  return es
}

export const getTemp = async (apiUrl = API_URI): Promise<Temperature> => {
  const res = await fetch(`${apiUrl}/temp/current`)

  if (res.ok) {
    const current = await res.json()

    if (isTemperature(current)) {
      return current
    } else {
      throw new ParseResultError(
        `The response received when getting the current temperature was not what was expected. Got: ${JSON.stringify(
          current,
        )}`,
      )
    }
  }

  throw new RequestNotOkError(
    `Recieved a ${res.status} when getting the current temperature`,
  )
}

export const setTemp = async (
  target: number,
  unit: string,
  apiUrl = API_URI,
): Promise<boolean> => {
  const res = await fetch(`${apiUrl}/temp/target`, {
    method: "POST",
    body: JSON.stringify({ target, unit }),
  })

  return res.ok
}

export const postConfig = async (
  config: Config,
  apiUrl = API_URI,
): Promise<boolean> => {
  try {
    const res = await fetch(`${apiUrl}/config`, {
      method: "POST",
      headers: {
        "content-type": "application/json",
      },
      body: JSON.stringify(config),
    })
    return res.ok
  } catch (err) {
    return false
  }
}

export function isTemperature(data: unknown): data is Temperature {
  return hasRequiredKeys(data, ["tempC", "tempF", "time"])
}

export function isConfig(data: unknown): data is Config {
  assert(isRecord(data))
  assert(typeof data.port === "string")
  assert(typeof data.boilerPin === "string")
  assert(typeof data.temperatureSampleRate === "string")
  assert(data.temperatureUnit === "C" || data.temperatureUnit === "F")
  assert(typeof data.temperatureTarget === "string")
  assert(
    Array.isArray(data.pid) &&
      data.pid.length === 3 &&
      data.pid.every(v => typeof v === "number"),
  )
  assert(typeof data.pidFrequency === "string")
  assert(typeof data.pidAutostart === "boolean")
  return true
}

const hasRequiredKeys = (data: unknown, keys: string[]) =>
  typeof data === "object" && data !== null && keys.every(key => key in data)
