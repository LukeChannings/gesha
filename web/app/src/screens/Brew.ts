import { AfixRangeSlider } from "https://unpkg.com/afix-range-slider@latest"
import { getStateStream, StateEvent } from "../api/api"
import { MountableComponent } from "../util/mount"
import { getInstances } from "../util/mount"
import { TimerScreen } from "./TimerScreen"
import { assert } from "../util/assert"
import "./Brew.css"

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
    const es = getStateStream()

    es.addEventListener("message", e => {
      const { detail } = e as StateEvent

      assert(rangeEl instanceof AfixRangeSlider)

      rangeEl.setShadowValue(detail.currentTemp.tempC)

      if (detail.isHeating) {
        this.shotVariables.temperature.classList.add("is-heating")
      } else {
        this.shotVariables.temperature.classList.remove("is-heating")
      }
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

  updateTemperature(temp: number, unit: string): void {
    const tempSlider = this.shotVariables.temperature.querySelector(
      "afix-range-slider",
    )
    const tempInput = this.shotVariables.temperature.querySelector("input")
    const label = this.shotVariables.temperature.querySelector(
      ".ShotVariables_Variable_Label_Unit",
    )
    assert(tempSlider instanceof AfixRangeSlider)
    assert(tempInput instanceof HTMLInputElement)
    tempSlider.setValue(temp)
    assert(label instanceof HTMLElement)
    label.innerHTML = `(° ${unit})`
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
