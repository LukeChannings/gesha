import { el } from './util.js'

export class Chart {
  constructor(temp$, targetTemp, range = 30) {
    this.node = el('#chart')
    this.targetTemp = targetTemp
    this.series = []

    this.subscribe(temp$)

    this.range = range
    this.yRange = [0, 200]
    this.getXRange = () => {
      const t = Date.now()
      return [t - 1000 * range, t]
    }

    requestAnimationFrame(this.render)
  }

  handleReceiveData = ({ time, temp }) => {
    // Store 35 seconds worth of data, assuming it's sampled every 100ms. Could be less.
    const maxItems = (35 * 1000) / 100

    // TODO: Use an ArrayBuffer for this or something?
    this.series = [[+time, temp], ...this.series.slice(0, maxItems - 1)]
  }

  handleDisconnection = err => {
    console.log('disconnected', err)
    this.destroy()
  }

  subscribe = temp$ => {
    if (this.subscription) {
      this.subscription.unsubscribe()
    }

    this.subscription = temp$.subscribe(
      this.handleReceiveData,
      this.handleDisconnection,
      this.handleDisconnection
    )
  }

  setTargetTemp = t => {
    this.targetTemp = t
  }

  // compute the value of a timestamp within the bounds
  x = (t, now) => {
    t - now
    return 500
  }

  // compute the value of a temperature within the chart bounds
  y = t => {
    const f = t / 200

    return 300 - f * 300
  }
  render = () => {
    const now = Date.now()
    const then = now - 1000 * this.range
    const s = this.series.filter(([x]) => x >= then)

    const xPos = x => ((x - then) / (now - then)) * 500

    const targetY = (this.targetTemp / 200) * 300

    this.node.innerHTML =
      s
        .map(
          ([x, y]) =>
            `<circle cx="${xPos(x)}" cy="${this.y(y)}" r="1" fill="#333" />`
        )
        .join('') +
      `<line x1="0" y1="${targetY}" x2="500" y2="${targetY}" stroke="#f00" />`

    this.animationId = requestAnimationFrame(this.render)
  }

  destroy = () => {
    if (this.animationId) {
      cancelAnimationFrame(this.animationId)
      this.animationId = null
    }
    this.subscription.unsubscribe()
    this.subscription = null
    this.series = []
  }
}
