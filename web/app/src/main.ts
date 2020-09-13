import "./main.css"
import "./components/VisuallyHidden.css"
import "./components/Header.css"
import "./components/Screen.css"
import "./components/Button.css"
import "./components/SecondaryButton.css"

import "https://cdn.skypack.dev/afix-range-slider@latest"
import "https://cdn.skypack.dev/afix-dialog@latest"
import "https://cdn.skypack.dev/afix-list-item@latest"

import { BrewScreen } from "./screens/Brew"
import { HistoryScreen } from "./screens/History"
import { SettingsScreen } from "./screens/Settings"
import { TimerScreen } from "./screens/TimerScreen"
import { Nav } from "./components/Nav"
import { mount, MountableComponent } from "./util/mount"

const mountableComponents: Record<string, typeof MountableComponent> = {
  BrewScreen,
  HistoryScreen,
  SettingsScreen,
  TimerScreen,
  Nav,
}

// A JS class is associated with a DOM element with the data-mount attribute
// Here, we query all elements with a data-mount specified and attempt to mount
// one of the mountableComponents above.
// See MountableComponent and util/mount.ts for more info.
document.querySelectorAll("[data-mount]").forEach(node => {
  if (node instanceof HTMLElement) {
    mount(node, mountableComponents)
  }
})


// The defau;t focus ring is annoying to some users, but important for accessibility.
// This code (in concert with some CSS) turns on the focus ring for keyboard users,
// but they're disabled by default.
document.addEventListener('keyup', e => {
  if (e.key === 'Tab') {
    document.documentElement.classList.add("focus-ring-enabled")
  }
})