import { Observable } from 'https://cdn.pika.dev/zen-observable-ts@^0.8.21'

// used for debugging
const API_HOST = __DEBUG__ ? 'http://192.168.20.24:3000' : ''

// api
export const makeStream = path => {
  let ess = {}
  return new Observable(observer => {
    const es = ess[path] ? ess[path] : new EventSource(API_HOST + path)
    ess[path] = es

    es.addEventListener('message', e => observer.next(JSON.parse(e.data)))
    es.addEventListener('error', () => {
      console.log('ERROR in stream ' + path)
      ess[path] = null
      observer.complete()
      toast(new Error('Connection lost'), null)
    })

    return () => {
      ess[path] = null
      es.close()
    }
  })
}

const apiCall = (path, method = 'GET', body = {}) => async (
  bodyOverride = {}
) => {
  try {
    const res = await fetch(API_HOST + '/api' + path, {
      method,
      ...(method === 'POST'
        ? {
            headers: { 'content-type': 'application/json' },
            body: JSON.stringify({ ...body, ...bodyOverride }),
          }
        : {}),
    })

    if (res.ok) {
      return await res.json()
    } else {
      throw new Error(`Server responded ${res.status}. ${await res.text()}`)
    }
  } catch (err) {
    throw err
  }
}

export const get = path => apiCall(path)
export const post = (path, data) => apiCall(path, 'POST', data)

// DOM
export const el = document.querySelector.bind(document)

export const on = (event, _, handler) =>
  (typeof _ === 'string' ? el(_) : _).addEventListener(event, handler)

const toastEl = el('#toast')
export const toast = (message, displayTimeMs = 3000) => {
  toastEl.setAttribute(
    'class',
    `Toast ${message instanceof Error ? '--error' : ''}`
  )

  toastEl.innerHTML = message

  clearTimeout(toast.timeout)

  if (typeof displayTimeMs === 'number') {
    toast.timeout = setTimeout(() => {
      toastEl.innerHTML = ''
    }, displayTimeMs)
  }
}

// translations
export const tr = name => globalThis.ctx.tr[name]
export const trBool = bool => globalThis.ctx.tr[bool ? 'globalOn' : 'globalOff']
