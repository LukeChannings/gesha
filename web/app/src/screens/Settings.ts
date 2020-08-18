import { hexToRgb, getHueFromRgb } from "../util/color"
import { MountableComponent } from "../util/mount"
import { getConfig, Config, isConfig, postConfig } from "../api/api"

export class SettingsScreen extends MountableComponent {
  config: Partial<Config> & Record<string, unknown> = {}
  formEl: HTMLFormElement

  constructor(node: HTMLElement) {
    super(node)

    const formEl = this.node.querySelector("form")

    if (!formEl) throw new Error("No form element found in Settings!")

    this.formEl = formEl

    this.loadConfig()

    node.addEventListener("submit", e => this.handleSubmit(e))
    node.addEventListener("change", e => this.handleChange(e))
  }

  handleChange(e: Event): void {
    if (e.target instanceof HTMLInputElement) {
      switch (e.target.name) {
        case "themeColor": {
          const hue = String(getHueFromRgb(...hexToRgb(e.target.value)))
          this.config.themeColor = { hue, hex: e.target.value }
          document.documentElement.style.setProperty("--base-color", hue)
          break
        }
        default: {
          if (e.target.name in this.config) {
            this.config[e.target.name] = e.target.value
          }
        }
      }
    }
  }

  handleSubmit(e: Event): void {
    e.preventDefault()

    if (isConfig(this.config)) {
      postConfig(this.config)
    }
  }

  syncConfig(): void {
    for (const [key, value] of Object.entries(this.config)) {
      const input = this.formEl.elements.namedItem(key)
      if (input instanceof RadioNodeList) {
        input.forEach((item, index) => {
          if (item instanceof HTMLInputElement) {
            if (item.type === "radio") {
              item.checked = item.value === value
            } else if (item.type === "text" && Array.isArray(value)) {
              item.value = String(value[index])
            }
          }
        })
      } else if (input instanceof HTMLInputElement) {
        input.value = String(
          key === "themeColor" ? this.config.themeColor?.hex : value,
        )
      }
    }
  }

  async loadConfig(): Promise<void> {
    this.config = {
      ...(await getConfig()),
      ...this.config,
    }

    this.syncConfig()
  }
}
