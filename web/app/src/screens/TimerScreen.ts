import { MountableComponent } from "../util/mount"
import { DialogElement } from "afix-dialog"
import { historyStore, Shot } from "../store/history"
import { assert } from "../util/assert"

export class TimerScreen extends MountableComponent {
  dialog: DialogElement
  doneButton: HTMLButtonElement
  cancelButton: HTMLButtonElement
  readouts: HTMLElement[]
  timeReadoutValue: HTMLElement
  openTime: number | undefined
  pendingShot?: Partial<Shot>

  constructor(node: HTMLElement) {
    super(node)

    const dialog = this.node.parentElement
    const doneButton = node.querySelector(".done")
    const cancelButton = node.querySelector(".cancel")

    this.readouts = [
      ...(node.querySelectorAll(".TimerReadout") as NodeListOf<HTMLElement>),
    ]

    const timeReadoutValue = this.readouts
      .find(node => node.classList.contains("time"))
      ?.querySelector(".TimerReadoutValue")

    assert(dialog instanceof DialogElement)
    assert(doneButton instanceof HTMLButtonElement)
    assert(cancelButton instanceof HTMLButtonElement)
    assert(timeReadoutValue instanceof HTMLElement)

    this.dialog = dialog
    this.doneButton = doneButton
    this.cancelButton = cancelButton
    this.timeReadoutValue = timeReadoutValue

    cancelButton.addEventListener("click", this.handleCancel)
    doneButton.addEventListener("click", this.handleDone)
    dialog.addEventListener("show", () => {
      this.openTime = Date.now()
      requestAnimationFrame(this.renderTimer)
    })
    document.addEventListener("keyup", e => {
      if (e.key === "Escape" && this.dialog.open) {
        this.handleCancel()
      }
    })
  }

  show(shot: Partial<Shot>): void {
    this.pendingShot = shot
    this.dialog.show()
    this.node.focus({ preventScroll: true })
  }

  private handleDone = (): void => {
    assert(typeof this.openTime === "number")

    const shot = {
      date: new Date(this.openTime),
      duration: Date.now() - this.openTime,
      averageTemp: 0,
      desiredTemp: 0,
      grind: 0,
      dose: 0,
      ...this.pendingShot,
    }

    historyStore.addItem(shot)

    this.dialog.close()
    this.openTime = undefined

    location.hash = "#History"
  }

  private handleCancel = (): void => {
    this.dialog.close()
    this.openTime = undefined
  }

  private renderTimer = (): void => {
    assert(typeof this.openTime === "number")

    this.timeReadoutValue.innerHTML = ((Date.now() - this.openTime) / 1_000)
      .toFixed(1)
      .padStart(4, "0")

    requestAnimationFrame(this.renderTimer)
  }
}
