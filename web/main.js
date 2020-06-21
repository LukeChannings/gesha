import { Observable } from 'https://cdn.pika.dev/zen-observable-ts@^0.8.21';

const temp$ = new Observable(observer => {
  const es = new EventSource("/api/stream/temp/current")

  es.addEventListener("message", e => observer.next(JSON.parse(e.data)));
  es.addEventListener("error", () => observer.complete())

  return () => es.close()
})

const tempEl = document.querySelector('#temp')
const lagEl = document.querySelector('#lag')

temp$.subscribe(({time, temp}) => {
  tempEl.innerHTML = temp.toFixed(2)
  lagEl.innerHTML = Date.now() - time
})
