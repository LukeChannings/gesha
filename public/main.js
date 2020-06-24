import { el, on, get, post, tr, trBool, toast, makeStream } from './util.js'
import chart from './chart.js'

const tempEl = el('#temp')
const lagEl = el('#tempLag')
const targetTempEl = el('#targetTemp')
const heatingEl = el('#heating')
const pidRunningEl = el('#pidRunning')

const getPIDRunning = get('/pid/running')
const setPID = post('/pid/running')
const setTemp = post('/temp/target')
const getTemp = get('/temp/target')

let pidOutputSubscription, tempSubscription, isPIDRunning, isHeating, targetTemp
;(async () => {
  on('click', '#togglePID', togglePID)
  on('submit', '#setTargetTemp', setTargetTemp)
  on('focus', targetTempEl, () => targetTempEl.select())
  on('visibilitychange', document, () =>
    document.hidden ? suspend() : resume()
  )

  await resume(true)
})()

async function suspend() {
  if (tempSubscription) {
    tempSubscription.unsubscribe()
    tempSubscription = null
  }
  if (pidOutputSubscription) {
    pidOutputSubscription.unsubscribe()
    pidOutputSubscription = null
  }
}

async function resume(isInit) {
  if (isInit) {
    isPIDRunning = ctx.pidRunning
    isHeating = ctx.heating
  } else {
    const pid = await getPIDRunning()
    heatingEl.innerHTML = trBool(pid.heating)
    pidRunningEl.innerHTML = trBool(pid.running)

    isPIDRunning = pid.running
    isHeating = pid.heating
  }

  if (isPIDRunning) trackPIDOutput()

  const { target } = await getTemp()
  targetTempEl.value = target
  targetTemp = target

  if (!tempSubscription) {
    trackTemp()
  }

  toast('Connected')
}

async function togglePID() {
  if (isPIDRunning) {
    await stopPID()
  } else {
    await startPID()
  }
}

async function startPID() {
  const result = await setPID({ running: true })
  if (result.ok) {
    toast(tr('messageStartPidSuccess'))
    trackPIDOutput()
    pidRunningEl.innerHTML = tr('globalOn')
    isPIDRunning = true
  } else {
    toast(new Error(tr('messageStartPidFailure')))
  }
}

async function stopPID() {
  const result = await setPID({ running: false })

  toast(
    result.ok
      ? tr('messageStopPidSuccess')
      : new Error(tr('messageStopPidFailure'))
  )

  if (result.ok) {
    if (pidOutputSubscription) pidOutputSubscription.unsubscribe()

    heatingEl.innerHTML = tr('globalOff')
    pidRunningEl.innerHTML = tr('globalOff')
    isPIDRunning = false
  }
}

async function setTargetTemp(e) {
  e.preventDefault()
  targetTemp = parseFloat(targetTempEl.value)

  try {
    const result = await setTemp({ target: targetTemp })
    // TODO: Translate and interpolate this.
    toast(
      `The temperature target was set to ${targetTemp} &deg;${ctx.tempUnit}`
    )
  } catch (err) {
    toast(new Error(tr('messageTargetTempSetFailure')))
  }
}

function trackPIDOutput() {
  pidOutputSubscription = makeStream('/api/stream/pid/output').subscribe(
    result => {
      heatingEl.innerHTML = trBool(result === 1)
    }
  )
}

function trackTemp() {
  tempSubscription = makeStream(
    '/api/stream/temp/current?sampleRateMs=100'
  ).subscribe(
    ({ time, temp }) => {
      tempEl.innerHTML = temp.toFixed(2)
      lagEl.innerHTML = ((Date.now() - +time) / 1000).toFixed(2)

      chart([time, temp], targetTemp)
    },
    // TODO: Implement retry
    err => {
      tempSubscription = null
      toast(new Error('Disconnected ' + err), null)
    },
    () => {
      tempSubscription = null
      toast(new Error('Disconnected'), null)
    }
  )
}
