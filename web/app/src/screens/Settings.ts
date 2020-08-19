import { MountableComponent } from "../util/mount"
import { isConfig, postConfig } from "../api/api"
import { assert } from "../util/assert"

export class SettingsScreen extends MountableComponent {
  formEl: HTMLFormElement

  constructor(node: HTMLElement) {
    super(node)

    const formEl = this.node.querySelector("form")

    if (!formEl) throw new Error("No form element found in Settings!")

    this.formEl = formEl

    node.addEventListener("submit", e => this.handleSubmit(e))
    node.addEventListener("change", e => this.handleChange(e))
  }

  handleChange(e: Event): void {
    if (
      (e.target as HTMLInputElement).name === "themeColorHue" &&
      e instanceof MessageEvent
    ) {
      document.documentElement.style.setProperty("--base-color", e.data.value)
    }
  }

  async handleSubmit(e: Event): Promise<void> {
    e.preventDefault()

    const formData = Object.fromEntries(new FormData(this.formEl));

    const config = {
      ...formData,
      pid: [formData["pid.p"], formData["pid.i"], formData["pid.d"]].map(
        Number,
      ),
      temperatureTarget: Number(formData.temperatureTarget),
      pidAutostart: formData.pidAutostart === "on",
      verbose: formData.verbose === "on",
      themeColorHue:
        String((this.formEl.querySelector("[name=themeColorHue]") as HTMLInputElement)
          ?.value) ?? "300",
    }

    assert(isConfig(config))

    postConfig(config)
  }
}
