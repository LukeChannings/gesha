import { MountableComponent } from "../util/mount"

export class Nav extends MountableComponent {
  constructor(node: HTMLElement) {
    super(node)

    if (location.hash.length <= 1) {
      location.hash = "#Brew"
    }

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
