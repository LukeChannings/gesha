import { Observable } from "https://cdn.pika.dev/zen-observable-ts@^0.8.21";
import chart from "./chart.js";

const API_HOST = 'http://192.168.20.24:3000'

const { tr } = globalThis

const tempEl = el("#temp");
const lagEl = el("#tempLag");
const targetTempEl = el("#targetTemp");
const heatingEl = el("#heating");
const toastEl = el("#toast");
const pidRunningEl = el("#pidRunning");

const getConfig = get('/config')
const getPIDRunning = get("/pid/running");
const setPID = post("/pid/running");
const setTemp = post("/temp/target");
const getTemp = get("/temp/target");

let pidOutputSubscription, tempSubscription, isPIDRunning, isHeating, config;

(async () => {

  onClick("#togglePID", togglePID);
  onClick("#setTargetTemp", setTargetTemp);

  targetTempEl.addEventListener("focus", () => targetTempEl.select());

  // window.addEventListener('blur', () => suspend())
  // window.addEventListener('focus', () => resume())

  await resume()
})();

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

async function resume() {
  config = await getConfig()
  tempEl.classList.add('--deg-' + config.temperatureUnit.toLowerCase())

  const pid = await getPIDRunning();
  if (pid.running) trackPIDOutput();
  heatingEl.innerHTML = onOffText(pid.heating)
  pidRunningEl.innerHTML = onOffText(pid.running)

  isPIDRunning = pid.running
  isHeating = pid.heating

  const { target } = await getTemp();
  targetTempEl.value = target;

  trackTemp();
}

async function togglePID() {
  if (isPIDRunning) {
    await stopPID();
  } else {
    await startPID();
  }
}

async function startPID() {
  const result = await setPID({ running: true });
  toast(result.ok ? "Started PID" : new Error("Could not start the PID"));
  if (result.ok) {
    trackPIDOutput();
    pidRunningEl.innerHTML = "On";
    isPIDRunning = true;
  }
}

async function stopPID() {
  const result = await setPID({ running: false });

  toast(result.ok ? "Stopped PID" : new Error("Could not stop the PID"));

  if (result.ok) {
    if (pidOutputSubscription) pidOutputSubscription.unsubscribe();

    heatingEl.innerHTML = "Off";
    pidRunningEl.innerHTML = "Off";
    isPIDRunning = false;
  }
}

async function setTargetTemp() {
  const target = parseFloat(targetTempEl.value);

  try {
    const result = await setTemp({ target });
    toast(`The temperature target was set to ${target} &deg;${config.temperatureUnit}`);
  } catch (err) {
    toast(new Error(`Something went wrong setting new target`));
  }
}

function trackPIDOutput() {
  pidOutputSubscription = makeStream("/api/stream/pid/output").subscribe(
    (result) => {
      heatingEl.innerHTML = onOffText(result === 1)
    }
  );
}

function trackTemp() {
  const temp$ = makeStream("/api/stream/temp/current?sampleRateMs=100")
  tempSubscription = temp$.subscribe(
    ({ time, temp }) => {
      tempEl.innerHTML = temp.toFixed(2);
      lagEl.innerHTML = ((Date.now() - +time) / 1000).toFixed(2);

      chart([time, temp])
    }
  );
}

// utils

function el(...args) {
  return document.querySelector(...args);
}

function makeStream(path) {
  return new Observable((observer) => {
    const es = new EventSource(API_HOST + path);

    es.addEventListener("message", (e) => observer.next(JSON.parse(e.data)));
    es.addEventListener("error", () => {
      console.log("ERROR in stream " + path);
      observer.complete();
    });

    return () => es.close();
  });
}

function apiCall(path, method = "GET", body = {}) {
  return async (bodyOverride = {}) => {
    const res = await fetch(API_HOST + "/api" + path, {
      method,
      ...(method === "POST"
        ? {
            headers: { "content-type": "application/json" },
            body: JSON.stringify({ ...body, ...bodyOverride }),
          }
        : {}),
    });

    if (res.ok) {
      return await res.json();
    } else {
      throw new Error(
        `Server responded ${res.status}. ${await res.text()}`
      );
    }
  };
}

function get(path) {
  return apiCall(path);
}
function post(path, data) {
  return apiCall(path, "POST", data);
}

function onClick(node, handler) {
  (typeof node === "string" ? el(node) : node).addEventListener(
    "click",
    handler
  );
}

function toast(message, displayTimeMs = 3000) {
  toastEl.setAttribute('class', `Toast ${message instanceof Error ? '--error' : ''}`)

  toastEl.innerHTML = message;

  clearTimeout(toast.timeout);
  toast.timeout = setTimeout(() => { toastEl.innerHTML = "" }, displayTimeMs);
}

function onOffText(bool) {
  return bool ? tr['global.on'] : tr['global.off']
}