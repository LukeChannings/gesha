import { MountableComponent } from "../util/mount"
import { Shot, historyStore } from "../store/history"
import AfixListItem from "afix-list-item"

export class HistoryScreen extends MountableComponent {
  contentEl: HTMLElement
  shots: Shot[] = historyStore.getItems().sort((a, z) => +z.date - +a.date)

  constructor(node: HTMLElement) {
    super(node)

    const contentEl = this.node.querySelector(".content")

    if (!(contentEl instanceof HTMLElement)) {
      throw new Error(`.content not found in History screen!`)
    }

    historyStore.registerListener(this.handleHistoryChange)
    this.contentEl = contentEl

    this.node.addEventListener("click", this.handleDelete)

    this.renderHistory()
  }

  private handleHistoryChange = (shots: Shot[]): void => {
    this.shots = shots.sort((a, z) => +z.date - +a.date)
    this.renderHistory()
  }

  private handleDelete = (e: Event): void => {
    if (
      e.target instanceof HTMLElement &&
      e.target.classList.contains("DeleteButton")
    ) {
      const item = e.composedPath().find(_ => _ instanceof AfixListItem)

      if (item) {
        historyStore.removeItem(
          this.shots[
            [...this.contentEl.children].indexOf((item as unknown) as Element)
          ],
        )
      }
    }
  }

  private renderHistory = (): void => {
    const { trDose, trGrind, trTemp, trDeg, trDelete } = this.node.dataset
    this.contentEl.innerHTML = `
    ${this.shots
      .map(
        shot =>
          `<afix-list-item>
            <p slot="content">${new Intl.DateTimeFormat().format(
              shot.date,
            )}<span>${new Intl.DateTimeFormat("default", {
            hour: "numeric",
            minute: "numeric",
            second: "numeric",
          }).format(shot.date)}</span></p>
            <p slot="content">${(shot.duration / 1000).toFixed(
              2,
            )}<span>Seconds</span></p>
            <p slot="content">${shot.dose}g<span>${trDose}</span></p>
            <p slot="content">${shot.grind}<span>${trGrind}</span></p>
            <p slot="content">${
              shot.desiredTemp
            }${trDeg}<span>${trTemp}</span></p>
            <button class="DeleteButton" slot="controls">
              <span>${trDelete}</span>
              <svg aria-hidden="true" class="icon" viewBox="0 0 448 512">
                <path
                  fill="currentColor"
                  d="M32 464a48 48 0 0048 48h288a48 48 0 0048-48V128H32zm272-256a16 16 0 0132 0v224a16 16 0 01-32 0zm-96 0a16 16 0 0132 0v224a16 16 0 01-32 0zm-96 0a16 16 0 0132 0v224a16 16 0 01-32 0zM432 32H312l-9.4-18.7A24 24 0 00281.1 0H166.8a23.72 23.72 0 00-21.4 13.3L136 32H16A16 16 0 000 48v32a16 16 0 0016 16h416a16 16 0 0016-16V48a16 16 0 00-16-16z"
                />
              </svg>
            </button>
          </afix-list-item>`,
      )
      .join("")}`.trim()
  }
}
