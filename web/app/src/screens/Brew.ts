import { getTempStream, TemperatureEvent } from "../api/api"
import { MountableComponent } from "../util/mount"
import { getInstances } from "../util/mount"
import { TimerScreen } from "./TimerScreen"

export class BrewScreen extends MountableComponent {
  shotVariables: Record<string, HTMLElement>

  constructor(node: HTMLElement) {
    super(node)

    this.shotVariables = [
      ...node.querySelectorAll("[data-shot-variable]"),
    ].reduce((acc, node) => {
      if (node instanceof HTMLElement && node.dataset.shotVariable) {
        return { ...acc, [node.dataset.shotVariable]: node }
      }
      return acc
    }, {} as Record<string, HTMLElement>)

    Object.values(this.shotVariables).forEach(this.bindValueUpdates)

    if (this.shotVariables.temperature) {
      const tempRangeEl = this.shotVariables.temperature.querySelector(
        "afix-range-slider",
      )
      if (tempRangeEl instanceof HTMLElement) this.bindTemperature(tempRangeEl)
    }

    this.node.addEventListener("submit", e => this.handleSubmit(e))
  }

  handleSubmit(e: Event): void {
    e.preventDefault()

    this.showTimerModal()
  }

  bindValueUpdates(variableEl: HTMLElement): void {
    const rangeEl = variableEl.querySelector("afix-range-slider")
    const inputEl = variableEl.querySelector("input")

    if (rangeEl instanceof HTMLElement && inputEl instanceof HTMLElement) {
      rangeEl.addEventListener("change", e => {
        if (isRangeSliderEvent(e) && inputEl) {
          if (inputEl.step !== "1") {
            inputEl.value = String(e.data.value.toFixed(1))
          } else {
            inputEl.value = String(e.data.value)
          }
        }
      })

      inputEl.addEventListener("change", () =>
        rangeEl.setAttribute("value", inputEl.value),
      )

      inputEl.addEventListener("focus", () => inputEl.select())
    }
  }

  bindTemperature(rangeEl: HTMLElement): void {
    const es = getTempStream()

    es.addEventListener("message", e => {
      const { detail } = e as TemperatureEvent

      rangeEl.setAttribute("shadow-value", String(Math.round(detail.temp)))
    })
  }

  showTimerModal(): void {
    const [timerScreen] = getInstances(TimerScreen)
    if (timerScreen) {
      timerScreen.show({
        desiredTemp: Number(
          this.shotVariables.temperature.querySelector("input")?.value,
        ),
        dose: Number(this.shotVariables.dose.querySelector("input")?.value),
        grind: Number(this.shotVariables.grind.querySelector("input")?.value),
      })
    } else {
      alert(
        "There was an internal error getting an instance of the shot timer modal. Please report a bug.",
      )
    }
  }
}

type RangeSliderValue = {
  value: number
  rawValue: number
}

interface RangeSliderEvent extends MessageEvent {
  data: RangeSliderValue
}

function isRangeSliderEvent(e: Event): e is RangeSliderEvent {
  if (
    e instanceof MessageEvent &&
    typeof e.data === "object" &&
    typeof e.data.value === "number" &&
    typeof e.data.rawValue === "number"
  ) {
    return true
  }

  return false
}
