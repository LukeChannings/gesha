const API_URI = "http://192.168.20.24/api"

export interface Temperature {
  temp: number
  time: string
}

export interface Config {
  port: string
  boilerPin: string
  temperatureSampleRate: string
  temperatureUnit: "C" | "F"
  temperatureTarget: 110
  pid: [number, number, number]
  pidFrequency: string
  pidAutostart: boolean
  themeColor: {
    hex: string
    hue: string
  }
}

export interface TemperatureEvent extends MessageEvent {
  detail: Temperature
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

export const getConfig = async (apiUrl = API_URI): Promise<Config> => {
  const res = await fetch(`${apiUrl}/config`)

  if (res.ok) {
    const config = await res.json()

    if (isConfig(config)) {
      return config
    } else {
      throw new ParseResultError(
        `The response received when getting the current temperature was not what was expected. Got: ${JSON.stringify(
          config,
        )}`,
      )
    }
  }

  throw new RequestNotOkError(
    `Recieved a ${res.status} when getting the config`,
  )
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
  return hasRequiredKeys(data, ["temp", "time"])
}

export function isConfig(data: unknown): data is Config {
  return hasRequiredKeys(data, [
    "port",
    "boilerPin",
    "temperatureSampleRate",
    "temperatureUnit",
    "temperatureTarget",
    "pid",
    "pidFrequency",
    "pidAutostart",
  ])
}

const hasRequiredKeys = (data: unknown, keys: string[]) =>
  typeof data === "object" && data !== null && keys.every(key => key in data)
