import { MountableComponent } from "../util/mount"
import "./Nav.css"

export class Nav extends MountableComponent {
  constructor(node: HTMLElement) {
    super(node)

    const validHashes = [...node.querySelectorAll("a")]
      .map(a => a.getAttribute("href"))
      .concat("#Nav")

    const validateHash = (e?: HashChangeEvent) => {
      if (!validHashes.includes(location.hash)) {
        const { hash } = new URL(e?.oldURL ?? "#Brew")
        location.hash = hash
      }
    }

    window.addEventListener("hashchange", validateHash)
    validateHash()

    /**
     * Resizing the browser stretches screens so the old scrollLeft position is out of date,
     * causing the current screen to be scrolled off screen. This code corrects that when resizing.
     */
    window.addEventListener("resize", () => {
      const screen = document.querySelector(location.hash)
      if (screen instanceof HTMLElement) {
        this.node.scrollLeft = screen.offsetLeft
      }
    })
  }
}
