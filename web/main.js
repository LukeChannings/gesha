import { Observable } from 'https://cdn.pika.dev/zen-observable-ts@^0.8.21';

const tempEl = el('#temp')
const lagEl = el('#lag')
const targetTempEl = el('#targetTemp')
const heatingEl = el('#heating')
const toastEl = el('#toast')
const pidRunningEl = el('#pidRunning')

const getPIDRunning = get('/pid/running')
const setPID = post('/pid/running')
const setTemp = post('/temp/target')
const getTemp = get('/temp/target')

let pidOutputSubscription, tempSubscription

(async () => {

  const { target } = await getTemp()
  const pid = await getPIDRunning()

  onClick('#startPID', startPID)
  onClick('#stopPID', stopPID)
  onClick("#setTargetTemp", setTargetTemp)

  trackTemp()

  if (pid.running) trackPIDOutput()
  heatingEl.innerHTML = pid.heating ? 'YES' : 'NO'
  pidRunningEl.innerHTML = pid.running ? 'YES' : 'NO'

  targetTempEl.value = target
})()

async function startPID() {
  const result = await setPID({ running: true })
  toast(result.ok ? "Started PID" : new Error("Could not start the PID"))
  if (result.ok) {
    trackPIDOutput()
    pidRunningEl.innerHTML = 'YES'
  }
}

async function stopPID() {
  const result = await setPID({ running: false })

  toast(result.ok ? "Stopped PID" : new Error("Could not stop the PID"))

  if (result.ok) {
    if (pidOutputSubscription) pidOutputSubscription.unsubscribe()

    heatingEl.innerHTML = "NO"
    pidRunningEl.innerHTML = 'NO'
  }
}

async function setTargetTemp() {
  const target = parseFloat(targetTempEl.value)
  const result = await setTemp({ target })

  toast(result.ok
    ? `Set temperature to <strong>${target}</strong>`
    : new Error("Could not set temperature"))
}

function trackPIDOutput() {
  pidOutputSubscription = makeStream("/api/stream/pid/output")
    .subscribe(result => {
      heatingEl.innerHTML = result === 1 ? 'YES' : 'NO'
    })
}

function trackTemp() {
  tempSubscription = makeStream("/api/stream/temp/current")
    .subscribe(({time, temp}) => {
      tempEl.innerHTML = temp.toFixed(2)
      lagEl.innerHTML = Date.now() - (+time)
    })
}

// utils

function el(...args) {
  return document.querySelector(...args)
}

function makeStream(path) {
  return new Observable(observer => {
    const es = new EventSource(path)

    es.addEventListener("message", e => observer.next(JSON.parse(e.data)));
    es.addEventListener("error", () => {
      console.log("ERROR in stream " + path)
      observer.complete()
    })

    return () => es.close()
  })
}

function apiCall(path, method = 'GET', body = {}) {
  return async (bodyOverride = {}) => {
    const res = await fetch('/api' + path, { method, ...(method === 'POST' ? {
      headers: { "content-type": "application/json" },
      body: JSON.stringify({...body, ...bodyOverride})
    } : {}) })

    if (res.ok) {
      return await res.json()
    } else {
      throw new APIFetchError(`Server responded ${res.status}. ${await res.text()}`)
    }
  }
}

function get(path) { return apiCall(path) }
function post(path, data) { return apiCall(path, 'POST', data) }

function onClick(node, handler) {
  (typeof node === 'string' ? el(node) : node).addEventListener('click', handler)
}

class APIFetchError extends Error {}

function toast(message, displayTimeMs = 3000) {
  clearTimeout(toast.timeout)

  if (message instanceof Error) {
    toastEl.classList.add('--error')
  } else {
    toastEl.classList.remove('--error')
  }

  toastEl.innerHTML = `<p>${message}</p>`

  toast.timeout = setTimeout(() => { toastEl.innerHTML = '' }, displayTimeMs)
}
