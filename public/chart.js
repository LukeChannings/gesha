import { el } from './util.js'

export class Chart {
  constructor(temp$, targetTemp, range = 30) {
    this.node = el('#chart')
    this.series = []

    this.subscribe(temp$)

    this.range = range

    this.canvasWidth = 500
    this.canvasHeight = 300

    this.createPath('currentTemp')

    this.render = this.render.bind(this)

    this.setTargetTemp(targetTemp)
    this.createSeconds()

    requestAnimationFrame(this.render)
  }

  handleReceiveData({ time, temp }) {
    // Store 35 seconds worth of data, assuming it's sampled every 100ms. Could be less.
    const maxItems = (35 * 1000) / 100

    // TODO: Use an ArrayBuffer for this or something?
    this.series = [[+time, temp], ...this.series.slice(0, maxItems - 1)]
  }

  subscribe(temp$) {
    if (this.subscription) {
      this.subscription.unsubscribe()
    }

    this.subscription = temp$.subscribe(
      (...args) => this.handleReceiveData(...args),
      err => this.destroy(),
      () => this.destroy()
    )

    this.createSeconds()
  }

  createPath(name) {
    if (!this.paths) {
      this.paths = {}
    }

    const p = document.createElementNS('http://www.w3.org/2000/svg', 'path')
    p.setAttribute('class', `path-${name}`)
    p.setAttribute('stroke', 'red')
    p.setAttribute('stroke-width', '2px')

    this.node.appendChild(p)
    this.paths[name] = p
  }

  createSeconds() {
    const secondsCount = (() => {
      switch (this.range) {
        case 30:
          return 6
        case 10:
          return 10
        default:
          return this.range / 2
      }
    })()
    const secondsGroup = this.node.querySelector('#seconds')
    secondsGroup.innerHTML = ''
    this.secondsNodes = Array.from({ length: secondsCount }).map((_, i) => {
      const line = document.createElementNS(
        'http://www.w3.org/2000/svg',
        'line'
      )
      line.setAttribute('class', 'line-second')
      line.setAttribute('y1', '0')
      line.setAttribute('y2', this.canvasHeight)
      secondsGroup.appendChild(line)
      return [line, Date.now() + i * -5000]
    })
  }

  setTargetTemp(t) {
    this.targetTemp = t

    if (!this.targetTempLine) {
      this.targetTempLine = document.createElementNS(
        'http://www.w3.org/2000/svg',
        'line'
      )
      this.targetTempLine.setAttribute('class', 'path-targetTemp')
      this.targetTempLine.setAttribute('x1', '0')
      this.targetTempLine.setAttribute('x2', this.canvasWidth)
      this.node.appendChild(this.targetTempLine)
    }

    const y = this.y(t, 0, 200)

    this.targetTempLine.setAttribute('y1', y)
    this.targetTempLine.setAttribute('y2', y)
  }

  x(t, X0, X1) {
    return ((t - X0) / (X1 - X0)) * this.canvasWidth
  }

  y(v, Y0, Y1) {
    return this.canvasHeight - (v - Y0 / Y1 - Y0) * this.canvasHeight
  }

  /* s: the series, a list of tuples of the form [time, temp]
   * X0: the lower X bound, e.g. 30 seconds ago
   * X1: the current time
   * Y0: the lowest temperature to be shown
   * Y1: the highest temperature to be shown
   */
  seriesToSvgPath(s, X0, X1, Y0, Y1) {
    // compute the series bounded by the canvas dimensions
    const bs = s
      .filter(([x]) => x >= Y0)
      .map(([x, y]) => [this.x(x, X0, X1), this.y(y, Y0, Y1)])
      .map(
        ([x, y], i) =>
          `${i === 0 ? `M ${this.canvasWidth}` : `L ${Math.max(0, x)}`},${y}`
      )
      .join(' ')

    return bs
  }

  render(t) {
    const now = Date.now()
    const then = now - 1000 * this.range

    this.paths.currentTemp.setAttribute(
      'd',
      this.seriesToSvgPath(this.series, then, now, 0, 200)
    )

    this.secondsNodes.forEach(second => {
      const [node, x] = second
      const δx = this.x(x, then, now)
      if (δx < 0) {
        second[1] = Date.now() + 100
      }
      node.setAttribute('x1', δx)
      node.setAttribute('x2', δx)
    })

    this.animationId = requestAnimationFrame(this.render)
  }

  destroy() {
    if (this.animationId) {
      cancelAnimationFrame(this.animationId)
      this.animationId = null
    }
    this.subscription.unsubscribe()
    this.subscription = null
    this.series = []
  }
}
