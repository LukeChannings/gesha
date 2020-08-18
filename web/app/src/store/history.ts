import { Store } from "./util"

export interface Shot {
  date: Date
  duration: number
  averageTemp: number
  desiredTemp: number
  dose: number
  grind: number
  beanName?: string
  rating?: string
}

export const historyStore = new Store<Shot>("gesha.history", (key, value) => {
  if (key === "date" && typeof value === "string") return new Date(value)
  return value
})
