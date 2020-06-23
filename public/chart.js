let chartEl, series = []

// debugging
globalThis.series = series

const x = (d, ref, range) => {
  const msSinceRef = d - ref
  const f = msSinceRef / range

  return 500 - (f * 500)
}

const y = (d) => {
  const f = d / 200
  return 300 - (f * 300)
}

const draw = () => {
  if (series.length > 2) {
    const sortedSeries = series.slice(0, 70)
    const now = sortedSeries[sortedSeries.length - 1][0]
    const then = sortedSeries[0][0]
    const xRange = then - now

    chartEl.innerHTML = sortedSeries
      .map(xy => {
        return [x(xy[0], now, xRange), y(xy[1])]
      })
      .map(([x, y]) => {
        return `<circle cx="${x}" cy="${y}" r="2" fill="#000" />`
      })
      .join('\n') +
      `<text x="430" y="300" style="font-size: 12px">${(xRange / 1000).toFixed(2)} seconds</text>`
  }

  requestAnimationFrame(draw)
}

let loopRunning

export default d => {
  series.unshift(d)
  if (!loopRunning) {
    chartEl = document.querySelector('#chart')
    window.requestAnimationFrame(draw)
    loopRunning = true
  }
}
